// Network Stack Performance Benchmarks
//
// This benchmark suite measures performance of key network stack operations
// using the performance_benchmarks framework.

use performance_benchmarks::{BenchmarkConfig, BenchmarkRunner};
use std::time::Duration;

// Import components for benchmarking
use cors_validator::{CorsConfig, CorsValidator};
use content_encoding::{ContentEncoder, Encoding};
use request_scheduler::{RequestScheduler, RequestPriority};
use bandwidth_limiter::{BandwidthLimiter, NetworkCondition};
use url_handlers::{DataUrlHandler, FileUrlHandler, FileSecurityPolicy};
use csp_processor::CspProcessor;
use network_types::{HttpMethod, NetworkRequest, RequestMode};
use std::collections::HashMap;

#[tokio::main]
async fn main() {
    println!("=".repeat(70));
    println!("CORTEN NETWORK STACK - PERFORMANCE BENCHMARKS");
    println!("=".repeat(70));
    println!();

    run_cors_benchmarks().await;
    run_content_encoding_benchmarks().await;
    run_request_scheduler_benchmarks().await;
    run_bandwidth_limiter_benchmarks().await;
    run_url_handler_benchmarks().await;
    run_csp_processor_benchmarks().await;

    println!("=".repeat(70));
    println!("BENCHMARK SUITE COMPLETE");
    println!("=".repeat(70));
}

async fn run_cors_benchmarks() {
    println!("\n📊 CORS VALIDATOR BENCHMARKS");
    println!("-".repeat(70));

    let config = BenchmarkConfig {
        warmup_iterations: 100,
        iterations: 1000,
        name: "CORS Validation".to_string(),
    };

    let runner = BenchmarkRunner::new(config);

    // Setup
    let cors_config = CorsConfig {
        enforce_same_origin: false,
        allow_credentials: true,
    };
    let validator = CorsValidator::new(cors_config);

    let request = NetworkRequest {
        url: "https://example.com/api".to_string(),
        method: HttpMethod::Get,
        headers: HashMap::new(),
        body: None,
        mode: RequestMode::Cors,
        credentials: network_types::CredentialsMode::Include,
        cache_mode: network_types::CacheMode::Default,
        redirect_mode: network_types::RedirectMode::Follow,
    };

    // Benchmark CORS validation
    let result = runner.run_benchmark("CORS Request Validation", || async {
        let _ = validator.validate_request(&request, "https://other.com");
    }).await;

    println!("{}", runner.format_result(&result));
}

async fn run_content_encoding_benchmarks() {
    println!("\n📊 CONTENT ENCODING BENCHMARKS");
    println!("-".repeat(70));

    let config = BenchmarkConfig {
        warmup_iterations: 50,
        iterations: 500,
        name: "Content Encoding".to_string(),
    };

    let runner = BenchmarkRunner::new(config);
    let encoder = ContentEncoder::new();

    // Test data: 1KB of text
    let data = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. ".repeat(20);
    let data_bytes = data.as_bytes();

    // Benchmark gzip encoding
    let result = runner.run_benchmark("Gzip Encode (1KB)", || async {
        let _ = encoder.encode(data_bytes, Encoding::Gzip).unwrap();
    }).await;
    println!("{}", runner.format_result(&result));

    // Benchmark brotli encoding
    let result = runner.run_benchmark("Brotli Encode (1KB)", || async {
        let _ = encoder.encode(data_bytes, Encoding::Brotli).unwrap();
    }).await;
    println!("{}", runner.format_result(&result));

    // Benchmark deflate encoding
    let result = runner.run_benchmark("Deflate Encode (1KB)", || async {
        let _ = encoder.encode(data_bytes, Encoding::Deflate).unwrap();
    }).await;
    println!("{}", runner.format_result(&result));

    // Benchmark decoding
    let compressed = encoder.encode(data_bytes, Encoding::Gzip).unwrap();
    let result = runner.run_benchmark("Gzip Decode (1KB)", || async {
        let _ = encoder.decode(&compressed, Encoding::Gzip).unwrap();
    }).await;
    println!("{}", runner.format_result(&result));
}

