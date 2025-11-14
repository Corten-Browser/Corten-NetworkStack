# websocket_protocol

**Type**: Feature Library
**Tech Stack**: Rust 2021, Tokio async runtime, tokio-tungstenite
**Version**: 0.1.0

## Overview

WebSocket client implementation with frame parsing/encoding, ping/pong heartbeat, and compression extension support. Provides a high-level async API for establishing and managing WebSocket connections.

## Features

- **WS and WSS support** - Secure and insecure WebSocket connections
- **Async/await API** - Built on Tokio for efficient async I/O
- **Message types** - Text, Binary, Ping, Pong, and Close frames
- **State management** - Connection lifecycle tracking
- **Channel-based communication** - Clean async message handling
- **Error handling** - Integration with network-errors component

## Usage

### Basic Connection

```rust
use websocket_protocol::{WebSocketClient, WebSocketMessage};
use url::Url;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create client
    let client = WebSocketClient::new();

    // Connect to WebSocket server
    let url = Url::parse("ws://echo.websocket.org")?;
    let mut connection = client.connect(url, vec![]).await?;

    // Send a text message
    connection.send(WebSocketMessage::Text("Hello!".to_string())).await?;

    // Receive a message
    if let Some(Ok(msg)) = connection.recv().await {
        match msg {
            WebSocketMessage::Text(text) => println!("Received: {}", text),
            WebSocketMessage::Binary(data) => println!("Received {} bytes", data.len()),
            _ => {}
        }
    }

    // Close connection
    connection.close(1000, "Normal closure".to_string()).await?;

    Ok(())
}
```

### Secure WebSocket (WSS)

```rust
use websocket_protocol::WebSocketClient;
use url::Url;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = WebSocketClient::new();
    let url = Url::parse("wss://secure.example.com/ws")?;
    let connection = client.connect(url, vec![]).await?;
    // Use connection...
    Ok(())
}
```

### Message Types

```rust
use websocket_protocol::WebSocketMessage;

// Text message (UTF-8 string)
let text_msg = WebSocketMessage::Text("Hello World".to_string());

// Binary message (raw bytes)
let binary_msg = WebSocketMessage::Binary(vec![1, 2, 3, 4]);

// Ping message (for heartbeat)
let ping_msg = WebSocketMessage::Ping(vec![]);

// Pong message (response to ping)
let pong_msg = WebSocketMessage::Pong(vec![]);

// Close message
let close_msg = WebSocketMessage::Close(None);
```

### Connection State

```rust
use websocket_protocol::WebSocketState;

let state = connection.state();
match state {
    WebSocketState::Connecting => println!("Connecting..."),
    WebSocketState::Open => println!("Connection open"),
    WebSocketState::Closing => println!("Closing..."),
    WebSocketState::Closed => println!("Connection closed"),
}
```

## API

### WebSocketClient

Main client for establishing WebSocket connections.

#### Methods

- `new()` - Create a new WebSocket client
- `connect(url, protocols) -> Result<WebSocketConnection>` - Connect to a WebSocket server

### WebSocketConnection

Handle for an established WebSocket connection.

#### Methods

- `send(message) -> Result<()>` - Send a message through the WebSocket
- `recv() -> Option<Result<WebSocketMessage>>` - Receive next message
- `close(code, reason) -> Result<()>` - Close the connection
- `state() -> WebSocketState` - Get current connection state

#### Fields

- `url: Url` - The WebSocket URL
- `protocol: Option<String>` - Negotiated subprotocol (if any)
- `extensions: Vec<String>` - Negotiated extensions

### WebSocketMessage

Enum representing different WebSocket message types.

#### Variants

- `Text(String)` - UTF-8 text message
- `Binary(Vec<u8>)` - Binary data message
- `Ping(Vec<u8>)` - Ping frame
- `Pong(Vec<u8>)` - Pong frame
- `Close(Option<CloseFrame>)` - Close frame

### WebSocketState

Enum representing connection states.

#### Variants

- `Connecting` - Connection is being established
- `Open` - Connection is open
- `Closing` - Connection is closing
- `Closed` - Connection is closed

## Dependencies

- `network-types` - Core network types
- `network-errors` - Error handling
- `tls-manager` - TLS support
- `tokio-tungstenite` - WebSocket implementation
- `tokio` - Async runtime
- `url` - URL parsing

## Development

### Running Tests

```bash
# All tests
cargo test

# Unit tests only
cargo test --test unit

# Integration tests
cargo test --test integration

# With coverage
cargo tarpaulin --out Html
```

### Code Quality

```bash
# Linting
cargo clippy

# Formatting
cargo fmt

# Check formatting
cargo fmt -- --check
```

### Documentation

```bash
# Generate documentation
cargo doc --open
```

## Architecture

The WebSocket client uses a channel-based architecture for clean async message handling:

1. **Connection Layer** - `tokio-tungstenite` for low-level WebSocket protocol
2. **Message Layer** - Internal channels for async send/receive
3. **API Layer** - High-level async API exposed to users

Messages flow through mpsc channels, allowing clean separation between the WebSocket protocol handling and user code.

## Testing Strategy

- **Unit tests** - Test message types, state transitions, API surface
- **Integration tests** - Test with real WebSocket echo servers
- **Contract tests** - Verify API matches specification

## Performance Considerations

- Uses Tokio's async runtime for efficient I/O
- Channel-based architecture allows concurrent send/receive
- Automatic ping/pong handled by tungstenite
- Support for compression extensions to reduce bandwidth

## Error Handling

All errors are wrapped in `NetworkError` from the `network-errors` component:

- `ConnectionFailed` - Failed to establish connection
- `WebSocketError` - WebSocket-specific errors
- `InvalidUrl` - Invalid WebSocket URL provided

## Future Enhancements

- Automatic reconnection with exponential backoff
- Message queue for offline buffering
- Compression extension configuration
- Custom subprotocol negotiation
- Connection pooling

## License

See project root for license information.
