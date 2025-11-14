#[cfg(test)]
mod tests {
    use certificate_transparency::{CtPolicy, CtResult, CtVerifier, SignedCertificateTimestamp};

    #[test]
    fn test_ct_verifier_creation() {
        let policy = CtPolicy {
            require_sct: true,
            min_sct_count: 2,
        };
        let _verifier = CtVerifier::new(policy);
        // Test passes if no panic
    }

    #[test]
    fn test_verify_scts_valid_with_sufficient_count() {
        let policy = CtPolicy {
            require_sct: true,
            min_sct_count: 2,
        };
        let verifier = CtVerifier::new(policy);

        let scts = vec![
            SignedCertificateTimestamp {
                version: 0,
                log_id: vec![1u8; 32], // Valid 32-byte log ID
                timestamp: 1234567890,
                signature: vec![5, 6, 7, 8],
            },
            SignedCertificateTimestamp {
                version: 0,
                log_id: vec![2u8; 32], // Valid 32-byte log ID
                timestamp: 1234567891,
                signature: vec![13, 14, 15, 16],
            },
        ];

        let result = verifier.verify_scts(&scts);
        match result {
            CtResult::Valid { sct_count } => assert_eq!(sct_count, 2),
            _ => panic!("Expected Valid result"),
        }
    }

    #[test]
    fn test_verify_scts_invalid_insufficient_count() {
        let policy = CtPolicy {
            require_sct: true,
            min_sct_count: 3,
        };
        let verifier = CtVerifier::new(policy);

        let scts = vec![SignedCertificateTimestamp {
            version: 0,
            log_id: vec![1u8; 32], // Valid 32-byte log ID
            timestamp: 1234567890,
            signature: vec![5, 6, 7, 8],
        }];

        let result = verifier.verify_scts(&scts);
        match result {
            CtResult::Invalid { reason } => {
                assert!(reason.contains("minimum") || reason.contains("Insufficient"));
            }
            _ => panic!("Expected Invalid result"),
        }
    }

    #[test]
    fn test_verify_scts_empty_list_when_required() {
        let policy = CtPolicy {
            require_sct: true,
            min_sct_count: 1,
        };
        let verifier = CtVerifier::new(policy);

        let scts = vec![];
        let result = verifier.verify_scts(&scts);
        match result {
            CtResult::Invalid { reason } => {
                assert!(reason.contains("minimum") || reason.contains("required"));
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
        match result {
            CtResult::NotChecked => (),
            _ => panic!("Expected NotChecked result"),
        }
    }

    #[test]
    fn test_verify_scts_invalid_version() {
        let policy = CtPolicy {
            require_sct: true,
            min_sct_count: 1,
        };
        let verifier = CtVerifier::new(policy);

        let scts = vec![SignedCertificateTimestamp {
            version: 99, // Invalid version
            log_id: vec![1u8; 32], // Valid 32-byte log ID
            timestamp: 1234567890,
            signature: vec![5, 6, 7, 8],
        }];

        let result = verifier.verify_scts(&scts);
        match result {
            CtResult::Invalid { reason } => {
                assert!(reason.contains("version"));
            }
            _ => panic!("Expected Invalid result"),
        }
    }

    #[test]
    fn test_parse_sct_extension_empty() {
        use certificate_transparency::CtVerifier;

        let extension = vec![];
        let result = CtVerifier::parse_sct_extension(&extension);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_sct_extension_invalid_data() {
        use certificate_transparency::CtVerifier;

        let extension = vec![1, 2, 3]; // Too short, invalid
        let result = CtVerifier::parse_sct_extension(&extension);
        assert!(result.is_err());
    }
}
