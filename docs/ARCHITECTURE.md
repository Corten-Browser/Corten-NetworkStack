# Network Stack Architecture Design

## Project Overview
**Component**: Corten-NetworkStack
**Version**: 0.1.0 (pre-release)
**Estimated Size**: 105,000 lines of code
**Language**: Rust
**Architecture**: Modular library with 13 components

## Design Principles

1. **Token Budget Compliance**: Each component stays within 110,000 token limit (~11,000 lines)
2. **Dependency Hierarchy**: Clear layering (base → core → feature → integration)
3. **Parallel Development**: Components grouped by dependency level for concurrent development
4. **Rust Best Practices**: Each component is a separate crate with clear public API

## Component Architecture

### Level 0: Base Layer (Shared Types)
**Purpose**: Foundation types with no external dependencies
**Development**: Phase 1 (parallel: 2 agents)

| Component | Size | Responsibility | Depends On |
|-----------|------|----------------|------------|
| `network_types` | ~6,000 lines | NetworkRequest, NetworkResponse, enums, traits | None |
| `network_errors` | ~4,000 lines | Error types, NetworkError enum, Result types | None |

**Total**: 10,000 lines, 2 components

### Level 1: Core Layer (Infrastructure)
**Purpose**: Core infrastructure components
**Development**: Phase 2 (parallel: 4 agents)

| Component | Size | Responsibility | Depends On |
|-----------|------|----------------|------------|
| `dns_resolver` | ~6,000 lines | DNS resolution, DNS-over-HTTPS | network_types, network_errors |
| `tls_manager` | ~10,000 lines | TLS configuration, certificate validation | network_types, network_errors |
| `cookie_manager` | ~5,000 lines | Cookie storage, jar, parser | network_types, network_errors |
| `http_cache` | ~10,000 lines | HTTP cache storage, policies | network_types, network_errors |

**Total**: 31,000 lines, 4 components

### Level 2: Protocol Layer (Feature)
**Purpose**: Protocol implementations
**Development**: Phase 3-4 (parallel: 2-4 agents)

#### Phase 3 (HTTP Core)
| Component | Size | Responsibility | Depends On |
|-----------|------|----------------|------------|
| `http1_protocol` | ~12,000 lines | HTTP/1.1 client, connection pooling | network_types, network_errors, dns_resolver, tls_manager, cookie_manager, http_cache |
| `http2_protocol` | ~12,000 lines | HTTP/2 client, multiplexing | network_types, network_errors, dns_resolver, tls_manager, cookie_manager, http_cache |

#### Phase 4 (Advanced Protocols)
| Component | Size | Responsibility | Depends On |
|-----------|------|----------------|------------|
| `http3_protocol` | ~10,000 lines | HTTP/3 and QUIC implementation | network_types, network_errors, dns_resolver, tls_manager |
| `websocket_protocol` | ~8,000 lines | WebSocket client, frame handling | network_types, network_errors, tls_manager |
| `webrtc_peer` | ~10,000 lines | Peer connections, ICE, STUN/TURN | network_types, network_errors, tls_manager |
| `webrtc_channels` | ~5,000 lines | Data channels, SCTP transport | network_types, network_errors, webrtc_peer |

**Total**: 57,000 lines, 6 components

### Level 3: Integration Layer (Application)
**Purpose**: Main NetworkStack implementation that orchestrates all protocols
**Development**: Phase 5 (single agent)

| Component | Size | Responsibility | Depends On |
|-----------|------|----------------|------------|
| `network_stack` | ~7,000 lines | NetworkStack trait implementation, message bus integration, protocol orchestration | ALL level 0-2 components |

**Total**: 7,000 lines, 1 component

## Overall Summary

- **Total Components**: 13
- **Total Estimated Lines**: 105,000
- **Maximum Component Size**: 12,000 lines (~120,000 tokens - within limit)
- **Development Phases**: 5 phases
- **Maximum Parallel Agents**: 4 (respects 7-agent limit)

## Development Order

```
Phase 1 (Level 0 - Base):
  └─ [Parallel: 2 agents]
     ├─ network_types
     └─ network_errors

Phase 2 (Level 1 - Core):
  └─ [Parallel: 4 agents]
     ├─ dns_resolver
     ├─ tls_manager
     ├─ cookie_manager
     └─ http_cache

Phase 3 (Level 2a - HTTP Core):
  └─ [Parallel: 2 agents]
     ├─ http1_protocol
     └─ http2_protocol

Phase 4 (Level 2b - Advanced):
  └─ [Parallel: 4 agents]
     ├─ http3_protocol
     ├─ websocket_protocol
     ├─ webrtc_peer
     └─ webrtc_channels

Phase 5 (Level 3 - Integration):
  └─ [Single agent]
     └─ network_stack
```

## Component Dependencies Graph

```
network_stack (Integration)
    ├─ http1_protocol ───┬─ dns_resolver ───┬─ network_types
    ├─ http2_protocol ───┤                  │
    ├─ http3_protocol ───┤                  ├─ network_errors
    ├─ websocket_protocol┤                  │
    ├─ webrtc_peer ──────┤                  │
    ├─ webrtc_channels ──┤                  │
    │                    ├─ tls_manager ────┤
    │                    ├─ cookie_manager ─┤
    │                    └─ http_cache ─────┘
```

## Technology Stack Per Component

All components use:
- **Language**: Rust 2021 edition (1.75+)
- **Async Runtime**: Tokio
- **Build System**: Cargo
- **Testing**: cargo test, criterion (benchmarks)

### Key Dependencies by Component

**network_types**:
- serde, url, bytes, uuid, chrono

**network_errors**:
- thiserror, anyhow

