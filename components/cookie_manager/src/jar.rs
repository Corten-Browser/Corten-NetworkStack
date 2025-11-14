//! Cookie jar implementation
//!
//! Provides CookieJar for organizing and matching cookies.

use cookie::Cookie;
use std::collections::HashMap;
use time::OffsetDateTime;
use url::Url;

/// Cookie jar for organizing cookies
///
/// A lightweight container for cookies that supports matching cookies against URLs.
pub struct CookieJar {
    cookies: HashMap<String, Cookie<'static>>,
}

impl CookieJar {
    /// Create a new empty CookieJar
    ///
    /// # Examples
    ///
    /// ```
    /// use cookie_manager::CookieJar;
    ///
    /// let jar = CookieJar::new();
    /// ```
    pub fn new() -> Self {
        Self {
            cookies: HashMap::new(),
        }
    }

    /// Add a cookie to the jar
    ///
    /// If a cookie with the same name and domain already exists, it will be replaced.
    ///
    /// # Arguments
    ///
    /// * `cookie` - The cookie to add
    ///
    /// # Examples
    ///
    /// ```
    /// use cookie_manager::CookieJar;
    /// use cookie::Cookie;
    ///
    /// let mut jar = CookieJar::new();
    /// let cookie = Cookie::new("session", "abc123");
    ///
    /// jar.add(cookie);
    /// ```
    pub fn add(&mut self, cookie: Cookie<'static>) {
        let key = Self::cookie_key(&cookie);
        self.cookies.insert(key, cookie);
    }

    /// Get all cookies that match the given URL
    ///
    /// Returns cookies that:
    /// - Match the domain (exact or subdomain match)
    /// - Match the path (exact or parent path match)
    /// - Respect Secure flag (only returned for HTTPS URLs)
    /// - Are not expired
    ///
    /// # Arguments
    ///
    /// * `url` - The URL to match cookies against
    ///
    /// # Returns
    ///
    /// A vector of cookies matching the URL
    ///
    /// # Examples
    ///
    /// ```
    /// use cookie_manager::CookieJar;
    /// use cookie::Cookie;
    /// use url::Url;
    ///
    /// let mut jar = CookieJar::new();
    /// let mut cookie = Cookie::new("session", "abc123");
    /// cookie.set_domain("example.com");
    ///
    /// jar.add(cookie);
    ///
    /// let url = Url::parse("https://example.com").unwrap();
    /// let matches = jar.matches(&url);
    ///
    /// assert_eq!(matches.len(), 1);
    /// ```
    pub fn matches(&self, url: &Url) -> Vec<Cookie<'static>> {
        let url_host = url.host_str().unwrap_or("");
        let url_path = url.path();
        let is_https = url.scheme() == "https";

        self.cookies
            .values()
            .filter(|cookie| {
                // Check if cookie is expired
                if Self::is_expired(cookie) {
                    return false;
                }

                // Check secure flag
                if cookie.secure().unwrap_or(false) && !is_https {
                    return false;
                }

                // Check domain match
                if !Self::domain_matches(cookie, url_host) {
                    return false;
                }

                // Check path match
                if !Self::path_matches(cookie, url_path) {
                    return false;
                }

                true
            })
            .cloned()
            .collect()
    }

    /// Check if a cookie is expired
    fn is_expired(cookie: &Cookie<'_>) -> bool {
        if let Some(expires) = cookie.expires() {
            let now = OffsetDateTime::now_utc();
            match expires {
                cookie::Expiration::DateTime(dt) => dt < now,
                cookie::Expiration::Session => false,
            }
        } else {
            false
        }
    }

    /// Check if cookie domain matches the URL host
    fn domain_matches(cookie: &Cookie<'_>, host: &str) -> bool {
        if let Some(domain) = cookie.domain() {
            // Handle leading dot (e.g., .example.com)
            let cookie_domain = domain.trim_start_matches('.');

            // Exact match
            if host == cookie_domain {
                return true;
            }

            // Subdomain match (e.g., sub.example.com matches .example.com)
            if domain.starts_with('.') && host.ends_with(cookie_domain) {
                return true;
            }

            // Subdomain match without leading dot
            if host.ends_with(&format!(".{}", cookie_domain)) {
                return true;
            }

            false
        } else {
            // No domain set - match any (though this shouldn't happen in practice)
            true
        }
    }

    /// Check if cookie path matches the URL path
    fn path_matches(cookie: &Cookie<'_>, url_path: &str) -> bool {
        if let Some(cookie_path) = cookie.path() {
            // Exact match
            if url_path == cookie_path {
                return true;
            }

            // Path prefix match (e.g., /admin matches /admin/users)
            if url_path.starts_with(cookie_path) {
                // Ensure it's a proper path boundary
                if cookie_path.ends_with('/') || url_path.len() == cookie_path.len() {
                    return true;
                }
                if url_path.chars().nth(cookie_path.len()) == Some('/') {
                    return true;
                }
            }

            false
        } else {
            // No path set - match all paths
            true
        }
    }

    /// Generate a unique key for a cookie based on name and domain
    fn cookie_key(cookie: &Cookie<'_>) -> String {
        format!(
            "{}:{}",
            cookie.name(),
            cookie.domain().unwrap_or("default")
        )
    }
}

impl Default for CookieJar {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jar_creation() {
        let jar = CookieJar::new();
        let url = Url::parse("https://example.com").unwrap();
        assert!(jar.matches(&url).is_empty());
    }

    #[test]
    fn test_add_and_match() {
        let mut jar = CookieJar::new();
        let mut cookie = Cookie::new("test", "value");
        cookie.set_domain("example.com");

        jar.add(cookie);

        let url = Url::parse("https://example.com").unwrap();
        let matches = jar.matches(&url);

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].name(), "test");
    }
}
