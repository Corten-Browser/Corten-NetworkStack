//! CORS header generation

use crate::CorsConfig;
use http::{HeaderMap, HeaderValue};
use network_types::{NetworkRequest, NetworkResponse};

/// CORS header builder
///
/// Builds appropriate Access-Control-* headers for requests and responses.
pub struct HeaderBuilder {
    config: CorsConfig,
}

impl HeaderBuilder {
    /// Create a new header builder with the given configuration
    pub fn new(config: CorsConfig) -> Self {
        Self { config }
    }

    /// Build CORS headers for a request
    ///
    /// Adds headers that should be included in the response to this request.
    /// When validating a request server-side, these headers indicate what
    /// the response should include.
    pub fn build_request_headers(&self, request: &NetworkRequest, _origin: &str) -> HeaderMap {
        let mut headers = HeaderMap::new();

        // Add Access-Control-Allow-Credentials if:
        // 1. Configuration allows credentials
        // 2. Request includes credentials
        use network_types::CredentialsMode;
        if self.config.allow_credentials && request.credentials == CredentialsMode::Include {
            headers.insert(
                "Access-Control-Allow-Credentials",
                HeaderValue::from_static("true"),
            );
        }

        headers
    }

    /// Build CORS headers for a response
    ///
    /// Adds Access-Control-* headers that the server should return.
    pub fn build_response_headers(&self, _response: &NetworkResponse, origin: &str) -> HeaderMap {
        let mut headers = HeaderMap::new();

        // Add Access-Control-Allow-Origin
        headers.insert(
            "Access-Control-Allow-Origin",
            HeaderValue::from_str(origin).unwrap_or(HeaderValue::from_static("*")),
        );

        // Add Access-Control-Allow-Credentials if enabled
        if self.config.allow_credentials {
            headers.insert(
                "Access-Control-Allow-Credentials",
                HeaderValue::from_static("true"),
            );
        }

        // Add Access-Control-Allow-Methods for preflight responses
        headers.insert(
            "Access-Control-Allow-Methods",
            HeaderValue::from_static("GET, POST, PUT, DELETE, PATCH, OPTIONS"),
        );

        // Add Access-Control-Allow-Headers for preflight responses
        headers.insert(
            "Access-Control-Allow-Headers",
            HeaderValue::from_static("Content-Type, Authorization, X-Requested-With"),
        );

        // Add Access-Control-Max-Age (cache preflight for 1 hour)
        headers.insert(
            "Access-Control-Max-Age",
            HeaderValue::from_static("3600"),
        );

        headers
    }

    /// Build headers for a preflight OPTIONS response
    pub fn build_preflight_response_headers(&self, origin: &str, requested_method: &str) -> HeaderMap {
        let mut headers = HeaderMap::new();

        // Add Access-Control-Allow-Origin
        headers.insert(
            "Access-Control-Allow-Origin",
            HeaderValue::from_str(origin).unwrap_or(HeaderValue::from_static("*")),
        );

        // Add Access-Control-Allow-Methods
        headers.insert(
            "Access-Control-Allow-Methods",
            HeaderValue::from_str(requested_method)
                .unwrap_or(HeaderValue::from_static("GET, POST, PUT, DELETE, OPTIONS")),
        );

        // Add Access-Control-Allow-Headers
        headers.insert(
            "Access-Control-Allow-Headers",
            HeaderValue::from_static("Content-Type, Authorization, X-Requested-With"),
        );

        // Add Access-Control-Max-Age
        headers.insert(
            "Access-Control-Max-Age",
            HeaderValue::from_static("3600"),
        );

        // Add Access-Control-Allow-Credentials if enabled
        if self.config.allow_credentials {
            headers.insert(
                "Access-Control-Allow-Credentials",
                HeaderValue::from_static("true"),
            );
        }

        headers
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_response_headers_includes_origin() {
        let config = CorsConfig::default();
        let builder = HeaderBuilder::new(config);
        let headers = builder.build_response_headers(
            &create_test_response(),
            "https://example.com",
        );

        assert!(headers.contains_key("Access-Control-Allow-Origin"));
        assert_eq!(
            headers.get("Access-Control-Allow-Origin").unwrap(),
            "https://example.com"
        );
    }

    #[test]
    fn test_build_response_headers_includes_credentials_when_enabled() {
        let config = CorsConfig {
            enforce_same_origin: false,
            allow_credentials: true,
        };
        let builder = HeaderBuilder::new(config);
        let headers = builder.build_response_headers(
            &create_test_response(),
            "https://example.com",
        );

        assert!(headers.contains_key("Access-Control-Allow-Credentials"));
        assert_eq!(
            headers.get("Access-Control-Allow-Credentials").unwrap(),
            "true"
        );
    }

    #[test]
    fn test_build_response_headers_no_credentials_when_disabled() {
        let config = CorsConfig {
            enforce_same_origin: false,
            allow_credentials: false,
        };
        let builder = HeaderBuilder::new(config);
        let headers = builder.build_response_headers(
            &create_test_response(),
            "https://example.com",
        );

        assert!(!headers.contains_key("Access-Control-Allow-Credentials"));
    }

    #[test]
    fn test_build_preflight_response_headers() {
        let config = CorsConfig::default();
        let builder = HeaderBuilder::new(config);
        let headers = builder.build_preflight_response_headers("https://example.com", "POST");

        assert!(headers.contains_key("Access-Control-Allow-Origin"));
        assert!(headers.contains_key("Access-Control-Allow-Methods"));
        assert!(headers.contains_key("Access-Control-Max-Age"));
    }

    fn create_test_response() -> NetworkResponse {
        use url::Url;
        NetworkResponse {
            url: Url::parse("https://example.com/api").unwrap(),
            status: 200,
            status_text: "OK".to_string(),
            headers: HeaderMap::new(),
            body: network_types::ResponseBody::Empty,
            redirected: false,
            type_: network_types::ResponseType::Cors,
            timing: network_types::ResourceTiming::default(),
        }
    }
}
