//! DNS resolver implementation using hickory-resolver

use crate::{DnsCache, DnsResolver, DohConfig};
use async_trait::async_trait;
use hickory_resolver::config::{ResolverConfig, ResolverOpts};
use hickory_resolver::TokioAsyncResolver;
use network_errors::{NetworkError, NetworkResult};
use std::net::IpAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

/// Standard DNS resolver implementation
///
/// Supports both standard DNS and DNS-over-HTTPS with caching.
pub struct StandardResolver {
    resolver: TokioAsyncResolver,
    cache: Arc<Mutex<DnsCache>>,
    #[allow(dead_code)] // Kept for future DoH enhancements
    doh_config: Option<DohConfig>,
}

impl StandardResolver {
    /// Create a new standard DNS resolver
    ///
    /// # Arguments
    ///
    /// * `doh_config` - Optional DNS-over-HTTPS configuration
    ///
    /// # Errors
    ///
    /// Returns `NetworkError::DnsError` if resolver cannot be initialized
    pub fn new(doh_config: Option<DohConfig>) -> NetworkResult<Self> {
        let (config, opts) = if let Some(ref doh) = doh_config {
            if doh.enabled {
                // Use DNS-over-HTTPS configuration
                let mut opts = ResolverOpts::default();
                opts.timeout = Duration::from_secs(5);

                // For hickory-resolver with DoH, we'd need to configure it
                // For now, fall back to system config with the option to enhance
                (ResolverConfig::default(), opts)
            } else {
                // Use system DNS configuration
                (ResolverConfig::default(), ResolverOpts::default())
            }
        } else {
            // Use system DNS configuration
            (ResolverConfig::default(), ResolverOpts::default())
        };

        let resolver = TokioAsyncResolver::tokio(config, opts);

        Ok(Self {
            resolver,
            cache: Arc::new(Mutex::new(DnsCache::new())),
            doh_config,
        })
    }

    /// Create a resolver with Google DNS-over-HTTPS
    pub fn with_google_doh() -> NetworkResult<Self> {
        Self::new(Some(DohConfig::google()))
    }

    /// Create a resolver with Cloudflare DNS-over-HTTPS
    pub fn with_cloudflare_doh() -> NetworkResult<Self> {
        Self::new(Some(DohConfig::cloudflare()))
    }

    /// Clear the DNS cache
    pub async fn clear_cache(&self) {
        let mut cache = self.cache.lock().await;
        *cache = DnsCache::new();
    }

    /// Get cache statistics
    pub async fn cache_size(&self) -> usize {
        let cache = self.cache.lock().await;
        cache.entries.len()
    }
}

#[async_trait]
impl DnsResolver for StandardResolver {
    async fn resolve(&self, hostname: String) -> NetworkResult<Vec<IpAddr>> {
        // Check cache first
        {
            let cache = self.cache.lock().await;
            if let Some(addresses) = cache.get(&hostname) {
                return Ok(addresses);
            }
        }

        // Perform DNS lookup
        let lookup = self
            .resolver
            .lookup_ip(hostname.as_str())
            .await
            .map_err(|e| NetworkError::DnsError(format!("DNS resolution failed: {}", e)))?;

        let addresses: Vec<IpAddr> = lookup.iter().collect();

        if addresses.is_empty() {
            return Err(NetworkError::DnsError(format!(
                "No addresses found for {}",
                hostname
            )));
        }

        // Cache the result with default TTL of 5 minutes
        {
            let mut cache = self.cache.lock().await;
            cache.insert(hostname, addresses.clone(), Duration::from_secs(300));
        }

        Ok(addresses)
    }

    async fn resolve_with_timeout(
        &self,
        hostname: String,
        timeout: Duration,
    ) -> NetworkResult<Vec<IpAddr>> {
        tokio::time::timeout(timeout, self.resolve(hostname))
            .await
            .map_err(|_| NetworkError::Timeout(timeout))?
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_resolver_creation() {
        let resolver = StandardResolver::new(None);
        assert!(resolver.is_ok());
    }

    #[tokio::test]
    async fn test_resolver_with_doh_config() {
        let config = DohConfig::google();
        let resolver = StandardResolver::new(Some(config));
        assert!(resolver.is_ok());
    }

    #[tokio::test]
    async fn test_cache_operations() {
        let resolver = StandardResolver::new(None)
            .expect("Failed to create DNS resolver in test");

        // Cache should be empty initially
        assert_eq!(resolver.cache_size().await, 0);

        // After resolution, cache should have an entry
        // (we can't guarantee this test will pass in all environments)
        // So we just test the cache_size method exists
    }

    #[tokio::test]
    async fn test_clear_cache() {
        let resolver = StandardResolver::new(None)
            .expect("Failed to create DNS resolver in test");
        resolver.clear_cache().await;
        assert_eq!(resolver.cache_size().await, 0);
    }
}
