# Corten-NetworkStack Architecture Map

**Date**: 2025-11-14
**Purpose**: Document component dependencies and integration points for cross-component testing

## Component Hierarchy

The Corten-NetworkStack follows a 4-level component hierarchy:

### Level 0: Base Libraries (No Dependencies)
1. **network_types** - Core types and data structures
   - Provides: `NetworkRequest`, `NetworkResponse`, `HttpMethod`, enums
   - Used by: ALL other components

2. **network_errors** - Error types and handling
   - Provides: `NetworkError` enum
   - Used by: ALL other components

### Level 1: Core Libraries (Depend on Base Only)
3. **dns_resolver** - DNS resolution and caching
   - Depends on: network_types, network_errors
   - Provides: `DnsResolver` trait, `DnsCache`
   - Used by: http1_protocol, http2_protocol, http3_protocol, network_stack

4. **tls_manager** - TLS/SSL configuration and certificate handling
   - Depends on: network_types, network_errors
   - Provides: `TlsConfig`, `CertificateStore`, `HstsStore`
   - Used by: http1_protocol, http2_protocol, http3_protocol, websocket_protocol, network_stack

5. **cookie_manager** - Cookie storage and management
   - Depends on: network_types, network_errors
   - Provides: `CookieStore`, `CookieJar`
   - Used by: http1_protocol, http2_protocol, http3_protocol, network_stack

6. **http_cache** - HTTP response caching
   - Depends on: network_types, network_errors
   - Provides: `HttpCache`, `CacheConfig`
   - Used by: http1_protocol, http2_protocol, http3_protocol, network_stack

### Level 2: Protocol Libraries (Depend on Base + Core)
7. **http1_protocol** - HTTP/1.1 implementation
   - Depends on: network_types, network_errors, dns_resolver, tls_manager, cookie_manager, http_cache
   - Provides: `Http1Client`, connection pooling
   - Used by: network_stack

8. **http2_protocol** - HTTP/2 implementation
   - Depends on: network_types, network_errors, dns_resolver, tls_manager, cookie_manager, http_cache
   - Provides: `Http2Client`, multiplexing
   - Used by: network_stack

9. **http3_protocol** - HTTP/3 over QUIC implementation
   - Depends on: network_types, network_errors, dns_resolver, tls_manager, cookie_manager, http_cache
   - Provides: `Http3Client`, QUIC transport
   - Used by: network_stack

10. **websocket_protocol** - WebSocket implementation
    - Depends on: network_types, network_errors, tls_manager
    - Provides: `WebSocketConnection`, `WebSocketClient`
    - Used by: network_stack

11. **webrtc_peer** - WebRTC peer connection
    - Depends on: network_types, network_errors, tls_manager
    - Provides: `RtcPeerConnection`, ICE handling
    - Used by: network_stack

12. **webrtc_channels** - WebRTC data channels
    - Depends on: network_types, network_errors, webrtc_peer
    - Provides: `DataChannel`, messaging
    - Used by: network_stack

### Level 3: Integration Layer
13. **network_stack** - Main integration layer
    - Depends on: ALL components
    - Provides: `NetworkStack` trait, unified API
    - Used by: External applications

## Critical Data Flows

### 1. DNS → TLS → HTTP Flow
```
User Request
    ↓
dns_resolver.resolve(hostname) → Vec<IpAddr>
    ↓
tls_manager.configure_tls() → TlsConfig
    ↓
http1/2/3_protocol.fetch(request) → NetworkResponse
```

**Integration Points:**
- DNS must resolve hostnames before HTTP connection
- TLS configuration applied during connection setup
- HTTP clients use resolved IPs and TLS config

### 2. Cookie Manager → HTTP Clients Flow
```
Incoming Response with Set-Cookie
    ↓
cookie_manager.add_cookie(cookie, url)
    ↓
Outgoing Request
    ↓
cookie_manager.get_cookies(url) → Vec<Cookie>
    ↓
HTTP Client adds cookies to request headers
```

**Integration Points:**
- HTTP responses trigger cookie storage
- HTTP requests retrieve and apply cookies
- Cookie scope/domain validation

