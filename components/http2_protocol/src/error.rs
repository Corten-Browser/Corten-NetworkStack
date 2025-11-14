//! Error types for HTTP/2 protocol

use std::fmt;

/// Errors that can occur during HTTP/2 operations
#[derive(Debug, Clone)]
pub enum Http2Error {
    /// Configuration validation error
    ConfigError(String),

    /// Connection error
    ConnectionError(String),

    /// Protocol error
    ProtocolError(String),

    /// Stream error
    StreamError(String),

    /// Timeout error
    TimeoutError(String),

    /// Network error
    NetworkError(String),
}

impl fmt::Display for Http2Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Http2Error::ConfigError(msg) => write!(f, "Configuration error: {}", msg),
            Http2Error::ConnectionError(msg) => write!(f, "Connection error: {}", msg),
            Http2Error::ProtocolError(msg) => write!(f, "Protocol error: {}", msg),
            Http2Error::StreamError(msg) => write!(f, "Stream error: {}", msg),
            Http2Error::TimeoutError(msg) => write!(f, "Timeout error: {}", msg),
            Http2Error::NetworkError(msg) => write!(f, "Network error: {}", msg),
        }
    }
}

impl std::error::Error for Http2Error {}

/// Result type for HTTP/2 operations
pub type Http2Result<T> = Result<T, Http2Error>;
