/// Test helper utilities for integration tests
use std::net::{IpAddr, Ipv4Addr};
use std::time::Duration;
use url::Url;

/// Create a test URL with the given scheme and host
pub fn create_test_url(scheme: &str, host: &str, path: &str) -> Url {
    Url::parse(&format!("{}://{}{}", scheme, host, path))
        .expect("Failed to parse test URL")
}

/// Create an HTTP URL for testing
pub fn http_url(host: &str, path: &str) -> Url {
    create_test_url("http", host, path)
}

/// Create an HTTPS URL for testing
pub fn https_url(host: &str, path: &str) -> Url {
    create_test_url("https", host, path)
}

/// Create a WebSocket URL for testing
pub fn ws_url(host: &str, path: &str) -> Url {
    create_test_url("ws", host, path)
}

/// Create a secure WebSocket URL for testing
pub fn wss_url(host: &str, path: &str) -> Url {
    create_test_url("wss", host, path)
}

/// Get a test IP address
pub fn test_ip() -> IpAddr {
    IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))
}

/// Get example.com IP addresses (for real DNS testing)
/// Note: example.com always resolves to specific IPs per RFC 2606
pub fn example_com_ips() -> Vec<IpAddr> {
    vec![
        IpAddr::V4(Ipv4Addr::new(93, 184, 216, 34)), // example.com actual IP
    ]
}

/// Standard test timeout for async operations
pub fn test_timeout() -> Duration {
    Duration::from_secs(30)
}

/// Short timeout for quick tests
pub fn short_timeout() -> Duration {
    Duration::from_secs(5)
}

/// Assert that an error is a specific NetworkError variant
#[macro_export]
macro_rules! assert_network_error {
    ($result:expr, $pattern:pat) => {
        match $result {
            Err($pattern) => {},
            other => panic!("Expected NetworkError::{}, got {:?}", stringify!($pattern), other),
        }
    };
}

/// Assert that a result is Ok and extract the value
#[macro_export]
macro_rules! assert_ok {
    ($result:expr) => {
        match $result {
            Ok(val) => val,
            Err(e) => panic!("Expected Ok, got Err: {:?}", e),
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_creation() {
        let url = http_url("example.com", "/test");
        assert_eq!(url.scheme(), "http");
        assert_eq!(url.host_str(), Some("example.com"));
        assert_eq!(url.path(), "/test");
    }

    #[test]
    fn test_https_url_creation() {
        let url = https_url("secure.example.com", "/api");
        assert_eq!(url.scheme(), "https");
        assert_eq!(url.host_str(), Some("secure.example.com"));
    }

    #[test]
    fn test_websocket_url_creation() {
        let url = wss_url("ws.example.com", "/socket");
        assert_eq!(url.scheme(), "wss");
        assert_eq!(url.host_str(), Some("ws.example.com"));
    }
}
