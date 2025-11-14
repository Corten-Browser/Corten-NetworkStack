#[cfg(test)]
mod integration_tests {
    use certificate_transparency::{CtPolicy, CtResult, CtVerifier, SignedCertificateTimestamp};

    #[test]
    fn test_complete_ct_verification_workflow() {
        // Create a strict policy
        let policy = CtPolicy::strict(2);
        let verifier = CtVerifier::new(policy);

        // Create valid SCTs
        let scts = vec![
            SignedCertificateTimestamp {
                version: 0,
                log_id: vec![0xAAu8; 32],
                timestamp: 1609459200000, // 2021-01-01
                signature: vec![0x01, 0x02, 0x03, 0x04, 0x05],
            },
            SignedCertificateTimestamp {
                version: 0,
                log_id: vec![0xBBu8; 32],
                timestamp: 1609459200001,
                signature: vec![0x06, 0x07, 0x08, 0x09, 0x0A],
            },
        ];

        // Verify
        let result = verifier.verify_scts(&scts);

        // Should be valid
        match result {
            CtResult::Valid { sct_count } => {
                assert_eq!(sct_count, 2);
            }
            _ => panic!("Expected valid result"),
        }
    }

    #[test]
    fn test_lenient_policy_workflow() {
        // Create a lenient policy
        let policy = CtPolicy::lenient();
        let verifier = CtVerifier::new(policy);

        // No SCTs
        let scts = vec![];

        // Verify - should not check
        let result = verifier.verify_scts(&scts);
        assert!(matches!(result, CtResult::NotChecked));
    }

    #[test]
    fn test_mixed_valid_and_invalid_scts() {
        let policy = CtPolicy::strict(1);
        let verifier = CtVerifier::new(policy);

        // One valid, one invalid SCT
        let scts = vec![
            SignedCertificateTimestamp {
                version: 0,
                log_id: vec![0xAAu8; 32],
                timestamp: 1609459200000,
                signature: vec![0x01, 0x02, 0x03],
            },
            SignedCertificateTimestamp {
                version: 99, // Invalid version
                log_id: vec![0xBBu8; 32],
                timestamp: 1609459200001,
                signature: vec![0x04, 0x05, 0x06],
            },
        ];

        // Should fail on validation
        let result = verifier.verify_scts(&scts);
        match result {
            CtResult::Invalid { reason } => {
                assert!(reason.contains("version"));
            }
            _ => panic!("Expected invalid result"),
        }
    }

    #[test]
    fn test_edge_case_exact_minimum_scts() {
        let policy = CtPolicy::strict(3);
        let verifier = CtVerifier::new(policy);

        // Exactly 3 SCTs (minimum)
        let scts = vec![
            SignedCertificateTimestamp {
                version: 0,
                log_id: vec![0xAAu8; 32],
                timestamp: 1609459200000,
                signature: vec![0x01],
            },
            SignedCertificateTimestamp {
                version: 0,
                log_id: vec![0xBBu8; 32],
                timestamp: 1609459200001,
                signature: vec![0x02],
            },
            SignedCertificateTimestamp {
                version: 0,
                log_id: vec![0xCCu8; 32],
                timestamp: 1609459200002,
                signature: vec![0x03],
            },
        ];

        let result = verifier.verify_scts(&scts);
        match result {
            CtResult::Valid { sct_count } => {
                assert_eq!(sct_count, 3);
            }
            _ => panic!("Expected valid result"),
        }
    }
}
