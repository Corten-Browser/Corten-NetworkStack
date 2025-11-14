# http1_protocol

**Type**: Protocol
**Tech Stack**: Rust, Tokio, Cargo
**Version**: 0.1.0
**Estimated Size**: ~12,000 lines

## Responsibility

HTTP/1.1 client implementation with connection pooling, keep-alive, pipelining support

## Structure

```
http1_protocol/
├── src/
│   ├── lib.rs       # Main library entry point
│   └── ...          # Implementation modules
├── tests/
│   ├── unit/        # Unit tests
│   └── integration/ # Integration tests
├── Cargo.toml       # Rust package manifest
├── CLAUDE.md        # Development instructions for Claude Code
└── README.md        # This file
```

## Development

This component is part of the Corten-NetworkStack multi-component architecture.

See `CLAUDE.md` for detailed development instructions.

## Dependencies

Level 2 component.
Dependencies will be specified in Cargo.toml after contract generation.

## Testing

```bash
cargo test                    # Run all tests
cargo test --test unit        # Unit tests only
cargo test --test integration # Integration tests only
cargo clippy                  # Linting
cargo fmt                     # Formatting
```

## Documentation

```bash
cargo doc --open  # Generate and open documentation
```
