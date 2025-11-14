# Phase 2: Architecture Plan for New Components
**Date**: 2025-11-14
**Session**: claude/orchestrate-full-01NaRyyt4DBVag5HkCbCjHgp (resumed)
**Target**: 100% specification feature coverage

---

## Architecture Overview

### Phase 1 (Complete)
- **13 components** implementing core protocols and infrastructure
- **4-level dependency hierarchy** (Base → Core → Protocol → Integration)
- **202 tests passing** (100% pass rate)

### Phase 2 (This Plan)
- **15 new components** to achieve 100% feature coverage
- **1 enhanced component** (network_stack) for full integration
- **Maintains 4-level hierarchy** with clear separation of concerns

---

## Component Architecture by Level

### Level 0: Base Layer (No Changes)

**Existing Components**:
1. network_types - Core types and traits
2. network_errors - Error handling

**No new components at Level 0**

---

### Level 1: Core Layer (12 New Components)

#### Component 1: proxy_support

**Responsibility**: HTTP and SOCKS5 proxy client implementation

**Public API**:
```rust
pub struct ProxyClient {
    config: ProxyConfig,
    http_connector: HttpProxyConnector,
    socks_connector: Socks5Connector,
}

pub enum ProxyConfig {
    None,
    Http {
        host: String,
        port: u16,
        auth: Option<ProxyAuth>,
    },
    Socks5 {
        host: String,
        port: u16,
        auth: Option<ProxyAuth>,
    },
    Auto {
        pac_url: Url,
    },
}

pub enum ProxyAuth {
    Basic { username: String, password: String },
    Digest { username: String, password: String },
}

impl ProxyClient {
    pub async fn connect(&self, target: SocketAddr) -> Result<TcpStream, NetworkError>;
    pub async fn connect_tls(&self, target: SocketAddr, domain: &str) -> Result<TlsStream, NetworkError>;
}
```

**Internal Structure**:
- `src/http_proxy.rs` - HTTP proxy CONNECT method
- `src/socks5.rs` - SOCKS5 protocol implementation
- `src/pac.rs` - Proxy Auto-Configuration parsing
- `src/auth.rs` - Proxy authentication handling

**Dependencies**: network_types, network_errors, tls_manager, dns_resolver
**Token Budget**: 70,000-80,000
**Test Coverage**: HTTP proxy tunneling, SOCKS5 connection, PAC parsing, auth methods

---

#### Component 2: cors_validator

**Responsibility**: CORS policy enforcement and validation

**Public API**:
```rust
pub struct CorsValidator {
    config: CorsConfig,
}

pub struct CorsConfig {
    pub enforce_same_origin: bool,
    pub allow_credentials: bool,
}

pub struct CorsResult {
    pub allowed: bool,
    pub reason: Option<String>,
    pub headers_to_add: HeaderMap,
}

impl CorsValidator {
    pub fn validate_request(&self, request: &NetworkRequest, origin: &str) -> CorsResult;
    pub fn validate_response(&self, response: &NetworkResponse, origin: &str) -> CorsResult;
    pub fn is_preflight_needed(&self, request: &NetworkRequest) -> bool;
    pub fn build_preflight_request(&self, request: &NetworkRequest) -> NetworkRequest;
}
```

**Internal Structure**:
- `src/validator.rs` - CORS validation logic
- `src/preflight.rs` - Preflight request handling
- `src/headers.rs` - Access-Control-* header parsing
- `src/policy.rs` - CORS policy configuration

**Dependencies**: network_types, network_errors
**Token Budget**: 60,000-70,000
**Test Coverage**: Same-origin checks, preflight handling, credential modes, header validation

---

#### Component 3: content_encoding

**Responsibility**: Content encoding/decoding (gzip, brotli, deflate)

**Public API**:
```rust
pub struct ContentEncoder {
    supported_encodings: Vec<Encoding>,
}

pub enum Encoding {
    Gzip,
    Deflate,
    Brotli,
    Identity,
}

impl ContentEncoder {
    pub fn encode(&self, data: &[u8], encoding: Encoding) -> Result<Vec<u8>, NetworkError>;
    pub fn decode(&self, data: &[u8], encoding: Encoding) -> Result<Vec<u8>, NetworkError>;
    pub fn decode_stream(&self, stream: impl Stream<Item = Bytes>, encoding: Encoding)
        -> impl Stream<Item = Result<Bytes, NetworkError>>;
    pub fn get_accept_encoding(&self) -> String;
}
```

