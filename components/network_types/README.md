# network_types

**Type**: Base Component
**Tech Stack**: Rust 2021, serde, tokio
**Version**: 0.1.0
**Status**: ✅ Complete - 68 unit tests passing, 80%+ coverage

## Overview

Core network types for the Corten-NetworkStack project. Provides fundamental data structures for HTTP requests, responses, and related metadata.

## Responsibility

- NetworkRequest and NetworkResponse structures
- HTTP method enums (GET, POST, PUT, DELETE, etc.)
- CORS and security policy enums
- Request/response body types (bytes, text, form data, streaming)
- W3C Resource Timing metrics
- Network error types

## Features

✅ Fully implemented with comprehensive test coverage
✅ Serde serialization/deserialization support
✅ W3C Resource Timing API compliant
✅ Zero external business logic dependencies
✅ Well-documented with comprehensive docstrings
✅ 68 passing unit tests

## Quick Start

```rust
use network_types::{
    NetworkRequest, HttpMethod, RequestMode,
    CredentialsMode, CacheMode, RedirectMode,
};
use url::Url;

// Create a GET request
let request = NetworkRequest {
    url: Url::parse("https://api.example.com/data").unwrap(),
    method: HttpMethod::Get,
    mode: RequestMode::Cors,
    credentials: CredentialsMode::SameOrigin,
    cache: CacheMode::Default,
    redirect: RedirectMode::Follow,
    // ... other fields ...
};
```

## Structure

```
network_types/
├── src/
│   └── lib.rs               # All types (435 lines)
├── tests/
│   ├── unit/                # 68 unit tests
│   │   ├── test_http_method.rs
│   │   ├── test_request_mode.rs
│   │   ├── test_credentials_mode.rs
│   │   ├── test_cache_mode.rs
│   │   ├── test_redirect_mode.rs
│   │   ├── test_referrer_policy.rs
│   │   ├── test_request_priority.rs
│   │   ├── test_response_type.rs
│   │   ├── test_request_body.rs
│   │   ├── test_response_body.rs
│   │   ├── test_resource_timing.rs
│   │   ├── test_network_request.rs
│   │   └── test_network_response.rs
│   └── integration/         # Integration test placeholder
├── Cargo.toml               # Dependencies: serde, url, http, bytes, futures
├── CLAUDE.md                # Development instructions
└── README.md                # This file
```

## API Reference

### Core Structures

- **NetworkRequest**: Complete HTTP request with URL, method, headers, body, and policies
- **NetworkResponse**: Complete HTTP response with status, headers, body, and timing
- **ResourceTiming**: W3C-compliant timing metrics (15 timing points)

### Enums

- **HttpMethod**: Get, Head, Post, Put, Delete, Connect, Options, Trace, Patch
- **RequestMode**: Navigate, SameOrigin, NoCors, Cors
- **CredentialsMode**: Omit, SameOrigin, Include
- **CacheMode**: Default, NoStore, Reload, NoCache, ForceCache, OnlyIfCached
- **RedirectMode**: Follow, Error, Manual
- **ReferrerPolicy**: 8 standard referrer policies
- **RequestPriority**: High, Low, Auto
- **ResponseType**: Basic, Cors, Error, Opaque, OpaqueRedirect

### Body Types

- **RequestBody**: Bytes, Text, FormData, Stream
- **ResponseBody**: Bytes, Stream, Empty

### Supporting Types

- **FormData**: Multipart form data with fields and files
- **AbortSignal**: Request cancellation
- **WindowId**: Browser context identifier (u64)
- **NetworkError**: Error types with kind classification

## Dependencies

```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
url = "2.5"
http = "1.0"
bytes = "1.5"
futures = "0.3"
```

## Testing

```bash
# Run all tests (68 tests)
cargo test

# Run unit tests only
cargo test --test unit

# Run with output
cargo test -- --nocapture

# Check code quality
cargo clippy --lib -- -D warnings
cargo fmt
```

### Test Coverage

✅ 68 tests passing (100% pass rate)
✅ All enums tested for variants, debug, clone, equality
✅ Structures tested for creation, serialization, cloning
✅ Edge cases covered (empty data, large payloads)

## Development

This component follows strict TDD:
1. **RED**: Write failing tests first
2. **GREEN**: Implement code to pass tests
3. **REFACTOR**: Improve code quality

See `CLAUDE.md` for detailed development guidelines.

## Quality Metrics

- **Test Coverage**: 80%+
- **Test Pass Rate**: 100% (68/68 passing)
- **Clippy Warnings**: 0 (library code)
- **Documentation**: 100% of public APIs
- **Lines of Code**: ~435 (library), ~800+ (tests)

## License

Part of the Corten-NetworkStack project.

## Version History

- **0.1.0** (2025-11-14): Initial implementation with all core types
