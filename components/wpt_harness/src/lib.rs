//! WPT (Web Platform Tests) Harness Adapter
//!
//! This module provides integration between the Corten-NetworkStack and the
//! Web Platform Tests suite. It translates WPT test requests into NetworkStack
//! API calls and formats responses for WPT's test runner.
//!
//! # Architecture
//!
//! ```text
//! WPT Test Server (Python)
//!      ↓ HTTP
//! WPT Harness Adapter (this crate)
//!      ↓ Rust API
//! NetworkStack (core implementation)
//! ```
//!
//! # Usage
//!
//! Run WPT tests using the harness binary:
//!
//! ```bash
//! # Build the harness
//! cargo build --release --bin wpt_runner
//!
//! # Run WPT tests (from WPT repository)
//! cd /path/to/wpt
//! ./wpt run --binary ./target/release/wpt_runner fetch
//! ```

use serde::{Deserialize, Serialize};

pub mod http_tests;

/// WPT test request from test server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WptRequest {
    /// HTTP method (GET, POST, etc.)
    pub method: String,
    /// Request URL
    pub url: String,
    /// Request headers
    #[serde(default)]
    pub headers: std::collections::HashMap<String, String>,
    /// Request body (optional)
    #[serde(default)]
    pub body: Option<Vec<u8>>,
    /// Test timeout in milliseconds
    #[serde(default)]
    pub timeout_ms: Option<u64>,
}

/// WPT test response to test server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WptResponse {
    /// HTTP status code
    pub status: u16,
    /// Response headers
    pub headers: std::collections::HashMap<String, String>,
    /// Response body
    #[serde(with = "serde_bytes")]
    pub body: Vec<u8>,
    /// Test execution time in milliseconds
    pub duration_ms: u64,
}

/// WPT test result
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum WptTestResult {
    /// Test passed
    Pass,
    /// Test failed
    Fail { reason: String },
    /// Test timed out
    Timeout,
    /// Test skipped
    Skip { reason: String },
    /// Test had an error
    Error { message: String },
}

/// WPT harness adapter
pub struct WptHarness {
    /// Whether to log verbose output
    verbose: bool,
}

impl WptHarness {
    /// Create a new WPT harness
    pub fn new() -> Self {
        Self { verbose: false }
    }

    /// Enable verbose logging
    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    /// Execute a WPT test request using NetworkStack
    ///
    /// This method translates a WPT test request into a NetworkStack API call
    /// and returns the result in WPT format.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use wpt_harness::{WptHarness, WptRequest};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let harness = WptHarness::new();
    /// let request = WptRequest {
    ///     method: "GET".to_string(),
    ///     url: "https://httpbin.org/get".to_string(),
    ///     headers: Default::default(),
    ///     body: None,
    ///     timeout_ms: Some(30000),
    /// };
    ///
    /// let response = harness.execute_request(request).await?;
    /// assert_eq!(response.status, 200);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn execute_request(
        &self,
        request: WptRequest,
    ) -> Result<WptResponse, Box<dyn std::error::Error>> {
        use network_types::{
            HttpMethod, NetworkRequest, RequestMode, CredentialsMode,
            CacheMode, RedirectMode, ReferrerPolicy, RequestPriority, RequestBody,
        };
        use http::HeaderMap;

        if self.verbose {
            eprintln!("[WPT] Executing: {} {}", request.method, request.url);
        }

        let start = std::time::Instant::now();

        // 1. Parse URL
        let url = url::Url::parse(&request.url)?;

        // 2. Convert HTTP method
        let method = match request.method.to_uppercase().as_str() {
            "GET" => HttpMethod::Get,
            "POST" => HttpMethod::Post,
            "PUT" => HttpMethod::Put,
            "DELETE" => HttpMethod::Delete,
            "HEAD" => HttpMethod::Head,
            "OPTIONS" => HttpMethod::Options,
            "PATCH" => HttpMethod::Patch,
            _ => HttpMethod::Get,
        };

        // 3. Convert headers
        let mut headers = HeaderMap::new();
        for (key, value) in request.headers {
            if let Ok(header_name) = http::header::HeaderName::from_bytes(key.as_bytes()) {
                if let Ok(header_value) = http::header::HeaderValue::from_str(&value) {
                    headers.insert(header_name, header_value);
                }
            }
        }

        // 4. Convert body
        let body = request.body.map(|b| RequestBody::Bytes(b));

        // 5. Create NetworkRequest
        let network_request = NetworkRequest {
            url,
            method,
            headers,
            body,
            mode: RequestMode::Cors,  // Default to CORS mode
            credentials: CredentialsMode::SameOrigin,
            cache: CacheMode::Default,
            redirect: RedirectMode::Follow,
            referrer: None,
            referrer_policy: ReferrerPolicy::NoReferrerWhenDowngrade,
            integrity: None,
            keepalive: false,
            signal: None,
            priority: RequestPriority::Auto,
            window: None,  // No associated window for WPT tests
        };

        // 6. Execute using HTTP/1.1 protocol directly (simplified approach)
        // For v0.2.0, we'll use the HTTP client directly rather than full NetworkStack
        // This allows testing HTTP functionality without full browser integration
        let network_response = self.execute_http_request(network_request).await?;

        // 7. Convert NetworkResponse to WptResponse
        let response = WptResponse {
            status: network_response.status,
            headers: network_response
                .headers
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
                .collect(),
            body: match network_response.body {
                network_types::ResponseBody::Bytes(bytes) => bytes,
                network_types::ResponseBody::Empty => Vec::new(),
                _ => Vec::new(), // Simplified - full implementation would read stream
            },
            duration_ms: start.elapsed().as_millis() as u64,
        };

        if self.verbose {
            eprintln!(
                "[WPT] Response: {} ({}ms)",
                response.status, response.duration_ms
            );
        }

        Ok(response)
    }

    /// Execute HTTP request using HTTP/1.1 client
    ///
    /// This is a simplified implementation that uses the HTTP/1.1 protocol directly.
    /// A full implementation would use NetworkStack::fetch() for protocol selection.
    async fn execute_http_request(
        &self,
        request: network_types::NetworkRequest,
    ) -> Result<network_types::NetworkResponse, Box<dyn std::error::Error>> {
        use http1_protocol::{Http1Client, Http1Config};

        // Create HTTP/1.1 client with default config
        let config = Http1Config {
            pool_size: 20,
            idle_timeout: std::time::Duration::from_secs(90),
            max_connections_per_host: 6,
            enable_keepalive: true,
            enable_pipelining: false,
        };
        let client = Http1Client::new(config);

        // Execute request
        let response = client.fetch(request).await.map_err(|e| {
            Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("HTTP request failed: {}", e),
            )) as Box<dyn std::error::Error>
        })?;

        Ok(response)
    }

    /// Run a WPT test and return the result
    pub async fn run_test(
        &self,
        _test_name: &str,
        request: WptRequest,
    ) -> WptTestResult {
        match self.execute_request(request).await {
            Ok(response) => {
                if response.status >= 200 && response.status < 300 {
                    WptTestResult::Pass
                } else {
                    WptTestResult::Fail {
                        reason: format!("Unexpected status code: {}", response.status),
                    }
                }
            }
            Err(e) => WptTestResult::Error {
                message: e.to_string(),
            },
        }
    }
}

