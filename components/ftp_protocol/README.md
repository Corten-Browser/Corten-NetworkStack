# FTP Protocol Component

**Type**: Protocol (Level 2)
**Version**: 0.1.0
**Tech Stack**: Rust, Tokio async runtime

## Overview

Basic FTP client implementation providing:
- Control connection management (port 21)
- Command/response protocol parsing
- Passive mode (PASV command)
- Active mode (PORT command) support
- Basic authentication (USER, PASS)
- File operations (LIST, RETR, STOR)
- Proper FTP response code parsing (2xx, 3xx, 4xx, 5xx)

## Public API

### FtpClient

```rust
use ftp_protocol::{FtpClient, FtpConfig};
use std::time::Duration;

// Create client
let config = FtpConfig {
    timeout: Duration::from_secs(30),
    passive_mode: true,
};
let mut client = FtpClient::new(config);

// Connect and authenticate
client.connect("ftp.example.com", 21).await?;
client.login("username", "password").await?;

// List files
let files = client.list(Some("/path")).await?;

// Download file
let data = client.download("/path/file.txt").await?;

// Upload file
client.upload("/path/newfile.txt", &data).await?;

// Disconnect
client.quit().await?;
```

### FTP Response Parsing

```rust
use ftp_protocol::responses::parse_response;

let response = parse_response("220 Service ready\r\n")?;
assert_eq!(response.code, 220);
assert!(response.is_success());
```

### Command Formatting

```rust
use ftp_protocol::commands::*;

let cmd = format_user("testuser");  // "USER testuser\r\n"
let cmd = format_pasv();            // "PASV\r\n"
let cmd = format_list(Some("/"));   // "LIST /\r\n"
```

## FTP Commands Implemented

- `USER <username>` - Set username
- `PASS <password>` - Set password
- `PASV` - Enter passive mode
- `PORT <address>` - Enter active mode (basic support)
- `LIST [path]` - List directory contents
- `RETR <filename>` - Download file
- `STOR <filename>` - Upload file
- `QUIT` - Disconnect from server

## FTP Response Codes

The client handles standard FTP response codes:
- `1xx` - Positive preliminary (continuation)
- `2xx` - Positive completion (success)
- `3xx` - Positive intermediate (needs more input)
- `4xx` - Transient negative completion (temporary error)
- `5xx` - Permanent negative completion (permanent error)

## Dependencies

- `network-types` - Core network types
- `network-errors` - Error handling (NetworkError, NetworkResult)
- `tokio` - Async runtime (net, io-util, time features)
- `bytes` - Byte buffer utilities

## Development

### Running Tests

```bash
# Run all tests
cargo test

# Run unit tests only
cargo test --test unit

# Run with output
cargo test -- --nocapture
```

### Test Coverage

- **Total Tests**: 21 (19 unit + 2 lib + placeholder integration)
- **Coverage**: 100% test pass rate
- **Response parsing**: 7 tests
- **Command formatting**: 8 tests
- **Client API**: 4 tests
- **Library tests**: 2 tests

### Code Structure

```
src/
├── lib.rs          - Public API exports
├── client.rs       - FtpClient implementation
├── commands.rs     - FTP command formatting
└── responses.rs    - FTP response parsing

tests/
├── unit/
│   ├── test_client.rs
│   ├── test_commands.rs
│   ├── test_responses.rs
│   └── mod.rs
└── integration/
    └── mod.rs
```

## Features and Limitations

### Supported Features
✅ Passive mode (PASV)
✅ Basic authentication (USER/PASS)
✅ File listing (LIST)
✅ File download (RETR)
✅ File upload (STOR)
✅ Proper response parsing
✅ Connection timeout handling

### Current Limitations
❌ Active mode PORT command (structure exists, not fully tested)
❌ Binary vs ASCII mode selection
❌ Resume/restart transfers
❌ TLS/SSL (FTPS)
❌ IPv6 support

### Future Enhancements
- Add FTPS (FTP over TLS) support
- Implement full active mode with PORT
- Add binary/ASCII mode switching
- Support for resume (REST command)
- Better error messages with context
- Connection pooling
- Retry logic

## Architecture

The component follows a layered approach:

1. **Commands Layer** (`commands.rs`): Formats FTP commands according to RFC 959
2. **Responses Layer** (`responses.rs`): Parses FTP server responses
3. **Client Layer** (`client.rs`): Orchestrates commands/responses for high-level operations

All async I/O is handled via Tokio's TcpStream with proper timeout handling.

## Error Handling

Uses `NetworkError` from `network-errors` component:
- `ConnectionFailed` - TCP connection issues
- `ProtocolError` - FTP protocol violations
- `Timeout` - Operation timeouts
- `Io` - Low-level I/O errors

## Testing Strategy

- **Unit tests**: Test individual functions in isolation (commands, response parsing)
- **Integration tests**: Test complete workflows (requires real/mock FTP server)
- **Library tests**: Test internal module functionality

All tests follow TDD (test-driven development) methodology.

## License

MIT OR Apache-2.0
