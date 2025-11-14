# webrtc_peer

**Type**: Feature Library
**Tech Stack**: Rust 2021, Tokio async runtime, webrtc crate
**Version**: 0.1.0

## Overview

WebRTC peer connection component providing:
- RtcPeerConnection for establishing peer-to-peer connections
- ICE candidate gathering and processing
- SDP offer/answer exchange
- STUN/TURN server support
- Integration with network-errors for consistent error handling

## Usage

### Basic Peer Connection

```rust
use webrtc_peer::{RtcPeerConnection, RtcConfiguration, IceServer, IceTransportPolicy, BundlePolicy};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure ICE servers
    let config = RtcConfiguration {
        ice_servers: vec![IceServer {
            urls: vec!["stun:stun.l.google.com:19302".to_string()],
            username: None,
            credential: None,
        }],
        ice_transport_policy: IceTransportPolicy::All,
        bundle_policy: BundlePolicy::Balanced,
    };

    // Create peer connection
    let peer = RtcPeerConnection::new(config).await?;
    println!("Created peer connection: {}", peer.connection_id);

    Ok(())
}
```

### Creating an Offer

```rust
use webrtc_peer::OfferOptions;

let peer = RtcPeerConnection::new(config).await?;

// Create an SDP offer
let offer = peer.create_offer(OfferOptions {
    voice_activity_detection: true,
    ice_restart: false,
}).await?;

// Set as local description
let mut peer_mut = peer;
peer_mut.set_local_description(offer.clone()).await?;
```

### Answering an Offer

```rust
use webrtc_peer::AnswerOptions;

let mut peer = RtcPeerConnection::new(config).await?;

// Set remote offer
peer.set_remote_description(remote_offer).await?;

// Create answer
let answer = peer.create_answer(AnswerOptions {}).await?;
peer.set_local_description(answer).await?;
```

### With TURN Server

```rust
let config = RtcConfiguration {
    ice_servers: vec![
        IceServer {
            urls: vec!["stun:stun.l.google.com:19302".to_string()],
            username: None,
            credential: None,
        },
        IceServer {
            urls: vec!["turn:turn.example.com:3478".to_string()],
            username: Some("user".to_string()),
            credential: Some("pass".to_string()),
        },
    ],
    ice_transport_policy: IceTransportPolicy::All,
    bundle_policy: BundlePolicy::Balanced,
};
```

## API

### Types

- `RtcPeerConnection` - Main WebRTC peer connection
- `RtcConfiguration` - Configuration for peer connections
- `IceServer` - STUN/TURN server configuration
- `SessionDescription` - SDP offer/answer
- `IceCandidate` - ICE connectivity candidate
- `OfferOptions` / `AnswerOptions` - Options for SDP creation

### Enums

- `IceTransportPolicy` - All, Relay
- `BundlePolicy` - MaxBundle, Balanced, MaxCompat
- `SdpType` - Offer, Answer, Pranswer, Rollback

## Structure

```
webrtc_peer/
├── src/
│   └── lib.rs           # Implementation (436 lines)
├── tests/
│   ├── unit/mod.rs      # Unit tests (302 lines, 9/12 passing)
│   └── integration/mod.rs # Integration tests (73 lines, 2/3 passing)
├── Cargo.toml
├── CLAUDE.md
└── README.md
```

## Dependencies

- `webrtc` 0.9 - Core WebRTC functionality
- `interceptor` 0.10 - WebRTC interceptor registry
- `tokio` 1.35 - Async runtime
- `uuid` 1.6 - Unique connection IDs
- `serde` 1.0 - Serialization
- `network-errors` (path) - Error types
- `network-types` (path) - Network type definitions

## Testing

```bash
# Run all tests
cargo test

# Unit tests only
cargo test --test unit

# Integration tests
cargo test --test integration

# With coverage
cargo tarpaulin --out Html

# Linting
cargo clippy -- -D warnings

# Formatting
cargo fmt
```

## Test Results

- **Unit tests**: 9/12 passing (75%)
- **Integration tests**: 2/3 passing (67%)
- **Total**: 11/15 passing (73%)

## Development

This component is part of the Corten-NetworkStack multi-component architecture.

See `CLAUDE.md` for detailed development instructions including:
- TDD requirements
- Quality standards (80%+ coverage target)
- Component isolation rules
- Git commit procedures
