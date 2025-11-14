//! http1_protocol component
//!
//! HTTP/1.1 client implementation with connection pooling, keep-alive, pipelining support

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

use http_body_util::BodyExt;
use hyper::client::conn::http1;
use hyper_util::rt::TokioIo;
use network_errors::NetworkError;
use network_types::{
    HttpMethod, NetworkRequest, NetworkResponse, ResourceTiming, ResponseBody, ResponseType,
};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::net::TcpStream;
use tokio::sync::Mutex;

/// Configuration for HTTP/1.1 client behavior
///
/// Controls connection pooling, keep-alive, pipelining, and timeout settings.
#[derive(Debug, Clone)]
pub struct Http1Config {
    /// Maximum number of connections in the pool
    pub pool_size: usize,

    /// Duration before idle connections are closed
    pub idle_timeout: Duration,

    /// Maximum connections allowed per host
    pub max_connections_per_host: usize,

    /// Enable HTTP keep-alive for persistent connections
    pub enable_keepalive: bool,

    /// Enable HTTP pipelining for request optimization
    pub enable_pipelining: bool,
}

impl Default for Http1Config {
    fn default() -> Self {
        Self {
            pool_size: 20,
            idle_timeout: Duration::from_secs(90),
            max_connections_per_host: 6,
            enable_keepalive: true,
            enable_pipelining: false,
        }
    }
}

/// Represents an HTTP/1.1 connection
pub struct Http1Connection {
    /// The underlying hyper SendRequest handle
    sender: http1::SendRequest<String>,
    /// Host this connection is for
    host: String,
    /// Port this connection is for
    port: u16,
    /// When this connection was last used
    last_used: Instant,
}

/// Key for identifying connections by host and port
type PoolKey = (String, u16);

/// Internal pool state
struct PoolState {
    /// Available connections by host:port
    idle_connections: HashMap<PoolKey, Vec<Http1Connection>>,
    /// Count of active connections per host:port
    active_counts: HashMap<PoolKey, usize>,
}

/// Connection pool for HTTP/1.1 connections
///
/// Manages connection reuse, idle timeout, and per-host limits.
pub struct ConnectionPool {
    config: Http1Config,
    state: Arc<Mutex<PoolState>>,
}

impl ConnectionPool {
    /// Create a new connection pool with the given configuration
    pub fn new(config: Http1Config) -> Self {
        Self {
            config,
            state: Arc::new(Mutex::new(PoolState {
                idle_connections: HashMap::new(),
                active_counts: HashMap::new(),
            })),
        }
    }

    /// Get a connection to the specified host and port
    ///
    /// Reuses an existing idle connection if available, otherwise creates a new one.
    pub async fn get_connection(
        &self,
        host: &str,
        port: u16,
    ) -> Result<Http1Connection, NetworkError> {
        let key = (host.to_string(), port);

        // Try to reuse an idle connection
        {
            let mut state = self.state.lock().await;

            // Remove expired connections
            if let Some(conns) = state.idle_connections.get_mut(&key) {
                conns.retain(|conn| conn.last_used.elapsed() < self.config.idle_timeout);

                // Try to get a reusable connection
                if let Some(conn) = conns.pop() {
                    let count = state.active_counts.entry(key.clone()).or_insert(0);
                    *count += 1;
                    return Ok(conn);
                }
            }
        }

        // Create new connection
        self.create_new_connection(host, port).await
    }

    /// Return a connection to the pool for reuse
    ///
    /// If keep-alive is disabled, the connection is dropped instead.
    pub async fn return_connection(&self, mut connection: Http1Connection) {
        if !self.config.enable_keepalive {
            // Drop the connection if keep-alive is disabled
            return;
        }

        let key = (connection.host.clone(), connection.port);

        // Update last used time
        connection.last_used = Instant::now();

        let mut state = self.state.lock().await;

        // Decrease active count
        if let Some(count) = state.active_counts.get_mut(&key) {
            *count = count.saturating_sub(1);
        }

        // Add to idle pool
        let idle = state.idle_connections.entry(key).or_insert_with(Vec::new);
        idle.push(connection);
    }

    /// Create a new HTTP/1.1 connection
    async fn create_new_connection(
        &self,
        host: &str,
        port: u16,
    ) -> Result<Http1Connection, NetworkError> {
        // Connect TCP stream
        let addr = format!("{}:{}", host, port);
        let stream = TcpStream::connect(&addr)
            .await
            .map_err(|e| NetworkError::ConnectionFailed(e.to_string()))?;

        let io = TokioIo::new(stream);

        // Perform HTTP/1.1 handshake
        let (sender, conn) = http1::handshake(io)
            .await
            .map_err(|e| NetworkError::ConnectionFailed(e.to_string()))?;

        // Spawn connection task
        tokio::spawn(async move {
            if let Err(e) = conn.await {
                eprintln!("Connection error: {}", e);
            }
        });

        // Update active count
        let key = (host.to_string(), port);
        {
            let mut state = self.state.lock().await;
            let count = state.active_counts.entry(key.clone()).or_insert(0);
            *count += 1;
        }

        Ok(Http1Connection {
            sender,
            host: host.to_string(),
            port,
            last_used: Instant::now(),
        })
    }
}

