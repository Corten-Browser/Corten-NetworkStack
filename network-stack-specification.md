# Network Stack Component Specification
## CortenBrowser Network Stack v1.1

### Component Overview

**Component Name**: network-stack  
**Project**: CortenBrowser (formerly codename CortenBrowser)
**Component Type**: Core Component  
**Current Implementation**: reqwest + hyper  
**Target Implementation**: Enhanced multi-protocol network stack with full HTTP/2, HTTP/3, WebSocket, and WebRTC support  
**Estimated Lines of Code**: 75,000-100,000  
**Development Priority**: Phase 1 (Basic), Phase 4 (Full implementation)  
**Context Window Fit**: Must be partitioned into sub-modules to fit 170,000 token limit  

### Purpose and Responsibilities

The Network Stack component is responsible for all network communication in the browser, providing:

1. **Protocol Support**
   - HTTP/1.1, HTTP/2, HTTP/3 (QUIC)
   - WebSocket (WS/WSS)
   - WebRTC data channels and media transport
   - FTP (basic support)
   - Data URLs
   - File URLs

2. **Core Features**
   - Connection pooling and multiplexing
   - Request/response streaming
   - Certificate validation and pinning
   - Cookie management
   - Cache implementation (HTTP cache)
   - Proxy support (HTTP, SOCKS5)
   - DNS resolution with DoH support
   - Request prioritization and scheduling
   - Bandwidth throttling
   - CORS enforcement
   - Content encoding/decoding (gzip, br, deflate)

3. **Security Features**
   - TLS 1.2/1.3 support
   - Certificate transparency validation
   - HSTS enforcement
   - Mixed content blocking
   - CSP header processing
   - Secure context enforcement

### Interface Specification

#### Public API

```rust
use std::sync::Arc;
use tokio::sync::mpsc;
use futures::stream::Stream;

/// Main Network Stack component interface
#[async_trait::async_trait]
pub trait NetworkStack: BrowserComponent {
    /// Initiate an HTTP request
    async fn fetch(&self, request: NetworkRequest) -> Result<NetworkResponse, NetworkError>;
    
    /// Stream response body chunks
    async fn stream_response(&self, request: NetworkRequest) 
        -> Result<Box<dyn Stream<Item = Result<Bytes, NetworkError>> + Send>, NetworkError>;
    
    /// Open a WebSocket connection
    async fn connect_websocket(&self, url: Url, protocols: Vec<String>) 
        -> Result<WebSocketConnection, NetworkError>;
    
    /// Initialize a WebRTC peer connection
    async fn create_rtc_peer_connection(&self, config: RtcConfiguration) 
        -> Result<RtcPeerConnection, NetworkError>;
    
    /// Get current network status
    fn get_network_status(&self) -> NetworkStatus;
    
    /// Set network conditions (for throttling/simulation)
    fn set_network_conditions(&mut self, conditions: NetworkConditions);
    
    /// Clear all cached data
    async fn clear_cache(&mut self) -> Result<(), NetworkError>;
    
    /// Get cookie store handle
    fn cookie_store(&self) -> Arc<CookieStore>;
    
    /// Get certificate store handle  
    fn cert_store(&self) -> Arc<CertificateStore>;
}

/// Network request structure
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// Network response structure
#[derive(Debug)]
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

/// Request body types
#[derive(Debug, Clone)]
pub enum RequestBody {
    Bytes(Vec<u8>),
    Text(String),
    FormData(FormData),
    Stream(BodyStream),
}

/// Response body types
#[derive(Debug)]
pub enum ResponseBody {
    Bytes(Vec<u8>),
    Stream(Box<dyn Stream<Item = Result<Bytes, NetworkError>> + Send>),
    Empty,
}

/// HTTP methods
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

/// Request modes (CORS)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RequestMode {
    Navigate,
    SameOrigin,
    NoCors,
    Cors,
}

/// Credentials mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CredentialsMode {
    Omit,
    SameOrigin,
    Include,
}

/// Cache modes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheMode {
    Default,
    NoStore,
    Reload,
    NoCache,
    ForceCache,
    OnlyIfCached,
}

/// Resource timing information
#[derive(Debug, Clone)]
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

#### WebSocket API

```rust
/// WebSocket connection handle
pub struct WebSocketConnection {
    pub url: Url,
    pub protocol: Option<String>,
    pub extensions: Vec<String>,
    sender: mpsc::Sender<WebSocketMessage>,
    receiver: mpsc::Receiver<WebSocketMessage>,
}

impl WebSocketConnection {
    /// Send a message through the WebSocket
    pub async fn send(&self, message: WebSocketMessage) -> Result<(), NetworkError>;
    
    /// Receive next message from the WebSocket
    pub async fn recv(&mut self) -> Option<Result<WebSocketMessage, NetworkError>>;
    
    /// Close the WebSocket connection
    pub async fn close(&self, code: u16, reason: String) -> Result<(), NetworkError>;
    
    /// Get connection state
    pub fn state(&self) -> WebSocketState;
}

/// WebSocket message types
#[derive(Debug, Clone)]
pub enum WebSocketMessage {
    Text(String),
    Binary(Vec<u8>),
    Ping(Vec<u8>),
    Pong(Vec<u8>),
    Close(Option<CloseFrame>),
}

/// WebSocket states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WebSocketState {
    Connecting,
    Open,
    Closing,
    Closed,
}
```

#### WebRTC API

```rust
/// WebRTC peer connection
pub struct RtcPeerConnection {
    pub connection_id: Uuid,
    configuration: RtcConfiguration,
    local_description: Option<SessionDescription>,
    remote_description: Option<SessionDescription>,
}

impl RtcPeerConnection {
    /// Create a new data channel
    pub async fn create_data_channel(&self, label: &str, options: DataChannelOptions) 
        -> Result<RtcDataChannel, NetworkError>;
    
    /// Add ICE candidate
    pub async fn add_ice_candidate(&mut self, candidate: IceCandidate) 
        -> Result<(), NetworkError>;
    
    /// Create offer
    pub async fn create_offer(&self, options: OfferOptions) 
        -> Result<SessionDescription, NetworkError>;
    
