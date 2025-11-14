//! Unit tests for network_errors component

use std::io;
use std::time::Duration;

// Import the types we'll test (these don't exist yet, so tests will fail)
// This demonstrates TDD - tests first!
use network_errors::{NetworkError, NetworkResult};

#[cfg(test)]
mod error_creation_tests {
    use super::*;

    #[test]
    fn test_connection_failed_error() {
        let error = NetworkError::ConnectionFailed("Host unreachable".to_string());
        assert!(matches!(error, NetworkError::ConnectionFailed(_)));
    }

    #[test]
    fn test_dns_error() {
        let error = NetworkError::DnsError("Name not resolved".to_string());
        assert!(matches!(error, NetworkError::DnsError(_)));
    }

    #[test]
    fn test_tls_error() {
        let error = NetworkError::TlsError("Certificate validation failed".to_string());
        assert!(matches!(error, NetworkError::TlsError(_)));
    }

    #[test]
    fn test_protocol_error() {
        let error = NetworkError::ProtocolError("HTTP/2 protocol violation".to_string());
        assert!(matches!(error, NetworkError::ProtocolError(_)));
    }

    #[test]
    fn test_timeout_error() {
        let duration = Duration::from_secs(30);
        let error = NetworkError::Timeout(duration);
        assert!(matches!(error, NetworkError::Timeout(_)));
    }

    #[test]
    fn test_aborted_error() {
        let error = NetworkError::Aborted;
        assert!(matches!(error, NetworkError::Aborted));
    }

    #[test]
    fn test_invalid_url_error() {
        let error = NetworkError::InvalidUrl("malformed://url".to_string());
        assert!(matches!(error, NetworkError::InvalidUrl(_)));
    }

    #[test]
    fn test_too_many_redirects_error() {
        let error = NetworkError::TooManyRedirects;
        assert!(matches!(error, NetworkError::TooManyRedirects));
    }

    #[test]
    fn test_cache_error() {
        let error = NetworkError::CacheError("Cache write failed".to_string());
        assert!(matches!(error, NetworkError::CacheError(_)));
    }

    #[test]
    fn test_proxy_error() {
        let error = NetworkError::ProxyError("Proxy authentication required".to_string());
        assert!(matches!(error, NetworkError::ProxyError(_)));
    }

    #[test]
    fn test_cors_error() {
        let error = NetworkError::CorsError("Origin not allowed".to_string());
        assert!(matches!(error, NetworkError::CorsError(_)));
    }

    #[test]
    fn test_mixed_content_error() {
        let error = NetworkError::MixedContent;
        assert!(matches!(error, NetworkError::MixedContent));
    }

    #[test]
    fn test_certificate_error() {
        let error = NetworkError::CertificateError("Certificate expired".to_string());
        assert!(matches!(error, NetworkError::CertificateError(_)));
    }

    #[test]
    fn test_websocket_error() {
        let error = NetworkError::WebSocketError("Connection closed unexpectedly".to_string());
        assert!(matches!(error, NetworkError::WebSocketError(_)));
    }

    #[test]
    fn test_webrtc_error() {
        let error = NetworkError::WebRtcError("ICE connection failed".to_string());
        assert!(matches!(error, NetworkError::WebRtcError(_)));
    }

    #[test]
    fn test_io_error() {
        let io_error = io::Error::new(io::ErrorKind::ConnectionRefused, "connection refused");
        let error = NetworkError::Io(io_error);
        assert!(matches!(error, NetworkError::Io(_)));
    }

    #[test]
    fn test_other_error() {
        let error = NetworkError::Other("Unexpected error".to_string());
        assert!(matches!(error, NetworkError::Other(_)));
    }
}

#[cfg(test)]
mod display_tests {
    use super::*;

    #[test]
    fn test_connection_failed_display() {
        let error = NetworkError::ConnectionFailed("Host unreachable".to_string());
        let display = format!("{}", error);
        assert!(display.contains("Connection failed"));
        assert!(display.contains("Host unreachable"));
    }

