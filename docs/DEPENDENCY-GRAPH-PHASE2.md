# Dependency Graph - Phase 2 Complete
**Date**: 2025-11-14
**Session**: claude/orchestrate-full-01NaRyyt4DBVag5HkCbCjHgp (resumed)
**Total Components**: 28 (13 existing + 15 new)

---

## Dependency Hierarchy (4 Levels)

### Level 0: Base Layer (2 components) - No Dependencies

```
network_types (existing)
network_errors (existing)
```

**Dependencies**: None
**Purpose**: Core types, traits, and error handling
**Imports**: Cannot import from other components

---

### Level 1: Core Layer (16 components) - Depend on Level 0 Only

#### Existing Components (4)

```
dns_resolver (existing)
├─ network_types
└─ network_errors

tls_manager (existing)
├─ network_types
└─ network_errors

cookie_manager (existing)
├─ network_types
└─ network_errors

http_cache (existing)
├─ network_types
└─ network_errors
```

#### New Components (12)

```
proxy_support (new)
├─ network_types
├─ network_errors
├─ tls_manager (Level 1, allowed)
└─ dns_resolver (Level 1, allowed)

cors_validator (new)
├─ network_types
└─ network_errors

content_encoding (new)
├─ network_types
└─ network_errors

request_scheduler (new)
├─ network_types
└─ network_errors

bandwidth_limiter (new)
├─ network_types
└─ network_errors

url_handlers (new)
├─ network_types
└─ network_errors

certificate_transparency (new)
├─ network_types
├─ network_errors
└─ tls_manager (Level 1, allowed)

mixed_content_blocker (new)
├─ network_types
└─ network_errors

csp_processor (new)
├─ network_types
└─ network_errors

certificate_pinning (new)
├─ network_types
├─ network_errors
└─ tls_manager (Level 1, allowed)

network_metrics (new)
└─ network_types

platform_integration (new)
├─ network_types
├─ network_errors
└─ tls_manager (Level 1, allowed)
```

**Note**: Level 1 components can depend on other Level 1 components (same level) as long as no circular dependencies exist.

---

### Level 2: Protocol Layer (9 components) - Depend on Levels 0-1

#### Existing Components (6)

```
http1_protocol (existing)
├─ network_types
├─ network_errors
├─ dns_resolver
├─ tls_manager
├─ cookie_manager
└─ http_cache

http2_protocol (existing)
├─ network_types
├─ network_errors
├─ dns_resolver
├─ tls_manager
├─ cookie_manager
└─ http_cache

http3_protocol (existing)
├─ network_types
├─ network_errors
├─ dns_resolver
├─ tls_manager
├─ cookie_manager
└─ http_cache

websocket_protocol (existing)
├─ network_types
├─ network_errors
├─ dns_resolver
└─ tls_manager

webrtc_peer (existing)
├─ network_types
├─ network_errors
├─ dns_resolver
└─ tls_manager

webrtc_channels (existing)
├─ network_types
├─ network_errors
└─ webrtc_peer (Level 2, allowed - sibling dependency)
```

#### New Components (3)

```
ftp_protocol (new)
├─ network_types
├─ network_errors
├─ dns_resolver
└─ tls_manager

wpt_harness (new - testing)
├─ network_types
├─ network_errors
├─ http1_protocol
├─ http2_protocol
├─ http3_protocol
├─ websocket_protocol
├─ webrtc_peer
├─ webrtc_channels
├─ ftp_protocol
├─ cors_validator
├─ content_encoding
├─ mixed_content_blocker
└─ csp_processor

performance_benchmarks (new - testing)
├─ network_types
├─ network_errors
├─ http1_protocol
├─ http2_protocol
├─ http3_protocol
├─ websocket_protocol
├─ dns_resolver
├─ http_cache
└─ request_scheduler
```

---

### Level 3: Integration Layer (1 component) - Depends on All Levels

```
network_stack (existing, enhanced)
├─ Level 0:
│  ├─ network_types
│  └─ network_errors
├─ Level 1:
│  ├─ dns_resolver
│  ├─ tls_manager
│  ├─ cookie_manager
│  ├─ http_cache
│  ├─ proxy_support (new)
│  ├─ cors_validator (new)
│  ├─ content_encoding (new)
│  ├─ request_scheduler (new)
│  ├─ bandwidth_limiter (new)
│  ├─ url_handlers (new)
│  ├─ certificate_transparency (new)
│  ├─ mixed_content_blocker (new)
│  ├─ csp_processor (new)
│  ├─ certificate_pinning (new)
│  ├─ network_metrics (new)
│  └─ platform_integration (new)
└─ Level 2:
   ├─ http1_protocol
   ├─ http2_protocol
   ├─ http3_protocol
   ├─ websocket_protocol
   ├─ webrtc_peer
   ├─ webrtc_channels
   └─ ftp_protocol (new)
```

**Total Dependencies**: 25 components (all except testing components)
**Purpose**: Orchestrates all network functionality
**Import Count**: High (25 imports) - This is CORRECT for integration layer

---

## Build Order (Topological Sort)

