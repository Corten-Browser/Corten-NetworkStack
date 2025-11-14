//! Unit tests for cookie parsing
//!
//! These tests verify Set-Cookie header parsing and cookie attribute handling.

use cookie_manager::parse_set_cookie;

/// Given a simple Set-Cookie header "name=value"
/// When parsing the header
/// Then a cookie with correct name and value should be created
#[test]
fn test_parse_simple_cookie() {
    // Given
    let header = "session=abc123";

    // When
    let result = parse_set_cookie(header);

    // Then
    assert!(result.is_ok(), "Should parse simple cookie");
    let cookie = result.unwrap();
    assert_eq!(cookie.name(), "session");
    assert_eq!(cookie.value(), "abc123");
}

/// Given a Set-Cookie header with Domain attribute
/// When parsing the header
/// Then the cookie should have the domain set
#[test]
fn test_parse_cookie_with_domain() {
    // Given
    let header = "session=abc; Domain=example.com";

    // When
    let result = parse_set_cookie(header);

    // Then
    assert!(result.is_ok());
    let cookie = result.unwrap();
    assert_eq!(cookie.domain(), Some("example.com"));
}

/// Given a Set-Cookie header with Secure flag
/// When parsing the header
/// Then the cookie should have secure flag set to true
#[test]
fn test_parse_cookie_with_secure_flag() {
    // Given
    let header = "session=abc; Secure";

    // When
    let result = parse_set_cookie(header);

    // Then
    assert!(result.is_ok());
    let cookie = result.unwrap();
    assert!(cookie.secure().unwrap_or(false), "Secure flag should be set");
}

/// Given an empty Set-Cookie header
/// When parsing the header
/// Then it should return an error
#[test]
fn test_parse_empty_string_fails() {
    // Given
    let header = "";

    // When
    let result = parse_set_cookie(header);

    // Then
    assert!(result.is_err(), "Empty header should fail to parse");
}
