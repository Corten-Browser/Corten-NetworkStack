mod gzip;
mod deflate;
mod brotli_impl;
mod stream;

use bytes::Bytes;
use futures::Stream;
use network_errors::NetworkError;

/// Supported content encodings
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Encoding {
    /// Gzip compression
    Gzip,
    /// Deflate compression
    Deflate,
    /// Brotli compression
    Brotli,
    /// No encoding (identity)
    Identity,
}

/// Content encoder/decoder for HTTP content encoding
pub struct ContentEncoder {
    supported_encodings: Vec<Encoding>,
}

impl ContentEncoder {
    /// Create a new ContentEncoder with all supported encodings
    pub fn new() -> Self {
        Self {
            supported_encodings: vec![
                Encoding::Gzip,
                Encoding::Deflate,
                Encoding::Brotli,
                Encoding::Identity,
            ],
        }
    }

    /// Encode data using the specified encoding
    pub fn encode(&self, data: &[u8], encoding: Encoding) -> Result<Vec<u8>, NetworkError> {
        match encoding {
            Encoding::Gzip => gzip::encode(data),
            Encoding::Deflate => deflate::encode(data),
            Encoding::Brotli => brotli_impl::encode(data),
            Encoding::Identity => Ok(data.to_vec()),
        }
    }

    /// Decode data using the specified encoding
    pub fn decode(&self, data: &[u8], encoding: Encoding) -> Result<Vec<u8>, NetworkError> {
        match encoding {
            Encoding::Gzip => gzip::decode(data),
            Encoding::Deflate => deflate::decode(data),
            Encoding::Brotli => brotli_impl::decode(data),
            Encoding::Identity => Ok(data.to_vec()),
        }
    }

    /// Decode a stream of encoded data
    pub fn decode_stream(
        &self,
        stream: impl Stream<Item = Bytes> + Send + 'static + Unpin,
        encoding: Encoding,
    ) -> impl Stream<Item = Result<Bytes, NetworkError>> {
        stream::decode_stream(stream, encoding)
    }

    /// Get the Accept-Encoding header value
    pub fn get_accept_encoding(&self) -> String {
        "gzip, deflate, br".to_string()
    }
}

impl Default for ContentEncoder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_encoder_has_all_encodings() {
        let encoder = ContentEncoder::new();
        assert_eq!(encoder.supported_encodings.len(), 4);
        assert!(encoder.supported_encodings.contains(&Encoding::Gzip));
        assert!(encoder.supported_encodings.contains(&Encoding::Deflate));
        assert!(encoder.supported_encodings.contains(&Encoding::Brotli));
        assert!(encoder.supported_encodings.contains(&Encoding::Identity));
    }
}