### Build Order Rules
1. Components at same level with no inter-dependencies can build in parallel
2. Components at higher levels must wait for dependencies to complete
3. Maximum parallel agents: 3 (configured limit)

### Optimal Build Sequence

#### Batch 1: Level 0 (Base) - Already Complete
```
✅ network_types (complete)
✅ network_errors (complete)
```

#### Batch 2: Level 1 (Core) - Phase 1 Complete, Phase 2 New Components
```
✅ dns_resolver (complete)
✅ tls_manager (complete)
✅ cookie_manager (complete)
✅ http_cache (complete)

Phase 2 Batch 2a (3 parallel):
1. cors_validator
2. content_encoding
3. request_scheduler

Phase 2 Batch 2b (3 parallel):
4. bandwidth_limiter
5. url_handlers
6. mixed_content_blocker

Phase 2 Batch 2c (3 parallel):
7. csp_processor
8. network_metrics
9. proxy_support (depends on tls_manager, dns_resolver)

Phase 2 Batch 2d (3 parallel):
10. certificate_transparency (depends on tls_manager)
11. certificate_pinning (depends on tls_manager)
12. platform_integration (depends on tls_manager)
```

#### Batch 3: Level 2 (Protocol) - Phase 1 Complete + 1 New
```
✅ http1_protocol (complete)
✅ http2_protocol (complete)
✅ http3_protocol (complete)
✅ websocket_protocol (complete)
✅ webrtc_peer (complete)
✅ webrtc_channels (complete)

Phase 2 Batch 3:
1. ftp_protocol (depends on dns_resolver, tls_manager)
```

#### Batch 4: Level 3 (Integration) - Enhancement
```
Phase 2 Batch 4:
1. network_stack (enhance - depends on ALL Level 1 and Level 2 components)
```

#### Batch 5: Testing Components (After Level 3 Complete)
```
Phase 2 Batch 5 (2 parallel):
1. wpt_harness (depends on all protocol + security components)
2. performance_benchmarks (depends on protocols + infrastructure)
```

---

## Dependency Validation Rules

### Rule 1: Level Restrictions
- ✅ Level 0 can depend on: Nothing
- ✅ Level 1 can depend on: Level 0, other Level 1 (no cycles)
- ✅ Level 2 can depend on: Level 0-1, other Level 2 (no cycles)
- ✅ Level 3 can depend on: Level 0-2, other Level 3 (no cycles)

### Rule 2: Circular Dependency Detection
**Must Check**: No component A depends on B while B depends on A (directly or transitively)

**Current Status**: ✅ No circular dependencies detected

**Validation**:
```
proxy_support → tls_manager (Level 1)
tls_manager → network_types (Level 0)
✅ No cycle

certificate_transparency → tls_manager (Level 1)
tls_manager → network_types (Level 0)
✅ No cycle

platform_integration → tls_manager (Level 1)
tls_manager → network_types (Level 0)
✅ No cycle
```

### Rule 3: Maximum Imports per Level
- Level 0: 0 imports (no dependencies)
- Level 1: 2-6 imports typical (depends on base + optionally other core)
- Level 2: 4-10 imports typical (protocols need core services)
- Level 3: 20-30 imports expected (integration orchestrates everything)

**Current Status**:
- Level 0: 0 imports ✅
- Level 1: 2-4 imports ✅
- Level 2: 4-8 imports ✅
- Level 3: 25 imports ✅ (correct for integration layer)

---

## Component Coupling Analysis

### Highly Coupled Components (Many Dependents)

**network_types** (Level 0):
- Used by: ALL 28 components
- Coupling: Universal (expected for base types)
- Status: ✅ Appropriate

**network_errors** (Level 0):
- Used by: ALL 27 components (except network_metrics)
- Coupling: Nearly universal (expected for error handling)
- Status: ✅ Appropriate

**tls_manager** (Level 1):
- Used by: 11 components (all protocols + security components)
- Coupling: High (expected for TLS-dependent features)
- Status: ✅ Appropriate

**dns_resolver** (Level 1):
- Used by: 8 components (all protocols + proxy_support)
- Coupling: Medium-high (expected for network protocols)
- Status: ✅ Appropriate

### Loosely Coupled Components (Few Dependents)

**network_metrics** (Level 1):
- Used by: network_stack only
- Coupling: Low (metrics collection is specialized)
- Status: ✅ Appropriate

**certificate_transparency** (Level 1):
- Used by: network_stack only
- Coupling: Low (security feature)
- Status: ✅ Appropriate

**csp_processor** (Level 1):
- Used by: network_stack, wpt_harness
- Coupling: Low (security feature)
- Status: ✅ Appropriate

---

## Cross-Component Communication

### Allowed: Public API Imports

```rust
// ✅ ALLOWED - Import public APIs
use network_types::{NetworkRequest, NetworkResponse, NetworkError};
use dns_resolver::DnsResolver;
use tls_manager::TlsConfig;
```

### Forbidden: Private Implementation Access

```rust
// ❌ FORBIDDEN - Access private internals
use network_types::_internal::secrets;
use dns_resolver::_cache::Cache;
```

### Allowed: Composition

