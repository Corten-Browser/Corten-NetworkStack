#!/usr/bin/env python3
"""
Script to create all 13 network stack components
Following the architecture design from docs/ARCHITECTURE.md
"""

import os
import subprocess

# Component definitions: (name, size_lines, level, responsibility)
COMPONENTS = [
    # Level 0: Base
    ("network_types", 6000, 0, "Core types: NetworkRequest, NetworkResponse, HTTP enums, request/response structures"),
    ("network_errors", 4000, 0, "Error handling: NetworkError enum, Result types, error conversion traits"),

    # Level 1: Core
    ("dns_resolver", 6000, 1, "DNS resolution with DNS-over-HTTPS support, caching, and async resolution"),
    ("tls_manager", 10000, 1, "TLS 1.2/1.3 configuration, certificate validation, ALPN negotiation, session resumption"),
    ("cookie_manager", 5000, 1, "Cookie storage, cookie jar implementation, Set-Cookie parsing, cookie policy enforcement"),
    ("http_cache", 10000, 1, "HTTP cache storage backend, cache policy enforcement, freshness validation, ETags"),

    # Level 2: Protocols
    ("http1_protocol", 12000, 2, "HTTP/1.1 client implementation with connection pooling, keep-alive, pipelining support"),
    ("http2_protocol", 12000, 2, "HTTP/2 client with multiplexing, stream prioritization, flow control, server push"),
    ("http3_protocol", 10000, 2, "HTTP/3 and QUIC protocol implementation with 0-RTT, connection migration"),
    ("websocket_protocol", 8000, 2, "WebSocket client with frame parsing/encoding, ping/pong, compression extensions"),
    ("webrtc_peer", 10000, 2, "WebRTC peer connections, ICE gathering, STUN/TURN, SDP negotiation"),
    ("webrtc_channels", 5000, 2, "WebRTC data channels, SCTP transport, reliable/unreliable messaging"),

    # Level 3: Integration
    ("network_stack", 7000, 3, "Main NetworkStack trait implementation, protocol orchestration, message bus integration"),
]

def create_component(name, size_lines, level, responsibility):
    """Create a single component with all necessary files"""
    print(f"\n📦 Creating component: {name}")

    # Create directory structure
    comp_dir = f"components/{name}"
    os.makedirs(f"{comp_dir}/src", exist_ok=True)
    os.makedirs(f"{comp_dir}/tests/unit", exist_ok=True)
    os.makedirs(f"{comp_dir}/tests/integration", exist_ok=True)

    # Read template
    with open("claude-orchestration-system/templates/component-generic.md", "r") as f:
        template = f.read()

    # Replace variables
    estimated_tokens = size_lines * 10  # 10:1 ratio
    tech_stack = "Rust 2021 edition, Tokio async runtime, Cargo build system"

    claude_md = template.replace("{{COMPONENT_NAME}}", name)
    claude_md = claude_md.replace("{{TECH_STACK}}", tech_stack)
    claude_md = claude_md.replace("{{CURRENT_TOKENS}}", "0")  # Initially zero
    claude_md = claude_md.replace("{{COMPONENT_RESPONSIBILITY}}", responsibility)
    claude_md = claude_md.replace("./", "/home/user/Corten-NetworkStack")
    claude_md = claude_md.replace("0.1.0", "0.1.0")  # Version from metadata

    # Write CLAUDE.md
    with open(f"{comp_dir}/CLAUDE.md", "w") as f:
        f.write(claude_md)

    # Create README.md
    readme = f"""# {name}

**Type**: {"Base" if level == 0 else "Core" if level == 1 else "Protocol" if level == 2 else "Integration"}
**Tech Stack**: Rust, Tokio, Cargo
**Version**: 0.1.0
**Estimated Size**: ~{size_lines:,} lines

## Responsibility

{responsibility}

## Structure

```
{name}/
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

Level {level} component.
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
"""

    with open(f"{comp_dir}/README.md", "w") as f:
        f.write(readme)

    # Create basic Cargo.toml
    cargo_toml = f"""[package]
name = "{name.replace('_', '-')}"
version = "0.1.0"
edition = "2021"
rust-version = "1.75"

[dependencies]
# Dependencies will be added during development

[dev-dependencies]
# Test dependencies will be added during development

[[test]]
name = "unit"
path = "tests/unit/mod.rs"

[[test]]
name = "integration"
path = "tests/integration/mod.rs"
"""

    with open(f"{comp_dir}/Cargo.toml", "w") as f:
        f.write(cargo_toml)

    # Create basic lib.rs
    lib_rs = f"""//! {name} component
//!
//! {responsibility}

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

// TODO: Implement component
"""

    with open(f"{comp_dir}/src/lib.rs", "w") as f:
        f.write(lib_rs)

    # Create test file stubs
    with open(f"{comp_dir}/tests/unit/mod.rs", "w") as f:
        f.write(f"// Unit tests for {name}\n")

    with open(f"{comp_dir}/tests/integration/mod.rs", "w") as f:
        f.write(f"// Integration tests for {name}\n")

    print(f"   ✅ Created: {comp_dir}")
    print(f"   📝 Files: CLAUDE.md, README.md, Cargo.toml, lib.rs, test stubs")

def main():
    print("=" * 70)
    print("Creating 13 Network Stack Components")
    print("=" * 70)

    for name, size_lines, level, responsibility in COMPONENTS:
        create_component(name, size_lines, level, responsibility)

    # Add all to git
    print("\n📌 Adding components to git repository...")
    subprocess.run(["git", "add", "components/"], check=False)

    print("\n✅ All 13 components created successfully!")
    print("\nComponents by level:")
    print("  Level 0 (Base):        network_types, network_errors")
    print("  Level 1 (Core):        dns_resolver, tls_manager, cookie_manager, http_cache")
    print("  Level 2 (Protocol):    http1_protocol, http2_protocol, http3_protocol, websocket_protocol, webrtc_peer, webrtc_channels")
    print("  Level 3 (Integration): network_stack")
    print("\nReady for Phase 3 (contract generation)")

if __name__ == "__main__":
    main()
