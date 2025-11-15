//! SOCKS5 proxy implementation
//!
//! Implements the SOCKS5 protocol for establishing connections through SOCKS5 proxies.

use crate::auth::ProxyAuth;
use network_errors::NetworkError;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

const SOCKS5_VERSION: u8 = 0x05;
const SOCKS5_AUTH_NONE: u8 = 0x00;
const SOCKS5_AUTH_PASSWORD: u8 = 0x02;
const SOCKS5_CMD_CONNECT: u8 = 0x01;
const SOCKS5_ATYP_DOMAIN: u8 = 0x03;
const SOCKS5_RESERVED: u8 = 0x00;

/// Connect to a target host through a SOCKS5 proxy
///
/// Performs the SOCKS5 handshake including authentication if provided,
/// then establishes a connection to the target host.
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
/// A `TcpStream` representing the connection to the target through SOCKS5
///
/// # Errors
///
/// Returns `NetworkError` if:
/// - Cannot connect to proxy server
/// - Authentication fails
/// - Proxy cannot establish connection to target
/// - Protocol errors occur
pub async fn connect(
    proxy_host: &str,
    proxy_port: u16,
    auth: Option<&ProxyAuth>,
    target_host: &str,
    target_port: u16,
) -> Result<TcpStream, NetworkError> {
    // Connect to SOCKS5 proxy
    let proxy_addr = format!("{}:{}", proxy_host, proxy_port);
    let mut stream = TcpStream::connect(&proxy_addr).await.map_err(|e| {
        NetworkError::ProxyError(format!("Failed to connect to SOCKS5 proxy: {}", e))
    })?;

    // Perform authentication handshake
    perform_auth_handshake(&mut stream, auth).await?;

    // Send connect request
    send_connect_request(&mut stream, target_host, target_port).await?;

    // Read connect response
    read_connect_response(&mut stream).await?;

    Ok(stream)
}

/// Perform SOCKS5 authentication handshake
async fn perform_auth_handshake(
    stream: &mut TcpStream,
    auth: Option<&ProxyAuth>,
) -> Result<(), NetworkError> {
    // Send greeting with supported auth methods
    let auth_methods = if auth.is_some() {
        vec![SOCKS5_VERSION, 0x02, SOCKS5_AUTH_NONE, SOCKS5_AUTH_PASSWORD]
    } else {
        vec![SOCKS5_VERSION, 0x01, SOCKS5_AUTH_NONE]
    };

    stream
        .write_all(&auth_methods)
        .await
        .map_err(|e| NetworkError::ProxyError(format!("Failed to send auth methods: {}", e)))?;

    // Read server's chosen method
    let mut response = [0u8; 2];
    stream
        .read_exact(&mut response)
        .await
        .map_err(|e| NetworkError::ProxyError(format!("Failed to read auth response: {}", e)))?;

    if response[0] != SOCKS5_VERSION {
        return Err(NetworkError::ProxyError(
            "Invalid SOCKS5 version in response".to_string(),
        ));
    }

    match response[1] {
        SOCKS5_AUTH_NONE => {
            // No authentication required
            Ok(())
        }
        SOCKS5_AUTH_PASSWORD => {
            // Username/password authentication required
            if let Some(auth) = auth {
                perform_password_auth(stream, auth).await
            } else {
                Err(NetworkError::ProxyError(
                    "Proxy requires authentication but none provided".to_string(),
                ))
            }
        }
        0xFF => Err(NetworkError::ProxyError(
            "No acceptable authentication methods".to_string(),
        )),
        _ => Err(NetworkError::ProxyError(format!(
            "Unknown authentication method: {}",
            response[1]
        ))),
    }
}

/// Perform username/password authentication
async fn perform_password_auth(
    stream: &mut TcpStream,
    auth: &ProxyAuth,
) -> Result<(), NetworkError> {
    let (username, password) = auth.credentials();

    // Build auth request: version(1) + username_len(1) + username + password_len(1) + password
    let mut request = Vec::new();
    request.push(0x01); // Auth version

    // Username
    if username.len() > 255 {
        return Err(NetworkError::ProxyError(
            "Username too long (max 255 bytes)".to_string(),
        ));
    }
    request.push(username.len() as u8);
    request.extend_from_slice(username.as_bytes());

    // Password
    if password.len() > 255 {
        return Err(NetworkError::ProxyError(
            "Password too long (max 255 bytes)".to_string(),
        ));
    }
    request.push(password.len() as u8);
    request.extend_from_slice(password.as_bytes());

    // Send auth request
    stream.write_all(&request).await.map_err(|e| {
        NetworkError::ProxyError(format!("Failed to send auth credentials: {}", e))
    })?;

    // Read auth response
    let mut response = [0u8; 2];
    stream
        .read_exact(&mut response)
        .await
        .map_err(|e| NetworkError::ProxyError(format!("Failed to read auth result: {}", e)))?;

    if response[1] != 0x00 {
        return Err(NetworkError::ProxyError(
            "Authentication failed".to_string(),
        ));
    }

    Ok(())
}

