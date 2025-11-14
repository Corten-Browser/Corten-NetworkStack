//! network_types component
//!
//! Core types for network operations: NetworkRequest, NetworkResponse, HTTP enums,
//! request/response structures.
//!
//! This component provides the fundamental data structures used across the network stack
//! for representing HTTP requests, responses, and related metadata.

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

use bytes::Bytes;
use futures::stream::Stream;
use http::HeaderMap;
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use url::Url;

/// HTTP methods enum
///
/// Represents standard HTTP request methods as defined in RFC 7231.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HttpMethod {
    /// GET method - retrieve data
    Get,
    /// HEAD method - retrieve headers only
    Head,
    /// POST method - submit data
    Post,
    /// PUT method - replace resource
    Put,
    /// DELETE method - remove resource
    Delete,
    /// CONNECT method - establish tunnel
    Connect,
    /// OPTIONS method - describe communication options
    Options,
    /// TRACE method - diagnostic loop-back
    Trace,
    /// PATCH method - partial modification
    Patch,
}

/// Request modes for CORS handling
///
/// Controls how the request interacts with CORS (Cross-Origin Resource Sharing) rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RequestMode {
    /// Navigate mode - for navigation requests
    Navigate,
    /// Same-origin mode - only allow same-origin requests
    SameOrigin,
    /// No-CORS mode - simple requests that don't trigger CORS preflight
    NoCors,
    /// CORS mode - full CORS handling with preflight
    Cors,
}

/// Credentials mode for request authentication
///
/// Controls whether credentials (cookies, authorization headers) are included in requests.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CredentialsMode {
    /// Omit credentials entirely
    Omit,
    /// Include credentials only for same-origin requests
    SameOrigin,
    /// Always include credentials, even for cross-origin requests
    Include,
}

/// Cache modes for cache control
///
/// Controls how the request interacts with HTTP caching.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CacheMode {
    /// Default cache behavior - use cached responses when fresh
    Default,
    /// No-store mode - don't use cache and don't store response
    NoStore,
    /// Reload mode - always fetch from origin, bypassing cache
    Reload,
    /// No-cache mode - validate with origin before using cached response
    NoCache,
    /// Force-cache mode - use cached response even if stale
    ForceCache,
    /// Only-if-cached mode - use cache only, fail if not cached
    OnlyIfCached,
}

/// Redirect handling mode
///
/// Controls how HTTP redirects are processed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RedirectMode {
    /// Follow redirects automatically
    Follow,
    /// Treat redirects as errors
    Error,
    /// Don't follow redirects, return redirect response
    Manual,
}

/// Referrer policy for requests
///
/// Controls what referrer information is sent with requests.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ReferrerPolicy {
    /// Never send referrer
    NoReferrer,
    /// Send referrer except when downgrading from HTTPS to HTTP
    NoReferrerWhenDowngrade,
    /// Send only the origin (no path)
    Origin,
    /// Send origin when cross-origin, full URL when same-origin
    OriginWhenCrossOrigin,
    /// Send referrer only for same-origin requests
    SameOrigin,
    /// Send origin only when not downgrading
    StrictOrigin,
    /// Combination of StrictOrigin and OriginWhenCrossOrigin
    StrictOriginWhenCrossOrigin,
    /// Always send full referrer (unsafe)
    UnsafeUrl,
}

/// Request priority levels
///
/// Controls the priority of the request in the network scheduler.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RequestPriority {
    /// High priority request
    High,
    /// Low priority request
    Low,
    /// Automatic priority (default)
    Auto,
}

/// Response type classification
///
/// Classifies the type of response based on CORS and error status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResponseType {
    /// Basic response - same-origin, non-error
    Basic,
    /// CORS response - cross-origin with proper CORS headers
    Cors,
    /// Error response - network error occurred
    Error,
    /// Opaque response - cross-origin no-cors request
    Opaque,
    /// Opaque redirect response - redirect in no-cors mode
    OpaqueRedirect,
}

/// Request body content types
///
/// Represents different formats a request body can take.
pub enum RequestBody {
    /// Raw bytes
    Bytes(Vec<u8>),
    /// Text string
    Text(String),
    /// Form data (multipart/form-data)
    FormData(FormData),
    /// Streaming body
    Stream(BodyStream),
}

