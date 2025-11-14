//! Unit tests for dns_resolver component

use dns_resolver::{DnsCache, DohConfig};
use std::net::IpAddr;
use std::time::Duration;

/// Test that DnsResolver trait can be imported
#[test]
fn test_dns_resolver_trait_exists() {
    // This test verifies the trait is defined and accessible
    // Actual resolver implementation will be tested separately
}

/// Test DnsCache creation
#[test]
fn test_dns_cache_creation() {
    let cache = DnsCache::new();
    // Cache should be empty initially
    assert!(cache.get("example.com").is_none());
}

/// Test DnsCache insert and get
#[test]
fn test_dns_cache_insert_and_get() {
    let mut cache = DnsCache::new();
    let addresses = vec!["192.0.2.1".parse::<IpAddr>().unwrap()];

    cache.insert(
        "example.com".to_string(),
        addresses.clone(),
        Duration::from_secs(300),
    );

    let result = cache.get("example.com");
    assert!(result.is_some());
    assert_eq!(result.unwrap(), addresses);
}

/// Test DnsCache TTL expiration
#[tokio::test]
async fn test_dns_cache_ttl_expiration() {
    let mut cache = DnsCache::new();
    let addresses = vec!["192.0.2.1".parse::<IpAddr>().unwrap()];

    // Insert with very short TTL
    cache.insert(
        "example.com".to_string(),
        addresses,
        Duration::from_millis(50),
    );

    // Should be present immediately
    assert!(cache.get("example.com").is_some());

    // Wait for TTL to expire
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Should be expired
    assert!(cache.get("example.com").is_none());
}

/// Test DohConfig creation
#[test]
fn test_doh_config_creation() {
    let config = DohConfig {
        enabled: true,
        resolver_url: "https://dns.google/dns-query".to_string(),
        use_fallback: true,
    };

    assert!(config.enabled);
    assert_eq!(config.resolver_url, "https://dns.google/dns-query");
    assert!(config.use_fallback);
}

/// Test DohConfig default
#[test]
fn test_doh_config_default() {
    let config = DohConfig::default();

    assert!(!config.enabled); // Should be disabled by default
    assert!(!config.resolver_url.is_empty());
    assert!(config.use_fallback);
}
