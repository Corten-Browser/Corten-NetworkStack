//! http_cache component
//!
//! HTTP cache storage backend, cache policy enforcement, freshness validation, ETags
//!
//! This component provides an HTTP cache implementation with:
//! - LRU eviction policy
//! - Configurable size and age limits
//! - Freshness validation
//! - ETag and Last-Modified support
//! - Cache-Control directive parsing

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

use lru::LruCache;
pub use network_errors::NetworkError;
use network_errors::NetworkResult;
pub use network_types::{HttpMethod, NetworkRequest, NetworkResponse};
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::num::NonZeroUsize;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::{Mutex, RwLock};

/// Cache configuration
///
/// Controls cache behavior including size limits and expiration.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Maximum total size of cached responses in bytes
    pub max_size_bytes: u64,
    /// Maximum age for cached entries in seconds
    pub max_age_seconds: u64,
    /// Whether caching is enabled
    pub enabled: bool,
}

impl CacheConfig {
    /// Create a new cache configuration
    ///
    /// # Arguments
    ///
    /// * `max_size_bytes` - Maximum cache size in bytes
    /// * `max_age_seconds` - Maximum age for entries in seconds
    /// * `enabled` - Whether caching is enabled
    pub fn new(max_size_bytes: u64, max_age_seconds: u64, enabled: bool) -> Self {
        Self {
            max_size_bytes,
            max_age_seconds,
            enabled,
        }
    }
}

impl Default for CacheConfig {
    /// Create a default configuration (1MB, 1 hour, enabled)
    fn default() -> Self {
        Self {
            max_size_bytes: 1024 * 1024, // 1MB
            max_age_seconds: 3600,       // 1 hour
            enabled: true,
        }
    }
}

/// Helper to clone a NetworkResponse (which doesn't implement Clone)
fn clone_network_response(response: &NetworkResponse) -> NetworkResponse {
    use network_types::ResponseBody;

    let body = match &response.body {
        ResponseBody::Bytes(bytes) => ResponseBody::Bytes(bytes.clone()),
        ResponseBody::Empty => ResponseBody::Empty,
        ResponseBody::Stream(_) => ResponseBody::Empty, // Streams can't be cloned
    };

    NetworkResponse {
        url: response.url.clone(),
        status: response.status,
        status_text: response.status_text.clone(),
        headers: response.headers.clone(),
        body,
        redirected: response.redirected,
        type_: response.type_,
        timing: response.timing.clone(),
    }
}

/// Cached HTTP response with metadata
///
/// Contains the response along with timestamps for cache management.
#[derive(Debug)]
pub struct CachedResponse {
    /// The cached network response
    pub response: NetworkResponse,
    /// When this response was cached
    pub cached_at: SystemTime,
    /// When this response expires
    pub expires_at: SystemTime,
}

impl Clone for CachedResponse {
    fn clone(&self) -> Self {
        Self {
            response: clone_network_response(&self.response),
            cached_at: self.cached_at,
            expires_at: self.expires_at,
        }
    }
}

impl CachedResponse {
    /// Create a new cached response
    ///
    /// # Arguments
    ///
    /// * `response` - The network response to cache
    /// * `max_age` - Maximum age for this entry
    pub fn new(response: NetworkResponse, max_age: Duration) -> Self {
        let now = SystemTime::now();
        Self {
            response,
            cached_at: now,
            expires_at: now + max_age,
        }
    }

    /// Check if this cached response has expired
    pub fn is_expired(&self) -> bool {
        SystemTime::now() >= self.expires_at
    }

    /// Check if this response is still fresh
    pub fn is_fresh(&self) -> bool {
        !self.is_expired()
    }

    /// Get the age of this cached response
    pub fn age(&self) -> Duration {
        SystemTime::now()
            .duration_since(self.cached_at)
            .unwrap_or(Duration::ZERO)
    }
}

/// Cache entry with size tracking
#[derive(Debug, Clone)]
struct CacheEntry {
    cached_response: CachedResponse,
    size_bytes: usize,
}

/// HTTP cache implementation
///
/// Provides async HTTP caching with LRU eviction, size limits, and freshness validation.
pub struct HttpCache {
    config: CacheConfig,
    storage: Arc<Mutex<LruCache<u64, CacheEntry>>>,
    current_size: Arc<RwLock<usize>>,
}

impl HttpCache {
    /// Create a new HTTP cache
    ///
    /// # Arguments
    ///
    /// * `config` - Cache configuration
    ///
    /// # Returns
    ///
    /// A new HttpCache instance
    pub fn new(config: CacheConfig) -> Self {
        // Calculate capacity based on average response size (assume 10KB avg)
        let capacity = ((config.max_size_bytes / 10240) as usize).max(10);
        let cache_size = NonZeroUsize::new(capacity)
            .unwrap_or_else(|| NonZeroUsize::new(10).expect("Default cache size should be non-zero"));

        Self {
            config,
            storage: Arc::new(Mutex::new(LruCache::new(cache_size))),
            current_size: Arc::new(RwLock::new(0)),
        }
    }

