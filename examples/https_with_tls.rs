//! HTTPS with TLS Configuration Example
//!
//! This example demonstrates how to make secure HTTPS requests
//! with custom TLS configuration including ALPN protocol negotiation.
//!
//! Run with:
//! ```sh
//! cargo run --example https_with_tls
//! ```

use network_stack::NetworkStack;
use network_stack_impl::NetworkStackImpl;
use network_types::{HttpMethod, NetworkRequest, RequestMode, CredentialsMode, CacheMode, RedirectMode, ReferrerPolicy, RequestPriority, ResponseBody};
use tls_manager::TlsConfig;
use url::Url;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== HTTPS with TLS Configuration Example ===\n");

    // Configure TLS with ALPN protocols
    // This allows negotiation of HTTP/2 or HTTP/1.1
    let tls_config = TlsConfig::new()
        .with_alpn_protocols(vec![
            b"h2".to_vec(),       // HTTP/2
            b"http/1.1".to_vec(), // HTTP/1.1
        ]);

    // Create network stack with TLS configuration
    let stack = NetworkStackImpl::with_tls(tls_config);

    // Build HTTPS request
    let mut headers = http::HeaderMap::new();
    headers.insert(
        http::header::USER_AGENT,
        http::HeaderValue::from_static("Corten-NetworkStack/0.1.0"),
    );

    let request = NetworkRequest {
        url: Url::parse("https://www.example.com")?,
        method: HttpMethod::Get,
        headers,
        body: None,
        mode: RequestMode::Cors,
        credentials: CredentialsMode::SameOrigin,
        cache: CacheMode::Default,
        redirect: RedirectMode::Follow,
        referrer: None,
        referrer_policy: ReferrerPolicy::StrictOriginWhenCrossOrigin,
        integrity: None,
        keepalive: true,
        signal: None,
        priority: RequestPriority::High,
        window: None,
    };

    println!("Fetching: {}", request.url);

    // Fetch the response
    let response = stack.fetch(request).await?;

    // Display response information
    println!("\n--- Response ---");
    println!("Status: {} {}", response.status, response.status_text);
    println!("URL: {}", response.url);

    // Display TLS-related headers
    println!("\n--- TLS Information ---");
    if let Some(server) = response.headers.get(http::header::SERVER) {
        println!("Server: {:?}", server);
    }
    if let Some(strict_transport) = response.headers.get("strict-transport-security") {
        println!("HSTS: {:?}", strict_transport);
    }

    // Display timing - TLS handshake time
    println!("\n--- TLS Performance ---");
    let timing = &response.timing;
    if timing.secure_connection_start > 0.0 {
        let tls_time = timing.connect_end - timing.secure_connection_start;
        println!("TLS handshake: {:.2}ms", tls_time);
    }
    println!("Total connect time: {:.2}ms", timing.connect_end - timing.connect_start);
    println!("Total request time: {:.2}ms", timing.response_end - timing.start_time);

    // Display body preview
    println!("\n--- Body Preview ---");
    match response.body {
        ResponseBody::Bytes(bytes) => {
            let body = String::from_utf8_lossy(&bytes);
            let preview = if body.len() > 500 {
                &body[..500]
            } else {
                &body
            };
            println!("{}", preview);
            if body.len() > 500 {
                println!("\n... ({} more bytes)", body.len() - 500);
            }
        }
        ResponseBody::Empty => println!("(empty)"),
        ResponseBody::Stream(_) => println!("(streaming body)"),
    }

    Ok(())
}
