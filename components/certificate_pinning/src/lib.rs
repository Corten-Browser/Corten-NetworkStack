//! Certificate Pinning Component
//!
//! This component provides certificate pinning functionality for TLS connections.
//! It allows storing and verifying certificate hashes against known good values
//! to prevent man-in-the-middle attacks.

#![warn(missing_docs)]

use sha2::{Digest, Sha256, Sha384, Sha512};
use std::collections::HashMap;

/// Pin types supported for certificate hashing
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PinType {
    /// SHA-256 hash
    Sha256,
    /// SHA-384 hash
    Sha384,
    /// SHA-512 hash
    Sha512,
}

/// A certificate pin with hash type and hash value
#[derive(Debug, Clone)]
pub struct Pin {
    /// Type of hash algorithm used
    pub pin_type: PinType,
    /// Hash value
    pub hash: Vec<u8>,
}

/// Result of pin verification
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PinResult {
    /// Certificate matches a pin
    Valid,
    /// Certificate doesn't match any pins
    Invalid {
        /// Reason for failure
        reason: String,
    },
    /// Host is not pinned
    NotPinned,
}

/// Certificate pinner for managing and verifying certificate pins
#[derive(Debug)]
pub struct CertificatePinner {
    /// Stored pins per hostname
    pub pins: HashMap<String, Vec<Pin>>,
}

impl CertificatePinner {
    /// Create a new certificate pinner with no pins
    ///
    /// # Examples
    ///
    /// ```
    /// use certificate_pinning::CertificatePinner;
    ///
    /// let pinner = CertificatePinner::new();
    /// ```
    pub fn new() -> Self {
        Self {
            pins: HashMap::new(),
        }
    }

    /// Add a pin for a specific host
    ///
    /// # Arguments
    ///
    /// * `host` - The hostname to pin
    /// * `pin` - The pin to add
    ///
    /// # Examples
    ///
    /// ```
    /// use certificate_pinning::{CertificatePinner, Pin, PinType};
    ///
    /// let mut pinner = CertificatePinner::new();
    /// let pin = Pin {
    ///     pin_type: PinType::Sha256,
    ///     hash: vec![0x01, 0x02, 0x03],
    /// };
    /// pinner.add_pin("example.com", pin);
    /// ```
    pub fn add_pin(&mut self, host: &str, pin: Pin) {
        self.pins
            .entry(host.to_string())
            .or_default()
            .push(pin);
    }

    /// Verify a certificate against stored pins for a host
    ///
    /// # Arguments
    ///
    /// * `host` - The hostname to verify
    /// * `cert_der` - The certificate in DER format
    ///
    /// # Returns
    ///
    /// * `PinResult::Valid` - Certificate matches a pin
    /// * `PinResult::Invalid` - Certificate doesn't match any pins
    /// * `PinResult::NotPinned` - Host has no pins
    ///
    /// # Examples
    ///
    /// ```
    /// use certificate_pinning::{CertificatePinner, Pin, PinType, PinResult};
    /// use sha2::{Sha256, Digest};
    ///
    /// let mut pinner = CertificatePinner::new();
    /// let cert_der = vec![0x30, 0x82, 0x01, 0x00];
    ///
    /// // Compute hash
    /// let mut hasher = Sha256::new();
    /// hasher.update(&cert_der);
    /// let hash = hasher.finalize().to_vec();
    ///
    /// let pin = Pin {
    ///     pin_type: PinType::Sha256,
    ///     hash,
    /// };
    /// pinner.add_pin("example.com", pin);
    ///
    /// let result = pinner.verify("example.com", &cert_der);
    /// assert!(matches!(result, PinResult::Valid));
    /// ```
    pub fn verify(&self, host: &str, cert_der: &[u8]) -> PinResult {
        // Check if host has any pins
        let pins = match self.pins.get(host) {
            Some(pins) => pins,
            None => return PinResult::NotPinned,
        };

        // Try to match against any pin
        for pin in pins {
            let computed_hash = Self::compute_hash(pin.pin_type, cert_der);

            if computed_hash == pin.hash {
                return PinResult::Valid;
            }
        }

        // No pins matched
        PinResult::Invalid {
            reason: "Certificate hash does not match any pinned values".to_string(),
        }
    }

    /// Remove all pins for a host
    ///
    /// # Arguments
    ///
    /// * `host` - The hostname to remove pins for
    ///
    /// # Examples
    ///
    /// ```
    /// use certificate_pinning::{CertificatePinner, Pin, PinType};
    ///
    /// let mut pinner = CertificatePinner::new();
    /// let pin = Pin {
    ///     pin_type: PinType::Sha256,
    ///     hash: vec![0x01, 0x02, 0x03],
    /// };
    /// pinner.add_pin("example.com", pin);
    /// pinner.remove_pins("example.com");
    /// ```
    pub fn remove_pins(&mut self, host: &str) {
        self.pins.remove(host);
    }

    /// Compute hash of certificate using specified algorithm
    fn compute_hash(pin_type: PinType, cert_der: &[u8]) -> Vec<u8> {
        match pin_type {
            PinType::Sha256 => {
                let mut hasher = Sha256::new();
                hasher.update(cert_der);
                hasher.finalize().to_vec()
            }
            PinType::Sha384 => {
                let mut hasher = Sha384::new();
                hasher.update(cert_der);
                hasher.finalize().to_vec()
            }
            PinType::Sha512 => {
                let mut hasher = Sha512::new();
                hasher.update(cert_der);
                hasher.finalize().to_vec()
            }
        }
    }
}

impl Default for CertificatePinner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_hash_sha256() {
        let data = b"test data";
        let hash = CertificatePinner::compute_hash(PinType::Sha256, data);
        assert_eq!(hash.len(), 32); // SHA-256 produces 32 bytes
    }

    #[test]
    fn test_compute_hash_sha384() {
        let data = b"test data";
        let hash = CertificatePinner::compute_hash(PinType::Sha384, data);
        assert_eq!(hash.len(), 48); // SHA-384 produces 48 bytes
    }

    #[test]
    fn test_compute_hash_sha512() {
        let data = b"test data";
        let hash = CertificatePinner::compute_hash(PinType::Sha512, data);
        assert_eq!(hash.len(), 64); // SHA-512 produces 64 bytes
    }
}
