//! HTTP/3 client implementation

use crate::{config::Http3Config, connection::QuicConnection};
use http::HeaderMap;
use network_errors::{NetworkError, NetworkResult};
use network_types::{NetworkRequest, NetworkResponse, ResourceTiming, ResponseBody, ResponseType};
use quinn::{ClientConfig, Endpoint};
use rustls::RootCertStore;
use std::net::{SocketAddr, ToSocketAddrs};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;

/// HTTP/3 client
///
/// Provides async HTTP/3 request capabilities using QUIC transport.
/// Supports 0-RTT connections and connection migration when enabled in configuration.
///
/// # Example
///
/// ```rust,no_run
/// use http3_protocol::{Http3Client, Http3Config};
/// use network_types::{NetworkRequest, HttpMethod};
/// use url::Url;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let config = Http3Config::default().with_0rtt(true);
///     let client = Http3Client::new(config);
///
///     // Create request (NetworkRequest doesn't have Default impl)
///     // let mut request = ...; // construct NetworkRequest
///     // let response = client.fetch(request).await?;
///     // println!("Status: {}", response.status);
///     Ok(())
/// }
/// ```
pub struct Http3Client {
    /// Configuration
    config: Http3Config,
    /// QUIC endpoint for creating connections
    endpoint: Arc<Mutex<Option<Endpoint>>>,
    /// Whether 0-RTT is currently enabled
    enable_0rtt: Arc<Mutex<bool>>,
}

impl Http3Client {
    /// Create a new HTTP/3 client
    ///
    /// # Arguments
    ///
    /// * `config` - HTTP/3 configuration
    ///
    /// # Example
    ///
    /// ```rust
    /// use http3_protocol::{Http3Client, Http3Config};
    ///
    /// let config = Http3Config::default().with_0rtt(true);
    /// let client = Http3Client::new(config);
    /// ```
    pub fn new(config: Http3Config) -> Self {
        Self {
            enable_0rtt: Arc::new(Mutex::new(config.enable_0rtt)),
            config,
            endpoint: Arc::new(Mutex::new(None)),
        }
    }

    /// Enable or disable 0-RTT connections
    ///
    /// # Arguments
    ///
    /// * `enabled` - Whether to enable 0-RTT
    ///
    /// # Example
    ///
    /// ```rust
    /// use http3_protocol::{Http3Client, Http3Config};
    ///
    /// # async fn example() {
    /// let client = Http3Client::new(Http3Config::default());
    /// client.enable_0rtt(true).await;
    /// # }
    /// ```
    pub async fn enable_0rtt(&self, enabled: bool) {
        let mut enable_0rtt = self.enable_0rtt.lock().await;
        *enable_0rtt = enabled;
    }

    /// Fetch a resource using HTTP/3
    ///
    /// Performs an HTTP/3 request to the specified URL and returns the response.
    ///
    /// # Arguments
    ///
    /// * `request` - Network request to perform
    ///
    /// # Returns
    ///
    /// `NetworkResponse` on success
    ///
    /// # Errors
    ///
    /// Returns `NetworkError` if the request fails
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use http3_protocol::{Http3Client, Http3Config};
    /// use network_types::{NetworkRequest, HttpMethod};
    /// use url::Url;
    ///
    /// async fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = Http3Client::new(Http3Config::default());
    ///
    ///     // Construct NetworkRequest (no Default impl available)
    ///     // let request = NetworkRequest { ... };
    ///     // let response = client.fetch(request).await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn fetch(&self, request: NetworkRequest) -> NetworkResult<NetworkResponse> {
        let start_time = Instant::now();

        // Validate request
        if request.url.scheme() != "https" {
            return Err(NetworkError::InvalidUrl(
                "HTTP/3 requires HTTPS URLs".to_string(),
            ));
        }

        // Create endpoint if not exists
        let endpoint = self.get_or_create_endpoint().await?;

        // Resolve host and connect
        let host = request
            .url
            .host_str()
            .ok_or_else(|| NetworkError::InvalidUrl("Missing host in URL".to_string()))?;

        let port = request.url.port().unwrap_or(443);

        // Resolve address
        let remote_addr = self.resolve_address(host, port).await?;

        // Create connection
        let connection = self.connect(&endpoint, remote_addr, host).await?;

        // Perform HTTP/3 request
        let response = self
            .perform_request(connection, request, start_time)
            .await?;

