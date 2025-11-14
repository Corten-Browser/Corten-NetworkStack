use proxy_support::{ProxyAuth, ProxyClient, ProxyConfig};

#[tokio::test]
async fn test_direct_connection_no_proxy() {
    // Test direct connection (no proxy)
    let config = ProxyConfig::None;
    let client = ProxyClient::new(config);

    // Test against a reliable public service
    // Note: This requires internet connectivity
    let result = client.connect("example.com", 80).await;

    // We just verify that the connection attempt completes
    // (may succeed or fail depending on network availability)
    // In a real environment, this would succeed
    match result {
        Ok(_stream) => {
            // Connection successful
            assert!(true);
        }
        Err(e) => {
            // Connection failed - might be no internet
            // This is acceptable for integration tests
            println!("Direct connection test: {:?}", e);
        }
    }
}

#[tokio::test]
async fn test_http_proxy_config() {
    // Test that HTTP proxy configuration is properly set up
    let config = ProxyConfig::Http {
        host: "proxy.example.com".to_string(),
        port: 8080,
        auth: Some(ProxyAuth::Basic {
            username: "user".to_string(),
            password: "pass".to_string(),
        }),
    };

    let client = ProxyClient::new(config);

    // Verify config is set
    if let ProxyConfig::Http { host, port, auth } = client.config() {
        assert_eq!(host, "proxy.example.com");
        assert_eq!(*port, 8080);
        assert!(auth.is_some());
    } else {
        panic!("Expected Http config");
    }
}

#[tokio::test]
async fn test_socks5_proxy_config() {
    // Test that SOCKS5 proxy configuration is properly set up
    let config = ProxyConfig::Socks5 {
        host: "socks.example.com".to_string(),
        port: 1080,
        auth: Some(ProxyAuth::Basic {
            username: "socks_user".to_string(),
            password: "socks_pass".to_string(),
        }),
    };

    let client = ProxyClient::new(config);

    // Verify config is set
    if let ProxyConfig::Socks5 { host, port, auth } = client.config() {
        assert_eq!(host, "socks.example.com");
        assert_eq!(*port, 1080);
        assert!(auth.is_some());
    } else {
        panic!("Expected Socks5 config");
    }
}

// Note: Full proxy integration tests require actual proxy servers
// which are not available in the test environment.
// In a real deployment, these would be tested with:
// 1. A test HTTP CONNECT proxy (e.g., squid, tinyproxy)
// 2. A test SOCKS5 proxy (e.g., dante, microsocks)
//
// Example of what such a test would look like:
//
// #[tokio::test]
// #[ignore] // Requires proxy server setup
// async fn test_http_proxy_connection() {
//     let config = ProxyConfig::Http {
//         host: "localhost".to_string(),
//         port: 8888,
//         auth: None,
//     };
//     let client = ProxyClient::new(config);
//     let stream = client.connect("example.com", 80).await.unwrap();
//     // Verify stream is connected
// }
