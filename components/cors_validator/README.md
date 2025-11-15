# cors_validator

CORS (Cross-Origin Resource Sharing) policy enforcement and validation component for the Corten Network Stack.

## Overview

This component provides comprehensive CORS validation for network requests and responses, handling:
- Same-origin policy enforcement
- Preflight request detection and building
- Credential mode validation
- Access-Control-* header management
- Cross-origin blocking logic

## Features

- **Same-Origin Policy**: Enforce same-origin restrictions or allow cross-origin requests
- **Preflight Handling**: Automatic detection of when preflight OPTIONS requests are needed
- **Credential Management**: Support for cookies and authorization headers with proper CORS rules
- **Header Generation**: Automatic generation of appropriate Access-Control-* headers
- **Wildcard Origins**: Support for wildcard (*) origins with proper credential restrictions

## Public API

### Core Types

```rust
pub struct CorsValidator {
    // Validates requests and responses according to CORS policy
}

pub struct CorsConfig {
    pub enforce_same_origin: bool,
    pub allow_credentials: bool,
}

pub struct CorsResult {
    pub allowed: bool,
    pub reason: Option<String>,
    pub headers_to_add: HeaderMap,
}
```

### Methods

```rust
impl CorsValidator {
    // Create a new CORS validator with configuration
    pub fn new(config: CorsConfig) -> Self;

    // Validate a network request
    pub fn validate_request(&self, request: &NetworkRequest, origin: &str) -> CorsResult;

    // Validate a network response
    pub fn validate_response(&self, response: &NetworkResponse, origin: &str) -> CorsResult;

    // Check if preflight is needed
    pub fn is_preflight_needed(&self, request: &NetworkRequest) -> bool;

    // Build a preflight OPTIONS request
    pub fn build_preflight_request(&self, request: &NetworkRequest) -> NetworkRequest;
}
```

## Usage Examples

### Basic CORS Validation

```rust
use cors_validator::{CorsValidator, CorsConfig};

// Create validator that enforces same-origin policy
let config = CorsConfig {
    enforce_same_origin: true,
    allow_credentials: false,
};
let validator = CorsValidator::new(config);

// Validate a request
let result = validator.validate_request(&request, "https://example.com");
if !result.allowed {
    println!("Request blocked: {}", result.reason.unwrap());
}
```

### Allow Cross-Origin with Credentials

```rust
// Create validator that allows cross-origin requests with credentials
let config = CorsConfig {
    enforce_same_origin: false,
    allow_credentials: true,
};
let validator = CorsValidator::new(config);

// Validate response
let result = validator.validate_response(&response, "https://trusted.com");
// result.headers_to_add will contain:
// - Access-Control-Allow-Origin: https://trusted.com
// - Access-Control-Allow-Credentials: true
```

### Preflight Handling

```rust
// Check if preflight needed
if validator.is_preflight_needed(&request) {
    // Build and send preflight request
    let preflight = validator.build_preflight_request(&request);
    // Send preflight and check response before sending actual request
}
```

## Architecture

### Module Structure

```
src/
├── lib.rs           # Public API and CorsResult
├── validator.rs     # Main CorsValidator implementation
├── preflight.rs     # Preflight request detection and building
├── headers.rs       # Access-Control-* header generation
└── policy.rs        # CorsConfig and policy definitions
```

### CORS Decision Flow

```
Request → validate_request() → Check same-origin policy
                             → Check request mode
                             → Build headers
                             → Return CorsResult

Response → validate_response() → Check wildcard + credentials
                                → Build CORS headers
                                → Return CorsResult
```

## Development

### Running Tests

```bash
# Run all tests
cargo test

# Run unit tests only
cargo test --lib

# Run integration tests only
cargo test --test integration

# Run with output
cargo test -- --nocapture
```

### Test Coverage

The component includes comprehensive tests:
- **Unit Tests**: 25+ tests covering all core functionality
  - Same-origin detection
  - Cross-origin handling
  - Preflight detection
  - Header generation
  - Credential modes
  - Wildcard origins

- **Integration Tests**: End-to-end CORS workflows
  - Complete preflight workflow
  - Same-origin workflow
  - Credentials workflow
  - Blocked cross-origin scenarios

### Code Quality

- **Test Coverage**: >80% (target: 95%)
- **Linting**: Passes `cargo clippy`
- **Formatting**: Follows `rustfmt` standards
- **Documentation**: All public APIs documented

## Dependencies

- `network-types`: Core network types (NetworkRequest, NetworkResponse, etc.)
- `network-errors`: Error handling
- `http`: HTTP types (HeaderMap, HeaderValue)
- `url`: URL parsing

## CORS Specification Compliance

This implementation follows the CORS specification (Fetch Standard):
- Simple requests (GET, HEAD, POST with simple content types)
- Preflight requests for non-simple methods
- Credential modes (omit, same-origin, include)
- Wildcard origin restrictions with credentials
- Access-Control-* headers

## Security Considerations

### Credential Security

When `allow_credentials` is enabled:
- Wildcard (*) origins are **blocked**
- Specific origins must be whitelisted
- Credentials include cookies, authorization headers, TLS client certificates

### Same-Origin Enforcement

When `enforce_same_origin` is enabled:
- Cross-origin requests are completely blocked
- Provides strongest security but least flexibility
- Use for sensitive operations

## Version

Current version: **0.1.0**

## License

MIT OR Apache-2.0
