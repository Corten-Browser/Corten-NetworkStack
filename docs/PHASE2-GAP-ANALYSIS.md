# Phase 2: Comprehensive Gap Analysis
**Date**: 2025-11-14
**Session**: claude/orchestrate-full-01NaRyyt4DBVag5HkCbCjHgp (resumed)
**Goal**: Implement 100% of network-stack-specification.md features

---

## Executive Summary

**Phase 1 Implementation Coverage**: ~40-50% (13 components, 202 tests passing)
**Remaining Work**: ~50-60% of specification features
**New Components Needed**: 16 additional components
**Enhanced Components**: 1 (network_stack)

---

## What Was Implemented (Phase 1)

### ✅ Protocols (5/8)
- ✅ HTTP/1.1 (http1_protocol) - 33 tests passing
- ✅ HTTP/2 (http2_protocol) - 21 tests passing
- ✅ HTTP/3/QUIC (http3_protocol) - 24 tests passing
- ✅ WebSocket (websocket_protocol) - 25 tests passing
- ✅ WebRTC (webrtc_peer, webrtc_channels) - 35 tests passing

### ✅ Core Infrastructure (7 components)
- ✅ DNS resolution with DoH (dns_resolver) - 21 tests
- ✅ TLS 1.2/1.3 (tls_manager) - 16 tests
- ✅ Cookie management (cookie_manager) - 37 tests
- ✅ HTTP caching (http_cache) - 17 tests
- ✅ Network types (network_types) - 68 tests
- ✅ Network errors (network_errors) - 48 tests
- ✅ Network stack integration (network_stack) - 16 tests

**Total**: 13 components, 202/202 tests passing (100% pass rate)

---

## Missing Features (Specification Lines Referenced)

### 1. MISSING PROTOCOLS (3 components needed)

#### ❌ FTP Protocol (Line 23)
**Status**: Not implemented
**Component Needed**: `ftp_protocol` (Level 2 - Protocol)
**Features Required**:
- Basic FTP client implementation
- Active and passive mode support
- File download capability
- File upload capability
- FTPS (FTP over TLS) support
- Directory listing
**Dependencies**: network_types, network_errors, dns_resolver, tls_manager
**Estimated Tokens**: 60,000-80,000

#### ❌ Data URLs (Line 24)
**Status**: Not implemented
**Component Needed**: `url_handlers` (Level 1 - Core)
**Features Required**:
- `data:` URL parsing
- Base64 decoding
- MIME type extraction
- Inline resource handling
**Dependencies**: network_types, network_errors
**Estimated Tokens**: 40,000 (combined with file URLs)

#### ❌ File URLs (Line 25)
**Status**: Not implemented
**Component Needed**: `url_handlers` (Level 1 - Core)
**Features Required**:
- `file:` URL parsing
- Local file access
- Security restrictions (same-origin policy)
- Directory traversal prevention
**Dependencies**: network_types, network_errors
**Estimated Tokens**: 40,000 (combined with data URLs)

---

### 2. MISSING CORE FEATURES (5 components needed)

#### ❌ Proxy Support (Lines 33, 395-398)
**Status**: Not implemented
**Component Needed**: `proxy_support` (Level 1 - Core)
**Features Required**:
- HTTP proxy client (CONNECT method)
- SOCKS5 proxy support
- Proxy authentication (Basic, Digest)
- Proxy auto-configuration (PAC)
- HTTPS tunneling through proxy
**Specification References**:
- Line 33: "Proxy support (HTTP, SOCKS5)"
- Lines 395-398: Module structure `proxy/`
- Line 439: ProxyConfig
- Line 790: ProxyError
**Dependencies**: network_types, network_errors, tls_manager, dns_resolver
**Estimated Tokens**: 70,000-80,000

#### ❌ Request Prioritization & Scheduling (Lines 35, 463-479)
**Status**: Not implemented
**Component Needed**: `request_scheduler` (Level 1 - Core)
**Features Required**:
- High priority queue (navigation, CSS, fonts)
- Medium priority queue (scripts, XHR)
- Low priority queue (images, prefetch)
- Concurrent request limiting
- Fair scheduling algorithm
- Request cancellation support
**Specification References**:
- Line 35: "Request prioritization and scheduling"
- Lines 463-479: RequestScheduler structure
- Line 107: RequestPriority field
**Dependencies**: network_types, network_errors
**Estimated Tokens**: 50,000-60,000