impl std::fmt::Debug for RequestBody {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RequestBody::Bytes(bytes) => f
                .debug_tuple("Bytes")
                .field(&format!("{} bytes", bytes.len()))
                .finish(),
            RequestBody::Text(text) => f.debug_tuple("Text").field(text).finish(),
            RequestBody::FormData(data) => f.debug_tuple("FormData").field(data).finish(),
            RequestBody::Stream(_) => f.debug_tuple("Stream").field(&"<stream>").finish(),
        }
    }
}

impl Clone for RequestBody {
    fn clone(&self) -> Self {
        match self {
            RequestBody::Bytes(bytes) => RequestBody::Bytes(bytes.clone()),
            RequestBody::Text(text) => RequestBody::Text(text.clone()),
            RequestBody::FormData(data) => RequestBody::FormData(data.clone()),
            // Streams cannot be cloned - panic if attempted
            RequestBody::Stream(_) => panic!("Cannot clone streaming request body"),
        }
    }
}

/// Response body content types
///
/// Represents different formats a response body can take.
pub enum ResponseBody {
    /// Raw bytes (fully loaded)
    Bytes(Vec<u8>),
    /// Streaming response body
    Stream(Box<dyn Stream<Item = Result<Bytes, NetworkError>> + Send + Sync + Unpin>),
    /// Empty response (no body)
    Empty,
}

impl std::fmt::Debug for ResponseBody {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResponseBody::Bytes(bytes) => f
                .debug_tuple("Bytes")
                .field(&format!("{} bytes", bytes.len()))
                .finish(),
            ResponseBody::Stream(_) => f.debug_tuple("Stream").field(&"<stream>").finish(),
            ResponseBody::Empty => f.debug_tuple("Empty").finish(),
        }
    }
}

/// Form data for multipart/form-data requests
///
/// Represents structured form data with fields and files.
#[derive(Debug, Clone)]
pub struct FormData {
    /// Form fields (name, value pairs)
    pub fields: Vec<(String, String)>,
    /// File uploads (field name, filename, content type, data)
    pub files: Vec<(String, String, String, Vec<u8>)>,
}

/// Body stream for streaming request bodies
///
/// Allows sending request body as a stream of chunks.
pub type BodyStream = Pin<Box<dyn Stream<Item = Result<Bytes, std::io::Error>> + Send + Sync>>;

/// Abort signal for cancelling requests
///
/// Allows requests to be cancelled mid-flight.
#[derive(Debug, Clone)]
pub struct AbortSignal {
    /// Whether the request has been aborted
    pub aborted: bool,
    /// Reason for abortion (if any)
    pub reason: Option<String>,
}

/// Window identifier for associating requests with browsing contexts
///
/// Links requests to specific browser windows/tabs.
pub type WindowId = u64;

/// Network error type
///
/// Represents errors that can occur during network operations.
#[derive(Debug, Clone)]
pub struct NetworkError {
    /// Error kind/category
    pub kind: NetworkErrorKind,
    /// Human-readable error message
    pub message: String,
}

/// Network error kinds
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkErrorKind {
    /// Connection failed
    ConnectionFailed,
    /// Timeout occurred
    Timeout,
    /// Invalid URL
    InvalidUrl,
    /// Invalid response
    InvalidResponse,
    /// Request aborted
    Aborted,
    /// Other error
    Other,
}

/// Resource timing information
///
/// Contains detailed timing metrics for a network resource fetch,
/// following the W3C Resource Timing specification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceTiming {
    /// Time when the fetch started (epoch time in milliseconds)
    pub start_time: f64,
    /// Time when redirect started
    pub redirect_start: f64,
    /// Time when redirect ended
    pub redirect_end: f64,
    /// Time when fetch actually started (after redirects)
    pub fetch_start: f64,
    /// Time when domain lookup started
    pub domain_lookup_start: f64,
    /// Time when domain lookup ended
    pub domain_lookup_end: f64,
    /// Time when connection started
    pub connect_start: f64,
    /// Time when connection established
    pub connect_end: f64,
    /// Time when secure connection (TLS) started
    pub secure_connection_start: f64,
    /// Time when request was sent
    pub request_start: f64,
    /// Time when response started arriving
    pub response_start: f64,
    /// Time when response fully received
    pub response_end: f64,
    /// Total transfer size in bytes (including headers)
    pub transfer_size: u64,
    /// Encoded body size in bytes (compressed)
    pub encoded_body_size: u64,
    /// Decoded body size in bytes (uncompressed)
    pub decoded_body_size: u64,
}

