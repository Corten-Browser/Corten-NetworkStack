use certificate_pinning::{CertificatePinner, Pin, PinResult, PinType};
use sha2::{Digest, Sha256, Sha384, Sha512};

#[test]
fn test_complete_pinning_workflow() {
    // Simulate a complete certificate pinning workflow
    let mut pinner = CertificatePinner::new();

    // Mock certificate data
    let cert_der = vec![
        0x30, 0x82, 0x03, 0x55, 0x30, 0x82, 0x02, 0x3d, 0xa0, 0x03, 0x02, 0x01, 0x02, 0x02, 0x10,
        0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
    ];

    // Compute SHA-256 hash
    let mut hasher = Sha256::new();
    hasher.update(&cert_der);
    let sha256_hash = hasher.finalize().to_vec();

    // Add pin for example.com
    let pin = Pin {
        pin_type: PinType::Sha256,
        hash: sha256_hash.clone(),
    };
    pinner.add_pin("example.com", pin);

    // Verify the certificate
    let result = pinner.verify("example.com", &cert_der);
    assert!(matches!(result, PinResult::Valid));

    // Verify different certificate fails
    let different_cert = vec![0x30, 0x82, 0x01, 0x00, 0x01, 0x02, 0x03];
    let result = pinner.verify("example.com", &different_cert);
    assert!(matches!(result, PinResult::Invalid { .. }));

    // Verify unpinned host
    let result = pinner.verify("other.com", &cert_der);
    assert!(matches!(result, PinResult::NotPinned));
}

#[test]
fn test_multiple_hosts_and_algorithms() {
    let mut pinner = CertificatePinner::new();

    // Certificate 1 for example.com (SHA-256)
    let cert1 = vec![0x30, 0x82, 0x01, 0x00, 0x01, 0x02];
    let mut hasher = Sha256::new();
    hasher.update(&cert1);
    let hash1 = hasher.finalize().to_vec();

    pinner.add_pin(
        "example.com",
        Pin {
            pin_type: PinType::Sha256,
            hash: hash1,
        },
    );

    // Certificate 2 for test.com (SHA-384)
    let cert2 = vec![0x30, 0x82, 0x02, 0x00, 0x01, 0x02, 0x03];
    let mut hasher = Sha384::new();
    hasher.update(&cert2);
    let hash2 = hasher.finalize().to_vec();

    pinner.add_pin(
        "test.com",
        Pin {
            pin_type: PinType::Sha384,
            hash: hash2,
        },
    );

    // Certificate 3 for secure.com (SHA-512)
    let cert3 = vec![0x30, 0x82, 0x03, 0x00, 0x01, 0x02, 0x03, 0x04];
    let mut hasher = Sha512::new();
    hasher.update(&cert3);
    let hash3 = hasher.finalize().to_vec();

    pinner.add_pin(
        "secure.com",
        Pin {
            pin_type: PinType::Sha512,
            hash: hash3,
        },
    );

    // Verify all hosts
    assert!(matches!(
        pinner.verify("example.com", &cert1),
        PinResult::Valid
    ));
    assert!(matches!(
        pinner.verify("test.com", &cert2),
        PinResult::Valid
    ));
    assert!(matches!(
        pinner.verify("secure.com", &cert3),
        PinResult::Valid
    ));

    // Verify cross-verification fails
    assert!(matches!(
        pinner.verify("example.com", &cert2),
        PinResult::Invalid { .. }
    ));
    assert!(matches!(
        pinner.verify("test.com", &cert1),
        PinResult::Invalid { .. }
    ));
}

#[test]
fn test_pin_rotation() {
    let mut pinner = CertificatePinner::new();

    // Old certificate
    let old_cert = vec![0x30, 0x82, 0x01, 0x00];
    let mut hasher = Sha256::new();
    hasher.update(&old_cert);
    let old_hash = hasher.finalize().to_vec();

    pinner.add_pin(
        "example.com",
        Pin {
            pin_type: PinType::Sha256,
            hash: old_hash,
        },
    );

    // Verify old cert works
    assert!(matches!(
        pinner.verify("example.com", &old_cert),
        PinResult::Valid
    ));

    // New certificate
    let new_cert = vec![0x30, 0x82, 0x02, 0x00];
    let mut hasher = Sha256::new();
    hasher.update(&new_cert);
    let new_hash = hasher.finalize().to_vec();

    // Add new pin (keep old one for transition period)
    pinner.add_pin(
        "example.com",
        Pin {
            pin_type: PinType::Sha256,
            hash: new_hash,
        },
    );

    // Both certs should work
    assert!(matches!(
        pinner.verify("example.com", &old_cert),
        PinResult::Valid
    ));
    assert!(matches!(
        pinner.verify("example.com", &new_cert),
        PinResult::Valid
    ));

    // Remove all pins and add only new one
    pinner.remove_pins("example.com");
    let mut hasher = Sha256::new();
    hasher.update(&new_cert);
    let new_hash = hasher.finalize().to_vec();

    pinner.add_pin(
        "example.com",
        Pin {
            pin_type: PinType::Sha256,
            hash: new_hash,
        },
    );

    // Only new cert should work
    assert!(matches!(
        pinner.verify("example.com", &old_cert),
        PinResult::Invalid { .. }
    ));
    assert!(matches!(
        pinner.verify("example.com", &new_cert),
        PinResult::Valid
    ));
}

#[test]
fn test_backup_pins() {
    let mut pinner = CertificatePinner::new();

    // Primary certificate
    let primary_cert = vec![0x30, 0x82, 0x01, 0x00];
    let mut hasher = Sha256::new();
    hasher.update(&primary_cert);
    let primary_hash = hasher.finalize().to_vec();

    // Backup certificate
    let backup_cert = vec![0x30, 0x82, 0x02, 0x00];
    let mut hasher = Sha256::new();
    hasher.update(&backup_cert);
    let backup_hash = hasher.finalize().to_vec();

    // Add both pins
    pinner.add_pin(
        "example.com",
        Pin {
            pin_type: PinType::Sha256,
            hash: primary_hash,
        },
    );
    pinner.add_pin(
        "example.com",
        Pin {
            pin_type: PinType::Sha256,
            hash: backup_hash,
        },
    );

    // Both should verify successfully
    assert!(matches!(
        pinner.verify("example.com", &primary_cert),
        PinResult::Valid
    ));
    assert!(matches!(
        pinner.verify("example.com", &backup_cert),
        PinResult::Valid
    ));

    // Unrelated cert should fail
    let unrelated_cert = vec![0x30, 0x82, 0x03, 0x00];
    assert!(matches!(
        pinner.verify("example.com", &unrelated_cert),
        PinResult::Invalid { .. }
    ));
}
