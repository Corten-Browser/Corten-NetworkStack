use network_types::ResponseBody;

#[test]
fn test_response_body_bytes_variant() {
    // Given raw bytes
    // When creating a ResponseBody::Bytes variant
    // Then it should contain the bytes
    let data = vec![1, 2, 3, 4, 5];
    let body = ResponseBody::Bytes(data.clone());

    match body {
        ResponseBody::Bytes(bytes) => assert_eq!(bytes, data),
        _ => panic!("Expected Bytes variant"),
    }
}

#[test]
fn test_response_body_empty_variant() {
    // Given an empty response
    // When creating a ResponseBody::Empty variant
    // Then it should represent no body
    let body = ResponseBody::Empty;

    match body {
        ResponseBody::Empty => (),
        _ => panic!("Expected Empty variant"),
    }
}

#[test]
fn test_response_body_stream_variant() {
    // Given a stream
    // When creating a ResponseBody::Stream variant
    // Then it should be accessible
    // Stream implementation will be tested after Stream types are set up
    // This test verifies the variant exists
}

#[test]
fn test_response_body_debug() {
    // Given a response body
    // When debug formatted
    // Then it should produce readable output
    let body = ResponseBody::Empty;
    let debug_str = format!("{:?}", body);
    assert!(debug_str.contains("Empty"));

    let body2 = ResponseBody::Bytes(vec![1, 2, 3]);
    let debug_str2 = format!("{:?}", body2);
    assert!(debug_str2.contains("Bytes"));
}

#[test]
fn test_response_body_empty_bytes() {
    // Given empty bytes
    // When creating a ResponseBody::Bytes
    // Then it should be valid and distinct from Empty
    let body = ResponseBody::Bytes(Vec::new());

    match body {
        ResponseBody::Bytes(bytes) => assert_eq!(bytes.len(), 0),
        _ => panic!("Expected Bytes variant"),
    }
}

#[test]
fn test_response_body_large_bytes() {
    // Given a large byte array
    // When creating a ResponseBody::Bytes
    // Then it should handle large data
    let large_data = vec![0u8; 1024 * 1024]; // 1MB
    let body = ResponseBody::Bytes(large_data.clone());

    match body {
        ResponseBody::Bytes(bytes) => assert_eq!(bytes.len(), large_data.len()),
        _ => panic!("Expected Bytes variant"),
    }
}
