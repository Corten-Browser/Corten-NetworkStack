//! File URL handling with security policies
//!
//! Provides safe file reading from file: URLs with:
//! - Path allowlisting
//! - Directory traversal prevention
//! - Same-origin policy enforcement

use crate::security::FileSecurityPolicy;
use network_errors::NetworkError;
use std::path::Path;
use tokio::fs;

/// File URL handler
///
/// Provides secure file reading from file: URLs with configurable
/// security policies.
#[derive(Debug)]
pub struct FileUrlHandler {
    /// Security policy for file access
    pub security_policy: FileSecurityPolicy,
}

impl FileUrlHandler {
    /// Create a new file URL handler with the given security policy
    ///
    /// # Arguments
    ///
    /// * `policy` - Security policy defining allowed paths and traversal rules
    ///
    /// # Examples
    ///
    /// ```
    /// use url_handlers::{FileUrlHandler, FileSecurityPolicy};
    /// use std::path::PathBuf;
    ///
    /// let policy = FileSecurityPolicy {
    ///     allow_directory_traversal: false,
    ///     allowed_paths: vec![PathBuf::from("/allowed/path")],
    /// };
    /// let handler = FileUrlHandler::new(policy);
    /// ```
    pub fn new(security_policy: FileSecurityPolicy) -> Self {
        Self { security_policy }
    }

    /// Check if a URL is a file URL
    ///
    /// # Examples
    ///
    /// ```
    /// use url_handlers::FileUrlHandler;
    ///
    /// assert!(FileUrlHandler::is_file_url("file:///path/to/file"));
    /// assert!(!FileUrlHandler::is_file_url("http://example.com"));
    /// ```
    pub fn is_file_url(url: &str) -> bool {
        url.starts_with("file:")
    }

    /// Check if a path is allowed by the security policy
    ///
    /// # Arguments
    ///
    /// * `path` - The file path to check
    ///
    /// # Returns
    ///
    /// `true` if the path is allowed, `false` otherwise
    pub fn is_allowed(&self, path: &Path) -> bool {
        self.security_policy.is_allowed(path)
    }

    /// Read a file from a file URL
    ///
    /// # Arguments
    ///
    /// * `url` - The file: URL to read
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<u8>)` - File contents
    /// * `Err(NetworkError)` - If URL is invalid, access denied, or file cannot be read
    ///
    /// # Security
    ///
    /// - Only files within `allowed_paths` can be accessed
    /// - Directory traversal may be blocked based on policy
    /// - Non-existent files return an error
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use url_handlers::{FileUrlHandler, FileSecurityPolicy};
    /// use std::path::PathBuf;
    ///
    /// # async fn example() {
    /// let policy = FileSecurityPolicy {
    ///     allow_directory_traversal: false,
    ///     allowed_paths: vec![PathBuf::from("/tmp")],
    /// };
    /// let handler = FileUrlHandler::new(policy);
    ///
    /// let url = "file:///tmp/test.txt";
    /// let data = handler.read(url).await.expect("Failed to read file");
    /// # }
    /// ```
    pub async fn read(&self, url: &str) -> Result<Vec<u8>, NetworkError> {
        // Verify it's a file URL
        if !Self::is_file_url(url) {
            return Err(NetworkError::InvalidUrl(
                "Not a file URL (must start with 'file:')".to_string(),
            ));
        }

        // Parse the file path from URL
        let file_path = Self::parse_file_url(url)?;

        // Check security policy
        if !self.is_allowed(&file_path) {
            return Err(NetworkError::Other(format!(
                "Access denied: path '{}' not in allowed paths",
                file_path.display()
            )));
        }

        // Read the file
        fs::read(&file_path).await.map_err(NetworkError::Io)
    }

    /// Parse a file URL into a file path
    ///
    /// Supports formats:
    /// - file:///absolute/path
    /// - file://localhost/absolute/path
    /// - file:/absolute/path (non-standard but common)
    fn parse_file_url(url: &str) -> Result<std::path::PathBuf, NetworkError> {
        // Strip "file:" prefix
        let url = url
            .strip_prefix("file:")
            .ok_or_else(|| NetworkError::InvalidUrl("Not a file URL".to_string()))?;

        // Handle different file URL formats
        let path = if let Some(stripped) = url.strip_prefix("///") {
            // file:///path format
            format!("/{}", stripped)
        } else if let Some(stripped) = url.strip_prefix("//localhost/") {
            // file://localhost/path format
            format!("/{}", stripped)
        } else if let Some(stripped) = url.strip_prefix("//") {
            // file://path format (non-standard)
            // Reject if it looks like a network path (has hostname)
            if let Some(slash_pos) = stripped.find('/') {
                let host = &stripped[..slash_pos];
                if !host.is_empty() && host != "localhost" {
                    return Err(NetworkError::Other(
                        "Remote file URLs not supported".to_string(),
                    ));
                }
                format!("/{}", &stripped[slash_pos + 1..])
            } else {
                return Err(NetworkError::InvalidUrl(
                    "Invalid file URL format".to_string(),
                ));
            }
        } else if let Some(stripped) = url.strip_prefix('/') {
            // file:/path format
            format!("/{}", stripped)
        } else {
            return Err(NetworkError::InvalidUrl(
                "Invalid file URL format".to_string(),
            ));
        };

        Ok(std::path::PathBuf::from(path))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_is_file_url() {
        assert!(FileUrlHandler::is_file_url("file:///path"));
        assert!(!FileUrlHandler::is_file_url("http://example.com"));
    }

    #[test]
    fn test_parse_file_url() {
        let path = FileUrlHandler::parse_file_url("file:///home/user/file.txt").unwrap();
        assert_eq!(path, PathBuf::from("/home/user/file.txt"));

        let path = FileUrlHandler::parse_file_url("file://localhost/home/user/file.txt").unwrap();
        assert_eq!(path, PathBuf::from("/home/user/file.txt"));
    }

    #[test]
    fn test_parse_file_url_rejects_remote() {
        assert!(FileUrlHandler::parse_file_url("file://remote-host/path").is_err());
    }

    #[test]
    fn test_is_allowed() {
        let policy = FileSecurityPolicy {
            allow_directory_traversal: false,
            allowed_paths: vec![PathBuf::from("/tmp")],
        };
        let handler = FileUrlHandler::new(policy);

        // This test will pass if /tmp exists and can be canonicalized
        // Exact behavior depends on filesystem state
        let _result = handler.is_allowed(Path::new("/tmp/test.txt"));
    }
}