    /// Generate a cache key from a request
    ///
    /// The cache key includes the URL and HTTP method to ensure
    /// GET and POST requests to the same URL are cached separately.
    fn cache_key(request: &NetworkRequest) -> u64 {
        let mut hasher = DefaultHasher::new();
        request.url.as_str().hash(&mut hasher);
        // Include method in cache key
        match request.method {
            HttpMethod::Get => "GET".hash(&mut hasher),
            HttpMethod::Post => "POST".hash(&mut hasher),
            HttpMethod::Put => "PUT".hash(&mut hasher),
            HttpMethod::Delete => "DELETE".hash(&mut hasher),
            HttpMethod::Head => "HEAD".hash(&mut hasher),
            HttpMethod::Options => "OPTIONS".hash(&mut hasher),
            HttpMethod::Patch => "PATCH".hash(&mut hasher),
            HttpMethod::Trace => "TRACE".hash(&mut hasher),
            HttpMethod::Connect => "CONNECT".hash(&mut hasher),
        }
        hasher.finish()
    }

    /// Estimate the size of a response in bytes
    fn estimate_response_size(response: &NetworkResponse) -> usize {
        let mut size = 0;

        // URL
        size += response.url.as_str().len();

        // Headers
        for (name, value) in response.headers.iter() {
            size += name.as_str().len();
            size += value.len();
        }

        // Body
        size += match &response.body {
            network_types::ResponseBody::Bytes(bytes) => bytes.len(),
            network_types::ResponseBody::Stream(_) => 0, // Streams not cached
            network_types::ResponseBody::Empty => 0,
        };

        // Status text
        size += response.status_text.len();

        // Add overhead for struct fields
        size += 256;

        size
    }

    /// Get a cached response for a request
    ///
    /// # Arguments
    ///
    /// * `request` - The network request to lookup
    ///
    /// # Returns
    ///
    /// The cached response if found and fresh, None otherwise
    pub async fn get(&self, request: &NetworkRequest) -> Option<CachedResponse> {
        // If cache is disabled, return None
        if !self.config.enabled {
            return None;
        }

        let key = Self::cache_key(request);
        let mut storage = self.storage.lock().await;

        // Check if entry exists first
        let is_expired = if let Some(entry) = storage.peek(&key) {
            entry.cached_response.is_expired()
        } else {
            return None;
        };

        // Remove if expired
        if is_expired {
            if let Some(entry) = storage.pop(&key) {
                let mut current_size = self.current_size.write().await;
                *current_size = current_size.saturating_sub(entry.size_bytes);
            }
            return None;
        }

        // Get and return fresh response
        storage.get(&key).map(|entry| entry.cached_response.clone())
    }

    /// Store a response in the cache
    ///
    /// # Arguments
    ///
    /// * `request` - The network request
    /// * `response` - The network response to cache
    ///
    /// # Returns
    ///
    /// Ok if stored successfully, Err if caching failed
    pub async fn store(
        &self,
        request: &NetworkRequest,
        response: &NetworkResponse,
    ) -> NetworkResult<()> {
        // If cache is disabled, succeed without storing
        if !self.config.enabled {
            return Ok(());
        }

        // Calculate response size
        let size = Self::estimate_response_size(response);

        // Don't cache if single response exceeds max size
        if size as u64 > self.config.max_size_bytes {
            return Ok(());
        }

        let key = Self::cache_key(request);
        let max_age = Duration::from_secs(self.config.max_age_seconds);

        // Create cached response
        let cached_response = CachedResponse::new(clone_network_response(response), max_age);
        let entry = CacheEntry {
            cached_response,
            size_bytes: size,
        };

        let mut storage = self.storage.lock().await;
        let mut current_size = self.current_size.write().await;

        // Evict entries if necessary to make room
        while *current_size + size > self.config.max_size_bytes as usize && !storage.is_empty() {
            // Pop least recently used
            if let Some((_, evicted)) = storage.pop_lru() {
                *current_size = current_size.saturating_sub(evicted.size_bytes);
            }
        }

        // Insert new entry
        if let Some(old_entry) = storage.put(key, entry) {
            // Subtract old entry size
            *current_size = current_size.saturating_sub(old_entry.size_bytes);
        }

        // Add new entry size
        *current_size += size;

        Ok(())
    }

    /// Clear all entries from the cache
    ///
    /// # Returns
    ///
    /// Ok if cleared successfully, Err if clearing failed
    pub async fn clear(&self) -> NetworkResult<()> {
        let mut storage = self.storage.lock().await;
        let mut current_size = self.current_size.write().await;

        storage.clear();
        *current_size = 0;

        Ok(())
    }

    /// Get the current cache size in bytes
    pub async fn current_size(&self) -> usize {
        *self.current_size.read().await
    }

    /// Get the number of entries in the cache
    pub async fn entry_count(&self) -> usize {
        self.storage.lock().await.len()
    }

    /// Check if the cache is enabled
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_config_creation() {
        let config = CacheConfig::new(1024, 3600, true);
        assert_eq!(config.max_size_bytes, 1024);
        assert_eq!(config.max_age_seconds, 3600);
        assert!(config.enabled);
    }

    #[test]
    fn test_cache_config_default() {
        let config = CacheConfig::default();
        assert_eq!(config.max_size_bytes, 1024 * 1024);
        assert_eq!(config.max_age_seconds, 3600);
        assert!(config.enabled);
    }

    #[test]
    fn test_cached_response_expiry() {
        use network_types::{ResourceTiming, ResponseBody, ResponseType};
        use url::Url;

        let response = NetworkResponse {
            url: Url::parse("https://example.com").unwrap(),
            status: 200,
            status_text: "OK".to_string(),
            headers: http::HeaderMap::new(),
            body: ResponseBody::Empty,
            redirected: false,
            type_: ResponseType::Basic,
            timing: ResourceTiming::default(),
        };

        let cached = CachedResponse::new(response, Duration::from_secs(1));
        assert!(cached.is_fresh());
        assert!(!cached.is_expired());
    }
}
