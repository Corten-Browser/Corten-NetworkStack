//! Tests for NetworkConfig structure
//!
//! Verifies configuration initialization and defaults.

use network_stack::NetworkConfig;

/// Test NetworkConfig default values
#[test]
fn test_network_config_default() {
    // Given/When: creating default config
    let config = NetworkConfig::default();

    // Then: should have sensible defaults
    assert!(config.http.is_some(), "Should have HTTP config");
    assert!(config.websocket.is_some(), "Should have WebSocket config");
    assert!(config.webrtc.is_some(), "Should have WebRTC config");
}

/// Test NetworkConfig can be customized
#[test]
fn test_network_config_customization() {
    // Given: a custom HTTP configuration
    let http_config = http1_protocol::Http1Config {
        pool_size: 10,
        idle_timeout: std::time::Duration::from_secs(30),
        max_connections_per_host: 4,
        enable_keepalive: true,
        enable_pipelining: false,
    };

    // When: creating NetworkConfig with custom values
    let config = NetworkConfig {
        http: Some(http_config),
        websocket: None,
        webrtc: None,
        cache: None,
        security: None,
        proxy: None,
        dns: None,
    };

    // Then: custom values should be preserved
    assert!(config.http.is_some(), "Should have custom HTTP config");
    assert!(config.websocket.is_none(), "Should not have WebSocket config");
}
