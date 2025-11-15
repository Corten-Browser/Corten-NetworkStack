# platform_integration

Platform-specific integrations for the Corten Network Stack.

## Overview

This component provides platform-specific functionality for:
- **System proxy configuration detection** from environment variables
- **System certificate store access** (basic implementation)
- **Network connectivity detection**

## Features

### System Proxy Configuration

Reads proxy settings from standard environment variables:
- `HTTP_PROXY`: HTTP proxy server URL
- `HTTPS_PROXY`: HTTPS proxy server URL
- `NO_PROXY`: Comma-separated list of domains to bypass proxy

### Certificate Store

Provides access to system certificate store (basic implementation). Returns empty vector with graceful degradation on unsupported platforms.

### Network Connectivity

Simple heuristic check for network online/offline status by attempting connection to public DNS server.

## Usage

```rust
use platform_integration::{PlatformIntegration, SystemProxyConfig};

// Get system proxy configuration
let proxy_config = PlatformIntegration::get_system_proxy_config()?;
if proxy_config.enabled {
    if let Some(http_proxy) = proxy_config.http_proxy {
        println!("HTTP proxy: {}", http_proxy);
    }
    if let Some(https_proxy) = proxy_config.https_proxy {
        println!("HTTPS proxy: {}", https_proxy);
    }
    for domain in &proxy_config.no_proxy {
        println!("No proxy for: {}", domain);
    }
}

// Check network connectivity
if PlatformIntegration::is_online() {
    println!("Network is online");
} else {
    println!("Network is offline");
}

// Get system certificates (basic implementation)
let certs = PlatformIntegration::get_system_cert_store()?;
println!("Found {} system certificates", certs.len());
```

## Platform Support

| Feature | Linux | Windows | macOS |
|---------|-------|---------|-------|
| Proxy Config (env vars) | ✅ Full | ✅ Full | ✅ Full |
| Certificate Store | ⚠️ Basic | ⚠️ Basic | ⚠️ Basic |
| Network Detection | ✅ Full | ✅ Full | ✅ Full |

**Note**: Certificate store currently returns empty vector. Future enhancements will add platform-specific certificate loading.

## Dependencies

- `network-errors`: Error types for network operations

## Testing

```bash
# Run all tests
cargo test --package platform_integration

# Run with output
cargo test --package platform_integration -- --nocapture

# Run specific test
cargo test --package platform_integration test_get_system_proxy_config
```

## Test Coverage

- **35 tests** covering all public APIs
- **Unit tests**: Proxy parsing, certificate access, network detection
- **Integration tests**: Public API availability
- **Doc tests**: Example code verification

## Architecture

```
platform_integration/
├── src/
│   ├── lib.rs          # Public API
│   ├── proxy.rs        # Proxy configuration
│   ├── certs.rs        # Certificate store
│   └── network.rs      # Network detection
├── tests/
│   ├── unit/           # Unit tests
│   │   ├── test_proxy.rs
│   │   ├── test_certs.rs
│   │   └── test_network.rs
│   └── integration/    # Integration tests
└── Cargo.toml
```

## Future Enhancements

- Platform-specific certificate store loading
  - Linux: `/etc/ssl/certs`, `/etc/pki/tls/certs`
  - Windows: Windows Certificate Store API
  - macOS: Security.framework
- More sophisticated network detection
- Proxy auto-configuration (PAC) support
- System DNS configuration detection

## License

MIT OR Apache-2.0