async fn run_request_scheduler_benchmarks() {
    println!("\n📊 REQUEST SCHEDULER BENCHMARKS");
    println!("-".repeat(70));

    let config = BenchmarkConfig {
        warmup_iterations: 100,
        iterations: 1000,
        name: "Request Scheduling".to_string(),
    };

    let runner = BenchmarkRunner::new(config);

    let request = NetworkRequest {
        url: "https://example.com/api".to_string(),
        method: HttpMethod::Get,
        headers: HashMap::new(),
        body: None,
        mode: RequestMode::Cors,
        credentials: network_types::CredentialsMode::Include,
        cache_mode: network_types::CacheMode::Default,
        redirect_mode: network_types::RedirectMode::Follow,
    };

    // Benchmark scheduling
    let result = runner.run_benchmark("Schedule High Priority Request", || async {
        let mut scheduler = RequestScheduler::new(10);
        scheduler.schedule(request.clone(), RequestPriority::High);
    }).await;
    println!("{}", runner.format_result(&result));

    // Benchmark next_request
    let mut scheduler = RequestScheduler::new(10);
    for _ in 0..100 {
        scheduler.schedule(request.clone(), RequestPriority::Medium);
    }

    let result = runner.run_benchmark("Get Next Request (100 queued)", || async {
        let _ = scheduler.next_request();
    }).await;
    println!("{}", runner.format_result(&result));
}

async fn run_bandwidth_limiter_benchmarks() {
    println!("\n📊 BANDWIDTH LIMITER BENCHMARKS");
    println!("-".repeat(70));

    let config = BenchmarkConfig {
        warmup_iterations: 50,
        iterations: 200,
        name: "Bandwidth Limiting".to_string(),
    };

    let runner = BenchmarkRunner::new(config);

    let mut limiter = BandwidthLimiter::new(None, None, Duration::from_millis(0));
    limiter.apply_condition(NetworkCondition::G4);

    // Test data: 10KB
    let data = vec![0u8; 10_000];

    // Benchmark throttling calculation (actual sleep not measured)
    let result = runner.run_benchmark("Calculate Throttle (10KB, 4G)", || async {
        let _ = limiter.throttle_download(&data).await;
    }).await;
    println!("{}", runner.format_result(&result));

    // Benchmark condition application
    let result = runner.run_benchmark("Apply Network Condition", || async {
        let mut limiter = BandwidthLimiter::new(None, None, Duration::from_millis(0));
        limiter.apply_condition(NetworkCondition::WiFi);
    }).await;
    println!("{}", runner.format_result(&result));
}

async fn run_url_handler_benchmarks() {
    println!("\n📊 URL HANDLER BENCHMARKS");
    println!("-".repeat(70));

    let config = BenchmarkConfig {
        warmup_iterations: 100,
        iterations: 1000,
        name: "URL Handlers".to_string(),
    };

    let runner = BenchmarkRunner::new(config);

    // Benchmark data URL parsing
    let data_url = "data:text/plain;charset=utf-8;base64,SGVsbG8gV29ybGQh";
    let handler = DataUrlHandler;

    let result = runner.run_benchmark("Parse Data URL (base64)", || async {
        let _ = handler.parse(data_url).unwrap();
    }).await;
    println!("{}", runner.format_result(&result));

    // Benchmark plain data URL
    let plain_url = "data:text/plain,Hello%20World";
    let result = runner.run_benchmark("Parse Data URL (plain)", || async {
        let _ = handler.parse(plain_url).unwrap();
    }).await;
    println!("{}", runner.format_result(&result));
}

async fn run_csp_processor_benchmarks() {
    println!("\n📊 CSP PROCESSOR BENCHMARKS");
    println!("-".repeat(70));

    let config = BenchmarkConfig {
        warmup_iterations: 100,
        iterations: 1000,
        name: "CSP Processing".to_string(),
    };

    let runner = BenchmarkRunner::new(config);

    // Complex CSP policy
    let csp_header = "default-src 'self'; script-src 'self' 'unsafe-inline' https://cdn.example.com; \
                      style-src 'self' 'unsafe-inline'; img-src * data:; font-src 'self' data:; \
                      connect-src 'self' https://api.example.com; frame-ancestors 'none'";

    // Benchmark CSP parsing
    let result = runner.run_benchmark("Parse Complex CSP Policy", || async {
        let _ = CspProcessor::new(csp_header).unwrap();
    }).await;
    println!("{}", runner.format_result(&result));

    // Benchmark CSP checking
    let processor = CspProcessor::new(csp_header).unwrap();
    let result = runner.run_benchmark("Check Script Source", || async {
        let _ = processor.check_source(
            csp_processor::CspDirective::ScriptSrc,
            "https://cdn.example.com/script.js"
        );
    }).await;
    println!("{}", runner.format_result(&result));
}
