//! websocket_protocol component
//!
//! WebSocket client with frame parsing/encoding, ping/pong, compression extensions,
//! and automatic reconnection with exponential backoff.

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

use futures::{SinkExt, StreamExt};
use network_errors::NetworkError;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, watch, Mutex};
use tokio_tungstenite::tungstenite::{
    self,
    protocol::CloseFrame as TungsteniteCloseFrame,
    protocol::WebSocketConfig as TungsteniteConfig,
};
use tokio_tungstenite::{connect_async, connect_async_with_config, WebSocketStream, MaybeTlsStream};
use url::Url;

// ==================== Compression Configuration ====================

/// Compression configuration for WebSocket connections (permessage-deflate)
///
/// Controls the permessage-deflate compression extension settings as defined in RFC 7692.
///
/// # Note
///
/// Compression negotiation depends on server support. The client will request compression
/// with these parameters during the WebSocket handshake, but the server may decline or
/// negotiate different parameters.
///
/// **Important**: Actual compression requires upstream `tungstenite` library support.
/// Currently, tungstenite 0.21 does not include built-in permessage-deflate.
/// This configuration prepares the API for when compression becomes available.
///
/// # Example
///
/// ```
/// use websocket_protocol::CompressionConfig;
///
/// // Enable compression with default settings
/// let config = CompressionConfig::enabled();
/// assert!(config.is_enabled());
///
/// // Enable compression with custom window bits
/// let config = CompressionConfig::enabled()
///     .with_client_max_window_bits(12)
///     .with_server_max_window_bits(12);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompressionConfig {
    /// Enable permessage-deflate compression extension
    enabled: bool,
    /// Client maximum window bits (8-15, default: 15)
    /// Lower values use less memory but may reduce compression ratio
    client_max_window_bits: u8,
    /// Server maximum window bits (8-15, default: 15)
    server_max_window_bits: u8,
    /// Request server to not use context takeover
    /// When true, the server must reset compression context after each message
    server_no_context_takeover: bool,
    /// Client will not use context takeover
    /// When true, the client resets compression context after each message
    client_no_context_takeover: bool,
}

impl Default for CompressionConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            client_max_window_bits: 15,
            server_max_window_bits: 15,
            server_no_context_takeover: false,
            client_no_context_takeover: false,
        }
    }
}

impl CompressionConfig {
    /// Create a new compression configuration with compression enabled
    ///
    /// Uses default window bits (15) and context takeover settings.
    ///
    /// # Example
    ///
    /// ```
    /// use websocket_protocol::CompressionConfig;
    ///
    /// let config = CompressionConfig::enabled();
    /// assert!(config.is_enabled());
    /// ```
    pub fn enabled() -> Self {
        Self {
            enabled: true,
            ..Default::default()
        }
    }

    /// Create a new compression configuration with compression disabled
    ///
    /// This is the same as `Default::default()`.
    pub fn disabled() -> Self {
        Self::default()
    }

    /// Check if compression is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Get the client maximum window bits setting
    pub fn client_max_window_bits(&self) -> u8 {
        self.client_max_window_bits
    }

    /// Get the server maximum window bits setting
    pub fn server_max_window_bits(&self) -> u8 {
        self.server_max_window_bits
    }

    /// Check if server no context takeover is requested
    pub fn server_no_context_takeover(&self) -> bool {
        self.server_no_context_takeover
    }

    /// Check if client no context takeover is set
    pub fn client_no_context_takeover(&self) -> bool {
        self.client_no_context_takeover
    }

    /// Set client maximum window bits (8-15)
    ///
    /// Lower values use less memory but may reduce compression ratio.
    /// The window size is 2^bits bytes.
    ///
    /// # Panics
    ///
    /// Panics if bits is not in range 8-15.
    ///
    /// # Example
    ///
    /// ```
    /// use websocket_protocol::CompressionConfig;
    ///
    /// let config = CompressionConfig::enabled()
    ///     .with_client_max_window_bits(12); // 4KB window
    /// assert_eq!(config.client_max_window_bits(), 12);
    /// ```
    pub fn with_client_max_window_bits(mut self, bits: u8) -> Self {
        assert!(
            (8..=15).contains(&bits),
            "Window bits must be between 8 and 15, got {}",
            bits
        );
        self.client_max_window_bits = bits;
        self
    }

    /// Set server maximum window bits (8-15)
    ///
    /// Requests the server to use at most this window size.
    ///
    /// # Panics
    ///
    /// Panics if bits is not in range 8-15.
    pub fn with_server_max_window_bits(mut self, bits: u8) -> Self {
        assert!(
            (8..=15).contains(&bits),
            "Window bits must be between 8 and 15, got {}",
            bits
        );
        self.server_max_window_bits = bits;
        self
    }

    /// Set server no context takeover
    ///
    /// When true, requests the server to reset compression context after each message.
    /// This uses more bandwidth but less memory on the server.
    pub fn with_server_no_context_takeover(mut self, no_takeover: bool) -> Self {
        self.server_no_context_takeover = no_takeover;
        self
    }

    /// Set client no context takeover
    ///
    /// When true, the client resets compression context after each message.
    /// This uses more bandwidth but less memory on the client.
    pub fn with_client_no_context_takeover(mut self, no_takeover: bool) -> Self {
        self.client_no_context_takeover = no_takeover;
        self
    }

    /// Generate the Sec-WebSocket-Extensions header value for this configuration
    ///
    /// Returns the extension parameter string to be sent during the WebSocket handshake,
    /// or None if compression is disabled.
    ///
    /// # Example
    ///
    /// ```
    /// use websocket_protocol::CompressionConfig;
    ///
    /// let config = CompressionConfig::enabled()
    ///     .with_client_max_window_bits(12);
    /// let header = config.to_extension_header();
    /// assert!(header.is_some());
    /// assert!(header.unwrap().contains("permessage-deflate"));
    /// ```
    pub fn to_extension_header(&self) -> Option<String> {
        if !self.enabled {
            return None;
        }

        let mut params = vec!["permessage-deflate".to_string()];

        if self.client_max_window_bits != 15 {
            params.push(format!("client_max_window_bits={}", self.client_max_window_bits));
        }

        if self.server_max_window_bits != 15 {
            params.push(format!("server_max_window_bits={}", self.server_max_window_bits));
        }

        if self.server_no_context_takeover {
            params.push("server_no_context_takeover".to_string());
        }

        if self.client_no_context_takeover {
            params.push("client_no_context_takeover".to_string());
        }

        Some(params.join("; "))
    }

    /// Parse a Sec-WebSocket-Extensions header value to determine compression settings
    ///
    /// Returns a CompressionConfig if permessage-deflate was negotiated.
    ///
    /// # Arguments
    ///
    /// * `header` - The Sec-WebSocket-Extensions header value from the server response
    pub fn from_extension_header(header: &str) -> Option<Self> {
        if !header.contains("permessage-deflate") {
            return None;
        }

        let mut config = Self::enabled();

        for part in header.split(';').map(|s| s.trim()) {
            if part.starts_with("client_max_window_bits=") {
                if let Ok(bits) = part.trim_start_matches("client_max_window_bits=").parse::<u8>() {
                    if (8..=15).contains(&bits) {
                        config.client_max_window_bits = bits;
                    }
                }
            } else if part.starts_with("server_max_window_bits=") {
                if let Ok(bits) = part.trim_start_matches("server_max_window_bits=").parse::<u8>() {
                    if (8..=15).contains(&bits) {
                        config.server_max_window_bits = bits;
                    }
                }
            } else if part == "server_no_context_takeover" {
                config.server_no_context_takeover = true;
            } else if part == "client_no_context_takeover" {
                config.client_no_context_takeover = true;
            }
        }

        Some(config)
    }
}

