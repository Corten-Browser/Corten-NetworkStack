//! network_stack component
//!
//! Main NetworkStack trait implementation, protocol orchestration, message bus integration
//!
//! This component provides the top-level NetworkStack trait that orchestrates all network
//! protocols (HTTP/1.1, HTTP/2, HTTP/3, WebSocket, WebRTC) and integrates with DNS, TLS,
//! cookie management, and HTTP caching.

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

use async_trait::async_trait;
use bytes::Bytes;
use futures::stream::Stream;
use network_errors::NetworkError;
use network_types::{NetworkRequest, NetworkResponse};
use std::pin::Pin;
use std::sync::Arc;
use url::Url;

mod config;
mod stack_impl;

pub use config::NetworkConfig;
pub use stack_impl::{NetworkStackImpl, NetworkStatus, ConnectionType, EffectiveConnectionType};

/// Main Network Stack component interface
///
/// This trait defines the high-level API for all network operations in the browser.
/// It handles protocol selection, request routing, and integration with supporting
/// components (DNS, TLS, cookies, cache).
#[async_trait]
pub trait NetworkStack: Send + Sync {
    /// Initiate an HTTP request
    ///
    /// Routes the request to the appropriate protocol handler (HTTP/1.1, HTTP/2, HTTP/3)
    /// based on URL scheme and configuration. Integrates with cookies, cache, DNS, and TLS.
    ///
    /// # Arguments
    /// * `request` - The network request to execute
    ///
    /// # Returns
    /// * `Ok(NetworkResponse)` - Successful response
    /// * `Err(NetworkError)` - Network or protocol error
    async fn fetch(&self, request: NetworkRequest) -> Result<NetworkResponse, NetworkError>;

    /// Stream response body chunks
    ///
    /// Similar to `fetch()` but returns a streaming response for large bodies.
    ///
    /// # Arguments
    /// * `request` - The network request to execute
    ///
    /// # Returns
    /// * `Ok(Stream)` - Stream of response body chunks
    /// * `Err(NetworkError)` - Network or protocol error
    async fn stream_response(
        &self,
        request: NetworkRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<Bytes, NetworkError>> + Send>>, NetworkError>;

    /// Open a WebSocket connection
    ///
    /// Establishes a WebSocket connection (ws:// or wss://) to the specified URL.
    ///
    /// # Arguments
    /// * `url` - WebSocket URL (ws:// or wss://)
    /// * `protocols` - Optional subprotocols to negotiate
    ///
    /// # Returns
    /// * `Ok(WebSocketConnection)` - Established WebSocket connection
    /// * `Err(NetworkError)` - Connection error
    async fn connect_websocket(
        &self,
        url: Url,
        protocols: Vec<String>,
    ) -> Result<websocket_protocol::WebSocketConnection, NetworkError>;

    /// Initialize a WebRTC peer connection
    ///
    /// Creates a new WebRTC peer connection with the specified configuration.
    ///
    /// # Arguments
    /// * `config` - RTC configuration (ICE servers, transport policy, etc.)
    ///
    /// # Returns
    /// * `Ok(RtcPeerConnection)` - New peer connection
    /// * `Err(NetworkError)` - Initialization error
    async fn create_rtc_peer_connection(
        &self,
        config: webrtc_peer::RtcConfiguration,
    ) -> Result<webrtc_peer::RtcPeerConnection, NetworkError>;

    /// Get current network status
    ///
    /// Returns information about network connectivity, connection type, and performance.
    fn get_network_status(&self) -> NetworkStatus;

    /// Set network conditions (for throttling/simulation)
    ///
    /// Allows simulating different network conditions for testing or throttling.
    ///
    /// # Arguments
    /// * `conditions` - Network conditions to apply
    fn set_network_conditions(&mut self, conditions: NetworkConditions);

    /// Clear all cached data
    ///
    /// Removes all entries from the HTTP cache.
    async fn clear_cache(&mut self) -> Result<(), NetworkError>;

    /// Get cookie store handle
    ///
    /// Returns a reference to the cookie store for cookie management.
    fn cookie_store(&self) -> Arc<cookie_manager::CookieStore>;

    /// Get certificate store handle
    ///
    /// Returns a reference to the certificate store for TLS certificate management.
    fn cert_store(&self) -> Arc<tls_manager::CertificateStore>;
}

/// Network conditions for throttling and simulation
///
/// Allows simulating different network conditions like slow connections or offline mode.
#[derive(Debug, Clone, Copy)]
pub struct NetworkConditions {
    /// Offline mode (no network connectivity)
    pub offline: bool,
    /// Download throughput in bytes per second (0 = unlimited)
    pub download_throughput: u64,
    /// Upload throughput in bytes per second (0 = unlimited)
    pub upload_throughput: u64,
    /// Added latency in milliseconds
    pub latency: u32,
}

impl Default for NetworkConditions {
    fn default() -> Self {
        Self {
            offline: false,
            download_throughput: 0,  // Unlimited
            upload_throughput: 0,    // Unlimited
            latency: 0,
        }
    }
}
