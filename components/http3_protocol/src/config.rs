//! HTTP/3 configuration

use std::time::Duration;

/// Configuration for HTTP/3 client
///
/// Controls QUIC and HTTP/3 protocol behavior including timeouts, payload sizes,
/// and feature flags for 0-RTT and connection migration.
///
/// # Example
///
/// ```rust
/// use http3_protocol::Http3Config;
/// use std::time::Duration;
///
/// let config = Http3Config::default()
///     .with_0rtt(true)
///     .with_connection_migration(true)
///     .with_max_idle_timeout(Duration::from_secs(60));
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Http3Config {
    /// Maximum idle timeout before connection is closed
    ///
    /// If no data is sent or received for this duration, the connection will be terminated.
    /// Default: 30 seconds
    pub max_idle_timeout: Duration,

    /// Maximum UDP payload size in bytes
    ///
    /// Controls the maximum size of QUIC packets. Must be between 1200 (IPv6 minimum MTU)
    /// and 65527 (UDP maximum - headers).
    /// Default: 1350 bytes (safe for most networks)
    pub max_udp_payload_size: u32,

    /// Enable 0-RTT (Zero Round Trip Time) connections
    ///
    /// When enabled, allows resuming previous connections without a full handshake.
    /// Reduces latency for subsequent connections but requires session resumption support.
    /// Default: false (for security)
    pub enable_0rtt: bool,

    /// Enable connection migration
    ///
    /// When enabled, allows connections to survive network changes (e.g., switching from
    /// WiFi to cellular). This is a core QUIC feature for mobile devices.
    /// Default: true
    pub enable_connection_migration: bool,
}

impl Default for Http3Config {
    fn default() -> Self {
        Self {
            max_idle_timeout: Duration::from_secs(30),
            max_udp_payload_size: 1350,
            enable_0rtt: false,
            enable_connection_migration: true,
        }
    }
}

impl Http3Config {
    /// Create a new configuration with default values
    ///
    /// # Example
    ///
    /// ```rust
    /// use http3_protocol::Http3Config;
    ///
    /// let config = Http3Config::new();
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable or disable 0-RTT connections
    ///
    /// # Arguments
    ///
    /// * `enabled` - Whether to enable 0-RTT
    ///
    /// # Example
    ///
    /// ```rust
    /// use http3_protocol::Http3Config;
    ///
    /// let config = Http3Config::default().with_0rtt(true);
    /// ```
    pub fn with_0rtt(mut self, enabled: bool) -> Self {
        self.enable_0rtt = enabled;
        self
    }

    /// Enable or disable connection migration
    ///
    /// # Arguments
    ///
    /// * `enabled` - Whether to enable connection migration
    ///
    /// # Example
    ///
    /// ```rust
    /// use http3_protocol::Http3Config;
    ///
    /// let config = Http3Config::default().with_connection_migration(false);
    /// ```
    pub fn with_connection_migration(mut self, enabled: bool) -> Self {
        self.enable_connection_migration = enabled;
        self
    }

    /// Set maximum idle timeout
    ///
    /// # Arguments
    ///
    /// * `timeout` - Maximum idle duration before connection closes
    ///
    /// # Example
    ///
    /// ```rust
    /// use http3_protocol::Http3Config;
    /// use std::time::Duration;
    ///
    /// let config = Http3Config::default()
    ///     .with_max_idle_timeout(Duration::from_secs(60));
    /// ```
    pub fn with_max_idle_timeout(mut self, timeout: Duration) -> Self {
        self.max_idle_timeout = timeout;
        self
    }

    /// Set maximum UDP payload size
    ///
    /// # Arguments
    ///
    /// * `size` - Maximum UDP payload size in bytes (1200-65527)
    ///
    /// # Panics
    ///
    /// Panics if size is outside valid range [1200, 65527]
    ///
    /// # Example
    ///
    /// ```rust
    /// use http3_protocol::Http3Config;
    ///
    /// let config = Http3Config::default()
    ///     .with_max_udp_payload_size(1400);
    /// ```
    pub fn with_max_udp_payload_size(mut self, size: u32) -> Self {
        assert!(
            (1200..=65527).contains(&size),
            "UDP payload size must be between 1200 and 65527"
        );
        self.max_udp_payload_size = size;
        self
    }

    /// Validate the configuration
    ///
    /// Checks that all configuration values are valid.
    ///
    /// # Returns
    ///
    /// `Ok(())` if configuration is valid, `Err(String)` with error message otherwise
    pub fn validate(&self) -> Result<(), String> {
        if self.max_udp_payload_size < 1200 {
            return Err("UDP payload size must be at least 1200 (IPv6 minimum MTU)".to_string());
        }
        if self.max_udp_payload_size > 65527 {
            return Err(
                "UDP payload size must not exceed 65527 (UDP maximum - headers)".to_string(),
            );
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Http3Config::default();
        assert_eq!(config.max_idle_timeout, Duration::from_secs(30));
        assert_eq!(config.max_udp_payload_size, 1350);
        assert!(!config.enable_0rtt);
        assert!(config.enable_connection_migration);
    }

    #[test]
    fn test_builder_pattern() {
        let config = Http3Config::default()
            .with_0rtt(true)
            .with_connection_migration(false)
            .with_max_idle_timeout(Duration::from_secs(60))
            .with_max_udp_payload_size(1400);

        assert!(config.enable_0rtt);
        assert!(!config.enable_connection_migration);
        assert_eq!(config.max_idle_timeout, Duration::from_secs(60));
        assert_eq!(config.max_udp_payload_size, 1400);
    }

    #[test]
    fn test_validate_success() {
        let config = Http3Config::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_payload_too_small() {
        let config = Http3Config {
            max_udp_payload_size: 1000,
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_payload_too_large() {
        let config = Http3Config {
            max_udp_payload_size: 70000,
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    #[should_panic(expected = "UDP payload size must be between 1200 and 65527")]
    fn test_with_invalid_udp_size_panics() {
        Http3Config::default().with_max_udp_payload_size(1000);
    }
}
