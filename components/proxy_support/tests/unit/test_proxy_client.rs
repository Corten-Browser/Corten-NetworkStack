use proxy_support::{ProxyAuth, ProxyClient, ProxyConfig};

#[test]
fn test_proxy_client_new_with_none() {
    let config = ProxyConfig::None;
    let client = ProxyClient::new(config);

    // Should create successfully
    assert!(matches!(client.config(), ProxyConfig::None));
}

#[test]
fn test_proxy_client_new_with_http() {
    let config = ProxyConfig::Http {
        host: "proxy.example.com".to_string(),
        port: 8080,
        auth: None,
    };
    let client = ProxyClient::new(config);

    if let ProxyConfig::Http { host, port, .. } = client.config() {
        assert_eq!(host, "proxy.example.com");
        assert_eq!(*port, 8080);
    } else {
        panic!("Expected Http config");
    }
}

#[test]
fn test_proxy_client_new_with_socks5() {
    let config = ProxyConfig::Socks5 {
        host: "socks.example.com".to_string(),
        port: 1080,
        auth: Some(ProxyAuth::Basic {
            username: "user".to_string(),
            password: "pass".to_string(),
        }),
    };
    let client = ProxyClient::new(config);

    if let ProxyConfig::Socks5 { host, port, auth } = client.config() {
        assert_eq!(host, "socks.example.com");
        assert_eq!(*port, 1080);
        assert!(auth.is_some());
    } else {
        panic!("Expected Socks5 config");
    }
}

// Integration tests for connect() will be in tests/integration/
// as they require actual network mocking or real connections
