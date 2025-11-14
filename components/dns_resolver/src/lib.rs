//! dns_resolver component
//!
//! DNS resolution with DNS-over-HTTPS support, caching, and async resolution
//!
//! This component provides async DNS resolution capabilities with support for:
//! - Standard DNS resolution
//! - DNS-over-HTTPS (DoH)
//! - DNS result caching with TTL
//! - Timeout handling
//!
//! # Examples
//!
//! ```no_run
//! use dns_resolver::{DnsResolver, StandardResolver, DohConfig};
//! use std::time::Duration;
//!
//! # async fn example() -> Result<(), network_errors::NetworkError> {
//! let resolver = StandardResolver::new(None)?;
//! let addresses = resolver.resolve("example.com".to_string()).await?;
//! # Ok(())
//! # }
//! ```

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

use async_trait::async_trait;
use network_errors::NetworkResult;
use std::collections::HashMap;
use std::net::IpAddr;
use std::time::{Duration, Instant};

/// DNS resolver trait
///
/// Provides async DNS resolution methods for hostname lookups.
#[async_trait]
pub trait DnsResolver: Send + Sync {
    /// Resolve a hostname to IP addresses
    ///
    /// # Arguments
    ///
    /// * `hostname` - The hostname to resolve
    ///
    /// # Returns
    ///
    /// A vector of IP addresses associated with the hostname
    ///
    /// # Errors
    ///
    /// Returns `NetworkError::DnsError` if resolution fails
    async fn resolve(&self, hostname: String) -> NetworkResult<Vec<IpAddr>>;

    /// Resolve a hostname with a timeout
    ///
    /// # Arguments
    ///
    /// * `hostname` - The hostname to resolve
    /// * `timeout` - Maximum duration to wait for resolution
    ///
    /// # Returns
    ///
    /// A vector of IP addresses associated with the hostname
    ///
    /// # Errors
    ///
    /// Returns `NetworkError::Timeout` if the operation times out,
    /// or `NetworkError::DnsError` if resolution fails
    async fn resolve_with_timeout(
        &self,
        hostname: String,
        timeout: Duration,
    ) -> NetworkResult<Vec<IpAddr>>;
}

/// Cache entry with TTL
struct CacheEntry {
    addresses: Vec<IpAddr>,
    expires_at: Instant,
}

/// DNS cache for storing resolved addresses with TTL
///
/// Caches DNS resolution results to reduce repeated lookups.
pub struct DnsCache {
    entries: HashMap<String, CacheEntry>,
}

impl DnsCache {
    /// Create a new empty DNS cache
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    /// Get cached addresses for a hostname
    ///
    /// Returns `None` if the hostname is not in cache or the entry has expired.
    ///
    /// # Arguments
    ///
    /// * `hostname` - The hostname to lookup
    pub fn get(&self, hostname: &str) -> Option<Vec<IpAddr>> {
        self.entries.get(hostname).and_then(|entry| {
            if Instant::now() < entry.expires_at {
                Some(entry.addresses.clone())
            } else {
                None
            }
        })
    }

    /// Insert addresses into cache with TTL
    ///
    /// # Arguments
    ///
    /// * `hostname` - The hostname to cache
    /// * `addresses` - IP addresses to cache
    /// * `ttl` - Time-to-live for the cache entry
    pub fn insert(&mut self, hostname: String, addresses: Vec<IpAddr>, ttl: Duration) {
        let expires_at = Instant::now() + ttl;
        self.entries.insert(
            hostname,
            CacheEntry {
                addresses,
                expires_at,
            },
        );
    }

    /// Clear expired entries from the cache
    pub fn clear_expired(&mut self) {
        let now = Instant::now();
        self.entries.retain(|_, entry| now < entry.expires_at);
    }
}

impl Default for DnsCache {
    fn default() -> Self {
        Self::new()
    }
}

/// DNS-over-HTTPS configuration
///
/// Configures DNS resolution to use DNS-over-HTTPS (DoH) protocol.
#[derive(Debug, Clone)]
pub struct DohConfig {
    /// Whether DoH is enabled
    pub enabled: bool,
    /// DoH resolver URL (e.g., "https://dns.google/dns-query")
    pub resolver_url: String,
    /// Whether to fallback to standard DNS if DoH fails
    pub use_fallback: bool,
}

impl DohConfig {
    /// Create a new DoH configuration
    ///
    /// # Arguments
    ///
    /// * `enabled` - Whether to enable DoH
    /// * `resolver_url` - The DoH resolver URL
    /// * `use_fallback` - Whether to fallback to standard DNS on failure
    pub fn new(enabled: bool, resolver_url: String, use_fallback: bool) -> Self {
        Self {
            enabled,
            resolver_url,
            use_fallback,
        }
    }

    /// Create configuration for Google DNS-over-HTTPS
    pub fn google() -> Self {
        Self {
            enabled: true,
            resolver_url: "https://dns.google/dns-query".to_string(),
            use_fallback: true,
        }
    }

    /// Create configuration for Cloudflare DNS-over-HTTPS
    pub fn cloudflare() -> Self {
        Self {
            enabled: true,
            resolver_url: "https://cloudflare-dns.com/dns-query".to_string(),
            use_fallback: true,
        }
    }
}

impl Default for DohConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            resolver_url: "https://dns.google/dns-query".to_string(),
            use_fallback: true,
        }
    }
}

mod resolver;

pub use resolver::StandardResolver;
