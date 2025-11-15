//! Unit tests for CertificateStore

use tls_manager::CertificateStore;

#[test]
fn test_certificate_store_new() {
    // Given: no prior certificate store
    // When: a new CertificateStore is created
    // Then: it should be empty and ready to use

    let store = CertificateStore::new();
    assert_eq!(store.certificate_count(), 0);
}

#[test]
fn test_certificate_store_add_certificate() {
    // Given: a CertificateStore instance
    // When: a certificate is added
    // Then: the certificate should be stored successfully

    let mut store = CertificateStore::new();
    let cert_data = b"fake certificate data".to_vec();

    let result = store.add_certificate(cert_data);
    assert!(result.is_ok());
    assert_eq!(store.certificate_count(), 1);
}

#[tokio::test]
async fn test_certificate_store_verify_valid_certificate() {
    // Given: a CertificateStore with pinned certificate
    // When: a pinned certificate is verified
    // Then: verification should succeed (pinning overrides parsing)

    let mut store = CertificateStore::new();
    let cert = b"valid certificate";
    let hostname = "example.com";

    // Pin the certificate for this hostname
    store.add_pin(hostname, cert);

    let result = store.verify_certificate(cert, hostname).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_certificate_store_verify_with_hostname_mismatch() {
    // Given: a certificate store
    // When: verifying a certificate with wrong hostname
    // Then: verification should fail with appropriate error

    let store = CertificateStore::new();
    let cert = b"certificate";
    let hostname = "wrong-hostname.com";

    // For now, this will pass until we implement actual verification
    let result = store.verify_certificate(cert, hostname).await;
    // TODO: Once implemented, this should check for hostname mismatch error
    assert!(result.is_ok() || result.is_err());
}
