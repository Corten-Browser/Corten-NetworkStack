//! HTTP/2 configuration

use crate::error::{Http2Error, Http2Result};

/// HTTP/2 client configuration
#[derive(Debug, Clone)]
pub struct Http2Config {
    /// Maximum number of concurrent streams per connection
    max_concurrent_streams: u32,

    /// Initial window size for flow control (bytes)
    initial_window_size: u32,

    /// Maximum frame size (bytes)
    max_frame_size: u32,

    /// Enable server push
    enable_push: bool,
}

impl Default for Http2Config {
    fn default() -> Self {
        Self {
            max_concurrent_streams: 100,
            initial_window_size: 65_535, // HTTP/2 default
            max_frame_size: 16_384,      // HTTP/2 default (16 KiB)
            enable_push: false,
        }
    }
}

impl Http2Config {
    /// Create a new HTTP/2 configuration with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set maximum concurrent streams
    pub fn with_max_concurrent_streams(mut self, max: u32) -> Self {
        self.max_concurrent_streams = max;
        self
    }

    /// Set initial window size
    pub fn with_initial_window_size(mut self, size: u32) -> Self {
        self.initial_window_size = size;
        self
    }

    /// Set maximum frame size
    pub fn with_max_frame_size(mut self, size: u32) -> Self {
        self.max_frame_size = size;
        self
    }

    /// Enable or disable server push
    pub fn with_push_enabled(mut self, enabled: bool) -> Self {
        self.enable_push = enabled;
        self
    }

    /// Get maximum concurrent streams
    pub fn max_concurrent_streams(&self) -> u32 {
        self.max_concurrent_streams
    }

    /// Get initial window size
    pub fn initial_window_size(&self) -> u32 {
        self.initial_window_size
    }

    /// Get maximum frame size
    pub fn max_frame_size(&self) -> u32 {
        self.max_frame_size
    }

    /// Check if server push is enabled
    pub fn enable_push(&self) -> bool {
        self.enable_push
    }

    /// Validate configuration according to HTTP/2 specification
    pub fn validate(&self) -> Http2Result<()> {
        // Validate max_concurrent_streams
        if self.max_concurrent_streams == 0 {
            return Err(Http2Error::ConfigError(
                "max_concurrent_streams must be greater than 0".to_string(),
            ));
        }

        // Validate initial_window_size (max is 2^31-1 per HTTP/2 spec)
        if self.initial_window_size > 2_147_483_647 {
            return Err(Http2Error::ConfigError(
                "initial_window_size must not exceed 2^31-1 (2,147,483,647)".to_string(),
            ));
        }

        // Validate max_frame_size (must be between 16,384 and 16,777,215 per HTTP/2 spec)
        if self.max_frame_size < 16_384 {
            return Err(Http2Error::ConfigError(
                "max_frame_size must be at least 16,384 bytes".to_string(),
            ));
        }

        if self.max_frame_size > 16_777_215 {
            return Err(Http2Error::ConfigError(
                "max_frame_size must not exceed 16,777,215 bytes".to_string(),
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Http2Config::default();
        assert_eq!(config.max_concurrent_streams(), 100);
        assert_eq!(config.initial_window_size(), 65_535);
        assert_eq!(config.max_frame_size(), 16_384);
        assert!(!config.enable_push());
    }

    #[test]
    fn test_builder_pattern() {
        let config = Http2Config::new()
            .with_max_concurrent_streams(200)
            .with_initial_window_size(131_072)
            .with_max_frame_size(32_768)
            .with_push_enabled(true);

        assert_eq!(config.max_concurrent_streams(), 200);
        assert_eq!(config.initial_window_size(), 131_072);
        assert_eq!(config.max_frame_size(), 32_768);
        assert!(config.enable_push());
    }

    #[test]
    fn test_validation_max_streams_zero() {
        let config = Http2Config::new().with_max_concurrent_streams(0);
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validation_window_size_too_large() {
        let config = Http2Config::new().with_initial_window_size(u32::MAX);
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validation_frame_size_too_small() {
        let config = Http2Config::new().with_max_frame_size(16_383);
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validation_frame_size_too_large() {
        let config = Http2Config::new().with_max_frame_size(16_777_216);
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validation_valid_config() {
        let config = Http2Config::new()
            .with_max_concurrent_streams(150)
            .with_initial_window_size(98_304)
            .with_max_frame_size(24_576);

        assert!(config.validate().is_ok());
    }
}