**Internal Structure**:
- `src/gzip.rs` - gzip compression/decompression
- `src/brotli.rs` - brotli compression/decompression
- `src/deflate.rs` - deflate compression/decompression
- `src/stream.rs` - Streaming decompression

**Dependencies**: network_types, network_errors
**External Crates**: flate2 = "1.0", brotli = "3.4"
**Token Budget**: 50,000-60,000
**Test Coverage**: Each encoding method, streaming decompression, header negotiation

---

#### Component 4: request_scheduler

**Responsibility**: Request prioritization and scheduling

**Public API**:
```rust
pub struct RequestScheduler {
    high_priority: VecDeque<PendingRequest>,
    medium_priority: VecDeque<PendingRequest>,
    low_priority: VecDeque<PendingRequest>,
    active_requests: HashMap<RequestId, ActiveRequest>,
    max_concurrent: usize,
}

pub enum RequestPriority {
    High,    // Navigation, CSS, fonts
    Medium,  // Scripts, XHR
    Low,     // Images, prefetch
}

impl RequestScheduler {
    pub fn schedule(&mut self, request: NetworkRequest, priority: RequestPriority) -> RequestId;
    pub fn next_request(&mut self) -> Option<NetworkRequest>;
    pub fn cancel_request(&mut self, id: RequestId) -> Result<(), NetworkError>;
    pub fn set_max_concurrent(&mut self, max: usize);
}
```

**Internal Structure**:
- `src/scheduler.rs` - Scheduling algorithm
- `src/priority.rs` - Priority queue management
- `src/request.rs` - Pending and active request tracking

**Dependencies**: network_types, network_errors
**Token Budget**: 50,000-60,000
**Test Coverage**: Priority ordering, concurrent limits, cancellation, fair scheduling

---

#### Component 5: bandwidth_limiter

**Responsibility**: Bandwidth throttling and network condition simulation

**Public API**:
```rust
pub struct BandwidthLimiter {
    download_limit: Option<u64>, // bytes per second
    upload_limit: Option<u64>,
    latency: Duration,
    tracker: BandwidthTracker,
}

pub struct NetworkConditions {
    pub connection_type: ConnectionType,
    pub effective_type: EffectiveConnectionType,
    pub downlink_mbps: f64,
    pub rtt_ms: u32,
}

pub enum EffectiveConnectionType {
    Slow2g,
    G2,
    G3,
    G4,
}

impl BandwidthLimiter {
    pub async fn throttle_read(&self, bytes: &[u8]) -> Result<(), NetworkError>;
    pub async fn throttle_write(&self, bytes: &[u8]) -> Result<(), NetworkError>;
    pub fn set_conditions(&mut self, conditions: NetworkConditions);
    pub fn get_usage(&self) -> BandwidthUsage;
}
```

**Internal Structure**:
- `src/throttle.rs` - Bandwidth throttling logic
- `src/tracker.rs` - Bandwidth usage tracking
- `src/conditions.rs` - Network condition simulation

**Dependencies**: network_types, network_errors
**Token Budget**: 50,000-60,000
**Test Coverage**: Download/upload throttling, latency injection, usage tracking

---

#### Component 6: url_handlers

**Responsibility**: Data and file URL handling

**Public API**:
```rust
pub struct UrlHandler;

pub enum SpecialUrl {
    Data {
        mime_type: String,
        base64: bool,
        data: Vec<u8>,
    },
    File {
        path: PathBuf,
    },
}

impl UrlHandler {
    pub fn parse_data_url(url: &Url) -> Result<SpecialUrl, NetworkError>;
    pub fn parse_file_url(url: &Url) -> Result<SpecialUrl, NetworkError>;
    pub async fn fetch_data_url(url: &Url) -> Result<NetworkResponse, NetworkError>;
    pub async fn fetch_file_url(url: &Url) -> Result<NetworkResponse, NetworkError>;
}
```

**Internal Structure**:
- `src/data_url.rs` - data: URL parsing and handling
- `src/file_url.rs` - file: URL parsing and handling
- `src/validation.rs` - Security validation (same-origin, path traversal)

**Dependencies**: network_types, network_errors
**Token Budget**: 40,000-50,000
**Test Coverage**: Data URL parsing, base64 decoding, file URL security, MIME type extraction

