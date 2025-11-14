use network_errors::NetworkError;
use std::io::{Read, Write};

/// Encode data using brotli compression
pub fn encode(data: &[u8]) -> Result<Vec<u8>, NetworkError> {
    let mut output = Vec::new();
    let mut compressor = brotli::CompressorWriter::new(
        &mut output,
        4096, // buffer size
        11,   // quality (0-11, 11 is best)
        22,   // lg_window_size
    );

    compressor.write_all(data)
        .map_err(|e| NetworkError::ProtocolError(format!("Brotli encoding failed: {}", e)))?;

    compressor.flush()
        .map_err(|e| NetworkError::ProtocolError(format!("Brotli flush failed: {}", e)))?;

    drop(compressor);
    Ok(output)
}

/// Decode brotli-compressed data
pub fn decode(data: &[u8]) -> Result<Vec<u8>, NetworkError> {
    let mut decompressor = brotli::Decompressor::new(data, 4096);
    let mut output = Vec::new();

    decompressor.read_to_end(&mut output)
        .map_err(|e| NetworkError::ProtocolError(format!("Brotli decoding failed: {}", e)))?;

    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_brotli_roundtrip() {
        let data = b"Hello, brotli!";
        let encoded = encode(data).unwrap();
        let decoded = decode(&encoded).unwrap();
        assert_eq!(decoded.as_slice(), data);
    }

    #[test]
    fn test_brotli_invalid_data() {
        let invalid = b"not brotli data";
        let result = decode(invalid);
        assert!(result.is_err());
    }
}
