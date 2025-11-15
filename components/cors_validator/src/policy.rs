//! CORS policy configuration

/// CORS policy configuration
///
/// Controls how CORS validation is performed.
#[derive(Debug, Clone)]
pub struct CorsConfig {
    /// Whether to enforce same-origin policy
    ///
    /// If true, cross-origin requests will be blocked.
    /// If false, cross-origin requests are allowed (subject to other CORS rules).
    pub enforce_same_origin: bool,

    /// Whether to allow credentials (cookies, authorization headers)
    ///
    /// If true, Access-Control-Allow-Credentials: true header is added to responses.
    /// Note: Cannot use wildcard (*) origin when credentials are allowed.
    pub allow_credentials: bool,

    /// Allowed origins for CORS requests
    ///
    /// If None, all origins are allowed (equivalent to wildcard).
    /// If Some, only the specified origins are allowed.
    /// Note: Cannot include wildcard (*) when credentials are allowed.
    pub allowed_origins: Option<Vec<String>>,
}

impl Default for CorsConfig {
    fn default() -> Self {
        Self {
            enforce_same_origin: false,
            allow_credentials: false,
            allowed_origins: None,
        }
    }
}

impl CorsConfig {
    /// Create a new CORS configuration
    pub fn new(enforce_same_origin: bool, allow_credentials: bool) -> Self {
        Self {
            enforce_same_origin,
            allow_credentials,
            allowed_origins: None,
        }
    }

    /// Create a configuration that enforces same-origin policy
    pub fn same_origin_only() -> Self {
        Self {
            enforce_same_origin: true,
            allow_credentials: false,
            allowed_origins: None,
        }
    }

    /// Create a configuration that allows all origins
    pub fn allow_all_origins() -> Self {
        Self {
            enforce_same_origin: false,
            allow_credentials: false,
            allowed_origins: None,
        }
    }

    /// Create a configuration that allows credentials
    pub fn with_credentials() -> Self {
        Self {
            enforce_same_origin: false,
            allow_credentials: true,
            allowed_origins: None,
        }
    }

    /// Validates the CORS configuration for security issues
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Credentials are enabled with wildcard origin (*)
    /// - Credentials are enabled with no specific origins (None)
    ///
    /// # Returns
    ///
    /// Ok(()) if the configuration is valid, Err with a descriptive message otherwise.
    pub fn validate(&self) -> Result<(), String> {
        // Check for wildcard origin with credentials
        if self.allow_credentials {
            match &self.allowed_origins {
                None => {
                    return Err(
                        "CORS misconfiguration: Cannot use wildcard origin (*) with credentials. \
                         Specify explicit allowed origins when credentials are enabled.".to_string()
                    );
                }
                Some(origins) => {
                    // Check if any origin is wildcard
                    if origins.iter().any(|origin| origin == "*") {
                        return Err(
                            "CORS misconfiguration: Cannot use wildcard origin (*) with credentials. \
                             Remove '*' from allowed_origins or disable credentials.".to_string()
                        );
                    }
                }
            }
        }
        Ok(())
    }
}
