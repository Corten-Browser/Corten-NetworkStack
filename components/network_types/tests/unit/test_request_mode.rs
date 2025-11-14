use network_types::RequestMode;

#[test]
fn test_request_mode_variants_exist() {
    // Given the RequestMode enum
    // When all CORS mode variants are defined
    // Then each variant should be accessible
    let _ = RequestMode::Navigate;
    let _ = RequestMode::SameOrigin;
    let _ = RequestMode::NoCors;
    let _ = RequestMode::Cors;
}

#[test]
fn test_request_mode_debug() {
    // Given a request mode
    // When debug formatted
    // Then it should produce readable output
    let mode = RequestMode::Cors;
    let debug_str = format!("{:?}", mode);
    assert!(debug_str.contains("Cors"));
}

#[test]
fn test_request_mode_clone() {
    // Given a request mode
    // When cloned
    // Then the clone should equal the original
    let mode = RequestMode::SameOrigin;
    let cloned = mode;
    assert_eq!(mode, cloned);
}

#[test]
fn test_request_mode_partial_eq() {
    // Given two request modes
    // When compared for equality
    // Then same variants should be equal
    // And different variants should not be equal
    assert_eq!(RequestMode::Cors, RequestMode::Cors);
    assert_eq!(RequestMode::Navigate, RequestMode::Navigate);
    assert_ne!(RequestMode::Cors, RequestMode::NoCors);
    assert_ne!(RequestMode::SameOrigin, RequestMode::Navigate);
}

#[test]
fn test_request_mode_all_variants() {
    // Given all RequestMode variants
    // When verifying distinctness
    // Then each should be unique
    let modes = [
        RequestMode::Navigate,
        RequestMode::SameOrigin,
        RequestMode::NoCors,
        RequestMode::Cors,
    ];
    assert_eq!(modes.len(), 4);

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
