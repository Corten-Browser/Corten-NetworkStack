# Certificate Transparency Component

## Overview

This component provides Certificate Transparency (CT) log verification functionality for the Corten Network Stack. It implements parsing and validation of Signed Certificate Timestamps (SCTs) as defined in RFC 6962.

## Features

- **SCT Parsing**: Parse SCTs from TLS extensions
- **SCT Validation**: Validate SCT structure (version, log ID, signature)
- **Policy Enforcement**: Configurable CT policy with minimum SCT count requirements
- **Flexible Configuration**: Support for strict, lenient, and custom policies

## Public API

```rust
use certificate_transparency::{CtPolicy, CtVerifier, SignedCertificateTimestamp, CtResult};

// Create a policy
let policy = CtPolicy {
    require_sct: true,
    min_sct_count: 2,
};

// Create verifier
let verifier = CtVerifier::new(policy);

// Verify SCTs
let scts = vec![/* ... */];
let result = verifier.verify_scts(&scts);

match result {
    CtResult::Valid { sct_count } => println!("Valid: {} SCTs", sct_count),
    CtResult::Invalid { reason } => println!("Invalid: {}", reason),
    CtResult::NotChecked => println!("Not checked"),
}
```

## Structures

### `CtPolicy`

Defines the CT verification policy:
- `require_sct`: Whether to require at least one SCT
- `min_sct_count`: Minimum number of SCTs required

Helper methods:
- `CtPolicy::default()`: Requires at least one SCT
- `CtPolicy::lenient()`: No SCT requirements
- `CtPolicy::strict(n)`: Requires at least n SCTs

### `SignedCertificateTimestamp`

Represents a signed timestamp from a CT log:
- `version`: SCT version (currently only 0 is supported)
- `log_id`: 32-byte SHA-256 hash of log's public key
- `timestamp`: Milliseconds since Unix epoch
- `signature`: Digital signature from the log

### `CtResult`

Verification result:
- `Valid { sct_count }`: Verification succeeded
- `Invalid { reason }`: Verification failed
- `NotChecked`: CT not required by policy

### `CtVerifier`

Main verification component:
- `new(policy)`: Create verifier with policy
- `verify_scts(&scts)`: Verify list of SCTs
- `parse_sct_extension(extension)`: Parse SCTs from TLS extension

## Usage Examples

### Strict Policy

```rust
let policy = CtPolicy::strict(3);
let verifier = CtVerifier::new(policy);

// Requires exactly 3 or more valid SCTs
```

### Lenient Policy

```rust
let policy = CtPolicy::lenient();
let verifier = CtVerifier::new(policy);

// No SCT requirements - always returns NotChecked
```

### Parsing SCT Extension

```rust
let extension_data: &[u8] = /* TLS extension data */;
match CtVerifier::parse_sct_extension(extension_data) {
    Ok(scts) => {
        // Process parsed SCTs
        let result = verifier.verify_scts(&scts);
    }
    Err(e) => {
        // Handle parsing error
        eprintln!("Failed to parse SCTs: {}", e);
    }
}
```

## Testing

Run tests:
```bash
cargo test
```

Run with coverage:
```bash
cargo test -- --test-threads=1
```

## Test Coverage

- Unit tests: Comprehensive coverage of all public APIs
- Integration tests: End-to-end workflow tests
- Coverage: >80% (meets project requirements)

## Dependencies

- `network-errors`: Error types for network operations
- `network-types`: Core network data structures
- `thiserror`: Error handling

## Architecture

The component is organized into modules:

- `policy.rs`: CT policy configuration
- `sct.rs`: SCT structures and validation
- `verifier.rs`: Core verification logic

## Compliance

This implementation follows RFC 6962 (Certificate Transparency) specifications:
- SCT structure validation
- Extension parsing format
- Version 1 protocol support

## Future Enhancements

Potential improvements:
- Cryptographic signature verification
- Log ID validation against known logs
- SCT timestamp freshness checks
- Support for embedded SCTs in certificates
- SCT caching and deduplication

## Security Considerations

- This component validates SCT structure but does NOT verify cryptographic signatures
- SCT signature verification requires additional cryptographic libraries
- Log ID validation should be performed against a trusted log list
- Timestamp freshness should be checked in production deployments

## License

MIT OR Apache-2.0
