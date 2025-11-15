use cors_validator::{CorsValidator, CorsConfig};
use network_types::{NetworkRequest, HttpMethod, RequestMode, CredentialsMode};
use url::Url;
use http::HeaderMap;

#[test]
fn test_preflight_for_put_request() {
    // Given: CORS validator
    let config = CorsConfig {
        enforce_same_origin: false,
        allow_credentials: false,
        allowed_origins: None,
    };
    let validator = CorsValidator::new(config);

    // When: PUT request in CORS mode
    let request = create_cors_request("https://api.example.com/data", HttpMethod::Put);

    let needs_preflight = validator.is_preflight_needed(&request);

    // Then: Preflight should be needed (PUT is not a simple method)
    assert!(needs_preflight);
}

#[test]
fn test_preflight_for_delete_request() {
    // Given: CORS validator
    let config = CorsConfig {
        enforce_same_origin: false,
        allow_credentials: false,
        allowed_origins: None,
    };
    let validator = CorsValidator::new(config);

    // When: DELETE request in CORS mode
    let request = create_cors_request("https://api.example.com/data", HttpMethod::Delete);

    let needs_preflight = validator.is_preflight_needed(&request);

    // Then: Preflight should be needed
    assert!(needs_preflight);
}

#[test]
fn test_preflight_for_patch_request() {
    // Given: CORS validator
    let config = CorsConfig {
        enforce_same_origin: false,
        allow_credentials: false,
        allowed_origins: None,
    };
    let validator = CorsValidator::new(config);

    // When: PATCH request in CORS mode
    let request = create_cors_request("https://api.example.com/data", HttpMethod::Patch);

    let needs_preflight = validator.is_preflight_needed(&request);

    // Then: Preflight should be needed
    assert!(needs_preflight);
}

#[test]
fn test_preflight_request_has_correct_headers() {
    // Given: CORS validator and POST request
    let config = CorsConfig {
        enforce_same_origin: false,
        allow_credentials: false,
        allowed_origins: None,
    };
    let validator = CorsValidator::new(config);
    let request = create_cors_request("https://api.example.com/data", HttpMethod::Post);

    // When: Building preflight request
    let preflight = validator.build_preflight_request(&request);

    // Then: Preflight should have Access-Control-Request-Method header
    assert_eq!(preflight.method, HttpMethod::Options);
    let method_header = preflight.headers.get("Access-Control-Request-Method");
    assert!(method_header.is_some());
}

#[test]
fn test_no_preflight_for_head_request() {
    // Given: CORS validator
    let config = CorsConfig {
        enforce_same_origin: false,
        allow_credentials: false,
        allowed_origins: None,
    };
    let validator = CorsValidator::new(config);

    // When: HEAD request in CORS mode
    let request = create_cors_request("https://api.example.com/data", HttpMethod::Head);

    let needs_preflight = validator.is_preflight_needed(&request);

    // Then: Preflight should not be needed (HEAD is a simple method)
    assert!(!needs_preflight);
}

#[test]
fn test_no_preflight_for_no_cors_mode() {
    // Given: CORS validator
    let config = CorsConfig {
        enforce_same_origin: false,
        allow_credentials: false,
        allowed_origins: None,
    };
    let validator = CorsValidator::new(config);

    // When: POST request in no-cors mode
    let mut request = create_cors_request("https://api.example.com/data", HttpMethod::Post);
    request.mode = RequestMode::NoCors;

    let needs_preflight = validator.is_preflight_needed(&request);

    // Then: Preflight should not be needed
    assert!(!needs_preflight);
}

// Helper function
fn create_cors_request(url: &str, method: HttpMethod) -> NetworkRequest {
    NetworkRequest {
        url: Url::parse(url).unwrap(),
        method,
        headers: HeaderMap::new(),
        body: None,
        mode: RequestMode::Cors,
        credentials: CredentialsMode::Omit,
        cache: network_types::CacheMode::Default,
        redirect: network_types::RedirectMode::Follow,
        referrer: None,
        referrer_policy: network_types::ReferrerPolicy::NoReferrerWhenDowngrade,
        integrity: None,
        keepalive: false,
        signal: None,
        priority: network_types::RequestPriority::Auto,
        window: None,
    }
}
