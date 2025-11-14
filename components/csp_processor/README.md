# CSP Processor

Content Security Policy (CSP) header processing component for the Corten Network Stack.

## Overview

This component provides parsing and validation of Content-Security-Policy headers according to the W3C CSP specification. It enables secure content loading by enforcing policies that control which resources can be loaded and executed.

## Features

- Parse CSP headers (Content-Security-Policy and Content-Security-Policy-Report-Only)
- Validate sources against directives
- Support for:
  - `'self'` keyword
  - Wildcard subdomains (`*.example.com`)
  - Nonces (`'nonce-abc123'`)
  - Hashes (`'sha256-...', 'sha384-...', 'sha512-...'`)
  - `'unsafe-inline'` and `'unsafe-eval'`
- Fallback to `default-src` directive
- Inline script/style checking
- Violation reporting

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
csp_processor = { path = "../csp_processor" }
```

## Usage

### Basic Parsing

```rust
use csp_processor::{CspProcessor, CspDirective};

// Parse a CSP header
let header = "default-src 'self'; script-src 'unsafe-inline' example.com";
let processor = CspProcessor::new(header).unwrap();

// Check if a source is allowed
let allowed = processor.check_source(
    CspDirective::ScriptSrc,
    "https://example.com/script.js"
);
assert!(allowed);
```

### Checking Inline Content

```rust
use csp_processor::{CspProcessor, CspDirective};

// CSP with nonce
let header = "script-src 'nonce-abc123'";
let processor = CspProcessor::new(header).unwrap();

// Check with matching nonce
assert!(processor.is_inline_allowed(CspDirective::ScriptSrc, Some("abc123")));

// Check with wrong nonce
assert!(!processor.is_inline_allowed(CspDirective::ScriptSrc, Some("wrong")));
```

### Wildcard Matching

```rust
use csp_processor::{CspProcessor, CspDirective};

// CSP with wildcard subdomain
let header = "script-src *.example.com";
let processor = CspProcessor::new(header).unwrap();

// Subdomains are allowed
assert!(processor.check_source(
    CspDirective::ScriptSrc,
    "https://api.example.com/script.js"
));

// Base domain is not allowed with wildcard
assert!(!processor.check_source(
    CspDirective::ScriptSrc,
    "https://example.com/script.js"
));
```

## API Reference

### `CspProcessor`

Main processor for CSP operations.

#### Methods

- `new(header: &str) -> Result<Self, CspError>` - Create processor from header string
- `parse_header(header: &str) -> Result<CspPolicy, CspError>` - Parse CSP header
- `check_source(directive: CspDirective, source: &str) -> bool` - Check if source is allowed
- `is_inline_allowed(directive: CspDirective, nonce: Option<&str>) -> bool` - Check inline content
- `report_violation(violation: CspViolation)` - Report CSP violation

### `CspDirective`

Enum representing CSP directive types:

- `DefaultSrc` - default-src
- `ScriptSrc` - script-src
- `StyleSrc` - style-src
- `ImgSrc` - img-src
- `ConnectSrc` - connect-src
- `FontSrc` - font-src
- `ObjectSrc` - object-src
- `MediaSrc` - media-src
- `FrameSrc` - frame-src
- `ReportUri` - report-uri

### `CspPolicy`

Represents a parsed CSP policy.

#### Fields

- `directives: HashMap<String, Vec<String>>` - Policy directives
- `report_only: bool` - Whether this is report-only mode

### `CspViolation`

Represents a CSP violation.

#### Fields

- `directive: String` - Violated directive
- `blocked_uri: String` - Blocked URI
- `violated_directive: String` - Specific directive violated
- `source_file: Option<String>` - Source file (if known)

## Development

### Running Tests

```bash
cargo test
```

### Running Tests with Coverage

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

## Architecture

The CSP processor uses a simple parsing approach:

1. Split header by semicolons to get directives
2. Split each directive by whitespace to get directive name and sources
3. Store in HashMap for fast lookup
4. Validate sources using pattern matching for wildcards, nonces, etc.

## Security Considerations

- All CSP headers are validated before parsing
- Empty headers are rejected
- Source matching is strict (wildcards must be explicit)
- Nonce validation is case-sensitive
- Hash validation supports SHA-256, SHA-384, and SHA-512

## Testing

The component includes comprehensive tests:

- Unit tests for parsing
- Unit tests for source validation
- Unit tests for wildcard matching
- Unit tests for nonce validation
- Unit tests for inline content checking
- Integration tests for real-world scenarios

## License

MIT OR Apache-2.0
