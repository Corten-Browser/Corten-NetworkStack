# Content Encoding Component

## Overview

This component provides HTTP content encoding and decoding functionality for the Corten Network Stack. It supports multiple compression algorithms including gzip, deflate, and brotli, with both synchronous and streaming decompression capabilities.

## Features

- **Multiple Encoding Support**: Gzip, Deflate, Brotli, and Identity (no encoding)
- **Synchronous Operations**: Encode and decode complete data buffers
- **Streaming Decompression**: Decompress data streams asynchronously
- **Accept-Encoding Header Generation**: Automatic header generation for HTTP requests

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
content_encoding = { path = "../content_encoding" }
```

## Usage

### Basic Encoding/Decoding

```rust
use content_encoding::{ContentEncoder, Encoding};

let encoder = ContentEncoder::new();
let data = b"Hello, World!";

// Encode with gzip
let compressed = encoder.encode(data, Encoding::Gzip)?;

// Decode
let decompressed = encoder.decode(&compressed, Encoding::Gzip)?;

assert_eq!(decompressed.as_slice(), data);
```

### Streaming Decompression

```rust
use content_encoding::{ContentEncoder, Encoding};
use bytes::Bytes;
use futures::stream::{self, StreamExt};

let encoder = ContentEncoder::new();
let data = b"Streaming data example";

// Encode first
let encoded = encoder.encode(data, Encoding::Gzip)?;

// Create a stream
let stream = stream::iter(vec![Bytes::from(encoded)]);

// Decode the stream
let mut decode_stream = encoder.decode_stream(stream, Encoding::Gzip);

while let Some(chunk_result) = decode_stream.next().await {
    let chunk = chunk_result?;
    // Process chunk
}
```

### Accept-Encoding Header

```rust
use content_encoding::ContentEncoder;

let encoder = ContentEncoder::new();
let accept_encoding = encoder.get_accept_encoding();

// Returns: "gzip, deflate, br"
```

## Supported Encodings

| Encoding | Description | Compression Ratio | Speed |
|----------|-------------|-------------------|-------|
| Gzip | DEFLATE with gzip wrapper | Good | Fast |
| Deflate | Raw DEFLATE stream | Good | Fast |
| Brotli | Modern compression | Best | Medium |
| Identity | No compression | None | Fastest |

## API Reference

### `ContentEncoder`

Main encoder/decoder struct with all supported encodings.

#### Methods

- `new()` - Create a new encoder with all supported encodings
- `encode(&self, data: &[u8], encoding: Encoding) -> Result<Vec<u8>, NetworkError>` - Encode data
- `decode(&self, data: &[u8], encoding: Encoding) -> Result<Vec<u8>, NetworkError>` - Decode data
- `decode_stream(&self, stream: impl Stream<Item = Bytes>, encoding: Encoding) -> impl Stream<Item = Result<Bytes, NetworkError>>` - Decode a stream
- `get_accept_encoding(&self) -> String` - Get Accept-Encoding header value

### `Encoding`

Enum representing supported encoding types.

#### Variants

- `Gzip` - Gzip compression
- `Deflate` - Deflate compression
- `Brotli` - Brotli compression
- `Identity` - No encoding (pass-through)

## Error Handling

All encoding/decoding operations return `Result<T, NetworkError>`. Errors can occur when:

- Invalid compressed data is provided
- Compression/decompression fails
- I/O errors occur during streaming

```rust
use content_encoding::{ContentEncoder, Encoding};

let encoder = ContentEncoder::new();
let corrupted_data = b"Not valid gzip data";

match encoder.decode(corrupted_data, Encoding::Gzip) {
    Ok(decoded) => println!("Decoded: {:?}", decoded),
    Err(e) => eprintln!("Decoding failed: {}", e),
}
```

## Performance Characteristics

### Time Complexity

- Encoding: O(n) where n is input size
- Decoding: O(n) where n is output size
- Streaming: O(n) with constant memory overhead

### Space Complexity

- Synchronous operations: O(n) for output buffer
- Streaming operations: O(1) for fixed-size buffers (4096 bytes)

## Testing

The component includes comprehensive tests covering:

- Round-trip encoding/decoding for all formats
- Empty data handling
- Large data handling (1MB+)
- Corrupted data error handling
- Streaming decompression
- Cross-encoding validation

Run tests (once workspace issues are resolved):

```bash
cargo test
```

Run with coverage:

```bash
cargo test --all-features --no-fail-fast -- --test-threads=1
cargo tarpaulin --out Html
```

## Dependencies

- `flate2` - Gzip and Deflate compression
- `brotli` - Brotli compression
- `bytes` - Efficient byte buffers
- `futures` - Async stream support
- `network-errors` - Error types
- `network-types` - Core network types

## Known Issues

### Workspace Configuration

The workspace has dependency naming issues in `proxy_support` component that prevent compilation:
- Uses `dns_resolver` instead of `dns-resolver`
- Uses `network_errors` instead of `network-errors`
- Uses `tls_manager` instead of `tls-manager`

This prevents running tests even though the code is correct. The orchestrator needs to fix these workspace-level dependency names.

## Architecture

The component is organized into focused modules:

```
src/
├── lib.rs           # Public API and ContentEncoder
├── gzip.rs          # Gzip encode/decode
├── deflate.rs       # Deflate encode/decode
├── brotli_impl.rs   # Brotli encode/decode
└── stream.rs        # Streaming decompression
```

### Design Decisions

1. **Separate modules per encoding**: Each compression algorithm in its own module for clarity
2. **Streaming support**: Async streams for large data and network responses
3. **Error handling**: All errors map to NetworkError for consistency
4. **No buffering in Identity**: Pass-through for unencoded data

## Future Enhancements

Potential improvements (not currently implemented):

- Quality/compression level configuration
- Custom window sizes for deflate/gzip
- Chunked streaming encoding (currently only decoding)
- Additional encodings (zstd, lz4)
- Compression statistics and metrics

## License

Part of the Corten Network Stack project.

## Contributing

Follow TDD practices:
1. Write test first (RED)
2. Implement to pass test (GREEN)
3. Refactor (REFACTOR)
4. All tests must pass with 100% pass rate
5. Maintain 80%+ coverage
