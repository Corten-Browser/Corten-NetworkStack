# Corten Network Stack - API Reference

**Version:** 0.1.0
**Language:** Rust 2021
**License:** MIT OR Apache-2.0

## Table of Contents

- [Overview](#overview)
- [Core Types](#core-types)
- [Main Network Stack](#main-network-stack)
- [Protocol Clients](#protocol-clients)
- [Security Components](#security-components)
- [Core Services](#core-services)
- [Utilities](#utilities)
- [Error Handling](#error-handling)

---

## Overview

The Corten Network Stack is a comprehensive, modular network library for Rust that provides:

- **Multi-Protocol Support**: HTTP/1.1, HTTP/2, HTTP/3, WebSocket, WebRTC, FTP
- **Advanced Security**: TLS 1.2/1.3, CORS, CSP, Certificate Pinning, Mixed Content Blocking
- **Core Services**: DNS resolution (with DoH), HTTP caching, Cookie management
- **Performance**: Connection pooling, request scheduling, bandwidth limiting
- **Compliance**: Standards-compliant implementations of web specifications

---

## Core Types

### `network_types` - Fundamental Data Structures

#### `NetworkRequest`

Represents a complete HTTP request with all metadata.

```rust
pub struct NetworkRequest {
    pub url: Url,
    pub method: HttpMethod,
    pub headers: HeaderMap,
    pub body: Option<RequestBody>,
    pub mode: RequestMode,
    pub credentials: CredentialsMode,
    pub cache: CacheMode,
    pub redirect: RedirectMode,
    pub referrer: Option<String>,
    pub referrer_policy: ReferrerPolicy,
    pub integrity: Option<String>,
    pub keepalive: bool,
    pub signal: Option<AbortSignal>,
    pub priority: RequestPriority,
    pub window: Option<WindowId>,
}
```

**Key Fields:**
- `url`: Target URL for the request
- `method`: HTTP method (GET, POST, PUT, etc.)
- `headers`: HTTP headers
- `body`: Optional request body (bytes, text, form data, or stream)
- `mode`: CORS mode (Navigate, SameOrigin, NoCors, Cors)
- `credentials`: Credential handling (Omit, SameOrigin, Include)
- `cache`: Cache control mode
- `redirect`: Redirect handling mode
- `priority`: Request priority for scheduling

#### `NetworkResponse`

Represents a complete HTTP response with all metadata.

```rust
pub struct NetworkResponse {
    pub url: Url,
    pub status: u16,
    pub status_text: String,
    pub headers: HeaderMap,
    pub body: ResponseBody,
    pub redirected: bool,
    pub type_: ResponseType,
    pub timing: ResourceTiming,
}
```

**Key Fields:**
- `status`: HTTP status code
- `body`: Response body (bytes, stream, or empty)
- `redirected`: Whether the response was redirected
- `type_`: Response type (Basic, Cors, Error, Opaque, OpaqueRedirect)
- `timing`: Detailed resource timing information

#### HTTP Method Enum

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpMethod {
    Get,
    Head,
    Post,
    Put,
    Delete,
    Connect,
    Options,
    Trace,
    Patch,
}
```

#### Request Modes

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RequestMode {
    Navigate,      // Navigation requests
    SameOrigin,    // Same-origin only
    NoCors,        // Simple requests without CORS preflight
    Cors,          // Full CORS with preflight
}
```

#### Credentials Mode

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CredentialsMode {
    Omit,         // Never include credentials
    SameOrigin,   // Include for same-origin only
    Include,      // Always include credentials
}
```

#### Cache Modes

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheMode {
    Default,        // Use cache when fresh
    NoStore,        // Don't use or store cache
    Reload,         // Always fetch from origin
    NoCache,        // Validate before using cache
    ForceCache,     // Use cache even if stale
    OnlyIfCached,   // Use cache only, fail if not cached
}
```

#### Resource Timing

W3C Resource Timing specification compliant timing information.

```rust
pub struct ResourceTiming {
    pub start_time: f64,
    pub redirect_start: f64,
    pub redirect_end: f64,
    pub fetch_start: f64,
    pub domain_lookup_start: f64,
    pub domain_lookup_end: f64,
    pub connect_start: f64,
    pub connect_end: f64,
    pub secure_connection_start: f64,
    pub request_start: f64,
    pub response_start: f64,
    pub response_end: f64,
    pub transfer_size: u64,
    pub encoded_body_size: u64,
    pub decoded_body_size: u64,
}
```

---

## Main Network Stack

### `NetworkStack` Trait

The main interface for all network operations.

```rust
#[async_trait]
pub trait NetworkStack: Send + Sync {
    // HTTP Requests
    async fn fetch(&self, request: NetworkRequest) -> Result<NetworkResponse, NetworkError>;
    async fn stream_response(&self, request: NetworkRequest)
        -> Result<Pin<Box<dyn Stream<Item = Result<Bytes, NetworkError>> + Send>>, NetworkError>;

    // WebSocket
    async fn connect_websocket(&self, url: Url, protocols: Vec<String>)
        -> Result<WebSocketConnection, NetworkError>;

    // WebRTC
    async fn create_rtc_peer_connection(&self, config: RtcConfiguration)
        -> Result<RtcPeerConnection, NetworkError>;

    // Network Status
    fn get_network_status(&self) -> NetworkStatus;
    fn set_network_conditions(&mut self, conditions: NetworkConditions);

    // Cache Management
    async fn clear_cache(&mut self) -> Result<(), NetworkError>;

    // Storage Access
    fn cookie_store(&self) -> Arc<CookieStore>;
    fn cert_store(&self) -> Arc<CertificateStore>;

    // Phase 2 Features
    fn get_bandwidth_stats(&self) -> BandwidthStats;
    fn set_csp_policy(&mut self, policy: &str);
    fn set_proxy_config(&mut self, config: Option<ProxyConfig>);
    fn add_certificate_pin(&mut self, host: &str, pin_hash: Vec<u8>);
}
```

#### Usage Example

```rust
use network_stack::{NetworkStackImpl, NetworkConfig};
use network_types::{NetworkRequest, HttpMethod};
use url::Url;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create network stack with default configuration
    let config = NetworkConfig::default();
    let stack = NetworkStackImpl::new(config)?;

    // Create a request
    let mut request = NetworkRequest::default();
    request.url = Url::parse("https://example.com")?;
    request.method = HttpMethod::Get;

    // Execute the request
    let response = stack.fetch(request).await?;

    println!("Status: {}", response.status);
    println!("Headers: {:?}", response.headers);

    Ok(())
}
```

### `NetworkConfig`

Configuration for the network stack.

```rust
pub struct NetworkConfig {
    pub proxy: Option<ProxyConfig>,
    pub content_encoding: ContentEncodingConfig,
    pub request_scheduling: RequestSchedulingConfig,
    pub url_handlers: UrlHandlersConfig,
    pub mixed_content: MixedContentConfig,
    pub csp: CspConfig,
    pub certificate_transparency: CertificateTransparencyConfig,
    pub certificate_pinning: CertificatePinningConfig,
    pub platform_integration: PlatformIntegrationConfig,
}
```

### `NetworkStatus`

Information about network connectivity and performance.

```rust
pub struct NetworkStatus {
    pub online: bool,
    pub connection_type: ConnectionType,
    pub effective_type: EffectiveConnectionType,
    pub downlink: Option<f64>,  // Mbps
    pub rtt: Option<Duration>,   // Round-trip time
    pub save_data: bool,
}
```

### `NetworkConditions`

Simulate different network conditions for testing or throttling.

```rust
pub struct NetworkConditions {
    pub offline: bool,
    pub download_throughput: u64,  // bytes/sec, 0 = unlimited
    pub upload_throughput: u64,     // bytes/sec, 0 = unlimited
    pub latency: u32,                // milliseconds
}
```

---

## Protocol Clients

### HTTP/1.1 - `http1_protocol`

HTTP/1.1 client with connection pooling and keep-alive.

#### `Http1Config`

```rust
pub struct Http1Config {
    pub pool_size: usize,                      // Default: 20
    pub idle_timeout: Duration,                 // Default: 90s
    pub max_connections_per_host: usize,       // Default: 6
    pub enable_keepalive: bool,                // Default: true
    pub enable_pipelining: bool,               // Default: false
}
```

#### `ConnectionPool`

```rust
pub struct ConnectionPool;

impl ConnectionPool {
    pub fn new(config: Http1Config) -> Self;
    pub async fn get_connection(&self, host: &str, port: u16)
        -> Result<Http1Connection, NetworkError>;
    pub async fn return_connection(&self, connection: Http1Connection);
}
```

### HTTP/2 - `http2_protocol`

HTTP/2 client with multiplexing and server push support.

```rust
pub struct Http2Config {
    pub enable_push: bool,
    pub initial_window_size: u32,
    pub max_concurrent_streams: u32,
}
```

### HTTP/3 - `http3_protocol`

HTTP/3 client over QUIC.

```rust
pub struct Http3Config {
    pub max_streams: usize,
    pub quic_config: QuicConfig,
}
```

### WebSocket - `websocket_protocol`

WebSocket client with frame parsing, ping/pong, and compression extensions.

#### `WebSocketConnection`

```rust
pub struct WebSocketConnection {
    pub url: Url,
    pub protocol: Option<String>,
    pub extensions: Vec<String>,
    // ... internal channels
}

impl WebSocketConnection {
    pub async fn send(&mut self, message: WebSocketMessage)
        -> Result<(), NetworkError>;
    pub async fn receive(&mut self)
        -> Result<Option<WebSocketMessage>, NetworkError>;
    pub async fn close(&mut self, code: u16, reason: String)
        -> Result<(), NetworkError>;
    pub fn state(&self) -> WebSocketState;
}
```

#### `WebSocketMessage`

```rust
pub enum WebSocketMessage {
    Text(String),
    Binary(Vec<u8>),
    Ping(Vec<u8>),
    Pong(Vec<u8>),
    Close(Option<CloseFrame>),
}
```

#### Usage Example

```rust
let url = Url::parse("wss://echo.websocket.org")?;
let mut ws = stack.connect_websocket(url, vec![]).await?;

// Send a message
ws.send(WebSocketMessage::Text("Hello, WebSocket!".to_string())).await?;

// Receive messages
while let Ok(Some(message)) = ws.receive().await {
    match message {
        WebSocketMessage::Text(text) => println!("Received: {}", text),
        WebSocketMessage::Binary(data) => println!("Received {} bytes", data.len()),
        WebSocketMessage::Close(_) => break,
        _ => {}
    }
}
```

### WebRTC - `webrtc_peer`

WebRTC peer connections for real-time communication.

```rust
pub struct RtcConfiguration {
    pub ice_servers: Vec<IceServer>,
    pub ice_transport_policy: IceTransportPolicy,
    pub bundle_policy: BundlePolicy,
}

pub struct RtcPeerConnection {
    // ... internal state
}

impl RtcPeerConnection {
    pub async fn create_offer(&self) -> Result<SessionDescription, NetworkError>;
    pub async fn create_answer(&self) -> Result<SessionDescription, NetworkError>;
    pub async fn set_local_description(&mut self, desc: SessionDescription)
        -> Result<(), NetworkError>;
    pub async fn set_remote_description(&mut self, desc: SessionDescription)
        -> Result<(), NetworkError>;
    pub async fn add_ice_candidate(&mut self, candidate: IceCandidate)
        -> Result<(), NetworkError>;
}
```

---

## Security Components

### TLS Manager - `tls_manager`

TLS 1.2/1.3 configuration, certificate validation, ALPN negotiation.

#### `TlsConfig`

```rust
pub struct TlsConfig {
    alpn_protocols: Vec<Vec<u8>>,
    root_cert_store: Option<RootCertStore>,
}

impl TlsConfig {
    pub fn new() -> Self;
    pub fn with_alpn_protocols(mut self, protocols: Vec<Vec<u8>>) -> Self;
    pub fn with_root_certificates(mut self, certs: RootCertStore) -> Self;
    pub fn alpn_protocols(&self) -> &[Vec<u8>];
    pub fn root_cert_store(&self) -> Option<&RootCertStore>;
}
```

**Example:**

```rust
let config = TlsConfig::new()
    .with_alpn_protocols(vec![
        b"h3".to_vec(),      // HTTP/3
        b"h2".to_vec(),      // HTTP/2
        b"http/1.1".to_vec() // HTTP/1.1
    ]);
```

#### `CertificateStore`

```rust
pub struct CertificateStore;

impl CertificateStore {
    pub fn new() -> Self;
    pub fn add_certificate(&mut self, cert: Vec<u8>) -> Result<(), NetworkError>;
    pub async fn verify_certificate(&self, cert: &[u8], hostname: &str)
        -> Result<(), NetworkError>;
    pub fn certificate_count(&self) -> usize;
}
```

#### `HstsStore`

HTTP Strict Transport Security (HSTS) policy management.

```rust
pub struct HstsStore;

impl HstsStore {
    pub fn new() -> Self;
    pub fn is_hsts_enabled(&self, domain: &str) -> bool;
    pub fn add_hsts_entry(&mut self, domain: String, max_age: Duration, include_subdomains: bool);
}
```

### CORS Validator - `cors_validator`

Cross-Origin Resource Sharing (CORS) validation and enforcement.

#### `CorsValidator`

```rust
pub struct CorsValidator;

impl CorsValidator {
    pub fn new(config: CorsConfig) -> Self;
    pub fn validate_request(&self, request: &NetworkRequest, origin: &str)
        -> CorsResult;
    pub fn validate_response(&self, response: &NetworkResponse, request: &NetworkRequest)
        -> CorsResult;
    pub fn needs_preflight(&self, request: &NetworkRequest) -> bool;
}
```

#### `CorsResult`

```rust
pub struct CorsResult {
    pub allowed: bool,
    pub reason: Option<String>,
    pub headers_to_add: HeaderMap,
}
```

#### `CorsConfig`

```rust
pub struct CorsConfig {
    pub allowed_origins: Vec<String>,
    pub allowed_methods: Vec<HttpMethod>,
    pub allowed_headers: Vec<String>,
    pub expose_headers: Vec<String>,
    pub max_age: Option<Duration>,
    pub allow_credentials: bool,
}
```

### Certificate Pinning - `certificate_pinning`

Public key pinning for enhanced security.

```rust
pub struct CertificatePinner {
    pins: HashMap<String, Vec<Vec<u8>>>,
}

impl CertificatePinner {
    pub fn new() -> Self;
    pub fn add_pin(&mut self, host: String, pin_hash: Vec<u8>);
    pub fn validate(&self, host: &str, cert_chain: &[Vec<u8>])
        -> Result<(), NetworkError>;
}
```

### CSP Processor - `csp_processor`

Content Security Policy processing and enforcement.

```rust
pub struct CspProcessor {
    policy: CspPolicy,
}

impl CspProcessor {
    pub fn new(policy_string: &str) -> Result<Self, NetworkError>;
    pub fn allows_script(&self, source: &str, nonce: Option<&str>) -> bool;
    pub fn allows_style(&self, source: &str) -> bool;
    pub fn allows_image(&self, source: &str) -> bool;
    pub fn allows_connect(&self, source: &str) -> bool;
}
```

---

## Core Services

### DNS Resolver - `dns_resolver`

DNS resolution with DNS-over-HTTPS (DoH) support and caching.

#### `DnsResolver` Trait

```rust
#[async_trait]
pub trait DnsResolver: Send + Sync {
    async fn resolve(&self, hostname: String) -> NetworkResult<Vec<IpAddr>>;
    async fn resolve_with_timeout(&self, hostname: String, timeout: Duration)
        -> NetworkResult<Vec<IpAddr>>;
}
```

#### `StandardResolver`

```rust
pub struct StandardResolver;

impl StandardResolver {
    pub fn new(doh_config: Option<DohConfig>) -> Result<Self, NetworkError>;
}
```

#### `DohConfig`

DNS-over-HTTPS configuration.

```rust
pub struct DohConfig {
    pub enabled: bool,
    pub resolver_url: String,  // e.g., "https://dns.google/dns-query"
}
```

#### `DnsCache`

```rust
pub struct DnsCache;

impl DnsCache {
    pub fn new() -> Self;
    pub fn get(&self, hostname: &str) -> Option<Vec<IpAddr>>;
    pub fn insert(&mut self, hostname: String, addresses: Vec<IpAddr>, ttl: Duration);
    pub fn clear_expired(&mut self);
}
```

**Example:**

```rust
use dns_resolver::{StandardResolver, DohConfig, DnsResolver};

let doh_config = DohConfig {
    enabled: true,
    resolver_url: "https://dns.google/dns-query".to_string(),
};

let resolver = StandardResolver::new(Some(doh_config))?;
let addresses = resolver.resolve("example.com".to_string()).await?;

for addr in addresses {
    println!("Resolved: {}", addr);
}
```

### Cookie Manager - `cookie_manager`

HTTP cookie management with SameSite support.

```rust
pub struct CookieStore;

impl CookieStore {
    pub fn new() -> Self;
    pub fn add_cookie(&mut self, cookie: Cookie, url: &Url) -> Result<(), NetworkError>;
    pub fn get_cookies(&self, url: &Url) -> Vec<Cookie>;
    pub fn clear(&mut self);
    pub fn clear_for_domain(&mut self, domain: &str);
}
```

### HTTP Cache - `http_cache`

HTTP caching with Cache-Control and ETag support.

```rust
pub struct HttpCache;

impl HttpCache {
    pub fn new(config: CacheConfig) -> Self;
    pub async fn get(&self, request: &NetworkRequest)
        -> Option<NetworkResponse>;
    pub async fn put(&mut self, request: &NetworkRequest, response: NetworkResponse);
    pub async fn clear(&mut self);
    pub fn size(&self) -> usize;
}

pub struct CacheConfig {
    pub max_size: usize,      // bytes
    pub max_age: Duration,
}
```

### Content Encoding - `content_encoding`

Content encoding/decoding (gzip, deflate, brotli).

```rust
pub struct ContentEncoder;

impl ContentEncoder {
    pub fn encode(&self, data: &[u8], encoding: Encoding)
        -> Result<Vec<u8>, NetworkError>;
    pub fn decode(&self, data: &[u8], encoding: Encoding)
        -> Result<Vec<u8>, NetworkError>;
}

pub enum Encoding {
    Gzip,
    Deflate,
    Brotli,
    Identity,
}
```

---

## Utilities

### Proxy Support - `proxy_support`

HTTP and SOCKS5 proxy configuration.

```rust
pub struct ProxyConfig {
    pub proxy_type: ProxyType,
    pub host: String,
    pub port: u16,
    pub auth: Option<ProxyAuth>,
}

pub enum ProxyType {
    Http,
    Https,
    Socks5,
}

pub struct ProxyAuth {
    pub username: String,
    pub password: String,
}
```

### Request Scheduler - `request_scheduler`

Request prioritization and scheduling.

```rust
pub struct RequestScheduler;

impl RequestScheduler {
    pub fn new(config: RequestSchedulingConfig) -> Self;
    pub async fn schedule(&mut self, request: NetworkRequest)
        -> Result<NetworkResponse, NetworkError>;
}

pub struct RequestSchedulingConfig {
    pub max_concurrent_requests: usize,
    pub max_requests_per_host: usize,
    pub priority_levels: usize,
}
```

### Bandwidth Limiter - `bandwidth_limiter`

Bandwidth throttling and statistics.

```rust
pub struct BandwidthLimiter;

impl BandwidthLimiter {
    pub fn new(download_limit: u64, upload_limit: u64) -> Self;
    pub fn get_stats(&self) -> BandwidthStats;
    pub async fn throttle_read(&mut self, size: usize);
    pub async fn throttle_write(&mut self, size: usize);
}

pub struct BandwidthStats {
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub current_download_rate: f64,  // bytes/sec
    pub current_upload_rate: f64,     // bytes/sec
}
```

### URL Handlers - `url_handlers`

Special URL scheme handlers (data:, file:, etc.).

```rust
pub trait UrlHandler: Send + Sync {
    fn can_handle(&self, url: &Url) -> bool;
    async fn handle(&self, url: &Url) -> Result<NetworkResponse, NetworkError>;
}

pub struct DataUrlHandler;  // Handles data: URLs
pub struct FileUrlHandler;  // Handles file: URLs
```

---

## Error Handling

### `NetworkError`

Comprehensive error type for all network operations.

```rust
#[derive(Debug, Error)]
pub enum NetworkError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("DNS resolution failed: {0}")]
    DnsError(String),

    #[error("TLS error: {0}")]
    TlsError(String),

    #[error("Protocol error: {0}")]
    ProtocolError(String),

    #[error("Timeout after {0:?}")]
    Timeout(Duration),

    #[error("Request aborted")]
    Aborted,

    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    #[error("Too many redirects")]
    TooManyRedirects,

    #[error("Cache error: {0}")]
    CacheError(String),

    #[error("Proxy error: {0}")]
    ProxyError(String),

    #[error("CORS violation: {0}")]
    CorsError(String),

    #[error("Mixed content blocked")]
    MixedContent,

    #[error("Certificate validation failed: {0}")]
    CertificateError(String),

    #[error("WebSocket error: {0}")]
    WebSocketError(String),

    #[error("WebRTC error: {0}")]
    WebRtcError(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Other error: {0}")]
    Other(String),
}

pub type NetworkResult<T> = Result<T, NetworkError>;
```

**Error Handling Example:**

```rust
use network_errors::{NetworkError, NetworkResult};

async fn fetch_data(url: &str) -> NetworkResult<String> {
    let request = create_request(url)?;
    let response = stack.fetch(request).await?;

    if response.status != 200 {
        return Err(NetworkError::ProtocolError(
            format!("HTTP {}", response.status)
        ));
    }

    // Process response...
    Ok(data)
}

// Usage
match fetch_data("https://example.com").await {
    Ok(data) => println!("Success: {}", data),
    Err(NetworkError::Timeout(d)) => eprintln!("Timeout after {:?}", d),
    Err(NetworkError::DnsError(msg)) => eprintln!("DNS failed: {}", msg),
    Err(e) => eprintln!("Error: {}", e),
}
```

---

## See Also

- [Quick Start Guide](QUICK-START.md) - Get started quickly
- [Examples](../examples/) - Working code examples
- [Generated rustdoc](../target/doc/network_stack/index.html) - Complete API documentation

---

**Generated:** 2025-11-15
**Documentation Version:** 0.1.0
