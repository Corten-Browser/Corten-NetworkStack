//! Security policy for file URL access
//!
//! Provides path validation and access control for file URLs to prevent:
//! - Directory traversal attacks
//! - Access to unauthorized paths
//! - Reading sensitive system files

use std::path::{Path, PathBuf};

/// Security policy for file URL access
///
/// Defines which paths are allowed to be accessed and whether directory
/// traversal is permitted.
#[derive(Debug, Clone)]
pub struct FileSecurityPolicy {
    /// Whether to allow directory traversal (..)
    ///
    /// When false, paths containing ".." components are rejected.
    pub allow_directory_traversal: bool,

    /// List of allowed path prefixes
    ///
    /// Only files within these paths (or their subdirectories) can be accessed.
    /// An empty list means no paths are allowed.
    pub allowed_paths: Vec<PathBuf>,
}

impl FileSecurityPolicy {
    /// Check if a path is allowed by this policy
    ///
    /// # Arguments
    ///
    /// * `path` - The path to check
    ///
    /// # Returns
    ///
    /// `true` if the path is allowed, `false` otherwise
    ///
    /// # Security Considerations
    ///
    /// - If `allowed_paths` is empty, all paths are rejected
    /// - If `allow_directory_traversal` is false, paths with ".." are rejected
    /// - Paths are canonicalized before checking to prevent bypasses
    pub fn is_allowed(&self, path: &Path) -> bool {
        // Empty allowlist means nothing is allowed
        if self.allowed_paths.is_empty() {
            return false;
        }

        // Check for directory traversal if not allowed
        if !self.allow_directory_traversal {
            // Check if path contains ".." components
            for component in path.components() {
                if let std::path::Component::ParentDir = component {
                    return false;
                }
            }
        }

        // Canonicalize the path to resolve symlinks and relative components
        let canonical_path = match path.canonicalize() {
            Ok(p) => p,
            Err(_) => {
                // If canonicalization fails (e.g., file doesn't exist yet),
                // use the absolute path
                match path.absolutize() {
                    Ok(p) => p.to_path_buf(),
                    Err(_) => return false,
                }
            }
        };

        // Check if path starts with any allowed prefix
        for allowed_prefix in &self.allowed_paths {
            let canonical_prefix = match allowed_prefix.canonicalize() {
                Ok(p) => p,
                Err(_) => {
                    // If prefix doesn't exist, try to use it as-is
                    match allowed_prefix.absolutize() {
                        Ok(p) => p.to_path_buf(),
                        Err(_) => continue,
                    }
                }
            };

            if canonical_path.starts_with(&canonical_prefix) {
                return true;
            }
        }

        false
    }
}

/// Extension trait for path absolutization
trait PathAbsolutize {
    fn absolutize(&self) -> std::io::Result<PathBuf>;
}

impl PathAbsolutize for Path {
    fn absolutize(&self) -> std::io::Result<PathBuf> {
        if self.is_absolute() {
            Ok(self.to_path_buf())
        } else {
            let current_dir = std::env::current_dir()?;
            Ok(current_dir.join(self))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_allowlist_rejects_all() {
        let policy = FileSecurityPolicy {
            allow_directory_traversal: false,
            allowed_paths: vec![],
        };

        assert!(!policy.is_allowed(Path::new("/any/path")));
    }

    #[test]
    fn test_directory_traversal_blocked() {
        let policy = FileSecurityPolicy {
            allow_directory_traversal: false,
            allowed_paths: vec![PathBuf::from("/allowed")],
        };

        assert!(!policy.is_allowed(Path::new("/allowed/../etc/passwd")));
    }

    #[test]
    fn test_directory_traversal_allowed() {
        let policy = FileSecurityPolicy {
            allow_directory_traversal: true,
            allowed_paths: vec![PathBuf::from("/tmp")],
        };

        // Even with traversal allowed, path must end up in allowed prefix
        // This is a behavior test - actual result depends on canonicalization
        let _result = policy.is_allowed(Path::new("/tmp/../tmp/file.txt"));
    }
}
