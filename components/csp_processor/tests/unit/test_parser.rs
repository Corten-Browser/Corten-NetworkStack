use csp_processor::{CspPolicy, CspProcessor};

#[test]
fn test_parse_simple_csp_header() {
    // Given: A simple CSP header with default-src directive
    let header = "default-src 'self'";

    // When: Parsing the header
    let result = CspProcessor::parse_header(header);

    // Then: Should successfully parse
    assert!(result.is_ok());
    let policy = result.unwrap();
    assert!(!policy.report_only);

    // And: Should have default-src directive
    assert!(policy.directives.contains_key("default-src"));
    let sources = &policy.directives["default-src"];
    assert_eq!(sources.len(), 1);
    assert_eq!(sources[0], "'self'");
}

#[test]
fn test_parse_multiple_directives() {
    // Given: CSP header with multiple directives
    let header = "default-src 'self'; script-src 'unsafe-inline' example.com";

    // When: Parsing the header
    let result = CspProcessor::parse_header(header);

    // Then: Should successfully parse
    assert!(result.is_ok());
    let policy = result.unwrap();

    // And: Should have both directives
    assert!(policy.directives.contains_key("default-src"));
    assert!(policy.directives.contains_key("script-src"));

    // And: default-src should have 'self'
    let default_sources = &policy.directives["default-src"];
    assert_eq!(default_sources, &vec!["'self'"]);

    // And: script-src should have unsafe-inline and example.com
    let script_sources = &policy.directives["script-src"];
    assert_eq!(script_sources.len(), 2);
    assert!(script_sources.contains(&"'unsafe-inline'".to_string()));
    assert!(script_sources.contains(&"example.com".to_string()));
}

#[test]
fn test_parse_empty_header() {
    // Given: An empty CSP header
    let header = "";

    // When: Parsing the header
    let result = CspProcessor::parse_header(header);

    // Then: Should return error
    assert!(result.is_err());
}

#[test]
fn test_parse_wildcard_source() {
    // Given: CSP header with wildcard subdomain
    let header = "script-src *.example.com";

    // When: Parsing the header
    let result = CspProcessor::parse_header(header);

    // Then: Should successfully parse
    assert!(result.is_ok());
    let policy = result.unwrap();

    // And: Should contain wildcard source
    let sources = &policy.directives["script-src"];
    assert_eq!(sources, &vec!["*.example.com"]);
}

#[test]
fn test_parse_nonce_source() {
    // Given: CSP header with nonce
    let header = "script-src 'nonce-abc123'";

    // When: Parsing the header
    let result = CspProcessor::parse_header(header);

    // Then: Should successfully parse
    assert!(result.is_ok());
    let policy = result.unwrap();

    // And: Should contain nonce source
    let sources = &policy.directives["script-src"];
    assert_eq!(sources, &vec!["'nonce-abc123'"]);
}

#[test]
fn test_parse_hash_source() {
    // Given: CSP header with SHA-256 hash
    let header = "script-src 'sha256-abc123=='";

    // When: Parsing the header
    let result = CspProcessor::parse_header(header);

    // Then: Should successfully parse
    assert!(result.is_ok());
    let policy = result.unwrap();

    // And: Should contain hash source
    let sources = &policy.directives["script-src"];
    assert_eq!(sources, &vec!["'sha256-abc123=='"]);
}