#### ❌ Bandwidth Throttling (Lines 36, 719)
**Status**: Not implemented
**Component Needed**: `bandwidth_limiter` (Level 1 - Core)
**Features Required**:
- Download speed limiting
- Upload speed limiting
- Network condition simulation (slow-2g, 2g, 3g, 4g)
- Latency injection
- Bandwidth usage tracking
**Specification References**:
- Line 36: "Bandwidth throttling"
- Line 79: set_network_conditions()
- Line 441: network_conditions
- Line 719: BandwidthTracker
**Dependencies**: network_types, network_errors
**Estimated Tokens**: 50,000-60,000

#### ❌ CORS Enforcement (Lines 37, 394, 1100-1105)
**Status**: Not implemented
**Component Needed**: `cors_validator` (Level 1 - Core)
**Features Required**:
- Origin header validation
- Allowed methods checking
- Preflight request handling (OPTIONS)
- Credentials mode enforcement
- Access-Control-Allow-* headers parsing
- Cross-origin blocking
**Specification References**:
- Line 37: "CORS enforcement"
- Line 394: `security/cors.rs`
- Lines 1100-1105: CORS Policy Enforcement
- Line 793: CorsError
**Dependencies**: network_types, network_errors
**Estimated Tokens**: 60,000-70,000

#### ❌ Content Encoding/Decoding (Lines 38, 403, 613-615)
**Status**: Not implemented
**Component Needed**: `content_encoding` (Level 1 - Core)
**Features Required**:
- gzip compression/decompression
- brotli (br) compression/decompression
- deflate compression/decompression
- Content-Encoding header handling
- Accept-Encoding negotiation
- Streaming decompression
**Specification References**:
- Line 38: "Content encoding/decoding (gzip, br, deflate)"
- Line 403: `utils/encoding.rs`
- Lines 613-615: Dependencies (flate2, brotli)
**Dependencies**: network_types, network_errors
**External Crates**: flate2 = "1.0", brotli = "3.4"
**Estimated Tokens**: 50,000-60,000

---

### 3. MISSING SECURITY FEATURES (5 components needed)

#### ❌ Certificate Transparency Validation (Lines 42, 1098)
**Status**: Not implemented
**Component Needed**: `certificate_transparency` (Level 1 - Core)
**Features Required**:
- CT log verification
- Signed Certificate Timestamp (SCT) validation
- CT policy enforcement
- CT log list management
- Embedded SCT extraction
**Specification References**:
- Line 42: "Certificate transparency validation"
- Line 1098: "Implement certificate transparency checks"
**Dependencies**: network_types, network_errors, tls_manager
**Estimated Tokens**: 60,000-70,000

#### ❌ Mixed Content Blocking (Lines 44, 796)
**Status**: Not implemented
**Component Needed**: `mixed_content_blocker` (Level 1 - Core)
**Features Required**:
- HTTPS context detection
- HTTP resource blocking in HTTPS pages
- Upgrade-Insecure-Requests support
- Mixed content warnings
- Active/passive content classification
**Specification References**:
- Line 44: "Mixed content blocking"
- Line 796: MixedContent error
**Dependencies**: network_types, network_errors
**Estimated Tokens**: 40,000-50,000

#### ❌ CSP Header Processing (Line 45)
**Status**: Not implemented
**Component Needed**: `csp_processor` (Level 1 - Core)
**Features Required**:
- Content-Security-Policy header parsing
- CSP directive enforcement
- CSP violation reporting
- Nonce validation
- Hash-based validation
- Unsafe-inline/unsafe-eval blocking
**Specification References**:
- Line 45: "CSP header processing"
**Dependencies**: network_types, network_errors
**Estimated Tokens**: 60,000-70,000

#### ❌ Secure Context Enforcement (Line 46)
**Status**: Partially implemented (TLS support exists, but secure context rules not enforced)
**Component Needed**: Enhancement to `mixed_content_blocker`
**Features Required**:
- Secure context determination (HTTPS, localhost, 127.0.0.1)
- Feature gating based on secure context
- Potentially trustworthy origin detection
**Specification References**:
- Line 46: "Secure context enforcement"
**Dependencies**: network_types, network_errors
**Estimated Tokens**: Included in mixed_content_blocker

#### ❌ Certificate Pinning (Lines 30, 1097)
**Status**: Not implemented
**Component Needed**: `certificate_pinning` (Level 1 - Core)
**Features Required**:
- Certificate pin storage
- Public key pinning (HPKP)
- Pin validation on connection
- Pin expiration management
- Expect-CT header support
**Specification References**:
- Line 30: "Certificate validation and pinning"
- Line 1097: "Support certificate pinning"
**Dependencies**: network_types, network_errors, tls_manager
**Estimated Tokens**: 50,000-60,000

