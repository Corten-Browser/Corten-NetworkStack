use network_types::RequestPriority;

#[test]
fn test_request_priority_variants_exist() {
    // Given the RequestPriority enum
    // When all priority variants are defined
    // Then each variant should be accessible
    let _ = RequestPriority::High;
    let _ = RequestPriority::Low;
    let _ = RequestPriority::Auto;
}

#[test]
fn test_request_priority_debug() {
    // Given a request priority
    // When debug formatted
    // Then it should produce readable output
    let priority = RequestPriority::High;
    let debug_str = format!("{:?}", priority);
    assert!(debug_str.contains("High"));
}

#[test]
fn test_request_priority_clone() {
    // Given a request priority
    // When cloned
    // Then the clone should equal the original
    let priority = RequestPriority::Low;
    let cloned = priority;
    assert_eq!(priority, cloned);
}

#[test]
fn test_request_priority_partial_eq() {
    // Given two request priorities
    // When compared for equality
    // Then same variants should be equal
    // And different variants should not be equal
    assert_eq!(RequestPriority::High, RequestPriority::High);
    assert_eq!(RequestPriority::Auto, RequestPriority::Auto);
    assert_ne!(RequestPriority::High, RequestPriority::Low);
    assert_ne!(RequestPriority::Auto, RequestPriority::High);
}