    /// Create answer
    pub async fn create_answer(&self, options: AnswerOptions) 
        -> Result<SessionDescription, NetworkError>;
    
    /// Set local description
    pub async fn set_local_description(&mut self, description: SessionDescription) 
        -> Result<(), NetworkError>;
    
    /// Set remote description
    pub async fn set_remote_description(&mut self, description: SessionDescription) 
        -> Result<(), NetworkError>;
    
    /// Get stats
    pub async fn get_stats(&self) -> Result<RtcStats, NetworkError>;
}
```

### Message Bus Integration

```rust
/// Messages this component handles
pub enum NetworkStackMessage {
    /// Request to fetch a resource
    FetchRequest {
        request_id: u64,
        request: NetworkRequest,
        response_channel: oneshot::Sender<NetworkResponse>,
    },
    
    /// WebSocket connection request
    WebSocketConnect {
        request_id: u64,
        url: Url,
        protocols: Vec<String>,
        response_channel: oneshot::Sender<WebSocketConnection>,
    },
    
    /// Clear cache request
    ClearCache {
        response_channel: oneshot::Sender<Result<(), NetworkError>>,
    },
    
    /// Network status query
    GetNetworkStatus {
        response_channel: oneshot::Sender<NetworkStatus>,
    },
    
    /// Update network conditions (for DevTools throttling)
    SetNetworkConditions {
        conditions: NetworkConditions,
    },
}

/// Messages this component emits
pub enum NetworkStackEvent {
    /// Network status changed
    NetworkStatusChanged(NetworkStatus),
    
    /// Security warning
    SecurityWarning {
        url: Url,
        warning_type: SecurityWarningType,
        details: String,
    },
    
    /// Performance metrics
    PerformanceMetrics {
        metrics: NetworkMetrics,
    },
}
```

### Internal Architecture

#### Module Structure

```
network-stack/
├── Cargo.toml
├── src/
│   ├── lib.rs                 # Component trait implementation
│   ├── http/
│   │   ├── mod.rs
│   │   ├── client.rs          # HTTP client implementation
│   │   ├── http1.rs           # HTTP/1.1 protocol
│   │   ├── http2.rs           # HTTP/2 protocol
│   │   ├── http3.rs           # HTTP/3 (QUIC) protocol
│   │   ├── connection_pool.rs # Connection pooling
│   │   └── stream.rs          # Response streaming
│   ├── websocket/
│   │   ├── mod.rs
│   │   ├── client.rs          # WebSocket client
│   │   ├── protocol.rs        # WebSocket protocol handler
│   │   └── frame.rs           # Frame parsing/encoding
│   ├── webrtc/
│   │   ├── mod.rs
│   │   ├── peer_connection.rs # Peer connection implementation
│   │   ├── data_channel.rs    # Data channel implementation
│   │   ├── ice.rs             # ICE handling
│   │   └── sdp.rs             # SDP parsing/generation
│   ├── cache/
│   │   ├── mod.rs
│   │   ├── http_cache.rs      # HTTP cache implementation
│   │   ├── storage.rs         # Cache storage backend
│   │   └── policy.rs          # Cache policy enforcement
│   ├── cookies/
│   │   ├── mod.rs
│   │   ├── store.rs           # Cookie storage
│   │   ├── jar.rs             # Cookie jar implementation
│   │   └── parser.rs          # Cookie parsing
│   ├── dns/
│   │   ├── mod.rs
│   │   ├── resolver.rs        # DNS resolver
│   │   ├── cache.rs           # DNS cache
│   │   └── doh.rs             # DNS-over-HTTPS
│   ├── security/
│   │   ├── mod.rs
│   │   ├── tls.rs             # TLS configuration
│   │   ├── certificates.rs    # Certificate validation
│   │   ├── hsts.rs            # HSTS enforcement
│   │   └── cors.rs            # CORS validation
│   ├── proxy/
│   │   ├── mod.rs
│   │   ├── http_proxy.rs      # HTTP proxy support
│   │   └── socks.rs           # SOCKS5 proxy support
│   └── utils/
│       ├── mod.rs
│       ├── url.rs             # URL utilities
│       ├── headers.rs         # Header utilities
│       └── encoding.rs        # Content encoding/decoding
└── tests/
    ├── integration/
    ├── unit/
    └── fixtures/
```

#### Core Components Design

##### HTTP Client Architecture

```rust
/// Main HTTP client implementation
pub struct HttpClient {
    /// Connection pool for HTTP/1.1 and HTTP/2
    connection_pool: Arc<ConnectionPool>,
    
    /// HTTP/3 connection manager
    http3_manager: Arc<Http3Manager>,
    
    /// Request scheduler
    scheduler: Arc<RequestScheduler>,
    
    /// Cache manager
    cache: Arc<HttpCache>,
    
    /// Cookie store
    cookies: Arc<CookieStore>,
    
    /// Security manager
    security: Arc<SecurityManager>,
    
    /// DNS resolver
    resolver: Arc<DnsResolver>,
    
    /// Proxy configuration
    proxy_config: Arc<RwLock<ProxyConfig>>,
    
    /// Network conditions (for throttling)
    network_conditions: Arc<RwLock<NetworkConditions>>,
}

/// Connection pool for connection reuse
pub struct ConnectionPool {
    /// Active HTTP/1.1 connections
    http1_connections: Arc<RwLock<HashMap<PoolKey, Vec<Http1Connection>>>>,
    
    /// Active HTTP/2 connections
    http2_connections: Arc<RwLock<HashMap<PoolKey, Http2Connection>>>,
    
    /// Maximum connections per host
    max_connections_per_host: usize,
    
    /// Idle timeout
    idle_timeout: Duration,
    
    /// Connection limits
    limits: ConnectionLimits,
}

/// Request scheduler for prioritization
pub struct RequestScheduler {
    /// High priority queue (navigation, CSS, fonts)
    high_priority: Arc<Mutex<VecDeque<PendingRequest>>>,
    
    /// Medium priority queue (scripts, XHR)
    medium_priority: Arc<Mutex<VecDeque<PendingRequest>>>,
    
    /// Low priority queue (images, prefetch)
    low_priority: Arc<Mutex<VecDeque<PendingRequest>>>,
    