// ==================== WebSocket Configuration ====================

/// WebSocket client configuration
///
/// Provides settings for WebSocket connection behavior including
/// compression, message size limits, and frame size limits.
///
/// # Example
///
/// ```
/// use websocket_protocol::{WebSocketConfig, CompressionConfig};
///
/// // Create config with compression enabled
/// let config = WebSocketConfig::with_compression();
/// assert!(config.is_compression_enabled());
///
/// // Create config with custom settings
/// let config = WebSocketConfig::new()
///     .compression(CompressionConfig::enabled().with_client_max_window_bits(12))
///     .max_message_size(Some(16 * 1024 * 1024))  // 16 MB
///     .max_frame_size(Some(4 * 1024 * 1024));    // 4 MB
/// ```
#[derive(Debug, Clone)]
pub struct WebSocketConfig {
    /// Compression configuration (permessage-deflate)
    compression: CompressionConfig,
    /// Maximum message size in bytes (None = unlimited)
    max_message_size: Option<usize>,
    /// Maximum frame size in bytes (None = unlimited)
    max_frame_size: Option<usize>,
    /// Maximum size of the send queue
    max_send_queue: Option<usize>,
    /// Accept unmasked frames from server (not recommended for security)
    accept_unmasked_frames: bool,
}

impl Default for WebSocketConfig {
    fn default() -> Self {
        Self {
            compression: CompressionConfig::default(),
            max_message_size: Some(64 << 20), // 64 MB
            max_frame_size: Some(16 << 20),   // 16 MB
            max_send_queue: None,
            accept_unmasked_frames: false,
        }
    }
}

impl WebSocketConfig {
    /// Create a new configuration with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a configuration with compression enabled using default compression settings
    ///
    /// # Example
    ///
    /// ```
    /// use websocket_protocol::WebSocketConfig;
    ///
    /// let config = WebSocketConfig::with_compression();
    /// assert!(config.is_compression_enabled());
    /// ```
    pub fn with_compression() -> Self {
        Self {
            compression: CompressionConfig::enabled(),
            ..Default::default()
        }
    }

    /// Set the compression configuration
    ///
    /// # Example
    ///
    /// ```
    /// use websocket_protocol::{WebSocketConfig, CompressionConfig};
    ///
    /// let config = WebSocketConfig::new()
    ///     .compression(CompressionConfig::enabled());
    /// assert!(config.is_compression_enabled());
    /// ```
    pub fn compression(mut self, config: CompressionConfig) -> Self {
        self.compression = config;
        self
    }

    /// Get the compression configuration
    pub fn compression_config(&self) -> &CompressionConfig {
        &self.compression
    }

    /// Check if compression is enabled in the configuration
    pub fn is_compression_enabled(&self) -> bool {
        self.compression.is_enabled()
    }

    /// Set maximum message size
    ///
    /// Messages larger than this will be rejected.
    ///
    /// # Arguments
    ///
    /// * `size` - Maximum message size in bytes, or None for unlimited
    pub fn max_message_size(mut self, size: Option<usize>) -> Self {
        self.max_message_size = size;
        self
    }

    /// Get the maximum message size setting
    pub fn get_max_message_size(&self) -> Option<usize> {
        self.max_message_size
    }

    /// Set maximum frame size
    ///
    /// Frames larger than this will be rejected.
    ///
    /// # Arguments
    ///
    /// * `size` - Maximum frame size in bytes, or None for unlimited
    pub fn max_frame_size(mut self, size: Option<usize>) -> Self {
        self.max_frame_size = size;
        self
    }

    /// Get the maximum frame size setting
    pub fn get_max_frame_size(&self) -> Option<usize> {
        self.max_frame_size
    }

    /// Set maximum send queue size
    pub fn max_send_queue(mut self, size: Option<usize>) -> Self {
        self.max_send_queue = size;
        self
    }

    /// Get the maximum send queue size setting
    pub fn get_max_send_queue(&self) -> Option<usize> {
        self.max_send_queue
    }

    /// Set whether to accept unmasked frames
    ///
    /// By default, clients reject unmasked frames from servers as this violates
    /// the WebSocket protocol. Only enable this for testing or known safe servers.
    pub fn accept_unmasked_frames(mut self, accept: bool) -> Self {
        self.accept_unmasked_frames = accept;
        self
    }

    /// Check if accepting unmasked frames is enabled
    pub fn is_accepting_unmasked_frames(&self) -> bool {
        self.accept_unmasked_frames
    }

    /// Convert to internal tungstenite configuration
    #[allow(deprecated)] // max_send_queue is deprecated in tungstenite but we expose it for API completeness
    fn to_tungstenite_config(&self) -> TungsteniteConfig {
        TungsteniteConfig {
            max_message_size: self.max_message_size,
            max_frame_size: self.max_frame_size,
            max_send_queue: self.max_send_queue,
            accept_unmasked_frames: self.accept_unmasked_frames,
            ..Default::default()
        }
    }
}

// ==================== WebSocket Message Types ====================

/// WebSocket message types
///
/// Represents different types of messages that can be sent/received over a WebSocket connection.
#[derive(Debug, Clone)]
pub enum WebSocketMessage {
    /// Text message (UTF-8 encoded string)
    Text(String),
    /// Binary message (arbitrary bytes)
    Binary(Vec<u8>),
    /// Ping message with optional payload
    Ping(Vec<u8>),
    /// Pong message (response to ping)
    Pong(Vec<u8>),
    /// Close message with optional close frame
    Close(Option<CloseFrame>),
}

impl From<tungstenite::Message> for WebSocketMessage {
    fn from(msg: tungstenite::Message) -> Self {
        match msg {
            tungstenite::Message::Text(text) => WebSocketMessage::Text(text),
            tungstenite::Message::Binary(data) => WebSocketMessage::Binary(data),
            tungstenite::Message::Ping(data) => WebSocketMessage::Ping(data),
            tungstenite::Message::Pong(data) => WebSocketMessage::Pong(data),
            tungstenite::Message::Close(frame) => {
                WebSocketMessage::Close(frame.map(|f| CloseFrame {
                    code: f.code.into(),
                    reason: f.reason.to_string(),
                }))
            }
            tungstenite::Message::Frame(_) => {
                // Raw frames are not exposed in our API
                WebSocketMessage::Binary(vec![])
            }
        }
    }
}

impl From<WebSocketMessage> for tungstenite::Message {
    fn from(msg: WebSocketMessage) -> Self {
        match msg {
            WebSocketMessage::Text(text) => tungstenite::Message::Text(text),
            WebSocketMessage::Binary(data) => tungstenite::Message::Binary(data),
            WebSocketMessage::Ping(data) => tungstenite::Message::Ping(data),
            WebSocketMessage::Pong(data) => tungstenite::Message::Pong(data),
            WebSocketMessage::Close(frame) => tungstenite::Message::Close(frame.map(|f| {
                TungsteniteCloseFrame {
                    code: f.code.into(),
                    reason: f.reason.into(),
                }
            })),
        }
    }
}

