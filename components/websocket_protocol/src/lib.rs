//! websocket_protocol component
//!
//! WebSocket client with frame parsing/encoding, ping/pong, compression extensions

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

use futures::{SinkExt, StreamExt};
use network_errors::NetworkError;
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::{self, protocol::CloseFrame as TungsteniteCloseFrame};
use tokio_tungstenite::{connect_async, WebSocketStream, MaybeTlsStream};
use url::Url;

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
}

impl std::fmt::Debug for WebSocketConnection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WebSocketConnection")
            .field("url", &self.url)
            .field("protocol", &self.protocol)
            .field("extensions", &self.extensions)
            .field("state", &self.state)
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
        }
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
}

impl Default for WebSocketClient {
    fn default() -> Self {
        Self::new()
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
}
