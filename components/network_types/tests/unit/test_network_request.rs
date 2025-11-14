use http::HeaderMap;
use network_types::{
    CacheMode, CredentialsMode, HttpMethod, NetworkRequest, RedirectMode, ReferrerPolicy,
    RequestMode, RequestPriority,
};
use url::Url;

#[test]
fn test_network_request_creation() {
    // Given request parameters
    // When creating a NetworkRequest
    // Then it should contain all fields
    let url = Url::parse("https://example.com").unwrap();
    let headers = HeaderMap::new();

    let request = NetworkRequest {
        url: url.clone(),
        method: HttpMethod::Get,
        headers: headers.clone(),
        body: None,
        mode: RequestMode::Cors,
        credentials: CredentialsMode::SameOrigin,
        cache: CacheMode::Default,
        redirect: RedirectMode::Follow,
        referrer: None,
        referrer_policy: ReferrerPolicy::NoReferrerWhenDowngrade,
        integrity: None,
        keepalive: false,
        signal: None,
        priority: RequestPriority::Auto,
        window: None,
    };

    assert_eq!(request.url, url);
    assert_eq!(request.method, HttpMethod::Get);
    assert_eq!(request.mode, RequestMode::Cors);
}

#[test]
fn test_network_request_with_body() {
    // Given a request with body data
    // When creating a NetworkRequest
    // Then the body should be accessible
    let url = Url::parse("https://example.com/api").unwrap();
    let body_data = b"test data".to_vec();

    let request = NetworkRequest {
        url,
        method: HttpMethod::Post,
        headers: HeaderMap::new(),
        body: Some(network_types::RequestBody::Bytes(body_data.clone())),
        mode: RequestMode::Cors,
        credentials: CredentialsMode::Include,
        cache: CacheMode::NoStore,
        redirect: RedirectMode::Follow,
        referrer: None,
        referrer_policy: ReferrerPolicy::StrictOrigin,
        integrity: Some("sha256-abc123".to_string()),
        keepalive: true,
        signal: None,
        priority: RequestPriority::High,
        window: None,
    };

    assert!(request.body.is_some());
    assert_eq!(request.method, HttpMethod::Post);
    assert_eq!(request.keepalive, true);
}

#[test]
fn test_network_request_debug() {
    // Given a network request
    // When debug formatted
    // Then it should produce readable output
    let url = Url::parse("https://example.com").unwrap();
    let request = NetworkRequest {
        url,
        method: HttpMethod::Get,
        headers: HeaderMap::new(),
        body: None,
        mode: RequestMode::NoCors,
        credentials: CredentialsMode::Omit,
        cache: CacheMode::Default,
        redirect: RedirectMode::Follow,
        referrer: None,
        referrer_policy: ReferrerPolicy::NoReferrer,
        integrity: None,
        keepalive: false,
        signal: None,
        priority: RequestPriority::Auto,
        window: None,
    };

    let debug_str = format!("{:?}", request);
    assert!(debug_str.contains("NetworkRequest"));
}

#[test]
fn test_network_request_clone() {
    // Given a network request
    // When cloned
    // Then the clone should have same values
    let url = Url::parse("https://example.com").unwrap();
    let request = NetworkRequest {
        url: url.clone(),
        method: HttpMethod::Put,
        headers: HeaderMap::new(),
        body: None,
        mode: RequestMode::SameOrigin,
        credentials: CredentialsMode::SameOrigin,
        cache: CacheMode::Reload,
        redirect: RedirectMode::Error,
        referrer: Some("https://referrer.com".to_string()),
        referrer_policy: ReferrerPolicy::Origin,
        integrity: None,
        keepalive: false,
        signal: None,
        priority: RequestPriority::Low,
        window: None,
    };

    let cloned = request.clone();
    assert_eq!(cloned.url, request.url);
    assert_eq!(cloned.method, request.method);
    assert_eq!(cloned.mode, request.mode);
}

#[test]
fn test_network_request_serde() {
    // Given a network request
    // When serialized to JSON
    // Then it should serialize successfully
    // And deserialization should restore the request
    let url = Url::parse("https://example.com").unwrap();
    let request = NetworkRequest {
        url,
        method: HttpMethod::Get,
        headers: HeaderMap::new(),
        body: None,
        mode: RequestMode::Cors,
        credentials: CredentialsMode::SameOrigin,
        cache: CacheMode::Default,
        redirect: RedirectMode::Follow,
        referrer: None,
        referrer_policy: ReferrerPolicy::NoReferrerWhenDowngrade,
        integrity: None,
        keepalive: false,
        signal: None,
        priority: RequestPriority::Auto,
        window: None,
    };

    let json = serde_json::to_string(&request).unwrap();
    let deserialized: NetworkRequest = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.method, request.method);
}
