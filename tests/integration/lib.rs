// Integration test library
// Provides shared utilities and helpers for cross-component integration tests

pub mod test_helpers;
pub mod test_data;

// Integration test modules
pub mod test_dns_tls_http;
pub mod test_cookie_http;
pub mod test_cache_http;
pub mod test_websocket;
pub mod test_network_stack;
pub mod test_network_stack_integration;

// Re-export commonly used types
pub use network_types::{NetworkRequest, NetworkResponse, HttpMethod};
pub use network_errors::NetworkError;
pub use url::Url;

/// Common test setup and assertions
pub mod prelude {
    pub use super::test_helpers::*;
    pub use super::test_data::*;
    pub use network_types::*;
    pub use network_errors::*;
    pub use url::Url;
    pub use std::str::FromStr;
}