/// Network request structure
///
/// Represents a complete HTTP request with all metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkRequest {
    /// Target URL for the request
    pub url: Url,
    /// HTTP method
    pub method: HttpMethod,
    /// HTTP headers
    #[serde(skip)]
    pub headers: HeaderMap,
    /// Request body (if any)
    #[serde(skip)]
    pub body: Option<RequestBody>,
    /// CORS mode
    pub mode: RequestMode,
    /// Credentials handling mode
    pub credentials: CredentialsMode,
    /// Cache mode
    pub cache: CacheMode,
    /// Redirect handling mode
    pub redirect: RedirectMode,
    /// Referrer URL (if any)
    pub referrer: Option<String>,
    /// Referrer policy
    pub referrer_policy: ReferrerPolicy,
    /// Subresource integrity hash (if any)
    pub integrity: Option<String>,
    /// Keep connection alive after completion
    pub keepalive: bool,
    /// Abort signal for cancellation (if any)
    #[serde(skip)]
    pub signal: Option<AbortSignal>,
    /// Request priority
    pub priority: RequestPriority,
    /// Associated window ID (if any)
    pub window: Option<WindowId>,
}

/// Network response structure
///
/// Represents a complete HTTP response with all metadata.
#[derive(Debug)]
pub struct NetworkResponse {
    /// Final URL after redirects
    pub url: Url,
    /// HTTP status code
    pub status: u16,
    /// HTTP status text
    pub status_text: String,
    /// HTTP headers
    pub headers: HeaderMap,
    /// Response body
    pub body: ResponseBody,
    /// Whether the response was redirected
    pub redirected: bool,
    /// Response type classification
    pub type_: ResponseType,
    /// Resource timing information
    pub timing: ResourceTiming,
}

impl Default for ResourceTiming {
    fn default() -> Self {
        Self {
            start_time: 0.0,
            redirect_start: 0.0,
            redirect_end: 0.0,
            fetch_start: 0.0,
            domain_lookup_start: 0.0,
            domain_lookup_end: 0.0,
            connect_start: 0.0,
            connect_end: 0.0,
            secure_connection_start: 0.0,
            request_start: 0.0,
            response_start: 0.0,
            response_end: 0.0,
            transfer_size: 0,
            encoded_body_size: 0,
            decoded_body_size: 0,
        }
    }
}

impl FormData {
    /// Create a new empty FormData
    pub fn new() -> Self {
        Self {
            fields: Vec::new(),
            files: Vec::new(),
        }
    }

    /// Add a text field
    pub fn add_field(&mut self, name: String, value: String) {
        self.fields.push((name, value));
    }

    /// Add a file upload
    pub fn add_file(
        &mut self,
        field_name: String,
        filename: String,
        content_type: String,
        data: Vec<u8>,
    ) {
        self.files.push((field_name, filename, content_type, data));
    }
}

impl Default for FormData {
    fn default() -> Self {
        Self::new()
    }
}

impl AbortSignal {
    /// Create a new abort signal
    pub fn new() -> Self {
        Self {
            aborted: false,
            reason: None,
        }
    }

    /// Check if aborted
    pub fn is_aborted(&self) -> bool {
        self.aborted
    }

    /// Abort with a reason
    pub fn abort(&mut self, reason: String) {
        self.aborted = true;
        self.reason = Some(reason);
    }
}

impl Default for AbortSignal {
    fn default() -> Self {
        Self::new()
    }
}

impl NetworkError {
    /// Create a new network error
    pub fn new(kind: NetworkErrorKind, message: String) -> Self {
        Self { kind, message }
    }
}

impl std::fmt::Display for NetworkError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}: {}", self.kind, self.message)
    }
}

impl std::error::Error for NetworkError {}
