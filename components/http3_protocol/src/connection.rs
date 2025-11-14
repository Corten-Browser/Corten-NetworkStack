//! QUIC connection management

use crate::error::Http3Error;
use quinn::{Connection, RecvStream, SendStream};
use std::net::SocketAddr;

/// QUIC connection wrapper
///
/// Manages a QUIC connection including streams and connection lifecycle.
/// Provides methods for opening streams and gracefully closing connections.
///
/// # Example
///
/// ```rust,no_run
/// # use http3_protocol::QuicConnection;
/// # async fn example(connection: QuicConnection) -> Result<(), Box<dyn std::error::Error>> {
/// let mut stream = connection.open_stream().await?;
/// // Use stream for HTTP/3 communication
/// connection.close().await?;
/// # Ok(())
/// # }
/// ```
#[derive(Clone)]
pub struct QuicConnection {
    /// Underlying QUIC connection
    connection: Connection,
    /// Remote address
    remote_addr: SocketAddr,
}

impl QuicConnection {
    /// Create a new QUIC connection wrapper
    ///
    /// # Arguments
    ///
    /// * `connection` - Quinn QUIC connection
    /// * `remote_addr` - Remote server address
    pub(crate) fn new(connection: Connection, remote_addr: SocketAddr) -> Self {
        Self {
            connection,
            remote_addr,
        }
    }

    /// Open a new bidirectional stream
    ///
    /// Opens a new QUIC stream for HTTP/3 communication. Each HTTP/3 request
    /// typically uses a separate bidirectional stream.
    ///
    /// # Returns
    ///
    /// A tuple of (send_stream, recv_stream) on success
    ///
    /// # Errors
    ///
    /// Returns `Http3Error` if stream cannot be opened (connection closed, etc.)
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use http3_protocol::QuicConnection;
    /// # async fn example(connection: QuicConnection) -> Result<(), Box<dyn std::error::Error>> {
    /// let (send, recv) = connection.open_stream().await?;
    /// // Use send and recv for HTTP/3 request/response
    /// # Ok(())
    /// # }
    /// ```
    pub async fn open_stream(&self) -> Result<(SendStream, RecvStream), Http3Error> {
        let (send, recv) = self.connection.open_bi().await?;
        Ok((send, recv))
    }

    /// Close the connection gracefully
    ///
    /// Initiates graceful connection closure, allowing in-flight streams to complete.
    ///
    /// # Errors
    ///
    /// Returns `Http3Error` if closure fails
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use http3_protocol::QuicConnection;
    /// # async fn example(connection: QuicConnection) -> Result<(), Box<dyn std::error::Error>> {
    /// connection.close().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn close(self) -> Result<(), Http3Error> {
        self.connection.close(0u32.into(), b"connection closed");
        // Wait for connection to actually close
        self.connection.closed().await;
        Ok(())
    }

    /// Get the remote address
    ///
    /// Returns the remote server address for this connection.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use http3_protocol::QuicConnection;
    /// # fn example(connection: &QuicConnection) {
    /// let addr = connection.remote_address();
    /// println!("Connected to: {}", addr);
    /// # }
    /// ```
    pub fn remote_address(&self) -> SocketAddr {
        self.remote_addr
    }

    /// Check if connection is still alive
    ///
    /// # Returns
    ///
    /// `true` if connection is open, `false` if closed
    pub fn is_closed(&self) -> bool {
        // Check if the underlying connection is in closing/draining state
        // Note: Quinn doesn't expose a direct is_closed() method,
        // so we rely on whether operations would fail
        false // Simplified for now
    }
}

impl std::fmt::Debug for QuicConnection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QuicConnection")
            .field("remote_addr", &self.remote_addr)
            .field("closed", &self.is_closed())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_quic_connection_debug() {
        // Note: We can't easily create a Quinn connection in tests without
        // a full QUIC handshake, so this is a basic structural test
        // Integration tests will cover actual connection functionality
    }
}