---

### 4. MISSING API IMPLEMENTATIONS (Enhancements to network_stack)

#### ❌ stream_response() (Lines 64-65)
**Status**: Stub returning NotImplementedError
**Location**: `components/network_stack/src/lib.rs`
**Required Implementation**:
```rust
async fn stream_response(&self, request: NetworkRequest)
    -> Result<Box<dyn Stream<Item = Result<Bytes, NetworkError>> + Send>, NetworkError>
```
**Features Required**:
- Streaming response body chunks
- Backpressure handling
- Stream cancellation
- Content encoding support during streaming

#### ❌ set_network_conditions() (Line 79)
**Status**: Not implemented (method exists but does nothing)
**Required Implementation**:
```rust
fn set_network_conditions(&mut self, conditions: NetworkConditions)
```
**Features Required**:
- Apply bandwidth throttling
- Apply latency injection
- Update connection type simulation
- Integrate with bandwidth_limiter component

#### ❌ clear_cache() (Line 82)
**Status**: Not implemented
**Required Implementation**:
```rust
async fn clear_cache(&mut self) -> Result<(), NetworkError>
```
**Features Required**:
- Clear HTTP cache
- Clear DNS cache
- Clear connection pool
- Return success/failure

#### ❌ Full get_network_status() (Line 76)
**Status**: Returns stub data
**Required Implementation**:
```rust
fn get_network_status(&self) -> NetworkStatus
```
**Features Required**:
- Real online/offline detection
- Connection type detection (WiFi, Ethernet, Cellular)
- Effective connection type (slow-2g, 2g, 3g, 4g)
- Downlink bandwidth estimate
- RTT estimate
- Active request count

---

### 5. MISSING MESSAGE BUS INTEGRATION (Lines 291-344)

#### ❌ NetworkStackMessage Handling
**Status**: Not implemented
**Required Messages**:
- FetchRequest (with oneshot response)
- WebSocketConnect (with oneshot response)
- ClearCache
- GetNetworkStatus
- SetNetworkConditions

#### ❌ NetworkStackEvent Emission
**Status**: Not implemented
**Required Events**:
- NetworkStatusChanged
- SecurityWarning (with warning type and details)
- PerformanceMetrics

---

### 6. MISSING INFRASTRUCTURE COMPONENTS (2 components needed)

#### ❌ Network Metrics Collection (Lines 1201-1276)
**Status**: Not implemented
**Component Needed**: `network_metrics` (Level 1 - Core)
**Features Required**:
- Request counters (total, success, failed)
- Bandwidth metrics (bytes sent/received)
- Connection metrics (active, idle, reused)
- Cache metrics (hits, misses, size)
- Performance metrics (avg DNS time, connect time, TTFB)
- Protocol distribution (HTTP/1-3, WebSocket, WebRTC)
- Prometheus export format
**Specification References**:
- Lines 1201-1276: NetworkMetrics structure and collection
**Dependencies**: network_types
**Estimated Tokens**: 50,000-60,000

#### ❌ Platform Integration (Lines 1107-1123)
**Status**: Not implemented
**Component Needed**: `platform_integration` (Level 1 - Core)
**Features Required**:
- **Windows**: Certificate store access via Windows APIs
- **macOS**: Keychain certificate access
- **Linux**: System certificate store via rustls-native-certs (already used)
- **All platforms**: System proxy configuration detection
**Specification References**:
- Lines 1107-1123: Platform-Specific Implementations
- Lines 638-642: Platform-specific dependencies
**Dependencies**: network_types, network_errors, tls_manager
**Platform Dependencies**:
- Windows: `windows = { version = "0.52", features = ["Win32_Networking_WinInet"] }`
- Unix: `libc = "0.2"`
**Estimated Tokens**: 60,000-70,000

---

### 7. MISSING TEST SUITES (3 components needed)

#### ❌ Web Platform Tests (WPT) Integration (Lines 1279-1805)
**Status**: Not implemented
**Component Needed**: `wpt_harness` (Level 2 - Testing)
**Features Required**:
- WPT test harness implementation
- Test server integration
- Result reporting and tracking
- Test suite execution automation
**Target Pass Rates**:
- /wpt/fetch/: 90% (800+ tests)
- /wpt/xhr/: 90% (400+ tests)
- /wpt/websockets/: 95% (200+ tests)
- /wpt/webrtc/: 85% (500+ tests)
- /wpt/cors/: 95% (300+ tests)
- /wpt/mixed-content/: 100% (100+ tests)
- /wpt/content-security-policy/: 95% (200+ tests)
**Specification References**:
- Lines 1281-1343: WPT Test Coverage
- Lines 1296-1343: WPT Test Harness Implementation
**Dependencies**: All protocol and security components
**Estimated Tokens**: 70,000-80,000

