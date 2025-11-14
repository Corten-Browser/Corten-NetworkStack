use csp_processor::{CspDirective, CspProcessor, CspViolation};

#[test]
fn test_complete_csp_workflow() {
    // Given: A comprehensive CSP policy
    let header = "default-src 'self'; script-src 'nonce-abc123' *.cdn.com https://trusted.com; style-src 'unsafe-inline'";

    // When: Creating a processor
    let processor = CspProcessor::new(header).unwrap();

    // Then: Should correctly validate various sources

    // Test script sources
    assert!(processor.check_source(CspDirective::ScriptSrc, "https://api.cdn.com/script.js")); // Wildcard match
    assert!(processor.check_source(CspDirective::ScriptSrc, "https://trusted.com/app.js")); // Exact match
    assert!(!processor.check_source(CspDirective::ScriptSrc, "https://evil.com/bad.js")); // Not allowed

    // Test inline with nonce
    assert!(processor.is_inline_allowed(CspDirective::ScriptSrc, Some("abc123"))); // Correct nonce
    assert!(!processor.is_inline_allowed(CspDirective::ScriptSrc, Some("wrong"))); // Wrong nonce
    assert!(!processor.is_inline_allowed(CspDirective::ScriptSrc, None)); // No nonce

    // Test style sources (has unsafe-inline)
    assert!(processor.is_inline_allowed(CspDirective::StyleSrc, None)); // unsafe-inline allows

    // Test fallback to default-src for img-src (not explicitly defined)
    assert!(processor.check_source(CspDirective::ImgSrc, "https://example.com/image.png")); // Falls back to 'self'
}

#[test]
fn test_real_world_strict_policy() {
    // Given: A strict real-world CSP policy
    let header = "default-src 'none'; script-src 'self' https://cdn.example.com; img-src *; connect-src 'self'";

    // When: Creating a processor
    let processor = CspProcessor::new(header).unwrap();

    // Then: Should enforce strict rules

    // Scripts only from self and CDN
    assert!(processor.check_source(CspDirective::ScriptSrc, "https://example.com/app.js")); // self
    assert!(processor.check_source(CspDirective::ScriptSrc, "https://cdn.example.com/lib.js")); // CDN
    assert!(!processor.check_source(CspDirective::ScriptSrc, "https://other.com/bad.js")); // Not allowed

    // Images from anywhere (*)
    assert!(processor.check_source(CspDirective::ImgSrc, "https://anywhere.com/pic.jpg"));

    // Connect only to self
    assert!(processor.check_source(CspDirective::ConnectSrc, "https://example.com/api")); // self

    // Object should fall back to default-src 'none' (deny all)
    assert!(!processor.check_source(CspDirective::ObjectSrc, "https://example.com/plugin.swf"));
}

#[test]
fn test_violation_reporting() {
    // Given: A CSP processor
    let header = "script-src 'self'";
    let processor = CspProcessor::new(header).unwrap();

    // When: Reporting a violation
    let violation = CspViolation {
        directive: "script-src".to_string(),
        blocked_uri: "https://evil.com/malware.js".to_string(),
        violated_directive: "script-src 'self'".to_string(),
        source_file: Some("index.html".to_string()),
    };

    // Then: Should accept and process violation (no panic)
    processor.report_violation(violation);
}

#[test]
fn test_subdomain_wildcard_patterns() {
    // Given: Various wildcard patterns
    let header = "script-src *.example.com *.cdn.net";
    let processor = CspProcessor::new(header).unwrap();

    // Then: Should match subdomains correctly
    assert!(processor.check_source(CspDirective::ScriptSrc, "https://api.example.com/script.js"));
    assert!(processor.check_source(CspDirective::ScriptSrc, "https://www.example.com/app.js"));
    assert!(processor.check_source(CspDirective::ScriptSrc, "https://static.cdn.net/lib.js"));

    // But not base domains or other domains
    assert!(!processor.check_source(CspDirective::ScriptSrc, "https://example.com/script.js")); // Base domain
    assert!(!processor.check_source(CspDirective::ScriptSrc, "https://evil.com/bad.js")); // Different domain
}

#[test]
fn test_multiple_source_types() {
    // Given: CSP with mixed source types
    let header = "script-src 'self' 'unsafe-inline' 'nonce-xyz' https://cdn.com";
    let processor = CspProcessor::new(header).unwrap();

    // Then: All types should work
    assert!(processor.check_source(CspDirective::ScriptSrc, "https://example.com/app.js")); // self
    assert!(processor.check_source(CspDirective::ScriptSrc, "https://cdn.com/lib.js")); // exact domain
    assert!(processor.is_inline_allowed(CspDirective::ScriptSrc, None)); // unsafe-inline
    assert!(processor.is_inline_allowed(CspDirective::ScriptSrc, Some("xyz"))); // nonce
}
