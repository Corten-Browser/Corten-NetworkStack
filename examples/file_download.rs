//! File Download Example
//!
//! This example demonstrates how to download a file using streaming,
//! with progress tracking and bandwidth monitoring.
//!
//! Run with:
//! ```sh
//! cargo run --example file_download
//! ```

use network_stack::NetworkStack;
use network_stack_impl::NetworkStackImpl;
use network_types::{HttpMethod, NetworkRequest, RequestMode, CredentialsMode, CacheMode, RedirectMode, ReferrerPolicy, RequestPriority};
use url::Url;
use tokio::io::AsyncWriteExt;
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== File Download Example ===\n");

    // Create network stack
    let stack = NetworkStackImpl::new();

    // URL of file to download (example: a small test file)
    let file_url = "https://httpbin.org/bytes/1048576"; // 1 MB test file
    let output_path = "/tmp/downloaded_file.bin";

    println!("Downloading file:");
    println!("  URL: {}", file_url);
    println!("  Output: {}", output_path);

    // Build download request
    let mut headers = http::HeaderMap::new();
    headers.insert(
        http::header::USER_AGENT,
        http::HeaderValue::from_static("Corten-NetworkStack/0.1.0"),
    );
    headers.insert(
        http::header::ACCEPT,
        http::HeaderValue::from_static("*/*"),
    );

    let request = NetworkRequest {
        url: Url::parse(file_url)?,
        method: HttpMethod::Get,
        headers,
        body: None,
        mode: RequestMode::Cors,
        credentials: CredentialsMode::SameOrigin,
        cache: CacheMode::NoStore,
        redirect: RedirectMode::Follow,
        referrer: None,
        referrer_policy: ReferrerPolicy::NoReferrer,
        integrity: None,
        keepalive: false,
        signal: None,
        priority: RequestPriority::High,
        window: None,
    };

    println!("\n--- Starting Download ---");

    // Use streaming to download the file
    let mut stream = stack.stream_response(request).await?;

    // Create output file
    let mut file = tokio::fs::File::create(output_path).await?;

    // Track progress
    let mut total_bytes = 0u64;
    let mut chunk_count = 0u32;

    println!("Receiving data...\n");

    // Stream and write chunks
    while let Some(chunk_result) = stream.next().await {
        match chunk_result {
            Ok(chunk) => {
                let chunk_size = chunk.len();
                total_bytes += chunk_size as u64;
                chunk_count += 1;

                // Write chunk to file
                file.write_all(&chunk).await?;

                // Display progress every 10 chunks
                if chunk_count % 10 == 0 {
                    println!("  Progress: {} bytes ({} chunks)", total_bytes, chunk_count);

                    // Get bandwidth stats
                    let stats = stack.get_bandwidth_stats();
                    println!("  Download speed: {:.2} KB/s", stats.download_speed_kbps);
                }
            }
            Err(e) => {
                eprintln!("\n✗ Error receiving chunk: {}", e);
                return Err(e.into());
            }
        }
    }

    // Flush and close file
    file.flush().await?;
    drop(file);

    println!("\n--- Download Complete ---");
    println!("✓ File saved to: {}", output_path);
    println!("  Total size: {} bytes ({:.2} MB)", total_bytes, total_bytes as f64 / 1_048_576.0);
    println!("  Chunks received: {}", chunk_count);

    // Get final bandwidth statistics
    let final_stats = stack.get_bandwidth_stats();
    println!("\n--- Bandwidth Statistics ---");
    println!("  Total downloaded: {} bytes", final_stats.bytes_received);
    println!("  Average speed: {:.2} KB/s", final_stats.download_speed_kbps);

    // Verify file exists and has correct size
    let metadata = tokio::fs::metadata(output_path).await?;
    println!("\n--- File Verification ---");
    println!("  File exists: {}", metadata.is_file());
    println!("  File size: {} bytes", metadata.len());

    if metadata.len() == total_bytes {
        println!("  ✓ Size verification passed");
    } else {
        println!("  ✗ Size mismatch!");
    }

    // Example: Download with resume support (Range header)
    println!("\n\n--- Example: Resumable Download ---");
    println!("To resume a download from byte 1000:");

    let mut resume_headers = http::HeaderMap::new();
    resume_headers.insert(
        http::header::RANGE,
        http::HeaderValue::from_static("bytes=1000-"),
    );

    println!("  Add header: Range: bytes=1000-");
    println!("  Server will respond with 206 Partial Content if supported");

    // Example: Download multiple files concurrently
    println!("\n--- Example: Concurrent Downloads ---");
    println!("To download multiple files concurrently:");
    println!("  1. Create multiple download tasks using tokio::spawn");
    println!("  2. Each task streams its own file");
    println!("  3. Use tokio::join! to wait for all downloads");

    Ok(())
}