#### ❌ Performance Benchmarks (Lines 1042-1063, 1382-1424)
**Status**: Not implemented
**Component Needed**: `performance_benchmarks` (Level 2 - Testing)
**Features Required**:
- Criterion-based benchmarks
- HTTP version comparison (HTTP/1.1 vs HTTP/2 vs HTTP/3)
- WebSocket throughput testing
- Connection pool benchmarks
- Cache performance testing
**Performance Targets**:
- DNS resolution: < 50ms (cached), < 200ms (uncached)
- TLS handshake: < 100ms (TLS 1.3)
- First byte: < 200ms (local), < 500ms (remote)
- HTTP/1.1 throughput: > 100 Mbps
- HTTP/2 throughput: > 200 Mbps
- HTTP/3 throughput: > 300 Mbps
- WebSocket throughput: > 50 Mbps
**Specification References**:
- Lines 1042-1063: Performance Requirements
- Lines 1382-1424: Performance Benchmarks code examples
**Dependencies**: All protocol components
**Estimated Tokens**: 60,000-70,000

#### ❌ Security Test Suite (Lines 1761-1768)
**Status**: Not implemented
**Features Required**:
- Certificate validation tests
- HSTS enforcement tests
- Mixed content blocking tests (100% WPT pass required)
- Secure cookie protection tests
- TLS version enforcement tests
- CSP header enforcement tests
- CORS violation blocking tests
**Specification References**:
- Lines 1761-1768: Security Test Suite validation checklist
**Dependencies**: All security components
**Estimated Tokens**: Included in component test suites

---

## New Components Summary

### Components to Create (16 new components)

| # | Component Name | Type | Level | Est. Tokens | Est. Lines | Dependencies |
|---|----------------|------|-------|-------------|------------|--------------|
| 1 | proxy_support | Core | 1 | 70-80k | 7-8k | network_types, network_errors, tls_manager, dns_resolver |
| 2 | cors_validator | Core | 1 | 60-70k | 6-7k | network_types, network_errors |
| 3 | content_encoding | Core | 1 | 50-60k | 5-6k | network_types, network_errors |
| 4 | request_scheduler | Core | 1 | 50-60k | 5-6k | network_types, network_errors |
| 5 | bandwidth_limiter | Core | 1 | 50-60k | 5-6k | network_types, network_errors |
| 6 | url_handlers | Core | 1 | 40-50k | 4-5k | network_types, network_errors |
| 7 | certificate_transparency | Core | 1 | 60-70k | 6-7k | network_types, network_errors, tls_manager |
| 8 | mixed_content_blocker | Core | 1 | 40-50k | 4-5k | network_types, network_errors |
| 9 | csp_processor | Core | 1 | 60-70k | 6-7k | network_types, network_errors |
| 10 | certificate_pinning | Core | 1 | 50-60k | 5-6k | network_types, network_errors, tls_manager |
| 11 | network_metrics | Core | 1 | 50-60k | 5-6k | network_types |
| 12 | platform_integration | Core | 1 | 60-70k | 6-7k | network_types, network_errors, tls_manager |
| 13 | ftp_protocol | Protocol | 2 | 70-80k | 7-8k | network_types, network_errors, dns_resolver, tls_manager |
| 14 | wpt_harness | Testing | 2 | 70-80k | 7-8k | All protocol components |
| 15 | performance_benchmarks | Testing | 2 | 60-70k | 6-7k | All protocol components |

**Total New Components**: 15 components
**Total Estimated Tokens**: 830,000-1,000,000 tokens
**Total Estimated Lines**: 83,000-100,000 lines

### Components to Enhance

| Component | Current Status | Enhancement Needed | Est. Additional Tokens |
|-----------|---------------|-------------------|----------------------|
| network_stack | Basic integration (16 tests) | Full integration with all new components, complete NetworkStack trait implementation, message bus integration | +40,000-50,000 |

---

## Token Budget Analysis

### Existing Components (13 components)
**Total Tokens**: ~850,000 tokens (estimated)
**Status**: All within limits (< 120,000 tokens per component)

### New Components (15 components)
**Total Tokens**: 830,000-1,000,000 tokens
**Average per Component**: 55,000-67,000 tokens
**Status**: All within optimal range (< 80,000 tokens per component)

