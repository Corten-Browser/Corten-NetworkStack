# tls_manager

**Type**: Core
**Tech Stack**: Rust, Tokio, Cargo
**Version**: 0.1.0

## Overview

TLS 1.2/1.3 configuration, certificate validation, ALPN negotiation, session resumption, and HSTS enforcement.

This component provides the core TLS management functionality for the Corten-NetworkStack, including:

- **TLS Configuration**: Builder pattern for configuring TLS settings
- **Certificate Management**: Certificate storage and validation
- **ALPN Support**: Application-Layer Protocol Negotiation for HTTP/2 and HTTP/3
- **HSTS Enforcement**: HTTP Strict Transport Security policy management

## Structure

```
tls_manager/
├── src/
│   ├── lib.rs       # Main library entry point
│   └── ...          # Implementation modules
├── tests/
│   ├── unit/        # Unit tests
│   └── integration/ # Integration tests
├── Cargo.toml       # Rust package manifest
├── CLAUDE.md        # Development instructions for Claude Code
└── README.md        # This file
```

## Development

This component is part of the Corten-NetworkStack multi-component architecture.

See `CLAUDE.md` for detailed development instructions.

## API

### TlsConfig

Builder for TLS configuration:

```rust
use tls_manager::TlsConfig;

let config = TlsConfig::new()
    .with_alpn_protocols(vec![
        b"h3".to_vec(),      // HTTP/3
        b"h2".to_vec(),      // HTTP/2
        b"http/1.1".to_vec() // HTTP/1.1
    ]);
```

### CertificateStore

Manages and validates certificates:

```rust
use tls_manager::CertificateStore;

let mut store = CertificateStore::new();
store.add_certificate(cert_data)?;

// Async certificate verification
store.verify_certificate(&cert, "example.com").await?;
```

### HstsStore

Enforces HSTS policies:

```rust
use std::time::Duration;
use tls_manager::HstsStore;

let mut store = HstsStore::new();
store.add_hsts_entry(
    "example.com".to_string(),
    Duration::from_secs(31536000), // 1 year
    true  // include subdomains
);

assert!(store.is_hsts_enabled("example.com"));
assert!(store.is_hsts_enabled("sub.example.com"));
```

## Dependencies

- `network-types`: Core network types
- `network-errors`: Error handling
- `rustls`: TLS implementation
- `tokio-rustls`: Async TLS
- `webpki-roots`: Root certificates
- `tokio`: Async runtime

## Testing

```bash
cargo test                    # Run all tests
cargo test --test unit        # Unit tests only
cargo test --test integration # Integration tests only
cargo clippy                  # Linting
cargo fmt                     # Formatting
```

## Documentation

```bash
cargo doc --open  # Generate and open documentation
```
