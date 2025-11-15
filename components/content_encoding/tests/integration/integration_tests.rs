use content_encoding::{ContentEncoder, Encoding};

#[test]
fn test_multiple_encoding_types_on_same_encoder() {
    let encoder = ContentEncoder::new();
    let data = b"Testing multiple encodings with the same encoder instance";

    // Test all encoding types work with the same encoder
    let gzip_encoded = encoder.encode(data, Encoding::Gzip).unwrap();
    let deflate_encoded = encoder.encode(data, Encoding::Deflate).unwrap();
    let brotli_encoded = encoder.encode(data, Encoding::Brotli).unwrap();
    let identity_encoded = encoder.encode(data, Encoding::Identity).unwrap();

    // All should decode correctly
    assert_eq!(encoder.decode(&gzip_encoded, Encoding::Gzip).unwrap().as_slice(), data);
    assert_eq!(encoder.decode(&deflate_encoded, Encoding::Deflate).unwrap().as_slice(), data);
    assert_eq!(encoder.decode(&brotli_encoded, Encoding::Brotli).unwrap().as_slice(), data);
    assert_eq!(encoder.decode(&identity_encoded, Encoding::Identity).unwrap().as_slice(), data);
}

#[test]
fn test_cross_encoding_fails_gracefully() {
    let encoder = ContentEncoder::new();
    let data = b"Test data for cross-encoding validation";

    let gzip_encoded = encoder.encode(data, Encoding::Gzip).unwrap();

    // Trying to decode gzip data as deflate should fail
    let result = encoder.decode(&gzip_encoded, Encoding::Deflate);
    assert!(result.is_err(), "Cross-encoding should fail");

    // Trying to decode gzip data as brotli should fail
    let result = encoder.decode(&gzip_encoded, Encoding::Brotli);
    assert!(result.is_err(), "Cross-encoding should fail");
}

#[test]
fn test_encoding_comparison() {
    let encoder = ContentEncoder::new();
    // Highly compressible data
    let data = vec![b'A'; 10000];

    let gzip_encoded = encoder.encode(&data, Encoding::Gzip).unwrap();
    let deflate_encoded = encoder.encode(&data, Encoding::Deflate).unwrap();
    let brotli_encoded = encoder.encode(&data, Encoding::Brotli).unwrap();

    // All should compress significantly
    assert!(gzip_encoded.len() < data.len() / 10);
    assert!(deflate_encoded.len() < data.len() / 10);
    assert!(brotli_encoded.len() < data.len() / 10);

    // All should decode back to original
    assert_eq!(encoder.decode(&gzip_encoded, Encoding::Gzip).unwrap(), data);
    assert_eq!(encoder.decode(&deflate_encoded, Encoding::Deflate).unwrap(), data);
    assert_eq!(encoder.decode(&brotli_encoded, Encoding::Brotli).unwrap(), data);
}

#[test]
fn test_real_world_json_compression() {
    let encoder = ContentEncoder::new();
    let json_data = br#"{
        "users": [
            {"id": 1, "name": "Alice", "email": "alice@example.com"},
            {"id": 2, "name": "Bob", "email": "bob@example.com"},
            {"id": 3, "name": "Charlie", "email": "charlie@example.com"}
        ],
        "metadata": {
            "version": "1.0",
            "timestamp": "2025-11-14T12:00:00Z"
        }
    }"#;

    // All encodings should handle JSON properly
    for encoding in [Encoding::Gzip, Encoding::Deflate, Encoding::Brotli, Encoding::Identity] {
        let encoded = encoder.encode(json_data, encoding).unwrap();
        let decoded = encoder.decode(&encoded, encoding).unwrap();
        assert_eq!(decoded.as_slice(), json_data);
    }
}

#[test]
fn test_binary_data_encoding() {
    let encoder = ContentEncoder::new();
    // Binary data with all byte values
    let binary_data: Vec<u8> = (0..=255).collect();

    for encoding in [Encoding::Gzip, Encoding::Deflate, Encoding::Brotli, Encoding::Identity] {
        let encoded = encoder.encode(&binary_data, encoding).unwrap();
        let decoded = encoder.decode(&encoded, encoding).unwrap();
        assert_eq!(decoded, binary_data);
    }
}

#[test]
fn test_accept_encoding_header_format() {
    let encoder = ContentEncoder::new();
    let accept_encoding = encoder.get_accept_encoding();

    // Should be comma-separated list
    assert!(accept_encoding.contains(',') || accept_encoding.len() > 0);

    // Common format: "gzip, deflate, br" or similar
    let parts: Vec<&str> = accept_encoding.split(',').map(|s| s.trim()).collect();
    assert!(parts.len() >= 3, "Should have at least 3 encodings");

    // Should include standard encoding names
    let combined = accept_encoding.to_lowercase();
    assert!(combined.contains("gzip"));
    assert!(combined.contains("deflate"));
    assert!(combined.contains("br") || combined.contains("brotli"));
}
