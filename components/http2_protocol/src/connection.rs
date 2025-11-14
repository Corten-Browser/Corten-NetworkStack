//! HTTP/2 connection management

use crate::config::Http2Config;
use crate::error::{Http2Error, Http2Result};
use bytes::Bytes;
use h2::client::{self, SendRequest};
use http::{Method, Request, Response, Version};
use network_errors::NetworkError;
use network_types::{
    HttpMethod, NetworkRequest, NetworkResponse, ResourceTiming, ResponseBody, ResponseType,
};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tracing::{debug, error, trace};

/// HTTP/2 connection wrapper
pub struct Http2Connection {
    /// HTTP/2 send request handle
    sender: Arc<Mutex<SendRequest<Bytes>>>,

    /// Connection configuration
    #[allow(dead_code)] // Reserved for future use (custom settings, timeouts, etc.)
    config: Http2Config,

    /// Connection start time for metrics
    created_at: Instant,
}

impl Http2Connection {
    /// Create a new HTTP/2 connection
    ///
    /// # Arguments
    ///
    /// * `stream` - TCP stream to use for connection
    /// * `config` - HTTP/2 configuration
    ///
    /// # Returns
    ///
    /// New HTTP/2 connection or error
    pub async fn new(stream: TcpStream, config: Http2Config) -> Http2Result<Self> {
        // Validate configuration first
        config.validate()?;

        // Create HTTP/2 handshake
        let (sender, connection) = client::handshake(stream)
            .await
            .map_err(|e| Http2Error::ConnectionError(format!("HTTP/2 handshake failed: {}", e)))?;

        // Spawn connection driver
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                error!("HTTP/2 connection error: {}", e);
            }
        });

        debug!("HTTP/2 connection established");

        Ok(Self {
            sender: Arc::new(Mutex::new(sender)),
            config,
            created_at: Instant::now(),
        })
    }

    /// Send a request over this connection
    ///
    /// # Arguments
    ///
    /// * `request` - Network request to send
    ///
    /// # Returns
    ///
    /// Network response or error
    pub async fn send_request(
        &self,
        request: NetworkRequest,
    ) -> Result<NetworkResponse, NetworkError> {
        trace!(
            "Sending HTTP/2 request: {:?} {}",
            request.method,
            request.url
        );

        // Convert NetworkRequest to http::Request
        let http_request = self.convert_request(&request)?;

        // Get sender and prepare request
        let (response, _) = {
            let sender = self.sender.lock().await;

            // Clone the sender (cheaply clonable per h2 docs) and check if connection is ready
            let mut ready_sender =
                sender.clone().ready().await.map_err(|e| {
                    NetworkError::ProtocolError(format!("Connection not ready: {}", e))
                })?;

            // Send request
            ready_sender
                .send_request(http_request, false)
                .map_err(|e| {
                    NetworkError::ProtocolError(format!("Failed to send request: {}", e))
                })?
        };

        // Wait for response
        let http_response = response.await.map_err(|e| {
            NetworkError::ProtocolError(format!("Failed to receive response: {}", e))
        })?;

        // Convert response
        self.convert_response(http_response, &request).await
    }

    /// Send a ping to measure round-trip time
    ///
    /// # Returns
    ///
    /// Round-trip time or error
    pub async fn ping(&self) -> Http2Result<Duration> {
        let start = Instant::now();

        // TODO: Implement actual ping using h2's ping mechanism
        // For now, return a placeholder duration
        let rtt = start.elapsed();
        debug!("HTTP/2 ping RTT (placeholder): {:?}", rtt);
        Ok(rtt)
    }

    /// Get connection age
    pub fn age(&self) -> Duration {
        self.created_at.elapsed()
    }

    /// Convert HttpMethod to http::Method
    fn convert_http_method(method: &HttpMethod) -> Method {
        match method {
            HttpMethod::Get => Method::GET,
            HttpMethod::Head => Method::HEAD,
            HttpMethod::Post => Method::POST,
            HttpMethod::Put => Method::PUT,
            HttpMethod::Delete => Method::DELETE,
            HttpMethod::Connect => Method::CONNECT,
            HttpMethod::Options => Method::OPTIONS,
            HttpMethod::Trace => Method::TRACE,
            HttpMethod::Patch => Method::PATCH,
        }
    }

    /// Convert NetworkRequest to http::Request
    fn convert_request(
        &self,
        network_request: &NetworkRequest,
    ) -> Result<Request<()>, NetworkError> {
        let mut builder = Request::builder()
            .method(Self::convert_http_method(&network_request.method))
            .uri(network_request.url.as_str())
            .version(Version::HTTP_2);

        // Add headers
        for (name, value) in network_request.headers.iter() {
            builder = builder.header(name, value);
        }

        builder
            .body(())
            .map_err(|e| NetworkError::ProtocolError(format!("Failed to build request: {}", e)))
    }

    /// Convert http::Response to NetworkResponse
    async fn convert_response(
        &self,
        http_response: Response<h2::RecvStream>,
        network_request: &NetworkRequest,
    ) -> Result<NetworkResponse, NetworkError> {
        let status = http_response.status();
        let headers = http_response.headers().clone();
        let mut body_stream = http_response.into_body();

        // Collect body
        let mut body_bytes = Vec::new();
        while let Some(chunk_result) = body_stream.data().await {
            let chunk = chunk_result
                .map_err(|e| NetworkError::ProtocolError(format!("Failed to read body: {}", e)))?;
            body_bytes.extend_from_slice(&chunk);
            let _ = body_stream.flow_control().release_capacity(chunk.len());
        }

        // Build NetworkResponse manually
        Ok(NetworkResponse {
            url: network_request.url.clone(),
            status: status.as_u16(),
            status_text: status.canonical_reason().unwrap_or("").to_string(),
            headers,
            body: ResponseBody::Bytes(body_bytes),
            redirected: false,
            type_: ResponseType::Basic,
            timing: ResourceTiming::default(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_config_validation() {
        // Test that config validation happens during connection creation
        let config = Http2Config::new().with_max_concurrent_streams(0);
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_http_method_conversion() {
        assert_eq!(
            Http2Connection::convert_http_method(&HttpMethod::Get),
            Method::GET
        );
        assert_eq!(
            Http2Connection::convert_http_method(&HttpMethod::Post),
            Method::POST
        );
    }
}
