//! Unit tests for TlsConfig

use tls_manager::TlsConfig;

#[test]
fn test_tls_config_new_creates_default_config() {
    // Given: no prior configuration
    // When: a new TlsConfig is created
    // Then: it should have safe default settings

    let config = TlsConfig::new();

    // Config should exist and be valid
    assert!(config.alpn_protocols().is_empty());
}

#[test]
fn test_tls_config_with_alpn_protocols() {
    // Given: a TlsConfig instance
    // When: ALPN protocols are added
    // Then: the config should store those protocols

    let protocols = vec![b"h3".to_vec(), b"h2".to_vec(), b"http/1.1".to_vec()];
    let config = TlsConfig::new().with_alpn_protocols(protocols.clone());

    assert_eq!(config.alpn_protocols(), &protocols);
}

#[test]
fn test_tls_config_builder_pattern() {
    // Given: a TlsConfig builder
    // When: multiple configurations are chained
    // Then: all configurations should be applied

    let protocols = vec![b"h2".to_vec()];
    let config = TlsConfig::new().with_alpn_protocols(protocols.clone());

    assert_eq!(config.alpn_protocols(), &protocols);
}

#[test]
fn test_tls_config_supports_http2_and_http3() {
    // Given: TLS configuration requirements
    // When: ALPN protocols for HTTP/2 and HTTP/3 are set
    // Then: both protocols should be available

    let protocols = vec![
        b"h3".to_vec(),       // HTTP/3
        b"h2".to_vec(),       // HTTP/2
        b"http/1.1".to_vec(), // HTTP/1.1
    ];
    let config = TlsConfig::new().with_alpn_protocols(protocols.clone());

    let alpn = config.alpn_protocols();
    assert!(alpn.contains(&b"h3".to_vec()));
    assert!(alpn.contains(&b"h2".to_vec()));
    assert!(alpn.contains(&b"http/1.1".to_vec()));
}