---

#### Component 7: certificate_transparency

**Responsibility**: Certificate Transparency validation

**Public API**:
```rust
pub struct CtValidator {
    log_list: CtLogList,
    policy: CtPolicy,
}

pub struct CtLogList {
    logs: Vec<CtLog>,
}

pub struct CtPolicy {
    pub require_sct: bool,
    pub min_sct_count: usize,
}

impl CtValidator {
    pub async fn validate_certificate(&self, cert: &Certificate) -> Result<CtValidation, NetworkError>;
    pub fn extract_embedded_scts(&self, cert: &Certificate) -> Vec<SignedCertificateTimestamp>;
    pub async fn verify_sct(&self, sct: &SignedCertificateTimestamp, cert: &Certificate) -> Result<bool, NetworkError>;
}
```

**Internal Structure**:
- `src/sct.rs` - Signed Certificate Timestamp parsing
- `src/log_list.rs` - CT log list management
- `src/validation.rs` - SCT verification
- `src/policy.rs` - CT policy enforcement

**Dependencies**: network_types, network_errors, tls_manager
**Token Budget**: 60,000-70,000
**Test Coverage**: SCT extraction, SCT verification, policy enforcement

---

#### Component 8: mixed_content_blocker

**Responsibility**: Mixed content detection and blocking

**Public API**:
```rust
pub struct MixedContentBlocker {
    config: MixedContentConfig,
}

pub struct MixedContentConfig {
    pub block_all: bool,
    pub upgrade_insecure: bool,
}

pub enum ContentType {
    Active,  // Scripts, stylesheets, iframes
    Passive, // Images, audio, video
}

pub enum BlockResult {
    Allow,
    Block { reason: String },
    Upgrade { new_url: Url },
}

impl MixedContentBlocker {
    pub fn check_request(&self, request: &NetworkRequest, context: &SecurityContext) -> BlockResult;
    pub fn is_secure_context(&self, url: &Url) -> bool;
}
```

**Internal Structure**:
- `src/detector.rs` - Mixed content detection
- `src/blocker.rs` - Blocking logic
- `src/upgrade.rs` - Upgrade-Insecure-Requests support

**Dependencies**: network_types, network_errors
**Token Budget**: 40,000-50,000
**Test Coverage**: HTTPS context detection, active/passive content, upgrade support

---

#### Component 9: csp_processor

**Responsibility**: Content Security Policy header processing

**Public API**:
```rust
pub struct CspProcessor {
    policies: Vec<CspPolicy>,
}

pub struct CspPolicy {
    directives: HashMap<String, CspDirective>,
    report_only: bool,
}

pub enum CspDirective {
    DefaultSrc(Vec<Source>),
    ScriptSrc(Vec<Source>),
    StyleSrc(Vec<Source>),
    ImgSrc(Vec<Source>),
    // ... other directives
}

pub struct CspViolation {
    pub directive: String,
    pub blocked_uri: String,
    pub source_file: Option<String>,
}

impl CspProcessor {
    pub fn parse_header(&self, header: &str) -> Result<CspPolicy, NetworkError>;
    pub fn check_resource(&self, url: &Url, resource_type: ResourceType) -> Result<(), CspViolation>;
    pub fn check_inline_script(&self, script: &str, nonce: Option<&str>) -> Result<(), CspViolation>;
}
```

**Internal Structure**:
- `src/parser.rs` - CSP header parsing
- `src/validator.rs` - Directive validation
- `src/reporter.rs` - Violation reporting
- `src/nonce.rs` - Nonce and hash validation

**Dependencies**: network_types, network_errors
**Token Budget**: 60,000-70,000
**Test Coverage**: Header parsing, directive enforcement, nonce validation, hash validation

---

#### Component 10: certificate_pinning

**Responsibility**: Certificate pinning and validation

**Public API**:
```rust
pub struct CertificatePinner {
    pins: HashMap<String, Vec<Pin>>,
}

pub struct Pin {
    pub algorithm: HashAlgorithm,
    pub hash: Vec<u8>,
    pub expiry: Option<DateTime<Utc>>,
}

pub enum HashAlgorithm {
    Sha256,
    Sha384,
    Sha512,
}

impl CertificatePinner {
    pub fn add_pin(&mut self, domain: &str, pin: Pin);
    pub fn validate_certificate(&self, domain: &str, cert: &Certificate) -> Result<(), NetworkError>;
    pub fn remove_expired_pins(&mut self);
}
```

