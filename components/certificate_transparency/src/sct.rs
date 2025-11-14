//! Signed Certificate Timestamp structures

/// Signed Certificate Timestamp
///
/// Represents a signed timestamp from a CT log server.
/// As defined in RFC 6962.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignedCertificateTimestamp {
    /// SCT version (currently only 0 is supported)
    pub version: u8,

    /// Log ID (32 bytes SHA-256 hash of log's public key)
    pub log_id: Vec<u8>,

    /// Timestamp (milliseconds since Unix epoch)
    pub timestamp: u64,

    /// Digital signature from the log
    pub signature: Vec<u8>,
}

impl SignedCertificateTimestamp {
    /// Validate the SCT structure
    ///
    /// Checks that:
    /// - Version is 0 (only supported version)
    /// - Log ID is present and valid length
    /// - Signature is present
    pub fn validate(&self) -> Result<(), String> {
        // Version must be 0 (v1 of the protocol)
        if self.version != 0 {
            return Err(format!("Unsupported SCT version: {}", self.version));
        }

        // Log ID should be 32 bytes (SHA-256)
        if self.log_id.len() != 32 {
            return Err(format!(
                "Invalid log ID length: {} (expected 32)",
                self.log_id.len()
            ));
        }

        // Signature must be present
        if self.signature.is_empty() {
            return Err("Empty signature".to_string());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sct_validate_valid() {
        let sct = SignedCertificateTimestamp {
            version: 0,
            log_id: vec![0u8; 32],
            timestamp: 1234567890,
            signature: vec![1, 2, 3, 4],
        };

        assert!(sct.validate().is_ok());
    }

    #[test]
    fn test_sct_validate_invalid_version() {
        let sct = SignedCertificateTimestamp {
            version: 1,
            log_id: vec![0u8; 32],
            timestamp: 1234567890,
            signature: vec![1, 2, 3, 4],
        };

        let result = sct.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("version"));
    }

    #[test]
    fn test_sct_validate_invalid_log_id_length() {
        let sct = SignedCertificateTimestamp {
            version: 0,
            log_id: vec![0u8; 16], // Wrong length
            timestamp: 1234567890,
            signature: vec![1, 2, 3, 4],
        };

        let result = sct.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("log ID"));
    }

    #[test]
    fn test_sct_validate_empty_signature() {
        let sct = SignedCertificateTimestamp {
            version: 0,
            log_id: vec![0u8; 32],
            timestamp: 1234567890,
            signature: vec![],
        };

        let result = sct.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("signature"));
    }
}
