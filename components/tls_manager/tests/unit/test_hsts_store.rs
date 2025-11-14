//! Unit tests for HstsStore

use std::time::Duration;
use tls_manager::HstsStore;

#[test]
fn test_hsts_store_new() {
    // Given: no prior HSTS store
    // When: a new HstsStore is created
    // Then: it should be empty

    let store = HstsStore::new();
    assert!(!store.is_hsts_enabled("example.com"));
}

#[test]
fn test_hsts_store_add_entry() {
    // Given: an HstsStore instance
    // When: an HSTS entry is added
    // Then: the domain should be marked as HSTS-enabled

    let mut store = HstsStore::new();
    store.add_hsts_entry(
        "example.com".to_string(),
        Duration::from_secs(31536000), // 1 year
        false,
    );

    assert!(store.is_hsts_enabled("example.com"));
}

#[test]
fn test_hsts_store_is_not_enabled_for_unknown_domain() {
    // Given: an HstsStore with entries
    // When: checking for a domain not in the store
    // Then: HSTS should not be enabled

    let mut store = HstsStore::new();
    store.add_hsts_entry(
        "example.com".to_string(),
        Duration::from_secs(31536000),
        false,
    );

    assert!(!store.is_hsts_enabled("other-domain.com"));
}

#[test]
fn test_hsts_store_subdomain_inclusion() {
    // Given: an HSTS entry with subdomain inclusion
    // When: checking a subdomain
    // Then: HSTS should be enabled if include_subdomains is true

    let mut store = HstsStore::new();
    store.add_hsts_entry(
        "example.com".to_string(),
        Duration::from_secs(31536000),
        true, // include subdomains
    );

    assert!(store.is_hsts_enabled("example.com"));
    assert!(store.is_hsts_enabled("sub.example.com"));
}

#[test]
fn test_hsts_store_subdomain_exclusion() {
    // Given: an HSTS entry without subdomain inclusion
    // When: checking a subdomain
    // Then: HSTS should not be enabled for subdomains

    let mut store = HstsStore::new();
    store.add_hsts_entry(
        "example.com".to_string(),
        Duration::from_secs(31536000),
        false, // do not include subdomains
    );

    assert!(store.is_hsts_enabled("example.com"));
    assert!(!store.is_hsts_enabled("sub.example.com"));
}
