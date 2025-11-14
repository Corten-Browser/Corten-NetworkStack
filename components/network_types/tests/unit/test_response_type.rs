use network_types::ResponseType;

#[test]
fn test_response_type_variants_exist() {
    // Given the ResponseType enum
    // When all response type variants are defined
    // Then each variant should be accessible
    let _ = ResponseType::Basic;
    let _ = ResponseType::Cors;
    let _ = ResponseType::Error;
    let _ = ResponseType::Opaque;
    let _ = ResponseType::OpaqueRedirect;
}

#[test]
fn test_response_type_debug() {
    // Given a response type
    // When debug formatted
    // Then it should produce readable output
    let resp_type = ResponseType::Basic;
    let debug_str = format!("{:?}", resp_type);
    assert!(debug_str.contains("Basic"));
}

#[test]
fn test_response_type_clone() {
    // Given a response type
    // When cloned
    // Then the clone should equal the original
    let resp_type = ResponseType::Cors;
    let cloned = resp_type;
    assert_eq!(resp_type, cloned);
}

#[test]
fn test_response_type_partial_eq() {
    // Given two response types
    // When compared for equality
    // Then same variants should be equal
    // And different variants should not be equal
    assert_eq!(ResponseType::Basic, ResponseType::Basic);
    assert_eq!(ResponseType::Error, ResponseType::Error);
    assert_ne!(ResponseType::Basic, ResponseType::Cors);
    assert_ne!(ResponseType::Opaque, ResponseType::OpaqueRedirect);
}

#[test]
fn test_response_type_all_variants() {
    // Given all ResponseType variants
    // When verifying distinctness
    // Then each should be unique
    let types = [
        ResponseType::Basic,
        ResponseType::Cors,
        ResponseType::Error,
        ResponseType::Opaque,
        ResponseType::OpaqueRedirect,
    ];

    assert_eq!(types.len(), 5);

    for (i, type1) in types.iter().enumerate() {
        for (j, type2) in types.iter().enumerate() {
            if i == j {
                assert_eq!(type1, type2);
            } else {
                assert_ne!(type1, type2);
            }
        }
    }
}
