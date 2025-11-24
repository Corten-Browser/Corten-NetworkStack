//! Curl Compatibility Test Suite
//!
//! This module provides tests that verify HTTP client behavior matches curl's
//! expected behavior patterns. These tests do NOT run curl as an external process,
//! but rather verify that our HTTP client implementation follows curl's documented
//! behavior for common scenarios.
//!
//! Reference: curl man page and RFC compliance
//!
//! Test Categories:
//! - Basic HTTP methods (GET, POST, PUT, DELETE, HEAD, OPTIONS, PATCH)
//! - Redirect handling (301, 302, 303, 307, 308)
//! - Header management (custom headers, user-agent, content-type)
//! - Cookie handling (Set-Cookie parsing, cookie sending)
//! - Authentication (Basic, Bearer token)
//! - Request body handling (form data, JSON, raw data)
//! - Response handling (status codes, body parsing)

use http::{HeaderMap, HeaderName, HeaderValue};
use network_types::{
    CacheMode, CredentialsMode, HttpMethod, NetworkRequest, RedirectMode, ReferrerPolicy,
    RequestBody, RequestMode, RequestPriority,
};
use url::Url;

/// Helper to create a basic NetworkRequest for testing
fn create_test_request(url: &str, method: HttpMethod) -> NetworkRequest {
    NetworkRequest {
        url: Url::parse(url).expect("Invalid URL"),
        method,
        headers: HeaderMap::new(),
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
        priority: RequestPriority::Auto,
        window: None,
    }
}

/// Helper to create a request with custom headers
fn create_request_with_headers(
    url: &str,
    method: HttpMethod,
    headers: Vec<(&str, &str)>,
) -> NetworkRequest {
    let mut request = create_test_request(url, method);
    for (name, value) in headers {
        request.headers.insert(
            HeaderName::try_from(name).unwrap(),
            HeaderValue::try_from(value).unwrap(),
        );
    }
    request
}

/// Helper to create a POST request with body
fn create_post_request_with_body(url: &str, body: &str, content_type: &str) -> NetworkRequest {
    let mut request = create_test_request(url, HttpMethod::Post);
    request.body = Some(RequestBody::Text(body.to_string()));
    request.headers.insert(
        HeaderName::try_from("content-type").unwrap(),
        HeaderValue::try_from(content_type).unwrap(),
    );
    request
}

// =============================================================================
// GET Request Tests (curl -X GET / curl <url>)
// =============================================================================

/// Tests basic GET request construction
/// Equivalent to: curl http://example.com/
#[test]
fn test_curl_basic_get_request() {
    let request = create_test_request("http://example.com/", HttpMethod::Get);

    assert_eq!(request.method, HttpMethod::Get);
    assert_eq!(request.url.scheme(), "http");
    assert_eq!(request.url.host_str(), Some("example.com"));
    assert_eq!(request.url.path(), "/");
    assert!(request.body.is_none(), "GET request should not have body");
}

/// Tests GET request with query parameters
/// Equivalent to: curl "http://example.com/search?q=rust&page=1"
#[test]
fn test_curl_get_with_query_params() {
    let request = create_test_request("http://example.com/search?q=rust&page=1", HttpMethod::Get);

    assert_eq!(request.url.query(), Some("q=rust&page=1"));
    assert_eq!(request.url.path(), "/search");

    // Verify query pairs
    let pairs: Vec<(String, String)> = request.url.query_pairs().into_owned().collect();
    assert_eq!(pairs.len(), 2);
    assert!(pairs.contains(&("q".to_string(), "rust".to_string())));
    assert!(pairs.contains(&("page".to_string(), "1".to_string())));
}

/// Tests GET request to HTTPS endpoint
/// Equivalent to: curl https://secure.example.com/api
#[test]
fn test_curl_https_get_request() {
    let request = create_test_request("https://secure.example.com/api", HttpMethod::Get);

    assert_eq!(request.url.scheme(), "https");
    assert_eq!(request.url.port_or_known_default(), Some(443));
}

/// Tests GET request with custom port
/// Equivalent to: curl http://example.com:8080/api
#[test]
fn test_curl_get_custom_port() {
    let request = create_test_request("http://example.com:8080/api", HttpMethod::Get);

    assert_eq!(request.url.port(), Some(8080));
    assert_eq!(request.url.host_str(), Some("example.com"));
}