    #[test]
    fn test_dns_error_display() {
        let error = NetworkError::DnsError("Name not resolved".to_string());
        let display = format!("{}", error);
        assert!(display.contains("DNS resolution failed"));
        assert!(display.contains("Name not resolved"));
    }

    #[test]
    fn test_tls_error_display() {
        let error = NetworkError::TlsError("Certificate validation failed".to_string());
        let display = format!("{}", error);
        assert!(display.contains("TLS error"));
        assert!(display.contains("Certificate validation failed"));
    }

    #[test]
    fn test_protocol_error_display() {
        let error = NetworkError::ProtocolError("HTTP/2 protocol violation".to_string());
        let display = format!("{}", error);
        assert!(display.contains("Protocol error"));
        assert!(display.contains("HTTP/2 protocol violation"));
    }

    #[test]
    fn test_timeout_display() {
        let duration = Duration::from_secs(30);
        let error = NetworkError::Timeout(duration);
        let display = format!("{}", error);
        assert!(display.contains("Timeout"));
        assert!(display.contains("30"));
    }

    #[test]
    fn test_aborted_display() {
        let error = NetworkError::Aborted;
        let display = format!("{}", error);
        assert!(display.contains("aborted"));
    }

    #[test]
    fn test_invalid_url_display() {
        let error = NetworkError::InvalidUrl("malformed://url".to_string());
        let display = format!("{}", error);
        assert!(display.contains("Invalid URL"));
        assert!(display.contains("malformed://url"));
    }

    #[test]
    fn test_too_many_redirects_display() {
        let error = NetworkError::TooManyRedirects;
        let display = format!("{}", error);
        assert!(display.contains("Too many redirects"));
    }

    #[test]
    fn test_cache_error_display() {
        let error = NetworkError::CacheError("Cache write failed".to_string());
        let display = format!("{}", error);
        assert!(display.contains("Cache error"));
        assert!(display.contains("Cache write failed"));
    }

    #[test]
    fn test_proxy_error_display() {
        let error = NetworkError::ProxyError("Proxy authentication required".to_string());
        let display = format!("{}", error);
        assert!(display.contains("Proxy error"));
        assert!(display.contains("Proxy authentication required"));
    }

    #[test]
    fn test_cors_error_display() {
        let error = NetworkError::CorsError("Origin not allowed".to_string());
        let display = format!("{}", error);
        assert!(display.contains("CORS"));
        assert!(display.contains("Origin not allowed"));
    }

    #[test]
    fn test_mixed_content_display() {
        let error = NetworkError::MixedContent;
        let display = format!("{}", error);
        assert!(display.contains("Mixed content"));
    }

    #[test]
    fn test_certificate_error_display() {
        let error = NetworkError::CertificateError("Certificate expired".to_string());
        let display = format!("{}", error);
        assert!(display.contains("Certificate"));
        assert!(display.contains("Certificate expired"));
    }

    #[test]
    fn test_websocket_error_display() {
        let error = NetworkError::WebSocketError("Connection closed".to_string());
        let display = format!("{}", error);
        assert!(display.contains("WebSocket"));
        assert!(display.contains("Connection closed"));
    }

    #[test]
    fn test_webrtc_error_display() {
        let error = NetworkError::WebRtcError("ICE failed".to_string());
        let display = format!("{}", error);
        assert!(display.contains("WebRTC"));
        assert!(display.contains("ICE failed"));
    }

    #[test]
    fn test_io_error_display() {
        let io_error = io::Error::new(io::ErrorKind::ConnectionRefused, "connection refused");
        let error = NetworkError::Io(io_error);
        let display = format!("{}", error);
        assert!(display.contains("IO error"));
        assert!(display.contains("connection refused"));
    }

    #[test]
    fn test_other_error_display() {
        let error = NetworkError::Other("Unexpected error".to_string());
        let display = format!("{}", error);
        assert!(display.contains("Other error"));
        assert!(display.contains("Unexpected error"));
    }
}

#[cfg(test)]
mod conversion_tests {
    use super::*;

