# url_handlers

Data URL and File URL handling with security policies for the Corten Network Stack.

## Overview

The `url_handlers` component provides secure handling for non-HTTP URL schemes:

- **Data URLs** (`data:`): Parse and decode data URLs with base64 and MIME type support
- **File URLs** (`file:///`): Read local files with security policies and path validation

## Features

### Data URL Handler
- Parse RFC 2397 data URLs
- Base64 decoding
- MIME type extraction
- Character set detection
- URL decoding for plain text data

### File URL Handler
- Secure file reading from `file://` URLs
- Path allowlisting
- Directory traversal prevention
- Same-origin policy enforcement

## Usage

### Data URLs

```rust
use url_handlers::DataUrlHandler;

// Parse a base64 data URL
let url = "data:text/plain;base64,SGVsbG8gV29ybGQ=";
let data = DataUrlHandler::parse(url)?;

assert_eq!(data.mime_type, "text/plain");
assert_eq!(String::from_utf8(data.data).unwrap(), "Hello World");

// Parse plain text data URL
let url = "data:text/plain,Hello%20World";
let data = DataUrlHandler::parse(url)?;
assert_eq!(String::from_utf8(data.data).unwrap(), "Hello World");
```

### File URLs

```rust
use url_handlers::{FileUrlHandler, FileSecurityPolicy};
use std::path::PathBuf;

// Create security policy
let policy = FileSecurityPolicy {
    allow_directory_traversal: false,
    allowed_paths: vec![PathBuf::from("/tmp")],
};

// Create file handler
let handler = FileUrlHandler::new(policy);

// Read file (async)
let url = "file:///tmp/example.txt";
let data = handler.read(url).await?;
```

## Security

### File Access Control

The `FileSecurityPolicy` provides fine-grained control over file access:

- **Path Allowlisting**: Only files within specified paths can be accessed
- **Directory Traversal Prevention**: Paths containing `..` can be blocked
- **Path Canonicalization**: Resolves symlinks and relative paths for security checks

```rust
let policy = FileSecurityPolicy {
    // Block directory traversal by default
    allow_directory_traversal: false,

    // Only allow access to these paths
    allowed_paths: vec![
        PathBuf::from("/var/www/public"),
        PathBuf::from("/tmp/uploads"),
    ],
};
```

### Security Best Practices

1. **Always use path allowlisting** - Never allow all paths
2. **Disable directory traversal** unless specifically needed
3. **Canonicalize paths** before checking (done automatically)
4. **Validate URLs** before passing to handlers

## API Reference

### DataUrlHandler

```rust
impl DataUrlHandler {
    pub fn is_data_url(url: &str) -> bool;
    pub fn parse(url: &str) -> Result<DataUrlData, NetworkError>;
}
```

### FileUrlHandler

```rust
impl FileUrlHandler {
    pub fn new(security_policy: FileSecurityPolicy) -> Self;
    pub fn is_file_url(url: &str) -> bool;
    pub fn is_allowed(&self, path: &Path) -> bool;
    pub async fn read(&self, url: &str) -> Result<Vec<u8>, NetworkError>;
}
```

### FileSecurityPolicy

```rust
pub struct FileSecurityPolicy {
    pub allow_directory_traversal: bool,
    pub allowed_paths: Vec<PathBuf>,
}
```

### DataUrlData

```rust
pub struct DataUrlData {
    pub mime_type: String,
    pub data: Vec<u8>,
    pub charset: Option<String>,
}
```

## Development

### Running Tests

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_parse_data_url_with_base64
```

### Test Coverage

```bash
cargo tarpaulin --out Html
```

### Linting

```bash
cargo clippy -- -D warnings
```

### Formatting

```bash
cargo fmt
```

## Dependencies

- `network_errors`: Error types
- `tokio`: Async runtime for file I/O
- `base64`: Base64 decoding
- `mime`: MIME type parsing (optional, used for validation)

## License

MIT OR Apache-2.0