**dns_resolver**:
- hickory-resolver (DoH support)

**tls_manager**:
- rustls, tokio-rustls, webpki-roots

**cookie_manager**:
- cookie_store, cookie

**http_cache**:
- lru, cached

**http1_protocol / http2_protocol**:
- hyper, hyper-util, h2, tower

**http3_protocol**:
- quinn, h3, h3-quinn

**websocket_protocol**:
- tokio-tungstenite, tungstenite

**webrtc_peer / webrtc_channels**:
- webrtc, webrtc-ice, webrtc-dtls, webrtc-sctp

**network_stack**:
- All above dependencies (integrator)

## Token Budget Analysis

| Component | Lines | Tokens (est.) | Status |
|-----------|-------|---------------|--------|
| network_types | 6,000 | 60,000 | ✅ Optimal |
| network_errors | 4,000 | 40,000 | ✅ Optimal |
| dns_resolver | 6,000 | 60,000 | ✅ Optimal |
| tls_manager | 10,000 | 100,000 | ⚠️ Warning threshold |
| cookie_manager | 5,000 | 50,000 | ✅ Optimal |
| http_cache | 10,000 | 100,000 | ⚠️ Warning threshold |
| http1_protocol | 12,000 | 120,000 | 🟠 Near split trigger |
| http2_protocol | 12,000 | 120,000 | 🟠 Near split trigger |
| http3_protocol | 10,000 | 100,000 | ⚠️ Warning threshold |
| websocket_protocol | 8,000 | 80,000 | ✅ Optimal |
| webrtc_peer | 10,000 | 100,000 | ⚠️ Warning threshold |
| webrtc_channels | 5,000 | 50,000 | ✅ Optimal |
| network_stack | 7,000 | 70,000 | ✅ Optimal |

**Legend**:
- ✅ Optimal: < 70,000 tokens
- ⚠️ Warning: 70,000-90,000 tokens (monitor growth)
- 🟠 Near limit: 90,000-110,000 tokens (OK but watch closely)

All components are within acceptable limits with safety margin.

## File Structure

```
Corten-NetworkStack/
├── components/
│   ├── network_types/
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── request.rs
│   │   │   ├── response.rs
│   │   │   ├── enums.rs
│   │   │   └── traits.rs
│   │   ├── tests/
│   │   ├── Cargo.toml
│   │   ├── CLAUDE.md
│   │   └── README.md
│   ├── network_errors/
│   ├── dns_resolver/
│   ├── tls_manager/
│   ├── cookie_manager/
│   ├── http_cache/
│   ├── http1_protocol/
│   ├── http2_protocol/
│   ├── http3_protocol/
│   ├── websocket_protocol/
│   ├── webrtc_peer/
│   ├── webrtc_channels/
│   └── network_stack/
├── contracts/
│   ├── network_types.yaml
│   ├── dns_resolver.yaml
│   ├── ... (one per component)
│   └── network_stack.yaml
├── shared-libs/
│   └── README.md
├── tests/
│   ├── integration/
│   ├── e2e/
│   └── utilities/
├── docs/
│   ├── ARCHITECTURE.md (this file)
│   └── adr/
├── orchestration/
└── network-stack-specification.md
```

## Quality Standards Per Component

Each component must meet:
- ✅ Test coverage ≥ 80%
- ✅ All tests passing (100% pass rate)
- ✅ TDD compliance (git history shows Red-Green-Refactor)
- ✅ Linting: zero errors (cargo clippy)
- ✅ Formatting: 100% compliant (cargo fmt)
- ✅ Documentation: All public APIs documented
- ✅ Security: No hardcoded secrets, all input validated
- ✅ Contract compliance: Implements contract exactly

## Integration Testing Strategy

### Phase 1-2: Component Tests
- Each component tests in isolation
- Mock dependencies where needed
- 100% unit test pass rate required

### Phase 3-4: Protocol Tests
- HTTP clients tested with wiremock
- WebSocket tested with echo servers
- WebRTC tested with peer simulation
- 100% integration test pass rate required

### Phase 5: System Integration
- Full NetworkStack end-to-end tests
- Cross-protocol interaction tests
- Performance benchmarks
- 100% integration test pass rate **MANDATORY**

## Performance Targets

### Latency
- DNS resolution: < 50ms (cached), < 200ms (uncached)
- TLS handshake: < 100ms (TLS 1.3)
- First byte: < 200ms (local), < 500ms (remote)
- WebSocket connection: < 300ms

### Throughput
- HTTP/1.1: > 100 Mbps
- HTTP/2: > 200 Mbps (multiplexed)
- HTTP/3: > 300 Mbps
- WebSocket: > 50 Mbps

## Security Requirements

### All Components
- ✅ TLS 1.2/1.3 only (no TLS 1.0/1.1)
- ✅ Certificate validation required
- ✅ HSTS enforcement
- ✅ Mixed content blocking
- ✅ CORS validation
- ✅ No hardcoded secrets
- ✅ Input sanitization

## Version Control

- **Current Version**: 0.1.0 (pre-release)
- **Lifecycle State**: pre-release
- **Breaking Changes**: Encouraged (0.x.x policy)
- **API Locked**: No (flexible during development)

## Notes for Development

1. **Start with base layer**: network_types and network_errors are foundation
2. **Parallel development**: Use max 4 agents per phase (within 7-agent limit)
3. **TDD enforcement**: Tests before implementation, always
4. **Token monitoring**: Check component sizes before major additions
5. **Contract-first**: Generate contracts before implementing components
6. **Integration gate**: 100% integration test pass rate required to proceed

---

**Architecture Version**: 1.0
**Date**: 2025-11-14
**Status**: Approved for implementation