    #[test]
    fn test_from_io_error() {
        let io_error = io::Error::new(io::ErrorKind::ConnectionRefused, "connection refused");
        let network_error: NetworkError = io_error.into();
        assert!(matches!(network_error, NetworkError::Io(_)));
    }

    #[test]
    fn test_from_io_error_preserves_message() {
        let io_error = io::Error::new(io::ErrorKind::PermissionDenied, "permission denied");
        let network_error: NetworkError = io_error.into();
        let display = format!("{}", network_error);
        assert!(display.contains("permission denied"));
    }
}

#[cfg(test)]
mod error_trait_tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn test_implements_error_trait() {
        let error = NetworkError::ConnectionFailed("test".to_string());
        // This tests that NetworkError implements std::error::Error
        let _: &dyn Error = &error;
    }

    #[test]
    fn test_error_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<NetworkError>();
    }

    #[test]
    fn test_debug_format() {
        let error = NetworkError::ConnectionFailed("test".to_string());
        let debug = format!("{:?}", error);
        assert!(debug.contains("ConnectionFailed"));
    }
}

#[cfg(test)]
mod result_type_tests {
    use super::*;

    #[test]
    fn test_network_result_ok() {
        let result: NetworkResult<i32> = Ok(42);
        assert!(result.is_ok());
        if let Ok(value) = result {
            assert_eq!(value, 42);
        }
    }

    #[test]
    fn test_network_result_err() {
        let result: NetworkResult<i32> = Err(NetworkError::Aborted);
        assert!(result.is_err());
    }

    #[test]
    fn test_network_result_with_function() {
        fn do_something() -> NetworkResult<String> {
            Ok("success".to_string())
        }

        let result = do_something();
        assert!(result.is_ok());
    }

    #[test]
    fn test_network_result_with_error() {
        fn do_something_that_fails() -> NetworkResult<String> {
            Err(NetworkError::Timeout(Duration::from_secs(10)))
        }

        let result = do_something_that_fails();
        assert!(result.is_err());
        if let Err(NetworkError::Timeout(d)) = result {
            assert_eq!(d, Duration::from_secs(10));
        } else {
            panic!("Expected Timeout error");
        }
    }

    #[test]
    fn test_network_result_question_mark_operator() {
        fn inner() -> NetworkResult<i32> {
            Err(NetworkError::ConnectionFailed("test".to_string()))
        }

        fn outer() -> NetworkResult<String> {
            let _value = inner()?; // Should propagate error
            Ok("never reached".to_string())
        }

        let result = outer();
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod contract_verification_tests {
    use super::*;

    /// Verify NetworkError enum exists and has all required variants
    #[test]
    fn test_all_contract_variants_exist() {
        // From contracts/network_errors.yaml
        // Verify each variant exists and can be constructed
        let _connection_failed = NetworkError::ConnectionFailed("test".to_string());
        let _dns_error = NetworkError::DnsError("test".to_string());
        let _tls_error = NetworkError::TlsError("test".to_string());
        let _protocol_error = NetworkError::ProtocolError("test".to_string());
        let _timeout = NetworkError::Timeout(Duration::from_secs(1));
        let _aborted = NetworkError::Aborted;
        let _invalid_url = NetworkError::InvalidUrl("test".to_string());
        let _too_many_redirects = NetworkError::TooManyRedirects;
        let _cache_error = NetworkError::CacheError("test".to_string());
        let _proxy_error = NetworkError::ProxyError("test".to_string());
        let _cors_error = NetworkError::CorsError("test".to_string());
        let _mixed_content = NetworkError::MixedContent;
        let _certificate_error = NetworkError::CertificateError("test".to_string());
        let _websocket_error = NetworkError::WebSocketError("test".to_string());
        let _webrtc_error = NetworkError::WebRtcError("test".to_string());
        let _io = NetworkError::Io(io::Error::other("test"));
        let _other = NetworkError::Other("test".to_string());
    }

    /// Verify NetworkResult type alias exists
    #[test]
    fn test_network_result_type_alias() {
        let _ok: NetworkResult<()> = Ok(());
        let _err: NetworkResult<()> = Err(NetworkError::Aborted);
    }
}
