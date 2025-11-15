//! Unit tests for proxy configuration detection
//!
//! Given proxy environment variables are set
//! When get_system_proxy_config is called
//! Then correct proxy configuration is returned

use platform_integration::{PlatformIntegration, SystemProxyConfig};
use std::env;

#[test]
fn test_get_system_proxy_config_returns_http_proxy_from_env() {
    // Given: HTTP_PROXY environment variable is set
    env::set_var("HTTP_PROXY", "http://proxy.example.com:8080");
    env::remove_var("HTTPS_PROXY");
    env::remove_var("NO_PROXY");

    // When: get_system_proxy_config is called
    let result = PlatformIntegration::get_system_proxy_config();

    // Then: proxy configuration contains HTTP proxy
    assert!(result.is_ok());
    let config = result.unwrap();
    assert!(config.enabled);
    assert_eq!(config.http_proxy, Some("http://proxy.example.com:8080".to_string()));
    assert_eq!(config.https_proxy, None);

    // Cleanup
    env::remove_var("HTTP_PROXY");
}

#[test]
fn test_get_system_proxy_config_returns_https_proxy_from_env() {
    // Given: HTTPS_PROXY environment variable is set
    env::remove_var("HTTP_PROXY");
    env::set_var("HTTPS_PROXY", "https://secure-proxy.example.com:443");
    env::remove_var("NO_PROXY");

    // When: get_system_proxy_config is called
    let result = PlatformIntegration::get_system_proxy_config();

    // Then: proxy configuration contains HTTPS proxy
    assert!(result.is_ok());
    let config = result.unwrap();
    assert!(config.enabled);
    assert_eq!(config.http_proxy, None);
    assert_eq!(config.https_proxy, Some("https://secure-proxy.example.com:443".to_string()));

    // Cleanup
    env::remove_var("HTTPS_PROXY");
}

#[test]
fn test_get_system_proxy_config_returns_both_proxies_from_env() {
    // Given: Both HTTP_PROXY and HTTPS_PROXY environment variables are set
    env::set_var("HTTP_PROXY", "http://proxy.example.com:8080");
    env::set_var("HTTPS_PROXY", "https://secure-proxy.example.com:443");
    env::remove_var("NO_PROXY");

    // When: get_system_proxy_config is called
    let result = PlatformIntegration::get_system_proxy_config();

    // Then: proxy configuration contains both proxies
    assert!(result.is_ok());
    let config = result.unwrap();
    assert!(config.enabled);
    assert_eq!(config.http_proxy, Some("http://proxy.example.com:8080".to_string()));
    assert_eq!(config.https_proxy, Some("https://secure-proxy.example.com:443".to_string()));

    // Cleanup
    env::remove_var("HTTP_PROXY");
    env::remove_var("HTTPS_PROXY");
}

#[test]
fn test_get_system_proxy_config_parses_no_proxy_single_domain() {
    // Given: NO_PROXY environment variable is set with a single domain
    env::remove_var("HTTP_PROXY");
    env::remove_var("HTTPS_PROXY");
    env::set_var("NO_PROXY", "localhost");

    // When: get_system_proxy_config is called
    let result = PlatformIntegration::get_system_proxy_config();

    // Then: proxy configuration contains no_proxy list
    assert!(result.is_ok());
    let config = result.unwrap();
    assert_eq!(config.no_proxy, vec!["localhost".to_string()]);

    // Cleanup
    env::remove_var("NO_PROXY");
}

#[test]
fn test_get_system_proxy_config_parses_no_proxy_multiple_domains() {
    // Given: NO_PROXY environment variable is set with comma-separated domains
    env::remove_var("HTTP_PROXY");
    env::remove_var("HTTPS_PROXY");
    env::set_var("NO_PROXY", "localhost,127.0.0.1,.example.com");

    // When: get_system_proxy_config is called
    let result = PlatformIntegration::get_system_proxy_config();

    // Then: proxy configuration contains parsed no_proxy list
    assert!(result.is_ok());
    let config = result.unwrap();
    assert_eq!(config.no_proxy, vec![
        "localhost".to_string(),
        "127.0.0.1".to_string(),
        ".example.com".to_string()
    ]);

    // Cleanup
    env::remove_var("NO_PROXY");
}

#[test]
fn test_get_system_proxy_config_handles_no_proxy_with_spaces() {
    // Given: NO_PROXY environment variable has spaces around commas
    env::remove_var("HTTP_PROXY");
    env::remove_var("HTTPS_PROXY");
    env::set_var("NO_PROXY", "localhost, 127.0.0.1 , .example.com");

    // When: get_system_proxy_config is called
    let result = PlatformIntegration::get_system_proxy_config();

    // Then: proxy configuration trims whitespace from entries
    assert!(result.is_ok());
    let config = result.unwrap();
    assert_eq!(config.no_proxy, vec![
        "localhost".to_string(),
        "127.0.0.1".to_string(),
        ".example.com".to_string()
    ]);

    // Cleanup
    env::remove_var("NO_PROXY");
}

#[test]
fn test_get_system_proxy_config_returns_disabled_when_no_env_vars() {
    // Given: No proxy environment variables are set
    env::remove_var("HTTP_PROXY");
    env::remove_var("HTTPS_PROXY");
    env::remove_var("NO_PROXY");

    // When: get_system_proxy_config is called
    let result = PlatformIntegration::get_system_proxy_config();

    // Then: proxy configuration is disabled
    assert!(result.is_ok());
    let config = result.unwrap();
    assert!(!config.enabled);
    assert_eq!(config.http_proxy, None);
    assert_eq!(config.https_proxy, None);
    assert_eq!(config.no_proxy, Vec::<String>::new());
}

#[test]
fn test_get_system_proxy_config_handles_empty_proxy_values() {
    // Given: Proxy environment variables are set but empty
    env::set_var("HTTP_PROXY", "");
    env::set_var("HTTPS_PROXY", "");
    env::remove_var("NO_PROXY");

    // When: get_system_proxy_config is called
    let result = PlatformIntegration::get_system_proxy_config();

    // Then: proxy configuration treats empty values as None
    assert!(result.is_ok());
    let config = result.unwrap();
    assert!(!config.enabled);
    assert_eq!(config.http_proxy, None);
    assert_eq!(config.https_proxy, None);

    // Cleanup
    env::remove_var("HTTP_PROXY");
    env::remove_var("HTTPS_PROXY");
}

#[test]
fn test_system_proxy_config_struct_creation() {
    // Given: SystemProxyConfig struct values
    let http_proxy = Some("http://proxy.example.com:8080".to_string());
    let https_proxy = Some("https://secure-proxy.example.com:443".to_string());
    let no_proxy = vec!["localhost".to_string(), "127.0.0.1".to_string()];

    // When: SystemProxyConfig is created
    let config = SystemProxyConfig {
        enabled: true,
        http_proxy: http_proxy.clone(),
        https_proxy: https_proxy.clone(),
        no_proxy: no_proxy.clone(),
    };

    // Then: all fields are accessible
    assert!(config.enabled);
    assert_eq!(config.http_proxy, http_proxy);
    assert_eq!(config.https_proxy, https_proxy);
    assert_eq!(config.no_proxy, no_proxy);
}