    /// Active requests
    active_requests: Arc<RwLock<HashMap<RequestId, ActiveRequest>>>,
    
    /// Maximum concurrent requests
    max_concurrent: usize,
}
```

##### WebSocket Implementation

```rust
/// WebSocket client manager
pub struct WebSocketManager {
    /// Active connections
    connections: Arc<RwLock<HashMap<Uuid, WebSocketConnectionImpl>>>,
    
    /// TLS configuration
    tls_config: Arc<TlsConfig>,
    
    /// Frame codec
    codec: WebSocketCodec,
}

/// Internal WebSocket connection
struct WebSocketConnectionImpl {
    /// Underlying TCP/TLS stream
    stream: WebSocketStream,
    
    /// Send channel
    tx: mpsc::Sender<WebSocketMessage>,
    
    /// Receive channel
    rx: mpsc::Receiver<WebSocketMessage>,
    
    /// Connection state
    state: Arc<RwLock<WebSocketState>>,
    
    /// Ping/pong handler
    heartbeat: Option<JoinHandle<()>>,
}
```

##### WebRTC Implementation

```rust
/// WebRTC manager
pub struct WebRtcManager {
    /// Active peer connections
    connections: Arc<RwLock<HashMap<Uuid, PeerConnectionImpl>>>,
    
    /// ICE servers configuration
    ice_servers: Vec<IceServer>,
    
    /// STUN/TURN client
    ice_agent: Arc<IceAgent>,
    
    /// DTLS configuration
    dtls_config: Arc<DtlsConfig>,
}

/// Internal peer connection
struct PeerConnectionImpl {
    /// Connection ID
    id: Uuid,
    
    /// ICE transport
    ice_transport: Arc<IceTransport>,
    
    /// DTLS transport
    dtls_transport: Arc<DtlsTransport>,
    
    /// SCTP transport (for data channels)
    sctp_transport: Option<Arc<SctpTransport>>,
    
    /// Data channels
    data_channels: Arc<RwLock<HashMap<u16, DataChannelImpl>>>,
    
    /// Connection state
    state: Arc<RwLock<PeerConnectionState>>,
}
```

### External Dependencies

```toml
[dependencies]
# Core async runtime
tokio = { version = "1.35", features = ["full"] }
async-trait = "0.1"

# HTTP implementation
hyper = { version = "1.0", features = ["full"] }
hyper-util = { version = "0.1", features = ["full"] }
http = "1.0"
http-body = "1.0"
http-body-util = "0.1"

# HTTP/2
h2 = "0.4"

# HTTP/3 and QUIC
quinn = "0.10"
h3 = "0.2"
h3-quinn = "0.0.4"

# WebSocket
tokio-tungstenite = "0.21"
tungstenite = "0.21"

# WebRTC
webrtc = "0.9"
webrtc-ice = "0.9"
webrtc-dtls = "0.7"
webrtc-sctp = "0.7"
webrtc-srtp = "0.11"
webrtc-mdns = "0.5"
webrtc-media = "0.5"

# TLS
rustls = "0.22"
rustls-native-certs = "0.7"
webpki-roots = "0.26"
tokio-rustls = "0.25"

# DNS
hickory-resolver = { version = "0.24", features = ["dns-over-https-rustls"] }

# Cookies
cookie_store = "0.20"
cookie = "0.18"

# URL parsing
url = "2.5"

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
bincode = "1.3"

# Compression
flate2 = "1.0"
brotli = "3.4"

# Caching
cached = "0.47"
lru = "0.12"

# Utilities
bytes = "1.5"
futures = "0.3"
pin-project = "1.1"
tower = { version = "0.4", features = ["full"] }
tower-service = "0.3"
tracing = "0.1"
thiserror = "1.0"
anyhow = "1.0"
uuid = { version = "1.6", features = ["v4", "serde"] }
chrono = "0.4"
base64 = "0.21"
percent-encoding = "2.3"
mime = "0.3"
mime_guess = "2.0"

# Platform-specific
[target.'cfg(windows)'.dependencies]
windows = { version = "0.52", features = ["Win32_Networking_WinInet"] }

[target.'cfg(unix)'.dependencies]
libc = "0.2"

[dev-dependencies]
mockito = "1.2"
wiremock = "0.5"
criterion = "0.5"
proptest = "1.4"
test-case = "3.1"

# WPT test harness support
wpt-runner = { path = "../wpt-runner" }  # Local harness implementation
serde_json = "1.0"
tempfile = "3.8"

# Performance testing
flame = "0.2"  # Flame graphs for performance analysis
pprof = "0.13"  # CPU profiling

# Test data generation
quickcheck = "1.0"
arbitrary = "1.3"
```

### Build Configuration

```toml
[package]
name = "browser-network-stack"
version = "0.1.0"
edition = "2021"
rust-version = "1.75"

[features]
default = ["http2", "http3", "websocket", "webrtc"]
http2 = ["dep:h2"]
http3 = ["dep:quinn", "dep:h3", "dep:h3-quinn"]
websocket = ["dep:tokio-tungstenite", "dep:tungstenite"]
webrtc = ["dep:webrtc", "dep:webrtc-ice", "dep:webrtc-dtls", "dep:webrtc-sctp"]
experimental = ["http3"]
testing = ["mockito", "wiremock"]

[profile.release]
opt-level = 3
lto = true
codegen-units = 1

[profile.dev]
opt-level = 0
debug = true
```

### State Management

#### Connection State

```rust
/// Global network state
pub struct NetworkState {
    /// Online/offline status
    online: AtomicBool,
    
    /// Connection type (WiFi, Ethernet, Cellular)
    connection_type: Arc<RwLock<ConnectionType>>,
    
    /// Effective connection type (slow-2g, 2g, 3g, 4g)
    effective_type: Arc<RwLock<EffectiveConnectionType>>,
    
    /// Downlink bandwidth (Mbps)
    downlink: AtomicU32,
    
    /// RTT estimate (ms)
    rtt: AtomicU32,
    
    /// Active request count
    active_requests: AtomicUsize,
    
    /// Bandwidth usage tracking
    bandwidth_tracker: Arc<BandwidthTracker>,
}

