//! CSP (Content Security Policy) Processor Component
//!
//! This component provides parsing and validation of Content-Security-Policy headers
//! according to the W3C CSP specification.

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

use std::collections::HashMap;
use url::Url;

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
    /// The document's origin for 'self' checks (scheme + host + port)
    document_origin: Option<Url>,
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
        Ok(Self {
            policy,
            document_origin: None,
        })
    }

    /// Set the document origin for 'self' checks (builder pattern)
    pub fn with_document_origin(mut self, origin: Url) -> Self {
        self.document_origin = Some(origin);
        self
    }

    /// Set the document origin for 'self' checks (mutable)
    pub fn set_document_origin(&mut self, origin: Url) {
        self.document_origin = Some(origin);
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

    /// Check if a source matches the 'self' keyword
    ///
    /// The 'self' keyword matches only sources from the same origin as the document.
    /// Origin is defined as: scheme + host + port
    fn check_self_source(&self, source: &str) -> bool {
        // Parse the source URL
        let source_url = match Url::parse(source) {
            Ok(url) => url,
            Err(_) => return false, // Invalid URL doesn't match 'self'
        };

        // Get document origin (if set)
        let document_origin = match &self.document_origin {
            Some(origin) => origin,
            None => {
                // No document origin set - cannot verify 'self'
                // For security, reject rather than allowing all
                return false;
            }
        };

        // Compare origins (scheme + host + port)
        // All three must match exactly for same-origin
        document_origin.scheme() == source_url.scheme()
            && document_origin.host_str() == source_url.host_str()
            && document_origin.port() == source_url.port()
    }

    /// Check if a source matches an allowed source pattern
    fn source_matches(&self, allowed: &str, actual: &str) -> bool {
        // Handle 'self'
        if allowed == "'self'" {
            return self.check_self_source(actual);
        }

        // Handle wildcard '*' (allows any source)
        if allowed == "*" {
            return true;
        }

        // Handle subdomain wildcards (*.example.com)
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
        if let Some(actual_host) = Self::extract_host(actual) {
            // If allowed is a full URL, extract its host
            let allowed_host = if allowed.contains("://") {
                Self::extract_host(allowed).unwrap_or(allowed.to_string())
            } else {
                allowed.to_string()
            };

            return actual_host == allowed_host || actual_host.starts_with(&format!("{}:", allowed_host));
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
