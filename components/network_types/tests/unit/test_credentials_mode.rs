use network_types::CredentialsMode;

#[test]
fn test_credentials_mode_variants_exist() {
    // Given the CredentialsMode enum
    // When all credential handling variants are defined
    // Then each variant should be accessible
    let _ = CredentialsMode::Omit;
    let _ = CredentialsMode::SameOrigin;
    let _ = CredentialsMode::Include;
}

#[test]
fn test_credentials_mode_debug() {
    // Given a credentials mode
    // When debug formatted
    // Then it should produce readable output
    let mode = CredentialsMode::Include;
    let debug_str = format!("{:?}", mode);
    assert!(debug_str.contains("Include"));
}

#[test]
fn test_credentials_mode_clone() {
    // Given a credentials mode
    // When cloned
    // Then the clone should equal the original
    let mode = CredentialsMode::SameOrigin;
    let cloned = mode;
    assert_eq!(mode, cloned);
}

#[test]
fn test_credentials_mode_partial_eq() {
    // Given two credentials modes
    // When compared for equality
    // Then same variants should be equal
    // And different variants should not be equal
    assert_eq!(CredentialsMode::Omit, CredentialsMode::Omit);
    assert_eq!(CredentialsMode::Include, CredentialsMode::Include);
    assert_ne!(CredentialsMode::Omit, CredentialsMode::Include);
    assert_ne!(CredentialsMode::SameOrigin, CredentialsMode::Omit);
}

#[test]
fn test_credentials_mode_all_variants() {
    // Given all CredentialsMode variants
    // When verifying distinctness
    // Then each should be unique
    let modes = [
        CredentialsMode::Omit,
        CredentialsMode::SameOrigin,
        CredentialsMode::Include,
    ];
    assert_eq!(modes.len(), 3);

    for (i, mode1) in modes.iter().enumerate() {
        for (j, mode2) in modes.iter().enumerate() {
            if i == j {
                assert_eq!(mode1, mode2);
            } else {
                assert_ne!(mode1, mode2);
            }
        }
    }
}
