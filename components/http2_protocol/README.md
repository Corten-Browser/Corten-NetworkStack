# HTTP/2 Protocol Component

An async HTTP/2 client implementation with connection pooling, multiplexing, and integration with DNS resolution, TLS, cookie management, and HTTP caching.

## Overview

This component provides a high-level HTTP/2 client that:
- Manages HTTP/2 connections with multiplexing support
- Pools and reuses connections across requests
- Integrates with DNS resolver for hostname resolution
- Manages cookies automatically using the cookie_manager component
- Caches responses using the http_cache component
- Follows HTTP redirects automatically
- Provides health checking via PING frames
- Uses the h2 crate for HTTP/2 protocol implementation

## Features

- **Connection Pooling**: Reuses HTTP/2 connections per host:port:scheme
- **Multiplexing**: Multiple concurrent requests over single connection (h2 level)
- **Cookie Management**: Automatic cookie storage and retrieval
- **HTTP Caching**: Intelligent response caching with cache control
- **Redirect Following**: Automatic redirect handling (configurable max redirects)
- **DNS Integration**: Async DNS resolution via dns_resolver component
- **TLS Support**: Ready for TLS integration (when tls_manager provides async stream wrapper)
- **Health Checks**: Connection health monitoring via HTTP/2 PING
- **Configurable**: Customizable stream limits, window sizes, frame sizes

## Architecture

```
Http2Client
├── Connection Pool (HashMap<PoolKey, Arc<Http2Connection>>)
├── DNS Resolver (Arc<dyn DnsResolver>)
├── Cookie Store (Arc<RwLock<CookieStore>>)
├── HTTP Cache (Arc<HttpCache>)
└── Configuration (Http2Config)

Http2Connection
├── h2::SendRequest (for sending requests)
├── Configuration (Http2Config)
└── Created timestamp (for metrics)

Http2Config
├── max_concurrent_streams (default: 100)
├── initial_connection_window_size (default: 65535)
├── initial_stream_window_size (default: 65535)
└── max_frame_size (default: 16384)
```

## Usage

### Basic Usage

```rust
use http2_protocol::{Http2Client, Http2Config};
use network_types::{NetworkRequest, HttpMethod};
use url::Url;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create client with default configuration
    let client = Http2Client::new(Http2Config::default())?;

    // Create request
    let request = NetworkRequest {
        url: Url::parse("http://example.com")?,
        method: HttpMethod::Get,
        headers: http::HeaderMap::new(),
        body: None,
        // ... other fields
    };

    // Fetch response
    let response = client.fetch(request).await?;

    println!("Status: {}", response.status);
    println!("Headers: {:?}", response.headers);

    Ok(())
}
```

### Custom Configuration

```rust
use http2_protocol::{Http2Client, Http2Config};

// Create custom configuration
let config = Http2Config::new()
    .with_max_concurrent_streams(200)
    .with_initial_connection_window_size(131072)
    .with_initial_stream_window_size(65536)
    .with_max_frame_size(32768);

// Create client with custom config
let client = Http2Client::new(config)?
    .with_max_redirects(5)
    .with_timeout(std::time::Duration::from_secs(10));
```

### Multiple Requests

```rust
let requests = vec![request1, request2, request3];

// Fetch multiple requests (sequential with connection reuse)
let responses = client.fetch_multiple(requests).await?;

for response in responses {
    println!("Got response: {}", response.status);
}
```

### Connection Health Check

```rust
// Perform health check on connection
let rtt = client.health_check("http://example.com").await?;
println!("Round-trip time: {:?}", rtt);
```

### Connection Pool Management

```rust
// Get number of pooled connections
let count = client.connection_count().await;
println!("Pooled connections: {}", count);

// Clear connection pool
client.clear_connections().await;
```

## Configuration

### Http2Config

| Field | Default | Valid Range | Description |
|-------|---------|-------------|-------------|
| `max_concurrent_streams` | 100 | 1 - 2³¹-1 | Maximum concurrent streams per connection |
| `initial_connection_window_size` | 65535 | 1 - 2³¹-1 | Initial flow control window for connection |
| `initial_stream_window_size` | 65535 | 1 - 2³¹-1 | Initial flow control window for streams |
| `max_frame_size` | 16384 | 16384 - 16777215 | Maximum size of HTTP/2 frames |

