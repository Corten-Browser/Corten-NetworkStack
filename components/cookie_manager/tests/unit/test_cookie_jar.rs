//! Unit tests for CookieJar
//!
//! These tests verify cookie jar creation and cookie matching logic.

use cookie_manager::CookieJar;
use cookie::Cookie;
use url::Url;

/// Given CookieJar::new is called
/// When checking the jar contents
/// Then it should be empty
#[test]
fn test_cookie_jar_new_creates_empty_jar() {
    // Given & When
    let jar = CookieJar::new();
    let url = Url::parse("https://example.com").unwrap();

    // Then
    let matches = jar.matches(&url);
    assert!(matches.is_empty(), "New jar should have no cookies");
}

/// Given a CookieJar
/// When a cookie is added
/// Then it should be stored in the jar
#[test]
fn test_cookie_jar_add_stores_cookie() {
    // Given
    let mut jar = CookieJar::new();
    let cookie = Cookie::new("name", "value");

    // When
    jar.add(cookie.clone());

    // Then - verify cookie was added
    let url = Url::parse("https://example.com").unwrap();
    let matches = jar.matches(&url);
    assert!(!matches.is_empty(), "Jar should contain the added cookie");
}

/// Given a CookieJar with multiple cookies
/// When matching against a URL
/// Then only cookies matching the URL should be returned
#[test]
fn test_cookie_jar_matches_returns_matching_cookies() {
    // Given
    let mut jar = CookieJar::new();

    let mut cookie1 = Cookie::new("session", "abc");
    cookie1.set_domain("example.com");
    cookie1.set_path("/");

    let mut cookie2 = Cookie::new("token", "xyz");
    cookie2.set_domain("other.com");
    cookie2.set_path("/");

    jar.add(cookie1);
    jar.add(cookie2);

    // When
    let url = Url::parse("https://example.com/path").unwrap();
    let matches = jar.matches(&url);

    // Then
    assert_eq!(matches.len(), 1, "Should match one cookie");
    assert_eq!(matches[0].name(), "session");
}

/// Given a CookieJar with cookies for specific domains
/// When matching against an unrelated URL
/// Then no cookies should be returned
#[test]
fn test_cookie_jar_matches_empty_for_no_match() {
    // Given
    let mut jar = CookieJar::new();

    let mut cookie = Cookie::new("session", "abc");
    cookie.set_domain("example.com");

    jar.add(cookie);

    // When
    let url = Url::parse("https://unrelated.com").unwrap();
    let matches = jar.matches(&url);

    // Then
    assert!(matches.is_empty(), "No cookies should match unrelated domain");
}

/// Given a CookieJar with a secure cookie
/// When matching against HTTP URL
/// Then the secure cookie should not be included
#[test]
fn test_cookie_jar_respects_secure_flag_in_matches() {
    // Given
    let mut jar = CookieJar::new();

    let mut cookie = Cookie::new("secure_cookie", "secret");
    cookie.set_secure(true);
    cookie.set_domain("example.com");

    jar.add(cookie);

    // When
    let http_url = Url::parse("http://example.com").unwrap();
    let https_url = Url::parse("https://example.com").unwrap();

    // Then
    let http_matches = jar.matches(&http_url);
    assert!(http_matches.is_empty(), "Secure cookie should not match HTTP");

    let https_matches = jar.matches(&https_url);
    assert_eq!(https_matches.len(), 1, "Secure cookie should match HTTPS");
}

/// Given a CookieJar with path-specific cookies
/// When matching against different paths
/// Then only cookies with matching paths should be returned
#[test]
fn test_cookie_jar_path_matching() {
    // Given
    let mut jar = CookieJar::new();

    let mut cookie1 = Cookie::new("root", "value1");
    cookie1.set_path("/");
    cookie1.set_domain("example.com");

    let mut cookie2 = Cookie::new("admin", "value2");
    cookie2.set_path("/admin");
    cookie2.set_domain("example.com");

    jar.add(cookie1);
    jar.add(cookie2);

    // When
    let root_url = Url::parse("https://example.com/").unwrap();
    let admin_url = Url::parse("https://example.com/admin/users").unwrap();

    // Then
    let root_matches = jar.matches(&root_url);
    assert_eq!(root_matches.len(), 1, "Root should only get root cookie");
    assert_eq!(root_matches[0].name(), "root");

    let admin_matches = jar.matches(&admin_url);
    assert_eq!(admin_matches.len(), 2, "Admin should get both cookies");
}

/// Given a CookieJar with an existing cookie
/// When adding a cookie with the same name and domain
/// Then the new cookie should replace the old one
#[test]
fn test_cookie_jar_add_updates_existing_cookie() {
    // Given
    let mut jar = CookieJar::new();

    let mut cookie1 = Cookie::new("session", "old_value");
    cookie1.set_domain("example.com");
    jar.add(cookie1);

    // When
    let mut cookie2 = Cookie::new("session", "new_value");
    cookie2.set_domain("example.com");
    jar.add(cookie2);

    // Then
    let url = Url::parse("https://example.com").unwrap();
    let matches = jar.matches(&url);
    assert_eq!(matches.len(), 1, "Should have only one cookie");
    assert_eq!(matches[0].value(), "new_value", "Should have updated value");
}

/// Given a CookieJar with an expired cookie
/// When matching against a URL
/// Then the expired cookie should not be included
#[test]
fn test_cookie_jar_ignores_expired_cookies() {
    // Given
    let mut jar = CookieJar::new();

    let mut cookie = Cookie::new("expired", "old");
    cookie.set_domain("example.com");
    cookie.set_expires(time::OffsetDateTime::now_utc() - time::Duration::days(1));

    jar.add(cookie);

    // When
    let url = Url::parse("https://example.com").unwrap();
    let matches = jar.matches(&url);

    // Then
    assert!(matches.is_empty(), "Expired cookies should not match");
}
