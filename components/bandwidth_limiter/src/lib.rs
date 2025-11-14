//! Bandwidth limiter component
//!
//! Provides bandwidth throttling and network condition simulation for testing
//! network-dependent code under various network conditions.
//!
//! # Examples
//!
//! ```
//! use bandwidth_limiter::{BandwidthLimiter, NetworkCondition};
//! use std::time::Duration;
//!
//! #[tokio::main]
//! async fn main() {
//!     let mut limiter = BandwidthLimiter::new();
//!
//!     // Simulate 3G network
//!     limiter.apply_condition(NetworkCondition::G3);
//!
//!     // Throttle a download
//!     let data = vec![0u8; 10000];
//!     let delay = limiter.throttle_download(&data).await;
//!     println!("Download delayed by: {:?}", delay);
//!
//!     // Check statistics
//!     let stats = limiter.get_stats();
//!     println!("Downloaded: {} bytes", stats.bytes_received);
//! }
//! ```

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

mod conditions;
mod limiter;
mod tracker;

pub use conditions::NetworkCondition;
pub use limiter::{BandwidthLimiter, BandwidthStats};
pub use tracker::BandwidthTracker;
