# Quick Start Guide

This guide will help you get started with the Corten Network Stack quickly.

## Table of Contents

- [Installation](#installation)
- [Basic HTTP Request](#basic-http-request)
- [HTTPS with TLS Configuration](#https-with-tls-configuration)
- [WebSocket Connection](#websocket-connection)
- [Advanced Features](#advanced-features)
  - [CORS Validation](#cors-validation)
  - [HTTP Caching](#http-caching)
  - [Request Scheduling](#request-scheduling)
  - [Bandwidth Limiting](#bandwidth-limiting)
  - [DNS-over-HTTPS (DoH)](#dns-over-https-doh)

## Installation

Add this crate to your `Cargo.toml`:

```toml
[dependencies]
# Core network stack
network_stack = { path = "components/network_stack" }
network_types = { path = "components/network_types" }
network_errors = { path = "components/network_errors" }

# Optional: Protocol clients (choose what you need)
http1_protocol = { path = "components/http1_protocol" }
http2_protocol = { path = "components/http2_protocol" }
websocket_protocol = { path = "components/websocket_protocol" }

# Optional: Security components
tls_manager = { path = "components/tls_manager" }
cors_validator = { path = "components/cors_validator" }

# Optional: Services
dns_resolver = { path = "components/dns_resolver" }
http_cache = { path = "components/http_cache" }
cookie_manager = { path = "components/cookie_manager" }

# Async runtime
tokio = { version = "1.0", features = ["full"] }
```

Or include the entire workspace:

```toml
[dependencies]
corten-network-stack = { path = "." }
tokio = { version = "1.0", features = ["full"] }
```

## Basic HTTP Request

Perform a simple HTTP GET request:

```rust
use network_stack::NetworkStack;
use network_types::{NetworkRequest, HttpMethod};
use url::Url;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a basic network stack implementation
    let stack = create_network_stack();

    // Build a request
    let request = NetworkRequest {
        url: Url::parse("http://httpbin.org/get")?,
        method: HttpMethod::Get,
        headers: http::HeaderMap::new(),
        body: None,
        mode: network_types::RequestMode::NoCors,
        credentials: network_types::CredentialsMode::Omit,
        cache: network_types::CacheMode::Default,
        redirect: network_types::RedirectMode::Follow,
        referrer: None,
        referrer_policy: network_types::ReferrerPolicy::NoReferrer,
        integrity: None,
        keepalive: true,
        signal: None,
        priority: network_types::RequestPriority::High,
        window: None,
    };

    // Fetch the response
    let response = stack.fetch(request).await?;

    println!("Status: {}", response.status);
    println!("Headers: {:?}", response.headers);

    // Read response body
    match response.body {
        network_types::ResponseBody::Bytes(bytes) => {
            let body = String::from_utf8_lossy(&bytes);
            println!("Body: {}", body);
        }
        _ => println!("No body or streaming body"),
    }

    Ok(())
}
```

### POST Request with JSON Body

```rust
use network_types::{NetworkRequest, HttpMethod, RequestBody};
use bytes::Bytes;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let stack = create_network_stack();

    // Prepare JSON body
    let json_data = r#"{"name": "John Doe", "email": "john@example.com"}"#;
    let body_bytes = Bytes::from(json_data.as_bytes().to_vec());

    // Build headers
    let mut headers = http::HeaderMap::new();
    headers.insert(
        http::header::CONTENT_TYPE,
        http::HeaderValue::from_static("application/json"),
    );

    // Create POST request
    let request = NetworkRequest {
        url: Url::parse("https://httpbin.org/post")?,
        method: HttpMethod::Post,
        headers,
        body: Some(RequestBody::Bytes(body_bytes)),
        mode: network_types::RequestMode::Cors,
        credentials: network_types::CredentialsMode::Include,
        cache: network_types::CacheMode::NoStore,
        redirect: network_types::RedirectMode::Follow,
        referrer: None,
        referrer_policy: network_types::ReferrerPolicy::StrictOriginWhenCrossOrigin,
        integrity: None,
        keepalive: false,
        signal: None,
        priority: network_types::RequestPriority::High,
        window: None,
    };

    let response = stack.fetch(request).await?;

    println!("Status: {}", response.status);

    Ok(())
}
```

## HTTPS with TLS Configuration

Configure TLS for secure HTTPS connections:

```rust
use tls_manager::{TlsConfig, CertificateStore};
use network_stack::NetworkStack;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure TLS with ALPN protocols
    let tls_config = TlsConfig::new()
        .with_alpn_protocols(vec![
            b"h2".to_vec(),      // HTTP/2
            b"http/1.1".to_vec(), // HTTP/1.1
        ]);

    // Create network stack with TLS configuration
    let stack = create_network_stack_with_tls(tls_config);

    // Make HTTPS request
    let request = NetworkRequest {
        url: Url::parse("https://www.example.com")?,
        method: HttpMethod::Get,
        headers: http::HeaderMap::new(),
        body: None,
        mode: network_types::RequestMode::Cors,
        credentials: network_types::CredentialsMode::SameOrigin,
        cache: network_types::CacheMode::Default,
        redirect: network_types::RedirectMode::Follow,
        referrer: None,
        referrer_policy: network_types::ReferrerPolicy::StrictOriginWhenCrossOrigin,
        integrity: None,
        keepalive: true,
        signal: None,
        priority: network_types::RequestPriority::High,
        window: None,
    };

    let response = stack.fetch(request).await?;

    println!("Status: {}", response.status);
    println!("TLS negotiated protocol: {:?}", response.headers.get("x-alpn-protocol"));

    Ok(())
}
```

### Certificate Pinning

Pin certificates for enhanced security:

```rust
use tls_manager::CertificateStore;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut stack = create_network_stack();

    // Add certificate pin for a specific host
    // Pin hash should be SHA-256 of the certificate's SPKI
    let pin_hash = vec![
        0x12, 0x34, 0x56, 0x78, // ... (example hash)
    ];

    stack.add_certificate_pin("api.example.com", pin_hash);

    // Now requests to api.example.com will require the pinned certificate
    let request = NetworkRequest {
        url: Url::parse("https://api.example.com/data")?,
        method: HttpMethod::Get,
        headers: http::HeaderMap::new(),
        body: None,
        mode: network_types::RequestMode::Cors,
        credentials: network_types::CredentialsMode::SameOrigin,
        cache: network_types::CacheMode::Default,
        redirect: network_types::RedirectMode::Follow,
        referrer: None,
        referrer_policy: network_types::ReferrerPolicy::NoReferrer,
        integrity: None,
        keepalive: true,
        signal: None,
        priority: network_types::RequestPriority::High,
        window: None,
    };

    // Will fail if certificate doesn't match the pin
    let response = stack.fetch(request).await?;

    Ok(())
}
```

## WebSocket Connection

Establish a WebSocket connection for real-time communication:

```rust
use network_stack::NetworkStack;
use websocket_protocol::{WebSocketMessage, WebSocketState};
use url::Url;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let stack = create_network_stack();

    // Connect to WebSocket server
    let url = Url::parse("wss://echo.websocket.org/")?;
    let protocols = vec!["chat".to_string(), "superchat".to_string()];

    let mut ws_connection = stack.connect_websocket(url, protocols).await?;

    println!("WebSocket connected!");
    println!("Protocol: {:?}", ws_connection.protocol);
    println!("Extensions: {:?}", ws_connection.extensions);

    // Send a text message
    let message = WebSocketMessage::Text("Hello, WebSocket!".to_string());
    ws_connection.send(message).await?;

    // Receive messages
    while let Some(received) = ws_connection.receive().await? {
        match received {
            WebSocketMessage::Text(text) => {
                println!("Received text: {}", text);
                break; // Exit after first message for demo
            }
            WebSocketMessage::Binary(data) => {
                println!("Received binary: {} bytes", data.len());
            }
            WebSocketMessage::Ping(data) => {
                println!("Received ping");
                // Automatically handle pong if needed
                let pong = WebSocketMessage::Pong(data);
                ws_connection.send(pong).await?;
            }
            WebSocketMessage::Pong(_) => {
                println!("Received pong");
            }
            WebSocketMessage::Close(frame) => {
                println!("Connection closed: {:?}", frame);
                break;
            }
        }
    }

    // Close connection gracefully
    ws_connection.close(1000, "Goodbye".to_string()).await?;

    Ok(())
}
```

## Advanced Features

### CORS Validation

Validate Cross-Origin Resource Sharing (CORS) policies:

```rust
use cors_validator::CorsValidator;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let validator = CorsValidator::new();

    // Create a CORS request
    let request = NetworkRequest {
        url: Url::parse("https://api.example.com/data")?,
        method: HttpMethod::Get,
        headers: {
            let mut headers = http::HeaderMap::new();
            headers.insert(
                http::header::ORIGIN,
                http::HeaderValue::from_static("https://myapp.com"),
            );
            headers
        },
        mode: network_types::RequestMode::Cors,
        credentials: network_types::CredentialsMode::Include,
        cache: network_types::CacheMode::Default,
        redirect: network_types::RedirectMode::Follow,
        referrer: None,
        referrer_policy: network_types::ReferrerPolicy::NoReferrer,
        integrity: None,
        keepalive: false,
        signal: None,
        priority: network_types::RequestPriority::High,
        window: None,
    };

    // Validate CORS (typically done internally by the stack)
    let result = validator.validate_request(&request);

    if result.allowed {
        println!("CORS request is allowed");
        println!("Headers to add: {:?}", result.headers_to_add);
    } else {
        println!("CORS request blocked: {:?}", result.reason);
    }

    Ok(())
}
```

### HTTP Caching

Use HTTP caching to improve performance:

```rust
use http_cache::{HttpCache, CacheConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure cache
    let cache_config = CacheConfig::new(
        10 * 1024 * 1024,  // 10 MB max size
        3600,               // 1 hour max age
        true,               // enabled
    );

    let cache = HttpCache::new(cache_config);

    // Create request
    let request = NetworkRequest {
        url: Url::parse("https://api.example.com/data")?,
        method: HttpMethod::Get,
        headers: http::HeaderMap::new(),
        body: None,
        mode: network_types::RequestMode::Cors,
        credentials: network_types::CredentialsMode::SameOrigin,
        cache: network_types::CacheMode::Default,
        redirect: network_types::RedirectMode::Follow,
        referrer: None,
        referrer_policy: network_types::ReferrerPolicy::NoReferrer,
        integrity: None,
        keepalive: true,
        signal: None,
        priority: network_types::RequestPriority::High,
        window: None,
    };

    // Check cache first
    if let Some(cached_response) = cache.get(&request).await {
        if cached_response.is_fresh() {
            println!("Using cached response (age: {:?})", cached_response.age());
            return Ok(());
        }
    }

    // Fetch from network
    let stack = create_network_stack();
    let response = stack.fetch(request.clone()).await?;

    // Store in cache
    cache.store(&request, &response).await?;

    println!("Response cached");
    println!("Cache size: {} bytes", cache.current_size().await);
    println!("Cache entries: {}", cache.entry_count().await);

    Ok(())
}
```

### Request Scheduling

Prioritize requests for optimal loading:

```rust
use request_scheduler::{RequestScheduler, PendingRequest};
use network_types::RequestPriority;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let scheduler = RequestScheduler::new();

    // Schedule high-priority request (navigation, CSS)
    let high_priority_request = NetworkRequest {
        url: Url::parse("https://example.com/critical.css")?,
        method: HttpMethod::Get,
        headers: http::HeaderMap::new(),
        body: None,
        mode: network_types::RequestMode::NoCors,
        credentials: network_types::CredentialsMode::Omit,
        cache: network_types::CacheMode::Default,
        redirect: network_types::RedirectMode::Follow,
        referrer: None,
        referrer_policy: network_types::ReferrerPolicy::NoReferrer,
        integrity: None,
        keepalive: false,
        signal: None,
        priority: RequestPriority::High,  // Critical resource
        window: None,
    };

    // Schedule low-priority request (images, prefetch)
    let low_priority_request = NetworkRequest {
        url: Url::parse("https://example.com/image.jpg")?,
        method: HttpMethod::Get,
        headers: http::HeaderMap::new(),
        body: None,
        mode: network_types::RequestMode::NoCors,
        credentials: network_types::CredentialsMode::Omit,
        cache: network_types::CacheMode::Default,
        redirect: network_types::RedirectMode::Follow,
        referrer: None,
        referrer_policy: network_types::ReferrerPolicy::NoReferrer,
        integrity: None,
        keepalive: false,
        signal: None,
        priority: RequestPriority::Low,  // Non-critical resource
        window: None,
    };

    // High-priority requests are processed first
    println!("Scheduler ensures critical resources load first");

    Ok(())
}
```

### Bandwidth Limiting

Control bandwidth usage and track statistics:

```rust
use network_stack::NetworkStack;
use bandwidth_limiter::BandwidthStats;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let stack = create_network_stack();

    // Make some requests...
    let request = NetworkRequest {
        url: Url::parse("https://example.com/largefile.zip")?,
        method: HttpMethod::Get,
        headers: http::HeaderMap::new(),
        body: None,
        mode: network_types::RequestMode::NoCors,
        credentials: network_types::CredentialsMode::Omit,
        cache: network_types::CacheMode::NoStore,
        redirect: network_types::RedirectMode::Follow,
        referrer: None,
        referrer_policy: network_types::ReferrerPolicy::NoReferrer,
        integrity: None,
        keepalive: false,
        signal: None,
        priority: network_types::RequestPriority::Medium,
        window: None,
    };

    let response = stack.fetch(request).await?;

    // Get bandwidth statistics
    let stats = stack.get_bandwidth_stats();

    println!("Bytes sent: {}", stats.bytes_sent);
    println!("Bytes received: {}", stats.bytes_received);
    println!("Current download speed: {} KB/s", stats.download_speed_kbps);
    println!("Current upload speed: {} KB/s", stats.upload_speed_kbps);

    Ok(())
}
```

### DNS-over-HTTPS (DoH)

Use DNS-over-HTTPS for privacy:

```rust
use dns_resolver::{DnsResolver, DohConfig};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure DNS-over-HTTPS
    let doh_config = DohConfig {
        enabled: true,
        resolver_url: "https://dns.google/dns-query".to_string(),
    };

    let resolver = create_dns_resolver_with_doh(doh_config);

    // Resolve hostname
    let hostname = "www.example.com".to_string();
    let addresses = resolver.resolve(hostname.clone()).await?;

    println!("Resolved {} to:", hostname);
    for addr in addresses {
        println!("  - {}", addr);
    }

    // Resolve with timeout
    let timeout = Duration::from_secs(5);
    let addresses = resolver.resolve_with_timeout(hostname, timeout).await?;

    Ok(())
}
```

## Helper Function Examples

Here are some helper functions used in the examples above:

```rust
use network_stack::NetworkStack;
use network_stack_impl::NetworkStackImpl; // Your implementation

fn create_network_stack() -> impl NetworkStack {
    NetworkStackImpl::new()
}

fn create_network_stack_with_tls(tls_config: TlsConfig) -> impl NetworkStack {
    NetworkStackImpl::with_tls(tls_config)
}

fn create_dns_resolver_with_doh(doh_config: DohConfig) -> impl DnsResolver {
    DnsResolverImpl::with_doh(doh_config)
}
```

## Error Handling

All network operations return `Result<T, NetworkError>`. Handle errors appropriately:

```rust
use network_errors::NetworkError;

#[tokio::main]
async fn main() {
    let stack = create_network_stack();

    let request = NetworkRequest {
        url: Url::parse("https://example.com/api").unwrap(),
        method: HttpMethod::Get,
        headers: http::HeaderMap::new(),
        body: None,
        mode: network_types::RequestMode::Cors,
        credentials: network_types::CredentialsMode::SameOrigin,
        cache: network_types::CacheMode::Default,
        redirect: network_types::RedirectMode::Follow,
        referrer: None,
        referrer_policy: network_types::ReferrerPolicy::NoReferrer,
        integrity: None,
        keepalive: true,
        signal: None,
        priority: network_types::RequestPriority::High,
        window: None,
    };

    match stack.fetch(request).await {
        Ok(response) => {
            println!("Success: {}", response.status);
        }
        Err(NetworkError::ConnectionFailed(msg)) => {
            eprintln!("Connection failed: {}", msg);
        }
        Err(NetworkError::Timeout(duration)) => {
            eprintln!("Request timed out after {:?}", duration);
        }
        Err(NetworkError::DnsError(msg)) => {
            eprintln!("DNS resolution failed: {}", msg);
        }
        Err(NetworkError::TlsError(msg)) => {
            eprintln!("TLS error: {}", msg);
        }
        Err(NetworkError::CorsError(msg)) => {
            eprintln!("CORS violation: {}", msg);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }
}
```

## Performance Monitoring

Track request timing using W3C Resource Timing:

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let stack = create_network_stack();

    let request = NetworkRequest {
        url: Url::parse("https://example.com")?,
        method: HttpMethod::Get,
        headers: http::HeaderMap::new(),
        body: None,
        mode: network_types::RequestMode::Cors,
        credentials: network_types::CredentialsMode::SameOrigin,
        cache: network_types::CacheMode::Default,
        redirect: network_types::RedirectMode::Follow,
        referrer: None,
        referrer_policy: network_types::ReferrerPolicy::NoReferrer,
        integrity: None,
        keepalive: true,
        signal: None,
        priority: network_types::RequestPriority::High,
        window: None,
    };

    let response = stack.fetch(request).await?;

    // Access W3C Resource Timing data
    let timing = &response.timing;

    println!("Performance Metrics:");
    println!("  DNS lookup: {:.2}ms", timing.domain_lookup_end - timing.domain_lookup_start);
    println!("  TCP connect: {:.2}ms", timing.connect_end - timing.connect_start);
    println!("  TLS handshake: {:.2}ms", timing.connect_end - timing.secure_connection_start);
    println!("  Request: {:.2}ms", timing.response_start - timing.request_start);
    println!("  Response: {:.2}ms", timing.response_end - timing.response_start);
    println!("  Total: {:.2}ms", timing.response_end - timing.start_time);
    println!("  Transfer size: {} bytes", timing.transfer_size);
    println!("  Encoded body size: {} bytes", timing.encoded_body_size);
    println!("  Decoded body size: {} bytes", timing.decoded_body_size);

    Ok(())
}
```

## Next Steps

- See [API-REFERENCE.md](API-REFERENCE.md) for complete API documentation
- Check the `examples/` directory for more working examples
- Read component-specific documentation in `components/*/README.md`
- Explore the generated rustdoc at `target/doc/network_stack/index.html`

## Additional Resources

- [Rust async programming](https://rust-lang.github.io/async-book/)
- [Tokio runtime documentation](https://docs.rs/tokio/)
- [W3C Fetch API specification](https://fetch.spec.whatwg.org/)
- [W3C Resource Timing specification](https://www.w3.org/TR/resource-timing/)
