use network_types::RedirectMode;

#[test]
fn test_redirect_mode_variants_exist() {
    // Given the RedirectMode enum
    // When all redirect handling variants are defined
    // Then each variant should be accessible
    let _ = RedirectMode::Follow;
    let _ = RedirectMode::Error;
    let _ = RedirectMode::Manual;
}

#[test]
fn test_redirect_mode_debug() {
    // Given a redirect mode
    // When debug formatted
    // Then it should produce readable output
    let mode = RedirectMode::Follow;
    let debug_str = format!("{:?}", mode);
    assert!(debug_str.contains("Follow"));
}

#[test]
fn test_redirect_mode_clone() {
    // Given a redirect mode
    // When cloned
    // Then the clone should equal the original
    let mode = RedirectMode::Manual;
    let cloned = mode;
    assert_eq!(mode, cloned);
}

#[test]
fn test_redirect_mode_partial_eq() {
    // Given two redirect modes
    // When compared for equality
    // Then same variants should be equal
    // And different variants should not be equal
    assert_eq!(RedirectMode::Follow, RedirectMode::Follow);
    assert_eq!(RedirectMode::Error, RedirectMode::Error);
    assert_ne!(RedirectMode::Follow, RedirectMode::Error);
    assert_ne!(RedirectMode::Manual, RedirectMode::Follow);
}
