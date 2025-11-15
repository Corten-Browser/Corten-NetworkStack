use bytes::Bytes;
use futures::stream::{Stream, StreamExt};
use futures::stream;
use network_errors::NetworkError;
use std::io::{Read, Write};
use flate2::write::{GzDecoder, DeflateDecoder};
use crate::Encoding;

/// Decode a stream of encoded bytes
pub fn decode_stream(
    input: impl Stream<Item = Bytes> + Send + 'static + Unpin,
    encoding: Encoding,
) -> impl Stream<Item = Result<Bytes, NetworkError>> {
    match encoding {
        Encoding::Gzip => Box::pin(decode_gzip_stream(input)) as std::pin::Pin<Box<dyn Stream<Item = Result<Bytes, NetworkError>> + Send>>,
        Encoding::Deflate => Box::pin(decode_deflate_stream(input)),
        Encoding::Brotli => Box::pin(decode_brotli_stream(input)),
        Encoding::Identity => Box::pin(decode_identity_stream(input)),
    }
}

/// Decode a gzip stream
fn decode_gzip_stream(
    input: impl Stream<Item = Bytes> + Send + 'static + Unpin,
) -> impl Stream<Item = Result<Bytes, NetworkError>> + Send {
    stream::unfold(
        (input, GzDecoder::new(Vec::new()), Vec::<u8>::new()),
        |(mut input, mut decoder, mut buffer)| async move {
            while let Some(chunk) = input.next().await {
                if let Err(e) = decoder.write_all(&chunk) {
                    return Some((
                        Err(NetworkError::ProtocolError(format!("Gzip stream decoding failed: {}", e))),
                        (input, decoder, buffer),
                    ));
                }

                // Try to flush decoded data
                let decoded = decoder.get_mut().drain(..).collect::<Vec<u8>>();
                if !decoded.is_empty() {
                    return Some((Ok(Bytes::from(decoded)), (input, decoder, buffer)));
                }
            }

            // Finish decoding
            match decoder.finish() {
                Ok(final_data) => {
                    if !final_data.is_empty() {
                        Some((Ok(Bytes::from(final_data)), (input, GzDecoder::new(Vec::new()), buffer)))
                    } else {
                        None
                    }
                }
                Err(e) => Some((
                    Err(NetworkError::ProtocolError(format!("Gzip stream finish failed: {}", e))),
                    (input, GzDecoder::new(Vec::new()), buffer),
                )),
            }
        },
    )
}

/// Decode a deflate stream
fn decode_deflate_stream(
    input: impl Stream<Item = Bytes> + Send + 'static + Unpin,
) -> impl Stream<Item = Result<Bytes, NetworkError>> + Send {
    stream::unfold(
        (input, DeflateDecoder::new(Vec::new())),
        |(mut input, mut decoder)| async move {
            while let Some(chunk) = input.next().await {
                if let Err(e) = decoder.write_all(&chunk) {
                    return Some((
                        Err(NetworkError::ProtocolError(format!("Deflate stream decoding failed: {}", e))),
                        (input, decoder),
                    ));
                }

                // Try to flush decoded data
                let decoded = decoder.get_mut().drain(..).collect::<Vec<u8>>();
                if !decoded.is_empty() {
                    return Some((Ok(Bytes::from(decoded)), (input, decoder)));
                }
            }

            // Finish decoding
            match decoder.finish() {
                Ok(final_data) => {
                    if !final_data.is_empty() {
                        Some((Ok(Bytes::from(final_data)), (input, DeflateDecoder::new(Vec::new()))))
                    } else {
                        None
                    }
                }
                Err(e) => Some((
                    Err(NetworkError::ProtocolError(format!("Deflate stream finish failed: {}", e))),
                    (input, DeflateDecoder::new(Vec::new())),
                )),
            }
        },
    )
}

/// Decode a brotli stream
fn decode_brotli_stream(
    input: impl Stream<Item = Bytes> + Send + 'static + Unpin,
) -> impl Stream<Item = Result<Bytes, NetworkError>> + Send {
    stream::unfold(
        (input, Vec::new()),
        |(mut input, mut buffer)| async move {
            // Collect all chunks first for brotli (it needs complete data)
            while let Some(chunk) = input.next().await {
                buffer.extend_from_slice(&chunk);
            }

            if buffer.is_empty() {
                return None;
            }

            // Decode all at once
            let mut decompressor = brotli::Decompressor::new(&buffer[..], 4096);
            let mut output = Vec::new();

            match decompressor.read_to_end(&mut output) {
                Ok(_) => Some((Ok(Bytes::from(output)), (input, Vec::new()))),
                Err(e) => Some((
                    Err(NetworkError::ProtocolError(format!("Brotli stream decoding failed: {}", e))),
                    (input, Vec::new()),
                )),
            }
        },
    )
}

/// Pass-through stream for identity encoding
fn decode_identity_stream(
    input: impl Stream<Item = Bytes> + Send + 'static + Unpin,
) -> impl Stream<Item = Result<Bytes, NetworkError>> + Send {
    input.map(|chunk| Ok(chunk))
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::stream;

    #[tokio::test]
    async fn test_identity_stream() {
        let data = vec![Bytes::from("test")];
        let input = stream::iter(data.clone());
        let mut output = decode_stream(input, Encoding::Identity);

        let result = output.next().await
            .expect("Stream should have next item")
            .expect("Decoding should succeed");
        assert_eq!(result, data[0]);
    }
}