### 3. HTTP Cache → HTTP Clients Flow
```
Outgoing Request
    ↓
http_cache.get(request) → Option<CachedResponse>
    ↓ (cache miss)
HTTP Client fetches from network
    ↓
http_cache.store(request, response)
    ↓ (cache hit next time)
Return cached response
```

**Integration Points:**
- Cache checked before network request
- Cache headers (ETag, Cache-Control) respected
- Conditional requests (If-None-Match) sent

### 4. WebSocket Integration Flow
```
User connects WebSocket
    ↓
tls_manager provides TLS config (for wss://)
    ↓
websocket_protocol.connect(url) → WebSocketConnection
    ↓
websocket_protocol.send/recv messages
```

**Integration Points:**
- TLS required for secure WebSocket (wss://)
- Upgrade from HTTP to WebSocket protocol

### 5. Network Stack Orchestration Flow
```
network_stack.fetch(request)
    ↓
Route to appropriate protocol handler:
  - HTTP/1.1 → http1_protocol
  - HTTP/2 → http2_protocol
  - HTTP/3 → http3_protocol
  - WebSocket → websocket_protocol
  - WebRTC → webrtc_peer/channels
    ↓
Each protocol uses:
  - dns_resolver for hostname resolution
  - tls_manager for security
  - cookie_manager for cookies
  - http_cache for caching
    ↓
Return unified NetworkResponse
```

**Integration Points:**
- Protocol selection based on URL scheme and negotiation
- Shared DNS, TLS, cookie, and cache infrastructure
- Consistent error handling across protocols

## Component Interface Contracts

### Exports from Base Components

**network_types exports:**
- `NetworkRequest`, `NetworkResponse`
- `HttpMethod`, `RequestBody`, `ResponseBody`
- `RequestMode`, `CacheMode`, `RedirectMode`

**network_errors exports:**
- `NetworkError` enum
- `NetworkResult<T>` type alias

### Exports from Core Components

**dns_resolver exports:**
- `DnsResolver` trait with `resolve()` method
- `DnsCache` for caching resolutions

**tls_manager exports:**
- `TlsConfig` for ALPN and certificate configuration
- `CertificateStore` for certificate management
- `HstsStore` for HSTS enforcement

**cookie_manager exports:**
- `CookieStore` with `add_cookie()`, `get_cookies()`, `clear()`
- `CookieJar` for cookie management

**http_cache exports:**
- `HttpCache` with `get()`, `store()`, `clear()`
- `CacheConfig` for cache settings

### Exports from Protocol Components

**http1_protocol exports:**
- `Http1Client` with `fetch()`, `stream_response()`
- `ConnectionPool` for connection reuse

**http2_protocol exports:**
- `Http2Client` with `fetch()`, `stream_response()`
- Multiplexing support

**http3_protocol exports:**
- `Http3Client` with `fetch()`, `stream_response()`
- QUIC transport

**websocket_protocol exports:**
- `WebSocketConnection` with `send()`, `recv()`, `close()`
- `WebSocketClient` with `connect()`

**webrtc_peer exports:**
- `RtcPeerConnection` with signaling methods
- ICE candidate handling

**webrtc_channels exports:**
- `DataChannel` for peer-to-peer messaging

### Exports from Integration Layer

**network_stack exports:**
- `NetworkStack` trait - unified interface
- `NetworkStackImpl` - concrete implementation
- `NetworkConfig` - configuration

## Integration Test Coverage Plan

### Test Suite 1: DNS → TLS → HTTP Integration
- DNS resolution before HTTP request
- TLS configuration applied to HTTPS requests
- HSTS enforcement redirects HTTP → HTTPS
- Certificate validation during TLS handshake

### Test Suite 2: Cookie Manager → HTTP Clients
- Set-Cookie headers stored in cookie manager
- Cookies sent with subsequent requests
- Cookie domain/path matching
- Secure cookie handling (Secure, HttpOnly flags)

### Test Suite 3: HTTP Cache → HTTP Clients
- Cache-Control headers respected
- ETag-based conditional requests
- Cache hit returns cached response
- Cache miss triggers network request

### Test Suite 4: WebSocket Protocol Integration
- WebSocket upgrade from HTTP
- TLS applied to secure WebSocket (wss://)
- Bidirectional message exchange
- Proper connection closure

### Test Suite 5: WebRTC Integration
- Peer connection establishment
- Data channel creation and messaging
- ICE candidate exchange
- DTLS for secure transport

### Test Suite 6: Network Stack End-to-End
- Protocol selection (HTTP/1.1, HTTP/2, HTTP/3, WebSocket, WebRTC)
- Shared infrastructure (DNS, TLS, cookies, cache) across protocols
- Error handling consistency
- Configuration propagation

## Testing Strategy

### Unit Tests (Per Component)
✅ Already implemented in each component's `tests/` directory
- Test individual component functionality in isolation
- Mock external dependencies

### Integration Tests (Cross-Component)
⚠️ TO BE IMPLEMENTED in `/tests/integration/`
- Test real component interactions
- Use actual implementations (NO MOCKING of internal components)
- Verify data flows between components
- Test contract compliance

### End-to-End Tests
⚠️ TO BE IMPLEMENTED in `/tests/e2e/`
- Test complete workflows through network_stack
- Verify system behavior with real network operations
- Test with actual HTTP servers (where feasible)

## Component Dependency Graph

```
                    ┌─────────────────┐
                    │ network_stack   │ (Level 3)
                    └────────┬────────┘
                             │
        ┌────────────────────┼────────────────────┐
        │                    │                    │
  ┌─────▼─────┐      ┌──────▼──────┐      ┌─────▼──────┐
  │ http1/2/3 │      │  websocket  │      │   webrtc   │ (Level 2)
  └─────┬─────┘      └──────┬──────┘      └─────┬──────┘
        │                   │                    │
  ┌─────┴──────┬───────┬────┴────┬──────────────┘
  │            │       │         │
┌─▼──┐  ┌─────▼─┐  ┌──▼───┐  ┌──▼────┐
│dns │  │ tls   │  │cookie│  │ cache │ (Level 1)
└─┬──┘  └───┬───┘  └──┬───┘  └───┬───┘
  │         │         │          │
  └─────────┴─────────┴──────────┘
                │
     ┌──────────┴──────────┐
     │                     │
┌────▼─────┐       ┌───────▼──────┐
│  types   │       │    errors    │ (Level 0)
└──────────┘       └──────────────┘
```

## Critical Integration Points

1. **DNS Resolution Must Precede Connection**
   - HTTP clients MUST call dns_resolver before connecting
   - Failure: Connection attempts to unresolved hostnames

2. **TLS Must Be Configured Before HTTPS**
   - HTTP clients MUST apply TLS config for HTTPS URLs
   - Failure: Insecure connections or TLS handshake failures

3. **Cookies Must Persist Across Requests**
   - HTTP responses MUST store cookies
   - HTTP requests MUST retrieve and send cookies
   - Failure: Authentication/session failures

4. **Cache Must Check Before Network**
   - HTTP clients MUST check cache before network request
   - Cache MUST respect HTTP caching headers
   - Failure: Unnecessary network requests, stale data

5. **Protocol Selection Must Be Correct**
   - network_stack MUST route to correct protocol handler
   - Failure: Wrong protocol used, connection failures

## Test Data Requirements

For integration testing, we need:
- Test DNS records (use public DNS like example.com)
- TLS certificates (use test certificates or public HTTPS endpoints)
- HTTP test servers (use public test endpoints like httpbin.org)
- WebSocket test servers (use public WebSocket echo servers)
- Sample cookies with various attributes
- Sample HTTP responses with cache headers

## Success Criteria

Integration tests are complete when:
- ✅ All component pairs that communicate have integration tests
- ✅ All critical data flows are tested
- ✅ 100% integration test pass rate
- ✅ NO MOCKING of internal components (only external services)
- ✅ Tests verify actual component communication
- ✅ Contract compliance verified for all interfaces
- ✅ Error scenarios tested (DNS failures, TLS errors, etc.)
