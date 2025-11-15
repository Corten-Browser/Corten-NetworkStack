# Corten Network Stack - Examples

This directory contains practical, runnable examples demonstrating the features of the Corten Network Stack.

## Prerequisites

- Rust 1.70 or later
- Cargo
- Internet connection (for live examples)

## Running Examples

Each example is a standalone Rust program. Run them using:

```bash
cargo run --example <example_name>
```

## Available Examples

### 1. Basic HTTP Request (`basic_http_request.rs`)

Demonstrates:
- Creating a simple HTTP GET request
- Fetching and displaying response
- Accessing headers and body
- Performance timing metrics

**Run:**
```bash
cargo run --example basic_http_request
```

**What it does:**
Makes a GET request to `http://httpbin.org/get` and displays the complete response including headers, body, and performance metrics.

---

### 2. HTTPS with TLS (`https_with_tls.rs`)

Demonstrates:
- TLS configuration with ALPN protocol negotiation
- Secure HTTPS connections
- TLS handshake timing
- HSTS (HTTP Strict Transport Security) headers

**Run:**
```bash
cargo run --example https_with_tls
```

**What it does:**
Configures TLS to support HTTP/2 and HTTP/1.1, makes a secure request to `https://www.example.com`, and displays TLS-specific information including handshake time.

**Key features:**
- ALPN (Application-Layer Protocol Negotiation)
- TLS 1.2/1.3 support
- Performance metrics for TLS handshake

---

### 3. WebSocket Client (`websocket_client.rs`)

Demonstrates:
- Establishing WebSocket connections
- Sending and receiving text messages
- Sending and receiving binary data
- Ping/Pong heartbeat
- Graceful connection closure

**Run:**
```bash
cargo run --example websocket_client
```

**What it does:**
Connects to the WebSocket echo server at `wss://echo.websocket.org/`, sends various types of messages (text, binary, ping), receives echoed responses, and closes the connection gracefully.

**Message types demonstrated:**
- Text messages
- Binary messages
- Ping/Pong (WebSocket heartbeat)
- Close frames

---

### 4. Proxy Request (`proxy_request.rs`)

Demonstrates:
- HTTP proxy configuration
- SOCKS5 proxy support
- Authenticated proxy connections
- Proxy-related headers

**Run:**
```bash
cargo run --example proxy_request
```

**Note:** This example requires a valid proxy server. Update the proxy configuration in the code with your actual proxy details before running.

**What it demonstrates:**
- Configuring HTTP proxy
- Configuring SOCKS5 proxy with authentication
- Enabling/disabling proxy
- Inspecting proxy-related headers (`Via`, `Forwarded`, `X-Forwarded-For`)

**Proxy types supported:**
- HTTP proxy
- HTTPS proxy (CONNECT method)
- SOCKS5 proxy

---

### 5. File Download (`file_download.rs`)

Demonstrates:
- Streaming large file downloads
- Progress tracking
- Bandwidth monitoring
- Chunk-by-chunk processing
- Resumable downloads (Range headers)

**Run:**
```bash
cargo run --example file_download
```

**What it does:**
Downloads a 1 MB test file from `https://httpbin.org/bytes/1048576` using streaming, tracks download progress, monitors bandwidth, and saves the file to `/tmp/downloaded_file.bin`.

**Features demonstrated:**
- Streaming API (`stream_response`)
- Real-time progress tracking
- Bandwidth statistics
- Writing chunks to disk
- File verification

**Advanced concepts shown:**
- Resume support with Range headers
- Concurrent downloads pattern

---

## Example Output

### Basic HTTP Request Output

```
=== Basic HTTP Request Example ===

Fetching: http://httpbin.org/get

--- Response ---
Status: 200 OK
Type: Basic
Redirected: false

--- Headers ---
date: "Mon, 15 Nov 2025 10:30:00 GMT"
content-type: "application/json"
content-length: "312"

--- Body ---
{
  "args": {},
  "headers": {
    "Host": "httpbin.org"
  },
  "origin": "203.0.113.1",
  "url": "http://httpbin.org/get"
}

--- Performance Metrics ---
DNS lookup: 12.34ms
TCP connect: 45.67ms
Request: 23.45ms
Response: 67.89ms
Total time: 150.35ms
Transfer size: 512 bytes
```

## Modifying Examples

All examples are fully documented and can be modified to suit your needs:

1. **Change URLs**: Update the `Url::parse()` calls to test different endpoints
2. **Add headers**: Modify the `headers` HashMap to include custom headers
3. **Adjust configuration**: Change TLS settings, proxy configuration, etc.
4. **Add error handling**: Expand error handling for production use

## Common Patterns

### Creating Headers

```rust
let mut headers = http::HeaderMap::new();
headers.insert(
    http::header::USER_AGENT,
    http::HeaderValue::from_static("MyApp/1.0"),
);
headers.insert(
    http::header::CONTENT_TYPE,
    http::HeaderValue::from_static("application/json"),
);
```

### Error Handling

```rust
match stack.fetch(request).await {
    Ok(response) => {
        // Handle success
    }
    Err(NetworkError::Timeout(duration)) => {
        eprintln!("Request timed out after {:?}", duration);
    }
    Err(NetworkError::ConnectionFailed(msg)) => {
        eprintln!("Connection failed: {}", msg);
    }
    Err(e) => {
        eprintln!("Error: {}", e);
    }
}
```

### Reading Response Body

```rust
match response.body {
    ResponseBody::Bytes(bytes) => {
        let text = String::from_utf8_lossy(&bytes);
        println!("{}", text);
    }
    ResponseBody::Stream(stream) => {
        // Handle streaming body
        while let Some(chunk) = stream.next().await {
            // Process chunk
        }
    }
    ResponseBody::Empty => {
        println!("No body");
    }
}
```

## Testing Against Local Servers

To test against your own servers:

1. Start a local HTTP server:
   ```bash
   python3 -m http.server 8000
   ```

2. Modify example URL:
   ```rust
   let url = Url::parse("http://localhost:8000")?;
   ```

3. Run the example

## Troubleshooting

### "Connection refused" errors

- Ensure you have internet connectivity
- Check if the target server is accessible
- Verify firewall settings

### TLS/SSL errors

- Ensure system has up-to-date CA certificates
- Check TLS version compatibility
- Verify server certificate is valid

### Proxy errors

- Verify proxy server address and port
- Check proxy authentication credentials
- Ensure proxy supports the protocol (HTTP/HTTPS/SOCKS5)

### WebSocket errors

- Verify WebSocket server is running
- Check WebSocket protocol negotiation
- Ensure firewall allows WebSocket connections

## Additional Resources

- [Quick Start Guide](../docs/QUICK-START.md)
- [API Reference](../docs/API-REFERENCE.md)
- [Generated rustdoc](../target/doc/network_stack/index.html)
- Component documentation in `components/*/README.md`

## Contributing

Found an issue or want to add more examples? Contributions are welcome!

1. Fork the repository
2. Create your example in this directory
3. Update this README
4. Submit a pull request

## License

These examples are part of the Corten Network Stack project and follow the same license.
