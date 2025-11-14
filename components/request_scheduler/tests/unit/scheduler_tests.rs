use request_scheduler::{RequestScheduler, RequestId, RequestPriority};
use network_types::{HttpMethod, NetworkRequest, RequestMode, CredentialsMode, CacheMode, RedirectMode, ReferrerPolicy};
use network_errors::NetworkError;
use url::Url;

/// Helper to create a minimal test request
fn create_test_request(url_str: &str) -> NetworkRequest {
    NetworkRequest {
        url: Url::parse(url_str).unwrap(),
        method: HttpMethod::Get,
        headers: http::HeaderMap::new(),
        body: None,
        mode: RequestMode::Navigate,
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
    }
}

#[test]
fn test_schedule_returns_unique_request_ids() {
    let mut scheduler = RequestScheduler::new(10);

    let req1 = create_test_request("https://example.com/1");
    let req2 = create_test_request("https://example.com/2");
    let req3 = create_test_request("https://example.com/3");

    let id1 = scheduler.schedule(req1, RequestPriority::High);
    let id2 = scheduler.schedule(req2, RequestPriority::High);
    let id3 = scheduler.schedule(req3, RequestPriority::High);

    // All IDs should be unique
    assert_ne!(id1, id2);
    assert_ne!(id2, id3);
    assert_ne!(id1, id3);
}

#[test]
fn test_high_priority_scheduled_before_medium() {
    let mut scheduler = RequestScheduler::new(10);

    let medium_req = create_test_request("https://example.com/medium");
    let high_req = create_test_request("https://example.com/high");

    // Schedule medium first, then high
    scheduler.schedule(medium_req, RequestPriority::Auto);  // Auto is medium
    scheduler.schedule(high_req, RequestPriority::High);

    // High priority should come out first
    let next = scheduler.next_request().expect("Should have request");
    assert_eq!(next.url.as_str(), "https://example.com/high");
}

#[test]
fn test_high_priority_scheduled_before_low() {
    let mut scheduler = RequestScheduler::new(10);

    let low_req = create_test_request("https://example.com/low");
    let high_req = create_test_request("https://example.com/high");

    // Schedule low first, then high
    scheduler.schedule(low_req, RequestPriority::Low);
    scheduler.schedule(high_req, RequestPriority::High);

    // High priority should come out first
    let next = scheduler.next_request().expect("Should have request");
    assert_eq!(next.url.as_str(), "https://example.com/high");
}

#[test]
fn test_medium_priority_scheduled_before_low() {
    let mut scheduler = RequestScheduler::new(10);

    let low_req = create_test_request("https://example.com/low");
    let medium_req = create_test_request("https://example.com/medium");

    // Schedule low first, then medium
    scheduler.schedule(low_req, RequestPriority::Low);
    scheduler.schedule(medium_req, RequestPriority::Auto);

    // Medium priority should come out first
    let next = scheduler.next_request().expect("Should have request");
    assert_eq!(next.url.as_str(), "https://example.com/medium");
}

#[test]
fn test_fifo_within_same_priority() {
    let mut scheduler = RequestScheduler::new(10);

    let req1 = create_test_request("https://example.com/1");
    let req2 = create_test_request("https://example.com/2");
    let req3 = create_test_request("https://example.com/3");

    // All same priority
    scheduler.schedule(req1, RequestPriority::High);
    scheduler.schedule(req2, RequestPriority::High);
    scheduler.schedule(req3, RequestPriority::High);

    // Should come out in FIFO order
    assert_eq!(scheduler.next_request().unwrap().url.as_str(), "https://example.com/1");
    assert_eq!(scheduler.next_request().unwrap().url.as_str(), "https://example.com/2");
    assert_eq!(scheduler.next_request().unwrap().url.as_str(), "https://example.com/3");
}

#[test]
fn test_next_request_returns_none_when_empty() {
    let mut scheduler = RequestScheduler::new(10);

    assert!(scheduler.next_request().is_none());
}

#[test]
fn test_next_request_returns_none_when_max_concurrent_reached() {
    let mut scheduler = RequestScheduler::new(2);  // Max 2 concurrent

    let req1 = create_test_request("https://example.com/1");
    let req2 = create_test_request("https://example.com/2");
    let req3 = create_test_request("https://example.com/3");

    scheduler.schedule(req1, RequestPriority::High);
    scheduler.schedule(req2, RequestPriority::High);
    scheduler.schedule(req3, RequestPriority::High);

    // First two should be returned
    assert!(scheduler.next_request().is_some());
    assert!(scheduler.next_request().is_some());

    // Third should be blocked by max concurrent limit
    assert!(scheduler.next_request().is_none());
}

