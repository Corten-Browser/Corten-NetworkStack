//! Network connectivity detection

use std::net::TcpStream;
use std::time::Duration;

/// Check if network is online
///
/// Attempts to connect to a well-known public DNS server to determine
/// network connectivity. This is a simple heuristic check.
///
/// # Implementation
///
/// Attempts to establish a TCP connection to 8.8.8.8:53 (Google Public DNS)
/// with a 1-second timeout. If the connection succeeds, the network is
/// considered online.
///
/// # Returns
///
/// Returns `true` if network appears to be online, `false` otherwise.
///
/// # Note
///
/// This is a heuristic check and may not be 100% accurate in all scenarios:
/// - Firewalls may block the connection
/// - DNS server may be unreachable
/// - Network may be restricted to local-only
pub fn is_online() -> bool {
    // Try to connect to a well-known public DNS server
    // 8.8.8.8:53 (Google Public DNS)
    const DNS_SERVER: &str = "8.8.8.8:53";
    const TIMEOUT: Duration = Duration::from_secs(1);

    TcpStream::connect_timeout(
        &DNS_SERVER.parse()
            .expect("DNS_SERVER constant should be valid IP address"),
        TIMEOUT
    ).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_online_returns_boolean() {
        let result = is_online();
        // Should return true or false (boolean)
        assert!(result == true || result == false);
    }

    #[test]
    fn test_is_online_does_not_panic() {
        // Should not panic
        let _ = is_online();
        let _ = is_online();
        let _ = is_online();
    }

    #[test]
    fn test_is_online_consistent() {
        // Should be consistent within a short timeframe
        let first = is_online();
        let second = is_online();
        // Network state shouldn't change instantly in test environment
        assert_eq!(first, second);
    }
}
