//! Unit tests for CookieStore
//!
//! These tests verify cookie storage, retrieval, and policy enforcement.

use cookie_manager::{CookieJar, CookieStore};
use cookie::Cookie;
use network_errors::NetworkError;
use url::Url;

/// Given a new CookieStore is created
/// When checking its state
/// Then it should be empty with no cookies
#[test]
fn test_cookie_store_new_creates_empty_store() {
    // Given & When
    let store = CookieStore::new();

    // Then
    let url = Url::parse("https://example.com").unwrap();
    let cookies = store.get_cookies(&url);
    assert!(cookies.is_empty(), "New store should have no cookies");
}

/// Given a CookieStore
/// When a cookie is added for a specific domain
/// Then it should be retrievable for that domain
#[test]
fn test_add_cookie_stores_cookie_for_domain() {
    // Given
    let mut store = CookieStore::new();
    let url = Url::parse("https://example.com/path").unwrap();
    let cookie = Cookie::new("session", "abc123");

    // When
    let result = store.add_cookie(cookie.clone(), &url);

    // Then
    assert!(result.is_ok(), "Adding valid cookie should succeed");
    let retrieved = store.get_cookies(&url);
    assert_eq!(retrieved.len(), 1, "Should retrieve one cookie");
    assert_eq!(retrieved[0].name(), "session");
    assert_eq!(retrieved[0].value(), "abc123");
}

/// Given a CookieStore with cookies from multiple domains
/// When requesting cookies for a specific domain
/// Then only cookies matching that domain should be returned
#[test]
fn test_get_cookies_returns_only_matching_domain() {
    // Given
    let mut store = CookieStore::new();
    let url1 = Url::parse("https://example.com").unwrap();
    let url2 = Url::parse("https://other.com").unwrap();

    let cookie1 = Cookie::new("session", "abc");
    let cookie2 = Cookie::new("token", "xyz");

    store.add_cookie(cookie1, &url1).unwrap();
    store.add_cookie(cookie2, &url2).unwrap();

    // When
    let cookies_example = store.get_cookies(&url1);
    let cookies_other = store.get_cookies(&url2);

    // Then
    assert_eq!(cookies_example.len(), 1);
    assert_eq!(cookies_example[0].name(), "session");

    assert_eq!(cookies_other.len(), 1);
    assert_eq!(cookies_other[0].name(), "token");
}

/// Given a cookie with Secure flag set
/// When requesting it over HTTP (non-secure)
/// Then the cookie should not be returned
/// And when requesting over HTTPS it should be returned
#[test]
fn test_cookie_store_respects_secure_flag() {
    // Given
    let mut store = CookieStore::new();
    let https_url = Url::parse("https://example.com").unwrap();
    let http_url = Url::parse("http://example.com").unwrap();

    let mut cookie = Cookie::new("secure_session", "secret");
    cookie.set_secure(true);

    store.add_cookie(cookie, &https_url).unwrap();

    // When & Then
    let http_cookies = store.get_cookies(&http_url);
    assert!(http_cookies.is_empty(), "Secure cookie should not be sent over HTTP");

    let https_cookies = store.get_cookies(&https_url);
    assert_eq!(https_cookies.len(), 1, "Secure cookie should be sent over HTTPS");
}

/// Given a cookie with HttpOnly flag set
/// When the cookie is retrieved
/// Then it should have the HttpOnly flag preserved
#[test]
fn test_cookie_store_respects_httponly_flag() {
    // Given
    let mut store = CookieStore::new();
    let url = Url::parse("https://example.com").unwrap();

    let mut cookie = Cookie::new("auth", "token123");
    cookie.set_http_only(true);

    // When
    store.add_cookie(cookie, &url).unwrap();
    let retrieved = store.get_cookies(&url);

    // Then
    assert_eq!(retrieved.len(), 1);
    assert!(retrieved[0].http_only().unwrap_or(false), "HttpOnly flag should be preserved");
}

/// Given a cookie set for a specific path
/// When requesting from a different path
/// Then the cookie should only be returned for matching paths
#[test]
fn test_cookie_store_handles_path_matching() {
    // Given
    let mut store = CookieStore::new();
    let url = Url::parse("https://example.com/admin/dashboard").unwrap();

    let mut cookie = Cookie::new("admin_session", "xyz");
    cookie.set_path("/admin");

    store.add_cookie(cookie, &url).unwrap();

    // When & Then
    let admin_url = Url::parse("https://example.com/admin/users").unwrap();
    let admin_cookies = store.get_cookies(&admin_url);
    assert_eq!(admin_cookies.len(), 1, "Cookie should match /admin path");

    let root_url = Url::parse("https://example.com/").unwrap();
    let root_cookies = store.get_cookies(&root_url);
    assert!(root_cookies.is_empty(), "Cookie should not match root path");
}

/// Given a CookieStore with multiple cookies
/// When clear is called
/// Then all cookies should be removed
#[test]
fn test_cookie_store_clear_removes_all_cookies() {
    // Given
    let mut store = CookieStore::new();
    let url1 = Url::parse("https://example.com").unwrap();
    let url2 = Url::parse("https://other.com").unwrap();

    store.add_cookie(Cookie::new("cookie1", "val1"), &url1).unwrap();
    store.add_cookie(Cookie::new("cookie2", "val2"), &url2).unwrap();

    // When
    store.clear();

    // Then
    assert!(store.get_cookies(&url1).is_empty());
    assert!(store.get_cookies(&url2).is_empty());
}

/// Given a cookie with past expiration date
/// When adding it to the store
/// Then the cookie_store crate rejects it (security feature)
#[test]
fn test_expired_cookies_not_returned() {
    // Given
    let mut store = CookieStore::new();
    let url = Url::parse("https://example.com").unwrap();

    let mut cookie = Cookie::new("expired", "old");
    // Set expiration to past date
    cookie.set_expires(time::OffsetDateTime::now_utc() - time::Duration::days(1));

    // When - the cookie_store crate rejects expired cookies
    let result = store.add_cookie(cookie, &url);

    // Then - it should error because expired cookies are rejected
    assert!(result.is_err(), "Expired cookies should be rejected");
}

/// Given a CookieStore with an existing cookie
/// When adding a cookie with the same name for the same domain
/// Then the new cookie should replace the old one
#[test]
fn test_cookie_update_replaces_existing() {
    // Given
    let mut store = CookieStore::new();
    let url = Url::parse("https://example.com").unwrap();

    let cookie1 = Cookie::new("session", "old_value");
    store.add_cookie(cookie1, &url).unwrap();

    // When
    let cookie2 = Cookie::new("session", "new_value");
    store.add_cookie(cookie2, &url).unwrap();

    // Then
    let cookies = store.get_cookies(&url);
    assert_eq!(cookies.len(), 1, "Should only have one cookie");
    assert_eq!(cookies[0].value(), "new_value", "Should have updated value");
}

/// Given a cookie set with domain .example.com
/// When requesting from subdomain.example.com
/// Then the cookie should be available
#[test]
fn test_subdomain_cookie_matching() {
    // Given
    let mut store = CookieStore::new();
    let url = Url::parse("https://example.com").unwrap();

    let mut cookie = Cookie::new("shared", "value");
    cookie.set_domain(".example.com");

    store.add_cookie(cookie, &url).unwrap();

    // When
    let subdomain_url = Url::parse("https://sub.example.com").unwrap();
    let cookies = store.get_cookies(&subdomain_url);

    // Then
    assert_eq!(cookies.len(), 1, "Cookie should be available on subdomain");
}
