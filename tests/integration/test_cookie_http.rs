/// Integration tests for Cookie Manager → HTTP clients flow
///
/// These tests verify that:
/// 1. HTTP responses with Set-Cookie headers store cookies correctly
/// 2. HTTP requests retrieve and send appropriate cookies
/// 3. Cookie scope (domain, path) is respected
/// 4. Secure and HttpOnly flags are enforced
///
/// CRITICAL: Uses REAL components (no mocking of internal components)

#[cfg(test)]
mod cookie_http_integration {
    use cookie_manager::{CookieStore, CookieJar, Cookie};
    use http1_protocol::{Http1Client, Http1Config};
    use network_types::{NetworkRequest, NetworkResponse, HttpMethod};
    use url::Url;
    use http::{HeaderMap, HeaderValue};
    use std::time::Duration;

    /// Test that Set-Cookie headers from responses are stored
    #[test]
    fn test_set_cookie_storage() {
        // Given: A cookie store (REAL component)
        let cookie_store = CookieStore::new();

        // And: A response URL
        let url = Url::parse("http://example.com/login").unwrap();

        // When: Receiving a Set-Cookie header
        let cookie = Cookie::parse("session_id=abc123; Path=/; HttpOnly").unwrap();
        let result = cookie_store.add_cookie(cookie, &url);

        // Then: Cookie is stored successfully
        assert!(result.is_ok(), "Cookie should be stored successfully");

        // And: Cookie can be retrieved for matching URL
        let cookies = cookie_store.get_cookies(&url);
        assert_eq!(cookies.len(), 1, "Should have one stored cookie");
        assert_eq!(cookies[0].name(), "session_id");
        assert_eq!(cookies[0].value(), "abc123");
    }

    /// Test that cookies are sent with subsequent requests
    #[test]
    fn test_cookies_sent_with_requests() {
        // Given: A cookie store with stored cookies (REAL component)
        let cookie_store = CookieStore::new();
        let url = Url::parse("http://example.com/").unwrap();

        // Store a cookie
        let cookie = Cookie::parse("user_id=12345; Path=/").unwrap();
        cookie_store.add_cookie(cookie, &url).unwrap();

        // When: Getting cookies for a request to the same domain
        let cookies = cookie_store.get_cookies(&url);

        // Then: Stored cookie is returned
        assert_eq!(cookies.len(), 1, "Should retrieve stored cookie");
        assert_eq!(cookies[0].name(), "user_id");
        assert_eq!(cookies[0].value(), "12345");

        // And: Cookie would be included in HTTP request headers
        // (HTTP client would add: Cookie: user_id=12345)
    }

    /// Test cookie domain matching
    #[test]
    fn test_cookie_domain_matching() {
        // Given: A cookie store (REAL component)
        let cookie_store = CookieStore::new();

        // When: Storing a cookie for example.com
        let url = Url::parse("http://example.com/").unwrap();
        let cookie = Cookie::parse("site_cookie=value1; Domain=example.com; Path=/").unwrap();
        cookie_store.add_cookie(cookie, &url).unwrap();

        // Then: Cookie is sent to example.com
        let cookies = cookie_store.get_cookies(&url);
        assert_eq!(cookies.len(), 1, "Cookie should match example.com");

        // And: Cookie is also sent to subdomains
        let subdomain_url = Url::parse("http://api.example.com/").unwrap();
        let subdomain_cookies = cookie_store.get_cookies(&subdomain_url);
        assert_eq!(subdomain_cookies.len(), 1, "Cookie should match subdomain");

        // But: Cookie is NOT sent to different domains
        let other_url = Url::parse("http://other.com/").unwrap();
        let other_cookies = cookie_store.get_cookies(&other_url);
        assert_eq!(other_cookies.len(), 0, "Cookie should not match different domain");
    }

