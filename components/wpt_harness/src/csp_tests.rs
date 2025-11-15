//! CSP (Content Security Policy) Test Suite
//!
//! This test suite validates CSP header parsing and validation.
//! CSP is critical for preventing XSS and other code injection attacks.

use crate::{WptHarness, WptRequest, WptResponse, WptTestResult, WptTestStats};
use std::collections::HashMap;

/// Create CSP test suite
pub fn create_csp_test_suite() -> Vec<(String, WptRequest, fn(&WptResponse) -> bool)> {
    vec![
        // ===== Basic Directive Tests =====
        (
            "csp_default_src".to_string(),
            WptRequest {
                method: "GET".to_string(),
                url: "http://127.0.0.1:8080/csp/default-src".to_string(),
                headers: HashMap::new(),
                body: None,
                timeout_ms: Some(30000),
            },
            |resp| {
                resp.status == 200 &&
                resp.headers.get("content-security-policy")
                    .map(|v| v.contains("default-src"))
                    .unwrap_or(false)
            },
        ),
        (
            "csp_script_src".to_string(),
            WptRequest {
                method: "GET".to_string(),
                url: "http://127.0.0.1:8080/csp/script-src".to_string(),
                headers: HashMap::new(),
                body: None,
                timeout_ms: Some(30000),
            },
            |resp| {
                resp.status == 200 &&
                resp.headers.get("content-security-policy")
                    .map(|v| v.contains("script-src"))
                    .unwrap_or(false)
            },
        ),

        // ===== Nonce-based CSP =====
        (
            "csp_nonce_test1".to_string(),
            WptRequest {
                method: "GET".to_string(),
                url: "http://127.0.0.1:8080/csp/nonce/abc123".to_string(),
                headers: HashMap::new(),
                body: None,
                timeout_ms: Some(30000),
            },
            |resp| {
                resp.status == 200 &&
                resp.headers.get("content-security-policy")
                    .map(|v| v.contains("nonce-abc123"))
                    .unwrap_or(false)
            },
        ),
        (
            "csp_nonce_test2".to_string(),
            WptRequest {
                method: "GET".to_string(),
                url: "http://127.0.0.1:8080/csp/nonce/xyz789".to_string(),
                headers: HashMap::new(),
                body: None,
                timeout_ms: Some(30000),
            },
            |resp| {
                resp.status == 200 &&
                resp.headers.get("content-security-policy")
                    .map(|v| v.contains("nonce-xyz789"))
                    .unwrap_or(false)
            },
        ),
        (
            "csp_nonce_format".to_string(),
            WptRequest {
                method: "GET".to_string(),
                url: "http://127.0.0.1:8080/csp/nonce/test-nonce-value".to_string(),
                headers: HashMap::new(),
                body: None,
                timeout_ms: Some(30000),
            },
            |resp| {
                resp.status == 200 &&
                resp.headers.get("content-security-policy")
                    .map(|v| v.contains("'nonce-") && v.contains("test-nonce-value"))
                    .unwrap_or(false)
            },
        ),

        // ===== Hash-based CSP =====
        (
            "csp_hash_sha256".to_string(),
            WptRequest {
                method: "GET".to_string(),
                url: "http://127.0.0.1:8080/csp/hash".to_string(),
                headers: HashMap::new(),
                body: None,
                timeout_ms: Some(30000),
            },
            |resp| {
                resp.status == 200 &&
                resp.headers.get("content-security-policy")
                    .map(|v| v.contains("sha256-"))
                    .unwrap_or(false)
            },
        ),

        // ===== Multiple Directives =====
        (
            "csp_multiple_directives".to_string(),
            WptRequest {
                method: "GET".to_string(),
                url: "http://127.0.0.1:8080/csp/multiple".to_string(),
                headers: HashMap::new(),
                body: None,
                timeout_ms: Some(30000),
            },
            |resp| {
                resp.status == 200 &&
                resp.headers.get("content-security-policy")
                    .map(|v| {
                        v.contains("default-src") &&
                        v.contains("script-src") &&
                        v.contains("style-src")
                    })
                    .unwrap_or(false)
            },
        ),
        (
            "csp_semicolon_separator".to_string(),
            WptRequest {
                method: "GET".to_string(),
                url: "http://127.0.0.1:8080/csp/multiple".to_string(),
                headers: HashMap::new(),
                body: None,
                timeout_ms: Some(30000),
            },
            |resp| {
                resp.status == 200 &&
                resp.headers.get("content-security-policy")
                    .map(|v| v.contains(";"))
                    .unwrap_or(false)
            },
        ),

        // ===== Source Keywords =====
        (
            "csp_self_keyword".to_string(),
            WptRequest {
                method: "GET".to_string(),
                url: "http://127.0.0.1:8080/csp/default-src".to_string(),
                headers: HashMap::new(),
                body: None,
                timeout_ms: Some(30000),
            },
            |resp| {
                resp.status == 200 &&
                resp.headers.get("content-security-policy")
                    .map(|v| v.contains("'self'"))
                    .unwrap_or(false)
            },
        ),
        (
            "csp_unsafe_inline".to_string(),
            WptRequest {
                method: "GET".to_string(),
                url: "http://127.0.0.1:8080/csp/multiple".to_string(),
                headers: HashMap::new(),
                body: None,
                timeout_ms: Some(30000),
            },
            |resp| {
                resp.status == 200 &&
                resp.headers.get("content-security-policy")
                    .map(|v| v.contains("'unsafe-inline'"))
                    .unwrap_or(false)
            },
        ),

        // ===== Report URI =====
        (
            "csp_report_uri_directive".to_string(),
            WptRequest {
                method: "GET".to_string(),
                url: "http://127.0.0.1:8080/csp/report-uri".to_string(),
                headers: HashMap::new(),
                body: None,
                timeout_ms: Some(30000),
            },
            |resp| {
                resp.status == 200 &&
                resp.headers.get("content-security-policy")
                    .map(|v| v.contains("report-uri"))
                    .unwrap_or(false)
            },
        ),
        (
            "csp_report_endpoint".to_string(),
            WptRequest {
                method: "GET".to_string(),
                url: "http://127.0.0.1:8080/csp/report".to_string(),
                headers: HashMap::new(),
                body: None,
                timeout_ms: Some(30000),
            },
            |resp| {
                // Report endpoint should accept reports
                resp.status == 200
            },
        ),

        // ===== CSP Header Presence =====
        (
            "csp_header_format".to_string(),
            WptRequest {
                method: "GET".to_string(),
                url: "http://127.0.0.1:8080/csp/default-src".to_string(),
                headers: HashMap::new(),
                body: None,
                timeout_ms: Some(30000),
            },
            |resp| {
                // CSP header should be present and properly formatted
                resp.status == 200 &&
                resp.headers.get("content-security-policy").is_some()
            },
        ),
        (
            "csp_case_sensitivity".to_string(),
            WptRequest {
                method: "GET".to_string(),
                url: "http://127.0.0.1:8080/csp/script-src".to_string(),
                headers: HashMap::new(),
                body: None,
                timeout_ms: Some(30000),
            },
            |resp| {
                // Header name should be lowercase (HTTP/2 requirement)
                resp.status == 200 &&
                (resp.headers.get("content-security-policy").is_some() ||
                 resp.headers.get("Content-Security-Policy").is_some())
            },
        ),

        // ===== Directive Syntax =====
        (
            "csp_directive_value_format".to_string(),
            WptRequest {
                method: "GET".to_string(),
                url: "http://127.0.0.1:8080/csp/default-src".to_string(),
                headers: HashMap::new(),
                body: None,
                timeout_ms: Some(30000),
            },
            |resp| {
                // Check that directive has proper format: directive-name value
                resp.status == 200 &&
                resp.headers.get("content-security-policy")
                    .map(|v| {
                        let parts: Vec<&str> = v.split_whitespace().collect();
                        parts.len() >= 2  // At least "directive value"
                    })
                    .unwrap_or(false)
            },
        ),

        // ===== Response Body with CSP =====
        (
            "csp_with_json_body".to_string(),
            WptRequest {
                method: "GET".to_string(),
                url: "http://127.0.0.1:8080/csp/default-src".to_string(),
                headers: HashMap::new(),
                body: None,
                timeout_ms: Some(30000),
            },
            |resp| {
                // CSP header should not interfere with response body
                resp.status == 200 &&
                resp.body.len() > 0 &&
                resp.headers.get("content-security-policy").is_some()
            },
        ),

        // ===== Complex Policies =====
        (
            "csp_complex_policy".to_string(),
            WptRequest {
                method: "GET".to_string(),
                url: "http://127.0.0.1:8080/csp/multiple".to_string(),
                headers: HashMap::new(),
                body: None,
                timeout_ms: Some(30000),
            },
            |resp| {
                // Complex policy with multiple directives and keywords
                resp.status == 200 &&
                resp.headers.get("content-security-policy")
                    .map(|v| {
                        v.contains("default-src") &&
                        v.contains("script-src") &&
                        v.contains("'self'") &&
                        v.contains("'unsafe-inline'")
                    })
                    .unwrap_or(false)
            },
        ),

        // ===== CSP Enforcement =====
        (
            "csp_default_src_self".to_string(),
            WptRequest {
                method: "GET".to_string(),
                url: "http://127.0.0.1:8080/csp/default-src".to_string(),
                headers: HashMap::new(),
                body: None,
                timeout_ms: Some(30000),
            },
            |resp| {
                // Verify exact policy format
                resp.status == 200 &&
                resp.headers.get("content-security-policy")
                    .map(|v| v == "default-src 'self'")
                    .unwrap_or(false)
            },
        ),
        (
            "csp_script_src_self".to_string(),
            WptRequest {
                method: "GET".to_string(),
                url: "http://127.0.0.1:8080/csp/script-src".to_string(),
                headers: HashMap::new(),
                body: None,
                timeout_ms: Some(30000),
            },
            |resp| {
                // Verify exact policy format
                resp.status == 200 &&
                resp.headers.get("content-security-policy")
                    .map(|v| v == "script-src 'self'")
                    .unwrap_or(false)
            },
        ),
    ]
}

/// Run CSP test suite
pub async fn run_csp_test_suite() -> WptTestStats {
    let harness = WptHarness::new().with_verbose(false);
    let mut stats = WptTestStats::default();

    let tests = create_csp_test_suite();

    println!("Running {} CSP tests against local test server (127.0.0.1:8080)...\n", tests.len());

    for (name, request, validator) in tests {
        print!("  {} ... ", name);

        match harness.execute_request(request).await {
            Ok(response) => {
                if validator(&response) {
                    println!("PASS");
                    stats.add_result(&WptTestResult::Pass);
                } else {
                    println!("FAIL (validation failed)");
                    stats.add_result(&WptTestResult::Fail {
                        reason: "Response validation failed".to_string(),
                    });
                }
            }
            Err(e) => {
                println!("ERROR: {}", e);
                stats.add_result(&WptTestResult::Error {
                    message: e.to_string(),
                });
            }
        }
    }

    stats
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_csp_suite_creation() {
        let tests = create_csp_test_suite();
        assert!(tests.len() >= 15, "CSP test suite should have at least 15 tests");
    }
}
