use csp_processor::{CspDirective, CspProcessor};

#[test]
fn test_check_self_source() {
    // Given: A CSP processor with 'self' in script-src
    let header = "script-src 'self'";
    let processor = CspProcessor::new(header).unwrap();

    // When: Checking if current origin is allowed
    let result = processor.check_source(CspDirective::ScriptSrc, "https://example.com/script.js");

    // Then: Should be allowed if from same origin
    // (This will need proper origin checking in implementation)
    assert!(result);
}

#[test]
fn test_check_exact_source_match() {
    // Given: A CSP processor with specific domain
    let header = "script-src example.com";
    let processor = CspProcessor::new(header).unwrap();

    // When: Checking if example.com source is allowed
    let result = processor.check_source(CspDirective::ScriptSrc, "https://example.com/script.js");

    // Then: Should be allowed
    assert!(result);
}

#[test]
fn test_check_source_not_allowed() {
    // Given: A CSP processor with specific domain
    let header = "script-src example.com";
    let processor = CspProcessor::new(header).unwrap();

    // When: Checking if different domain is allowed
    let result = processor.check_source(CspDirective::ScriptSrc, "https://evil.com/script.js");

    // Then: Should not be allowed
    assert!(!result);
}

#[test]
fn test_check_wildcard_subdomain() {
    // Given: A CSP processor with wildcard subdomain
    let header = "script-src *.example.com";
    let processor = CspProcessor::new(header).unwrap();

    // When: Checking various subdomains
    let sub1 = processor.check_source(CspDirective::ScriptSrc, "https://api.example.com/script.js");
    let sub2 = processor.check_source(CspDirective::ScriptSrc, "https://cdn.example.com/script.js");
    let base = processor.check_source(CspDirective::ScriptSrc, "https://example.com/script.js");
    let other = processor.check_source(CspDirective::ScriptSrc, "https://evil.com/script.js");

    // Then: Subdomains should be allowed, base domain and others not
    assert!(sub1);
    assert!(sub2);
    assert!(!base); // Wildcard doesn't match base domain
    assert!(!other);
}

#[test]
fn test_check_unsafe_inline() {
    // Given: A CSP processor with unsafe-inline
    let header = "script-src 'unsafe-inline'";
    let processor = CspProcessor::new(header).unwrap();

    // When: Checking if inline is allowed
    let result = processor.is_inline_allowed(CspDirective::ScriptSrc, None);

    // Then: Should be allowed
    assert!(result);
}

#[test]
fn test_check_inline_not_allowed() {
    // Given: A CSP processor without unsafe-inline
    let header = "script-src example.com";
    let processor = CspProcessor::new(header).unwrap();

    // When: Checking if inline is allowed
    let result = processor.is_inline_allowed(CspDirective::ScriptSrc, None);

    // Then: Should not be allowed
    assert!(!result);
}

#[test]
fn test_check_inline_with_nonce() {
    // Given: A CSP processor with nonce
    let header = "script-src 'nonce-abc123'";
    let processor = CspProcessor::new(header).unwrap();

    // When: Checking with matching nonce
    let valid = processor.is_inline_allowed(CspDirective::ScriptSrc, Some("abc123"));

    // And: Checking with wrong nonce
    let invalid = processor.is_inline_allowed(CspDirective::ScriptSrc, Some("wrong"));

    // And: Checking without nonce
    let no_nonce = processor.is_inline_allowed(CspDirective::ScriptSrc, None);

    // Then: Only matching nonce should be allowed
    assert!(valid);
    assert!(!invalid);
    assert!(!no_nonce);
}

#[test]
fn test_fallback_to_default_src() {
    // Given: A CSP processor with default-src but no script-src
    let header = "default-src 'self'";
    let processor = CspProcessor::new(header).unwrap();

    // When: Checking script-src (which isn't defined)
    let result = processor.check_source(CspDirective::ScriptSrc, "https://example.com/script.js");

    // Then: Should fall back to default-src rules
    assert!(result); // Assuming 'self' check passes for same origin
}
