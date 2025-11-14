//! Network stack configuration
//!
//! Defines configuration structures for all network protocols and features.

use std::time::Duration;

/// Network stack configuration
///
/// Aggregates configuration for all network protocols and features.
#[derive(Debug, Clone)]
pub struct NetworkConfig {
    /// HTTP/1.1 configuration
    pub http1: Option<http1_protocol::Http1Config>,

    /// HTTP/2 configuration
    pub http2: Option<http2_protocol::Http2Config>,

    /// HTTP/3 configuration
    pub http3: Option<http3_protocol::Http3Config>,

    /// WebSocket configuration
    pub websocket: Option<WebSocketConfig>,

    /// WebRTC configuration
    pub webrtc: Option<WebRtcConfig>,

    /// HTTP cache configuration
    pub cache: Option<CacheConfig>,

    /// Security/TLS configuration
    pub security: Option<SecurityConfig>,

    /// Proxy configuration
    pub proxy: Option<ProxyConfig>,

    /// DNS configuration
    pub dns: Option<dns_resolver::DnsConfig>,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            http1: Some(http1_protocol::Http1Config::default()),
            http2: Some(http2_protocol::Http2Config::default()),
            http3: None, // HTTP/3 disabled by default
            websocket: Some(WebSocketConfig::default()),
            webrtc: Some(WebRtcConfig::default()),
            cache: Some(CacheConfig::default()),
            security: Some(SecurityConfig::default()),
            proxy: None,
            dns: Some(dns_resolver::DnsConfig::default()),
        }
    }
}

/// WebSocket client configuration
#[derive(Debug, Clone)]
pub struct WebSocketConfig {
    /// Maximum message size (bytes)
    pub max_message_size: usize,

    /// Connection timeout
    pub connect_timeout: Duration,

    /// Enable message compression
    pub enable_compression: bool,
}

impl Default for WebSocketConfig {
    fn default() -> Self {
        Self {
            max_message_size: 64 * 1024 * 1024, // 64 MB
            connect_timeout: Duration::from_secs(30),
            enable_compression: true,
        }
    }
}

/// WebRTC configuration
#[derive(Debug, Clone)]
pub struct WebRtcConfig {
    /// Maximum number of peer connections
    pub max_peer_connections: usize,

    /// Enable STUN/TURN
    pub enable_ice: bool,
}

impl Default for WebRtcConfig {
    fn default() -> Self {
        Self {
            max_peer_connections: 100,
            enable_ice: true,
        }
    }
}

/// HTTP cache configuration
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Maximum cache size (bytes)
    pub max_size: u64,

    /// Enable caching
    pub enabled: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_size: 100 * 1024 * 1024, // 100 MB
            enabled: true,
        }
    }
}

/// Security and TLS configuration
#[derive(Debug, Clone)]
pub struct SecurityConfig {
    /// Verify TLS certificates
    pub verify_certificates: bool,

    /// Enable certificate transparency
    pub enable_ct: bool,

    /// Enable HSTS
    pub enable_hsts: bool,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            verify_certificates: true,
            enable_ct: true,
            enable_hsts: true,
        }
    }
}

/// Proxy configuration
#[derive(Debug, Clone)]
pub struct ProxyConfig {
    /// Proxy URL (http:// or socks5://)
    pub url: String,

    /// Proxy authentication (if required)
    pub auth: Option<ProxyAuth>,
}

/// Proxy authentication credentials
#[derive(Debug, Clone)]
pub struct ProxyAuth {
    /// Username
    pub username: String,

    /// Password
    pub password: String,
}