### Enhanced Component (1 component)
**network_stack Enhancement**: +40,000-50,000 tokens
**Total network_stack after enhancement**: ~60,000-70,000 tokens

### Overall Project
**Current Total**: ~850,000 tokens (13 components)
**After Phase 2**: ~1,720,000-1,900,000 tokens (28 components + 1 enhanced)
**Average per Component**: ~61,000-68,000 tokens
**Status**: All components well within limits

---

## Dependency Graph Updates

### Current Dependency Levels (Phase 1)

**Level 0 (Base)**:
- network_types
- network_errors

**Level 1 (Core)**:
- dns_resolver (depends on Level 0)
- tls_manager (depends on Level 0)
- cookie_manager (depends on Level 0)
- http_cache (depends on Level 0)

**Level 2 (Protocol)**:
- http1_protocol (depends on Levels 0-1)
- http2_protocol (depends on Levels 0-1)
- http3_protocol (depends on Levels 0-1)
- websocket_protocol (depends on Levels 0-1)
- webrtc_peer (depends on Levels 0-1)
- webrtc_channels (depends on Levels 0-1)

**Level 3 (Integration)**:
- network_stack (depends on all levels)

### New Dependency Levels (Phase 2)

**Level 0 (Base)**: No changes
- network_types
- network_errors

**Level 1 (Core)**: +12 new components
- dns_resolver
- tls_manager
- cookie_manager
- http_cache
- **+ proxy_support** (new)
- **+ cors_validator** (new)
- **+ content_encoding** (new)
- **+ request_scheduler** (new)
- **+ bandwidth_limiter** (new)
- **+ url_handlers** (new)
- **+ certificate_transparency** (new)
- **+ mixed_content_blocker** (new)
- **+ csp_processor** (new)
- **+ certificate_pinning** (new)
- **+ network_metrics** (new)
- **+ platform_integration** (new)

**Level 2 (Protocol)**: +3 new components
- http1_protocol
- http2_protocol
- http3_protocol
- websocket_protocol
- webrtc_peer
- webrtc_channels
- **+ ftp_protocol** (new)
- **+ wpt_harness** (new - testing)
- **+ performance_benchmarks** (new - testing)

**Level 3 (Integration)**: Enhanced
- network_stack (enhanced with all new components)

---

## Build Order for New Components

### Batch 1: Core Infrastructure (can run in parallel)
1. proxy_support
2. cors_validator
3. content_encoding
4. request_scheduler
5. bandwidth_limiter
6. url_handlers

### Batch 2: Security Components (can run in parallel, after Batch 1)
7. certificate_transparency
8. mixed_content_blocker
9. csp_processor
10. certificate_pinning

### Batch 3: Metrics and Platform (can run in parallel, after Batch 1)
11. network_metrics
12. platform_integration

### Batch 4: Protocols (after Batch 1 complete)
13. ftp_protocol

### Batch 5: Integration (after all components complete)
14. Enhance network_stack

### Batch 6: Testing (after all components including enhanced network_stack)
15. wpt_harness
16. performance_benchmarks

---

## Success Criteria

### Component Completion
- [ ] All 15 new components created
- [ ] network_stack enhanced with full integration
- [ ] All components pass 12-check verification (v0.5.0)
- [ ] 100% test pass rate for all components

### Feature Coverage
- [ ] FTP protocol support
- [ ] Data/File URL handling
- [ ] Proxy support (HTTP, SOCKS5)
- [ ] Request scheduling with prioritization
- [ ] Bandwidth throttling
- [ ] CORS enforcement
- [ ] Content encoding/decoding (gzip, br, deflate)
- [ ] Certificate transparency validation
- [ ] Mixed content blocking
- [ ] CSP header processing
- [ ] Certificate pinning
- [ ] Network metrics collection
- [ ] Platform-specific integrations

### API Completeness
- [ ] stream_response() fully implemented
- [ ] set_network_conditions() functional
- [ ] clear_cache() implemented
- [ ] get_network_status() returns real data
- [ ] Message bus integration complete

### Test Coverage
- [ ] WPT tests: 90-95% pass rate (2,700+ tests)
- [ ] Performance benchmarks: Within 2x Chrome performance
- [ ] Security tests: 100% pass rate

### Overall
- [ ] **100% of specification features implemented**
- [ ] All quality gates passed
- [ ] Integration tests: 100% pass rate
- [ ] Final completion report generated

---

**Next Step**: Proceed to Phase 1 Architecture Planning for all 16 new components

