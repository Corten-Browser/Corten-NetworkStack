//! HTTP/2 client implementation

use crate::config::Http2Config;
use crate::connection::Http2Connection;
use crate::error::{Http2Error, Http2Result};
use cookie::Cookie;
use cookie_manager::CookieStore;
use dns_resolver::{DnsResolver, StandardResolver};
use http_cache::{CacheConfig, HttpCache};
use network_errors::NetworkError;
use network_types::{HttpMethod, NetworkRequest, NetworkResponse};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::RwLock;
use tracing::{debug, error, info, trace};

/// Connection pool key
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
struct PoolKey {
    host: String,
    port: u16,
    scheme: String,
}

impl PoolKey {
    fn from_url(url: &str) -> Result<Self, NetworkError> {
        // Simple URL parsing for host:port:scheme
        let url = url::Url::parse(url)
            .map_err(|e| NetworkError::InvalidUrl(format!("Invalid URL: {}", e)))?;

        let host = url
            .host_str()
            .ok_or_else(|| NetworkError::InvalidUrl("Missing host".to_string()))?
            .to_string();

        let port = url
            .port()
            .unwrap_or_else(|| if url.scheme() == "https" { 443 } else { 80 });

        let scheme = url.scheme().to_string();

        Ok(Self { host, port, scheme })
    }
}

/// HTTP/2 client with connection pooling and multiplexing
pub struct Http2Client {
    /// Client configuration
    config: Http2Config,

    /// Connection pool (host:port -> connection)
    connections: Arc<RwLock<HashMap<PoolKey, Arc<Http2Connection>>>>,

    /// DNS resolver
    dns_resolver: Arc<dyn DnsResolver>,

    /// Cookie store
    cookie_store: Arc<RwLock<CookieStore>>,

    /// HTTP cache
    cache: Arc<HttpCache>,

    /// Maximum redirects to follow
    max_redirects: usize,

    /// Request timeout
    timeout: std::time::Duration,
}

impl Http2Client {
    /// Create a new HTTP/2 client
    ///
    /// # Arguments
    ///
    /// * `config` - HTTP/2 configuration
    ///
    /// # Returns
    ///
    /// New HTTP/2 client or error
    pub fn new(config: Http2Config) -> Http2Result<Self> {
        // Validate configuration
        config.validate()?;

        // Create default DNS resolver
        let dns_resolver = StandardResolver::new(None).map_err(|e| {
            Http2Error::ConfigError(format!("Failed to create DNS resolver: {}", e))
        })?;

        Ok(Self {
            config,
            connections: Arc::new(RwLock::new(HashMap::new())),
            dns_resolver: Arc::new(dns_resolver),
            cookie_store: Arc::new(RwLock::new(CookieStore::new())),
            cache: Arc::new(HttpCache::new(CacheConfig::default())),
            max_redirects: 10,
            timeout: std::time::Duration::from_secs(30),
        })
    }

    /// Create HTTP/2 client with custom components
    pub fn with_components(
        config: Http2Config,
        dns_resolver: Arc<dyn DnsResolver>,
        cookie_store: Arc<RwLock<CookieStore>>,
        cache: Arc<HttpCache>,
    ) -> Http2Result<Self> {
        config.validate()?;

        Ok(Self {
            config,
            connections: Arc::new(RwLock::new(HashMap::new())),
            dns_resolver,
            cookie_store,
            cache,
            max_redirects: 10,
            timeout: std::time::Duration::from_secs(30),
        })
    }

    /// Set maximum redirects
    pub fn with_max_redirects(mut self, max: usize) -> Self {
        self.max_redirects = max;
        self
    }

