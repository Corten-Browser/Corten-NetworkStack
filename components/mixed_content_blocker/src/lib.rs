//! mixed_content_blocker component
//!
//! Mixed content detection and blocking for secure HTTPS browsing.
//!
//! This component implements the W3C Mixed Content specification, which prevents
//! HTTPS pages from loading insecure HTTP resources that could compromise security.
//!
//! # Features
//!
//! - Detects mixed content (HTTP resources in HTTPS pages)
//! - Distinguishes between active and passive mixed content
//! - Supports Upgrade-Insecure-Requests header
//! - Configurable blocking policies
//!
//! # Examples
//!
//! ```
//! use mixed_content_blocker::{MixedContentBlocker, MixedContentPolicy, ContentType};
//! use url::Url;
//!
//! let policy = MixedContentPolicy {
//!     block_all_mixed_content: true,
//!     upgrade_insecure_requests: false,
//! };
//!
//! let blocker = MixedContentBlocker::new(policy);
//!
//! let page_url = Url::parse("https://example.com").unwrap();
//! let resource_url = Url::parse("http://example.com/script.js").unwrap();
//!
//! let result = blocker.check_request(&page_url, &resource_url, ContentType::Active);
//! assert!(result.blocked);
//! ```

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

use network_errors::NetworkError;
use url::Url;

/// Mixed content policy configuration
///
/// Controls how the browser handles mixed content (HTTP resources in HTTPS pages).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MixedContentPolicy {
    /// Block all mixed content (both active and passive)
    ///
    /// When `true`, both active and passive mixed content are blocked.
    /// When `false`, only active mixed content is blocked (passive content generates warnings).
    pub block_all_mixed_content: bool,

    /// Enable Upgrade-Insecure-Requests
    ///
    /// When `true`, HTTP resources are automatically upgraded to HTTPS instead of being blocked.
    /// This implements the W3C Upgrade Insecure Requests specification.
    pub upgrade_insecure_requests: bool,
}

/// Type of content being loaded
///
/// Mixed content is classified into two categories based on security impact:
/// - **Active content** can execute code or modify the page (high risk)
/// - **Passive content** is display-only (lower risk)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentType {
    /// Active content (scripts, stylesheets, objects, iframes)
    ///
    /// Active content can execute code or modify the DOM, making it high-risk.
    /// Examples: JavaScript files, CSS files, plugins, iframes.
    Active,

    /// Passive content (images, audio, video)
    ///
    /// Passive content is display-only and cannot execute code, making it lower risk.
    /// Examples: images, audio files, video files.
    Passive,
}

/// Result of a mixed content check
///
/// Contains information about whether content was blocked and why.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MixedContentResult {
    /// Whether the request was blocked
    pub blocked: bool,

    /// Reason for blocking or warning (if applicable)
    pub reason: Option<String>,

    /// Upgraded URL (if upgrade-insecure-requests is enabled)
    pub upgraded_url: Option<Url>,
}

/// Mixed content blocker
///
/// Detects and blocks HTTP resources loaded from HTTPS pages according to policy.
#[derive(Debug, Clone)]
pub struct MixedContentBlocker {
    policy: MixedContentPolicy,
}

impl MixedContentBlocker {
    /// Create a new mixed content blocker with the given policy
    ///
    /// # Arguments
    ///
    /// * `policy` - The mixed content policy to enforce
    ///
    /// # Examples
    ///
    /// ```
    /// use mixed_content_blocker::{MixedContentBlocker, MixedContentPolicy};
    ///
    /// let policy = MixedContentPolicy {
    ///     block_all_mixed_content: true,
    ///     upgrade_insecure_requests: false,
    /// };
    ///
    /// let blocker = MixedContentBlocker::new(policy);
    /// ```
    pub fn new(policy: MixedContentPolicy) -> Self {
        Self { policy }
    }

