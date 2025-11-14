//! Cookie parsing implementation
//!
//! Provides functions for parsing Set-Cookie headers.

use cookie::Cookie;
use network_errors::NetworkError;

/// Parse a Set-Cookie header value into a Cookie
///
/// Parses HTTP Set-Cookie header strings and creates Cookie instances with
/// appropriate attributes (Domain, Path, Secure, HttpOnly, SameSite, etc.).
///
/// # Arguments
///
/// * `header` - The Set-Cookie header value to parse
///
/// # Returns
///
/// * `Ok(Cookie)` if parsing succeeded
/// * `Err(NetworkError)` if parsing failed
///
/// # Examples
///
/// ```
/// use cookie_manager::parse_set_cookie;
///
/// let cookie = parse_set_cookie("session=abc123; Domain=example.com; Secure").unwrap();
/// assert_eq!(cookie.name(), "session");
/// assert_eq!(cookie.value(), "abc123");
/// assert_eq!(cookie.domain(), Some("example.com"));
/// assert!(cookie.secure().unwrap_or(false));
/// ```
///
/// # Supported Attributes
///
/// - `Domain` - Cookie domain
/// - `Path` - Cookie path
/// - `Secure` - Secure flag
/// - `HttpOnly` - HttpOnly flag
/// - `SameSite` - SameSite attribute (Strict, Lax, None)
/// - `Max-Age` - Maximum age in seconds
/// - `Expires` - Expiration date
pub fn parse_set_cookie(header: &str) -> Result<Cookie<'static>, NetworkError> {
    if header.is_empty() {
        return Err(NetworkError::Other("Empty Set-Cookie header".to_string()));
    }

    // Use the cookie crate's built-in parsing
    Cookie::parse(header)
        .map(|c| c.into_owned())
        .map_err(|e| NetworkError::Other(format!("Failed to parse cookie: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple() {
        let result = parse_set_cookie("name=value");
        assert!(result.is_ok());

        let cookie = result.unwrap();
        assert_eq!(cookie.name(), "name");
        assert_eq!(cookie.value(), "value");
    }

    #[test]
    fn test_parse_with_attributes() {
        let result = parse_set_cookie("session=abc; Domain=example.com; Secure; HttpOnly");
        assert!(result.is_ok());

        let cookie = result.unwrap();
        assert_eq!(cookie.name(), "session");
        assert_eq!(cookie.value(), "abc");
        assert_eq!(cookie.domain(), Some("example.com"));
        assert!(cookie.secure().unwrap_or(false));
        assert!(cookie.http_only().unwrap_or(false));
    }

    #[test]
    fn test_parse_empty_fails() {
        let result = parse_set_cookie("");
        assert!(result.is_err());
    }
}
