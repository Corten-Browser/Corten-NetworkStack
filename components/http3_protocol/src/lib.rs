//! http3_protocol component
//!
//! HTTP/3 and QUIC protocol implementation with 0-RTT, connection migration
//!
//! This component provides an HTTP/3 client built on QUIC (Quick UDP Internet Connections),
//! featuring support for 0-RTT (Zero Round Trip Time) connections and connection migration.
//!
//! # Features
//!
//! - **HTTP/3 over QUIC**: Modern HTTP protocol using QUIC transport
//! - **0-RTT Support**: Resume connections without additional handshake (when enabled)
//! - **Connection Migration**: Maintain connections across network changes
//! - **Async/Await**: Full Tokio async runtime support
//!
//! # Example
//!
//! ```rust,ignore
//! use http3_protocol::{Http3Client, Http3Config};
//! use std::time::Duration;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create configuration
//!     let config = Http3Config::default()
//!         .with_0rtt(true)
//!         .with_max_idle_timeout(Duration::from_secs(60));
//!
//!     // Create client
//!     let client = Http3Client::new(config);
//!
//!     // Construct NetworkRequest and fetch
//!     // (NetworkRequest construction details omitted)
//!     // let response = client.fetch(request).await?;
//!     // println!("Status: {}", response.status);
//!     Ok(())
//! }
//! ```
//!
//! # Experimental Status
//!
//! HTTP/3 and QUIC are relatively new protocols. This implementation is based on:
//! - `quinn` for QUIC transport
//! - `h3` for HTTP/3 protocol
//! - `h3-quinn` for integration layer
//!
//! The protocol specifications are stable, but implementations are still evolving.

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

mod client;
mod config;
mod connection;
mod error;

pub use client::Http3Client;
pub use config::Http3Config;
pub use connection::QuicConnection;
