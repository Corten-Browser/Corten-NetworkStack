//! url_handlers component
//!
//! Data URL and File URL handling with security policies
//!
//! This component provides handlers for non-HTTP URL schemes:
//! - Data URLs (data:) with base64 decoding and MIME type parsing
//! - File URLs (file:) with security policy enforcement
//!
//! # Examples
//!
//! ## Data URL parsing
//!
//! ```
//! use url_handlers::DataUrlHandler;
//!
//! let url = "data:text/plain;base64,SGVsbG8gV29ybGQ=";
//! let data = DataUrlHandler::parse(url).expect("Failed to parse data URL");
//! assert_eq!(data.mime_type, "text/plain");
//! assert_eq!(String::from_utf8(data.data).unwrap(), "Hello World");
//! ```
//!
//! ## File URL reading
//!
//! ```no_run
//! use url_handlers::{FileUrlHandler, FileSecurityPolicy};
//! use std::path::PathBuf;
//!
//! # async fn example() {
//! let policy = FileSecurityPolicy {
//!     allow_directory_traversal: false,
//!     allowed_paths: vec![PathBuf::from("/allowed/path")],
//! };
//! let handler = FileUrlHandler::new(policy);
//!
//! let url = "file:///allowed/path/file.txt";
//! let data = handler.read(url).await.expect("Failed to read file");
//! # }
//! ```

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

mod data;
mod file;
mod security;

pub use data::{DataUrlData, DataUrlHandler};
pub use file::FileUrlHandler;
pub use security::FileSecurityPolicy;

/// URL handler enum for dispatching to specific handlers
#[derive(Debug)]
pub enum UrlHandler {
    /// Data URL handler
    Data(DataUrlHandler),
    /// File URL handler
    File(FileUrlHandler),
}