/// Per-request state
pub struct RequestState {
    /// Request ID
    id: RequestId,
    
    /// Request phase
    phase: RequestPhase,
    
    /// Timing information
    timing: ResourceTiming,
    
    /// Abort controller
    abort_handle: Option<AbortHandle>,
    
    /// Response builder
    response_builder: ResponseBuilder,
    
    /// Error state
    error: Option<NetworkError>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RequestPhase {
    Queued,
    DnsLookup,
    Connecting,
    TlsHandshake,
    Sending,
    Waiting,
    Receiving,
    Complete,
    Failed,
}
```

### Error Handling

```rust
/// Network error types
#[derive(Debug, thiserror::Error)]
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

/// Result type alias
pub type NetworkResult<T> = Result<T, NetworkError>;
```

### Test Strategy

#### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_http1_request() {
        let client = HttpClient::new(Default::default());
        let request = NetworkRequest {
            url: "https://example.com".parse().unwrap(),
            method: HttpMethod::Get,
            ..Default::default()
        };
        
        let response = client.fetch(request).await.unwrap();
        assert_eq!(response.status, 200);
    }
    
    #[tokio::test]
    async fn test_connection_pooling() {
        let pool = ConnectionPool::new(ConnectionLimits::default());
        
        // First request creates new connection
        let conn1 = pool.get_connection(&"example.com:443".parse().unwrap()).await.unwrap();
        pool.return_connection(conn1);
        
        // Second request reuses connection
        let conn2 = pool.get_connection(&"example.com:443".parse().unwrap()).await.unwrap();
        assert!(conn2.was_reused());
    }
    
    #[tokio::test]
    async fn test_websocket_connection() {
        let manager = WebSocketManager::new(Default::default());
        let ws = manager.connect("wss://echo.websocket.org", vec![]).await.unwrap();
        
        ws.send(WebSocketMessage::Text("Hello".into())).await.unwrap();
        let msg = ws.recv().await.unwrap().unwrap();
        
        match msg {
            WebSocketMessage::Text(text) => assert_eq!(text, "Hello"),
            _ => panic!("Expected text message"),
        }
    }
    
    #[tokio::test]
    async fn test_cache_hit() {
        let cache = HttpCache::new(CacheConfig::default());
        let request = create_cacheable_request();
        let response = create_cacheable_response();
        
        cache.store(&request, &response).await.unwrap();
        let cached = cache.get(&request).await.unwrap();
        
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().status, response.status);
    }
}
```

#### Integration Tests

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    use wiremock::{MockServer, Mock, ResponseTemplate};
    use wiremock::matchers::{method, path};
    
    #[tokio::test]
    async fn test_full_request_lifecycle() {
        let mock_server = MockServer::start().await;
        
        Mock::given(method("GET"))
            .and(path("/test"))
            .respond_with(ResponseTemplate::new(200)
                .set_body_string("Test response"))
            .mount(&mock_server)
            .await;
        
        let network_stack = NetworkStack::new(Default::default());
        let request = NetworkRequest {
            url: format!("{}/test", mock_server.uri()).parse().unwrap(),
            method: HttpMethod::Get,
            ..Default::default()
        };
        
        let response = network_stack.fetch(request).await.unwrap();
        assert_eq!(response.status, 200);
        
        let body = match response.body {
            ResponseBody::Bytes(bytes) => String::from_utf8(bytes).unwrap(),
            _ => panic!("Expected bytes body"),
        };
        assert_eq!(body, "Test response");
    }
    
    #[tokio::test]
    async fn test_http2_multiplexing() {
        // Test multiple concurrent requests over single HTTP/2 connection
        let network_stack = NetworkStack::new(Default::default());
        
        let requests: Vec<_> = (0..10)
            .map(|i| NetworkRequest {
                url: format!("https://http2.golang.org/reqinfo?n={}", i).parse().unwrap(),
                method: HttpMethod::Get,
                ..Default::default()
            })
            .collect();
        
        let responses = futures::future::join_all(
            requests.into_iter().map(|r| network_stack.fetch(r))
        ).await;
        
        for response in responses {
            assert!(response.is_ok());
            assert_eq!(response.unwrap().status, 200);
        }
    }
}
```

#### Performance Benchmarks

```rust
#[cfg(test)]
mod benchmarks {
    use super::*;
    use criterion::{black_box, criterion_group, criterion_main, Criterion};
    
    fn benchmark_request_parsing(c: &mut Criterion) {
        c.bench_function("parse_http_request", |b| {
            let raw_request = b"GET /path HTTP/1.1\r\nHost: example.com\r\n\r\n";
            b.iter(|| {
                parse_http_request(black_box(raw_request))
            });
        });
    }
    
    fn benchmark_connection_pool(c: &mut Criterion) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let pool = ConnectionPool::new(Default::default());
        