**Internal Structure**:
- `src/pin_storage.rs` - Pin storage and management
- `src/validator.rs` - Pin validation
- `src/hpkp.rs` - HTTP Public Key Pinning support

**Dependencies**: network_types, network_errors, tls_manager
**Token Budget**: 50,000-60,000
**Test Coverage**: Pin storage, pin validation, expiry handling

---

#### Component 11: network_metrics

**Responsibility**: Network metrics collection and export

**Public API**:
```rust
pub struct NetworkMetrics {
    // Counters
    pub requests_total: AtomicU64,
    pub requests_success: AtomicU64,
    pub requests_failed: AtomicU64,

    // Bandwidth
    pub bytes_sent: AtomicU64,
    pub bytes_received: AtomicU64,

    // Connections
    pub connections_active: AtomicUsize,
    pub connections_idle: AtomicUsize,
    pub connections_reused: AtomicU64,

    // Cache
    pub cache_hits: AtomicU64,
    pub cache_misses: AtomicU64,
    pub cache_size_bytes: AtomicU64,

    // Performance
    avg_dns_time_ms: RwLock<MovingAverage>,
    avg_connect_time_ms: RwLock<MovingAverage>,
    avg_ttfb_ms: RwLock<MovingAverage>,

    // Protocol distribution
    pub http1_requests: AtomicU64,
    pub http2_requests: AtomicU64,
    pub http3_requests: AtomicU64,
    pub websocket_connections: AtomicU64,
    pub webrtc_connections: AtomicU64,
}

impl NetworkMetrics {
    pub fn record_request(&self, protocol: Protocol, success: bool);
    pub fn record_timing(&self, phase: Phase, duration: Duration);
    pub fn record_bytes(&self, sent: u64, received: u64);
    pub fn export_prometheus(&self) -> String;
    pub fn snapshot(&self) -> MetricsSnapshot;
}
```

**Internal Structure**:
- `src/collector.rs` - Metrics collection
- `src/storage.rs` - Atomic counter management
- `src/export.rs` - Prometheus export format
- `src/moving_average.rs` - Moving average calculation

**Dependencies**: network_types
**Token Budget**: 50,000-60,000
**Test Coverage**: Counter updates, timing recording, Prometheus format, snapshot accuracy

---

#### Component 12: platform_integration

**Responsibility**: Platform-specific certificate store and proxy configuration

**Public API**:
```rust
pub struct PlatformIntegration;

impl PlatformIntegration {
    pub fn load_system_certificates() -> Result<Vec<Certificate>, NetworkError>;
    pub fn get_system_proxy_config() -> Result<ProxyConfig, NetworkError>;

    #[cfg(target_os = "windows")]
    pub fn load_windows_certificates() -> Result<Vec<Certificate>, NetworkError>;

    #[cfg(target_os = "macos")]
    pub fn load_keychain_certificates() -> Result<Vec<Certificate>, NetworkError>;

    #[cfg(target_os = "linux")]
    pub fn load_linux_certificates() -> Result<Vec<Certificate>, NetworkError>;
}
```

**Internal Structure**:
- `src/windows.rs` - Windows certificate store and WinInet proxy
- `src/macos.rs` - macOS Keychain and SystemConfiguration
- `src/linux.rs` - Linux system certificate stores
- `src/proxy.rs` - System proxy detection

**Dependencies**: network_types, network_errors, tls_manager
**Platform Dependencies**:
- Windows: `windows = { version = "0.52", features = ["Win32_Networking_WinInet"] }`
- Unix: `libc = "0.2"`
**Token Budget**: 60,000-70,000
**Test Coverage**: Platform-specific loading, proxy detection, error handling

---

### Level 2: Protocol Layer (3 New Components)

#### Component 13: ftp_protocol

**Responsibility**: FTP and FTPS protocol implementation