impl Default for WptHarness {
    fn default() -> Self {
        Self::new()
    }
}

/// WPT test statistics
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct WptTestStats {
    /// Total tests executed
    pub total: usize,
    /// Tests passed
    pub passed: usize,
    /// Tests failed
    pub failed: usize,
    /// Tests timed out
    pub timeout: usize,
    /// Tests skipped
    pub skipped: usize,
    /// Tests with errors
    pub errors: usize,
}

impl WptTestStats {
    /// Add a test result to statistics
    pub fn add_result(&mut self, result: &WptTestResult) {
        self.total += 1;
        match result {
            WptTestResult::Pass => self.passed += 1,
            WptTestResult::Fail { .. } => self.failed += 1,
            WptTestResult::Timeout => self.timeout += 1,
            WptTestResult::Skip { .. } => self.skipped += 1,
            WptTestResult::Error { .. } => self.errors += 1,
        }
    }

    /// Calculate pass rate as percentage
    pub fn pass_rate(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            (self.passed as f64 / self.total as f64) * 100.0
        }
    }

    /// Print summary
    pub fn print_summary(&self) {
        println!("\nWPT Test Results:");
        println!("  Total:    {}", self.total);
        println!("  Passed:   {} ({}%)", self.passed, self.pass_rate());
        println!("  Failed:   {}", self.failed);
        println!("  Timeout:  {}", self.timeout);
        println!("  Skipped:  {}", self.skipped);
        println!("  Errors:   {}", self.errors);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_wpt_harness_creation() {
        let harness = WptHarness::new();
        assert!(!harness.verbose);
    }

    #[tokio::test]
    async fn test_wpt_harness_verbose() {
        let harness = WptHarness::new().with_verbose(true);
        assert!(harness.verbose);
    }

    #[tokio::test]
    async fn test_execute_request() {
        let harness = WptHarness::new();
        let request = WptRequest {
            method: "GET".to_string(),
            url: "https://example.com".to_string(),
            headers: Default::default(),
            body: None,
            timeout_ms: Some(30000),
        };

        let response = harness.execute_request(request).await.unwrap();
        assert_eq!(response.status, 200);
    }

    #[tokio::test]
    async fn test_run_test() {
        let harness = WptHarness::new();
        let request = WptRequest {
            method: "GET".to_string(),
            url: "https://example.com".to_string(),
            headers: Default::default(),
            body: None,
            timeout_ms: Some(30000),
        };

        let result = harness.run_test("sample_test", request).await;
        matches!(result, WptTestResult::Pass);
    }

    #[test]
    fn test_stats_pass_rate() {
        let mut stats = WptTestStats::default();
        stats.total = 100;
        stats.passed = 90;
        assert_eq!(stats.pass_rate(), 90.0);
    }

    #[test]
    fn test_stats_add_result() {
        let mut stats = WptTestStats::default();
        stats.add_result(&WptTestResult::Pass);
        stats.add_result(&WptTestResult::Fail {
            reason: "test".to_string(),
        });
        assert_eq!(stats.total, 2);
        assert_eq!(stats.passed, 1);
        assert_eq!(stats.failed, 1);
    }
}
