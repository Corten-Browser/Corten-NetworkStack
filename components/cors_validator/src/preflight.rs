//! Preflight request handling

use network_types::{HttpMethod, NetworkRequest, RequestMode, CredentialsMode};
use http::{HeaderMap, HeaderValue};

/// Preflight request checker
///
/// Determines if a preflight request is needed and builds preflight requests.
pub struct PreflightChecker;

impl PreflightChecker {
    /// Create a new preflight checker
    pub fn new() -> Self {
        Self
    }

    /// Check if a preflight request is needed
    ///
    /// Preflight is needed for:
    /// - Non-simple methods (anything other than GET, HEAD, POST)
    /// - POST requests with non-simple content types
    /// - Requests with custom headers
    /// - CORS mode requests (not same-origin or no-cors)
    pub fn is_preflight_needed(&self, request: &NetworkRequest) -> bool {
        // No preflight for same-origin or no-cors modes
        if request.mode == RequestMode::SameOrigin || request.mode == RequestMode::NoCors {
            return false;
        }

        // CORS mode - check if method is simple
        !self.is_simple_method(&request.method)
    }

    /// Build a preflight OPTIONS request
    ///
    /// Creates a preflight request with appropriate Access-Control-Request-* headers.
    pub fn build_preflight_request(&self, request: &NetworkRequest) -> NetworkRequest {
        let mut preflight = NetworkRequest {
            url: request.url.clone(),
            method: HttpMethod::Options,
            headers: HeaderMap::new(),
            body: None,
            mode: request.mode,
            credentials: request.credentials,
            cache: request.cache,
            redirect: request.redirect,
            referrer: request.referrer.clone(),
            referrer_policy: request.referrer_policy,
            integrity: None,
            keepalive: false,
            signal: None,
            priority: request.priority,
            window: request.window,
        };

        // Add Access-Control-Request-Method header
        let method_str = method_to_string(&request.method);
        preflight.headers.insert(
            "Access-Control-Request-Method",
            HeaderValue::from_str(&method_str)
                .unwrap_or_else(|_| HeaderValue::from_static("GET")),
        );

        // If original request has custom headers, add Access-Control-Request-Headers
        if !request.headers.is_empty() {
            let header_names: Vec<String> = request
                .headers
                .keys()
                .map(|k| k.as_str().to_lowercase())
                .collect();
            let header_names_str = header_names.join(", ");
            preflight.headers.insert(
                "Access-Control-Request-Headers",
                HeaderValue::from_str(&header_names_str)
                    .unwrap_or_else(|_| HeaderValue::from_static("")),
            );
        }

        preflight
    }

    /// Check if a method is a "simple" method (doesn't require preflight)
    ///
    /// Simple methods are: GET, HEAD, POST (with simple content type)
    fn is_simple_method(&self, method: &HttpMethod) -> bool {
        matches!(method, HttpMethod::Get | HttpMethod::Head)
    }
}

impl Default for PreflightChecker {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert HTTP method to string
fn method_to_string(method: &HttpMethod) -> String {
    match method {
        HttpMethod::Get => "GET",
        HttpMethod::Head => "HEAD",
        HttpMethod::Post => "POST",
        HttpMethod::Put => "PUT",
        HttpMethod::Delete => "DELETE",
        HttpMethod::Connect => "CONNECT",
        HttpMethod::Options => "OPTIONS",
        HttpMethod::Trace => "TRACE",
        HttpMethod::Patch => "PATCH",
    }
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use url::Url;

    fn create_request(method: HttpMethod, mode: RequestMode) -> NetworkRequest {
        NetworkRequest {
            url: Url::parse("https://example.com/api").unwrap(),
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

    #[test]
    fn test_simple_method_get() {
        let checker = PreflightChecker::new();
        assert!(checker.is_simple_method(&HttpMethod::Get));
    }

    #[test]
    fn test_simple_method_head() {
        let checker = PreflightChecker::new();
        assert!(checker.is_simple_method(&HttpMethod::Head));
    }

    #[test]
    fn test_not_simple_method_put() {
        let checker = PreflightChecker::new();
        assert!(!checker.is_simple_method(&HttpMethod::Put));
    }

    #[test]
    fn test_not_simple_method_delete() {
        let checker = PreflightChecker::new();
        assert!(!checker.is_simple_method(&HttpMethod::Delete));
    }

    #[test]
    fn test_preflight_needed_for_put() {
        let checker = PreflightChecker::new();
        let request = create_request(HttpMethod::Put, RequestMode::Cors);
        assert!(checker.is_preflight_needed(&request));
    }

    #[test]
    fn test_preflight_not_needed_for_same_origin_mode() {
        let checker = PreflightChecker::new();
        let request = create_request(HttpMethod::Put, RequestMode::SameOrigin);
        assert!(!checker.is_preflight_needed(&request));
    }

    #[test]
    fn test_build_preflight_has_options_method() {
        let checker = PreflightChecker::new();
        let request = create_request(HttpMethod::Post, RequestMode::Cors);
        let preflight = checker.build_preflight_request(&request);
        assert_eq!(preflight.method, HttpMethod::Options);
    }

    #[test]
    fn test_build_preflight_has_request_method_header() {
        let checker = PreflightChecker::new();
        let request = create_request(HttpMethod::Post, RequestMode::Cors);
        let preflight = checker.build_preflight_request(&request);
        assert!(preflight.headers.contains_key("Access-Control-Request-Method"));
    }
}
