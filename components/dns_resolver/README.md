# dns_resolver

**Type**: Core Library
**Tech Stack**: Rust 2021, Tokio async runtime, hickory-resolver
**Version**: 0.1.0

## Overview

DNS resolution component with DNS-over-HTTPS (DoH) support, caching, and async resolution. Provides a clean async API for hostname to IP address resolution with built-in caching and timeout handling.

## Features

- ✅ **Async DNS resolution** - Tokio-based async resolution
- ✅ **DNS-over-HTTPS** - Secure DNS using HTTPS (DoH)
- ✅ **Caching with TTL** - Automatic result caching with time-to-live
- ✅ **Timeout handling** - Configurable operation timeouts
- ✅ **Fallback support** - Automatic fallback from DoH to standard DNS
- ✅ **Multiple DoH providers** - Google DNS, Cloudflare DNS

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
dns-resolver = { path = "../dns_resolver" }
network-errors = { path = "../network_errors" }
tokio = { version = "1.35", features = ["full"] }
```

## Usage

### Basic DNS Resolution

```rust
use dns_resolver::{DnsResolver, StandardResolver};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a standard DNS resolver
    let resolver = StandardResolver::new(None)?;

    // Resolve a hostname
    let addresses = resolver.resolve("example.com".to_string()).await?;

    for addr in addresses {
        println!("Resolved: {}", addr);
    }

    Ok(())
}
```

### DNS with Timeout

```rust
use dns_resolver::{DnsResolver, StandardResolver};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let resolver = StandardResolver::new(None)?;

    // Resolve with 5-second timeout
    let addresses = resolver
        .resolve_with_timeout("example.com".to_string(), Duration::from_secs(5))
        .await?;

    Ok(())
}
```

### DNS-over-HTTPS (DoH)

```rust
use dns_resolver::{StandardResolver, DohConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Option 1: Google DNS-over-HTTPS
    let resolver = StandardResolver::with_google_doh()?;

    // Option 2: Cloudflare DNS-over-HTTPS
    let resolver = StandardResolver::with_cloudflare_doh()?;

    // Option 3: Custom DoH configuration
    let config = DohConfig {
        enabled: true,
        resolver_url: "https://dns.google/dns-query".to_string(),
        use_fallback: true,
    };
    let resolver = StandardResolver::new(Some(config))?;

    let addresses = resolver.resolve("example.com".to_string()).await?;

    Ok(())
}
```

### Manual Cache Management

```rust
use dns_resolver::DnsCache;
use std::net::IpAddr;
use std::time::Duration;

fn main() {
    let mut cache = DnsCache::new();

    // Insert addresses with 5-minute TTL
    let addresses = vec!["192.0.2.1".parse::<IpAddr>().unwrap()];
    cache.insert(
        "example.com".to_string(),
        addresses,
        Duration::from_secs(300)
    );

    // Retrieve from cache
    if let Some(addrs) = cache.get("example.com") {
        println!("Cached addresses: {:?}", addrs);
    }

    // Clear expired entries
    cache.clear_expired();
}
```

## API

### DnsResolver Trait

```rust
#[async_trait]
pub trait DnsResolver: Send + Sync {
    async fn resolve(&self, hostname: String) -> NetworkResult<Vec<IpAddr>>;

    async fn resolve_with_timeout(
        &self,
        hostname: String,
        timeout: Duration,
    ) -> NetworkResult<Vec<IpAddr>>;
}
```

### DnsCache

```rust
pub struct DnsCache;

impl DnsCache {
    pub fn new() -> Self;
    pub fn get(&self, hostname: &str) -> Option<Vec<IpAddr>>;
    pub fn insert(&mut self, hostname: String, addresses: Vec<IpAddr>, ttl: Duration);
    pub fn clear_expired(&mut self);
}
```

### DohConfig

```rust
pub struct DohConfig {
    pub enabled: bool,
    pub resolver_url: String,
    pub use_fallback: bool,
}

impl DohConfig {
    pub fn new(enabled: bool, resolver_url: String, use_fallback: bool) -> Self;
    pub fn google() -> Self;
    pub fn cloudflare() -> Self;
}
```

### StandardResolver

```rust
pub struct StandardResolver;

impl StandardResolver {
    pub fn new(doh_config: Option<DohConfig>) -> NetworkResult<Self>;
    pub fn with_google_doh() -> NetworkResult<Self>;
    pub fn with_cloudflare_doh() -> NetworkResult<Self>;
    pub async fn clear_cache(&self);
    pub async fn cache_size(&self) -> usize;
}
```

## Error Handling

All resolution methods return `NetworkResult<T>` which is `Result<T, NetworkError>`.

Common error variants:
- `NetworkError::DnsError(String)` - DNS resolution failed
- `NetworkError::Timeout(Duration)` - Operation timed out

```rust
use network_errors::NetworkError;

match resolver.resolve("example.com".to_string()).await {
    Ok(addresses) => println!("Success: {:?}", addresses),
    Err(NetworkError::DnsError(msg)) => println!("DNS error: {}", msg),
    Err(NetworkError::Timeout(d)) => println!("Timeout after {:?}", d),
    Err(e) => println!("Other error: {}", e),
}
```

## Testing

```bash
# Run all tests (21 tests)
cargo test

# Run specific test suites
cargo test --lib          # Library tests (4 tests)
cargo test --test unit    # Unit tests (6 tests)
cargo test --test integration  # Integration tests (10 tests)

# Run with output
cargo test -- --nocapture

# Check test coverage
cargo test --all-features
```

### Test Coverage

- **Unit tests**: Core functionality (cache, config)
- **Integration tests**: Real DNS resolution, contract validation
- **Contract tests**: API contract compliance
- **Coverage**: High coverage of all public APIs

## Development

See `CLAUDE.md` for detailed development instructions including:
- TDD workflow (Red-Green-Refactor)
- Code quality standards
- Contribution guidelines

### Building

```bash
# Build the component
cargo build

# Build with optimizations
cargo build --release

# Check for errors without building
cargo check
```

### Code Quality

```bash
# Format code
cargo fmt

# Lint code
cargo clippy -- -D warnings

# Generate documentation
cargo doc --open
```

## Dependencies

- `network-errors` - Error types
- `hickory-resolver` - DNS resolution library
- `tokio` - Async runtime
- `async-trait` - Async trait support

## Architecture

This component follows the Corten-NetworkStack architecture:
- **Level**: Core library (Level 1)
- **Type**: Stateless service
- **Thread-safe**: Yes (Arc/Mutex for cache)
- **Async**: Fully async using Tokio

## Performance

- **Caching**: Reduces repeated DNS lookups
- **TTL**: Automatic cache expiration
- **Timeout**: Prevents hanging operations
- **Async**: Non-blocking I/O

## License

MIT OR Apache-2.0