        c.bench_function("connection_pool_get", |b| {
            b.to_async(&rt).iter(|| async {
                let conn = pool.get_connection(&"example.com:443".parse().unwrap()).await;
                black_box(conn)
            });
        });
    }
    
    criterion_group!(benches, benchmark_request_parsing, benchmark_connection_pool);
    criterion_main!(benches);
}
```

### Implementation Milestones

#### Phase 1: Basic HTTP (Week 1-2)
- [ ] Basic HTTP/1.1 client using hyper
- [ ] Simple connection pooling
- [ ] Basic cookie support
- [ ] URL parsing and validation
- [ ] Integration with browser message bus
- **Internal Tests**: 100% unit test coverage
- **Validation**: Loads simple HTML pages
- **WPT Target**: 60% pass rate on basic fetch tests

#### Phase 2: Enhanced HTTP (Week 3-4)
- [ ] HTTP/2 support
- [ ] Full connection pooling
- [ ] Request prioritization
- [ ] Cache implementation
- [ ] Proxy support
- **Internal Tests**: 100% unit + integration tests pass
- **WPT Target**: 85% pass rate on fetch, xhr tests
- **Performance**: Within 3x Chrome speed
- **Validation**: Passes 80% of WPT fetch tests

#### Phase 3: Security Features (Week 5-6)
- [ ] TLS configuration
- [ ] Certificate validation
- [ ] HSTS enforcement
- [ ] Mixed content blocking
- [ ] CORS implementation
- **Internal Tests**: 100% pass including security tests
- **WPT Target**: 95% pass rate on cors, mixed-content tests
- **Security Validation**: 100% of security test suite
- **Validation**: Security audit pass

#### Phase 4: WebSocket (Week 7)
- [ ] WebSocket client implementation
- [ ] Frame parsing/encoding
- [ ] Compression extension
- [ ] Auto-reconnection logic
- **Internal Tests**: Full WebSocket test coverage
- **WPT Target**: 95% pass rate on websockets tests
- **Validation**: WebSocket echo server tests

#### Phase 5: Advanced Features (Week 8-9)
- [ ] HTTP/3 (QUIC) support
- [ ] DNS-over-HTTPS
- [ ] Bandwidth throttling
- [ ] Network condition simulation
- **Internal Tests**: Complete test coverage
- **WPT Target**: 90% overall network test pass rate
- **Performance**: Within 2x Chrome performance
- **Validation**: HTTP/3 interop tests

#### Phase 6: WebRTC (Week 10-12)
- [ ] ICE gathering
- [ ] DTLS transport
- [ ] Data channels
- [ ] Basic media transport
- **Internal Tests**: WebRTC unit + integration tests
- **WPT Target**: 85% pass rate on webrtc tests
- **Validation**: WebRTC samples working
- **Final Target**: 95% overall WPT pass rate

### Performance Requirements

#### Latency Targets
- DNS resolution: < 50ms (cached), < 200ms (uncached)
- TLS handshake: < 100ms (TLS 1.3)
- First byte: < 200ms (local), < 500ms (remote)
- WebSocket connection: < 300ms

#### Throughput Targets
- HTTP/1.1: > 100 Mbps
- HTTP/2: > 200 Mbps (multiplexed)
- HTTP/3: > 300 Mbps
- WebSocket: > 50 Mbps

#### Resource Limits
- Max connections per host: 6 (HTTP/1.1), 1 (HTTP/2)
- Max total connections: 256
- Connection pool size: 128
- DNS cache entries: 1000
- Cookie jar size: 3000 cookies

### Security Considerations

#### TLS Configuration
```rust
pub fn create_tls_config() -> rustls::ClientConfig {
    let mut config = rustls::ClientConfig::builder()
        .with_safe_default_cipher_suites()
        .with_safe_default_kx_groups()
        .with_protocol_versions(&[
            &rustls::version::TLS13,
            &rustls::version::TLS12,
        ])
        .unwrap()
        .with_root_certificates(load_root_certificates())
        .with_no_client_auth();
    
    // Enable ALPN for HTTP/2 and HTTP/3
    config.alpn_protocols = vec![
        b"h3".to_vec(),
        b"h2".to_vec(),
        b"http/1.1".to_vec(),
    ];
    
    // Enable session resumption
    config.session_storage = Arc::new(rustls::client::ClientSessionMemoryCache::new(256));
    
    config
}
```

#### Certificate Validation
- Verify certificate chain to trusted root
- Check certificate validity period
- Validate hostname matching
- Support certificate pinning
- Implement certificate transparency checks

#### CORS Policy Enforcement
- Check origin headers
- Validate allowed methods
- Handle preflight requests
- Enforce credentials mode
- Block disallowed cross-origin requests

### Platform-Specific Implementations

#### Linux
- Use system certificate store via rustls-native-certs
- Support for AF_PACKET for raw socket access (future)
- Integration with system proxy settings via environment variables

#### Windows
- Certificate store access via Windows APIs
- WinInet proxy configuration support
- Windows Filtering Platform integration (future)

#### macOS
- Keychain certificate access
- System proxy configuration via SystemConfiguration framework
- Network.framework integration (future)

### Configuration Schema

```rust
/// Network stack configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// HTTP client settings
    pub http: HttpConfig,
    
    /// WebSocket settings
    pub websocket: WebSocketConfig,
    
    /// WebRTC settings
    pub webrtc: WebRtcConfig,
    
    /// Cache settings
    pub cache: CacheConfig,
    
    /// Security settings
    pub security: SecurityConfig,
    
    /// Proxy settings
    pub proxy: ProxyConfig,
    
    /// DNS settings
    pub dns: DnsConfig,
    
    /// Performance settings
    pub performance: PerformanceConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpConfig {
    /// User agent string
    pub user_agent: String,
    
    /// Accept-Language header
    pub accept_language: String,
    
    /// Maximum redirects to follow
    pub max_redirects: u32,
    
    /// Request timeout
    pub timeout: Duration,
    
    /// Enable HTTP/2
    pub enable_http2: bool,
    
    /// Enable HTTP/3
    pub enable_http3: bool,
    
    /// Connection pool size
    pub pool_size: usize,
    
    /// Idle connection timeout
    pub idle_timeout: Duration,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            http: HttpConfig {
                user_agent: "CortenBrowser/1.0".into(),
                accept_language: "en-US,en;q=0.9".into(),
                max_redirects: 10,
                timeout: Duration::from_secs(30),
                enable_http2: true,
                enable_http3: false, // Experimental
                pool_size: 128,
                idle_timeout: Duration::from_secs(90),
            },
            // ... other defaults
        }
    }
}
```

### Monitoring and Metrics

```rust
/// Network metrics
#[derive(Debug, Clone, Serialize)]
pub struct NetworkMetrics {
    /// Request counters
    pub requests_total: u64,
    pub requests_success: u64,
    pub requests_failed: u64,
    
    /// Bandwidth metrics
    pub bytes_sent: u64,
    pub bytes_received: u64,
    
    /// Connection metrics
    pub connections_active: usize,
    pub connections_idle: usize,
    pub connections_reused: u64,
    
    /// Cache metrics
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub cache_size_bytes: u64,
    
    /// Performance metrics
    pub avg_dns_time_ms: f64,
    pub avg_connect_time_ms: f64,
    pub avg_ttfb_ms: f64,
    pub avg_download_time_ms: f64,
    
    /// Protocol distribution
    pub http1_requests: u64,
    pub http2_requests: u64,
    pub http3_requests: u64,
    pub websocket_connections: u64,
    pub webrtc_connections: u64,
}

