//! CT verification logic

use crate::{CtPolicy, SignedCertificateTimestamp};
use network_errors::NetworkError;

/// Result of CT verification
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CtResult {
    /// CT verification succeeded
    Valid {
        /// Number of valid SCTs found
        sct_count: usize,
    },

    /// CT verification failed
    Invalid {
        /// Reason for failure
        reason: String,
    },

    /// CT was not checked (policy allows this)
    NotChecked,
}

/// Certificate Transparency verifier
///
/// Verifies SCTs according to a configured policy.
pub struct CtVerifier {
    policy: CtPolicy,
}

impl CtVerifier {
    /// Create a new CT verifier with the given policy
    pub fn new(policy: CtPolicy) -> Self {
        Self { policy }
    }

    /// Verify a list of SCTs according to the policy
    ///
    /// # Arguments
    ///
    /// * `scts` - List of SCTs to verify
    ///
    /// # Returns
    ///
    /// * `CtResult::Valid` if verification succeeded
    /// * `CtResult::Invalid` if verification failed
    /// * `CtResult::NotChecked` if CT is not required by policy
    pub fn verify_scts(&self, scts: &[SignedCertificateTimestamp]) -> CtResult {
        // If SCTs are not required, skip verification
        if !self.policy.require_sct {
            return CtResult::NotChecked;
        }

        // Check minimum count
        if scts.len() < self.policy.min_sct_count {
            return CtResult::Invalid {
                reason: format!(
                    "Insufficient SCTs: found {}, minimum required {}",
                    scts.len(),
                    self.policy.min_sct_count
                ),
            };
        }

        // Validate each SCT
        for (i, sct) in scts.iter().enumerate() {
            if let Err(e) = sct.validate() {
                return CtResult::Invalid {
                    reason: format!("SCT {} validation failed: {}", i, e),
                };
            }
        }

        // All checks passed
        CtResult::Valid {
            sct_count: scts.len(),
        }
    }

