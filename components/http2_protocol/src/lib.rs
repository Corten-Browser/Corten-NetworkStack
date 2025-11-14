//! http2_protocol component
//!
//! HTTP/2 client with multiplexing, stream prioritization, flow control, server push

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

mod client;
mod config;
mod connection;
mod error;

pub use client::Http2Client;
pub use config::Http2Config;
pub use connection::Http2Connection;
pub use error::Http2Error;
