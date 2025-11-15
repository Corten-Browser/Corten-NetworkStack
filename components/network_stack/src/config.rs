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
    pub dns: Option<dns_resolver::DohConfig>,

    // Phase 2 configurations
    /// CORS validation configuration
    pub cors: Option<cors_validator::CorsConfig>,

    /// Content encoding configuration (enabled by default)
    pub content_encoding: Option<ContentEncodingConfig>,

    /// Request scheduling configuration
    pub request_scheduling: Option<RequestSchedulingConfig>,

    /// Bandwidth limiting configuration
    pub bandwidth_limit: Option<u64>,

    /// URL handlers configuration
    pub url_handlers: Option<UrlHandlersConfig>,

    /// Mixed content blocking configuration
    pub mixed_content: Option<MixedContentConfig>,

    /// CSP (Content Security Policy) configuration
    pub csp: Option<CspConfig>,

    /// Certificate transparency configuration
    pub certificate_transparency: Option<CertificateTransparencyConfig>,

    /// Certificate pinning configuration
    pub certificate_pinning: Option<CertificatePinningConfig>,

    /// Platform integration configuration
    pub platform_integration: Option<PlatformIntegrationConfig>,

    /// FTP protocol configuration
    pub ftp: Option<ftp_protocol::FtpConfig>,
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
            dns: Some(dns_resolver::DohConfig::default()),
            // Phase 2 defaults
            cors: Some(cors_validator::CorsConfig::default()),
            content_encoding: Some(ContentEncodingConfig::default()),
            request_scheduling: Some(RequestSchedulingConfig::default()),
            bandwidth_limit: None, // No bandwidth limit by default
            url_handlers: Some(UrlHandlersConfig::default()),
            mixed_content: Some(MixedContentConfig::default()),
            csp: None, // No CSP policy by default
            certificate_transparency: Some(CertificateTransparencyConfig::default()),
            certificate_pinning: Some(CertificatePinningConfig::default()),
            platform_integration: Some(PlatformIntegrationConfig::default()),
            ftp: Some(ftp_protocol::FtpConfig::default()),
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

// Phase 2 Configuration Structs

/// Content encoding configuration
#[derive(Debug, Clone)]
pub struct ContentEncodingConfig {
    /// Enable content encoding/decoding
    pub enabled: bool,
}

impl Default for ContentEncodingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
        }
    }
}

/// Request scheduling configuration
#[derive(Debug, Clone)]
pub struct RequestSchedulingConfig {
    /// Enable request scheduling
    pub enabled: bool,
    /// Maximum concurrent requests
    pub max_concurrent: usize,
}

impl Default for RequestSchedulingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_concurrent: 6,
        }
    }
}

/// URL handlers configuration
#[derive(Debug, Clone)]
pub struct UrlHandlersConfig {
    /// Enable data: URL handler
    pub enable_data_urls: bool,
    /// Enable file: URL handler
    pub enable_file_urls: bool,
    /// Allowed file paths for file: URLs
    pub allowed_file_paths: Vec<std::path::PathBuf>,
}

impl Default for UrlHandlersConfig {
    fn default() -> Self {
        Self {
            enable_data_urls: true,
            enable_file_urls: false, // Disabled by default for security
            allowed_file_paths: vec![],
        }
    }
}

/// Mixed content blocking configuration
#[derive(Debug, Clone)]
pub struct MixedContentConfig {
    /// Block all mixed content (both active and passive)
    pub block_all_mixed_content: bool,
    /// Enable upgrade-insecure-requests
    pub upgrade_insecure_requests: bool,
}

impl Default for MixedContentConfig {
    fn default() -> Self {
        Self {
            block_all_mixed_content: true,
            upgrade_insecure_requests: false,
        }
    }
}

/// CSP configuration
#[derive(Debug, Clone)]
pub struct CspConfig {
    /// Default CSP policy
    pub policy: Option<String>,
    /// Enable CSP enforcement
    pub enabled: bool,
}

impl Default for CspConfig {
    fn default() -> Self {
        Self {
            policy: None,
            enabled: false,
        }
    }
}

/// Certificate transparency configuration
#[derive(Debug, Clone)]
pub struct CertificateTransparencyConfig {
    /// Require SCT (Signed Certificate Timestamp)
    pub require_sct: bool,
    /// Minimum number of SCTs required
    pub min_sct_count: usize,
}

impl Default for CertificateTransparencyConfig {
    fn default() -> Self {
        Self {
            require_sct: true,
            min_sct_count: 2,
        }
    }
}

/// Certificate pinning configuration
#[derive(Debug, Clone)]
pub struct CertificatePinningConfig {
    /// Enable certificate pinning
    pub enabled: bool,
}

impl Default for CertificatePinningConfig {
    fn default() -> Self {
        Self {
            enabled: false,
        }
    }
}

/// Platform integration configuration
#[derive(Debug, Clone)]
pub struct PlatformIntegrationConfig {
    /// Enable system proxy detection
    pub use_system_proxy: bool,
    /// Enable system certificate store
    pub use_system_certs: bool,
}

impl Default for PlatformIntegrationConfig {
    fn default() -> Self {
        Self {
            use_system_proxy: true,
            use_system_certs: true,
        }
    }
}
