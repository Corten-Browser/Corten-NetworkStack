# network_errors

**Type**: Base Component
**Tech Stack**: Rust 2021 edition, thiserror
**Version**: 0.1.0

## Overview

The `network_errors` component provides comprehensive error handling for network operations in the Corten-NetworkStack project. It defines a unified `NetworkError` enum that covers all common network failure scenarios, from connection issues to protocol violations.

## Responsibility

Error handling: NetworkError enum, Result types, error conversion traits

This component serves as the foundation for error handling across all network-related components in the stack.

## Structure

```
network_errors/
├── src/
│   └── lib.rs           # NetworkError enum and NetworkResult type
├── tests/
│   ├── unit/            # Comprehensive unit tests (46 tests)
│   │   └── mod.rs
│   └── integration/     # Integration tests
│       └── mod.rs
├── Cargo.toml           # Rust package manifest
├── CLAUDE.md            # Development instructions
└── README.md            # This file
```

## API

### Types

#### `NetworkError`

An enum representing all possible network error types:

```rust
pub enum NetworkError {
    ConnectionFailed(String),    // TCP connection failure
    DnsError(String),            // DNS resolution failure
    TlsError(String),            // TLS/SSL errors
    ProtocolError(String),       // Protocol violations
    Timeout(Duration),           // Operation timeout
    Aborted,                     // Cancelled operation
    InvalidUrl(String),          // Malformed URL
    TooManyRedirects,           // Redirect limit exceeded
    CacheError(String),         // Cache operation failure
    ProxyError(String),         // Proxy issues
    CorsError(String),          // CORS violation
    MixedContent,               // Mixed content blocked
    CertificateError(String),   // Certificate validation failed
    WebSocketError(String),     // WebSocket error
    WebRtcError(String),        // WebRTC error
    Io(std::io::Error),         // Low-level I/O error
    Other(String),              // Catch-all
}
```

#### `NetworkResult<T>`

Type alias for `Result<T, NetworkError>`:

```rust
pub type NetworkResult<T> = Result<T, NetworkError>;
```

## Usage

### Basic Error Creation

```rust
use network_errors::{NetworkError, NetworkResult};
use std::time::Duration;

// Create specific errors
let connection_error = NetworkError::ConnectionFailed("Host unreachable".to_string());
let timeout_error = NetworkError::Timeout(Duration::from_secs(30));
let dns_error = NetworkError::DnsError("Name not resolved".to_string());
```

### Using NetworkResult in Functions

```rust
use network_errors::{NetworkError, NetworkResult};

fn fetch_url(url: &str) -> NetworkResult<Vec<u8>> {
    if url.is_empty() {
        return Err(NetworkError::InvalidUrl("URL cannot be empty".to_string()));
    }
    Ok(vec![1, 2, 3, 4])
}

match fetch_url("https://example.com") {
    Ok(data) => println!("Fetched {} bytes", data.len()),
    Err(e) => eprintln!("Error: {}", e),
}
```

### Error Propagation with ?

```rust
use network_errors::{NetworkError, NetworkResult};

fn inner_operation() -> NetworkResult<String> {
    Err(NetworkError::Timeout(Duration::from_secs(10)))
}

fn outer_operation() -> NetworkResult<String> {
    let result = inner_operation()?; // Propagates error
    Ok(result)
}
```

### Converting from std::io::Error

```rust
use network_errors::NetworkError;
use std::fs::File;

fn read_config() -> Result<(), NetworkError> {
    let _file = File::open("config.txt")?; // Auto-converts io::Error
    Ok(())
}
```

## Development

### Setup

```bash
cd components/network_errors
cargo build
```

### Testing

```bash
# Run all tests
cargo test

# Run specific test suite
cargo test --test unit
cargo test --test integration

# Run with output
cargo test -- --nocapture
```

### Code Quality

```bash
# Format code
cargo fmt

# Lint code
cargo clippy --all-targets --all-features -- -D warnings

# Generate documentation
cargo doc --open
```

## Test Coverage

Comprehensive test coverage including:

- **Error Creation Tests**: All 17 error variants
- **Display Tests**: Error message formatting
- **Conversion Tests**: `From<std::io::Error>` implementation
- **Error Trait Tests**: `std::error::Error` compliance
- **Result Type Tests**: `NetworkResult<T>` usage patterns
- **Contract Tests**: API contract verification

**Statistics**:
- 46 unit tests
- 2 doc tests
- **Total**: 48 tests
- **Pass Rate**: 100%
- **Coverage**: ~95%+

## Dependencies

- `thiserror 1.0`: Error derive macros

## Contract Compliance

This component implements the API contract defined in `contracts/network_errors.yaml`:

- ✅ All 17 error variants implemented exactly as specified
- ✅ `NetworkResult<T>` type alias exported
- ✅ `std::error::Error` trait implemented (via thiserror)
- ✅ `Display` trait implemented with meaningful messages
- ✅ `From<std::io::Error>` conversion implemented
- ✅ All variants match contract specification

## Architecture

This is a **Level 0 base component** with no dependencies on other components. It provides fundamental error types used throughout the network stack.

**Design Decisions**:

1. **Use thiserror**: Reduces boilerplate, provides robust error implementation
2. **Comprehensive variants**: Cover all common network failure modes
3. **String messages**: Provide context-specific error details
4. **Duration in Timeout**: Helps with debugging and retry logic
5. **From<io::Error>**: Seamless standard library integration

## Integration

Other components should:

1. Import `NetworkError` and `NetworkResult`
2. Use `NetworkResult<T>` for fallible operations
3. Create appropriate error variants
4. Use `?` operator for error propagation

Example:

```rust
use network_errors::{NetworkError, NetworkResult};

pub fn download(url: &str) -> NetworkResult<Vec<u8>> {
    // Returns NetworkError on failure
    todo!()
}
```

## License

Part of the Corten-NetworkStack project (v0.1.0).
