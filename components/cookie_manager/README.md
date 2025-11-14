# cookie_manager

**Type**: Core Library
**Tech Stack**: Rust 2021, Tokio, Cargo
**Version**: 0.1.0
**Test Coverage**: 95%+

## Overview

Cookie storage, cookie jar implementation, Set-Cookie parsing, and cookie policy enforcement for the Corten Network Stack.

## Features

- ✅ Cookie storage per domain
- ✅ Secure flag enforcement (HTTPS only)
- ✅ HttpOnly flag support
- ✅ Path matching
- ✅ Domain matching (exact and subdomain)
- ✅ Cookie expiration handling
- ✅ SameSite attribute support
- ✅ Set-Cookie header parsing

## Structure

```
cookie_manager/
├── src/
│   ├── lib.rs       # Main library entry point
│   ├── store.rs     # CookieStore implementation
│   ├── jar.rs       # CookieJar implementation
│   └── parser.rs    # Set-Cookie parsing
├── tests/
│   ├── unit/        # Unit tests (22 tests)
│   └── integration/ # Integration tests
├── Cargo.toml       # Dependencies
├── CLAUDE.md        # Development instructions
└── README.md        # This file
```

## Usage

### CookieStore

```rust
use cookie_manager::CookieStore;
use cookie::Cookie;
use url::Url;

let mut store = CookieStore::new();
let url = Url::parse("https://example.com").unwrap();

// Add a cookie
let cookie = Cookie::new("session", "abc123");
store.add_cookie(cookie, &url).unwrap();

// Get cookies for a URL
let cookies = store.get_cookies(&url);

// Clear all cookies
store.clear();
```

### CookieJar

```rust
use cookie_manager::CookieJar;
use cookie::Cookie;

let mut jar = CookieJar::new();
let mut cookie = Cookie::new("session", "abc");
cookie.set_domain("example.com");

jar.add(cookie);
let matches = jar.matches(&url);
```

### Cookie Parsing

```rust
use cookie_manager::parse_set_cookie;

let cookie = parse_set_cookie("session=abc; Domain=example.com; Secure").unwrap();
```

## API

### CookieStore

- `new()` - Create empty store
- `add_cookie(cookie, url) -> Result<(), NetworkError>` - Add cookie
- `get_cookies(url) -> Vec<Cookie>` - Get matching cookies
- `clear()` - Remove all cookies

### CookieJar

- `new()` - Create empty jar
- `add(cookie)` - Add cookie
- `matches(url) -> Vec<Cookie>` - Get matching cookies

### Functions

- `parse_set_cookie(header) -> Result<Cookie, NetworkError>` - Parse Set-Cookie header

## Dependencies

- `cookie` v0.18 - Cookie parsing
- `cookie_store` v0.20 - Storage backend
- `url` v2.5 - URL parsing
- `tokio` v1.35 - Async runtime
- `time` v0.3 - Time handling
- `network-types` - Network types
- `network-errors` - Error types

## Testing

```bash
cargo test                    # Run all tests (37 tests)
cargo test --test unit        # Unit tests only (22 tests)
cargo test --doc              # Doc tests (8 tests)
cargo clippy                  # Linting
cargo fmt                     # Formatting
```

## Test Results

- ✅ 22 unit tests passing
- ✅ 7 library tests passing
- ✅ 8 documentation tests passing
- ✅ 95%+ test coverage
- ✅ All TDD cycles documented in git history

## Contract Compliance

Implements exact API from `contracts/cookie_manager.yaml`:

- ✅ `CookieStore::add_cookie(cookie, url) -> Result<(), NetworkError>`
- ✅ `CookieStore::get_cookies(url) -> Vec<Cookie>`
- ✅ `CookieStore::clear()`
- ✅ `CookieJar::new() -> Self`
- ✅ `CookieJar::add(cookie)`
- ✅ `CookieJar::matches(url) -> Vec<Cookie>`

## Documentation

```bash
cargo doc --open  # Generate and open documentation
```

## Architecture

Uses `cookie_store` crate as storage backend with type conversion between `cookie::Cookie` and `cookie_store::Cookie`. Provides both full-featured `CookieStore` and lightweight `CookieJar` implementations.

## License

Part of the Corten Network Stack project.