**Public API**:
```rust
pub struct FtpClient {
    config: FtpConfig,
    control_conn: Option<TcpStream>,
    data_conn: Option<TcpStream>,
}

pub struct FtpConfig {
    pub passive_mode: bool,
    pub enable_tls: bool,
    pub timeout: Duration,
}

pub enum FtpCommand {
    List,
    Retrieve(String),
    Store(String),
    Delete(String),
    MakeDir(String),
    RemoveDir(String),
}

impl FtpClient {
    pub async fn connect(&mut self, host: &str, port: u16) -> Result<(), NetworkError>;
    pub async fn login(&mut self, username: &str, password: &str) -> Result<(), NetworkError>;
    pub async fn execute(&mut self, command: FtpCommand) -> Result<FtpResponse, NetworkError>;
    pub async fn download(&mut self, remote_path: &str) -> Result<Vec<u8>, NetworkError>;
    pub async fn upload(&mut self, remote_path: &str, data: &[u8]) -> Result<(), NetworkError>;
}
```

**Internal Structure**:
- `src/client.rs` - FTP client implementation
- `src/commands.rs` - FTP command handling
- `src/passive.rs` - Passive mode (PASV/EPSV)
- `src/active.rs` - Active mode (PORT)
- `src/tls.rs` - FTPS (FTP over TLS) support

**Dependencies**: network_types, network_errors, dns_resolver, tls_manager
**Token Budget**: 70,000-80,000
**Test Coverage**: Connect/login, passive mode, active mode, FTPS, file transfer

---

#### Component 14: wpt_harness

**Responsibility**: Web Platform Tests harness implementation

**Public API**:
```rust
pub struct WptHarness {
    test_server: WptTestServer,
    stack: Arc<NetworkStack>,
    results: Arc<Mutex<Vec<WptTestResult>>>,
}

pub struct WptTestResult {
    pub test_path: String,
    pub status: TestStatus,
    pub duration_ms: u64,
    pub error: Option<String>,
}

pub enum TestStatus {
    Pass,
    Fail,
    Timeout,
    Skip,
}

impl WptHarness {
    pub async fn run_test_suite(&self, suite: &str) -> Vec<WptTestResult>;
    pub async fn fetch(&self, url: &str, options: FetchOptions) -> Result<WptResponse, NetworkError>;
    pub async fn create_websocket(&self, url: &str) -> Result<WptWebSocket, NetworkError>;
    pub fn export_results(&self, format: ExportFormat) -> String;
}
```

**Internal Structure**:
- `src/harness.rs` - Test harness implementation
- `src/server.rs` - WPT test server integration
- `src/adapter.rs` - Adapt NetworkStack to WPT interface
- `src/reporter.rs` - Test result reporting

**Dependencies**: All protocol components, network_stack
**Token Budget**: 70,000-80,000
**Test Coverage**: Test execution, result reporting, server integration

---

#### Component 15: performance_benchmarks

**Responsibility**: Performance benchmarking suite

**Public API**:
```rust
pub struct PerformanceBenchmarks {
    stack: Arc<NetworkStack>,
    baseline: Option<Baseline>,
}

pub struct BenchmarkResult {
    pub name: String,
    pub throughput_mbps: f64,
    pub latency_ms: f64,
    pub requests_per_sec: f64,
}

impl PerformanceBenchmarks {
    pub async fn benchmark_http_versions(&self) -> Vec<BenchmarkResult>;
    pub async fn benchmark_websocket_throughput(&self) -> BenchmarkResult;
    pub async fn benchmark_connection_pool(&self) -> BenchmarkResult;
    pub async fn benchmark_cache(&self) -> BenchmarkResult;
    pub fn compare_to_baseline(&self, results: &[BenchmarkResult]) -> ComparisonReport;
}
```

**Internal Structure**:
- `src/http_bench.rs` - HTTP/1-3 benchmarks
- `src/websocket_bench.rs` - WebSocket throughput
- `src/connection_bench.rs` - Connection pooling
- `src/cache_bench.rs` - Cache performance
- `src/baseline.rs` - Baseline comparison

**Dependencies**: All protocol components, network_stack, criterion
**External Crates**: criterion = "0.5"
**Token Budget**: 60,000-70,000
**Test Coverage**: All benchmark suites, baseline comparison, result export

---

### Level 3: Integration Layer (1 Enhanced Component)

#### Enhanced Component: network_stack

**Current Status**: Basic implementation with 16 tests

**Required Enhancements**:

1. **Full NetworkStack Trait Implementation**:
```rust
impl NetworkStack {
    // ✅ Already implemented
    async fn fetch(&self, request: NetworkRequest) -> Result<NetworkResponse, NetworkError>;
    async fn connect_websocket(&self, url: Url, protocols: Vec<String>) -> Result<WebSocketConnection, NetworkError>;
    async fn create_rtc_peer_connection(&self, config: RtcConfiguration) -> Result<RtcPeerConnection, NetworkError>;

    // ❌ Need to implement
    async fn stream_response(&self, request: NetworkRequest)
        -> Result<Box<dyn Stream<Item = Result<Bytes, NetworkError>> + Send>, NetworkError>;
    async fn clear_cache(&mut self) -> Result<(), NetworkError>;
    fn get_network_status(&self) -> NetworkStatus;
    fn set_network_conditions(&mut self, conditions: NetworkConditions);
    fn cookie_store(&self) -> Arc<CookieStore>;
    fn cert_store(&self) -> Arc<CertificateStore>;
}
```

2. **Integration with New Components**:
```rust
pub struct NetworkStackImpl {
    // Existing components
    http1_client: Arc<Http1Client>,
    http2_client: Arc<Http2Client>,
    http3_client: Arc<Http3Client>,
    websocket_client: Arc<WebSocketClient>,
    webrtc_manager: Arc<WebRtcManager>,
    dns_resolver: Arc<DnsResolver>,
    tls_manager: Arc<TlsManager>,
    cookie_manager: Arc<CookieManager>,
    http_cache: Arc<HttpCache>,

    // New component integrations
    proxy_client: Arc<ProxyClient>,
    cors_validator: Arc<CorsValidator>,
    content_encoder: Arc<ContentEncoder>,
    scheduler: Arc<Mutex<RequestScheduler>>,
    bandwidth_limiter: Arc<BandwidthLimiter>,
    url_handler: Arc<UrlHandler>,
    ct_validator: Arc<CtValidator>,
    mixed_content_blocker: Arc<MixedContentBlocker>,
    csp_processor: Arc<CspProcessor>,
    cert_pinner: Arc<CertificatePinner>,
    metrics: Arc<NetworkMetrics>,
    platform_integration: Arc<PlatformIntegration>,
    ftp_client: Arc<FtpClient>,
}
```

3. **Message Bus Integration**:
```rust
// Handle incoming messages
pub enum NetworkStackMessage {
    FetchRequest { request_id: u64, request: NetworkRequest, response_channel: oneshot::Sender<NetworkResponse> },
    WebSocketConnect { request_id: u64, url: Url, protocols: Vec<String>, response_channel: oneshot::Sender<WebSocketConnection> },
    ClearCache { response_channel: oneshot::Sender<Result<(), NetworkError>> },
    GetNetworkStatus { response_channel: oneshot::Sender<NetworkStatus> },
    SetNetworkConditions { conditions: NetworkConditions },
}

// Emit outgoing events
pub enum NetworkStackEvent {
    NetworkStatusChanged(NetworkStatus),
    SecurityWarning { url: Url, warning_type: SecurityWarningType, details: String },
    PerformanceMetrics { metrics: NetworkMetrics },
}
```

4. **Full Request Pipeline**:
```rust
async fn fetch_internal(&self, request: NetworkRequest) -> Result<NetworkResponse, NetworkError> {
    // 1. Check special URLs (data:, file:)
    if request.url.scheme() == "data" {
        return self.url_handler.fetch_data_url(&request.url).await;
    }
    if request.url.scheme() == "file" {
        return self.url_handler.fetch_file_url(&request.url).await;
    }
    if request.url.scheme() == "ftp" {
        return self.ftp_client.fetch(&request).await;
    }

    // 2. Security checks
    self.mixed_content_blocker.check_request(&request, &context)?;
    self.cors_validator.validate_request(&request, origin)?;

    // 3. Check cache
    if let Some(cached) = self.http_cache.get(&request).await? {
        return Ok(cached);
    }

    // 4. Schedule request
    let request_id = self.scheduler.lock().await.schedule(request.clone(), priority);

    // 5. Wait for slot
    let request = self.scheduler.lock().await.next_request();

    // 6. Select protocol (HTTP/1-3)
    let response = match self.select_protocol(&request.url).await? {
        Protocol::Http1 => self.http1_client.fetch(request).await?,
        Protocol::Http2 => self.http2_client.fetch(request).await?,
        Protocol::Http3 => self.http3_client.fetch(request).await?,
    };

    // 7. Content decoding
    let decoded = self.content_encoder.decode(response.body, encoding).await?;

    // 8. CSP validation
    self.csp_processor.check_resource(&request.url, resource_type)?;

    // 9. Certificate pinning (if HTTPS)
    if request.url.scheme() == "https" {
        self.cert_pinner.validate_certificate(&domain, &cert)?;
    }

    // 10. Record metrics
    self.metrics.record_request(protocol, true);

    // 11. Cache response
    self.http_cache.store(&request, &response).await?;

    Ok(response)
}
```