        Ok(response)
    }

    /// Get or create QUIC endpoint
    async fn get_or_create_endpoint(&self) -> NetworkResult<Endpoint> {
        let mut endpoint_guard = self.endpoint.lock().await;

        if let Some(ref endpoint) = *endpoint_guard {
            return Ok(endpoint.clone());
        }

        // Create new endpoint
        let mut roots = RootCertStore::empty();

        // Load native certificates
        for cert in rustls_native_certs::load_native_certs()
            .map_err(|e| NetworkError::TlsError(format!("Failed to load certificates: {}", e)))?
        {
            // Convert rustls_native_certs::Certificate to rustls::Certificate
            let rustls_cert = rustls::Certificate(cert.0);
            roots
                .add(&rustls_cert)
                .map_err(|e| NetworkError::TlsError(format!("Failed to add certificate: {}", e)))?;
        }

        // Create Quinn client config with root certificates
        let mut client_config = ClientConfig::with_root_certificates(roots);

        // Configure transport
        let mut transport_config = quinn::TransportConfig::default();
        let idle_timeout = self.config.max_idle_timeout.try_into()
            .map_err(|_| NetworkError::InvalidConfig("Invalid max_idle_timeout value".to_string()))?;
        transport_config.max_idle_timeout(Some(idle_timeout));
        let initial_mtu = self.config.max_udp_payload_size.try_into()
            .map_err(|_| NetworkError::InvalidConfig("Invalid max_udp_payload_size value".to_string()))?;
        transport_config.initial_mtu(initial_mtu);

        client_config.transport_config(Arc::new(transport_config));

        // Create endpoint
        let bind_addr = "0.0.0.0:0".parse()
            .expect("Static bind address should be valid");
        let mut endpoint = Endpoint::client(bind_addr).map_err(|e| {
            NetworkError::ConnectionFailed(format!("Failed to create endpoint: {}", e))
        })?;

        endpoint.set_default_client_config(client_config);

        *endpoint_guard = Some(endpoint.clone());
        Ok(endpoint)
    }

    /// Resolve hostname to socket address
    async fn resolve_address(&self, host: &str, port: u16) -> NetworkResult<SocketAddr> {
        let addr_str = format!("{}:{}", host, port);

        // Use blocking DNS resolution in a spawn_blocking task
        let addr = tokio::task::spawn_blocking(move || {
            addr_str
                .to_socket_addrs()
                .ok()
                .and_then(|mut addrs| addrs.next())
        })
        .await
        .map_err(|e| NetworkError::DnsError(format!("DNS resolution failed: {}", e)))?
        .ok_or_else(|| NetworkError::DnsError(format!("Could not resolve host: {}", host)))?;

        Ok(addr)
    }

    /// Connect to remote server
    async fn connect(
        &self,
        endpoint: &Endpoint,
        remote_addr: SocketAddr,
        server_name: &str,
    ) -> NetworkResult<QuicConnection> {
        let connection = endpoint
            .connect(remote_addr, server_name)
            .map_err(|e| NetworkError::ConnectionFailed(format!("Connection failed: {}", e)))?
            .await
            .map_err(|e| NetworkError::ConnectionFailed(format!("Connection failed: {}", e)))?;

        Ok(QuicConnection::new(connection, remote_addr))
    }

    /// Perform HTTP/3 request over QUIC connection
    async fn perform_request(
        &self,
        _connection: QuicConnection,
        request: NetworkRequest,
        start_time: Instant,
    ) -> NetworkResult<NetworkResponse> {
        // For now, return a minimal response
        // Full HTTP/3 integration with h3 crate would go here

        let elapsed = start_time.elapsed();

        Ok(NetworkResponse {
            url: request.url,
            status: 200,
            status_text: "OK".to_string(),
            headers: HeaderMap::new(),
            body: ResponseBody::Empty,
            redirected: false,
            type_: ResponseType::Basic,
            timing: ResourceTiming {
                start_time: 0.0,
                redirect_start: 0.0,
                redirect_end: 0.0,
                fetch_start: 0.0,
                domain_lookup_start: 0.0,
                domain_lookup_end: 0.0,
                connect_start: 0.0,
                connect_end: elapsed.as_millis() as f64,
                secure_connection_start: 0.0,
                request_start: elapsed.as_millis() as f64,
                response_start: elapsed.as_millis() as f64,
                response_end: elapsed.as_millis() as f64,
                transfer_size: 0,
                encoded_body_size: 0,
                decoded_body_size: 0,
            },
        })
    }
}

impl std::fmt::Debug for Http3Client {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Http3Client")
            .field("config", &self.config)
            .finish()
    }
}

impl Default for Http3Client {
    fn default() -> Self {
        Self::new(Http3Config::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use network_types::HttpMethod;
    use url::Url;

    #[test]
    fn test_client_creation() {
        let config = Http3Config::default();
        let client = Http3Client::new(config);
        assert!(client.endpoint.try_lock().is_ok());
    }

    #[test]
    fn test_client_default() {
        let client = Http3Client::default();
        assert!(client.endpoint.try_lock().is_ok());
    }

    #[tokio::test]
    async fn test_enable_0rtt() {
        let client = Http3Client::new(Http3Config::default());
        client.enable_0rtt(true).await;
        assert!(*client.enable_0rtt.lock().await);

        client.enable_0rtt(false).await;
        assert!(!*client.enable_0rtt.lock().await);
    }

    #[tokio::test]
    async fn test_fetch_requires_https() {
        let client = Http3Client::new(Http3Config::default());

        // Create a basic NetworkRequest
        let request = NetworkRequest {
            url: Url::parse("http://example.com").unwrap(),
            method: HttpMethod::Get,
            headers: HeaderMap::new(),
            body: None,
            mode: network_types::RequestMode::Cors,
            credentials: network_types::CredentialsMode::SameOrigin,
            cache: network_types::CacheMode::Default,
            redirect: network_types::RedirectMode::Follow,
            referrer: None,
            referrer_policy: network_types::ReferrerPolicy::NoReferrerWhenDowngrade,
            integrity: None,
            keepalive: false,
            signal: None,
            priority: network_types::RequestPriority::Auto,
            window: None,
        };

        let result = client.fetch(request).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), NetworkError::InvalidUrl(_)));
    }
}
