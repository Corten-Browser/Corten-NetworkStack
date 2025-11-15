//! Proxy support component
//!
//! HTTP and SOCKS5 proxy client implementation for network connections.
//!
//! This component provides proxy support for establishing TCP connections through
//! HTTP CONNECT proxies and SOCKS5 proxies, with optional authentication.
//!
//! # Examples
//!
//! ```no_run
//! use proxy_support::{ProxyClient, ProxyConfig, ProxyAuth};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // HTTP proxy with authentication
//! let config = ProxyConfig::Http {
//!     host: "proxy.example.com".to_string(),
//!     port: 8080,
//!     auth: Some(ProxyAuth::Basic {
//!         username: "user".to_string(),
//!         password: "pass".to_string(),
//!     }),
//! };
//!
//! let client = ProxyClient::new(config);
//! let stream = client.connect("target.example.com", 443).await?;
//! # Ok(())
//! # }
//! ```

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

use network_errors::NetworkError;
use tokio::net::TcpStream;

mod http_proxy;
mod socks5;
mod auth;

pub use auth::ProxyAuth;

/// Proxy configuration options
///
/// Defines the type of proxy to use and its connection details.
#[derive(Debug, Clone)]
pub enum ProxyConfig {
    /// No proxy - direct connection
    None,

    /// HTTP proxy using CONNECT method
    Http {
        /// Proxy server hostname
        host: String,
        /// Proxy server port
        port: u16,
        /// Optional authentication credentials
        auth: Option<ProxyAuth>,
    },

    /// SOCKS5 proxy
    Socks5 {
        /// Proxy server hostname
        host: String,
        /// Proxy server port
        port: u16,
        /// Optional authentication credentials
        auth: Option<ProxyAuth>,
    },
}

/// Proxy client for establishing connections through proxies
///
/// Handles connection establishment through HTTP CONNECT or SOCKS5 proxies.
pub struct ProxyClient {
    config: ProxyConfig,
}

impl ProxyClient {
    /// Create a new proxy client with the given configuration
    ///
    /// # Arguments
    ///
    /// * `config` - Proxy configuration specifying type and connection details
    ///
    /// # Examples
    ///
    /// ```
    /// use proxy_support::{ProxyClient, ProxyConfig};
    ///
    /// let config = ProxyConfig::None;
    /// let client = ProxyClient::new(config);
    /// ```
    pub fn new(config: ProxyConfig) -> Self {
        Self { config }
    }

    /// Get a reference to the proxy configuration
    ///
    /// Returns the current proxy configuration.
    pub fn config(&self) -> &ProxyConfig {
        &self.config
    }

    /// Establish a connection to the target host through the proxy
    ///
    /// Depending on the proxy configuration, this will either:
    /// - Connect directly if ProxyConfig::None
    /// - Use HTTP CONNECT method if ProxyConfig::Http
    /// - Use SOCKS5 protocol if ProxyConfig::Socks5
    ///
    /// # Arguments
    ///
    /// * `target_host` - Target hostname or IP address
    /// * `target_port` - Target port number
    ///
    /// # Returns
    ///
    /// A `TcpStream` connected to the target through the proxy (or direct if no proxy)
    ///
    /// # Errors
    ///
    /// Returns `NetworkError` if:
    /// - Proxy connection fails
    /// - Authentication fails
    /// - Target connection through proxy fails
    /// - Protocol errors occur
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use proxy_support::{ProxyClient, ProxyConfig};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = ProxyConfig::None;
    /// let client = ProxyClient::new(config);
    /// let stream = client.connect("example.com", 80).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn connect(
        &self,
        target_host: &str,
        target_port: u16,
    ) -> Result<TcpStream, NetworkError> {
        match &self.config {
            ProxyConfig::None => {
                // Direct connection
                self.direct_connect(target_host, target_port).await
            }
            ProxyConfig::Http { host, port, auth } => {
                // HTTP CONNECT proxy
                http_proxy::connect(host, *port, auth.as_ref(), target_host, target_port).await
            }
            ProxyConfig::Socks5 { host, port, auth } => {
                // SOCKS5 proxy
                socks5::connect(host, *port, auth.as_ref(), target_host, target_port).await
            }
        }
    }

    async fn direct_connect(
        &self,
        target_host: &str,
        target_port: u16,
    ) -> Result<TcpStream, NetworkError> {
        let addr = format!("{}:{}", target_host, target_port);
        TcpStream::connect(&addr)
            .await
            .map_err(|e| NetworkError::ConnectionFailed(format!("Direct connection failed: {}", e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proxy_config_variants() {
        let none = ProxyConfig::None;
        assert!(matches!(none, ProxyConfig::None));

        let http = ProxyConfig::Http {
            host: "proxy".to_string(),
            port: 8080,
            auth: None,
        };
        assert!(matches!(http, ProxyConfig::Http { .. }));

        let socks5 = ProxyConfig::Socks5 {
            host: "socks".to_string(),
            port: 1080,
            auth: None,
        };
        assert!(matches!(socks5, ProxyConfig::Socks5 { .. }));
    }
}