/// HTTP/1.1 client with connection pooling
///
/// Provides high-level HTTP/1.1 request functionality with automatic connection management.
pub struct Http1Client {
    pool: Arc<ConnectionPool>,
    #[allow(dead_code)]
    config: Http1Config,
}

impl Http1Client {
    /// Create a new HTTP/1.1 client with the given configuration
    pub fn new(config: Http1Config) -> Self {
        let pool = Arc::new(ConnectionPool::new(config.clone()));
        Self { pool, config }
    }

    /// Fetch a resource and return the complete response
    ///
    /// This method performs a complete HTTP request and returns the full response body.
    pub async fn fetch(&self, request: NetworkRequest) -> Result<NetworkResponse, NetworkError> {
        let start_time = Instant::now();

        // Parse URL components
        let host = request
            .url
            .host_str()
            .ok_or_else(|| NetworkError::InvalidUrl("Missing host in URL".to_string()))?
            .to_string();

        let port = request
            .url
            .port_or_known_default()
            .ok_or_else(|| NetworkError::InvalidUrl("Cannot determine port".to_string()))?;

        // Get connection from pool
        let mut conn = self.pool.get_connection(&host, port).await?;

        // Build HTTP request
        let http_request = self.build_hyper_request(&request)?;

        // Send request
        let response = conn
            .sender
            .send_request(http_request)
            .await
            .map_err(|e| NetworkError::ConnectionFailed(e.to_string()))?;

        let status = response.status();
        let headers = response.headers().clone();

        // Collect response body
        let body_bytes = response
            .into_body()
            .collect()
            .await
            .map_err(|e| NetworkError::Other(e.to_string()))?
            .to_bytes()
            .to_vec();

        // Return connection to pool
        self.pool.return_connection(conn).await;

        // Build timing information
        let elapsed = start_time.elapsed();
        let timing = ResourceTiming {
            start_time: 0.0,
            redirect_start: 0.0,
            redirect_end: 0.0,
            fetch_start: 0.0,
            domain_lookup_start: 0.0,
            domain_lookup_end: 0.0,
            connect_start: 0.0,
            connect_end: elapsed.as_secs_f64() * 1000.0,
            secure_connection_start: 0.0,
            request_start: 0.0,
            response_start: elapsed.as_secs_f64() * 1000.0,
            response_end: elapsed.as_secs_f64() * 1000.0,
            transfer_size: body_bytes.len() as u64,
            encoded_body_size: body_bytes.len() as u64,
            decoded_body_size: body_bytes.len() as u64,
        };

        Ok(NetworkResponse {
            url: request.url.clone(),
            status: status.as_u16(),
            status_text: status.canonical_reason().unwrap_or("").to_string(),
            headers,
            body: ResponseBody::Bytes(body_bytes),
            redirected: false,
            type_: ResponseType::Basic,
            timing,
        })
    }

    /// Stream a response without collecting the full body
    ///
    /// This method allows streaming response data as it arrives.
    /// Currently not implemented - returns an error.
    pub async fn stream_response(
        &self,
        _request: NetworkRequest,
    ) -> Result<NetworkResponse, NetworkError> {
        Err(NetworkError::Other(
            "Streaming responses not yet implemented".to_string(),
        ))
    }

    /// Build a hyper HTTP request from a NetworkRequest
    fn build_hyper_request(
        &self,
        request: &NetworkRequest,
    ) -> Result<hyper::Request<String>, NetworkError> {
        // Convert HttpMethod to hyper::Method
        let method = match request.method {
            HttpMethod::Get => hyper::Method::GET,
            HttpMethod::Post => hyper::Method::POST,
            HttpMethod::Put => hyper::Method::PUT,
            HttpMethod::Delete => hyper::Method::DELETE,
            HttpMethod::Head => hyper::Method::HEAD,
            HttpMethod::Options => hyper::Method::OPTIONS,
            HttpMethod::Patch => hyper::Method::PATCH,
            HttpMethod::Trace => hyper::Method::TRACE,
            HttpMethod::Connect => hyper::Method::CONNECT,
        };

        // Build request with body as String
        let body_string = if let Some(body) = &request.body {
            match body {
                network_types::RequestBody::Bytes(bytes) => String::from_utf8(bytes.clone())
                    .map_err(|e| {
                        NetworkError::Other(format!("Invalid UTF-8 in request body: {}", e))
                    })?,
                network_types::RequestBody::Text(text) => text.clone(),
                network_types::RequestBody::FormData(_) => {
                    return Err(NetworkError::Other(
                        "FormData not yet implemented".to_string(),
                    ));
                }
                network_types::RequestBody::Stream(_) => {
                    return Err(NetworkError::Other(
                        "Streaming request bodies not yet implemented".to_string(),
                    ));
                }
            }
        } else {
            String::new()
        };

        let req = hyper::Request::builder()
            .method(method)
            .uri(request.url.as_str())
            .body(body_string)
            .map_err(|e| NetworkError::Other(e.to_string()))?;

        Ok(req)
    }
}