/// Close frame information
///
/// Contains the status code and reason when closing a WebSocket connection.
#[derive(Debug, Clone)]
pub struct CloseFrame {
    /// Status code indicating the reason for closure
    pub code: u16,
    /// Human-readable reason for closure
    pub reason: String,
}

/// WebSocket connection states
///
/// Represents the current state of a WebSocket connection through its lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WebSocketState {
    /// Connection is being established
    Connecting,
    /// Connection is open and ready for communication
    Open,
    /// Connection is in the process of closing
    Closing,
    /// Connection is closed
    Closed,
}

/// WebSocket connection handle
///
/// Provides methods to send/receive messages and manage the WebSocket connection.
pub struct WebSocketConnection {
    /// Target URL of the WebSocket
    pub url: Url,
    /// Negotiated subprotocol (if any)
    pub protocol: Option<String>,
    /// Negotiated extensions
    pub extensions: Vec<String>,
    /// Channel sender for outgoing messages
    sender: mpsc::Sender<WebSocketMessage>,
    /// Channel receiver for incoming messages
    receiver: mpsc::Receiver<WebSocketMessage>,
    /// Current connection state
    state: WebSocketState,
    /// Whether compression was requested for this connection
    compression_requested: bool,
    /// Negotiated compression configuration (if compression was successfully negotiated)
    compression_config: Option<CompressionConfig>,
}

impl std::fmt::Debug for WebSocketConnection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WebSocketConnection")
            .field("url", &self.url)
            .field("protocol", &self.protocol)
            .field("extensions", &self.extensions)
            .field("state", &self.state)
            .field("compression_requested", &self.compression_requested)
            .field("compression_config", &self.compression_config)
            .finish()
    }
}

impl WebSocketConnection {
    /// Create a new WebSocket connection handle
    ///
    /// # Arguments
    ///
    /// * `url` - The WebSocket URL
    /// * `protocol` - Optional subprotocol
    /// * `extensions` - List of negotiated extensions
    pub fn new(
        url: Url,
        protocol: Option<String>,
        extensions: Vec<String>,
    ) -> (Self, mpsc::Sender<WebSocketMessage>, mpsc::Receiver<WebSocketMessage>) {
        let (tx_out, rx_out) = mpsc::channel(100);
        let (tx_in, rx_in) = mpsc::channel(100);

        let connection = Self {
            url,
            protocol,
            extensions,
            sender: tx_out,
            receiver: rx_in,
            state: WebSocketState::Connecting,
            compression_requested: false,
            compression_config: None,
        };

        (connection, tx_in, rx_out)
    }

    /// Create a connection from an established WebSocket stream
    ///
    /// # Arguments
    ///
    /// * `url` - The WebSocket URL
    /// * `ws_stream` - The established WebSocket stream
    /// * `protocol` - Optional negotiated subprotocol
    pub fn from_stream(
        url: Url,
        ws_stream: WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>,
        protocol: Option<String>,
    ) -> Self {
        let (tx_in, mut rx_in) = mpsc::channel::<WebSocketMessage>(100);
        let (tx_out, rx_out) = mpsc::channel::<WebSocketMessage>(100);

        let (mut ws_sender, mut ws_receiver) = ws_stream.split();

        // Task to forward messages from WebSocket to internal channel
        tokio::spawn(async move {
            while let Some(msg_result) = ws_receiver.next().await {
                match msg_result {
                    Ok(msg) => {
                        let ws_msg: WebSocketMessage = msg.into();
                        if tx_out.send(ws_msg).await.is_err() {
                            break; // Channel closed
                        }
                    }
                    Err(_) => break, // Error receiving
                }
            }
        });

        // Task to forward messages from internal channel to WebSocket
        tokio::spawn(async move {
            while let Some(msg) = rx_in.recv().await {
                let tungstenite_msg: tungstenite::Message = msg.into();
                if ws_sender.send(tungstenite_msg).await.is_err() {
                    break; // Error sending or closed
                }
            }
        });

        Self {
            url,
            protocol,
            extensions: vec![],
            sender: tx_in,
            receiver: rx_out,
            state: WebSocketState::Open,
            compression_requested: false,
            compression_config: None,
        }
    }

    /// Create a connection from an established WebSocket stream with compression info
    ///
    /// # Arguments
    ///
    /// * `url` - The WebSocket URL
    /// * `ws_stream` - The established WebSocket stream
    /// * `protocol` - Optional negotiated subprotocol
    /// * `compression_requested` - Whether compression was requested during handshake
    /// * `compression_config` - Negotiated compression configuration (if any)
    pub fn from_stream_with_compression(
        url: Url,
        ws_stream: WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>,
        protocol: Option<String>,
        compression_requested: bool,
        compression_config: Option<CompressionConfig>,
    ) -> Self {
        let (tx_in, mut rx_in) = mpsc::channel::<WebSocketMessage>(100);
        let (tx_out, rx_out) = mpsc::channel::<WebSocketMessage>(100);

        let (mut ws_sender, mut ws_receiver) = ws_stream.split();

        // Task to forward messages from WebSocket to internal channel
        tokio::spawn(async move {
            while let Some(msg_result) = ws_receiver.next().await {
                match msg_result {
                    Ok(msg) => {
                        let ws_msg: WebSocketMessage = msg.into();
                        if tx_out.send(ws_msg).await.is_err() {
                            break; // Channel closed
                        }
                    }
                    Err(_) => break, // Error receiving
                }
            }
        });

        // Task to forward messages from internal channel to WebSocket
        tokio::spawn(async move {
            while let Some(msg) = rx_in.recv().await {
                let tungstenite_msg: tungstenite::Message = msg.into();
                if ws_sender.send(tungstenite_msg).await.is_err() {
                    break; // Error sending or closed
                }
            }
        });

        // Build extensions list based on compression
        let extensions = if compression_config.is_some() {
            vec!["permessage-deflate".to_string()]
        } else {
            vec![]
        };

        Self {
            url,
            protocol,
            extensions,
            sender: tx_in,
            receiver: rx_out,
            state: WebSocketState::Open,
            compression_requested,
            compression_config,
        }
    }

    /// Check if compression was requested for this connection
    ///
    /// Returns true if the client requested compression during the WebSocket handshake.
    /// Note: This does not mean compression is active - use `is_compression_active()` for that.
    pub fn compression_requested(&self) -> bool {
        self.compression_requested
    }

    /// Check if compression is active on this connection
    ///
    /// Returns true if compression was successfully negotiated with the server.
    /// When active, messages will be compressed using permessage-deflate.
    ///
    /// # Note
    ///
    /// Currently, tungstenite 0.21 does not support permessage-deflate,
    /// so this will always return false until upstream support is added.
    pub fn is_compression_active(&self) -> bool {
        self.compression_config.is_some()
    }

    /// Get the negotiated compression configuration
    ///
    /// Returns the compression configuration if compression was successfully negotiated,
    /// or None if compression is not active.
    pub fn negotiated_compression(&self) -> Option<&CompressionConfig> {
        self.compression_config.as_ref()
    }

    /// Send a message through the WebSocket
    ///
    /// # Arguments
    ///
    /// * `message` - The message to send
    ///
    /// # Returns
    ///
    /// Result indicating success or network error
    pub async fn send(&self, message: WebSocketMessage) -> Result<(), NetworkError> {
        self.sender
            .send(message)
            .await
            .map_err(|e| NetworkError::WebSocketError(format!("Failed to send message: {}", e)))
    }

