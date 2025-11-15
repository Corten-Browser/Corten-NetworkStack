use csp_processor::{CspDirective, CspProcessor};
use url::Url;

#[test]
fn test_self_keyword_same_origin() {
    // Given: A CSP processor with 'self' in script-src and document origin set
    let header = "script-src 'self'";
    let processor = CspProcessor::new(header)
        .unwrap()
        .with_document_origin(Url::parse("https://example.com").unwrap());

    // When: Checking a source from the same origin
    let result = processor.check_source(CspDirective::ScriptSrc, "https://example.com/script.js");

    // Then: Should be allowed
    assert!(result);
}

#[test]
fn test_self_keyword_different_scheme() {
    // Given: A CSP processor with document origin https://example.com
    let header = "script-src 'self'";
    let processor = CspProcessor::new(header)
        .unwrap()
        .with_document_origin(Url::parse("https://example.com").unwrap());

    // When: Checking a source with different scheme (http vs https)
    let result = processor.check_source(CspDirective::ScriptSrc, "http://example.com/script.js");

    // Then: Should NOT be allowed (different scheme)
    assert!(!result);
}

#[test]
fn test_self_keyword_different_host() {
    // Given: A CSP processor with document origin https://example.com
    let header = "script-src 'self'";
    let processor = CspProcessor::new(header)
        .unwrap()
        .with_document_origin(Url::parse("https://example.com").unwrap());

    // When: Checking a source from a different host
    let result = processor.check_source(CspDirective::ScriptSrc, "https://other.com/script.js");

    // Then: Should NOT be allowed (different host)
    assert!(!result);
}

#[test]
fn test_self_keyword_different_port() {
    // Given: A CSP processor with document origin https://example.com:443
    let header = "script-src 'self'";
    let processor = CspProcessor::new(header)
        .unwrap()
        .with_document_origin(Url::parse("https://example.com:443").unwrap());

    // When: Checking a source with different port
    let result = processor.check_source(CspDirective::ScriptSrc, "https://example.com:8080/script.js");

    // Then: Should NOT be allowed (different port)
    assert!(!result);
}

#[test]
fn test_self_keyword_without_document_origin() {
    // Given: A CSP processor with 'self' but NO document origin set
    let header = "script-src 'self'";
    let processor = CspProcessor::new(header).unwrap();

    // When: Checking any source
    let result = processor.check_source(CspDirective::ScriptSrc, "https://example.com/script.js");

    // Then: Should NOT be allowed (no document origin = reject for security)
    assert!(!result);
}

#[test]
fn test_self_keyword_with_path() {
    // Given: A CSP processor with document origin
    let header = "script-src 'self'";
    let processor = CspProcessor::new(header)
        .unwrap()
        .with_document_origin(Url::parse("https://example.com/page").unwrap());

    // When: Checking a source from same origin but different path
    let result = processor.check_source(CspDirective::ScriptSrc, "https://example.com/other/script.js");

    // Then: Should be allowed (path doesn't matter for origin check)
    assert!(result);
}

#[test]
fn test_self_keyword_default_ports() {
    // Given: A CSP processor with https document (default port 443)
    let header = "script-src 'self'";
    let processor = CspProcessor::new(header)
        .unwrap()
        .with_document_origin(Url::parse("https://example.com").unwrap());

    // When: Checking a source with explicit default port
    let result = processor.check_source(CspDirective::ScriptSrc, "https://example.com:443/script.js");

    // Then: Should be allowed (443 is default for https)
    assert!(result);
}

#[test]
fn test_self_keyword_subdomain_not_allowed() {
    // Given: A CSP processor with document origin https://example.com
    let header = "script-src 'self'";
    let processor = CspProcessor::new(header)
        .unwrap()
        .with_document_origin(Url::parse("https://example.com").unwrap());

    // When: Checking a subdomain
    let result = processor.check_source(CspDirective::ScriptSrc, "https://sub.example.com/script.js");

    // Then: Should NOT be allowed (subdomain is different host)
    assert!(!result);
}

#[test]
fn test_self_keyword_invalid_url() {
    // Given: A CSP processor with document origin
    let header = "script-src 'self'";
    let processor = CspProcessor::new(header)
        .unwrap()
        .with_document_origin(Url::parse("https://example.com").unwrap());

    // When: Checking an invalid URL
    let result = processor.check_source(CspDirective::ScriptSrc, "not-a-valid-url");

    // Then: Should NOT be allowed
    assert!(!result);
}

#[test]
fn test_check_self_source() {
    // Given: A CSP processor with 'self' in script-src
    let header = "script-src 'self'";
    let processor = CspProcessor::new(header)
        .unwrap()
        .with_document_origin(Url::parse("https://example.com").unwrap());

    // When: Checking if current origin is allowed
    let result = processor.check_source(CspDirective::ScriptSrc, "https://example.com/script.js");

    // Then: Should be allowed if from same origin
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
    let processor = CspProcessor::new(header)
        .unwrap()
        .with_document_origin(Url::parse("https://example.com").unwrap());

    // When: Checking script-src (which isn't defined)
    let result = processor.check_source(CspDirective::ScriptSrc, "https://example.com/script.js");

    // Then: Should fall back to default-src rules
    assert!(result); // 'self' check passes for same origin
}
