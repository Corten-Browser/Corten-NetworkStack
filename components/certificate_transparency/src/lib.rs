//! Certificate Transparency (CT) log verification
//!
//! This component provides Certificate Transparency verification functionality,
//! including parsing and validating Signed Certificate Timestamps (SCTs).
//!
//! # Examples
//!
//! ```
//! use certificate_transparency::{CtPolicy, CtVerifier, SignedCertificateTimestamp};
//!
//! let policy = CtPolicy {
//!     require_sct: true,
//!     min_sct_count: 2,
//! };
//!
//! let verifier = CtVerifier::new(policy);
//! let scts = vec![
//!     SignedCertificateTimestamp {
//!         version: 0,
//!         log_id: vec![1, 2, 3, 4],
//!         timestamp: 1234567890,
//!         signature: vec![5, 6, 7, 8],
//!     },
//! ];
//!
//! let result = verifier.verify_scts(&scts);
//! ```

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

mod policy;
mod sct;
mod verifier;

pub use policy::CtPolicy;
pub use sct::SignedCertificateTimestamp;
pub use verifier::{CtResult, CtVerifier};
