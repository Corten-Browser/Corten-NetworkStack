//! Integration tests for tls_manager
//!
//! Tests components working together in realistic scenarios

use std::time::Duration;
use tls_manager::{CertificateStore, HstsStore, TlsConfig};

#[test]
fn test_complete_tls_configuration() {
    // Given: Complete TLS setup requirements
    // When: Configuring TLS with ALPN, certificates, and HSTS
    // Then: All components should work together seamlessly

    // Configure TLS with ALPN protocols
    let config = TlsConfig::new().with_alpn_protocols(vec![
        b"h3".to_vec(),
        b"h2".to_vec(),
        b"http/1.1".to_vec(),
    ]);

    // Verify ALPN configuration
    assert_eq!(config.alpn_protocols().len(), 3);
    assert!(config.alpn_protocols().contains(&b"h2".to_vec()));

    // Set up certificate store
    let mut cert_store = CertificateStore::new();
    let cert = vec![0x30, 0x82]; // Mock DER certificate
    assert!(cert_store.add_certificate(cert).is_ok());

    // Set up HSTS store
    let mut hsts_store = HstsStore::new();
    hsts_store.add_hsts_entry(
        "secure-domain.com".to_string(),
        Duration::from_secs(31536000),
        true,
    );

    // Verify HSTS enforcement
    assert!(hsts_store.is_hsts_enabled("secure-domain.com"));
    assert!(hsts_store.is_hsts_enabled("subdomain.secure-domain.com"));
}

#[tokio::test]
async fn test_certificate_validation_workflow() {
    // Given: A certificate store with certificate pinning
    // When: Verifying pinned certificates for different domains
    // Then: Validation should succeed for pinned certificates

    let mut store = CertificateStore::new();

    // Create certificates (using simple data since we're testing pinning)
    let cert1 = b"certificate for domain1.com".to_vec();
    let cert2 = b"certificate for domain2.com".to_vec();

    assert!(store.add_certificate(cert1.clone()).is_ok());
    assert!(store.add_certificate(cert2.clone()).is_ok());
    assert_eq!(store.certificate_count(), 2);

    // Pin certificates for their respective domains
    // Pinning allows these non-standard certificates to validate
    store.add_pin("domain1.com", &cert1);
    store.add_pin("domain2.com", &cert2);

    // Verify pinned certificates - should succeed
    let result1 = store.verify_certificate(&cert1, "domain1.com").await;
    assert!(result1.is_ok(), "Pinned cert1 should validate");

    let result2 = store.verify_certificate(&cert2, "domain2.com").await;
    assert!(result2.is_ok(), "Pinned cert2 should validate");

    // Verify that wrong certificate fails (cert1 for domain2)
    let result_wrong = store.verify_certificate(&cert1, "domain2.com").await;
    assert!(result_wrong.is_err(), "Wrong pinned cert should fail");
}

#[test]
fn test_hsts_with_multiple_domains() {
    // Given: Multiple domains with different HSTS policies
    // When: Checking HSTS status for various domains
    // Then: Policies should be enforced correctly

    let mut store = HstsStore::new();

    // Domain with subdomain inclusion
    store.add_hsts_entry(
        "secure.com".to_string(),
        Duration::from_secs(31536000),
        true,
    );

    // Domain without subdomain inclusion
    store.add_hsts_entry(
        "partial.com".to_string(),
        Duration::from_secs(31536000),
        false,
    );

    // Verify HSTS for secure.com (with subdomains)
    assert!(store.is_hsts_enabled("secure.com"));
    assert!(store.is_hsts_enabled("www.secure.com"));
    assert!(store.is_hsts_enabled("api.secure.com"));

    // Verify HSTS for partial.com (without subdomains)
    assert!(store.is_hsts_enabled("partial.com"));
    assert!(!store.is_hsts_enabled("www.partial.com"));

    // Verify non-HSTS domain
    assert!(!store.is_hsts_enabled("insecure.com"));
}
