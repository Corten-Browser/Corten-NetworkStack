//! Proxy authentication support
//!
//! Provides authentication mechanisms for proxy connections.

use base64::{engine::general_purpose::STANDARD, Engine};

/// Proxy authentication credentials
///
/// Currently supports HTTP Basic authentication used by both
/// HTTP CONNECT proxies and SOCKS5 proxies.
#[derive(Debug, Clone)]
pub enum ProxyAuth {
    /// HTTP Basic authentication with username and password
    Basic {
        /// Username
        username: String,
        /// Password
        password: String,
    },
}

impl ProxyAuth {
    /// Encode credentials as HTTP Basic authentication header value
    ///
    /// Returns the base64-encoded "username:password" format.
    ///
    /// # Examples
    ///
    /// ```
    /// use proxy_support::ProxyAuth;
    ///
    /// let auth = ProxyAuth::Basic {
    ///     username: "user".to_string(),
    ///     password: "pass".to_string(),
    /// };
    ///
    /// let encoded = auth.encode_basic();
    /// // encoded will be base64("user:pass")
    /// ```
    pub fn encode_basic(&self) -> String {
        match self {
            ProxyAuth::Basic { username, password } => {
                let credentials = format!("{}:{}", username, password);
                STANDARD.encode(credentials.as_bytes())
            }
        }
    }

    /// Get username and password from Basic auth
    ///
    /// Returns a tuple of (username, password) for Basic authentication.
    ///
    /// # Examples
    ///
    /// ```
    /// use proxy_support::ProxyAuth;
    ///
    /// let auth = ProxyAuth::Basic {
    ///     username: "user".to_string(),
    ///     password: "pass".to_string(),
    /// };
    ///
    /// let (user, pass) = auth.credentials();
    /// assert_eq!(user, "user");
    /// assert_eq!(pass, "pass");
    /// ```
    pub fn credentials(&self) -> (&str, &str) {
        match self {
            ProxyAuth::Basic { username, password } => (username.as_str(), password.as_str()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_auth_encode() {
        let auth = ProxyAuth::Basic {
            username: "user".to_string(),
            password: "pass".to_string(),
        };

        let encoded = auth.encode_basic();

        // "user:pass" in base64 is "dXNlcjpwYXNz"
        assert_eq!(encoded, "dXNlcjpwYXNz");
    }

    #[test]
    fn test_basic_auth_credentials() {
        let auth = ProxyAuth::Basic {
            username: "testuser".to_string(),
            password: "testpass".to_string(),
        };

        let (user, pass) = auth.credentials();
        assert_eq!(user, "testuser");
        assert_eq!(pass, "testpass");
    }

    #[test]
    fn test_basic_auth_with_special_chars() {
        let auth = ProxyAuth::Basic {
            username: "user@domain".to_string(),
            password: "p@ss:w0rd!".to_string(),
        };

        let encoded = auth.encode_basic();

        // Verify encoding works with special characters
        assert!(!encoded.is_empty());

        let (user, pass) = auth.credentials();
        assert_eq!(user, "user@domain");
        assert_eq!(pass, "p@ss:w0rd!");
    }
}
