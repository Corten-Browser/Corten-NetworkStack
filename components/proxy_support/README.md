# proxy_support

HTTP and SOCKS5 proxy client implementation for establishing TCP connections through proxy servers.

## Overview

This component provides proxy support for network connections, allowing TCP connections to be established through:
- HTTP CONNECT proxies
- SOCKS5 proxies
- Direct connections (no proxy)

Both proxy types support optional Basic authentication.

## Features

- **HTTP CONNECT Proxy**: Tunnel TCP connections through HTTP proxies using the CONNECT method
- **SOCKS5 Proxy**: Full SOCKS5 protocol implementation with authentication support
- **Basic Authentication**: Username/password authentication for both proxy types
- **Direct Connections**: Fallback to direct connections when no proxy is configured
- **Async/Await**: Fully asynchronous using Tokio
- **Error Handling**: Comprehensive error types for different failure modes

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
proxy_support = { path = "../proxy_support" }
```

## Usage

### Direct Connection (No Proxy)

```rust
use proxy_support::{ProxyClient, ProxyConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = ProxyConfig::None;
    let client = ProxyClient::new(config);

    let stream = client.connect("example.com", 80).await?;
    // Use stream...

    Ok(())
}
```

### HTTP CONNECT Proxy

```rust
use proxy_support::{ProxyClient, ProxyConfig, ProxyAuth};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Without authentication
    let config = ProxyConfig::Http {
        host: "proxy.example.com".to_string(),
        port: 8080,
        auth: None,
    };

    let client = ProxyClient::new(config);
    let stream = client.connect("target.example.com", 443).await?;

    Ok(())
}
```

### HTTP Proxy with Authentication

```rust
use proxy_support::{ProxyClient, ProxyConfig, ProxyAuth};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = ProxyConfig::Http {
        host: "proxy.example.com".to_string(),
        port: 8080,
        auth: Some(ProxyAuth::Basic {
            username: "user".to_string(),
            password: "password".to_string(),
        }),
    };

    let client = ProxyClient::new(config);
    let stream = client.connect("target.example.com", 443).await?;

    Ok(())
}
```

### SOCKS5 Proxy

```rust
use proxy_support::{ProxyClient, ProxyConfig, ProxyAuth};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = ProxyConfig::Socks5 {
        host: "socks.example.com".to_string(),
        port: 1080,
        auth: Some(ProxyAuth::Basic {
            username: "socks_user".to_string(),
            password: "socks_pass".to_string(),
        }),
    };

    let client = ProxyClient::new(config);
    let stream = client.connect("target.example.com", 443).await?;

    Ok(())
}
```

## API

### `ProxyConfig`

Enum defining proxy configuration:

- `ProxyConfig::None` - No proxy (direct connection)
- `ProxyConfig::Http { host, port, auth }` - HTTP CONNECT proxy
- `ProxyConfig::Socks5 { host, port, auth }` - SOCKS5 proxy

### `ProxyAuth`

Enum for authentication credentials:

- `ProxyAuth::Basic { username, password }` - HTTP Basic authentication

### `ProxyClient`

Main client struct for establishing connections:

- `ProxyClient::new(config)` - Create a new proxy client
- `ProxyClient::connect(host, port)` - Connect to target through proxy

## Development

### Running Tests

```bash
# Run all tests
cargo test

# Run unit tests only
cargo test --lib

# Run integration tests only
cargo test --test integration

# Run with output
cargo test -- --nocapture
```

### Test Coverage

The component includes:
- **Unit tests**: Test individual components (auth, config, protocol formatting)
- **Integration tests**: Test complete connection flows
- **Doc tests**: Ensure examples in documentation work

### Code Quality

```bash
# Check formatting
cargo fmt --check

# Run linter
cargo clippy -- -D warnings

# Build documentation
cargo doc --no-deps
```

## Architecture

### Components

- `lib.rs` - Public API and `ProxyClient` implementation
- `auth.rs` - Authentication credential handling
- `http_proxy.rs` - HTTP CONNECT protocol implementation
- `socks5.rs` - SOCKS5 protocol implementation

### HTTP CONNECT Flow

1. Connect to proxy server
2. Send CONNECT request with target host/port
3. Add Proxy-Authorization header if auth provided
4. Read response status
5. Return tunneled stream if successful

### SOCKS5 Flow

1. Connect to proxy server
2. Send greeting with supported auth methods
3. Perform authentication if required
4. Send CONNECT request for target
5. Read response
6. Return connected stream if successful

## Error Handling

All errors are returned as `NetworkError::ProxyError` with detailed messages:

- Proxy connection failures
- Authentication failures
- Target connection failures
- Protocol errors
- Timeout errors

## Dependencies

- `tokio` - Async runtime
- `network-errors` - Error types
- `base64` - Authentication encoding

## Testing Strategy

### Unit Tests

- Test configuration variants
- Test authentication encoding
- Test protocol message formatting
- Test error handling

### Integration Tests

- Test direct connections
- Test configuration setup
- (Full proxy tests require actual proxy servers)

### Future Enhancements

- Connection pooling
- Proxy rotation
- SOCKS4 support
- Digest authentication
- Automatic proxy detection

## License

See project root LICENSE file.
