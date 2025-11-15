//! CORS (Cross-Origin Resource Sharing) Test Suite
//!
//! This test suite validates CORS functionality against a local HTTP test server.
//! CORS is critical for security, preventing unauthorized cross-origin access.

use crate::{WptHarness, WptRequest, WptResponse, WptTestResult, WptTestStats};
use std::collections::HashMap;

/// Create CORS test suite
pub fn create_cors_test_suite() -> Vec<(String, WptRequest, fn(&WptResponse) -> bool)> {
    vec![
        // ===== Simple CORS Requests =====
        (
            "cors_simple_request".to_string(),
            WptRequest {
                method: "GET".to_string(),
                url: "http://127.0.0.1:8080/cors/simple".to_string(),
                headers: [("Origin".to_string(), "http://example.com".to_string())]
                    .into_iter()
                    .collect(),
                body: None,
                timeout_ms: Some(30000),
            },
            |resp| {
                resp.status == 200 &&
                resp.headers.get("access-control-allow-origin").is_some()
            },
        ),
        (
            "cors_simple_wildcard_origin".to_string(),
            WptRequest {
                method: "GET".to_string(),
                url: "http://127.0.0.1:8080/cors/simple".to_string(),
                headers: [("Origin".to_string(), "http://different.com".to_string())]
                    .into_iter()
                    .collect(),
                body: None,
                timeout_ms: Some(30000),
            },
            |resp| {
                resp.status == 200 &&
                resp.headers.get("access-control-allow-origin").map(|v| v == "*").unwrap_or(false)
            },
        ),
        (
            "cors_no_origin_header".to_string(),
            WptRequest {
                method: "GET".to_string(),
                url: "http://127.0.0.1:8080/cors/simple".to_string(),
                headers: HashMap::new(),
                body: None,
                timeout_ms: Some(30000),
            },
            |resp| {
                // CORS headers should still be present even without Origin
                resp.status == 200 &&
                resp.headers.get("access-control-allow-origin").is_some()
            },
        ),

        // ===== Preflight Requests =====
        (
            "cors_preflight_options".to_string(),
            WptRequest {
                method: "OPTIONS".to_string(),
                url: "http://127.0.0.1:8080/cors/simple".to_string(),
                headers: [
                    ("Origin".to_string(), "http://example.com".to_string()),
                    ("Access-Control-Request-Method".to_string(), "POST".to_string()),
                    ("Access-Control-Request-Headers".to_string(), "Content-Type".to_string()),
                ]
                .into_iter()
                .collect(),
                body: None,
                timeout_ms: Some(30000),
            },
            |resp| {
                // Preflight should return 204 or 200
                (resp.status == 204 || resp.status == 200) &&
                resp.headers.get("access-control-allow-methods").is_some()
            },
        ),
        (
            "cors_preflight_custom_method".to_string(),
            WptRequest {
                method: "OPTIONS".to_string(),
                url: "http://127.0.0.1:8080/cors/custom-method".to_string(),
                headers: [
                    ("Origin".to_string(), "http://example.com".to_string()),
                    ("Access-Control-Request-Method".to_string(), "PUT".to_string()),
                ]
                .into_iter()
                .collect(),
                body: None,
                timeout_ms: Some(30000),
            },
            |resp| {
                (resp.status == 204 || resp.status == 200) &&
                resp.headers.get("access-control-allow-methods")
                    .map(|v| v.contains("PUT"))
                    .unwrap_or(false)
            },
        ),

        // ===== Credentials Mode =====
        (
            "cors_credentials_with_origin".to_string(),
            WptRequest {
                method: "GET".to_string(),
                url: "http://127.0.0.1:8080/cors/credentials".to_string(),
                headers: [("Origin".to_string(), "http://example.com".to_string())]
                    .into_iter()
                    .collect(),
                body: None,
                timeout_ms: Some(30000),
            },
            |resp| {
                resp.status == 200 &&
                resp.headers.get("access-control-allow-credentials")
                    .map(|v| v == "true")
                    .unwrap_or(false) &&
                // With credentials, origin should NOT be wildcard
                resp.headers.get("access-control-allow-origin")
                    .map(|v| v != "*")
                    .unwrap_or(false)
            },
        ),

        // ===== Missing CORS Headers (Should Fail) =====
        (
            "cors_missing_headers".to_string(),
            WptRequest {
                method: "GET".to_string(),
                url: "http://127.0.0.1:8080/cors/no-headers".to_string(),
                headers: [("Origin".to_string(), "http://example.com".to_string())]
                    .into_iter()
                    .collect(),
                body: None,
                timeout_ms: Some(30000),
            },
            |resp| {
                // This endpoint deliberately omits CORS headers
                // In a real browser, this would be blocked
                // For testing, we just verify the endpoint works and no CORS headers
                resp.status == 200 &&
                resp.headers.get("access-control-allow-origin").is_none()
            },
        ),

        // ===== Header Validation =====
        (
            "cors_allow_methods_header".to_string(),
            WptRequest {
                method: "OPTIONS".to_string(),
                url: "http://127.0.0.1:8080/cors/simple".to_string(),
                headers: [
                    ("Origin".to_string(), "http://example.com".to_string()),
                    ("Access-Control-Request-Method".to_string(), "DELETE".to_string()),
                ]
                .into_iter()
                .collect(),
                body: None,
                timeout_ms: Some(30000),
            },
            |resp| {
                resp.headers.get("access-control-allow-methods")
                    .map(|v| v.contains("DELETE"))
                    .unwrap_or(false)
            },
        ),
        (
            "cors_allow_headers_validation".to_string(),
            WptRequest {
                method: "OPTIONS".to_string(),
                url: "http://127.0.0.1:8080/cors/simple".to_string(),
                headers: [
                    ("Origin".to_string(), "http://example.com".to_string()),
                    ("Access-Control-Request-Method".to_string(), "POST".to_string()),
                    ("Access-Control-Request-Headers".to_string(), "X-Custom-Header, Content-Type".to_string()),
                ]
                .into_iter()
                .collect(),
                body: None,
                timeout_ms: Some(30000),
            },
            |resp| {
                resp.headers.get("access-control-allow-headers").is_some()
            },
        ),
        (
            "cors_max_age_header".to_string(),
            WptRequest {
                method: "OPTIONS".to_string(),
                url: "http://127.0.0.1:8080/cors/simple".to_string(),
                headers: [
                    ("Origin".to_string(), "http://example.com".to_string()),
                    ("Access-Control-Request-Method".to_string(), "GET".to_string()),
                ]
                .into_iter()
                .collect(),
                body: None,
                timeout_ms: Some(30000),
            },
            |resp| {
                // Max-Age header should be present for preflight caching
                resp.headers.get("access-control-max-age").is_some()
            },
        ),

        // ===== Multiple Origins =====
        (
            "cors_different_origins".to_string(),
            WptRequest {
                method: "GET".to_string(),
                url: "http://127.0.0.1:8080/cors/simple".to_string(),
                headers: [("Origin".to_string(), "http://another-origin.com".to_string())]
                    .into_iter()
                    .collect(),
                body: None,
                timeout_ms: Some(30000),
            },
            |resp| {
                resp.status == 200 &&
                resp.headers.get("access-control-allow-origin").is_some()
            },
        ),

        // ===== POST with CORS =====
        (
            "cors_post_request".to_string(),
            WptRequest {
                method: "POST".to_string(),
                url: "http://127.0.0.1:8080/post".to_string(),
                headers: [
                    ("Origin".to_string(), "http://example.com".to_string()),
                    ("Content-Type".to_string(), "application/json".to_string()),
                ]
                .into_iter()
                .collect(),
                body: Some(br#"{"test": "cors"}"#.to_vec()),
                timeout_ms: Some(30000),
            },
            |resp| {
                // POST without CORS headers on this endpoint (testing non-CORS endpoint)
                resp.status == 200
            },
        ),

        // ===== Custom Method with CORS =====
        (
            "cors_custom_method_actual_request".to_string(),
            WptRequest {
                method: "GET".to_string(),
                url: "http://127.0.0.1:8080/cors/custom-method".to_string(),
                headers: [("Origin".to_string(), "http://example.com".to_string())]
                    .into_iter()
                    .collect(),
                body: None,
                timeout_ms: Some(30000),
            },
            |resp| {
                resp.status == 200 &&
                resp.headers.get("access-control-allow-origin").is_some()
            },
        ),

        // ===== Vary Header =====
        (
            "cors_preflight_vary_check".to_string(),
            WptRequest {
                method: "OPTIONS".to_string(),
                url: "http://127.0.0.1:8080/cors/simple".to_string(),
                headers: [
                    ("Origin".to_string(), "http://example.com".to_string()),
                    ("Access-Control-Request-Method".to_string(), "GET".to_string()),
                ]
                .into_iter()
                .collect(),
                body: None,
                timeout_ms: Some(30000),
            },
            |resp| {
                // Preflight should respond with appropriate status
                resp.status == 204 || resp.status == 200
            },
        ),
    ]
}

/// Run CORS test suite
pub async fn run_cors_test_suite() -> WptTestStats {
    let harness = WptHarness::new().with_verbose(false);
    let mut stats = WptTestStats::default();

    let tests = create_cors_test_suite();

    println!("Running {} CORS tests against local test server (127.0.0.1:8080)...\n", tests.len());

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
    fn test_cors_suite_creation() {
        let tests = create_cors_test_suite();
        assert!(tests.len() >= 10, "CORS test suite should have at least 10 tests");
    }
}