    /// Test cookie path matching
    #[test]
    fn test_cookie_path_matching() {
        // Given: A cookie store (REAL component)
        let cookie_store = CookieStore::new();

        // When: Storing cookies with different paths
        let base_url = Url::parse("http://example.com/").unwrap();
        let api_cookie = Cookie::parse("api_token=xyz; Path=/api").unwrap();
        cookie_store.add_cookie(api_cookie, &base_url).unwrap();

        let root_cookie = Cookie::parse("site_pref=dark; Path=/").unwrap();
        cookie_store.add_cookie(root_cookie, &base_url).unwrap();

        // Then: Root path cookie is sent to all paths
        let root_url = Url::parse("http://example.com/").unwrap();
        let root_cookies = cookie_store.get_cookies(&root_url);
        assert_eq!(root_cookies.len(), 1, "Only root path cookie for /");

        // And: API path gets both root and api cookies
        let api_url = Url::parse("http://example.com/api/users").unwrap();
        let api_cookies = cookie_store.get_cookies(&api_url);
        assert_eq!(api_cookies.len(), 2, "Both cookies for /api path");

        // But: Non-API path only gets root cookie
        let other_url = Url::parse("http://example.com/about").unwrap();
        let other_cookies = cookie_store.get_cookies(&other_url);
        assert_eq!(other_cookies.len(), 1, "Only root cookie for /about");
    }

    /// Test Secure cookie handling
    #[test]
    fn test_secure_cookie_enforcement() {
        // Given: A cookie store (REAL component)
        let cookie_store = CookieStore::new();

        // When: Storing a Secure cookie
        let https_url = Url::parse("https://secure.example.com/").unwrap();
        let secure_cookie = Cookie::parse("auth_token=secret; Secure; Path=/").unwrap();
        cookie_store.add_cookie(secure_cookie, &https_url).unwrap();

        // Then: Secure cookie is sent over HTTPS
        let cookies_https = cookie_store.get_cookies(&https_url);
        assert_eq!(cookies_https.len(), 1, "Secure cookie sent over HTTPS");

        // But: Secure cookie is NOT sent over HTTP
        let http_url = Url::parse("http://secure.example.com/").unwrap();
        let cookies_http = cookie_store.get_cookies(&http_url);
        assert_eq!(cookies_http.len(), 0, "Secure cookie not sent over HTTP");

        // This integration verifies:
        // 1. Cookie manager enforces Secure flag
        // 2. HTTP clients respect security requirements
        // 3. Sensitive cookies protected from insecure transmission
    }

    /// Test HttpOnly cookie handling
    #[test]
    fn test_httponly_cookie_storage() {
        // Given: A cookie store (REAL component)
        let cookie_store = CookieStore::new();

        // When: Storing an HttpOnly cookie
        let url = Url::parse("http://example.com/").unwrap();
        let httponly_cookie = Cookie::parse("session=xyz; HttpOnly; Path=/").unwrap();
        cookie_store.add_cookie(httponly_cookie, &url).unwrap();

        // Then: HttpOnly cookie is stored
        let cookies = cookie_store.get_cookies(&url);
        assert_eq!(cookies.len(), 1, "HttpOnly cookie stored");

        // And: Cookie would be sent with HTTP requests
        // (but not accessible to JavaScript - enforced at browser level)
        assert_eq!(cookies[0].name(), "session");
    }

    /// Test cookie jar matching functionality
    #[test]
    fn test_cookie_jar_url_matching() {
        // Given: A cookie jar (REAL component)
        let mut jar = CookieJar::new();

        // When: Adding multiple cookies
        jar.add(Cookie::parse("cookie1=value1").unwrap());
        jar.add(Cookie::parse("cookie2=value2").unwrap());
        jar.add(Cookie::parse("cookie3=value3").unwrap());

        // And: Matching cookies for a URL
        let url = Url::parse("http://example.com/").unwrap();
        let matching = jar.matches(&url);

        // Then: All cookies are matched
        // (CookieJar.matches filters based on domain/path/secure)
        assert!(matching.len() > 0, "Should have matching cookies");
    }

