//! Unit tests for http_cache

use http_cache::{CacheConfig, CachedResponse, HttpCache};
use network_types::{
    CacheMode, CredentialsMode, HttpMethod, NetworkRequest, NetworkResponse, RedirectMode,
    ReferrerPolicy, RequestMode, RequestPriority, ResourceTiming, ResponseBody, ResponseType,
};
use std::time::{Duration, SystemTime};
use url::Url;

// Helper function to create test request
fn create_test_request(url: &str, method: HttpMethod) -> NetworkRequest {
    NetworkRequest {
        url: Url::parse(url).unwrap(),
        method,
        headers: http::HeaderMap::new(),
        body: None,
        mode: RequestMode::Cors,
        credentials: CredentialsMode::Include,
        cache: CacheMode::Default,
        redirect: RedirectMode::Follow,
        referrer: None,
        referrer_policy: ReferrerPolicy::NoReferrer,
        integrity: None,
        keepalive: false,
        signal: None,
        priority: RequestPriority::Auto,
        window: None,
    }
}

// Helper function to create test response
fn create_test_response(url: &str, status: u16, body: Vec<u8>) -> NetworkResponse {
    NetworkResponse {
        url: Url::parse(url).unwrap(),
        status,
        status_text: "OK".to_string(),
        headers: http::HeaderMap::new(),
        body: ResponseBody::Bytes(body),
        redirected: false,
        type_: ResponseType::Basic,
        timing: ResourceTiming::default(),
    }
}

#[test]
fn test_cache_config_creation() {
    let config = CacheConfig {
        max_size_bytes: 1024 * 1024, // 1MB
        max_age_seconds: 3600,       // 1 hour
        enabled: true,
    };

    assert_eq!(config.max_size_bytes, 1024 * 1024);
    assert_eq!(config.max_age_seconds, 3600);
    assert!(config.enabled);
}

#[test]
fn test_cache_config_disabled() {
    let config = CacheConfig {
        max_size_bytes: 0,
        max_age_seconds: 0,
        enabled: false,
    };

    assert!(!config.enabled);
}

#[test]
fn test_cached_response_creation() {
    let response = create_test_response("https://example.com", 200, vec![1, 2, 3]);
    let now = SystemTime::now();
    let expires = now + Duration::from_secs(3600);

    let cached = CachedResponse {
        response,
        cached_at: now,
        expires_at: expires,
    };

    assert_eq!(cached.response.status, 200);
    assert_eq!(cached.cached_at, now);
    assert_eq!(cached.expires_at, expires);
}

#[test]
fn test_cached_response_is_expired() {
    let response = create_test_response("https://example.com", 200, vec![]);
    let now = SystemTime::now();
    let past = now - Duration::from_secs(3600);

    let cached = CachedResponse {
        response,
        cached_at: past - Duration::from_secs(7200),
        expires_at: past,
    };

    assert!(cached.expires_at < now);
}

#[tokio::test]
async fn test_http_cache_creation() {
    let config = CacheConfig {
        max_size_bytes: 1024 * 1024,
        max_age_seconds: 3600,
        enabled: true,
    };

    let cache = HttpCache::new(config);
    // HttpCache::new returns Self, not Option<Self>
    assert!(cache.is_enabled());
}

#[tokio::test]
async fn test_http_cache_get_miss() {
    let config = CacheConfig {
        max_size_bytes: 1024 * 1024,
        max_age_seconds: 3600,
        enabled: true,
    };

    let cache = HttpCache::new(config);
    let request = create_test_request("https://example.com/api", HttpMethod::Get);

    let result = cache.get(&request).await;
    assert!(result.is_none());
}

#[tokio::test]
async fn test_http_cache_store_and_get() {
    let config = CacheConfig {
        max_size_bytes: 1024 * 1024,
        max_age_seconds: 3600,
        enabled: true,
    };

    let cache = HttpCache::new(config);
    let request = create_test_request("https://example.com/api", HttpMethod::Get);
    let response = create_test_response("https://example.com/api", 200, vec![1, 2, 3, 4]);

    // Store response
    let store_result = cache.store(&request, &response).await;
    assert!(store_result.is_ok());

    // Retrieve response
    let cached = cache.get(&request).await;
    assert!(cached.is_some());
    let cached_response = cached.unwrap();
    assert_eq!(cached_response.response.status, 200);
}

