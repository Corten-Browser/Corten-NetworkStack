use cors_validator::{CorsValidator, CorsConfig};
use network_types::{NetworkRequest, NetworkResponse, HttpMethod, RequestMode, CredentialsMode, ResponseType};
use url::Url;
use http::HeaderMap;

#[test]
fn test_complete_cors_preflight_workflow() {
    // Given: CORS validator and cross-origin POST request
    let config = CorsConfig {
        enforce_same_origin: false,
        allow_credentials: false,
        allowed_origins: None,
    };
    let validator = CorsValidator::new(config);

    let request = create_request("https://api.example.com/data", HttpMethod::Post, RequestMode::Cors);
    let origin = "https://app.other.com";

    // When: Check if preflight needed
    let needs_preflight = validator.is_preflight_needed(&request);
    assert!(needs_preflight, "POST request should need preflight");

    // When: Build and validate preflight request
    let preflight = validator.build_preflight_request(&request);
    assert_eq!(preflight.method, HttpMethod::Options);

    // When: Validate actual request
    let result = validator.validate_request(&request, origin);
    assert!(result.allowed);

    // When: Validate response
    let response = create_response(200, ResponseType::Cors);
    let response_result = validator.validate_response(&response, origin);
    assert!(response_result.allowed);
    assert!(response_result.headers_to_add.get("Access-Control-Allow-Origin").is_some());
}

#[test]
fn test_same_origin_workflow_no_preflight() {
    // Given: CORS validator and same-origin request
    let config = CorsConfig {
        enforce_same_origin: true,
        allow_credentials: false,
        allowed_origins: None,
    };
    let validator = CorsValidator::new(config);

    let request = create_request("https://example.com/api/data", HttpMethod::Post, RequestMode::SameOrigin);
    let origin = "https://example.com";

    // When: Check if preflight needed
    let needs_preflight = validator.is_preflight_needed(&request);
    assert!(!needs_preflight, "Same-origin mode should not need preflight");

    // When: Validate request
    let result = validator.validate_request(&request, origin);
    assert!(result.allowed);
    assert!(result.reason.is_none());
}

#[test]
fn test_credentials_workflow() {
    // Given: CORS validator with credentials enabled
    let config = CorsConfig {
        enforce_same_origin: false,
        allow_credentials: true,
        allowed_origins: Some(vec!["https://trusted.com".to_string()]),
    };
    let validator = CorsValidator::new(config);

    let mut request = create_request("https://api.example.com/data", HttpMethod::Get, RequestMode::Cors);
    request.credentials = CredentialsMode::Include;
    let origin = "https://trusted.com";

    // When: Validate request with credentials
    let result = validator.validate_request(&request, origin);
    assert!(result.allowed);

    // When: Validate response
    let response = create_response(200, ResponseType::Cors);
    let response_result = validator.validate_response(&response, origin);

    // Then: Should include credentials header
    assert!(response_result.allowed);
    let cred_header = response_result.headers_to_add.get("Access-Control-Allow-Credentials");
    assert!(cred_header.is_some());
    assert_eq!(cred_header.unwrap().to_str().unwrap(), "true");

    // Then: Origin should be specific (not wildcard)
    let origin_header = response_result.headers_to_add.get("Access-Control-Allow-Origin");
    assert_eq!(origin_header.unwrap().to_str().unwrap(), origin);
}

#[test]
fn test_blocked_cross_origin_workflow() {
    // Given: CORS validator with same-origin enforcement
    let config = CorsConfig {
        enforce_same_origin: true,
        allow_credentials: false,
        allowed_origins: None,
    };
    let validator = CorsValidator::new(config);

    let request = create_request("https://api.example.com/data", HttpMethod::Get, RequestMode::Cors);
    let origin = "https://malicious.com";

    // When: Validate cross-origin request with enforcement
    let result = validator.validate_request(&request, origin);

    // Then: Should be blocked
    assert!(!result.allowed);
    assert!(result.reason.is_some());
    let reason = result.reason.as_ref().unwrap();
    assert!(reason.contains("origin") || reason.contains("Origin"));
}

// Helper functions
fn create_request(url: &str, method: HttpMethod, mode: RequestMode) -> NetworkRequest {
    NetworkRequest {
        url: Url::parse(url).unwrap(),
        method,
        headers: HeaderMap::new(),
        body: None,
        mode,
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

fn create_response(status: u16, response_type: ResponseType) -> NetworkResponse {
    NetworkResponse {
        url: Url::parse("https://api.example.com/data").unwrap(),
        status,
        status_text: "OK".to_string(),
        headers: HeaderMap::new(),
        body: network_types::ResponseBody::Empty,
        redirected: false,
        type_: response_type,
        timing: network_types::ResourceTiming::default(),
    }
}