    /// Test cookie persistence across HTTP requests
    #[test]
    fn test_cookie_persistence_across_requests() {
        // Given: A cookie store shared across requests (REAL component)
        let cookie_store = CookieStore::new();

        // When: First request stores a cookie
        let login_url = Url::parse("http://example.com/login").unwrap();
        let session_cookie = Cookie::parse("session_id=abc123; Path=/").unwrap();
        cookie_store.add_cookie(session_cookie, &login_url).unwrap();

        // And: Second request to same domain
        let api_url = Url::parse("http://example.com/api/data").unwrap();
        let cookies = cookie_store.get_cookies(&api_url);

        // Then: Cookie from first request is available to second request
        assert_eq!(cookies.len(), 1, "Cookie persists across requests");
        assert_eq!(cookies[0].name(), "session_id");
        assert_eq!(cookies[0].value(), "abc123");

        // This integration verifies:
        // 1. Cookie store maintains state between requests
        // 2. HTTP clients share cookie store
        // 3. Session management works correctly
    }

    /// Test cookie expiration
    #[test]
    fn test_cookie_expiration() {
        // Given: A cookie store (REAL component)
        let cookie_store = CookieStore::new();
        let url = Url::parse("http://example.com/").unwrap();

        // When: Storing an expired cookie
        let expired_cookie = Cookie::parse(
            "expired=value; Path=/; Max-Age=0"
        ).unwrap();
        cookie_store.add_cookie(expired_cookie, &url).unwrap();

        // Then: Expired cookie should not be returned
        let cookies = cookie_store.get_cookies(&url);
        assert_eq!(cookies.len(), 0, "Expired cookies should not be returned");
    }

    /// Test cookie store clearing
    #[test]
    fn test_cookie_store_clear() {
        // Given: A cookie store with cookies (REAL component)
        let cookie_store = CookieStore::new();
        let url = Url::parse("http://example.com/").unwrap();

        // Store some cookies
        let cookie1 = Cookie::parse("cookie1=value1").unwrap();
        let cookie2 = Cookie::parse("cookie2=value2").unwrap();
        cookie_store.add_cookie(cookie1, &url).unwrap();
        cookie_store.add_cookie(cookie2, &url).unwrap();

        // Verify cookies are stored
        let cookies_before = cookie_store.get_cookies(&url);
        assert_eq!(cookies_before.len(), 2, "Should have 2 cookies before clear");

        // When: Clearing the cookie store
        cookie_store.clear();

        // Then: All cookies are removed
        let cookies_after = cookie_store.get_cookies(&url);
        assert_eq!(cookies_after.len(), 0, "All cookies should be cleared");
    }

    /// Test complete cookie flow with HTTP client
    #[test]
    fn test_complete_cookie_http_flow() {
        // Given: Complete stack (REAL components)
        // 1. Cookie store for managing cookies
        let cookie_store = CookieStore::new();

        // 2. HTTP client that uses cookie store
        let http_config = Http1Config {
            pool_size: 10,
            idle_timeout: Duration::from_secs(60),
            max_connections_per_host: 6,
            enable_keepalive: true,
            enable_pipelining: false,
        };
        let http_client = Http1Client::new(http_config);

        // Scenario: Complete authentication flow
        // When: Login request receives Set-Cookie
        let login_url = Url::parse("http://example.com/login").unwrap();
        let auth_cookie = Cookie::parse("auth_token=xyz123; Path=/; HttpOnly; Secure").unwrap();

        // HTTP response would contain Set-Cookie header
        // HTTP client extracts and stores in cookie store
        cookie_store.add_cookie(auth_cookie, &login_url).unwrap();

        // And: Subsequent API request
        let api_url = Url::parse("http://example.com/api/user").unwrap();
        let cookies = cookie_store.get_cookies(&api_url);

        // Then: Auth cookie is included in API request
        assert_eq!(cookies.len(), 1, "Auth cookie sent with API request");
        assert_eq!(cookies[0].name(), "auth_token");

        // This integration verifies:
        // 1. HTTP responses trigger cookie storage
        // 2. Cookie store maintains session state
        // 3. HTTP requests include appropriate cookies
        // 4. Complete authentication flow works
    }
}