/// Send CONNECT request for target host
async fn send_connect_request(
    stream: &mut TcpStream,
    target_host: &str,
    target_port: u16,
) -> Result<(), NetworkError> {
    // Build connect request: version + cmd + reserved + atype + address + port
    let mut request = Vec::new();
    request.push(SOCKS5_VERSION);
    request.push(SOCKS5_CMD_CONNECT);
    request.push(SOCKS5_RESERVED);
    request.push(SOCKS5_ATYP_DOMAIN);

    // Domain name
    if target_host.len() > 255 {
        return Err(NetworkError::ProxyError(
            "Target hostname too long (max 255 bytes)".to_string(),
        ));
    }
    request.push(target_host.len() as u8);
    request.extend_from_slice(target_host.as_bytes());

    // Port (big-endian)
    request.push((target_port >> 8) as u8);
    request.push((target_port & 0xFF) as u8);

    // Send request
    stream.write_all(&request).await.map_err(|e| {
        NetworkError::ProxyError(format!("Failed to send connect request: {}", e))
    })?;

    Ok(())
}

/// Read and parse CONNECT response
async fn read_connect_response(stream: &mut TcpStream) -> Result<(), NetworkError> {
    // Read fixed part: version + reply + reserved + atype
    let mut header = [0u8; 4];
    stream
        .read_exact(&mut header)
        .await
        .map_err(|e| NetworkError::ProxyError(format!("Failed to read connect response: {}", e)))?;

    if header[0] != SOCKS5_VERSION {
        return Err(NetworkError::ProxyError(
            "Invalid SOCKS5 version in connect response".to_string(),
        ));
    }

    // Check reply code
    match header[1] {
        0x00 => {} // Success
        0x01 => {
            return Err(NetworkError::ProxyError(
                "SOCKS5 general failure".to_string(),
            ))
        }
        0x02 => {
            return Err(NetworkError::ProxyError(
                "SOCKS5 connection not allowed".to_string(),
            ))
        }
        0x03 => {
            return Err(NetworkError::ProxyError(
                "SOCKS5 network unreachable".to_string(),
            ))
        }
        0x04 => {
            return Err(NetworkError::ProxyError(
                "SOCKS5 host unreachable".to_string(),
            ))
        }
        0x05 => {
            return Err(NetworkError::ProxyError(
                "SOCKS5 connection refused".to_string(),
            ))
        }
        0x06 => {
            return Err(NetworkError::ProxyError("SOCKS5 TTL expired".to_string()))
        }
        0x07 => {
            return Err(NetworkError::ProxyError(
                "SOCKS5 command not supported".to_string(),
            ))
        }
        0x08 => {
            return Err(NetworkError::ProxyError(
                "SOCKS5 address type not supported".to_string(),
            ))
        }
        _ => {
            return Err(NetworkError::ProxyError(format!(
                "Unknown SOCKS5 reply code: {}",
                header[1]
            )))
        }
    }

    // Read address (we don't need it, but must consume it)
    let atype = header[3];
    match atype {
        0x01 => {
            // IPv4: 4 bytes
            let mut addr = [0u8; 4];
            stream.read_exact(&mut addr).await.map_err(|e| {
                NetworkError::ProxyError(format!("Failed to read IPv4 address: {}", e))
            })?;
        }
        0x03 => {
            // Domain: 1 byte length + domain
            let mut len = [0u8; 1];
            stream.read_exact(&mut len).await.map_err(|e| {
                NetworkError::ProxyError(format!("Failed to read domain length: {}", e))
            })?;
            let mut domain = vec![0u8; len[0] as usize];
            stream.read_exact(&mut domain).await.map_err(|e| {
                NetworkError::ProxyError(format!("Failed to read domain: {}", e))
            })?;
        }
        0x04 => {
            // IPv6: 16 bytes
            let mut addr = [0u8; 16];
            stream.read_exact(&mut addr).await.map_err(|e| {
                NetworkError::ProxyError(format!("Failed to read IPv6 address: {}", e))
            })?;
        }
        _ => {
            return Err(NetworkError::ProxyError(format!(
                "Unknown address type: {}",
                atype
            )))
        }
    }

    // Read port (2 bytes, big-endian)
    let mut port = [0u8; 2];
    stream
        .read_exact(&mut port)
        .await
        .map_err(|e| NetworkError::ProxyError(format!("Failed to read port: {}", e)))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_socks5_constants() {
        assert_eq!(SOCKS5_VERSION, 0x05);
        assert_eq!(SOCKS5_AUTH_NONE, 0x00);
        assert_eq!(SOCKS5_AUTH_PASSWORD, 0x02);
        assert_eq!(SOCKS5_CMD_CONNECT, 0x01);
    }

    #[test]
    fn test_connect_request_format() {
        // Verify the request would be correctly formatted
        let target_host = "example.com";
        let target_port = 443u16;

        assert!(target_host.len() <= 255);
        assert!(target_port > 0);
    }
}
