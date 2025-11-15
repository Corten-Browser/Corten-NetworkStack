use cors_validator::{CorsValidator, CorsConfig};
use network_types::{NetworkRequest, NetworkResponse, HttpMethod, RequestMode, CredentialsMode};
use url::Url;
use http::HeaderMap;

#[test]
fn test_same_origin_request_allowed() {
    // Given: CORS validator with same-origin enforcement
    let config = CorsConfig {
        enforce_same_origin: true,
        allow_credentials: false,
        allowed_origins: None,
    };
    let validator = CorsValidator::new(config);

    // When: Request from same origin
    let request = create_test_request("https://example.com/api", HttpMethod::Get);
    let origin = "https://example.com";

    let result = validator.validate_request(&request, origin);

    // Then: Request should be allowed
    assert!(result.allowed, "Same-origin request should be allowed");
    assert_eq!(result.reason, None);
}

#[test]
fn test_cross_origin_request_blocked_with_same_origin_enforcement() {
    // Given: CORS validator with same-origin enforcement
    let config = CorsConfig {
        enforce_same_origin: true,
        allow_credentials: false,
        allowed_origins: None,
    };
    let validator = CorsValidator::new(config);

    // When: Request from different origin
    let request = create_test_request("https://example.com/api", HttpMethod::Get);
    let origin = "https://other.com";

    let result = validator.validate_request(&request, origin);

    // Then: Request should be blocked
    assert!(!result.allowed, "Cross-origin request should be blocked");
    assert!(result.reason.is_some());
    assert!(result.reason.unwrap().contains("origin"));
}

#[test]
fn test_cross_origin_request_allowed_without_enforcement() {
    // Given: CORS validator without same-origin enforcement
    let config = CorsConfig {
        enforce_same_origin: false,
        allow_credentials: false,
        allowed_origins: None,
    };
    let validator = CorsValidator::new(config);

    // When: Request from different origin
    let request = create_test_request("https://example.com/api", HttpMethod::Get);
    let origin = "https://other.com";

    let result = validator.validate_request(&request, origin);

    // Then: Request should be allowed
    assert!(result.allowed, "Cross-origin request should be allowed when enforcement disabled");
}

#[test]
fn test_preflight_needed_for_cors_mode() {
    // Given: CORS validator
    let config = CorsConfig {
        enforce_same_origin: false,
        allow_credentials: false,
        allowed_origins: None,
    };
    let validator = CorsValidator::new(config);

    // When: Request with CORS mode
    let request = create_test_request_with_mode("https://example.com/api", HttpMethod::Post, RequestMode::Cors);

    let needs_preflight = validator.is_preflight_needed(&request);

    // Then: Preflight should be needed
    assert!(needs_preflight, "Preflight should be needed for CORS mode POST request");
}

#[test]
fn test_preflight_not_needed_for_same_origin_mode() {
    // Given: CORS validator
    let config = CorsConfig {
        enforce_same_origin: false,
        allow_credentials: false,
        allowed_origins: None,
    };
    let validator = CorsValidator::new(config);

    // When: Request with same-origin mode
    let request = create_test_request_with_mode("https://example.com/api", HttpMethod::Post, RequestMode::SameOrigin);

    let needs_preflight = validator.is_preflight_needed(&request);

    // Then: Preflight should not be needed
    assert!(!needs_preflight, "Preflight should not be needed for same-origin mode");
}

#[test]
fn test_preflight_not_needed_for_simple_get_request() {
    // Given: CORS validator
    let config = CorsConfig {
        enforce_same_origin: false,
        allow_credentials: false,
        allowed_origins: None,
    };
    let validator = CorsValidator::new(config);

    // When: Simple GET request
    let request = create_test_request_with_mode("https://example.com/api", HttpMethod::Get, RequestMode::Cors);

    let needs_preflight = validator.is_preflight_needed(&request);

    // Then: Preflight should not be needed for simple GET
    assert!(!needs_preflight, "Preflight should not be needed for simple GET request");
}

#[test]
fn test_credential_mode_validation() {
    // Given: CORS validator with credentials allowed
    let config = CorsConfig {
        enforce_same_origin: false,
        allow_credentials: true,
        allowed_origins: Some(vec!["https://example.com".to_string()]),
    };
    let validator = CorsValidator::new(config);

    // When: Request with include credentials mode
    let request = create_test_request_with_credentials("https://example.com/api", HttpMethod::Get, CredentialsMode::Include);
    let origin = "https://other.com";

    let result = validator.validate_request(&request, origin);

    // Then: Headers should include Access-Control-Allow-Credentials
    assert!(result.allowed);
    let cred_header = result.headers_to_add.get("Access-Control-Allow-Credentials");
    assert!(cred_header.is_some());
}

#[test]
fn test_validate_response_adds_cors_headers() {
    // Given: CORS validator
    let config = CorsConfig {
        enforce_same_origin: false,
        allow_credentials: false,
        allowed_origins: None,
    };
    let validator = CorsValidator::new(config);

    // When: Validating response
    let response = create_test_response(200);
    let origin = "https://other.com";

    let result = validator.validate_response(&response, origin);

    // Then: Result should include Access-Control-Allow-Origin header
    assert!(result.allowed);
    let origin_header = result.headers_to_add.get("Access-Control-Allow-Origin");
    assert!(origin_header.is_some());
}

#[test]
fn test_build_preflight_request() {
    // Given: CORS validator and original request
    let config = CorsConfig {
        enforce_same_origin: false,
        allow_credentials: false,
        allowed_origins: None,
    };
    let validator = CorsValidator::new(config);
    let request = create_test_request("https://example.com/api", HttpMethod::Post);

    // When: Building preflight request
    let preflight = validator.build_preflight_request(&request);

    // Then: Preflight should be OPTIONS request
    assert_eq!(preflight.method, HttpMethod::Options);
    assert_eq!(preflight.url, request.url);
}

