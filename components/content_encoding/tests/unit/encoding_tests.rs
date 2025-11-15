use content_encoding::{ContentEncoder, Encoding};

#[test]
fn test_gzip_encode_decode_roundtrip() {
    let encoder = ContentEncoder::new();
    let original_data = b"Hello, World! This is a test message for gzip compression.";

    // Encode
    let encoded = encoder.encode(original_data, Encoding::Gzip)
        .expect("Gzip encoding should succeed");

    // Encoded data should be different from original
    assert_ne!(encoded.as_slice(), original_data);

    // Decode
    let decoded = encoder.decode(&encoded, Encoding::Gzip)
        .expect("Gzip decoding should succeed");

    // Decoded should match original
    assert_eq!(decoded.as_slice(), original_data);
}

#[test]
fn test_deflate_encode_decode_roundtrip() {
    let encoder = ContentEncoder::new();
    let original_data = b"Testing deflate compression with some repetitive text text text";

    let encoded = encoder.encode(original_data, Encoding::Deflate)
        .expect("Deflate encoding should succeed");

    assert_ne!(encoded.as_slice(), original_data);

    let decoded = encoder.decode(&encoded, Encoding::Deflate)
        .expect("Deflate decoding should succeed");

    assert_eq!(decoded.as_slice(), original_data);
}

#[test]
fn test_brotli_encode_decode_roundtrip() {
    let encoder = ContentEncoder::new();
    let original_data = b"Brotli compression test data with repeated patterns patterns patterns";

    let encoded = encoder.encode(original_data, Encoding::Brotli)
        .expect("Brotli encoding should succeed");

    assert_ne!(encoded.as_slice(), original_data);

    let decoded = encoder.decode(&encoded, Encoding::Brotli)
        .expect("Brotli decoding should succeed");

    assert_eq!(decoded.as_slice(), original_data);
}

#[test]
fn test_identity_encoding_is_noop() {
    let encoder = ContentEncoder::new();
    let original_data = b"Identity encoding should not modify data";

    let encoded = encoder.encode(original_data, Encoding::Identity)
        .expect("Identity encoding should succeed");

    // Identity should return exact same data
    assert_eq!(encoded.as_slice(), original_data);

    let decoded = encoder.decode(&encoded, Encoding::Identity)
        .expect("Identity decoding should succeed");

    assert_eq!(decoded.as_slice(), original_data);
}

#[test]
fn test_gzip_compression_actually_compresses() {
    let encoder = ContentEncoder::new();
    // Highly compressible data
    let original_data = vec![b'A'; 1000];

    let encoded = encoder.encode(&original_data, Encoding::Gzip)
        .expect("Gzip encoding should succeed");

    // Compressed data should be significantly smaller
    assert!(encoded.len() < original_data.len() / 2);
}

#[test]
fn test_decode_corrupted_gzip_data_fails() {
    let encoder = ContentEncoder::new();
    let corrupted_data = b"This is not valid gzip data";

    let result = encoder.decode(corrupted_data, Encoding::Gzip);

    assert!(result.is_err(), "Decoding corrupted gzip data should fail");
}

#[test]
fn test_decode_corrupted_deflate_data_fails() {
    let encoder = ContentEncoder::new();
    let corrupted_data = b"Invalid deflate stream";

    let result = encoder.decode(corrupted_data, Encoding::Deflate);

    assert!(result.is_err(), "Decoding corrupted deflate data should fail");
}

#[test]
fn test_decode_corrupted_brotli_data_fails() {
    let encoder = ContentEncoder::new();
    let corrupted_data = b"Not a brotli stream";

    let result = encoder.decode(corrupted_data, Encoding::Brotli);

    assert!(result.is_err(), "Decoding corrupted brotli data should fail");
}

#[test]
fn test_get_accept_encoding_header() {
    let encoder = ContentEncoder::new();
    let accept_encoding = encoder.get_accept_encoding();

    // Should include all supported encodings
    assert!(accept_encoding.contains("gzip"));
    assert!(accept_encoding.contains("deflate"));
    assert!(accept_encoding.contains("br"));
}

#[test]
fn test_empty_data_encoding() {
    let encoder = ContentEncoder::new();
    let empty_data = b"";

    // Gzip should handle empty data
    let encoded = encoder.encode(empty_data, Encoding::Gzip)
        .expect("Encoding empty data should succeed");
    let decoded = encoder.decode(&encoded, Encoding::Gzip)
        .expect("Decoding should succeed");
    assert_eq!(decoded.as_slice(), empty_data);

    // Deflate should handle empty data
    let encoded = encoder.encode(empty_data, Encoding::Deflate)
        .expect("Encoding empty data should succeed");
    let decoded = encoder.decode(&encoded, Encoding::Deflate)
        .expect("Decoding should succeed");
    assert_eq!(decoded.as_slice(), empty_data);

    // Brotli should handle empty data
    let encoded = encoder.encode(empty_data, Encoding::Brotli)
        .expect("Encoding empty data should succeed");
    let decoded = encoder.decode(&encoded, Encoding::Brotli)
        .expect("Decoding should succeed");
    assert_eq!(decoded.as_slice(), empty_data);
}

#[test]
fn test_large_data_encoding() {
    let encoder = ContentEncoder::new();
    // 1MB of data
    let large_data = vec![b'X'; 1024 * 1024];

    let encoded = encoder.encode(&large_data, Encoding::Gzip)
        .expect("Encoding large data should succeed");
    let decoded = encoder.decode(&encoded, Encoding::Gzip)
        .expect("Decoding large data should succeed");

    assert_eq!(decoded.len(), large_data.len());
    assert_eq!(decoded, large_data);
}