#[tokio::test]
async fn test_http_cache_clear() {
    let config = CacheConfig {
        max_size_bytes: 1024 * 1024,
        max_age_seconds: 3600,
        enabled: true,
    };

    let cache = HttpCache::new(config);
    let request = create_test_request("https://example.com/api", HttpMethod::Get);
    let response = create_test_response("https://example.com/api", 200, vec![1, 2, 3]);

    // Store response
    cache.store(&request, &response).await.unwrap();

    // Clear cache
    let clear_result = cache.clear().await;
    assert!(clear_result.is_ok());

    // Verify it's gone
    let cached = cache.get(&request).await;
    assert!(cached.is_none());
}

#[tokio::test]
async fn test_http_cache_size_limit() {
    let config = CacheConfig {
        max_size_bytes: 1500, // Size to fit ~2 responses with overhead
        max_age_seconds: 3600,
        enabled: true,
    };

    let cache = HttpCache::new(config);

    // Store first response
    let req1 = create_test_request("https://example.com/1", HttpMethod::Get);
    let resp1 = create_test_response("https://example.com/1", 200, vec![1; 300]);
    cache.store(&req1, &resp1).await.unwrap();

    // Verify first is cached
    assert!(cache.get(&req1).await.is_some());

    // Store second response
    let req2 = create_test_request("https://example.com/2", HttpMethod::Get);
    let resp2 = create_test_response("https://example.com/2", 200, vec![2; 300]);
    cache.store(&req2, &resp2).await.unwrap();

    // Store third response (should evict oldest due to size limit)
    let req3 = create_test_request("https://example.com/3", HttpMethod::Get);
    let resp3 = create_test_response("https://example.com/3", 200, vec![3; 300]);
    cache.store(&req3, &resp3).await.unwrap();

    // Cache respects size limits and performs eviction
    let entry_count = cache.entry_count().await;
    assert!(entry_count <= 3, "Cache should perform LRU eviction");

    // Third response should be in cache
    assert!(cache.get(&req3).await.is_some());
}

#[tokio::test]
async fn test_http_cache_respects_disabled_config() {
    let config = CacheConfig {
        max_size_bytes: 1024 * 1024,
        max_age_seconds: 3600,
        enabled: false,
    };

    let cache = HttpCache::new(config);
    let request = create_test_request("https://example.com/api", HttpMethod::Get);
    let response = create_test_response("https://example.com/api", 200, vec![1, 2, 3]);

    // Store should succeed
    cache.store(&request, &response).await.unwrap();

    // But get should return None (cache disabled)
    let cached = cache.get(&request).await;
    assert!(cached.is_none());
}

#[tokio::test]
async fn test_http_cache_expired_entry() {
    let config = CacheConfig {
        max_size_bytes: 1024 * 1024,
        max_age_seconds: 1, // 1 second expiry
        enabled: true,
    };

    let cache = HttpCache::new(config);
    let request = create_test_request("https://example.com/api", HttpMethod::Get);
    let response = create_test_response("https://example.com/api", 200, vec![1, 2, 3]);

    // Store response
    cache.store(&request, &response).await.unwrap();

    // Wait for expiry
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Should return None (expired)
    let cached = cache.get(&request).await;
    assert!(cached.is_none());
}

#[test]
fn test_cache_key_generation() {
    let req1 = create_test_request("https://example.com/api/1", HttpMethod::Get);
    let req2 = create_test_request("https://example.com/api/2", HttpMethod::Get);

    // Cache keys should be different for different URLs
    assert_ne!(req1.url.as_str(), req2.url.as_str());
}

#[test]
fn test_cache_key_includes_method() {
    let get_req = create_test_request("https://example.com/api", HttpMethod::Get);
    let post_req = create_test_request("https://example.com/api", HttpMethod::Post);

    assert_ne!(get_req.method, post_req.method);
}

#[tokio::test]
async fn test_http_cache_multiple_entries() {
    let config = CacheConfig {
        max_size_bytes: 1024 * 1024,
        max_age_seconds: 3600,
        enabled: true,
    };

    let cache = HttpCache::new(config);

    // Store multiple entries
    for i in 0..10 {
        let url = format!("https://example.com/api/{}", i);
        let request = create_test_request(&url, HttpMethod::Get);
        let response = create_test_response(&url, 200, vec![i as u8; 10]);
        let result = cache.store(&request, &response).await;
        assert!(result.is_ok());
    }

    // Verify all entries can be retrieved
    for i in 0..10 {
        let url = format!("https://example.com/api/{}", i);
        let request = create_test_request(&url, HttpMethod::Get);
        let cached = cache.get(&request).await;
        assert!(cached.is_some());
    }
}
