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
    // Given: A certificate store with certificates
    // When: Verifying certificates for different domains
    // Then: Validation should work correctly

    let mut store = CertificateStore::new();

    // Add certificates
    let cert1 = b"certificate for domain1.com".to_vec();
    let cert2 = b"certificate for domain2.com".to_vec();

    assert!(store.add_certificate(cert1.clone()).is_ok());
    assert!(store.add_certificate(cert2.clone()).is_ok());
    assert_eq!(store.certificate_count(), 2);

    // Verify certificates
    let result = store.verify_certificate(&cert1, "domain1.com").await;
    assert!(result.is_ok());
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