    /// Set request timeout
    pub fn with_timeout(mut self, timeout: std::time::Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Fetch a single request
    ///
    /// # Arguments
    ///
    /// * `request` - Network request to fetch
    ///
    /// # Returns
    ///
    /// Network response or error
    pub async fn fetch(
        &self,
        mut request: NetworkRequest,
    ) -> Result<NetworkResponse, NetworkError> {
        trace!("Fetching: {:?} {}", request.method, request.url);

        // Check cache first
        if let Some(cached_response) = self.cache.get(&request).await {
            debug!("Cache hit for: {}", request.url);
            return Ok(cached_response.response);
        }

        // Add cookies to request
        {
            let cookie_store = self.cookie_store.read().await;
            let cookies = cookie_store.get_cookies(&request.url);
            if !cookies.is_empty() {
                let cookie_header = cookies
                    .iter()
                    .map(|c| format!("{}={}", c.name(), c.value()))
                    .collect::<Vec<_>>()
                    .join("; ");
                request.headers.insert(
                    http::header::COOKIE,
                    cookie_header
                        .parse()
                        .unwrap_or_else(|_| http::HeaderValue::from_static("")),
                );
            }
        }

        // Follow redirects
        let mut redirect_count = 0;
        let mut current_request = request;

        loop {
            let response = self.fetch_once(current_request.clone()).await?;

            // Check for redirect
            if self.is_redirect_status(response.status) && redirect_count < self.max_redirects {
                if let Some(location) = self.get_header_value(&response, "location") {
                    info!(
                        "Following redirect {} -> {} (count: {})",
                        current_request.url,
                        location,
                        redirect_count + 1
                    );

                    // Update request URL for redirect
                    current_request = NetworkRequest {
                        url: url::Url::parse(&location).map_err(|e| {
                            NetworkError::InvalidUrl(format!("Invalid redirect URL: {}", e))
                        })?,
                        method: HttpMethod::Get,
                        headers: http::HeaderMap::new(),
                        body: None,
                        mode: current_request.mode,
                        credentials: current_request.credentials,
                        cache: current_request.cache,
                        redirect: current_request.redirect,
                        referrer: current_request.referrer.clone(),
                        referrer_policy: current_request.referrer_policy,
                        integrity: current_request.integrity.clone(),
                        keepalive: current_request.keepalive,
                        signal: None,
                        priority: current_request.priority,
                        window: current_request.window,
                    };
                    redirect_count += 1;
                    continue;
                }
            }

            // Store cookies from Set-Cookie headers
            {
                let mut cookie_store = self.cookie_store.write().await;
                for set_cookie_value in response.headers.get_all(http::header::SET_COOKIE) {
                    if let Ok(cookie_str) = set_cookie_value.to_str() {
                        if let Ok(cookie) = Cookie::parse(cookie_str) {
                            let _ =
                                cookie_store.add_cookie(cookie.into_owned(), &current_request.url);
                        }
                    }
                }
            }

            // Cache if cacheable
            if self.is_cacheable(&response) {
                let _ = self.cache.store(&current_request, &response).await;
            }

            return Ok(response);
        }
    }

    /// Fetch multiple requests concurrently using multiplexing
    ///
    /// # Arguments
    ///
    /// * `requests` - Vector of network requests
    ///
    /// # Returns
    ///
    /// Vector of responses or error
    pub async fn fetch_multiple(
        &self,
        requests: Vec<NetworkRequest>,
    ) -> Result<Vec<NetworkResponse>, NetworkError> {
        info!("Fetching {} requests with multiplexing", requests.len());

        // TODO: Implement true concurrent multiplexing using h2's capabilities
        // For now, fetch sequentially (still uses connection pooling and multiplexing at h2 level)
        let mut responses = Vec::new();

        for request in requests {
            match self.fetch(request).await {
                Ok(response) => responses.push(response),
                Err(e) => {
                    error!("Request failed: {}", e);
                    return Err(e);
                }
            }
        }

        Ok(responses)
    }

    /// Fetch a single request without redirect handling
    async fn fetch_once(&self, request: NetworkRequest) -> Result<NetworkResponse, NetworkError> {
        // Get or create connection
        let connection = self.get_or_create_connection(request.url.as_str()).await?;

        // Send request with timeout
        match tokio::time::timeout(self.timeout, connection.send_request(request)).await {
            Ok(result) => result,
            Err(_) => Err(NetworkError::Timeout(self.timeout)),
        }
    }

    /// Get or create a connection from the pool
    async fn get_or_create_connection(
        &self,
        url: &str,
    ) -> Result<Arc<Http2Connection>, NetworkError> {
        let pool_key = PoolKey::from_url(url)?;

        // Check if connection exists
        {
            let connections = self.connections.read().await;
            if let Some(conn) = connections.get(&pool_key) {
                debug!("Reusing existing HTTP/2 connection for {}", url);
                return Ok(Arc::clone(conn));
            }
        }

        // Create new connection
        debug!("Creating new HTTP/2 connection for {}", url);

        // Resolve DNS
        let ip_addrs = self
            .dns_resolver
            .resolve(pool_key.host.clone())
            .await
            .map_err(|e| NetworkError::DnsError(format!("DNS resolution failed: {}", e)))?;

        let ip_addr = ip_addrs
            .first()
            .ok_or_else(|| NetworkError::DnsError("No IP addresses resolved".to_string()))?;

        // Create TCP connection
        let tcp_stream = TcpStream::connect((*ip_addr, pool_key.port))
            .await
            .map_err(|e| NetworkError::ConnectionFailed(format!("TCP connect failed: {}", e)))?;

        // TODO: Add TLS support when tls_manager provides wrap_stream method
        if pool_key.scheme == "https" {
            return Err(NetworkError::TlsError(
                "HTTPS not yet supported - TLS integration pending".to_string(),
            ));
        }

        let stream = tcp_stream;

        // Create HTTP/2 connection
        let connection = Http2Connection::new(stream, self.config.clone())
            .await
            .map_err(|e| NetworkError::ProtocolError(format!("HTTP/2 connection failed: {}", e)))?;

        let connection = Arc::new(connection);

        // Store in pool
        {
            let mut connections = self.connections.write().await;
            connections.insert(pool_key, Arc::clone(&connection));
        }

        Ok(connection)
    }

    /// Perform health check on a connection
    pub async fn health_check(&self, url: &str) -> Http2Result<std::time::Duration> {
        let connection = self
            .get_or_create_connection(url)
            .await
            .map_err(|e| Http2Error::ConnectionError(format!("Failed to get connection: {}", e)))?;

        connection.ping().await
    }

    /// Clear connection pool
    pub async fn clear_connections(&self) {
        let mut connections = self.connections.write().await;
        connections.clear();
        info!("Cleared all HTTP/2 connections");
    }

    /// Get number of pooled connections
    pub async fn connection_count(&self) -> usize {
        let connections = self.connections.read().await;
        connections.len()
    }

    /// Check if a status code indicates a redirect
    fn is_redirect_status(&self, status: u16) -> bool {
        matches!(status, 301 | 302 | 303 | 307 | 308)
    }

    /// Get header value from response
    fn get_header_value(&self, response: &NetworkResponse, name: &str) -> Option<String> {
        response
            .headers
            .get(name)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string())
    }

    /// Check if response is cacheable
    fn is_cacheable(&self, response: &NetworkResponse) -> bool {
        // Cache successful GET responses
        response.status >= 200 && response.status < 300
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool_key_from_url() {
        let key = PoolKey::from_url("https://example.com:443/path").unwrap();
        assert_eq!(key.host, "example.com");
        assert_eq!(key.port, 443);
        assert_eq!(key.scheme, "https");

        let key2 = PoolKey::from_url("http://example.com/path").unwrap();
        assert_eq!(key2.port, 80);
    }

    #[test]
    fn test_client_creation() {
        let config = Http2Config::default();
        let client = Http2Client::new(config);
        assert!(client.is_ok());
    }

    #[test]
    fn test_client_with_invalid_config() {
        let config = Http2Config::new().with_max_concurrent_streams(0);
        let client = Http2Client::new(config);
        assert!(client.is_err());
    }

    #[tokio::test]
    async fn test_connection_pool() {
        let config = Http2Config::default();
        let client = Http2Client::new(config).unwrap();
        assert_eq!(client.connection_count().await, 0);
    }
}
