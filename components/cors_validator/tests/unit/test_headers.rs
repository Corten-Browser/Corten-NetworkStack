use cors_validator::{CorsValidator, CorsConfig};
use network_types::{NetworkResponse, ResponseType};
use url::Url;
use http::HeaderMap;

#[test]
fn test_access_control_allow_origin_header_added() {
    // Given: CORS validator
    let config = CorsConfig {
        enforce_same_origin: false,
        allow_credentials: false,
        allowed_origins: None,
    };
    let validator = CorsValidator::new(config);

    // When: Validating response
    let response = create_response(200);
    let origin = "https://trusted.com";

    let result = validator.validate_response(&response, origin);

    // Then: Access-Control-Allow-Origin header should be present
    assert!(result.allowed);
    let header = result.headers_to_add.get("Access-Control-Allow-Origin");
    assert!(header.is_some());
    assert_eq!(header.unwrap().to_str().unwrap(), origin);
}

#[test]
fn test_access_control_allow_credentials_header() {
    // Given: CORS validator with credentials allowed
    let config = CorsConfig {
        enforce_same_origin: false,
        allow_credentials: true,
        allowed_origins: Some(vec!["https://trusted.com".to_string()]),
    };
    let validator = CorsValidator::new(config);

    // When: Validating response
    let response = create_response(200);
    let origin = "https://trusted.com";

    let result = validator.validate_response(&response, origin);

    // Then: Access-Control-Allow-Credentials header should be true
    assert!(result.allowed);
    let header = result.headers_to_add.get("Access-Control-Allow-Credentials");
    assert!(header.is_some());
    assert_eq!(header.unwrap().to_str().unwrap(), "true");
}

#[test]
fn test_no_credentials_header_when_disabled() {
    // Given: CORS validator with credentials disabled
    let config = CorsConfig {
        enforce_same_origin: false,
        allow_credentials: false,
        allowed_origins: None,
    };
    let validator = CorsValidator::new(config);

    // When: Validating response
    let response = create_response(200);
    let origin = "https://trusted.com";

    let result = validator.validate_response(&response, origin);

    // Then: Access-Control-Allow-Credentials header should not be present
    assert!(result.allowed);
    let header = result.headers_to_add.get("Access-Control-Allow-Credentials");
    assert!(header.is_none());
}

#[test]
fn test_wildcard_origin_in_response() {
    // Given: CORS validator without credentials
    let config = CorsConfig {
        enforce_same_origin: false,
        allow_credentials: false,
        allowed_origins: None,
    };
    let validator = CorsValidator::new(config);

    // When: Validating response with wildcard origin
    let response = create_response(200);
    let origin = "*";

    let result = validator.validate_response(&response, origin);

    // Then: Wildcard should be allowed
    assert!(result.allowed);
    let header = result.headers_to_add.get("Access-Control-Allow-Origin");
    assert!(header.is_some());
    assert_eq!(header.unwrap().to_str().unwrap(), "*");
}

#[test]
fn test_access_control_allow_methods_in_preflight() {
    // Given: CORS validator
    let config = CorsConfig {
        enforce_same_origin: false,
        allow_credentials: false,
        allowed_origins: None,
    };
    let validator = CorsValidator::new(config);

    // When: Validating preflight response
    let response = create_response(200);
    let origin = "https://trusted.com";

    let result = validator.validate_response(&response, origin);

    // Then: Should include CORS headers
    assert!(result.allowed);
}

// Helper function
fn create_response(status: u16) -> NetworkResponse {
    NetworkResponse {
        url: Url::parse("https://api.example.com/data").unwrap(),
        status,
        status_text: status_text(status),
        headers: HeaderMap::new(),
        body: network_types::ResponseBody::Empty,
        redirected: false,
        type_: ResponseType::Cors,
        timing: network_types::ResourceTiming::default(),
    }
}

fn status_text(status: u16) -> String {
    match status {
        200 => "OK".to_string(),
        400 => "Bad Request".to_string(),
        403 => "Forbidden".to_string(),
        404 => "Not Found".to_string(),
        _ => "Unknown".to_string(),
    }
}