#[test]
fn test_wildcard_origin_handling() {
    // Given: CORS validator without credentials
    let config = CorsConfig {
        enforce_same_origin: false,
        allow_credentials: false,
        allowed_origins: None,
    };
    let validator = CorsValidator::new(config);

    // When: Validating response with wildcard
    let response = create_test_response(200);
    let origin = "*";

    let result = validator.validate_response(&response, origin);

    // Then: Should allow wildcard when credentials not required
    assert!(result.allowed);
}

#[test]
fn test_wildcard_origin_blocked_with_credentials() {
    // Given: CORS validator with credentials allowed
    let config = CorsConfig {
        enforce_same_origin: false,
        allow_credentials: true,
        allowed_origins: Some(vec!["https://example.com".to_string()]),
    };
    let validator = CorsValidator::new(config);

    // When: Trying to validate with wildcard origin and credentials
    let response = create_test_response(200);
    let origin = "*";

    let result = validator.validate_response(&response, origin);

    // Then: Should be blocked (wildcard not allowed with credentials)
    assert!(!result.allowed);
    assert!(result.reason.is_some());
}

// Configuration validation tests
#[test]
#[should_panic(expected = "Invalid CORS configuration")]
fn test_reject_wildcard_with_credentials_none() {
    // Given: CORS config with credentials enabled but no specific origins (None = wildcard)
    let config = CorsConfig {
        enforce_same_origin: false,
        allow_credentials: true,
        allowed_origins: None,
    };

    // When/Then: Creating validator should panic
    CorsValidator::new(config);
}

#[test]
#[should_panic(expected = "Invalid CORS configuration")]
fn test_reject_wildcard_with_credentials_explicit() {
    // Given: CORS config with credentials enabled and explicit wildcard
    let config = CorsConfig {
        enforce_same_origin: false,
        allow_credentials: true,
        allowed_origins: Some(vec!["*".to_string()]),
    };

    // When/Then: Creating validator should panic
    CorsValidator::new(config);
}

#[test]
fn test_allow_wildcard_without_credentials() {
    // Given: CORS config with wildcard but no credentials
    let config = CorsConfig {
        enforce_same_origin: false,
        allow_credentials: false,
        allowed_origins: Some(vec!["*".to_string()]),
    };

    // When: Creating validator
    let _validator = CorsValidator::new(config);

    // Then: Should succeed (wildcard OK without credentials)
    // Validator created successfully without panic
}

#[test]
fn test_allow_specific_origins_with_credentials() {
    // Given: CORS config with specific origins and credentials
    let config = CorsConfig {
        enforce_same_origin: false,
        allow_credentials: true,
        allowed_origins: Some(vec!["https://example.com".to_string()]),
    };

    // When: Creating validator
    let _validator = CorsValidator::new(config);

    // Then: Should succeed (specific origin OK with credentials)
    // Validator created successfully without panic
}

#[test]
fn test_try_new_returns_error_for_wildcard_with_credentials() {
    // Given: CORS config with wildcard and credentials
    let config = CorsConfig {
        enforce_same_origin: false,
        allow_credentials: true,
        allowed_origins: Some(vec!["*".to_string()]),
    };

    // When: Using try_new
    let result = CorsValidator::try_new(config);

    // Then: Should return error
    assert!(result.is_err());
    if let Err(error_msg) = result {
        assert!(error_msg.contains("wildcard"));
        assert!(error_msg.contains("credentials"));
    }
}

#[test]
fn test_try_new_succeeds_for_valid_config() {
    // Given: Valid CORS config with specific origins and credentials
    let config = CorsConfig {
        enforce_same_origin: false,
        allow_credentials: true,
        allowed_origins: Some(vec!["https://example.com".to_string(), "https://api.example.com".to_string()]),
    };

    // When: Using try_new
    let result = CorsValidator::try_new(config);

    // Then: Should succeed
    assert!(result.is_ok());
}

#[test]
fn test_config_validate_method() {
    // Given: Invalid config (wildcard with credentials)
    let invalid_config = CorsConfig {
        enforce_same_origin: false,
        allow_credentials: true,
        allowed_origins: None,
    };

    // When: Calling validate
    let result = invalid_config.validate();

    // Then: Should return error
    assert!(result.is_err());
    if let Err(err) = result {
        assert!(err.contains("wildcard"));
    }

    // Given: Valid config
    let valid_config = CorsConfig {
        enforce_same_origin: false,
        allow_credentials: true,
        allowed_origins: Some(vec!["https://example.com".to_string()]),
    };

    // When: Calling validate
    let result = valid_config.validate();

    // Then: Should succeed
    assert!(result.is_ok());
}

// Helper functions to create test data
fn create_test_request(url: &str, method: HttpMethod) -> NetworkRequest {
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

fn create_test_request_with_mode(url: &str, method: HttpMethod, mode: RequestMode) -> NetworkRequest {
    let mut request = create_test_request(url, method);
    request.mode = mode;
    request
}

fn create_test_request_with_credentials(url: &str, method: HttpMethod, credentials: CredentialsMode) -> NetworkRequest {
    let mut request = create_test_request(url, method);
    request.credentials = credentials;
    request
}

fn create_test_response(status: u16) -> NetworkResponse {
    NetworkResponse {
        url: Url::parse("https://example.com/api").unwrap(),
        status,
        status_text: "OK".to_string(),
        headers: HeaderMap::new(),
        body: network_types::ResponseBody::Empty,
        redirected: false,
        type_: network_types::ResponseType::Basic,
        timing: network_types::ResourceTiming::default(),
    }
}