    /// Check if a request should be blocked or upgraded
    ///
    /// This is the main entry point for mixed content checking.
    ///
    /// # Arguments
    ///
    /// * `page_url` - URL of the page making the request
    /// * `resource_url` - URL of the resource being requested
    /// * `content_type` - Type of content (Active or Passive)
    ///
    /// # Returns
    ///
    /// A `MixedContentResult` indicating whether the request is blocked,
    /// any warning/error reason, and the upgraded URL if applicable.
    ///
    /// # Examples
    ///
    /// ```
    /// use mixed_content_blocker::{MixedContentBlocker, MixedContentPolicy, ContentType};
    /// use url::Url;
    ///
    /// let policy = MixedContentPolicy {
    ///     block_all_mixed_content: true,
    ///     upgrade_insecure_requests: false,
    /// };
    /// let blocker = MixedContentBlocker::new(policy);
    ///
    /// let page = Url::parse("https://example.com").unwrap();
    /// let resource = Url::parse("http://example.com/script.js").unwrap();
    ///
    /// let result = blocker.check_request(&page, &resource, ContentType::Active);
    /// assert!(result.blocked);
    /// ```
    pub fn check_request(
        &self,
        page_url: &Url,
        resource_url: &Url,
        content_type: ContentType,
    ) -> MixedContentResult {
        // Check if the page is HTTPS
        let page_is_https = page_url.scheme() == "https";

        // If page is not HTTPS, no mixed content concerns
        if !page_is_https {
            return MixedContentResult {
                blocked: false,
                reason: None,
                upgraded_url: None,
            };
        }

        // Check if resource is HTTP (potential mixed content)
        let resource_is_http = resource_url.scheme() == "http";

        // If resource is not HTTP, it's secure
        if !resource_is_http {
            return MixedContentResult {
                blocked: false,
                reason: None,
                upgraded_url: None,
            };
        }

        // We have mixed content: HTTPS page loading HTTP resource

        // Check if we should upgrade instead of blocking
        if self.policy.upgrade_insecure_requests {
            if let Ok(upgraded) = self.upgrade_to_https(resource_url) {
                return MixedContentResult {
                    blocked: false,
                    reason: Some("HTTP resource upgraded to HTTPS".to_string()),
                    upgraded_url: Some(upgraded),
                };
            }
        }

        // Determine if we should block based on content type and policy
        let should_block = match content_type {
            ContentType::Active => {
                // Always block active mixed content
                true
            }
            ContentType::Passive => {
                // Block passive content only if policy is strict
                self.policy.block_all_mixed_content
            }
        };

        let reason = if should_block {
            Some(format!(
                "Mixed content: HTTPS page attempted to load HTTP {} content",
                match content_type {
                    ContentType::Active => "active",
                    ContentType::Passive => "passive",
                }
            ))
        } else {
            Some(format!(
                "Mixed content warning: HTTPS page loading HTTP {} content",
                match content_type {
                    ContentType::Active => "active",
                    ContentType::Passive => "passive",
                }
            ))
        };

        MixedContentResult {
            blocked: should_block,
            reason,
            upgraded_url: None,
        }
    }

    /// Check if a URL should be upgraded to HTTPS
    ///
    /// Returns `true` if the URL is HTTP and upgrade-insecure-requests is enabled.
    ///
    /// # Arguments
    ///
    /// * `url` - The URL to check
    ///
    /// # Examples
    ///
    /// ```
    /// use mixed_content_blocker::{MixedContentBlocker, MixedContentPolicy};
    /// use url::Url;
    ///
    /// let policy = MixedContentPolicy {
    ///     block_all_mixed_content: false,
    ///     upgrade_insecure_requests: true,
    /// };
    /// let blocker = MixedContentBlocker::new(policy);
    ///
    /// let http_url = Url::parse("http://example.com").unwrap();
    /// assert!(blocker.should_upgrade(&http_url));
    ///
    /// let https_url = Url::parse("https://example.com").unwrap();
    /// assert!(!blocker.should_upgrade(&https_url));
    /// ```
    pub fn should_upgrade(&self, url: &Url) -> bool {
        self.policy.upgrade_insecure_requests && url.scheme() == "http"
    }

    /// Upgrade an HTTP URL to HTTPS
    ///
    /// Converts an HTTP URL to HTTPS, preserving all other components
    /// (host, port, path, query, fragment).
    ///
    /// # Arguments
    ///
    /// * `url` - The HTTP URL to upgrade
    ///
    /// # Returns
    ///
    /// - `Ok(Url)` - The upgraded HTTPS URL
    /// - `Err(NetworkError)` - If the URL cannot be upgraded (e.g., not HTTP)
    ///
    /// # Examples
    ///
    /// ```
    /// use mixed_content_blocker::{MixedContentBlocker, MixedContentPolicy};
    /// use url::Url;
    ///
    /// let policy = MixedContentPolicy {
    ///     block_all_mixed_content: false,
    ///     upgrade_insecure_requests: true,
    /// };
    /// let blocker = MixedContentBlocker::new(policy);
    ///
    /// let http_url = Url::parse("http://example.com/path").unwrap();
    /// let https_url = blocker.upgrade_to_https(&http_url).unwrap();
    ///
    /// assert_eq!(https_url.scheme(), "https");
    /// assert_eq!(https_url.path(), "/path");
    /// ```
    pub fn upgrade_to_https(&self, url: &Url) -> Result<Url, NetworkError> {
        // If already HTTPS, return as-is
        if url.scheme() == "https" {
            return Ok(url.clone());
        }

        // Only upgrade HTTP URLs
        if url.scheme() != "http" {
            return Err(NetworkError::InvalidUrl(format!(
                "Cannot upgrade {} URL to HTTPS",
                url.scheme()
            )));
        }

        // Build new HTTPS URL preserving all components
        let mut upgraded = url.clone();
        upgraded
            .set_scheme("https")
            .map_err(|_| NetworkError::InvalidUrl("Failed to set HTTPS scheme".to_string()))?;

        Ok(upgraded)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_policy_creation() {
        let policy = MixedContentPolicy {
            block_all_mixed_content: true,
            upgrade_insecure_requests: false,
        };
        assert!(policy.block_all_mixed_content);
        assert!(!policy.upgrade_insecure_requests);
    }

    #[test]
    fn test_blocker_creation() {
        let policy = MixedContentPolicy {
            block_all_mixed_content: true,
            upgrade_insecure_requests: false,
        };
        let blocker = MixedContentBlocker::new(policy);
        assert_eq!(blocker.policy, policy);
    }

    #[test]
    fn test_content_type_variants() {
        let active = ContentType::Active;
        let passive = ContentType::Passive;
        assert_ne!(active, passive);
    }
}
