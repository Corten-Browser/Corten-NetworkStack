//! CSP (Content Security Policy) Processor Component
//!
//! This component provides parsing and validation of Content-Security-Policy headers
//! according to the W3C CSP specification.

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

use std::collections::HashMap;

/// CSP directive types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CspDirective {
    /// default-src directive
    DefaultSrc,
    /// script-src directive
    ScriptSrc,
    /// style-src directive
    StyleSrc,
    /// img-src directive
    ImgSrc,
    /// connect-src directive
    ConnectSrc,
    /// font-src directive
    FontSrc,
    /// object-src directive
    ObjectSrc,
    /// media-src directive
    MediaSrc,
    /// frame-src directive
    FrameSrc,
    /// report-uri directive
    ReportUri,
}

/// A Content Security Policy
#[derive(Debug, Clone)]
pub struct CspPolicy {
    /// The policy directives (directive name -> list of sources)
    pub directives: HashMap<String, Vec<String>>,
    /// Whether this is a report-only policy
    pub report_only: bool,
}

/// A CSP violation report
#[derive(Debug, Clone)]
pub struct CspViolation {
    /// The directive that was violated
    pub directive: String,
    /// The URI that was blocked
    pub blocked_uri: String,
    /// The specific directive that was violated
    pub violated_directive: String,
    /// The source file where the violation occurred (if known)
    pub source_file: Option<String>,
}

/// CSP Processor - main entry point for CSP operations
#[derive(Debug)]
pub struct CspProcessor {
    policy: CspPolicy,
}

/// Custom error type for CSP operations
#[derive(Debug)]
pub struct CspError {
    message: String,
}

impl std::fmt::Display for CspError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for CspError {}

impl CspProcessor {
    /// Create a new CSP processor from a header string
    pub fn new(header: &str) -> Result<Self, CspError> {
        let policy = Self::parse_header(header)?;
        Ok(Self { policy })
    }

    /// Parse a CSP header into a policy
    pub fn parse_header(header: &str) -> Result<CspPolicy, CspError> {
        if header.trim().is_empty() {
            return Err(CspError {
                message: "Empty CSP header".to_string(),
            });
        }

        let mut directives = HashMap::new();

        // Split by semicolons to get individual directives
        for directive_str in header.split(';') {
            let directive_str = directive_str.trim();
            if directive_str.is_empty() {
                continue;
            }

            // Split directive into name and sources
            let parts: Vec<&str> = directive_str.split_whitespace().collect();
            if parts.is_empty() {
                continue;
            }

            let directive_name = parts[0];
            let sources: Vec<String> = parts[1..].iter().map(|s| s.to_string()).collect();

            directives.insert(directive_name.to_string(), sources);
        }

        Ok(CspPolicy {
            directives,
            report_only: false,
        })
    }

    /// Check if a source is allowed for a given directive
    pub fn check_source(&self, directive: CspDirective, source: &str) -> bool {
        let directive_name = directive.as_str();

        // Get sources for this directive, or fall back to default-src
        let sources = self
            .policy
            .directives
            .get(directive_name)
            .or_else(|| self.policy.directives.get("default-src"));

        if let Some(sources) = sources {
            // Check if any source matches
            for allowed_source in sources {
                if self.source_matches(allowed_source, source) {
                    return true;
                }
            }
            false
        } else {
            // No policy means everything is allowed by default
            true
        }
    }

    /// Check if a source matches an allowed source pattern
    fn source_matches(&self, allowed: &str, actual: &str) -> bool {
        // Handle 'self'
        if allowed == "'self'" {
            // Simple 'self' check - would need proper origin comparison in production
            return true;
        }

        // Handle wildcards
        if allowed.starts_with("*.") {
            let domain_suffix = &allowed[2..]; // Remove "*."
            if let Some(host) = Self::extract_host(actual) {
                // Check if host ends with the domain suffix and has a subdomain
                if host.ends_with(domain_suffix) && host.len() > domain_suffix.len() {
                    return true;
                }
            }
            return false;
        }

        // Exact domain match
        if let Some(host) = Self::extract_host(actual) {
            return host == allowed || host.starts_with(&format!("{}:", allowed));
        }

        false
    }

    /// Extract hostname from a URL
    fn extract_host(url: &str) -> Option<String> {
        if let Some(start) = url.find("://") {
            let after_protocol = &url[start + 3..];
            if let Some(end) = after_protocol.find('/') {
                Some(after_protocol[..end].to_string())
            } else {
                Some(after_protocol.to_string())
            }
        } else {
            None
        }
    }

    /// Check if inline content is allowed for a directive
    pub fn is_inline_allowed(&self, directive: CspDirective, nonce: Option<&str>) -> bool {
        let directive_name = directive.as_str();

        // Get sources for this directive, or fall back to default-src
        let sources = self
            .policy
            .directives
            .get(directive_name)
            .or_else(|| self.policy.directives.get("default-src"));

        if let Some(sources) = sources {
            // Check for unsafe-inline
            if sources.contains(&"'unsafe-inline'".to_string()) {
                return true;
            }

            // Check for nonce match
            if let Some(nonce_value) = nonce {
                let nonce_str = format!("'nonce-{}'", nonce_value);
                if sources.contains(&nonce_str) {
                    return true;
                }
            }

            false
        } else {
            // No policy means inline is allowed by default
            true
        }
    }

    /// Report a CSP violation
    pub fn report_violation(&self, _violation: CspViolation) {
        // In a real implementation, this would send the violation report
        // to the report-uri endpoint specified in the policy
    }
}

impl CspDirective {
    /// Convert directive to its string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            CspDirective::DefaultSrc => "default-src",
            CspDirective::ScriptSrc => "script-src",
            CspDirective::StyleSrc => "style-src",
            CspDirective::ImgSrc => "img-src",
            CspDirective::ConnectSrc => "connect-src",
            CspDirective::FontSrc => "font-src",
            CspDirective::ObjectSrc => "object-src",
            CspDirective::MediaSrc => "media-src",
            CspDirective::FrameSrc => "frame-src",
            CspDirective::ReportUri => "report-uri",
        }
    }
}
