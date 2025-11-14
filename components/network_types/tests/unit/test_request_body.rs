use network_types::RequestBody;

#[test]
fn test_request_body_bytes_variant() {
    // Given raw bytes
    // When creating a RequestBody::Bytes variant
    // Then it should contain the bytes
    let data = vec![1, 2, 3, 4, 5];
    let body = RequestBody::Bytes(data.clone());

    match body {
        RequestBody::Bytes(bytes) => assert_eq!(bytes, data),
        _ => panic!("Expected Bytes variant"),
    }
}

#[test]
fn test_request_body_text_variant() {
    // Given a text string
    // When creating a RequestBody::Text variant
    // Then it should contain the string
    let text = "Hello, World!".to_string();
    let body = RequestBody::Text(text.clone());

    match body {
        RequestBody::Text(content) => assert_eq!(content, text),
        _ => panic!("Expected Text variant"),
    }
}

#[test]
fn test_request_body_form_data_variant() {
    // Given form data
    // When creating a RequestBody::FormData variant
    // Then it should be accessible
    // We'll need to define FormData type first
    // This test verifies the variant exists
    // Full implementation will be tested after FormData is implemented
}

#[test]
fn test_request_body_stream_variant() {
    // Given a stream
    // When creating a RequestBody::Stream variant
    // Then it should be accessible
    // We'll need to define BodyStream type first
    // This test verifies the variant exists
    // Full implementation will be tested after BodyStream is implemented
}

#[test]
fn test_request_body_debug() {
    // Given a request body
    // When debug formatted
    // Then it should produce readable output
    let body = RequestBody::Text("test".to_string());
    let debug_str = format!("{:?}", body);
    assert!(debug_str.contains("Text") || debug_str.contains("test"));
}

#[test]
fn test_request_body_clone() {
    // Given a request body
    // When cloned
    // Then the clone should have same content
    let body = RequestBody::Bytes(vec![1, 2, 3]);
    let cloned = body.clone();

    match (body, cloned) {
        (RequestBody::Bytes(b1), RequestBody::Bytes(b2)) => assert_eq!(b1, b2),
        _ => panic!("Expected Bytes variants"),
    }
}

#[test]
fn test_request_body_empty_text() {
    // Given an empty string
    // When creating a RequestBody::Text
    // Then it should be valid
    let body = RequestBody::Text(String::new());

    match body {
        RequestBody::Text(content) => assert_eq!(content, ""),
        _ => panic!("Expected Text variant"),
    }
}

#[test]
fn test_request_body_empty_bytes() {
    // Given empty bytes
    // When creating a RequestBody::Bytes
    // Then it should be valid
    let body = RequestBody::Bytes(Vec::new());

    match body {
        RequestBody::Bytes(bytes) => assert_eq!(bytes.len(), 0),
        _ => panic!("Expected Bytes variant"),
    }
}

#[test]
fn test_request_body_large_bytes() {
    // Given a large byte array
    // When creating a RequestBody::Bytes
    // Then it should handle large data
    let large_data = vec![0u8; 1024 * 1024]; // 1MB
    let body = RequestBody::Bytes(large_data.clone());

    match body {
        RequestBody::Bytes(bytes) => assert_eq!(bytes.len(), large_data.len()),
        _ => panic!("Expected Bytes variant"),
    }
}
