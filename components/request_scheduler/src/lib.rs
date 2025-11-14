//! request_scheduler component
//!
//! Request prioritization and scheduling for network requests.
//!
//! This component implements a multi-priority request scheduler that ensures
//! high-priority requests (navigation, CSS, fonts) are processed before lower
//! priority requests (images, prefetch) while maintaining fairness.

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

mod request;
mod scheduler;

pub use request::{ActiveRequest, PendingRequest};
pub use scheduler::RequestScheduler;

// Re-export types from network_types for convenience
pub use network_types::RequestPriority;

/// Request ID type for tracking scheduled requests
pub type RequestId = u64;