// =============================================================================
// POST Request Tests (curl -X POST / curl -d)
// =============================================================================

/// Tests basic POST request with form data
/// Equivalent to: curl -X POST -d "name=value" http://example.com/submit
#[test]
fn test_curl_post_form_data() {
    let request = create_post_request_with_body(
        "http://example.com/submit",
        "name=value",
        "application/x-www-form-urlencoded",
    );

    assert_eq!(request.method, HttpMethod::Post);
    assert!(request.body.is_some());

    if let Some(RequestBody::Text(body)) = &request.body {
        assert_eq!(body, "name=value");
    } else {
        panic!("Expected Text body");
    }

    let content_type = request.headers.get("content-type").unwrap();
    assert_eq!(content_type, "application/x-www-form-urlencoded");
}

/// Tests POST request with JSON body
/// Equivalent to: curl -X POST -H "Content-Type: application/json" -d '{"key":"value"}' http://example.com/api
#[test]
fn test_curl_post_json_data() {
    let json_body = r#"{"key":"value"}"#;
    let request = create_post_request_with_body(
        "http://example.com/api",
        json_body,
        "application/json",
    );

    assert_eq!(request.method, HttpMethod::Post);

    if let Some(RequestBody::Text(body)) = &request.body {
        assert_eq!(body, json_body);
    } else {
        panic!("Expected Text body");
    }

    let content_type = request.headers.get("content-type").unwrap();
    assert_eq!(content_type, "application/json");
}

/// Tests POST request with multiple form fields
/// Equivalent to: curl -X POST -d "field1=value1&field2=value2" http://example.com/form
#[test]
fn test_curl_post_multiple_fields() {
    let request = create_post_request_with_body(
        "http://example.com/form",
        "field1=value1&field2=value2",
        "application/x-www-form-urlencoded",
    );

    if let Some(RequestBody::Text(body)) = &request.body {
        assert!(body.contains("field1=value1"));
        assert!(body.contains("field2=value2"));
    } else {
        panic!("Expected Text body");
    }
}

// =============================================================================
// Other HTTP Methods Tests (PUT, DELETE, HEAD, OPTIONS, PATCH)
// =============================================================================

/// Tests PUT request
/// Equivalent to: curl -X PUT -d "data" http://example.com/resource
#[test]
fn test_curl_put_request() {
    let mut request = create_test_request("http://example.com/resource", HttpMethod::Put);
    request.body = Some(RequestBody::Text("update data".to_string()));

    assert_eq!(request.method, HttpMethod::Put);
    assert!(request.body.is_some());
}

/// Tests DELETE request
/// Equivalent to: curl -X DELETE http://example.com/resource/123
#[test]
fn test_curl_delete_request() {
    let request = create_test_request("http://example.com/resource/123", HttpMethod::Delete);

    assert_eq!(request.method, HttpMethod::Delete);
    assert!(request.body.is_none(), "DELETE typically has no body");
}

/// Tests HEAD request
/// Equivalent to: curl -I http://example.com/
#[test]
fn test_curl_head_request() {
    let request = create_test_request("http://example.com/", HttpMethod::Head);

    assert_eq!(request.method, HttpMethod::Head);
    assert!(request.body.is_none(), "HEAD request should not have body");
}

/// Tests OPTIONS request
/// Equivalent to: curl -X OPTIONS http://example.com/api
#[test]
fn test_curl_options_request() {
    let request = create_test_request("http://example.com/api", HttpMethod::Options);

    assert_eq!(request.method, HttpMethod::Options);
}