    /// Parse SCT extension data from TLS extension
    ///
    /// Parses the SCT list from the TLS Certificate Transparency extension.
    /// The extension format is defined in RFC 6962 Section 3.3.
    ///
    /// # Arguments
    ///
    /// * `extension` - Raw extension data
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<SignedCertificateTimestamp>)` on success
    /// * `Err(NetworkError)` if parsing fails
    pub fn parse_sct_extension(extension: &[u8]) -> Result<Vec<SignedCertificateTimestamp>, NetworkError> {
        // Extension must contain at least length prefix (2 bytes)
        if extension.len() < 2 {
            return Err(NetworkError::TlsError(
                "SCT extension too short".to_string(),
            ));
        }

        // Read total length (big-endian u16)
        let total_len = u16::from_be_bytes([extension[0], extension[1]]) as usize;

        // Verify total length matches
        if extension.len() < 2 + total_len {
            return Err(NetworkError::TlsError(
                "SCT extension length mismatch".to_string(),
            ));
        }

        let mut scts = Vec::new();
        let mut offset = 2;

        // Parse individual SCTs
        while offset < 2 + total_len {
            // Need at least 2 bytes for SCT length
            if offset + 2 > extension.len() {
                return Err(NetworkError::TlsError(
                    "Truncated SCT in extension".to_string(),
                ));
            }

            // Read SCT length (big-endian u16)
            let sct_len = u16::from_be_bytes([extension[offset], extension[offset + 1]]) as usize;
            offset += 2;

            // Verify we have enough data
            if offset + sct_len > extension.len() {
                return Err(NetworkError::TlsError(
                    "Truncated SCT data".to_string(),
                ));
            }

            // Parse SCT structure
            let sct_data = &extension[offset..offset + sct_len];
            let sct = Self::parse_sct(sct_data)?;
            scts.push(sct);

            offset += sct_len;
        }

        if scts.is_empty() {
            return Err(NetworkError::TlsError(
                "No SCTs found in extension".to_string(),
            ));
        }

        Ok(scts)
    }

    /// Parse a single SCT from binary data
    ///
    /// SCT format (RFC 6962):
    /// - version (1 byte)
    /// - log_id (32 bytes)
    /// - timestamp (8 bytes)
    /// - extensions length (2 bytes)
    /// - extensions (variable)
    /// - signature (variable)
    fn parse_sct(data: &[u8]) -> Result<SignedCertificateTimestamp, NetworkError> {
        // Minimum size: version(1) + log_id(32) + timestamp(8) + ext_len(2) = 43 bytes
        if data.len() < 43 {
            return Err(NetworkError::TlsError(
                "SCT data too short".to_string(),
            ));
        }

        let mut offset = 0;

        // Parse version
        let version = data[offset];
        offset += 1;

        // Parse log ID (32 bytes)
        let log_id = data[offset..offset + 32].to_vec();
        offset += 32;

        // Parse timestamp (8 bytes, big-endian)
        let timestamp = u64::from_be_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
            data[offset + 4],
            data[offset + 5],
            data[offset + 6],
            data[offset + 7],
        ]);
        offset += 8;

        // Parse extensions length (2 bytes, big-endian)
        let ext_len = u16::from_be_bytes([data[offset], data[offset + 1]]) as usize;
        offset += 2;

        // Skip extensions (we don't process them for now)
        if offset + ext_len > data.len() {
            return Err(NetworkError::TlsError(
                "Invalid extensions length".to_string(),
            ));
        }
        offset += ext_len;

        // Remaining data is the signature
        if offset >= data.len() {
            return Err(NetworkError::TlsError(
                "Missing signature".to_string(),
            ));
        }
        let signature = data[offset..].to_vec();

        Ok(SignedCertificateTimestamp {
            version,
            log_id,
            timestamp,
            signature,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verifier_creation() {
        let policy = CtPolicy {
            require_sct: true,
            min_sct_count: 2,
        };
        let _verifier = CtVerifier::new(policy);
    }

    #[test]
    fn test_verify_scts_valid() {
        let policy = CtPolicy {
            require_sct: true,
            min_sct_count: 2,
        };
        let verifier = CtVerifier::new(policy);

        let scts = vec![
            SignedCertificateTimestamp {
                version: 0,
                log_id: vec![0u8; 32],
                timestamp: 1234567890,
                signature: vec![1, 2, 3, 4],
            },
            SignedCertificateTimestamp {
                version: 0,
                log_id: vec![1u8; 32],
                timestamp: 1234567891,
                signature: vec![5, 6, 7, 8],
            },
        ];

        let result = verifier.verify_scts(&scts);
        assert!(matches!(result, CtResult::Valid { sct_count: 2 }));
    }

    #[test]
    fn test_verify_scts_insufficient() {
        let policy = CtPolicy {
            require_sct: true,
            min_sct_count: 3,
        };
        let verifier = CtVerifier::new(policy);

        let scts = vec![SignedCertificateTimestamp {
            version: 0,
            log_id: vec![0u8; 32],
            timestamp: 1234567890,
            signature: vec![1, 2, 3, 4],
        }];

        let result = verifier.verify_scts(&scts);
        match result {
            CtResult::Invalid { reason } => {
                assert!(reason.contains("Insufficient"));
            }
            _ => panic!("Expected Invalid result"),
        }
    }

    #[test]
    fn test_verify_scts_not_required() {
        let policy = CtPolicy {
            require_sct: false,
            min_sct_count: 0,
        };
        let verifier = CtVerifier::new(policy);

        let scts = vec![];
        let result = verifier.verify_scts(&scts);
        assert!(matches!(result, CtResult::NotChecked));
    }

    #[test]
    fn test_parse_sct_extension_empty() {
        let result = CtVerifier::parse_sct_extension(&[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_sct_extension_too_short() {
        let result = CtVerifier::parse_sct_extension(&[0]);
        assert!(result.is_err());
    }
}
