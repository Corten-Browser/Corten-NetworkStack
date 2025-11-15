//! Proxy Request Example
//!
//! This example demonstrates how to make HTTP requests through a proxy server.
//!
//! Run with:
//! ```sh
//! cargo run --example proxy_request
//! ```

use network_stack::NetworkStack;
use network_stack_impl::NetworkStackImpl;
use network_types::{HttpMethod, NetworkRequest, RequestMode, CredentialsMode, CacheMode, RedirectMode, ReferrerPolicy, RequestPriority, ResponseBody, ProxyConfig};
use url::Url;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Proxy Request Example ===\n");

    // Create network stack
    let mut stack = NetworkStackImpl::new();

    // Configure HTTP proxy
    // Note: Replace with your actual proxy server details
    let proxy_config = ProxyConfig {
        proxy_type: network_types::ProxyType::Http,
        host: "proxy.example.com".to_string(),
        port: 8080,
        username: None,  // Add if proxy requires authentication
        password: None,
    };

    println!("Configuring proxy:");
    println!("  Type: {:?}", proxy_config.proxy_type);
    println!("  Host: {}", proxy_config.host);
    println!("  Port: {}", proxy_config.port);

    // Set proxy configuration
    stack.set_proxy_config(Some(proxy_config));

    // Build request
    let mut headers = http::HeaderMap::new();
    headers.insert(
        http::header::USER_AGENT,
        http::HeaderValue::from_static("Corten-NetworkStack/0.1.0"),
    );

    let request = NetworkRequest {
        url: Url::parse("http://httpbin.org/get")?,
        method: HttpMethod::Get,
        headers,
        body: None,
        mode: RequestMode::Cors,
        credentials: CredentialsMode::SameOrigin,
        cache: CacheMode::NoStore,  // Don't cache proxied requests
        redirect: RedirectMode::Follow,
        referrer: None,
        referrer_policy: ReferrerPolicy::NoReferrer,
        integrity: None,
        keepalive: false,
        signal: None,
        priority: RequestPriority::High,
        window: None,
    };

    println!("\nFetching through proxy: {}", request.url);

    // Fetch the response through proxy
    match stack.fetch(request).await {
        Ok(response) => {
            println!("\n--- Response ---");
            println!("Status: {} {}", response.status, response.status_text);

            // Check for proxy-related headers
            println!("\n--- Proxy Headers ---");
            if let Some(via) = response.headers.get("via") {
                println!("Via: {:?}", via);
            }
            if let Some(forwarded) = response.headers.get("forwarded") {
                println!("Forwarded: {:?}", forwarded);
            }
            if let Some(x_forwarded_for) = response.headers.get("x-forwarded-for") {
                println!("X-Forwarded-For: {:?}", x_forwarded_for);
            }

            // Display body
            println!("\n--- Response Body ---");
            match response.body {
                ResponseBody::Bytes(bytes) => {
                    let body = String::from_utf8_lossy(&bytes);
                    println!("{}", body);
                }
                ResponseBody::Empty => println!("(empty)"),
                ResponseBody::Stream(_) => println!("(streaming body)"),
            }

            // Display timing
            println!("\n--- Performance ---");
            let timing = &response.timing;
            println!("Total time: {:.2}ms", timing.response_end - timing.start_time);
        }
        Err(e) => {
            eprintln!("\n✗ Error: {}", e);
            eprintln!("\nNote: This example requires a valid proxy server.");
            eprintln!("Update the proxy configuration in the code to use a real proxy.");
        }
    }

    // Example: SOCKS5 proxy configuration
    println!("\n\n--- Alternative: SOCKS5 Proxy ---");
    let socks_proxy = ProxyConfig {
        proxy_type: network_types::ProxyType::Socks5,
        host: "socks.example.com".to_string(),
        port: 1080,
        username: Some("user".to_string()),
        password: Some("pass".to_string()),
    };

    println!("SOCKS5 proxy configuration:");
    println!("  Type: {:?}", socks_proxy.proxy_type);
    println!("  Host: {}", socks_proxy.host);
    println!("  Port: {}", socks_proxy.port);
    println!("  Authentication: {}", socks_proxy.username.is_some());

    // To use SOCKS5:
    // stack.set_proxy_config(Some(socks_proxy));

    // To disable proxy:
    println!("\n--- Disabling Proxy ---");
    stack.set_proxy_config(None);
    println!("✓ Proxy disabled - requests will go direct");

    Ok(())
}
