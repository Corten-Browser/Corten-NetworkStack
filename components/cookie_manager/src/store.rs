//! Cookie storage implementation
//!
//! Provides CookieStore for storing and retrieving cookies with policy enforcement.

use cookie::Cookie;
use cookie_store::CookieStore as ExternalCookieStore;
use network_errors::NetworkError;
use url::Url;

/// Cookie storage structure
///
/// Manages cookies per domain with support for Secure, HttpOnly, Path, and Domain matching.
pub struct CookieStore {
    inner: ExternalCookieStore,
}

impl CookieStore {
    /// Create a new empty CookieStore
    ///
    /// # Examples
    ///
    /// ```
    /// use cookie_manager::CookieStore;
    ///
    /// let store = CookieStore::new();
    /// ```
    pub fn new() -> Self {
        Self {
            inner: ExternalCookieStore::default(),
        }
    }

    /// Add a cookie to the store for a specific URL
    ///
    /// The cookie will be associated with the domain and path from the URL.
    /// Existing cookies with the same name, domain, and path will be replaced.
    ///
    /// # Arguments
    ///
    /// * `cookie` - The cookie to store
    /// * `url` - The URL context for the cookie
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the cookie was stored successfully
    /// * `Err(NetworkError)` if the cookie could not be stored
    ///
    /// # Examples
    ///
    /// ```
    /// use cookie_manager::CookieStore;
    /// use cookie::Cookie;
    /// use url::Url;
    ///
    /// let mut store = CookieStore::new();
    /// let url = Url::parse("https://example.com").unwrap();
    /// let cookie = Cookie::new("session", "abc123");
    ///
    /// store.add_cookie(cookie, &url).unwrap();
    /// ```
    pub fn add_cookie(&mut self, cookie: Cookie<'static>, url: &Url) -> Result<(), NetworkError> {
        // Convert cookie::Cookie to cookie_store::Cookie by serializing to a string
        // and parsing it back with the cookie_store crate
        let cookie_header = cookie.to_string();
        let owned_str: String = cookie_header;
        let leaked_str: &'static str = Box::leak(owned_str.into_boxed_str());

        let store_cookie = cookie_store::Cookie::parse(leaked_str, url)
            .map_err(|e| NetworkError::Other(format!("Failed to convert cookie: {}", e)))?;

        self.inner
            .insert(store_cookie, url)
            .map(|_| ())
            .map_err(|e| NetworkError::Other(format!("Failed to add cookie: {}", e)))
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
    /// use cookie_manager::CookieStore;
    /// use cookie::Cookie;
    /// use url::Url;
    ///
    /// let mut store = CookieStore::new();
    /// let url = Url::parse("https://example.com").unwrap();
    /// let cookie = Cookie::new("session", "abc123");
    ///
    /// store.add_cookie(cookie, &url).unwrap();
    /// let cookies = store.get_cookies(&url);
    ///
    /// assert_eq!(cookies.len(), 1);
    /// ```
    pub fn get_cookies(&self, url: &Url) -> Vec<Cookie<'static>> {
        self.inner
            .matches(url)
            .into_iter()
            .filter_map(|c| {
                // Convert cookie_store::Cookie to cookie::Cookie
                Cookie::parse(c.to_string()).ok().map(|c| c.into_owned())
            })
            .collect()
    }

    /// Clear all cookies from the store
    ///
    /// Removes all stored cookies.
    ///
    /// # Examples
    ///
    /// ```
    /// use cookie_manager::CookieStore;
    /// use cookie::Cookie;
    /// use url::Url;
    ///
    /// let mut store = CookieStore::new();
    /// let url = Url::parse("https://example.com").unwrap();
    ///
    /// store.add_cookie(Cookie::new("session", "abc"), &url).unwrap();
    /// store.clear();
    ///
    /// assert!(store.get_cookies(&url).is_empty());
    /// ```
    pub fn clear(&mut self) {
        self.inner.clear();
    }
}

impl Default for CookieStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_store_creation() {
        let store = CookieStore::new();
        let url = Url::parse("https://example.com").unwrap();
        assert!(store.get_cookies(&url).is_empty());
    }

    #[test]
    fn test_add_and_get_cookie() {
        let mut store = CookieStore::new();
        let url = Url::parse("https://example.com").unwrap();
        let cookie = Cookie::new("test", "value");

        store.add_cookie(cookie, &url).unwrap();
        let cookies = store.get_cookies(&url);

        assert_eq!(cookies.len(), 1);
        assert_eq!(cookies[0].name(), "test");
    }
}
