use content_encoding::{ContentEncoder, Encoding};
use bytes::Bytes;
use futures::stream::{self, StreamExt};

#[tokio::test]
async fn test_gzip_stream_decode() {
    let encoder = ContentEncoder::new();
    let original_data = b"Streaming test data for gzip decompression";

    // Encode the data first
    let encoded = encoder.encode(original_data, Encoding::Gzip)
        .expect("Encoding should succeed");

    // Create a stream from the encoded data
    let stream = stream::iter(vec![Bytes::from(encoded)]);

    // Decode the stream
    let mut decode_stream = encoder.decode_stream(stream, Encoding::Gzip);

    // Collect all chunks
    let mut result = Vec::new();
    while let Some(chunk_result) = decode_stream.next().await {
        let chunk = chunk_result.expect("Stream decoding should succeed");
        result.extend_from_slice(&chunk);
    }

    assert_eq!(result.as_slice(), original_data);
}

#[tokio::test]
async fn test_deflate_stream_decode() {
    let encoder = ContentEncoder::new();
    let original_data = b"Testing deflate streaming decompression functionality";

    let encoded = encoder.encode(original_data, Encoding::Deflate)
        .expect("Encoding should succeed");

    let stream = stream::iter(vec![Bytes::from(encoded)]);
    let mut decode_stream = encoder.decode_stream(stream, Encoding::Deflate);

    let mut result = Vec::new();
    while let Some(chunk_result) = decode_stream.next().await {
        let chunk = chunk_result.expect("Stream decoding should succeed");
        result.extend_from_slice(&chunk);
    }

    assert_eq!(result.as_slice(), original_data);
}

#[tokio::test]
async fn test_brotli_stream_decode() {
    let encoder = ContentEncoder::new();
    let original_data = b"Brotli streaming test with multiple chunks of data";

    let encoded = encoder.encode(original_data, Encoding::Brotli)
        .expect("Encoding should succeed");

    let stream = stream::iter(vec![Bytes::from(encoded)]);
    let mut decode_stream = encoder.decode_stream(stream, Encoding::Brotli);

    let mut result = Vec::new();
    while let Some(chunk_result) = decode_stream.next().await {
        let chunk = chunk_result.expect("Stream decoding should succeed");
        result.extend_from_slice(&chunk);
    }

    assert_eq!(result.as_slice(), original_data);
}

#[tokio::test]
async fn test_identity_stream_decode() {
    let encoder = ContentEncoder::new();
    let original_data = b"Identity stream should pass through unchanged";

    let stream = stream::iter(vec![Bytes::from(original_data.to_vec())]);
    let mut decode_stream = encoder.decode_stream(stream, Encoding::Identity);

    let mut result = Vec::new();
    while let Some(chunk_result) = decode_stream.next().await {
        let chunk = chunk_result.expect("Stream decoding should succeed");
        result.extend_from_slice(&chunk);
    }

    assert_eq!(result.as_slice(), original_data);
}

#[tokio::test]
async fn test_stream_decode_multiple_chunks() {
    let encoder = ContentEncoder::new();
    let original_data = b"Data split across multiple chunks for streaming test";

    let encoded = encoder.encode(original_data, Encoding::Gzip)
        .expect("Encoding should succeed");

    // Split encoded data into multiple chunks
    let chunk_size = 10;
    let chunks: Vec<Bytes> = encoded
        .chunks(chunk_size)
        .map(|chunk| Bytes::from(chunk.to_vec()))
        .collect();

    assert!(chunks.len() > 1, "Should have multiple chunks");

    let stream = stream::iter(chunks);
    let mut decode_stream = encoder.decode_stream(stream, Encoding::Gzip);

    let mut result = Vec::new();
    while let Some(chunk_result) = decode_stream.next().await {
        let chunk = chunk_result.expect("Stream decoding should succeed");
        result.extend_from_slice(&chunk);
    }

    assert_eq!(result.as_slice(), original_data);
}

#[tokio::test]
async fn test_stream_decode_empty_stream() {
    let encoder = ContentEncoder::new();
    let empty_stream = stream::iter(vec![]);

    let mut decode_stream = encoder.decode_stream(empty_stream, Encoding::Gzip);

    let mut result = Vec::new();
    while let Some(chunk_result) = decode_stream.next().await {
        let chunk = chunk_result.expect("Stream decoding should succeed");
        result.extend_from_slice(&chunk);
    }

    assert!(result.is_empty());
}
