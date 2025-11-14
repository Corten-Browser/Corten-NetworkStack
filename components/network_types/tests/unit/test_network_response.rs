use http::HeaderMap;
use network_types::{NetworkResponse, ResourceTiming, ResponseBody, ResponseType};
use url::Url;

#[test]
fn test_network_response_creation() {
    // Given response parameters
    // When creating a NetworkResponse
    // Then it should contain all fields
    let url = Url::parse("https://example.com").unwrap();
    let headers = HeaderMap::new();
    let timing = ResourceTiming {
        start_time: 0.0,
        redirect_start: 0.0,
        redirect_end: 0.0,
        fetch_start: 0.0,
        domain_lookup_start: 0.0,
        domain_lookup_end: 0.0,
        connect_start: 0.0,
        connect_end: 0.0,
        secure_connection_start: 0.0,
        request_start: 0.0,
        response_start: 0.0,
        response_end: 0.0,
        transfer_size: 0,
        encoded_body_size: 0,
        decoded_body_size: 0,
    };

    let response = NetworkResponse {
        url: url.clone(),
        status: 200,
        status_text: "OK".to_string(),
        headers: headers.clone(),
        body: ResponseBody::Empty,
        redirected: false,
        type_: ResponseType::Basic,
        timing: timing.clone(),
    };

    assert_eq!(response.url, url);
    assert_eq!(response.status, 200);
    assert_eq!(response.status_text, "OK");
    assert_eq!(response.redirected, false);
    assert_eq!(response.type_, ResponseType::Basic);
}

#[test]
fn test_network_response_with_body() {
    // Given a response with body data
    // When creating a NetworkResponse
    // Then the body should be accessible
    let url = Url::parse("https://api.example.com/data").unwrap();
    let body_data = b"response data".to_vec();
    let timing = ResourceTiming {
        start_time: 1.0,
        redirect_start: 0.0,
        redirect_end: 0.0,
        fetch_start: 1.0,
        domain_lookup_start: 1.5,
        domain_lookup_end: 2.0,
        connect_start: 2.0,
        connect_end: 3.0,
        secure_connection_start: 2.5,
        request_start: 3.0,
        response_start: 4.0,
        response_end: 5.0,
        transfer_size: 1024,
        encoded_body_size: 512,
        decoded_body_size: 768,
    };

    let response = NetworkResponse {
        url,
        status: 200,
        status_text: "OK".to_string(),
        headers: HeaderMap::new(),
        body: ResponseBody::Bytes(body_data.clone()),
        redirected: false,
        type_: ResponseType::Cors,
        timing,
    };

    match response.body {
        ResponseBody::Bytes(bytes) => assert_eq!(bytes, body_data),
        _ => panic!("Expected Bytes body"),
    }
}

#[test]
fn test_network_response_error_status() {
    // Given an error response
    // When creating a NetworkResponse
    // Then it should handle error status codes
    let url = Url::parse("https://example.com/notfound").unwrap();
    let timing = ResourceTiming {
        start_time: 0.0,
        redirect_start: 0.0,
        redirect_end: 0.0,
        fetch_start: 0.0,
        domain_lookup_start: 0.0,
        domain_lookup_end: 0.0,
        connect_start: 0.0,
        connect_end: 0.0,
        secure_connection_start: 0.0,
        request_start: 0.0,
        response_start: 0.0,
        response_end: 0.0,
        transfer_size: 0,
        encoded_body_size: 0,
        decoded_body_size: 0,
    };

    let response = NetworkResponse {
        url,
        status: 404,
        status_text: "Not Found".to_string(),
        headers: HeaderMap::new(),
        body: ResponseBody::Empty,
        redirected: false,
        type_: ResponseType::Error,
        timing,
    };

    assert_eq!(response.status, 404);
    assert_eq!(response.status_text, "Not Found");
    assert_eq!(response.type_, ResponseType::Error);
}

#[test]
fn test_network_response_redirected() {
    // Given a redirected response
    // When creating a NetworkResponse
    // Then it should track redirect status
    let url = Url::parse("https://example.com/final").unwrap();
    let timing = ResourceTiming {
        start_time: 0.0,
        redirect_start: 1.0,
        redirect_end: 2.0,
        fetch_start: 2.0,
        domain_lookup_start: 0.0,
        domain_lookup_end: 0.0,
        connect_start: 0.0,
        connect_end: 0.0,
        secure_connection_start: 0.0,
        request_start: 0.0,
        response_start: 0.0,
        response_end: 0.0,
        transfer_size: 0,
        encoded_body_size: 0,
        decoded_body_size: 0,
    };

    let response = NetworkResponse {
        url,
        status: 200,
        status_text: "OK".to_string(),
        headers: HeaderMap::new(),
        body: ResponseBody::Empty,
        redirected: true,
        type_: ResponseType::Basic,
        timing,
    };

    assert_eq!(response.redirected, true);
    assert!(response.timing.redirect_start > 0.0);
}

#[test]
fn test_network_response_debug() {
    // Given a network response
    // When debug formatted
    // Then it should produce readable output
    let url = Url::parse("https://example.com").unwrap();
    let timing = ResourceTiming {
        start_time: 0.0,
        redirect_start: 0.0,
        redirect_end: 0.0,
        fetch_start: 0.0,
        domain_lookup_start: 0.0,
        domain_lookup_end: 0.0,
        connect_start: 0.0,
        connect_end: 0.0,
        secure_connection_start: 0.0,
        request_start: 0.0,
        response_start: 0.0,
        response_end: 0.0,
        transfer_size: 0,
        encoded_body_size: 0,
        decoded_body_size: 0,
    };

    let response = NetworkResponse {
        url,
        status: 200,
        status_text: "OK".to_string(),
        headers: HeaderMap::new(),
        body: ResponseBody::Empty,
        redirected: false,
        type_: ResponseType::Basic,
        timing,
    };

    let debug_str = format!("{:?}", response);
    assert!(debug_str.contains("NetworkResponse"));
}
