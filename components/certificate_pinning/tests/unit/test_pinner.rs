use certificate_pinning::{CertificatePinner, Pin, PinResult, PinType};

#[test]
fn test_new_pinner_is_empty() {
    let pinner = CertificatePinner::new();
    // Verify new pinner has no pins
    assert!(pinner.pins.is_empty());
}

#[test]
fn test_add_pin_sha256() {
    let mut pinner = CertificatePinner::new();
    let pin = Pin {
        pin_type: PinType::Sha256,
        hash: vec![0x01, 0x02, 0x03],
    };

    pinner.add_pin("example.com", pin);

    // Verify pin was added
    assert!(pinner.pins.contains_key("example.com"));
    assert_eq!(pinner.pins.get("example.com").unwrap().len(), 1);
}

#[test]
fn test_add_multiple_pins_for_same_host() {
    let mut pinner = CertificatePinner::new();
    let pin1 = Pin {
        pin_type: PinType::Sha256,
        hash: vec![0x01, 0x02, 0x03],
    };
    let pin2 = Pin {
        pin_type: PinType::Sha384,
        hash: vec![0x04, 0x05, 0x06],
    };

    pinner.add_pin("example.com", pin1);
    pinner.add_pin("example.com", pin2);

    // Verify both pins were added
    assert_eq!(pinner.pins.get("example.com").unwrap().len(), 2);
}

#[test]
fn test_verify_not_pinned() {
    let pinner = CertificatePinner::new();
    let cert_der = vec![0x30, 0x82, 0x01, 0x00]; // Mock DER certificate

    let result = pinner.verify("example.com", &cert_der);

    match result {
        PinResult::NotPinned => {}
        _ => panic!("Expected NotPinned result"),
    }
}

#[test]
fn test_verify_valid_pin() {
    let mut pinner = CertificatePinner::new();

    // Create a test certificate (simplified)
    let cert_der = vec![0x30, 0x82, 0x01, 0x00, 0x01, 0x02, 0x03];

    // Compute SHA-256 hash of the certificate
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(&cert_der);
    let hash = hasher.finalize().to_vec();

    // Add pin with the correct hash
    let pin = Pin {
        pin_type: PinType::Sha256,
        hash: hash.clone(),
    };
    pinner.add_pin("example.com", pin);

    // Verify should succeed
    let result = pinner.verify("example.com", &cert_der);

    match result {
        PinResult::Valid => {}
        _ => panic!("Expected Valid result, got: {:?}", result),
    }
}

#[test]
fn test_verify_invalid_pin() {
    let mut pinner = CertificatePinner::new();

    // Add pin with wrong hash
    let pin = Pin {
        pin_type: PinType::Sha256,
        hash: vec![0x01, 0x02, 0x03], // Wrong hash
    };
    pinner.add_pin("example.com", pin);

    let cert_der = vec![0x30, 0x82, 0x01, 0x00];

    // Verify should fail
    let result = pinner.verify("example.com", &cert_der);

    match result {
        PinResult::Invalid { reason } => {
            assert!(reason.contains("match"));
        }
        _ => panic!("Expected Invalid result"),
    }
}

#[test]
fn test_verify_with_multiple_pins() {
    let mut pinner = CertificatePinner::new();

    let cert_der = vec![0x30, 0x82, 0x01, 0x00];

    // Compute correct hash
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(&cert_der);
    let correct_hash = hasher.finalize().to_vec();

    // Add wrong pin first
    let wrong_pin = Pin {
        pin_type: PinType::Sha256,
        hash: vec![0x01, 0x02, 0x03],
    };
    pinner.add_pin("example.com", wrong_pin);

    // Add correct pin second
    let correct_pin = Pin {
        pin_type: PinType::Sha256,
        hash: correct_hash,
    };
    pinner.add_pin("example.com", correct_pin);

    // Should succeed (one of the pins matches)
    let result = pinner.verify("example.com", &cert_der);

    match result {
        PinResult::Valid => {}
        _ => panic!("Expected Valid result"),
    }
}

#[test]
fn test_remove_pins() {
    let mut pinner = CertificatePinner::new();

    let pin = Pin {
        pin_type: PinType::Sha256,
        hash: vec![0x01, 0x02, 0x03],
    };
    pinner.add_pin("example.com", pin);

    assert!(pinner.pins.contains_key("example.com"));

    pinner.remove_pins("example.com");

    assert!(!pinner.pins.contains_key("example.com"));
}

#[test]
fn test_sha384_hash_verification() {
    let mut pinner = CertificatePinner::new();

    let cert_der = vec![0x30, 0x82, 0x01, 0x00, 0x01, 0x02];

    // Compute SHA-384 hash
    use sha2::{Digest, Sha384};
    let mut hasher = Sha384::new();
    hasher.update(&cert_der);
    let hash = hasher.finalize().to_vec();

    let pin = Pin {
        pin_type: PinType::Sha384,
        hash: hash.clone(),
    };
    pinner.add_pin("example.com", pin);

    let result = pinner.verify("example.com", &cert_der);

    match result {
        PinResult::Valid => {}
        _ => panic!("Expected Valid result for SHA-384"),
    }
}

#[test]
fn test_sha512_hash_verification() {
    let mut pinner = CertificatePinner::new();

    let cert_der = vec![0x30, 0x82, 0x01, 0x00, 0x01, 0x02, 0x03];

    // Compute SHA-512 hash
    use sha2::{Digest, Sha512};
    let mut hasher = Sha512::new();
    hasher.update(&cert_der);
    let hash = hasher.finalize().to_vec();

    let pin = Pin {
        pin_type: PinType::Sha512,
        hash: hash.clone(),
    };
    pinner.add_pin("example.com", pin);

    let result = pinner.verify("example.com", &cert_der);

    match result {
        PinResult::Valid => {}
        _ => panic!("Expected Valid result for SHA-512"),
    }
}