/// Metrics collection
impl NetworkStack {
    pub fn collect_metrics(&self) -> NetworkMetrics {
        NetworkMetrics {
            requests_total: self.metrics.requests_total.load(Ordering::Relaxed),
            requests_success: self.metrics.requests_success.load(Ordering::Relaxed),
            requests_failed: self.metrics.requests_failed.load(Ordering::Relaxed),
            // ... collect all metrics
        }
    }
    
    pub fn export_metrics_prometheus(&self) -> String {
        let metrics = self.collect_metrics();
        format!(
            r#"# HELP network_requests_total Total number of network requests
# TYPE network_requests_total counter
network_requests_total {{status="success"}} {}
network_requests_total {{status="failed"}} {}

# HELP network_bytes_total Total bytes transferred
# TYPE network_bytes_total counter  
network_bytes_total {{direction="sent"}} {}
network_bytes_total {{direction="received"}} {}

# HELP network_connections Active connections
# TYPE network_connections gauge
network_connections {{state="active"}} {}
network_connections {{state="idle"}} {}"#,
            metrics.requests_success,
            metrics.requests_failed,
            metrics.bytes_sent,
            metrics.bytes_received,
            metrics.connections_active,
            metrics.connections_idle,
        )
    }
}
```

### Public Test Suite Integration

#### Web Platform Tests (WPT) Integration
The Network Stack component must be validated against public browser test suites after internal tests pass. These tests should only be attempted when the component has successfully passed all TDD, BDD, and integration tests.

##### WPT Test Coverage for Network Stack
| Test Directory | Test Count | Target Pass Rate | Priority |
|---------------|------------|------------------|----------|
| /wpt/fetch/ | 800+ | 90% | Essential |
| /wpt/xhr/ | 400+ | 90% | Essential |
| /wpt/websockets/ | 200+ | 95% | Essential |
| /wpt/webrtc/ | 500+ | 85% | Phase 4 |
| /wpt/cors/ | 300+ | 95% | Essential |
| /wpt/service-workers/cache-storage/ | 150+ | 85% | Phase 3 |
| /wpt/mixed-content/ | 100+ | 100% | Security |
| /wpt/content-security-policy/ | 200+ | 95% | Security |

##### WPT Test Harness Implementation
```rust
// network-stack/tests/wpt_harness.rs
use wpt_runner::{TestHarness, TestResult};
use browser_network_stack::NetworkStack;

pub struct NetworkStackWptHarness {
    stack: NetworkStack,
    test_server: WptTestServer,
}

impl TestHarness for NetworkStackWptHarness {
    async fn fetch(&self, url: &str, options: FetchOptions) -> TestResult {
        let request = self.convert_wpt_request(url, options);
        match self.stack.fetch(request).await {
            Ok(response) => TestResult::from_response(response),
            Err(e) => TestResult::error(e.to_string()),
        }
    }
    
    async fn create_websocket(&self, url: &str) -> TestResult {
        match self.stack.connect_websocket(url.parse()?, vec![]).await {
            Ok(ws) => TestResult::websocket(ws),
            Err(e) => TestResult::error(e.to_string()),
        }
    }
    
    async fn xhr_request(&self, request: XhrRequest) -> TestResult {
        // Map XHR to fetch internally
        self.fetch(&request.url, request.into()).await
    }
}
```

##### Running WPT Tests
```bash
# Clone WPT repository (one-time setup)
git clone --depth=1 https://github.com/web-platform-tests/wpt.git

# Run Network Stack specific tests
./wpt run --include fetch,xhr,websockets,cors \
          --binary ./target/release/network-stack-harness \
          --log-raw test-results.json

# Run security-focused tests
./wpt run --include mixed-content,content-security-policy \
          --binary ./target/release/network-stack-harness
```

#### Chromium Network Tests
Extract and adapt relevant tests from Chromium's network test suite:

```bash
# Relevant Chromium test directories
chromium/src/net/test/        # 5,000+ network tests
chromium/src/services/network/test/  # Service layer tests
chromium/src/content/test/data/  # Integration test data
```

##### Test Extraction Script
```python
# extract_chromium_net_tests.py
import json
import re

def extract_network_tests():
    """Extract network tests relevant to our implementation"""
    test_categories = {
        'http2': [
            'net/test/http2_*_test.cc',
            'net/spdy/*_test.cc'
        ],
        'http3': [
            'net/test/quic_*_test.cc',
            'net/third_party/quiche/src/quic/test/*'
        ],
        'websocket': [
            'net/websockets/*_test.cc'
        ],
        'cache': [
            'net/http/http_cache_*_test.cc'
        ]
    }
    return convert_to_rust_tests(test_categories)
```

#### Performance Benchmarks

##### Network-Specific Performance Tests
```rust
// network-stack/benches/performance.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};

fn benchmark_http_versions(c: &mut Criterion) {
    let mut group = c.benchmark_group("http_versions");
    
    for size in [1024, 10240, 102400, 1048576].iter() {
        group.bench_with_input(BenchmarkId::new("http1", size), size, |b, &size| {
            b.to_async(&runtime).iter(|| async {
                fetch_http1_data(black_box(size)).await
            });
        });
        
        group.bench_with_input(BenchmarkId::new("http2", size), size, |b, &size| {
            b.to_async(&runtime).iter(|| async {
                fetch_http2_data(black_box(size)).await
            });
        });
        
        group.bench_with_input(BenchmarkId::new("http3", size), size, |b, &size| {
            b.to_async(&runtime).iter(|| async {
                fetch_http3_data(black_box(size)).await
            });
        });
    }
    group.finish();
}

fn benchmark_websocket_throughput(c: &mut Criterion) {
    c.bench_function("websocket_messages_per_second", |b| {
        b.to_async(&runtime).iter(|| async {
            send_websocket_messages(black_box(1000)).await
        });
    });
}

criterion_group!(benches, benchmark_http_versions, benchmark_websocket_throughput);
criterion_main!(benches);
```

#### curl Test Suite Integration
Validate HTTP implementation against curl's test suite:

```bash
# Run curl test suite
git clone https://github.com/curl/curl.git
cd curl/tests
./runtests.pl --verbose \
              --curl ./network-stack-curl-compat \
              1-500  # Run first 500 tests

