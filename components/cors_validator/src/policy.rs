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
}

impl Default for CorsConfig {
    fn default() -> Self {
        Self {
            enforce_same_origin: false,
            allow_credentials: false,
        }
    }
}

impl CorsConfig {
    /// Create a new CORS configuration
    pub fn new(enforce_same_origin: bool, allow_credentials: bool) -> Self {
        Self {
            enforce_same_origin,
            allow_credentials,
        }
    }

    /// Create a configuration that enforces same-origin policy
    pub fn same_origin_only() -> Self {
        Self {
            enforce_same_origin: true,
            allow_credentials: false,
        }
    }

    /// Create a configuration that allows all origins
    pub fn allow_all_origins() -> Self {
        Self {
            enforce_same_origin: false,
            allow_credentials: false,
        }
    }

    /// Create a configuration that allows credentials
    pub fn with_credentials() -> Self {
        Self {
            enforce_same_origin: false,
            allow_credentials: true,
        }
    }
}
