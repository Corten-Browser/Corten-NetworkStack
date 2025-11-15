//! Basic HTTP Request Example
//!
//! This example demonstrates how to make a simple HTTP GET request
//! using the Corten Network Stack.
//!
//! Run with:
//! ```sh
//! cargo run --example basic_http_request
//! ```

use network_stack::NetworkStack;
use network_stack_impl::NetworkStackImpl;
use network_types::{HttpMethod, NetworkRequest, RequestMode, CredentialsMode, CacheMode, RedirectMode, ReferrerPolicy, RequestPriority, ResponseBody};
use url::Url;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Basic HTTP Request Example ===\n");

    // Create the network stack
    let stack = NetworkStackImpl::new();

    // Build a simple GET request
    let request = NetworkRequest {
        url: Url::parse("http://httpbin.org/get")?,
        method: HttpMethod::Get,
        headers: http::HeaderMap::new(),
        body: None,
        mode: RequestMode::NoCors,
        credentials: CredentialsMode::Omit,
        cache: CacheMode::Default,
        redirect: RedirectMode::Follow,
        referrer: None,
        referrer_policy: ReferrerPolicy::NoReferrer,
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
    println!("Type: {:?}", response.type_);
    println!("Redirected: {}", response.redirected);

    // Display headers
    println!("\n--- Headers ---");
    for (name, value) in response.headers.iter() {
        println!("{}: {:?}", name, value);
    }

    // Display body
    println!("\n--- Body ---");
    match response.body {
        ResponseBody::Bytes(bytes) => {
            let body = String::from_utf8_lossy(&bytes);
            println!("{}", body);
        }
        ResponseBody::Empty => println!("(empty)"),
        ResponseBody::Stream(_) => println!("(streaming body)"),
    }

    // Display timing information
    println!("\n--- Performance Metrics ---");
    let timing = &response.timing;
    println!("DNS lookup: {:.2}ms", timing.domain_lookup_end - timing.domain_lookup_start);
    println!("TCP connect: {:.2}ms", timing.connect_end - timing.connect_start);
    println!("Request: {:.2}ms", timing.response_start - timing.request_start);
    println!("Response: {:.2}ms", timing.response_end - timing.response_start);
    println!("Total time: {:.2}ms", timing.response_end - timing.start_time);
    println!("Transfer size: {} bytes", timing.transfer_size);

    Ok(())
}