```rust
// ✅ ALLOWED - Use components as libraries
pub struct NetworkStack {
    dns: Arc<DnsResolver>,
    tls: Arc<TlsManager>,
    http1: Arc<Http1Client>,
    http2: Arc<Http2Client>,
}
```

### Forbidden: Direct File Modification

```rust
// ❌ FORBIDDEN - Modify other component's files
std::fs::write("../dns_resolver/config.json", data); // VIOLATION
```

---

## Integration Dependency Flow

### Request Flow Through Components

```
NetworkRequest
    ↓
[network_stack] (Level 3)
    ↓
┌─────────────────────────────────────┐
│ Pre-Processing                      │
├─────────────────────────────────────┤
│ url_handlers (data:/file:/ftp:)    │ Level 1
│ mixed_content_blocker (security)   │ Level 1
│ cors_validator (CORS check)        │ Level 1
│ csp_processor (CSP check)          │ Level 1
│ request_scheduler (prioritize)     │ Level 1
└─────────────────────────────────────┘
    ↓
┌─────────────────────────────────────┐
│ Infrastructure                      │
├─────────────────────────────────────┤
│ http_cache (check cache)           │ Level 1
│ proxy_support (proxy routing)      │ Level 1
│ dns_resolver (DNS lookup)          │ Level 1
│ tls_manager (TLS setup)            │ Level 1
└─────────────────────────────────────┘
    ↓
┌─────────────────────────────────────┐
│ Protocol Selection                  │
├─────────────────────────────────────┤
│ http1_protocol OR                  │ Level 2
│ http2_protocol OR                  │ Level 2
│ http3_protocol OR                  │ Level 2
│ ftp_protocol OR                    │ Level 2
│ websocket_protocol OR              │ Level 2
│ webrtc_peer + webrtc_channels      │ Level 2
└─────────────────────────────────────┘
    ↓
┌─────────────────────────────────────┐
│ Post-Processing                     │
├─────────────────────────────────────┤
│ bandwidth_limiter (throttle)       │ Level 1
│ content_encoding (decode)          │ Level 1
│ certificate_transparency (verify)  │ Level 1
│ certificate_pinning (check)        │ Level 1
│ http_cache (store)                 │ Level 1
│ network_metrics (record)           │ Level 1
└─────────────────────────────────────┘
    ↓
NetworkResponse
```

---

## Workspace Configuration Update

### New Cargo.toml (Phase 2 Complete)

```toml
[workspace]
members = [
    # Level 0: Base
    "components/network_types",
    "components/network_errors",

    # Level 1: Core (existing)
    "components/dns_resolver",
    "components/tls_manager",
    "components/cookie_manager",
    "components/http_cache",

    # Level 1: Core (new)
    "components/proxy_support",
    "components/cors_validator",
    "components/content_encoding",
    "components/request_scheduler",
    "components/bandwidth_limiter",
    "components/url_handlers",
    "components/certificate_transparency",
    "components/mixed_content_blocker",
    "components/csp_processor",
    "components/certificate_pinning",
    "components/network_metrics",
    "components/platform_integration",

    # Level 2: Protocol (existing)
    "components/http1_protocol",
    "components/http2_protocol",
    "components/http3_protocol",
    "components/websocket_protocol",
    "components/webrtc_peer",
    "components/webrtc_channels",

    # Level 2: Protocol (new)
    "components/ftp_protocol",

    # Level 2: Testing (new)
    "components/wpt_harness",
    "components/performance_benchmarks",

    # Level 3: Integration
    "components/network_stack",
]
```

**Total Members**: 28 components

---

## Dependency Metrics

### By Level

| Level | Components | Avg Dependencies | Max Dependencies | Min Dependencies |
|-------|------------|------------------|------------------|------------------|
| 0 | 2 | 0 | 0 | 0 |
| 1 | 16 | 2.8 | 4 | 1 |
| 2 | 9 | 6.2 | 15 | 3 |
| 3 | 1 | 25 | 25 | 25 |

### Overall

- **Total Components**: 28
- **Total Dependency Relationships**: ~160 (estimated)
- **Average Dependencies per Component**: ~5.7
- **Maximum Dependencies**: 25 (network_stack - integration layer)
- **Minimum Dependencies**: 0 (network_types, network_errors - base layer)

---

## Validation Checklist

### Dependency Rules
- [x] No circular dependencies
- [x] All dependencies respect level hierarchy
- [x] No higher-level dependencies in lower-level components
- [x] Integration layer (Level 3) correctly depends on all others

### Build Order
- [x] Level 0 can build first (no dependencies)
- [x] Level 1 can build after Level 0
- [x] Level 2 can build after Levels 0-1
- [x] Level 3 can build after Levels 0-2
- [x] Testing components build last

### Component Isolation
- [x] Each component has single responsibility
- [x] No component directly modifies another component's files
- [x] All communication through public APIs only
- [x] No _internal/ or _private/ imports across components

---

## Next Steps

✅ Gap analysis complete
✅ Architecture plan complete
✅ Token budget analysis complete
✅ Dependency graph complete

➡️ **Next**: Phase 2 - Create 15 new component directories with CLAUDE.md files

