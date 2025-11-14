//! CORS validator implementation

use crate::{CorsConfig, CorsResult};
use crate::headers::HeaderBuilder;
use crate::preflight::PreflightChecker;
use network_types::{NetworkRequest, NetworkResponse, RequestMode};
use url::Url;

/// CORS validator
///
/// Validates network requests and responses according to CORS policy.
pub struct CorsValidator {
    config: CorsConfig,
    header_builder: HeaderBuilder,
    preflight_checker: PreflightChecker,
}

impl CorsValidator {
    /// Create a new CORS validator with the given configuration
    pub fn new(config: CorsConfig) -> Self {
        Self {
            header_builder: HeaderBuilder::new(config.clone()),
            preflight_checker: PreflightChecker::new(),
            config,
        }
    }

    /// Validate a network request
    ///
    /// Checks if the request is allowed under CORS policy for the given origin.
    ///
    /// # Arguments
    ///
    /// * `request` - The network request to validate
    /// * `origin` - The origin making the request
    ///
    /// # Returns
    ///
    /// A `CorsResult` indicating whether the request is allowed and any headers to add.
    pub fn validate_request(&self, request: &NetworkRequest, origin: &str) -> CorsResult {
        // Check same-origin policy
        if self.config.enforce_same_origin {
            if !self.is_same_origin(&request.url, origin) {
                return CorsResult::blocked("Cross-origin request blocked by same-origin policy".to_string());
            }
        }

        // For same-origin mode, no CORS headers needed
        if request.mode == RequestMode::SameOrigin {
            if !self.is_same_origin(&request.url, origin) {
                return CorsResult::blocked("Request mode is same-origin but origins differ".to_string());
            }
            return CorsResult::allowed();
        }

        // Build appropriate headers
        let headers = self.header_builder.build_request_headers(request, origin);

        CorsResult::allowed_with_headers(headers)
    }

    /// Validate a network response
    ///
    /// Adds appropriate CORS headers to the response based on the origin.
    ///
    /// # Arguments
    ///
    /// * `response` - The network response to validate
    /// * `origin` - The origin that made the request
    ///
    /// # Returns
    ///
    /// A `CorsResult` with headers to add to the response.
    pub fn validate_response(&self, response: &NetworkResponse, origin: &str) -> CorsResult {
        // Wildcard origin is not allowed with credentials
        if origin == "*" && self.config.allow_credentials {
            return CorsResult::blocked(
                "Wildcard origin (*) is not allowed when credentials are enabled".to_string()
            );
        }

        // Build appropriate CORS headers for response
        let headers = self.header_builder.build_response_headers(response, origin);

        CorsResult::allowed_with_headers(headers)
    }

    /// Check if a preflight request is needed
    ///
    /// Preflight requests (OPTIONS) are needed for:
    /// - Non-simple methods (PUT, DELETE, PATCH, etc.)
    /// - Requests with custom headers
    /// - Requests in CORS mode (not same-origin or no-cors)
    ///
    /// # Arguments
    ///
    /// * `request` - The network request to check
    ///
    /// # Returns
    ///
    /// `true` if preflight is needed, `false` otherwise.
    pub fn is_preflight_needed(&self, request: &NetworkRequest) -> bool {
        self.preflight_checker.is_preflight_needed(request)
    }

    /// Build a preflight OPTIONS request
    ///
    /// Creates a preflight request that can be used to check if the actual request
    /// will be allowed by the server.
    ///
    /// # Arguments
    ///
    /// * `request` - The original request that needs preflight
    ///
    /// # Returns
    ///
    /// A new `NetworkRequest` configured as a preflight OPTIONS request.
    pub fn build_preflight_request(&self, request: &NetworkRequest) -> NetworkRequest {
        self.preflight_checker.build_preflight_request(request)
    }

    /// Check if a URL and origin are same-origin
    fn is_same_origin(&self, url: &Url, origin: &str) -> bool {
        // Parse origin URL
        let origin_url = match Url::parse(origin) {
            Ok(url) => url,
            Err(_) => return false,
        };

        // Check scheme, host, and port
        url.scheme() == origin_url.scheme()
            && url.host_str() == origin_url.host_str()
            && url.port() == origin_url.port()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use network_types::HttpMethod;

    #[test]
    fn test_is_same_origin_true() {
        let config = CorsConfig::default();
        let validator = CorsValidator::new(config);

        let url = Url::parse("https://example.com/api").unwrap();
        let origin = "https://example.com";

        assert!(validator.is_same_origin(&url, origin));
    }

    #[test]
    fn test_is_same_origin_false_different_host() {
        let config = CorsConfig::default();
        let validator = CorsValidator::new(config);

        let url = Url::parse("https://example.com/api").unwrap();
        let origin = "https://other.com";

        assert!(!validator.is_same_origin(&url, origin));
    }

    #[test]
    fn test_is_same_origin_false_different_scheme() {
        let config = CorsConfig::default();
        let validator = CorsValidator::new(config);

        let url = Url::parse("https://example.com/api").unwrap();
        let origin = "http://example.com";

        assert!(!validator.is_same_origin(&url, origin));
    }

    #[test]
    fn test_is_same_origin_false_different_port() {
        let config = CorsConfig::default();
        let validator = CorsValidator::new(config);

        let url = Url::parse("https://example.com:8080/api").unwrap();
        let origin = "https://example.com";

        assert!(!validator.is_same_origin(&url, origin));
    }
}
