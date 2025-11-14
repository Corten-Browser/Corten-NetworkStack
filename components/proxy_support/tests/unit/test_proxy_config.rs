use proxy_support::{ProxyAuth, ProxyConfig};

#[test]
fn test_proxy_config_none_variant() {
    let config = ProxyConfig::None;
    assert!(matches!(config, ProxyConfig::None));
}

#[test]
fn test_proxy_config_http_without_auth() {
    let config = ProxyConfig::Http {
        host: "proxy.example.com".to_string(),
        port: 8080,
        auth: None,
    };

    if let ProxyConfig::Http { host, port, auth } = config {
        assert_eq!(host, "proxy.example.com");
        assert_eq!(port, 8080);
        assert!(auth.is_none());
    } else {
        panic!("Expected Http variant");
    }
}

#[test]
fn test_proxy_config_http_with_auth() {
    let auth = ProxyAuth::Basic {
        username: "user".to_string(),
        password: "pass".to_string(),
    };

    let config = ProxyConfig::Http {
        host: "proxy.example.com".to_string(),
        port: 8080,
        auth: Some(auth),
    };

    if let ProxyConfig::Http {
        host,
        port,
        auth: Some(auth_inner),
    } = config
    {
        assert_eq!(host, "proxy.example.com");
        assert_eq!(port, 8080);

        if let ProxyAuth::Basic { username, password } = auth_inner {
            assert_eq!(username, "user");
            assert_eq!(password, "pass");
        } else {
            panic!("Expected Basic auth");
        }
    } else {
        panic!("Expected Http variant with auth");
    }
}

#[test]
fn test_proxy_config_socks5_without_auth() {
    let config = ProxyConfig::Socks5 {
        host: "socks.example.com".to_string(),
        port: 1080,
        auth: None,
    };

    if let ProxyConfig::Socks5 { host, port, auth } = config {
        assert_eq!(host, "socks.example.com");
        assert_eq!(port, 1080);
        assert!(auth.is_none());
    } else {
        panic!("Expected Socks5 variant");
    }
}

#[test]
fn test_proxy_config_socks5_with_auth() {
    let auth = ProxyAuth::Basic {
        username: "socks_user".to_string(),
        password: "socks_pass".to_string(),
    };

    let config = ProxyConfig::Socks5 {
        host: "socks.example.com".to_string(),
        port: 1080,
        auth: Some(auth),
    };

    if let ProxyConfig::Socks5 {
        host,
        port,
        auth: Some(auth_inner),
    } = config
    {
        assert_eq!(host, "socks.example.com");
        assert_eq!(port, 1080);

        if let ProxyAuth::Basic { username, password } = auth_inner {
            assert_eq!(username, "socks_user");
            assert_eq!(password, "socks_pass");
        } else {
            panic!("Expected Basic auth");
        }
    } else {
        panic!("Expected Socks5 variant with auth");
    }
}

#[test]
fn test_proxy_auth_basic() {
    let auth = ProxyAuth::Basic {
        username: "testuser".to_string(),
        password: "testpass".to_string(),
    };

    if let ProxyAuth::Basic { username, password } = auth {
        assert_eq!(username, "testuser");
        assert_eq!(password, "testpass");
    } else {
        panic!("Expected Basic auth variant");
    }
}
