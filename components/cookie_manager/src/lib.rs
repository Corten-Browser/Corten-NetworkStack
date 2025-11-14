//! cookie_manager component
//!
//! Cookie storage, cookie jar implementation, Set-Cookie parsing, cookie policy enforcement
//!
//! This component provides cookie management functionality for the network stack,
//! including storage, retrieval, parsing, and policy enforcement (Secure, HttpOnly, SameSite).

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

mod store;
mod jar;
mod parser;

pub use store::CookieStore;
pub use jar::CookieJar;
pub use parser::parse_set_cookie;

// Re-export Cookie from cookie crate for convenience
pub use cookie::Cookie as CookieType;
