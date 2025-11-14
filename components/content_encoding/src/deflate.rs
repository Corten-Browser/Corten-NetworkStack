use flate2::write::{DeflateEncoder, DeflateDecoder};
use flate2::Compression;
use network_errors::NetworkError;
use std::io::Write;

/// Encode data using deflate compression
pub fn encode(data: &[u8]) -> Result<Vec<u8>, NetworkError> {
    let mut encoder = DeflateEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(data)
        .map_err(|e| NetworkError::ProtocolError(format!("Deflate encoding failed: {}", e)))?;
    encoder.finish()
        .map_err(|e| NetworkError::ProtocolError(format!("Deflate finish failed: {}", e)))
}

/// Decode deflate-compressed data
pub fn decode(data: &[u8]) -> Result<Vec<u8>, NetworkError> {
    let mut decoder = DeflateDecoder::new(Vec::new());
    decoder.write_all(data)
        .map_err(|e| NetworkError::ProtocolError(format!("Deflate decoding failed: {}", e)))?;
    decoder.finish()
        .map_err(|e| NetworkError::ProtocolError(format!("Deflate finish failed: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deflate_roundtrip() {
        let data = b"Hello, deflate!";
        let encoded = encode(data).unwrap();
        let decoded = decode(&encoded).unwrap();
        assert_eq!(decoded.as_slice(), data);
    }

    #[test]
    fn test_deflate_invalid_data() {
        let invalid = b"not deflate data";
        let result = decode(invalid);
        assert!(result.is_err());
    }
}