#[test]
fn test_cancel_request_removes_pending_request() {
    let mut scheduler = RequestScheduler::new(10);

    let req = create_test_request("https://example.com/cancel");
    let id = scheduler.schedule(req, RequestPriority::High);

    // Cancel should succeed
    assert!(scheduler.cancel_request(id).is_ok());

    // Next request should be None (nothing left)
    assert!(scheduler.next_request().is_none());
}

#[test]
fn test_cancel_request_returns_error_for_nonexistent_id() {
    let mut scheduler = RequestScheduler::new(10);

    let result = scheduler.cancel_request(9999);
    assert!(result.is_err());
}

#[test]
fn test_cancel_active_request_frees_slot() {
    let mut scheduler = RequestScheduler::new(1);  // Max 1 concurrent

    let req1 = create_test_request("https://example.com/1");
    let req2 = create_test_request("https://example.com/2");

    let id1 = scheduler.schedule(req1, RequestPriority::High);
    scheduler.schedule(req2, RequestPriority::High);

    // Get first request (now active)
    scheduler.next_request().expect("Should have request");

    // Second request blocked by max concurrent
    assert!(scheduler.next_request().is_none());

    // Cancel active request
    scheduler.cancel_request(id1).expect("Should cancel");

    // Now second request can be retrieved
    assert!(scheduler.next_request().is_some());
}

#[test]
fn test_set_max_concurrent_updates_limit() {
    let mut scheduler = RequestScheduler::new(1);

    let req1 = create_test_request("https://example.com/1");
    let req2 = create_test_request("https://example.com/2");

    scheduler.schedule(req1, RequestPriority::High);
    scheduler.schedule(req2, RequestPriority::High);

    // Only one should be available with limit 1
    scheduler.next_request().expect("Should have request");
    assert!(scheduler.next_request().is_none());

    // Increase limit
    scheduler.set_max_concurrent(2);

    // Now second request should be available
    assert!(scheduler.next_request().is_some());
}

#[test]
fn test_fair_scheduling_low_priority_eventually_scheduled() {
    let mut scheduler = RequestScheduler::new(10);

    let low_req = create_test_request("https://example.com/low");
    let high_req1 = create_test_request("https://example.com/high1");
    let high_req2 = create_test_request("https://example.com/high2");

    // Schedule low first, then highs
    scheduler.schedule(low_req, RequestPriority::Low);
    scheduler.schedule(high_req1, RequestPriority::High);
    scheduler.schedule(high_req2, RequestPriority::High);

    // Get all requests
    let r1 = scheduler.next_request().unwrap();
    let r2 = scheduler.next_request().unwrap();
    let r3 = scheduler.next_request().unwrap();

    // Highs should come first
    assert_eq!(r1.url.as_str(), "https://example.com/high1");
    assert_eq!(r2.url.as_str(), "https://example.com/high2");

    // But low should still be scheduled eventually
    assert_eq!(r3.url.as_str(), "https://example.com/low");
}

#[test]
fn test_mixed_priority_ordering() {
    let mut scheduler = RequestScheduler::new(10);

    // Schedule in mixed order
    scheduler.schedule(create_test_request("https://example.com/low1"), RequestPriority::Low);
    scheduler.schedule(create_test_request("https://example.com/high1"), RequestPriority::High);
    scheduler.schedule(create_test_request("https://example.com/medium1"), RequestPriority::Auto);
    scheduler.schedule(create_test_request("https://example.com/high2"), RequestPriority::High);
    scheduler.schedule(create_test_request("https://example.com/low2"), RequestPriority::Low);
    scheduler.schedule(create_test_request("https://example.com/medium2"), RequestPriority::Auto);

    // Should come out: high1, high2, medium1, medium2, low1, low2
    assert_eq!(scheduler.next_request().unwrap().url.as_str(), "https://example.com/high1");
    assert_eq!(scheduler.next_request().unwrap().url.as_str(), "https://example.com/high2");
    assert_eq!(scheduler.next_request().unwrap().url.as_str(), "https://example.com/medium1");
    assert_eq!(scheduler.next_request().unwrap().url.as_str(), "https://example.com/medium2");
    assert_eq!(scheduler.next_request().unwrap().url.as_str(), "https://example.com/low1");
    assert_eq!(scheduler.next_request().unwrap().url.as_str(), "https://example.com/low2");
}