### Http2Client Options

| Method | Default | Description |
|--------|---------|-------------|
| `with_max_redirects(usize)` | 10 | Maximum number of redirects to follow |
| `with_timeout(Duration)` | 30s | Request timeout duration |

## Dependencies

### Level 0 (Core Types)
- `network-types`: Core network types (NetworkRequest, NetworkResponse)
- `network-errors`: Error types (NetworkError)

### Level 1 (Support Services)
- `dns-resolver`: DNS resolution
- `tls-manager`: TLS connection handling (integration pending)
- `cookie-manager`: Cookie storage and management
- `http-cache`: HTTP response caching

### External Crates
- `h2`: HTTP/2 protocol implementation
- `hyper`: HTTP types and utilities
- `tokio`: Async runtime
- `url`: URL parsing
- `cookie`: Cookie parsing
- `bytes`: Byte buffer utilities

## Development

### Running Tests

```bash
# Run all tests
cargo test

# Run only unit tests
cargo test --lib

# Run with output
cargo test -- --nocapture
```

### Code Quality

```bash
# Format code
cargo fmt

# Run linter
cargo clippy -- -D warnings

# Check for issues
cargo check
```

## Testing

### Test Coverage

- ✅ Http2Config: Validation, builder pattern, defaults
- ✅ Http2Connection: Configuration, HTTP method conversion
- ✅ Http2Client: PoolKey parsing, client creation, connection pooling
- ⏳ Integration tests: Async functionality with mock HTTP/2 servers (TODO)

**Current Unit Test Results**: 13/13 passing

### Test Organization

```
tests/
├── unit/
│   ├── mod.rs          # Unit test suite
│   ├── config.rs       # Http2Config tests
│   ├── connection.rs   # Http2Connection tests
│   └── client.rs       # Http2Client tests
└── integration/
    └── mod.rs          # Integration test suite (TODO)
```

## Implementation Notes

### Current Limitations

1. **TLS Support**: TLS integration is pending the tls_manager component providing an async stream wrapper method. HTTP connections work, HTTPS returns an error noting TLS is not yet supported.

2. **Multiplexing**: The `fetch_multiple()` method currently executes requests sequentially but reuses connections. True concurrent multiplexing using h2's stream capabilities is planned.

3. **PING Implementation**: The `ping()` method is a placeholder. Proper HTTP/2 PING frame handling needs to be implemented using h2's ping API.

### Future Enhancements

- [ ] True concurrent multiplexing for `fetch_multiple()`
- [ ] Server push support
- [ ] Stream prioritization
- [ ] Proper HTTP/2 PING implementation
- [ ] TLS integration when tls_manager ready
- [ ] Request/response compression
- [ ] Custom header compression tables
- [ ] Connection graceful shutdown

## Error Handling

All errors are wrapped in `NetworkError` from the `network-errors` component:

```rust
match client.fetch(request).await {
    Ok(response) => { /* handle response */ },
    Err(NetworkError::DnsError(msg)) => { /* DNS failed */ },
    Err(NetworkError::ConnectionFailed(msg)) => { /* Connection failed */ },
    Err(NetworkError::TlsError(msg)) => { /* TLS error */ },
    Err(NetworkError::ProtocolError(msg)) => { /* HTTP/2 protocol error */ },
    Err(NetworkError::Timeout(duration)) => { /* Request timeout */ },
    Err(e) => { /* Other error */ },
}
```

## Performance Considerations

- **Connection Reuse**: Connections are pooled and reused across requests to the same host
- **Multiplexing**: h2 handles stream multiplexing internally for efficient use of connections
- **Async I/O**: All I/O operations are fully async using Tokio
- **Zero-Copy**: Uses `bytes::Bytes` for efficient buffer handling
- **Caching**: Successful responses are cached to avoid redundant requests

## License

See project root LICENSE file.

## Contributing

This component is part of the Corten-NetworkStack project. See the project README for contribution guidelines.