    /// Receive next message from the WebSocket
    ///
    /// # Returns
    ///
    /// Option containing the result of receiving a message, or None if the channel is closed
    pub async fn recv(&mut self) -> Option<Result<WebSocketMessage, NetworkError>> {
        self.receiver.recv().await.map(Ok)
    }

    /// Close the WebSocket connection
    ///
    /// # Arguments
    ///
    /// * `code` - Close status code
    /// * `reason` - Human-readable reason for closure
    ///
    /// # Returns
    ///
    /// Result indicating success or network error
    pub async fn close(&mut self, code: u16, reason: String) -> Result<(), NetworkError> {
        self.state = WebSocketState::Closing;
        let close_msg = WebSocketMessage::Close(Some(CloseFrame { code, reason }));
        self.send(close_msg).await?;
        self.state = WebSocketState::Closed;
        Ok(())
    }

    /// Get connection state
    ///
    /// # Returns
    ///
    /// Current WebSocket connection state
    pub fn state(&self) -> WebSocketState {
        self.state
    }

    /// Update connection state (internal use)
    #[allow(dead_code)]
    fn set_state(&mut self, state: WebSocketState) {
        self.state = state;
    }
}

/// WebSocket client
///
/// Provides methods to establish WebSocket connections.
pub struct WebSocketClient;

impl WebSocketClient {
    /// Create a new WebSocket client
    pub fn new() -> Self {
        Self
    }

    /// Connect to a WebSocket server
    ///
    /// # Arguments
    ///
    /// * `url` - The WebSocket URL to connect to (ws:// or wss://)
    /// * `protocols` - List of subprotocols to request
    ///
    /// # Returns
    ///
    /// Result containing the WebSocket connection or network error
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use websocket_protocol::WebSocketClient;
    /// use url::Url;
    ///
    /// # async fn example() -> Result<(), network_errors::NetworkError> {
    /// let client = WebSocketClient::new();
    /// let url = Url::parse("ws://echo.websocket.org").unwrap();
    /// let connection = client.connect(url, vec![]).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn connect(
        &self,
        url: Url,
        _protocols: Vec<String>,
    ) -> Result<WebSocketConnection, NetworkError> {
        // Validate URL scheme
        let scheme = url.scheme();
        if scheme != "ws" && scheme != "wss" {
            return Err(NetworkError::InvalidUrl(format!(
                "Invalid WebSocket scheme: {}. Expected 'ws' or 'wss'",
                scheme
            )));
        }

        // Connect using tokio-tungstenite
        let (ws_stream, _response) = connect_async(url.as_str())
            .await
            .map_err(|e| NetworkError::WebSocketError(format!("Connection failed: {}", e)))?;

        // TODO: Extract negotiated protocol from response headers
        let protocol = None;

        Ok(WebSocketConnection::from_stream(url, ws_stream, protocol))
    }

    /// Connect to a WebSocket server with custom configuration
    ///
    /// This method allows specifying configuration options including compression
    /// settings, message size limits, and frame size limits.
    ///
    /// # Arguments
    ///
    /// * `url` - The WebSocket URL to connect to (ws:// or wss://)
    /// * `protocols` - List of subprotocols to request
    /// * `config` - WebSocket configuration including compression settings
    ///
    /// # Returns
    ///
    /// Result containing the WebSocket connection or network error
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use websocket_protocol::{WebSocketClient, WebSocketConfig, CompressionConfig};
    /// use url::Url;
    ///
    /// # async fn example() -> Result<(), network_errors::NetworkError> {
    /// let client = WebSocketClient::new();
    /// let url = Url::parse("ws://echo.websocket.org").unwrap();
    ///
    /// // Connect with compression enabled
    /// let config = WebSocketConfig::with_compression();
    /// let connection = client.connect_with_config(url, vec![], config).await?;
    ///
    /// // Check if compression was requested
    /// assert!(connection.compression_requested());
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Note
    ///
    /// Compression negotiation depends on server support. Currently, tungstenite 0.21
    /// does not include built-in permessage-deflate support. When compression is
    /// enabled in the config, the connection will record that compression was requested,
    /// but actual compression will only be active when upstream support is available.
    pub async fn connect_with_config(
        &self,
        url: Url,
        _protocols: Vec<String>,
        config: WebSocketConfig,
    ) -> Result<WebSocketConnection, NetworkError> {
        // Validate URL scheme
        let scheme = url.scheme();
        if scheme != "ws" && scheme != "wss" {
            return Err(NetworkError::InvalidUrl(format!(
                "Invalid WebSocket scheme: {}. Expected 'ws' or 'wss'",
                scheme
            )));
        }

        let compression_requested = config.is_compression_enabled();
        let tungstenite_config = config.to_tungstenite_config();

        // Connect using tokio-tungstenite with configuration
        // Note: disable_nagle is set to false to allow Nagle's algorithm for better throughput
        let (ws_stream, _response) = connect_async_with_config(
            url.as_str(),
            Some(tungstenite_config),
            false, // disable_nagle
        )
        .await
        .map_err(|e| NetworkError::WebSocketError(format!("Connection failed: {}", e)))?;

        // TODO: Extract negotiated protocol and extensions from response headers
        // When tungstenite adds permessage-deflate support, we'll parse the
        // Sec-WebSocket-Extensions header to determine if compression was negotiated
        let protocol = None;

        // Currently, compression is not negotiated since tungstenite doesn't support it yet
        // When support is added, we would parse the response headers here
        let compression_config = None;

        Ok(WebSocketConnection::from_stream_with_compression(
            url,
            ws_stream,
            protocol,
            compression_requested,
            compression_config,
        ))
    }
}

impl Default for WebSocketClient {
    fn default() -> Self {
        Self::new()
    }
}

/// Configuration for WebSocket reconnection behavior
///
/// Controls how the reconnecting WebSocket handles connection drops
/// and implements exponential backoff.
#[derive(Debug, Clone)]
pub struct ReconnectConfig {
    /// Initial delay before first reconnection attempt in milliseconds (default: 1000ms)
    pub initial_delay_ms: u64,
    /// Maximum delay between reconnection attempts in milliseconds (default: 30000ms)
    pub max_delay_ms: u64,
    /// Maximum number of reconnection attempts (None = infinite)
    pub max_attempts: Option<u32>,
    /// Multiplier for exponential backoff (default: 2.0)
    pub multiplier: f64,
}

impl Default for ReconnectConfig {
    fn default() -> Self {
        Self {
            initial_delay_ms: 1000,
            max_delay_ms: 30000,
            max_attempts: None,
            multiplier: 2.0,
        }
    }
}

impl ReconnectConfig {
    /// Create a new ReconnectConfig with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the initial delay before first reconnection attempt
    pub fn with_initial_delay_ms(mut self, delay_ms: u64) -> Self {
        self.initial_delay_ms = delay_ms;
        self
    }

    /// Set the maximum delay between reconnection attempts
    pub fn with_max_delay_ms(mut self, delay_ms: u64) -> Self {
        self.max_delay_ms = delay_ms;
        self
    }

    /// Set the maximum number of reconnection attempts
    pub fn with_max_attempts(mut self, attempts: u32) -> Self {
        self.max_attempts = Some(attempts);
        self
    }

    /// Set the multiplier for exponential backoff
    pub fn with_multiplier(mut self, multiplier: f64) -> Self {
        self.multiplier = multiplier;
        self
    }