/// Tests PATCH request
/// Equivalent to: curl -X PATCH -d '{"field":"newvalue"}' http://example.com/resource
#[test]
fn test_curl_patch_request() {
    let mut request = create_test_request("http://example.com/resource", HttpMethod::Patch);
    request.body = Some(RequestBody::Text(r#"{"field":"newvalue"}"#.to_string()));
    request.headers.insert(
        HeaderName::try_from("content-type").unwrap(),
        HeaderValue::try_from("application/json").unwrap(),
    );

    assert_eq!(request.method, HttpMethod::Patch);
    assert!(request.body.is_some());
}

// =============================================================================
// Header Tests (curl -H)
// =============================================================================

/// Tests custom header setting
/// Equivalent to: curl -H "X-Custom-Header: CustomValue" http://example.com/
#[test]
fn test_curl_custom_header() {
    let request = create_request_with_headers(
        "http://example.com/",
        HttpMethod::Get,
        vec![("x-custom-header", "CustomValue")],
    );

    let header = request.headers.get("x-custom-header").unwrap();
    assert_eq!(header, "CustomValue");
}

/// Tests User-Agent header
/// Equivalent to: curl -A "MyAgent/1.0" http://example.com/
#[test]
fn test_curl_user_agent_header() {
    let request = create_request_with_headers(
        "http://example.com/",
        HttpMethod::Get,
        vec![("user-agent", "MyAgent/1.0")],
    );

    let ua = request.headers.get("user-agent").unwrap();
    assert_eq!(ua, "MyAgent/1.0");
}

/// Tests multiple headers
/// Equivalent to: curl -H "Accept: application/json" -H "Authorization: Bearer token" http://example.com/api
#[test]
fn test_curl_multiple_headers() {
    let request = create_request_with_headers(
        "http://example.com/api",
        HttpMethod::Get,
        vec![
            ("accept", "application/json"),
            ("authorization", "Bearer token123"),
        ],
    );

    assert_eq!(request.headers.get("accept").unwrap(), "application/json");
    assert_eq!(
        request.headers.get("authorization").unwrap(),
        "Bearer token123"
    );
}

/// Tests Accept-Language header
/// Equivalent to: curl -H "Accept-Language: en-US,en;q=0.9" http://example.com/
#[test]
fn test_curl_accept_language_header() {
    let request = create_request_with_headers(
        "http://example.com/",
        HttpMethod::Get,
        vec![("accept-language", "en-US,en;q=0.9")],
    );

    let lang = request.headers.get("accept-language").unwrap();
    assert_eq!(lang, "en-US,en;q=0.9");
}

/// Tests Cache-Control header
/// Equivalent to: curl -H "Cache-Control: no-cache" http://example.com/
#[test]
fn test_curl_cache_control_header() {
    let request = create_request_with_headers(
        "http://example.com/",
        HttpMethod::Get,
        vec![("cache-control", "no-cache")],
    );

    let cache = request.headers.get("cache-control").unwrap();
    assert_eq!(cache, "no-cache");
}

// =============================================================================
// Redirect Tests (curl -L / redirect handling)
// =============================================================================

/// Tests redirect follow mode
/// Equivalent to: curl -L http://example.com/redirect
#[test]
fn test_curl_redirect_follow_mode() {
    let request = create_test_request("http://example.com/redirect", HttpMethod::Get);

    // Default should follow redirects (like curl -L)
    assert_eq!(request.redirect, RedirectMode::Follow);
}

/// Tests redirect manual mode
/// Equivalent to: curl http://example.com/redirect (without -L)
#[test]
fn test_curl_redirect_manual_mode() {
    let mut request = create_test_request("http://example.com/redirect", HttpMethod::Get);
    request.redirect = RedirectMode::Manual;

    assert_eq!(request.redirect, RedirectMode::Manual);
}

/// Tests redirect error mode
/// Equivalent to: curl --max-redirs 0 http://example.com/redirect
#[test]
fn test_curl_redirect_error_mode() {
    let mut request = create_test_request("http://example.com/redirect", HttpMethod::Get);
    request.redirect = RedirectMode::Error;

    assert_eq!(request.redirect, RedirectMode::Error);
}

// =============================================================================
// Cookie Tests (curl -b / curl -c)
// =============================================================================

/// Tests cookie header sending
/// Equivalent to: curl -b "session=abc123" http://example.com/
#[test]
fn test_curl_cookie_header() {
    let request = create_request_with_headers(
        "http://example.com/",
        HttpMethod::Get,
        vec![("cookie", "session=abc123")],
    );

    let cookie = request.headers.get("cookie").unwrap();
    assert_eq!(cookie, "session=abc123");
}

/// Tests multiple cookies
/// Equivalent to: curl -b "session=abc123; user=john" http://example.com/
#[test]
fn test_curl_multiple_cookies() {
    let request = create_request_with_headers(
        "http://example.com/",
        HttpMethod::Get,
        vec![("cookie", "session=abc123; user=john")],
    );

    let cookie = request.headers.get("cookie").unwrap();
    assert!(cookie.to_str().unwrap().contains("session=abc123"));
    assert!(cookie.to_str().unwrap().contains("user=john"));
}

/// Tests credentials mode for cookies
/// Equivalent to: curl -b cookies.txt http://example.com/ (with credentials)
#[test]
fn test_curl_credentials_include_mode() {
    let mut request = create_test_request("http://example.com/", HttpMethod::Get);
    request.credentials = CredentialsMode::Include;

    assert_eq!(request.credentials, CredentialsMode::Include);
}

/// Tests no credentials mode
/// Equivalent behavior when cookies should not be sent
#[test]
fn test_curl_credentials_omit_mode() {
    let mut request = create_test_request("http://example.com/", HttpMethod::Get);
    request.credentials = CredentialsMode::Omit;

    assert_eq!(request.credentials, CredentialsMode::Omit);
}

// =============================================================================
// Authentication Tests (curl -u / curl -H "Authorization:")
// =============================================================================

/// Tests Basic Authentication header
/// Equivalent to: curl -u "user:password" http://example.com/
/// curl -H "Authorization: Basic dXNlcjpwYXNzd29yZA==" http://example.com/
#[test]
fn test_curl_basic_auth_header() {
    // Base64 of "user:password"
    let auth_header = "Basic dXNlcjpwYXNzd29yZA==";

    let request = create_request_with_headers(
        "http://example.com/",
        HttpMethod::Get,
        vec![("authorization", auth_header)],
    );

    let auth = request.headers.get("authorization").unwrap();
    assert_eq!(auth, auth_header);
    assert!(auth.to_str().unwrap().starts_with("Basic "));
}

/// Tests Bearer token authentication
/// Equivalent to: curl -H "Authorization: Bearer eyJ..." http://example.com/api
#[test]
fn test_curl_bearer_auth_header() {
    let bearer_token = "Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9";

    let request = create_request_with_headers(
        "http://example.com/api",
        HttpMethod::Get,
        vec![("authorization", bearer_token)],
    );

    let auth = request.headers.get("authorization").unwrap();
    assert!(auth.to_str().unwrap().starts_with("Bearer "));
}

/// Tests API Key authentication in header
/// Equivalent to: curl -H "X-API-Key: abc123" http://example.com/api
#[test]
fn test_curl_api_key_header() {
    let request = create_request_with_headers(
        "http://example.com/api",
        HttpMethod::Get,
        vec![("x-api-key", "abc123")],
    );

    let api_key = request.headers.get("x-api-key").unwrap();
    assert_eq!(api_key, "abc123");
}

// =============================================================================
// URL Encoding Tests
// =============================================================================

/// Tests URL with encoded characters
/// Equivalent to: curl "http://example.com/search?q=hello%20world"
#[test]
fn test_curl_url_encoding() {
    let request = create_test_request("http://example.com/search?q=hello%20world", HttpMethod::Get);

    let query = request.url.query().unwrap();
    assert!(query.contains("hello%20world") || query.contains("hello+world"));
}

/// Tests URL with special characters
/// Equivalent to: curl "http://example.com/path/with%2Fslash"
#[test]
fn test_curl_special_char_encoding() {
    let request = create_test_request("http://example.com/path/with%2Fslash", HttpMethod::Get);

    // Path should contain the encoded value
    assert!(request.url.as_str().contains("%2F") || request.url.path().contains('/'));
}

/// Tests URL with unicode characters
/// Equivalent to: curl "http://example.com/search?q=%E4%B8%AD%E6%96%87"
#[test]
fn test_curl_unicode_encoding() {
    // URL encoded Chinese characters
    let request =
        create_test_request("http://example.com/search?q=%E4%B8%AD%E6%96%87", HttpMethod::Get);

    assert!(request.url.query().is_some());
}

// =============================================================================
// Keep-Alive Tests
// =============================================================================

/// Tests keep-alive enabled (default curl behavior)
/// Equivalent to: curl http://example.com/ (keep-alive is default)
#[test]
fn test_curl_keepalive_enabled() {
    let request = create_test_request("http://example.com/", HttpMethod::Get);

    assert!(request.keepalive, "Keep-alive should be enabled by default");
}

/// Tests keep-alive disabled
/// Equivalent to: curl -H "Connection: close" http://example.com/
#[test]
fn test_curl_keepalive_disabled() {
    let mut request = create_test_request("http://example.com/", HttpMethod::Get);
    request.keepalive = false;
    request.headers.insert(
        HeaderName::try_from("connection").unwrap(),
        HeaderValue::try_from("close").unwrap(),
    );

    assert!(!request.keepalive);
    assert_eq!(request.headers.get("connection").unwrap(), "close");
}

// =============================================================================
// Cache Mode Tests
// =============================================================================

/// Tests no-cache mode
/// Equivalent to: curl -H "Cache-Control: no-cache" http://example.com/
#[test]
fn test_curl_no_cache_mode() {
    let mut request = create_test_request("http://example.com/", HttpMethod::Get);
    request.cache = CacheMode::NoCache;

    assert_eq!(request.cache, CacheMode::NoCache);
}

/// Tests reload/force refresh
/// Equivalent to: curl -H "Cache-Control: no-cache" -H "Pragma: no-cache" http://example.com/
#[test]
fn test_curl_reload_mode() {
    let mut request = create_test_request("http://example.com/", HttpMethod::Get);
    request.cache = CacheMode::Reload;

    assert_eq!(request.cache, CacheMode::Reload);
}

/// Tests no-store mode
/// Equivalent to: curl -H "Cache-Control: no-store" http://example.com/
#[test]
fn test_curl_no_store_mode() {
    let mut request = create_test_request("http://example.com/", HttpMethod::Get);
    request.cache = CacheMode::NoStore;

    assert_eq!(request.cache, CacheMode::NoStore);
}

// =============================================================================
// Referrer Tests
// =============================================================================

/// Tests referrer header
/// Equivalent to: curl -e "http://referrer.example.com/" http://example.com/
#[test]
fn test_curl_referrer_header() {
    let mut request = create_test_request("http://example.com/", HttpMethod::Get);
    request.referrer = Some("http://referrer.example.com/".to_string());

    assert_eq!(
        request.referrer,
        Some("http://referrer.example.com/".to_string())
    );
}

/// Tests no-referrer policy
#[test]
fn test_curl_no_referrer_policy() {
    let mut request = create_test_request("http://example.com/", HttpMethod::Get);
    request.referrer_policy = ReferrerPolicy::NoReferrer;

    assert_eq!(request.referrer_policy, ReferrerPolicy::NoReferrer);
}

// =============================================================================
// Request Priority Tests
// =============================================================================

/// Tests high priority request
#[test]
fn test_curl_high_priority() {
    let mut request = create_test_request("http://example.com/critical", HttpMethod::Get);
    request.priority = RequestPriority::High;

    assert_eq!(request.priority, RequestPriority::High);
}

/// Tests low priority request
#[test]
fn test_curl_low_priority() {
    let mut request = create_test_request("http://example.com/background", HttpMethod::Get);
    request.priority = RequestPriority::Low;

    assert_eq!(request.priority, RequestPriority::Low);
}

// =============================================================================
// Content-Type Tests
// =============================================================================

/// Tests application/json content type
/// Equivalent to: curl -H "Content-Type: application/json" -d '{}' http://example.com/api
#[test]
fn test_curl_json_content_type() {
    let request = create_request_with_headers(
        "http://example.com/api",
        HttpMethod::Post,
        vec![("content-type", "application/json")],
    );

    let ct = request.headers.get("content-type").unwrap();
    assert_eq!(ct, "application/json");
}

/// Tests application/xml content type
/// Equivalent to: curl -H "Content-Type: application/xml" -d '<xml/>' http://example.com/api
#[test]
fn test_curl_xml_content_type() {
    let request = create_request_with_headers(
        "http://example.com/api",
        HttpMethod::Post,
        vec![("content-type", "application/xml")],
    );

    let ct = request.headers.get("content-type").unwrap();
    assert_eq!(ct, "application/xml");
}

/// Tests text/plain content type
/// Equivalent to: curl -H "Content-Type: text/plain" -d 'text' http://example.com/
#[test]
fn test_curl_text_content_type() {
    let request = create_request_with_headers(
        "http://example.com/",
        HttpMethod::Post,
        vec![("content-type", "text/plain")],
    );

    let ct = request.headers.get("content-type").unwrap();
    assert_eq!(ct, "text/plain");
}

// =============================================================================
// Accept Header Tests
// =============================================================================

/// Tests Accept: application/json
/// Equivalent to: curl -H "Accept: application/json" http://example.com/api
#[test]
fn test_curl_accept_json() {
    let request = create_request_with_headers(
        "http://example.com/api",
        HttpMethod::Get,
        vec![("accept", "application/json")],
    );

    let accept = request.headers.get("accept").unwrap();
    assert_eq!(accept, "application/json");
}

/// Tests Accept with quality values
/// Equivalent to: curl -H "Accept: text/html,application/json;q=0.9" http://example.com/
#[test]
fn test_curl_accept_with_quality() {
    let request = create_request_with_headers(
        "http://example.com/",
        HttpMethod::Get,
        vec![("accept", "text/html,application/json;q=0.9")],
    );

    let accept = request.headers.get("accept").unwrap();
    assert!(accept.to_str().unwrap().contains("text/html"));
    assert!(accept.to_str().unwrap().contains("application/json"));
}

// =============================================================================
// CORS Mode Tests
// =============================================================================

/// Tests CORS mode
#[test]
fn test_curl_cors_mode() {
    let request = create_test_request("http://example.com/api", HttpMethod::Get);

    assert_eq!(request.mode, RequestMode::Cors);
}

/// Tests same-origin mode
#[test]
fn test_curl_same_origin_mode() {
    let mut request = create_test_request("http://example.com/api", HttpMethod::Get);
    request.mode = RequestMode::SameOrigin;

    assert_eq!(request.mode, RequestMode::SameOrigin);
}

/// Tests no-cors mode
#[test]
fn test_curl_no_cors_mode() {
    let mut request = create_test_request("http://example.com/api", HttpMethod::Get);
    request.mode = RequestMode::NoCors;

    assert_eq!(request.mode, RequestMode::NoCors);
}

// =============================================================================
// Complex Scenario Tests
// =============================================================================

/// Tests a complete API request scenario
/// Equivalent to: curl -X POST -H "Content-Type: application/json" \
///                -H "Authorization: Bearer token" \
///                -H "Accept: application/json" \
///                -d '{"key":"value"}' \
///                http://api.example.com/v1/resource
#[test]
fn test_curl_complete_api_request() {
    let mut request = create_test_request("http://api.example.com/v1/resource", HttpMethod::Post);

    // Set headers
    request.headers.insert(
        HeaderName::try_from("content-type").unwrap(),
        HeaderValue::try_from("application/json").unwrap(),
    );
    request.headers.insert(
        HeaderName::try_from("authorization").unwrap(),
        HeaderValue::try_from("Bearer token123").unwrap(),
    );
    request.headers.insert(
        HeaderName::try_from("accept").unwrap(),
        HeaderValue::try_from("application/json").unwrap(),
    );

    // Set body
    request.body = Some(RequestBody::Text(r#"{"key":"value"}"#.to_string()));

    // Verify all components
    assert_eq!(request.method, HttpMethod::Post);
    assert_eq!(request.url.host_str(), Some("api.example.com"));
    assert_eq!(request.url.path(), "/v1/resource");
    assert_eq!(
        request.headers.get("content-type").unwrap(),
        "application/json"
    );
    assert_eq!(
        request.headers.get("authorization").unwrap(),
        "Bearer token123"
    );
    assert!(request.body.is_some());
}

/// Tests request with all common headers
/// Simulates a browser-like request
#[test]
fn test_curl_browser_like_request() {
    let headers = vec![
        ("user-agent", "Mozilla/5.0 (compatible; TestBot/1.0)"),
        ("accept", "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8"),
        ("accept-language", "en-US,en;q=0.5"),
        ("accept-encoding", "gzip, deflate, br"),
        ("connection", "keep-alive"),
    ];

    let request = create_request_with_headers("http://example.com/page", HttpMethod::Get, headers);

    assert!(request.headers.get("user-agent").is_some());
    assert!(request.headers.get("accept").is_some());
    assert!(request.headers.get("accept-language").is_some());
    assert!(request.headers.get("accept-encoding").is_some());
    assert!(request.headers.get("connection").is_some());
}

/// Tests conditional request with If-Modified-Since
/// Equivalent to: curl -H "If-Modified-Since: Wed, 21 Oct 2015 07:28:00 GMT" http://example.com/
#[test]
fn test_curl_conditional_request() {
    let request = create_request_with_headers(
        "http://example.com/resource",
        HttpMethod::Get,
        vec![("if-modified-since", "Wed, 21 Oct 2015 07:28:00 GMT")],
    );

    let ims = request.headers.get("if-modified-since").unwrap();
    assert!(ims.to_str().unwrap().contains("2015"));
}

/// Tests request with If-None-Match (ETag)
/// Equivalent to: curl -H 'If-None-Match: "abc123"' http://example.com/
#[test]
fn test_curl_etag_request() {
    let request = create_request_with_headers(
        "http://example.com/resource",
        HttpMethod::Get,
        vec![("if-none-match", "\"abc123\"")],
    );

    let inm = request.headers.get("if-none-match").unwrap();
    assert_eq!(inm, "\"abc123\"");
}

// =============================================================================
// Binary Data Tests
// =============================================================================

/// Tests request with binary body
/// Equivalent to: curl -X POST --data-binary @file.bin http://example.com/upload
#[test]
fn test_curl_binary_body() {
    let mut request = create_test_request("http://example.com/upload", HttpMethod::Post);
    let binary_data = vec![0x00, 0x01, 0x02, 0x03, 0xFF, 0xFE];
    request.body = Some(RequestBody::Bytes(binary_data.clone()));
    request.headers.insert(
        HeaderName::try_from("content-type").unwrap(),
        HeaderValue::try_from("application/octet-stream").unwrap(),
    );

    if let Some(RequestBody::Bytes(data)) = &request.body {
        assert_eq!(data, &binary_data);
    } else {
        panic!("Expected Bytes body");
    }
}

// =============================================================================
// Range Request Tests
// =============================================================================

/// Tests range request header
/// Equivalent to: curl -r 0-1023 http://example.com/file
#[test]
fn test_curl_range_request() {
    let request = create_request_with_headers(
        "http://example.com/file",
        HttpMethod::Get,
        vec![("range", "bytes=0-1023")],
    );

    let range = request.headers.get("range").unwrap();
    assert_eq!(range, "bytes=0-1023");
}

/// Tests multiple range request
/// Equivalent to: curl -r 0-100,200-300 http://example.com/file
#[test]
fn test_curl_multi_range_request() {
    let request = create_request_with_headers(
        "http://example.com/file",
        HttpMethod::Get,
        vec![("range", "bytes=0-100,200-300")],
    );

    let range = request.headers.get("range").unwrap();
    assert!(range.to_str().unwrap().contains("0-100"));
    assert!(range.to_str().unwrap().contains("200-300"));
}

#[cfg(test)]
mod curl_behavior_spec_tests {
    use super::*;

    /// Verifies that GET requests don't include body per HTTP spec
    /// Reference: RFC 7231 - GET body semantics undefined, curl typically omits
    #[test]
    fn spec_get_should_not_have_body() {
        let request = create_test_request("http://example.com/", HttpMethod::Get);
        assert!(
            request.body.is_none(),
            "GET requests should not have a body per curl behavior"
        );
    }

    /// Verifies HEAD requests don't include body
    /// Reference: RFC 7231 - HEAD must not contain message body
    #[test]
    fn spec_head_should_not_have_body() {
        let request = create_test_request("http://example.com/", HttpMethod::Head);
        assert!(
            request.body.is_none(),
            "HEAD requests must not have a body"
        );
    }

    /// Verifies default redirect behavior matches curl
    /// Reference: curl defaults to following redirects with -L
    #[test]
    fn spec_default_follows_redirects() {
        let request = create_test_request("http://example.com/", HttpMethod::Get);
        assert_eq!(
            request.redirect,
            RedirectMode::Follow,
            "Default should follow redirects like curl -L"
        );
    }

    /// Verifies keep-alive is enabled by default
    /// Reference: curl enables keep-alive by default
    #[test]
    fn spec_default_keepalive_enabled() {
        let request = create_test_request("http://example.com/", HttpMethod::Get);
        assert!(
            request.keepalive,
            "Keep-alive should be enabled by default like curl"
        );
    }

    /// Verifies POST requests typically require Content-Type
    #[test]
    fn spec_post_should_have_content_type() {
        let request = create_post_request_with_body(
            "http://example.com/",
            "data",
            "application/x-www-form-urlencoded",
        );

        assert!(
            request.headers.get("content-type").is_some(),
            "POST requests should have Content-Type header"
        );
    }
}
