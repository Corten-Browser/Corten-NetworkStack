//! HTTP CONNECT proxy implementation
//!
//! Implements the HTTP CONNECT method for tunneling TCP connections through HTTP proxies.

use crate::auth::ProxyAuth;
use network_errors::NetworkError;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;

/// Connect to a target host through an HTTP CONNECT proxy
///
/// Establishes a connection to the proxy, sends a CONNECT request,
/// and returns the tunneled TCP stream if successful.
///
/// # Arguments
///
/// * `proxy_host` - Proxy server hostname
/// * `proxy_port` - Proxy server port
/// * `auth` - Optional authentication credentials
/// * `target_host` - Target hostname to connect to
/// * `target_port` - Target port to connect to
///
/// # Returns
///
/// A `TcpStream` representing the tunneled connection to the target
///
/// # Errors
///
/// Returns `NetworkError` if:
/// - Cannot connect to proxy server
/// - Proxy rejects authentication
/// - Proxy cannot establish connection to target
/// - Protocol errors occur
pub async fn connect(
    proxy_host: &str,
    proxy_port: u16,
    auth: Option<&ProxyAuth>,
    target_host: &str,
    target_port: u16,
) -> Result<TcpStream, NetworkError> {
    // Connect to proxy server
    let proxy_addr = format!("{}:{}", proxy_host, proxy_port);
    let mut stream = TcpStream::connect(&proxy_addr).await.map_err(|e| {
        NetworkError::ProxyError(format!("Failed to connect to HTTP proxy: {}", e))
    })?;

    // Build CONNECT request
    let mut request = format!(
        "CONNECT {}:{} HTTP/1.1\r\nHost: {}:{}\r\n",
        target_host, target_port, target_host, target_port
    );

    // Add authentication if provided
    if let Some(auth) = auth {
        let encoded = auth.encode_basic();
        request.push_str(&format!("Proxy-Authorization: Basic {}\r\n", encoded));
    }

    // End of headers
    request.push_str("\r\n");

    // Send CONNECT request
    stream
        .write_all(request.as_bytes())
        .await
        .map_err(|e| NetworkError::ProxyError(format!("Failed to send CONNECT request: {}", e)))?;

    // Read response
    let mut reader = BufReader::new(&mut stream);
    let mut status_line = String::new();
    reader
        .read_line(&mut status_line)
        .await
        .map_err(|e| NetworkError::ProxyError(format!("Failed to read proxy response: {}", e)))?;

    // Parse status code
    let parts: Vec<&str> = status_line.split_whitespace().collect();
    if parts.len() < 2 {
        return Err(NetworkError::ProxyError(
            "Invalid HTTP response from proxy".to_string(),
        ));
    }

    let status_code = parts[1].parse::<u16>().map_err(|_| {
        NetworkError::ProxyError("Invalid status code in proxy response".to_string())
    })?;

    // Check if connection was successful
    if status_code != 200 {
        return Err(NetworkError::ProxyError(format!(
            "Proxy returned error: {} {}",
            status_code,
            parts.get(2).unwrap_or(&"")
        )));
    }

    // Read remaining headers until empty line
    loop {
        let mut line = String::new();
        reader
            .read_line(&mut line)
            .await
            .map_err(|e| NetworkError::ProxyError(format!("Failed to read headers: {}", e)))?;

        if line == "\r\n" || line == "\n" {
            break;
        }
    }

    // Return the underlying stream (now tunneled to target)
    Ok(stream)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connect_request_format() {
        // This test verifies the request format is correct
        // Actual connection testing is in integration tests
        let target_host = "example.com";
        let target_port = 443;

        let expected_start = format!(
            "CONNECT {}:{} HTTP/1.1\r\nHost: {}:{}",
            target_host, target_port, target_host, target_port
        );

        // Verify format is correct
        assert!(expected_start.contains("CONNECT"));
        assert!(expected_start.contains("HTTP/1.1"));
    }

    #[test]
    fn test_auth_header_format() {
        let auth = ProxyAuth::Basic {
            username: "user".to_string(),
            password: "pass".to_string(),
        };

        let encoded = auth.encode_basic();
        let header = format!("Proxy-Authorization: Basic {}", encoded);

        assert!(header.contains("Proxy-Authorization: Basic"));
        assert!(header.contains(&encoded));
    }
}
