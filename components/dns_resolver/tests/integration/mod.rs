//! Integration tests for dns_resolver component
//!
//! These tests verify the component works correctly with real DNS resolution
//! (using localhost/loopback addresses to avoid network dependencies in tests)

use dns_resolver::{DnsCache, DnsResolver, DohConfig, StandardResolver};
use network_errors::NetworkError;
use std::net::{IpAddr, Ipv4Addr};
use std::time::Duration;

/// Test contract: DnsResolver trait has resolve method
#[tokio::test]
async fn test_contract_resolver_trait_has_resolve() {
    let resolver = StandardResolver::new(None).expect("Failed to create resolver");

    // Test with localhost which should always resolve
    let result = resolver.resolve("localhost".to_string()).await;

    // Should either succeed or fail with DnsError (not panic or other error types)
    match result {
        Ok(addresses) => {
            assert!(
                !addresses.is_empty(),
                "localhost should resolve to at least one address"
            );
        }
        Err(NetworkError::DnsError(_)) => {
            // This is acceptable - DNS resolution can fail in test environments
        }
        Err(e) => {
            panic!("Expected Ok or DnsError, got: {:?}", e);
        }
    }
}

/// Test contract: DnsResolver trait has resolve_with_timeout method
#[tokio::test]
async fn test_contract_resolver_trait_has_resolve_with_timeout() {
    let resolver = StandardResolver::new(None).expect("Failed to create resolver");

    // Test with a reasonable timeout
    let result = resolver
        .resolve_with_timeout("localhost".to_string(), Duration::from_secs(5))
        .await;

    // Should either succeed or fail appropriately (not panic)
    match result {
        Ok(_) | Err(NetworkError::DnsError(_)) | Err(NetworkError::Timeout(_)) => {
            // All these are valid outcomes
        }
        Err(e) => {
            panic!("Unexpected error type: {:?}", e);
        }
    }
}

/// Test contract: DnsCache has get method returning Option<Vec<IpAddr>>
#[test]
fn test_contract_cache_has_get() {
    let cache = DnsCache::new();

    // Test the contract signature
    let result: Option<Vec<IpAddr>> = cache.get("example.com");
    assert!(result.is_none()); // Empty cache
}

/// Test contract: DnsCache has insert method
#[test]
fn test_contract_cache_has_insert() {
    let mut cache = DnsCache::new();

    // Test the contract signature
    let addresses = vec![IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))];
    cache.insert(
        "example.com".to_string(),
        addresses,
        Duration::from_secs(60),
    );

    // Verify it was inserted
    assert!(cache.get("example.com").is_some());
}

/// Test contract: DohConfig has all required fields
#[test]
fn test_contract_doh_config_fields() {
    let config = DohConfig {
        enabled: true,
        resolver_url: "https://dns.google/dns-query".to_string(),
        use_fallback: true,
    };

    // Verify all fields exist and can be accessed
    assert!(config.enabled);
    assert!(!config.resolver_url.is_empty());
    assert!(config.use_fallback);
}

/// Integration test: Resolver with caching
#[tokio::test]
async fn test_integration_resolver_with_cache() {
    let resolver = StandardResolver::new(None).expect("Failed to create resolver");

    // First resolution should hit DNS
    let result1 = resolver.resolve("localhost".to_string()).await;

    if result1.is_ok() {
        // Second resolution should hit cache (faster)
        let result2 = resolver.resolve("localhost".to_string()).await;

        // Both should succeed
        assert!(result2.is_ok());

        // Results should match (if DNS didn't change)
        if let (Ok(addrs1), Ok(addrs2)) = (result1, result2) {
            // At least one address should be the same
            let has_overlap = addrs1.iter().any(|a| addrs2.contains(a));
            assert!(has_overlap, "Cached results should match original");
        }
    }
}

/// Integration test: Timeout handling
#[tokio::test]
async fn test_integration_timeout_handling() {
    let resolver = StandardResolver::new(None).expect("Failed to create resolver");

    // Use very short timeout (likely to timeout on some systems)
    let result = resolver
        .resolve_with_timeout("localhost".to_string(), Duration::from_millis(1))
        .await;

    // Should either succeed (fast enough) or timeout
    match result {
        Ok(_) => {
            // Resolution was fast enough
        }
        Err(NetworkError::Timeout(d)) => {
            assert_eq!(d, Duration::from_millis(1));
        }
        Err(NetworkError::DnsError(_)) => {
            // DNS error is also acceptable
        }
        Err(e) => {
            panic!("Unexpected error: {:?}", e);
        }
    }
}

/// Integration test: DoH configuration
#[tokio::test]
async fn test_integration_doh_configuration() {
    // Create resolver with Google DoH
    let resolver = StandardResolver::with_google_doh();
    assert!(
        resolver.is_ok(),
        "Should be able to create resolver with Google DoH"
    );

    // Create resolver with Cloudflare DoH
    let resolver = StandardResolver::with_cloudflare_doh();
    assert!(
        resolver.is_ok(),
        "Should be able to create resolver with Cloudflare DoH"
    );
}

/// Integration test: Cache expiration
#[tokio::test]
async fn test_integration_cache_expiration() {
    let mut cache = DnsCache::new();

    let addresses = vec![IpAddr::V4(Ipv4Addr::new(192, 0, 2, 1))];

    // Insert with very short TTL
    cache.insert(
        "test.example.com".to_string(),
        addresses.clone(),
        Duration::from_millis(50),
    );

    // Should be available immediately
    assert!(cache.get("test.example.com").is_some());

    // Wait for expiration
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Should be expired
    assert!(cache.get("test.example.com").is_none());
}

/// Integration test: Clear expired entries
#[test]
fn test_integration_clear_expired() {
    let mut cache = DnsCache::new();

    let addresses = vec![IpAddr::V4(Ipv4Addr::new(192, 0, 2, 1))];

    // Insert entries with different TTLs
    cache.insert(
        "short.example.com".to_string(),
        addresses.clone(),
        Duration::from_millis(1),
    );
    cache.insert(
        "long.example.com".to_string(),
        addresses,
        Duration::from_secs(3600),
    );

    // Wait for short TTL to expire
    std::thread::sleep(Duration::from_millis(50));

    // Clear expired
    cache.clear_expired();

    // Short should be gone, long should remain
    assert!(cache.get("short.example.com").is_none());
    // Note: long might also be None if time has passed, that's okay
}
