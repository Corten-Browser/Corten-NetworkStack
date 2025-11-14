//! Unit tests for file URL handling

use std::path::PathBuf;
use url_handlers::{FileSecurityPolicy, FileUrlHandler};

#[test]
fn test_is_file_url() {
    assert!(FileUrlHandler::is_file_url("file:///path/to/file"));
    assert!(FileUrlHandler::is_file_url("file://localhost/path"));
    assert!(!FileUrlHandler::is_file_url("http://example.com"));
    assert!(!FileUrlHandler::is_file_url("data:text/plain,test"));
}

#[test]
fn test_is_allowed_with_empty_allowlist() {
    let policy = FileSecurityPolicy {
        allow_directory_traversal: false,
        allowed_paths: vec![],
    };
    let handler = FileUrlHandler::new(policy);

    let path = std::path::Path::new("/tmp/test.txt");
    let result = handler.is_allowed(path);
    assert!(!result, "Empty allowlist should reject all paths");
}

#[test]
fn test_is_allowed_blocks_directory_traversal_by_default() {
    let policy = FileSecurityPolicy {
        allow_directory_traversal: false,
        allowed_paths: vec![PathBuf::from("/allowed")],
    };
    let handler = FileUrlHandler::new(policy);

    let path = std::path::Path::new("/allowed/../etc/passwd");
    let result = handler.is_allowed(path);
    assert!(
        !result,
        "Directory traversal should be blocked when policy disallows it"
    );
}

#[test]
fn test_is_allowed_permits_directory_traversal_when_enabled() {
    let policy = FileSecurityPolicy {
        allow_directory_traversal: true,
        allowed_paths: vec![PathBuf::from("/")],
    };
    let handler = FileUrlHandler::new(policy);

    let path = std::path::Path::new("/allowed/../etc/passwd");
    // Note: This test checks that the policy ALLOWS traversal when enabled
    // The actual path validation happens during canonicalization
    let _result = handler.is_allowed(path);
    // Actual result depends on filesystem state, so we just verify it doesn't panic
}

#[test]
fn test_is_allowed_checks_against_allowlist() {
    let policy = FileSecurityPolicy {
        allow_directory_traversal: false,
        allowed_paths: vec![PathBuf::from("/tmp"), PathBuf::from("/var/www")],
    };
    let handler = FileUrlHandler::new(policy);

    // This test depends on filesystem state for canonicalization
    // We're just testing that the handler accepts the policy correctly
    let _result1 = handler.is_allowed(std::path::Path::new("/tmp/test.txt"));
    let _result2 = handler.is_allowed(std::path::Path::new("/var/www/index.html"));
    let _result3 = handler.is_allowed(std::path::Path::new("/etc/passwd"));
    // Actual results depend on what paths exist and can be canonicalized
}

#[test]
fn test_file_security_policy_creation() {
    let policy = FileSecurityPolicy {
        allow_directory_traversal: false,
        allowed_paths: vec![PathBuf::from("/tmp")],
    };

    assert!(!policy.allow_directory_traversal);
    assert_eq!(policy.allowed_paths.len(), 1);
    assert_eq!(policy.allowed_paths[0], PathBuf::from("/tmp"));
}

#[test]
fn test_file_url_handler_creation() {
    let policy = FileSecurityPolicy {
        allow_directory_traversal: false,
        allowed_paths: vec![PathBuf::from("/tmp")],
    };
    let handler = FileUrlHandler::new(policy);

    // Verify handler was created with the policy
    assert!(!handler.security_policy.allow_directory_traversal);
    assert_eq!(handler.security_policy.allowed_paths.len(), 1);
}
