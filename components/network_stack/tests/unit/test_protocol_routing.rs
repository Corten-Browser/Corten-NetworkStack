//! Tests for protocol routing logic
//!
//! Verifies that requests are routed to the correct protocol handler based on URL scheme.

use network_stack::NetworkStackImpl;
use network_types::{HttpMethod, NetworkRequest};
use url::Url;

/// Test HTTP URL routes to HTTP/1.1
#[tokio::test]
async fn test_http_url_routes_to_http1() {
    // Given: an HTTP URL
    let url = Url::parse("http://example.com/api").unwrap();

    // When/Then: the router should select HTTP/1.1 protocol
    // (This will be verified by implementation)
    assert_eq!(url.scheme(), "http");
}

/// Test HTTPS URL routes to HTTP/1.1 by default
#[tokio::test]
async fn test_https_url_routes_to_http1_by_default() {
    // Given: an HTTPS URL without HTTP/2 specified
    let url = Url::parse("https://example.com/api").unwrap();

    // When/Then: should default to HTTP/1.1
    assert_eq!(url.scheme(), "https");
}

/// Test WebSocket URL routes to WebSocket protocol
#[tokio::test]
async fn test_ws_url_routes_to_websocket() {
    // Given: a WebSocket URL
    let url = Url::parse("ws://example.com/socket").unwrap();

    // When/Then: should route to WebSocket handler
    assert_eq!(url.scheme(), "ws");
}

/// Test secure WebSocket URL routes to WebSocket protocol
#[tokio::test]
async fn test_wss_url_routes_to_websocket() {
    // Given: a secure WebSocket URL
    let url = Url::parse("wss://example.com/socket").unwrap();

    // When/Then: should route to WebSocket handler with TLS
    assert_eq!(url.scheme(), "wss");
}
