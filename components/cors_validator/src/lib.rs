//! CORS (Cross-Origin Resource Sharing) validator component
//!
//! This component provides CORS policy enforcement and validation for network requests and responses.
//! It handles same-origin policy checks, preflight request handling, credential mode enforcement,
//! and Access-Control-* header management.

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

use http::HeaderMap;
use network_types::{HttpMethod, NetworkRequest, NetworkResponse, RequestMode};
use url::Url;

mod validator;
mod preflight;
mod headers;
mod policy;

pub use validator::CorsValidator;
pub use policy::CorsConfig;

/// Result of CORS validation
///
/// Contains whether the request/response is allowed under CORS policy,
/// the reason if blocked, and any headers that should be added.
#[derive(Debug, Clone)]
pub struct CorsResult {
    /// Whether the request/response is allowed
    pub allowed: bool,
    /// Reason for blocking (if not allowed)
    pub reason: Option<String>,
    /// Headers to add to the request/response
    pub headers_to_add: HeaderMap,
}

impl CorsResult {
    /// Create a new CORS result indicating the request is allowed
    pub fn allowed() -> Self {
        Self {
            allowed: true,
            reason: None,
            headers_to_add: HeaderMap::new(),
        }
    }

    /// Create a new CORS result indicating the request is allowed with headers
    pub fn allowed_with_headers(headers: HeaderMap) -> Self {
        Self {
            allowed: true,
            reason: None,
            headers_to_add: headers,
        }
    }

    /// Create a new CORS result indicating the request is blocked
    pub fn blocked(reason: String) -> Self {
        Self {
            allowed: false,
            reason: Some(reason),
            headers_to_add: HeaderMap::new(),
        }
    }
}
