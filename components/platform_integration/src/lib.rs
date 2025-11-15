//! Platform-specific integrations
//!
//! This component provides platform-specific functionality for:
//! - System proxy configuration detection
//! - System certificate store access
//! - Network connectivity detection
//!
//! # Examples
//!
//! ```
//! use platform_integration::PlatformIntegration;
//!
//! // Get system proxy configuration
//! let proxy_config = PlatformIntegration::get_system_proxy_config();
//! if let Ok(config) = proxy_config {
//!     if config.enabled {
//!         println!("HTTP proxy: {:?}", config.http_proxy);
//!     }
//! }
//!
//! // Check network connectivity
//! if PlatformIntegration::is_online() {
//!     println!("Network is online");
//! }
//! ```

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

use network_errors::NetworkError;

mod proxy;
mod certs;
mod network;

pub use proxy::SystemProxyConfig;

/// Platform integration service
///
/// Provides static methods for accessing platform-specific functionality.
pub struct PlatformIntegration;

impl PlatformIntegration {
    /// Get system proxy configuration
    ///
    /// Reads proxy configuration from environment variables:
    /// - HTTP_PROXY: HTTP proxy server URL
    /// - HTTPS_PROXY: HTTPS proxy server URL
    /// - NO_PROXY: Comma-separated list of domains to bypass proxy
    ///
    /// # Returns
    ///
    /// Returns a `SystemProxyConfig` containing proxy settings, or an error if
    /// configuration cannot be determined.
    ///
    /// # Examples
    ///
    /// ```
    /// use platform_integration::PlatformIntegration;
    ///
    /// let config = PlatformIntegration::get_system_proxy_config().unwrap();
    /// if config.enabled {
    ///     println!("Proxy configured");
    /// }
    /// ```
    pub fn get_system_proxy_config() -> Result<SystemProxyConfig, NetworkError> {
        proxy::get_system_proxy_config()
    }

    /// Get system certificate store
    ///
    /// Retrieves certificates from the system's certificate store.
    /// Returns an empty vector on unsupported platforms (graceful degradation).
    ///
    /// # Returns
    ///
    /// Returns a vector of certificates, where each certificate is represented
    /// as a Vec<u8> (DER-encoded certificate data).
    ///
    /// # Examples
    ///
    /// ```
    /// use platform_integration::PlatformIntegration;
    ///
    /// let certs = PlatformIntegration::get_system_cert_store().unwrap();
    /// println!("Found {} system certificates", certs.len());
    /// ```
    pub fn get_system_cert_store() -> Result<Vec<Vec<u8>>, NetworkError> {
        certs::get_system_cert_store()
    }

    /// Check if network is online
    ///
    /// Detects whether the system has network connectivity.
    /// This is a simple heuristic check and may not be 100% accurate.
    ///
    /// # Returns
    ///
    /// Returns `true` if network appears to be online, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use platform_integration::PlatformIntegration;
    ///
    /// if PlatformIntegration::is_online() {
    ///     println!("Network is available");
    /// } else {
    ///     println!("Network is offline");
    /// }
    /// ```
    pub fn is_online() -> bool {
        network::is_online()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_integration_exists() {
        // Verify PlatformIntegration type exists
        let _: PlatformIntegration;
    }
}
