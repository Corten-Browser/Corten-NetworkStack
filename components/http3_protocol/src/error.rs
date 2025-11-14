//! HTTP/3 specific errors

use thiserror::Error;

/// Errors specific to HTTP/3 and QUIC operations
#[derive(Debug, Error)]
pub enum Http3Error {
    /// QUIC connection error
    #[error("QUIC connection error: {0}")]
    QuicError(#[from] quinn::ConnectionError),

    /// QUIC write error
    #[error("QUIC write error: {0}")]
    QuicWriteError(#[from] quinn::WriteError),

    /// QUIC read error
    #[error("QUIC read error: {0}")]
    QuicReadError(#[from] quinn::ReadError),

    /// HTTP/3 protocol error
    #[error("HTTP/3 protocol error: {0}")]
    H3Error(String),

    /// Connection closed
    #[error("Connection closed: {0}")]
    ConnectionClosed(String),

    /// Stream error
    #[error("Stream error: {0}")]
    StreamError(String),

    /// Invalid configuration
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    /// TLS error
    #[error("TLS error: {0}")]
    TlsError(String),

    /// DNS resolution error
    #[error("DNS resolution error: {0}")]
    DnsError(String),

    /// Timeout error
    #[error("Operation timed out")]
    Timeout,

    /// Invalid URL
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    /// Invalid request
    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    /// Invalid response
    #[error("Invalid response: {0}")]
    InvalidResponse(String),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

impl From<Http3Error> for network_errors::NetworkError {
    fn from(err: Http3Error) -> Self {
        match err {
            Http3Error::QuicError(e) => {
                network_errors::NetworkError::ConnectionFailed(e.to_string())
            }
            Http3Error::QuicWriteError(e) => {
                network_errors::NetworkError::ConnectionFailed(e.to_string())
            }
            Http3Error::QuicReadError(e) => {
                network_errors::NetworkError::ConnectionFailed(e.to_string())
            }
            Http3Error::H3Error(e) => network_errors::NetworkError::ProtocolError(e),
            Http3Error::ConnectionClosed(e) => network_errors::NetworkError::ConnectionFailed(e),
            Http3Error::StreamError(e) => network_errors::NetworkError::ProtocolError(e),
            Http3Error::InvalidConfig(e) => network_errors::NetworkError::Other(e),
            Http3Error::TlsError(e) => network_errors::NetworkError::TlsError(e),
            Http3Error::DnsError(e) => network_errors::NetworkError::DnsError(e),
            Http3Error::Timeout => {
                network_errors::NetworkError::Timeout(std::time::Duration::from_secs(0))
            }
            Http3Error::InvalidUrl(e) => network_errors::NetworkError::InvalidUrl(e),
            Http3Error::InvalidRequest(e) => network_errors::NetworkError::ProtocolError(e),
            Http3Error::InvalidResponse(e) => network_errors::NetworkError::ProtocolError(e),
            Http3Error::Io(e) => network_errors::NetworkError::Io(e),
        }
    }
}