    /// Calculate the delay for a given attempt number using exponential backoff
    ///
    /// The delay follows the pattern: initial_delay * multiplier^attempt
    /// capped at max_delay_ms
    pub fn calculate_delay(&self, attempt: u32) -> Duration {
        let delay_ms = (self.initial_delay_ms as f64 * self.multiplier.powi(attempt as i32)) as u64;
        let capped_delay = delay_ms.min(self.max_delay_ms);
        Duration::from_millis(capped_delay)
    }
}

/// Reconnection state events
///
/// Represents the various states during the reconnection lifecycle.
#[derive(Debug, Clone, PartialEq)]
pub enum ReconnectionEvent {
    /// Initial connection attempt
    Connecting,
    /// Successfully connected
    Connected,
    /// Connection lost, attempting to reconnect
    Reconnecting {
        /// Current attempt number (0-indexed)
        attempt: u32,
        /// Delay before this attempt in milliseconds
        delay_ms: u64,
    },
    /// Reconnection failed after max attempts
    ReconnectionFailed {
        /// Total number of attempts made
        total_attempts: u32,
    },
    /// Connection was closed intentionally
    Disconnected,
}

/// A WebSocket client with automatic reconnection support
///
/// Wraps the basic WebSocket connection and automatically handles
/// reconnection with exponential backoff when the connection drops unexpectedly.
///
/// # Example
///
/// ```no_run
/// use websocket_protocol::{ReconnectingWebSocket, ReconnectConfig};
/// use url::Url;
///
/// # async fn example() -> Result<(), network_errors::NetworkError> {
/// let config = ReconnectConfig::new()
///     .with_initial_delay_ms(1000)
///     .with_max_delay_ms(30000)
///     .with_max_attempts(10);
///
/// let url = Url::parse("ws://echo.websocket.org").unwrap();
/// let ws = ReconnectingWebSocket::new(url, vec![], config);
///
/// // Subscribe to reconnection events
/// let mut events = ws.subscribe_events();
/// tokio::spawn(async move {
///     while let Ok(()) = events.changed().await {
///         let event = events.borrow().clone();
///         println!("Reconnection event: {:?}", event);
///     }
/// });
///
/// // Connect and use the WebSocket
/// ws.connect().await?;
/// # Ok(())
/// # }
/// ```
pub struct ReconnectingWebSocket {
    url: Url,
    protocols: Vec<String>,
    config: ReconnectConfig,
    connection: Arc<Mutex<Option<WebSocketConnection>>>,
    event_sender: watch::Sender<ReconnectionEvent>,
    event_receiver: watch::Receiver<ReconnectionEvent>,
    is_intentionally_closed: Arc<Mutex<bool>>,
}

impl ReconnectingWebSocket {
    /// Create a new ReconnectingWebSocket
    ///
    /// # Arguments
    ///
    /// * `url` - The WebSocket URL to connect to
    /// * `protocols` - List of subprotocols to request
    /// * `config` - Reconnection configuration
    pub fn new(url: Url, protocols: Vec<String>, config: ReconnectConfig) -> Self {
        let (event_sender, event_receiver) = watch::channel(ReconnectionEvent::Disconnected);
        Self {
            url,
            protocols,
            config,
            connection: Arc::new(Mutex::new(None)),
            event_sender,
            event_receiver,
            is_intentionally_closed: Arc::new(Mutex::new(false)),
        }
    }

    /// Subscribe to reconnection events
    ///
    /// Returns a watch receiver that can be used to monitor reconnection state changes.
    pub fn subscribe_events(&self) -> watch::Receiver<ReconnectionEvent> {
        self.event_receiver.clone()
    }

    /// Get the current reconnection event state
    pub fn current_event(&self) -> ReconnectionEvent {
        self.event_receiver.borrow().clone()
    }

    /// Connect to the WebSocket server
    ///
    /// Establishes the initial connection. If the connection drops unexpectedly,
    /// automatic reconnection will be attempted according to the configuration.
    pub async fn connect(&self) -> Result<(), NetworkError> {
        *self.is_intentionally_closed.lock().await = false;
        let _ = self.event_sender.send(ReconnectionEvent::Connecting);

        let client = WebSocketClient::new();
        let connection = client.connect(self.url.clone(), self.protocols.clone()).await?;

        let _ = self.event_sender.send(ReconnectionEvent::Connected);
        *self.connection.lock().await = Some(connection);

        Ok(())
    }

    /// Attempt to reconnect with exponential backoff
    ///
    /// This method is called internally when a connection drop is detected,
    /// but can also be called manually to force a reconnection attempt.
    pub async fn reconnect(&self) -> Result<(), NetworkError> {
        // Check if intentionally closed
        if *self.is_intentionally_closed.lock().await {
            return Err(NetworkError::WebSocketError(
                "Connection was intentionally closed".to_string(),
            ));
        }

        let client = WebSocketClient::new();
        let mut attempt: u32 = 0;

        loop {
            // Check max attempts
            if let Some(max) = self.config.max_attempts {
                if attempt >= max {
                    let _ = self.event_sender.send(ReconnectionEvent::ReconnectionFailed {
                        total_attempts: attempt,
                    });
                    return Err(NetworkError::WebSocketError(format!(
                        "Failed to reconnect after {} attempts",
                        attempt
                    )));
                }
            }

            // Calculate delay for this attempt
            let delay = self.config.calculate_delay(attempt);
            let delay_ms = delay.as_millis() as u64;

            let _ = self.event_sender.send(ReconnectionEvent::Reconnecting {
                attempt,
                delay_ms,
            });

            // Wait before attempting
            tokio::time::sleep(delay).await;

            // Check again if intentionally closed during sleep
            if *self.is_intentionally_closed.lock().await {
                let _ = self.event_sender.send(ReconnectionEvent::Disconnected);
                return Err(NetworkError::WebSocketError(
                    "Connection was intentionally closed during reconnection".to_string(),
                ));
            }

            // Attempt connection
            match client.connect(self.url.clone(), self.protocols.clone()).await {
                Ok(connection) => {
                    let _ = self.event_sender.send(ReconnectionEvent::Connected);
                    *self.connection.lock().await = Some(connection);
                    return Ok(());
                }
                Err(_) => {
                    attempt += 1;
                    // Continue to next attempt
                }
            }
        }
    }

    /// Send a message through the WebSocket
    ///
    /// If not connected, returns an error.
    pub async fn send(&self, message: WebSocketMessage) -> Result<(), NetworkError> {
        let guard = self.connection.lock().await;
        match &*guard {
            Some(conn) => conn.send(message).await,
            None => Err(NetworkError::WebSocketError("Not connected".to_string())),
        }
    }

    /// Close the WebSocket connection intentionally
    ///
    /// This prevents automatic reconnection from occurring.
    pub async fn close(&self, code: u16, reason: String) -> Result<(), NetworkError> {
        *self.is_intentionally_closed.lock().await = true;
        let _ = self.event_sender.send(ReconnectionEvent::Disconnected);

        let mut guard = self.connection.lock().await;
        if let Some(ref mut conn) = *guard {
            conn.close(code, reason).await?;
        }
        *guard = None;

        Ok(())
    }

    /// Check if currently connected
    pub async fn is_connected(&self) -> bool {
        let guard = self.connection.lock().await;
        if let Some(ref conn) = *guard {
            conn.state() == WebSocketState::Open
        } else {
            false
        }
    }

    /// Get the URL this WebSocket connects to
    pub fn url(&self) -> &Url {
        &self.url
    }

