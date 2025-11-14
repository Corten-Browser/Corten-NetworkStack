use flate2::write::{GzEncoder, GzDecoder};
use flate2::Compression;
use network_errors::NetworkError;
use std::io::Write;

/// Encode data using gzip compression
pub fn encode(data: &[u8]) -> Result<Vec<u8>, NetworkError> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(data)
        .map_err(|e| NetworkError::ProtocolError(format!("Gzip encoding failed: {}", e)))?;
    encoder.finish()
        .map_err(|e| NetworkError::ProtocolError(format!("Gzip finish failed: {}", e)))
}

/// Decode gzip-compressed data
pub fn decode(data: &[u8]) -> Result<Vec<u8>, NetworkError> {
    let mut decoder = GzDecoder::new(Vec::new());
    decoder.write_all(data)
        .map_err(|e| NetworkError::ProtocolError(format!("Gzip decoding failed: {}", e)))?;
    decoder.finish()
        .map_err(|e| NetworkError::ProtocolError(format!("Gzip finish failed: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gzip_roundtrip() {
        let data = b"Hello, gzip!";
        let encoded = encode(data).unwrap();
        let decoded = decode(&encoded).unwrap();
        assert_eq!(decoded.as_slice(), data);
    }

    #[test]
    fn test_gzip_invalid_data() {
        let invalid = b"not gzip data";
        let result = decode(invalid);
        assert!(result.is_err());
    }
}
