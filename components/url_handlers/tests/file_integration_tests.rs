//! Integration tests for file URL reading with real filesystem

use std::path::PathBuf;
use tempfile::TempDir;
use tokio::fs;
use url_handlers::{FileSecurityPolicy, FileUrlHandler};

#[tokio::test]
async fn test_read_file_url_with_real_file() {
    // Create temporary directory and file
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.txt");
    fs::write(&file_path, b"Hello from file").await.unwrap();

    // Create handler with security policy allowing temp directory
    let policy = FileSecurityPolicy {
        allow_directory_traversal: false,
        allowed_paths: vec![PathBuf::from(temp_dir.path())],
    };
    let handler = FileUrlHandler::new(policy);

    // Read file using file: URL
    let url = format!("file://{}", file_path.display());
    let result = handler.read(&url).await;

    assert!(result.is_ok(), "Should successfully read allowed file");
    let data = result.unwrap();
    assert_eq!(data, b"Hello from file");
}

#[tokio::test]
async fn test_read_file_url_access_denied() {
    // Create temporary directory and file
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.txt");
    fs::write(&file_path, b"Secret content").await.unwrap();

    // Create handler with security policy that does NOT allow temp directory
    let policy = FileSecurityPolicy {
        allow_directory_traversal: false,
        allowed_paths: vec![PathBuf::from("/tmp/other")],
    };
    let handler = FileUrlHandler::new(policy);

    // Try to read file
    let url = format!("file://{}", file_path.display());
    let result = handler.read(&url).await;

    assert!(
        result.is_err(),
        "Should deny access to file outside allowed paths"
    );
}

#[tokio::test]
async fn test_read_file_url_with_binary_data() {
    // Create temporary directory and file with binary data
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("binary.dat");
    let binary_data = vec![0u8, 1, 2, 3, 255, 254, 128, 127];
    fs::write(&file_path, &binary_data).await.unwrap();

    // Create handler
    let policy = FileSecurityPolicy {
        allow_directory_traversal: false,
        allowed_paths: vec![PathBuf::from(temp_dir.path())],
    };
    let handler = FileUrlHandler::new(policy);

    // Read file
    let url = format!("file://{}", file_path.display());
    let result = handler.read(&url).await;

    assert!(result.is_ok(), "Should read binary file");
    let data = result.unwrap();
    assert_eq!(data, binary_data, "Binary data should match exactly");
}

#[tokio::test]
async fn test_read_file_url_nonexistent_file() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("nonexistent.txt");

    let policy = FileSecurityPolicy {
        allow_directory_traversal: false,
        allowed_paths: vec![PathBuf::from(temp_dir.path())],
    };
    let handler = FileUrlHandler::new(policy);

    let url = format!("file://{}", file_path.display());
    let result = handler.read(&url).await;

    assert!(result.is_err(), "Should fail for nonexistent file");
}

#[tokio::test]
async fn test_read_file_url_invalid_url_scheme() {
    let policy = FileSecurityPolicy {
        allow_directory_traversal: false,
        allowed_paths: vec![PathBuf::from("/tmp")],
    };
    let handler = FileUrlHandler::new(policy);

    // Try with HTTP URL
    let result = handler.read("http://example.com/file.txt").await;
    assert!(result.is_err(), "Should reject non-file URL");

    // Try with data URL
    let result = handler.read("data:text/plain,test").await;
    assert!(result.is_err(), "Should reject data URL");
}

#[tokio::test]
async fn test_read_file_url_with_localhost() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.txt");
    fs::write(&file_path, b"localhost file").await.unwrap();

    let policy = FileSecurityPolicy {
        allow_directory_traversal: false,
        allowed_paths: vec![PathBuf::from(temp_dir.path())],
    };
    let handler = FileUrlHandler::new(policy);

    // Use file://localhost/ format
    let url = format!("file://localhost{}", file_path.display());
    let result = handler.read(&url).await;

    assert!(result.is_ok(), "Should read file with localhost in URL");
    let data = result.unwrap();
    assert_eq!(data, b"localhost file");
}

#[tokio::test]
async fn test_read_file_url_empty_file() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("empty.txt");
    fs::write(&file_path, b"").await.unwrap();

    let policy = FileSecurityPolicy {
        allow_directory_traversal: false,
        allowed_paths: vec![PathBuf::from(temp_dir.path())],
    };
    let handler = FileUrlHandler::new(policy);

    let url = format!("file://{}", file_path.display());
    let result = handler.read(&url).await;

    assert!(result.is_ok(), "Should read empty file");
    let data = result.unwrap();
    assert_eq!(data, b"", "Empty file should return empty data");
}