    /// Get the reconnection configuration
    pub fn config(&self) -> &ReconnectConfig {
        &self.config
    }
}

impl std::fmt::Debug for ReconnectingWebSocket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReconnectingWebSocket")
            .field("url", &self.url)
            .field("protocols", &self.protocols)
            .field("config", &self.config)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_websocket_state_transitions() {
        assert_ne!(WebSocketState::Connecting, WebSocketState::Open);
        assert_ne!(WebSocketState::Open, WebSocketState::Closing);
        assert_ne!(WebSocketState::Closing, WebSocketState::Closed);
    }

    #[test]
    fn test_close_frame_creation() {
        let frame = CloseFrame {
            code: 1000,
            reason: "Normal closure".to_string(),
        };
        assert_eq!(frame.code, 1000);
        assert_eq!(frame.reason, "Normal closure");
    }

    #[test]
    fn test_message_conversions() {
        // Test Text conversion
        let text_msg = WebSocketMessage::Text("Hello".to_string());
        let tungstenite_msg: tungstenite::Message = text_msg.clone().into();
        let back_msg: WebSocketMessage = tungstenite_msg.into();
        match back_msg {
            WebSocketMessage::Text(text) => assert_eq!(text, "Hello"),
            _ => panic!("Expected Text message"),
        }

        // Test Binary conversion
        let binary_msg = WebSocketMessage::Binary(vec![1, 2, 3]);
        let tungstenite_msg: tungstenite::Message = binary_msg.clone().into();
        let back_msg: WebSocketMessage = tungstenite_msg.into();
        match back_msg {
            WebSocketMessage::Binary(data) => assert_eq!(data, vec![1, 2, 3]),
            _ => panic!("Expected Binary message"),
        }
    }

    #[test]
    fn test_invalid_url_scheme() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let client = WebSocketClient::new();
            let url = Url::parse("http://example.com").unwrap();
            let result = client.connect(url, vec![]).await;
            assert!(result.is_err());
            match result.unwrap_err() {
                NetworkError::InvalidUrl(msg) => {
                    assert!(msg.contains("Invalid WebSocket scheme"));
                }
                _ => panic!("Expected InvalidUrl error"),
            }
        });
    }

    // ==================== ReconnectConfig Tests ====================

    #[test]
    fn test_reconnect_config_default_values() {
        let config = ReconnectConfig::default();
        assert_eq!(config.initial_delay_ms, 1000);
        assert_eq!(config.max_delay_ms, 30000);
        assert_eq!(config.max_attempts, None);
        assert_eq!(config.multiplier, 2.0);
    }

    #[test]
    fn test_reconnect_config_new() {
        let config = ReconnectConfig::new();
        assert_eq!(config.initial_delay_ms, 1000);
        assert_eq!(config.max_delay_ms, 30000);
    }

    #[test]
    fn test_reconnect_config_builder_initial_delay() {
        let config = ReconnectConfig::new().with_initial_delay_ms(500);
        assert_eq!(config.initial_delay_ms, 500);
    }

    #[test]
    fn test_reconnect_config_builder_max_delay() {
        let config = ReconnectConfig::new().with_max_delay_ms(60000);
        assert_eq!(config.max_delay_ms, 60000);
    }

    #[test]
    fn test_reconnect_config_builder_max_attempts() {
        let config = ReconnectConfig::new().with_max_attempts(5);
        assert_eq!(config.max_attempts, Some(5));
    }

    #[test]
    fn test_reconnect_config_builder_multiplier() {
        let config = ReconnectConfig::new().with_multiplier(1.5);
        assert_eq!(config.multiplier, 1.5);
    }

    #[test]
    fn test_reconnect_config_builder_chain() {
        let config = ReconnectConfig::new()
            .with_initial_delay_ms(500)
            .with_max_delay_ms(60000)
            .with_max_attempts(10)
            .with_multiplier(1.5);

        assert_eq!(config.initial_delay_ms, 500);
        assert_eq!(config.max_delay_ms, 60000);
        assert_eq!(config.max_attempts, Some(10));
        assert_eq!(config.multiplier, 1.5);
    }

    #[test]
    fn test_reconnect_config_calculate_delay_first_attempt() {
        let config = ReconnectConfig::new().with_initial_delay_ms(1000);
        let delay = config.calculate_delay(0);
        assert_eq!(delay, Duration::from_millis(1000));
    }

    #[test]
    fn test_reconnect_config_calculate_delay_exponential_backoff() {
        let config = ReconnectConfig::new()
            .with_initial_delay_ms(1000)
            .with_multiplier(2.0)
            .with_max_delay_ms(30000);

        // Expected sequence: 1s, 2s, 4s, 8s, 16s, 30s (capped)
        assert_eq!(config.calculate_delay(0), Duration::from_millis(1000));
        assert_eq!(config.calculate_delay(1), Duration::from_millis(2000));
        assert_eq!(config.calculate_delay(2), Duration::from_millis(4000));
        assert_eq!(config.calculate_delay(3), Duration::from_millis(8000));
        assert_eq!(config.calculate_delay(4), Duration::from_millis(16000));
    }

    #[test]
    fn test_reconnect_config_calculate_delay_capped_at_max() {
        let config = ReconnectConfig::new()
            .with_initial_delay_ms(1000)
            .with_multiplier(2.0)
            .with_max_delay_ms(30000);

        // After attempt 4 (16s), attempt 5 would be 32s but capped at 30s
        assert_eq!(config.calculate_delay(5), Duration::from_millis(30000));
        assert_eq!(config.calculate_delay(6), Duration::from_millis(30000));
        assert_eq!(config.calculate_delay(10), Duration::from_millis(30000));
    }

    #[test]
    fn test_reconnect_config_calculate_delay_custom_multiplier() {
        let config = ReconnectConfig::new()
            .with_initial_delay_ms(100)
            .with_multiplier(3.0)
            .with_max_delay_ms(10000);

        // Expected: 100, 300, 900, 2700, 8100, 10000 (capped)
        assert_eq!(config.calculate_delay(0), Duration::from_millis(100));
        assert_eq!(config.calculate_delay(1), Duration::from_millis(300));
        assert_eq!(config.calculate_delay(2), Duration::from_millis(900));
        assert_eq!(config.calculate_delay(3), Duration::from_millis(2700));
        assert_eq!(config.calculate_delay(4), Duration::from_millis(8100));
        assert_eq!(config.calculate_delay(5), Duration::from_millis(10000)); // capped
    }

    #[test]
    fn test_reconnect_config_clone() {
        let config = ReconnectConfig::new()
            .with_initial_delay_ms(500)
            .with_max_attempts(5);
        let cloned = config.clone();
        assert_eq!(cloned.initial_delay_ms, 500);
        assert_eq!(cloned.max_attempts, Some(5));
    }

    #[test]
    fn test_reconnect_config_debug() {
        let config = ReconnectConfig::new();
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("ReconnectConfig"));
        assert!(debug_str.contains("initial_delay_ms"));
    }

    // ==================== ReconnectionEvent Tests ====================

    #[test]
    fn test_reconnection_event_connecting() {
        let event = ReconnectionEvent::Connecting;
        assert_eq!(event, ReconnectionEvent::Connecting);
    }

    #[test]
    fn test_reconnection_event_connected() {
        let event = ReconnectionEvent::Connected;
        assert_eq!(event, ReconnectionEvent::Connected);
    }

    #[test]
    fn test_reconnection_event_reconnecting() {
        let event = ReconnectionEvent::Reconnecting {
            attempt: 3,
            delay_ms: 4000,
        };
        match event {
            ReconnectionEvent::Reconnecting { attempt, delay_ms } => {
                assert_eq!(attempt, 3);
                assert_eq!(delay_ms, 4000);
            }
            _ => panic!("Expected Reconnecting event"),
        }
    }

    #[test]
    fn test_reconnection_event_failed() {
        let event = ReconnectionEvent::ReconnectionFailed { total_attempts: 5 };
        match event {
            ReconnectionEvent::ReconnectionFailed { total_attempts } => {
                assert_eq!(total_attempts, 5);
            }
            _ => panic!("Expected ReconnectionFailed event"),
        }
    }

    #[test]
    fn test_reconnection_event_disconnected() {
        let event = ReconnectionEvent::Disconnected;
        assert_eq!(event, ReconnectionEvent::Disconnected);
    }

    #[test]
    fn test_reconnection_event_clone() {
        let event = ReconnectionEvent::Reconnecting {
            attempt: 2,
            delay_ms: 2000,
        };
        let cloned = event.clone();
        assert_eq!(event, cloned);
    }

    #[test]
    fn test_reconnection_event_debug() {
        let event = ReconnectionEvent::Reconnecting {
            attempt: 1,
            delay_ms: 1000,
        };
        let debug_str = format!("{:?}", event);
        assert!(debug_str.contains("Reconnecting"));
        assert!(debug_str.contains("attempt"));
        assert!(debug_str.contains("delay_ms"));
    }

    // ==================== ReconnectingWebSocket Tests ====================

    #[test]
    fn test_reconnecting_websocket_new() {
        let url = Url::parse("ws://localhost:8080/ws").unwrap();
        let config = ReconnectConfig::new().with_max_attempts(5);
        let ws = ReconnectingWebSocket::new(url.clone(), vec!["chat".to_string()], config);

        assert_eq!(ws.url(), &url);
        assert_eq!(ws.config().max_attempts, Some(5));
    }

    #[test]
    fn test_reconnecting_websocket_initial_state() {
        let url = Url::parse("ws://localhost:8080/ws").unwrap();
        let config = ReconnectConfig::new();
        let ws = ReconnectingWebSocket::new(url, vec![], config);

        assert_eq!(ws.current_event(), ReconnectionEvent::Disconnected);
    }

    #[test]
    fn test_reconnecting_websocket_subscribe_events() {
        let url = Url::parse("ws://localhost:8080/ws").unwrap();
        let config = ReconnectConfig::new();
        let ws = ReconnectingWebSocket::new(url, vec![], config);

        let receiver = ws.subscribe_events();
        assert_eq!(*receiver.borrow(), ReconnectionEvent::Disconnected);
    }

    #[test]
    fn test_reconnecting_websocket_debug() {
        let url = Url::parse("ws://localhost:8080/ws").unwrap();
        let config = ReconnectConfig::new();
        let ws = ReconnectingWebSocket::new(url, vec![], config);

        let debug_str = format!("{:?}", ws);
        assert!(debug_str.contains("ReconnectingWebSocket"));
        assert!(debug_str.contains("localhost"));
    }

    #[tokio::test]
    async fn test_reconnecting_websocket_not_connected_send_error() {
        let url = Url::parse("ws://localhost:8080/ws").unwrap();
        let config = ReconnectConfig::new();
        let ws = ReconnectingWebSocket::new(url, vec![], config);

        let result = ws.send(WebSocketMessage::Text("test".to_string())).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            NetworkError::WebSocketError(msg) => {
                assert!(msg.contains("Not connected"));
            }
            _ => panic!("Expected WebSocketError"),
        }
    }

    #[tokio::test]
    async fn test_reconnecting_websocket_is_connected_when_not_connected() {
        let url = Url::parse("ws://localhost:8080/ws").unwrap();
        let config = ReconnectConfig::new();
        let ws = ReconnectingWebSocket::new(url, vec![], config);

        assert!(!ws.is_connected().await);
    }

    #[tokio::test]
    async fn test_reconnecting_websocket_close_when_not_connected() {
        let url = Url::parse("ws://localhost:8080/ws").unwrap();
        let config = ReconnectConfig::new();
        let ws = ReconnectingWebSocket::new(url, vec![], config);

        // Close should succeed even when not connected
        let result = ws.close(1000, "Normal closure".to_string()).await;
        assert!(result.is_ok());
        assert_eq!(ws.current_event(), ReconnectionEvent::Disconnected);
    }

    #[tokio::test]
    async fn test_reconnecting_websocket_reconnect_after_close_fails() {
        let url = Url::parse("ws://localhost:8080/ws").unwrap();
        let config = ReconnectConfig::new().with_max_attempts(1);
        let ws = ReconnectingWebSocket::new(url, vec![], config);

        // Close first (sets intentionally closed flag)
        ws.close(1000, "Test".to_string()).await.unwrap();

        // Reconnect should fail because it was intentionally closed
        let result = ws.reconnect().await;
        assert!(result.is_err());
        match result.unwrap_err() {
            NetworkError::WebSocketError(msg) => {
                assert!(msg.contains("intentionally closed"));
            }
            _ => panic!("Expected WebSocketError about intentional close"),
        }
    }

    // ==================== CompressionConfig Tests ====================

    #[test]
    fn test_compression_config_default() {
        let config = CompressionConfig::default();
        assert!(!config.is_enabled());
        assert_eq!(config.client_max_window_bits(), 15);
        assert_eq!(config.server_max_window_bits(), 15);
        assert!(!config.server_no_context_takeover());
        assert!(!config.client_no_context_takeover());
    }

    #[test]
    fn test_compression_config_enabled() {
        let config = CompressionConfig::enabled();
        assert!(config.is_enabled());
        assert_eq!(config.client_max_window_bits(), 15);
        assert_eq!(config.server_max_window_bits(), 15);
    }

    #[test]
    fn test_compression_config_disabled() {
        let config = CompressionConfig::disabled();
        assert!(!config.is_enabled());
    }

    #[test]
    fn test_compression_config_client_window_bits() {
        let config = CompressionConfig::enabled()
            .with_client_max_window_bits(12);
        assert_eq!(config.client_max_window_bits(), 12);
    }

    #[test]
    fn test_compression_config_server_window_bits() {
        let config = CompressionConfig::enabled()
            .with_server_max_window_bits(10);
        assert_eq!(config.server_max_window_bits(), 10);
    }

    #[test]
    fn test_compression_config_server_no_context_takeover() {
        let config = CompressionConfig::enabled()
            .with_server_no_context_takeover(true);
        assert!(config.server_no_context_takeover());
    }

    #[test]
    fn test_compression_config_client_no_context_takeover() {
        let config = CompressionConfig::enabled()
            .with_client_no_context_takeover(true);
        assert!(config.client_no_context_takeover());
    }

    #[test]
    fn test_compression_config_builder_chain() {
        let config = CompressionConfig::enabled()
            .with_client_max_window_bits(12)
            .with_server_max_window_bits(11)
            .with_server_no_context_takeover(true)
            .with_client_no_context_takeover(true);

        assert!(config.is_enabled());
        assert_eq!(config.client_max_window_bits(), 12);
        assert_eq!(config.server_max_window_bits(), 11);
        assert!(config.server_no_context_takeover());
        assert!(config.client_no_context_takeover());
    }

    #[test]
    #[should_panic(expected = "Window bits must be between 8 and 15")]
    fn test_compression_config_client_window_bits_too_low() {
        CompressionConfig::enabled().with_client_max_window_bits(7);
    }

    #[test]
    #[should_panic(expected = "Window bits must be between 8 and 15")]
    fn test_compression_config_client_window_bits_too_high() {
        CompressionConfig::enabled().with_client_max_window_bits(16);
    }

    #[test]
    #[should_panic(expected = "Window bits must be between 8 and 15")]
    fn test_compression_config_server_window_bits_too_low() {
        CompressionConfig::enabled().with_server_max_window_bits(7);
    }

    #[test]
    #[should_panic(expected = "Window bits must be between 8 and 15")]
    fn test_compression_config_server_window_bits_too_high() {
        CompressionConfig::enabled().with_server_max_window_bits(16);
    }

    #[test]
    fn test_compression_config_to_extension_header_disabled() {
        let config = CompressionConfig::disabled();
        assert!(config.to_extension_header().is_none());
    }

    #[test]
    fn test_compression_config_to_extension_header_default() {
        let config = CompressionConfig::enabled();
        let header = config.to_extension_header();
        assert!(header.is_some());
        assert_eq!(header.unwrap(), "permessage-deflate");
    }

    #[test]
    fn test_compression_config_to_extension_header_with_params() {
        let config = CompressionConfig::enabled()
            .with_client_max_window_bits(12)
            .with_server_max_window_bits(11)
            .with_server_no_context_takeover(true);

        let header = config.to_extension_header().unwrap();
        assert!(header.contains("permessage-deflate"));
        assert!(header.contains("client_max_window_bits=12"));
        assert!(header.contains("server_max_window_bits=11"));
        assert!(header.contains("server_no_context_takeover"));
    }

    #[test]
    fn test_compression_config_from_extension_header_basic() {
        let config = CompressionConfig::from_extension_header("permessage-deflate");
        assert!(config.is_some());
        let config = config.unwrap();
        assert!(config.is_enabled());
        assert_eq!(config.client_max_window_bits(), 15);
        assert_eq!(config.server_max_window_bits(), 15);
    }

    #[test]
    fn test_compression_config_from_extension_header_with_params() {
        let header = "permessage-deflate; client_max_window_bits=12; server_max_window_bits=11; server_no_context_takeover";
        let config = CompressionConfig::from_extension_header(header);
        assert!(config.is_some());
        let config = config.unwrap();
        assert!(config.is_enabled());
        assert_eq!(config.client_max_window_bits(), 12);
        assert_eq!(config.server_max_window_bits(), 11);
        assert!(config.server_no_context_takeover());
    }

    #[test]
    fn test_compression_config_from_extension_header_no_compression() {
        let config = CompressionConfig::from_extension_header("some-other-extension");
        assert!(config.is_none());
    }

    #[test]
    fn test_compression_config_equality() {
        let config1 = CompressionConfig::enabled().with_client_max_window_bits(12);
        let config2 = CompressionConfig::enabled().with_client_max_window_bits(12);
        let config3 = CompressionConfig::enabled().with_client_max_window_bits(13);

        assert_eq!(config1, config2);
        assert_ne!(config1, config3);
    }

    #[test]
    fn test_compression_config_clone() {
        let config = CompressionConfig::enabled()
            .with_client_max_window_bits(12);
        let cloned = config.clone();
        assert_eq!(config, cloned);
    }

    #[test]
    fn test_compression_config_debug() {
        let config = CompressionConfig::enabled();
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("CompressionConfig"));
        assert!(debug_str.contains("enabled"));
    }

    // ==================== WebSocketConfig Tests ====================

    #[test]
    fn test_websocket_config_default() {
        let config = WebSocketConfig::default();
        assert!(!config.is_compression_enabled());
        assert_eq!(config.get_max_message_size(), Some(64 << 20));
        assert_eq!(config.get_max_frame_size(), Some(16 << 20));
        assert_eq!(config.get_max_send_queue(), None);
        assert!(!config.is_accepting_unmasked_frames());
    }

    #[test]
    fn test_websocket_config_new() {
        let config = WebSocketConfig::new();
        assert!(!config.is_compression_enabled());
    }

    #[test]
    fn test_websocket_config_with_compression() {
        let config = WebSocketConfig::with_compression();
        assert!(config.is_compression_enabled());
    }

    #[test]
    fn test_websocket_config_compression_setter() {
        let compression = CompressionConfig::enabled().with_client_max_window_bits(12);
        let config = WebSocketConfig::new().compression(compression);
        assert!(config.is_compression_enabled());
        assert_eq!(config.compression_config().client_max_window_bits(), 12);
    }

    #[test]
    fn test_websocket_config_max_message_size() {
        let config = WebSocketConfig::new().max_message_size(Some(1024 * 1024));
        assert_eq!(config.get_max_message_size(), Some(1024 * 1024));
    }

    #[test]
    fn test_websocket_config_max_message_size_unlimited() {
        let config = WebSocketConfig::new().max_message_size(None);
        assert_eq!(config.get_max_message_size(), None);
    }

    #[test]
    fn test_websocket_config_max_frame_size() {
        let config = WebSocketConfig::new().max_frame_size(Some(512 * 1024));
        assert_eq!(config.get_max_frame_size(), Some(512 * 1024));
    }

    #[test]
    fn test_websocket_config_max_send_queue() {
        let config = WebSocketConfig::new().max_send_queue(Some(100));
        assert_eq!(config.get_max_send_queue(), Some(100));
    }

    #[test]
    fn test_websocket_config_accept_unmasked_frames() {
        let config = WebSocketConfig::new().accept_unmasked_frames(true);
        assert!(config.is_accepting_unmasked_frames());
    }

    #[test]
    fn test_websocket_config_builder_chain() {
        let config = WebSocketConfig::new()
            .compression(CompressionConfig::enabled())
            .max_message_size(Some(32 << 20))
            .max_frame_size(Some(8 << 20))
            .max_send_queue(Some(50))
            .accept_unmasked_frames(true);

        assert!(config.is_compression_enabled());
        assert_eq!(config.get_max_message_size(), Some(32 << 20));
        assert_eq!(config.get_max_frame_size(), Some(8 << 20));
        assert_eq!(config.get_max_send_queue(), Some(50));
        assert!(config.is_accepting_unmasked_frames());
    }

    #[test]
    fn test_websocket_config_clone() {
        let config = WebSocketConfig::with_compression()
            .max_message_size(Some(1024));
        let cloned = config.clone();
        assert!(cloned.is_compression_enabled());
        assert_eq!(cloned.get_max_message_size(), Some(1024));
    }

    #[test]
    fn test_websocket_config_debug() {
        let config = WebSocketConfig::with_compression();
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("WebSocketConfig"));
        assert!(debug_str.contains("compression"));
    }

    // ==================== WebSocket Connect with Config Tests ====================

    #[test]
    fn test_connect_with_config_invalid_scheme() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let client = WebSocketClient::new();
            let url = Url::parse("http://example.com").unwrap();
            let config = WebSocketConfig::with_compression();
            let result = client.connect_with_config(url, vec![], config).await;
            assert!(result.is_err());
            match result.unwrap_err() {
                NetworkError::InvalidUrl(msg) => {
                    assert!(msg.contains("Invalid WebSocket scheme"));
                }
                _ => panic!("Expected InvalidUrl error"),
            }
        });
    }
}
