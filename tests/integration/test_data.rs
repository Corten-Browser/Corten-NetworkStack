/// Test data generators for integration tests
use network_types::{NetworkRequest, NetworkResponse, HttpMethod};
use url::Url;
use http::HeaderMap;

/// Create a simple GET request for testing
pub fn create_get_request(url: Url) -> NetworkRequest {
    NetworkRequest {
        url,
        method: HttpMethod::Get,
        headers: HeaderMap::new(),
        body: None,
        mode: Default::default(),
        credentials: Default::default(),
        cache: Default::default(),
        redirect: Default::default(),
        referrer: None,
        referrer_policy: Default::default(),
        integrity: None,
        keepalive: false,
        signal: None,
        priority: Default::default(),
        window: None,
    }
}

/// Create a POST request with body for testing
pub fn create_post_request(url: Url, body: Vec<u8>) -> NetworkRequest {
    NetworkRequest {
        url,
        method: HttpMethod::Post,
        headers: HeaderMap::new(),
        body: Some(network_types::RequestBody::Bytes(body)),
        mode: Default::default(),
        credentials: Default::default(),
        cache: Default::default(),
        redirect: Default::default(),
        referrer: None,
        referrer_policy: Default::default(),
        integrity: None,
        keepalive: false,
        signal: None,
        priority: Default::default(),
        window: None,
    }
}

/// Create a mock response for testing
pub fn create_test_response(status: u16, body: Vec<u8>) -> NetworkResponse {
    NetworkResponse {
        url: Url::parse("http://example.com/").unwrap(),
        status,
        status_text: status_text(status),
        headers: HeaderMap::new(),
        body: network_types::ResponseBody::Bytes(body),
        redirected: false,
        type_: Default::default(),
        timing: Default::default(),
    }
}

/// Get standard HTTP status text
fn status_text(status: u16) -> String {
    match status {
        200 => "OK",
        201 => "Created",
        204 => "No Content",
        301 => "Moved Permanently",
        302 => "Found",
        304 => "Not Modified",
        400 => "Bad Request",
        401 => "Unauthorized",
        403 => "Forbidden",
        404 => "Not Found",
        500 => "Internal Server Error",
        502 => "Bad Gateway",
        503 => "Service Unavailable",
        _ => "Unknown",
    }.to_string()
}

/// Sample HTML response body
pub fn sample_html_body() -> Vec<u8> {
    br#"<!DOCTYPE html>
<html>
<head><title>Test Page</title></head>
<body><h1>Integration Test</h1></body>
</html>"#.to_vec()
}

/// Sample JSON response body
pub fn sample_json_body() -> Vec<u8> {
    br#"{"status": "ok", "message": "Integration test response"}"#.to_vec()
}

/// Sample plain text response body
pub fn sample_text_body() -> Vec<u8> {
    b"Integration test response body".to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_get_request() {
        let url = Url::parse("http://example.com/test").unwrap();
        let request = create_get_request(url.clone());

        assert_eq!(request.url, url);
        assert_eq!(request.method, HttpMethod::Get);
        assert!(request.body.is_none());
    }

    #[test]
    fn test_create_post_request() {
        let url = Url::parse("http://example.com/api").unwrap();
        let body = b"test data".to_vec();
        let request = create_post_request(url.clone(), body.clone());

        assert_eq!(request.url, url);
        assert_eq!(request.method, HttpMethod::Post);
        assert!(request.body.is_some());
    }

    #[test]
    fn test_create_test_response() {
        let response = create_test_response(200, sample_html_body());

        assert_eq!(response.status, 200);
        assert_eq!(response.status_text, "OK");
        assert!(!response.redirected);
    }

    #[test]
    fn test_status_text_mapping() {
        let response = create_test_response(404, vec![]);
        assert_eq!(response.status_text, "Not Found");
    }
}