**Estimated Additional Tokens**: +40,000-50,000 (total ~60,000-70,000)

---

## Build Order and Parallelization

### Batch 1: Core Infrastructure (Max 3 parallel)
Run in parallel (components independent):
1. proxy_support
2. cors_validator
3. content_encoding

Then run in parallel:
4. request_scheduler
5. bandwidth_limiter
6. url_handlers

**Estimated Time**: 6-8 hours total (2 batches of 3 components each)

### Batch 2: Security Components (Max 3 parallel)
Run in parallel:
7. certificate_transparency
8. mixed_content_blocker
9. csp_processor

Then run in parallel:
10. certificate_pinning
11. network_metrics
12. platform_integration

**Estimated Time**: 6-8 hours total (2 batches of 3 components each)

### Batch 3: Protocol Component (Sequential)
13. ftp_protocol (depends on Batch 1 completion)

**Estimated Time**: 2-3 hours

### Batch 4: Integration Enhancement (Sequential)
14. Enhance network_stack (depends on all components 1-13)

**Estimated Time**: 2-3 hours

### Batch 5: Testing Components (Max 2 parallel)
Run in parallel:
15. wpt_harness
16. performance_benchmarks

**Estimated Time**: 3-4 hours

**Total Estimated Time**: 19-26 hours of autonomous development

---

## Integration Points

### network_stack Integration Flow

```
NetworkRequest
    ↓
[URL Handler Check] → data:/file:/ftp: URLs
    ↓
[Security Checks] → Mixed Content, CORS, CSP
    ↓
[Cache Check] → Return cached if valid
    ↓
[Request Scheduler] → Priority queue
    ↓
[Proxy Check] → Route through proxy if configured
    ↓
[Protocol Selection] → HTTP/1, HTTP/2, HTTP/3
    ↓
[Certificate Checks] → CT validation, pinning
    ↓
[Request Execution] → DNS → Connect → TLS → Send → Receive
    ↓
[Bandwidth Limiter] → Throttle if configured
    ↓
[Content Decoder] → gzip/brotli/deflate
    ↓
[Metrics Recording] → Record timing, bytes, success
    ↓
[Cache Storage] → Store if cacheable
    ↓
NetworkResponse
```

---

## Token Budget Summary

| Component | Type | Level | Est. Tokens | Status |
|-----------|------|-------|-------------|---------|
| proxy_support | Core | 1 | 70-80k | Within optimal |
| cors_validator | Core | 1 | 60-70k | Within optimal |
| content_encoding | Core | 1 | 50-60k | Within optimal |
| request_scheduler | Core | 1 | 50-60k | Within optimal |
| bandwidth_limiter | Core | 1 | 50-60k | Within optimal |
| url_handlers | Core | 1 | 40-50k | Within optimal |
| certificate_transparency | Core | 1 | 60-70k | Within optimal |
| mixed_content_blocker | Core | 1 | 40-50k | Within optimal |
| csp_processor | Core | 1 | 60-70k | Within optimal |
| certificate_pinning | Core | 1 | 50-60k | Within optimal |
| network_metrics | Core | 1 | 50-60k | Within optimal |
| platform_integration | Core | 1 | 60-70k | Within optimal |
| ftp_protocol | Protocol | 2 | 70-80k | Within optimal |
| wpt_harness | Testing | 2 | 70-80k | Within optimal |
| performance_benchmarks | Testing | 2 | 60-70k | Within optimal |
| network_stack (enhanced) | Integration | 3 | +40-50k | Within optimal |

**Total New/Enhanced**: 830,000-1,000,000 tokens
**Average per Component**: ~55,000-67,000 tokens
**All components within optimal range (< 80,000 tokens)**

---

## Next Steps

✅ Gap analysis complete
✅ Architecture plan complete
➡️ **Next**: Token budget detailed analysis
➡️ **Then**: Update dependency graph
➡️ **Then**: Proceed to Phase 2 (Component Creation)

