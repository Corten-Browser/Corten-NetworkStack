//! network_errors component
//!
//! Error handling: NetworkError enum, Result types, error conversion traits
//!
//! This component provides a comprehensive error type for network operations,
//! covering common failure scenarios like connection failures, DNS errors,
//! TLS/certificate issues, timeouts, and protocol violations.
//!
//! # Examples
//!
//! ```
//! use network_errors::{NetworkError, NetworkResult};
//! use std::time::Duration;
//!
//! fn perform_request() -> NetworkResult<String> {
//!     // Simulate a timeout
//!     Err(NetworkError::Timeout(Duration::from_secs(30)))
//! }
//!
//! match perform_request() {
//!     Ok(data) => println!("Success: {}", data),
//!     Err(NetworkError::Timeout(d)) => println!("Request timed out after {:?}", d),
//!     Err(e) => println!("Error: {}", e),
//! }
//! ```

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

use std::time::Duration;
use thiserror::Error;

/// Network error types
///
/// Represents various failure modes that can occur during network operations.
/// Each variant provides specific context about the nature of the failure.
#[derive(Debug, Error)]
pub enum NetworkError {
    /// Connection failed to establish
    ///
    /// Includes details about why the connection could not be established,
    /// such as host unreachable, connection refused, or network unreachable.
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    /// DNS resolution failed
    ///
    /// Occurs when a hostname cannot be resolved to an IP address.
    #[error("DNS resolution failed: {0}")]
    DnsError(String),

    /// TLS/SSL error
    ///
    /// Represents failures in the TLS handshake or encryption layer.
    #[error("TLS error: {0}")]
    TlsError(String),

    /// Protocol-level error
    ///
    /// Indicates a violation of the network protocol being used (e.g., HTTP, HTTP/2).
    #[error("Protocol error: {0}")]
    ProtocolError(String),

    /// Operation timed out
    ///
    /// Contains the duration after which the timeout occurred.
    #[error("Timeout after {0:?}")]
    Timeout(Duration),

    /// Request was aborted
    ///
    /// The operation was cancelled before completion.
    #[error("Request aborted")]
    Aborted,

    /// Invalid URL provided
    ///
    /// The URL could not be parsed or is malformed.
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    /// Too many redirects
    ///
    /// The request followed too many redirects and was terminated.
    #[error("Too many redirects")]
    TooManyRedirects,

    /// Cache operation failed
    ///
    /// Error related to cache read/write operations.
    #[error("Cache error: {0}")]
    CacheError(String),

    /// Proxy-related error
    ///
    /// Issues with proxy configuration or authentication.
    #[error("Proxy error: {0}")]
    ProxyError(String),

    /// CORS policy violation
    ///
    /// Cross-Origin Resource Sharing (CORS) policy prevented the request.
    #[error("CORS violation: {0}")]
    CorsError(String),

    /// Mixed content blocked
    ///
    /// HTTPS page attempted to load insecure HTTP content.
    #[error("Mixed content blocked")]
    MixedContent,

    /// Certificate validation failed
    ///
    /// TLS certificate could not be validated (expired, self-signed, etc.).
    #[error("Certificate validation failed: {0}")]
    CertificateError(String),

    /// WebSocket error
    ///
    /// Error specific to WebSocket connections.
    #[error("WebSocket error: {0}")]
    WebSocketError(String),

    /// WebRTC error
    ///
    /// Error specific to WebRTC connections.
    #[error("WebRTC error: {0}")]
    WebRtcError(String),

    /// I/O error
    ///
    /// Low-level I/O error from std::io::Error.
    /// Automatically converted via the From trait.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Other unclassified error
    ///
    /// Catch-all for errors that don't fit other categories.
    #[error("Other error: {0}")]
    Other(String),
}

/// Result type alias for network operations
///
/// A convenience type alias that uses `NetworkError` as the error type.
///
/// # Examples
///
/// ```
/// use network_errors::{NetworkError, NetworkResult};
///
/// fn fetch_data(url: &str) -> NetworkResult<Vec<u8>> {
///     if url.is_empty() {
///         return Err(NetworkError::InvalidUrl("URL cannot be empty".to_string()));
///     }
///     Ok(vec![1, 2, 3])
/// }
/// ```
pub type NetworkResult<T> = Result<T, NetworkError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let error = NetworkError::ConnectionFailed("test".to_string());
        assert!(matches!(error, NetworkError::ConnectionFailed(_)));
    }

    #[test]
    fn test_result_type() {
        let result: NetworkResult<i32> = Ok(42);
        assert!(result.is_ok());
    }
}