# Expected results
# - Protocol tests: 95% pass rate
# - Feature tests: 90% pass rate
# - Error handling: 100% pass rate
```

### Test Automation Infrastructure

#### Continuous Integration Pipeline
```yaml
# .github/workflows/network-stack-tests.yml
name: Network Stack Test Suite

on:
  push:
    paths:
      - 'components/network-stack/**'
  pull_request:
    paths:
      - 'components/network-stack/**'

jobs:
  internal-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Run Internal Tests
        run: |
          cd components/network-stack
          cargo test --all-features
          cargo test --no-default-features
      
      - name: Run Integration Tests
        run: cargo test --test integration
      
      - name: Check Test Coverage
        run: |
          cargo tarpaulin --out Xml
          # Fail if coverage < 80%
          coverage=$(grep 'line-rate' cobertura.xml | grep -o '[0-9.]*')
          if (( $(echo "$coverage < 0.8" | bc -l) )); then
            echo "Coverage $coverage is below 80%"
            exit 1
          fi

  wpt-tests:
    needs: internal-tests
    runs-on: ubuntu-latest
    strategy:
      matrix:
        test-suite: [fetch, xhr, websockets, cors, mixed-content]
    steps:
      - uses: actions/checkout@v3
      - name: Setup WPT
        run: |
          git clone --depth=1 https://github.com/web-platform-tests/wpt.git
          pip install -e wpt/tools/wpt
      
      - name: Build Test Harness
        run: |
          cd components/network-stack
          cargo build --release --features wpt-harness
      
      - name: Run WPT ${{ matrix.test-suite }}
        run: |
          wpt run --channel dev \
                  --log-raw results-${{ matrix.test-suite }}.json \
                  --include ${{ matrix.test-suite }} \
                  --binary ./target/release/network-stack-harness
      
      - name: Analyze Results
        run: |
          python analyze_wpt_results.py \
                 results-${{ matrix.test-suite }}.json \
                 --expected-pass-rate 0.85

  performance-benchmarks:
    needs: internal-tests
    runs-on: ubuntu-latest
    steps:
      - name: Run Benchmarks
        run: |
          cd components/network-stack
          cargo bench --features bench
          
      - name: Compare with Baseline
        run: |
          # Compare against Chrome/Firefox baselines
          python compare_benchmarks.py \
                 target/criterion \
                 --max-regression 0.1
```

#### Test Results Database
```sql
-- network_stack_test_results.sql
CREATE TABLE wpt_results (
    id INTEGER PRIMARY KEY,
    test_suite TEXT NOT NULL,
    test_path TEXT NOT NULL,
    result TEXT CHECK(result IN ('PASS', 'FAIL', 'TIMEOUT', 'SKIP')),
    duration_ms INTEGER,
    error_message TEXT,
    commit_hash TEXT NOT NULL,
    run_date TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_test_suite ON wpt_results(test_suite);
CREATE INDEX idx_result ON wpt_results(result);
CREATE INDEX idx_date ON wpt_results(run_date);

-- Track progress over time
CREATE VIEW test_progress AS
SELECT 
    test_suite,
    run_date,
    COUNT(*) as total_tests,
    SUM(CASE WHEN result = 'PASS' THEN 1 ELSE 0 END) as passed,
    ROUND(100.0 * SUM(CASE WHEN result = 'PASS' THEN 1 ELSE 0 END) / COUNT(*), 2) as pass_rate
FROM wpt_results
GROUP BY test_suite, DATE(run_date)
ORDER BY run_date DESC;
```

### Test-Driven Implementation Strategy

#### Phase-Based Test Enablement
```rust
// network-stack/src/test_config.rs
pub struct TestPhases {
    pub phase_1_basic: TestSuite {
        internal_tests: true,
        wpt_basic: true,      // Only fetch, xhr basics
        wpt_full: false,
        chromium_tests: false,
        performance_tests: false,
    },
    
    pub phase_2_enhanced: TestSuite {
        internal_tests: true,
        wpt_basic: true,
        wpt_full: true,       // All fetch, xhr, CORS
        chromium_tests: false,
        performance_tests: true,
    },
    
    pub phase_3_security: TestSuite {
        internal_tests: true,
        wpt_basic: true,
        wpt_full: true,
        wpt_security: true,   // Mixed content, CSP
        chromium_tests: true,
        performance_tests: true,
    },
    
    pub phase_4_complete: TestSuite {
        all_tests: true,      // Everything including WebRTC
    },
}
```

#### Progressive Test Target Metrics
| Phase | Internal Tests | WPT Pass Rate | Performance vs Chrome | Security Tests |
|-------|---------------|---------------|----------------------|----------------|
| Phase 1 | 100% | 60% (basic) | N/A | Basic |
| Phase 2 | 100% | 85% (HTTP) | 3x slower | CORS/HTTPS |
| Phase 3 | 100% | 90% (all) | 2x slower | 100% pass |
| Phase 4 | 100% | 95% (all) | 1.5x slower | 100% pass |

### Development Commands for Claude Code

```bash
# Initial setup
cargo new browser-network-stack --lib
cd browser-network-stack

# Add all dependencies to Cargo.toml
# Copy the dependencies section from this specification

# Create module structure
mkdir -p src/{http,websocket,webrtc,cache,cookies,dns,security,proxy,utils}
touch src/{http,websocket,webrtc,cache,cookies,dns,security,proxy,utils}/mod.rs

# Set up test infrastructure
mkdir -p tests/{unit,integration,wpt,benchmarks}

# Clone test suites (one-time setup)
git clone --depth=1 https://github.com/web-platform-tests/wpt.git ../wpt
git clone --depth=1 https://github.com/curl/curl.git ../curl-tests

# Build the component
cargo build --release

# Run internal tests (must pass before public tests)
cargo test --all-features
cargo test --no-default-features

# Run integration tests
cargo test --test integration

# After internal tests pass, run WPT tests
cd ../wpt
./wpt run --include fetch,xhr,websockets,cors \
          --binary ../browser-network-stack/target/release/network-stack-harness

# Run benchmarks
cd ../browser-network-stack
cargo bench --features bench

# Generate documentation
cargo doc --open

# Check for security issues
cargo audit

# Format code
cargo fmt

# Lint code
cargo clippy -- -D warnings

# Run with different feature sets
cargo build --no-default-features --features http2
cargo build --no-default-features --features websocket
cargo build --all-features

# Generate test coverage report
cargo tarpaulin --out Html --output-dir ./coverage

# Run performance comparison
cargo bench --bench http_versions -- --save-baseline phase1
```

### Integration with Browser Components

#### Message Bus Integration Example

```rust
impl BrowserComponent for NetworkStack {
    fn initialize(&mut self, config: ComponentConfig) -> Result<(), ComponentError> {
        // Initialize HTTP client
        self.http_client = HttpClient::new(config.network);
        
        // Initialize WebSocket manager
        self.websocket_manager = WebSocketManager::new(config.websocket);
        
        // Initialize WebRTC manager
        self.webrtc_manager = WebRtcManager::new(config.webrtc);
        
        // Start background tasks
        self.start_connection_cleanup_task();
        self.start_metrics_collection_task();
        
        Ok(())
    }
    
    fn handle_message(&mut self, msg: ComponentMessage) -> Result<ComponentResponse, ComponentError> {
        match msg {
            ComponentMessage::FetchRequest { url, options } => {
                let request = self.build_request(url, options)?;
                let response = self.http_client.fetch(request).await?;
                Ok(ComponentResponse::FetchComplete(response))
            }
            
            ComponentMessage::WebSocketConnect { url, protocols } => {
                let connection = self.websocket_manager.connect(url, protocols).await?;
                Ok(ComponentResponse::WebSocketConnected(connection))
            }
            
            _ => Ok(ComponentResponse::NotHandled),
        }
    }
    
    fn shutdown(&mut self) -> Result<(), ComponentError> {
        // Close all connections
        self.http_client.shutdown().await?;
        self.websocket_manager.shutdown().await?;
        self.webrtc_manager.shutdown().await?;
        
        Ok(())
    }
}
```

### Validation Checklist

#### Core Functionality (Internal Tests)
- [ ] HTTP/1.1 GET requests work
- [ ] HTTP/1.1 POST with body works
- [ ] HTTP/2 multiplexing works
- [ ] HTTPS connections work
- [ ] Redirects are followed correctly
- [ ] Cookies are stored and sent
- [ ] Cache serves cached responses
- [ ] Proxy connections work

#### Advanced Features (Internal Tests)
- [ ] HTTP/3 connections work (when server supports)
- [ ] WebSocket connections establish
- [ ] WebSocket messages send/receive
- [ ] WebRTC data channels work
- [ ] Certificate validation works
- [ ] CORS blocks disallowed requests
- [ ] Network throttling works
- [ ] DNS-over-HTTPS resolves

#### Web Platform Tests (After Internal Tests Pass)
- [ ] WPT fetch: 90% pass rate (800+ tests)
- [ ] WPT xhr: 90% pass rate (400+ tests)
- [ ] WPT websockets: 95% pass rate (200+ tests)
- [ ] WPT cors: 95% pass rate (300+ tests)
- [ ] WPT mixed-content: 100% pass rate (100+ tests)
- [ ] WPT content-security-policy: 95% pass rate (200+ tests)
- [ ] WPT webrtc: 85% pass rate (500+ tests, Phase 4 only)

#### Performance Benchmarks
- [ ] Meets latency targets
- [ ] Meets throughput targets
- [ ] Connection pooling reduces latency
- [ ] HTTP/2 multiplexing improves performance
- [ ] Memory usage is bounded
- [ ] Within 2x Chrome performance (Phase 3)
- [ ] Within 1.5x Chrome performance (Phase 4)

#### Security Test Suite
- [ ] Invalid certificates are rejected
- [ ] HSTS is enforced
- [ ] Mixed content is blocked (100% WPT pass)
- [ ] Secure cookies are protected
- [ ] TLS 1.2+ only
- [ ] CSP headers properly enforced
- [ ] CORS violations blocked

#### Chromium/curl Test Compatibility
- [ ] curl protocol tests: 95% pass rate
- [ ] Chromium net/test subset: 85% pass rate
- [ ] curl feature tests: 90% pass rate

### Notes for Claude Code

This specification is designed to be self-contained for autonomous implementation by Claude Code. Key points:

1. **Test-Driven Development**: Follow strict TDD approach - write tests first, implement to pass
2. **Progressive Test Validation**: 
   - Internal tests must pass 100% before attempting public test suites
   - Start with WPT basic tests (fetch, xhr) before advanced features
   - Only run performance benchmarks after functional tests pass
3. **Start with Phase 1**: Implement basic HTTP functionality first using hyper
4. **Progressive enhancement**: Each phase builds on the previous
5. **Test continuously**: Run tests after each major feature
6. **Use existing libraries**: Don't reinvent the wheel for complex protocols
7. **Modular design**: Keep sub-modules under 10,000 lines each
8. **Document everything**: Include inline documentation for complex logic
9. **Error handling**: Use Result types consistently
10. **Performance**: Profile and optimize critical paths only after tests pass
11. **Security**: Default to secure, require opt-in for insecure
12. **Platform compatibility**: Test on Linux primarily, ensure abstractions for other platforms
13. **Public Test Suites**: WPT, Chromium tests, and curl tests are validation goals, not development drivers

When implementing, create a local git repository and commit after each milestone for easy rollback if needed. Use cargo features to allow partial builds during development.

**Test Suite Priority**:
1. Unit tests (must pass 100%)
2. Integration tests (must pass 100%)
3. WPT basic subset (target 60% initially)
4. WPT full subset (target 90% by phase completion)
5. Performance benchmarks (compare against baseline)
6. Security test suite (must pass 100% for security features)

---

**End of Network Stack Component Specification v1.1**

**Updates in v1.1**:
- Added comprehensive public test suite integration (WPT, Chromium, curl)
- Specified test-driven development workflow
- Added progressive test validation targets per phase
- Included test automation infrastructure
- Updated project name to CortenBrowser
- Added test harness implementations for WPT compatibility
