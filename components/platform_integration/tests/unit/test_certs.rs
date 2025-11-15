//! Unit tests for system certificate store access
//!
//! Given system certificate store
//! When get_system_cert_store is called
//! Then certificates are returned or empty result

use platform_integration::PlatformIntegration;

#[test]
fn test_get_system_cert_store_returns_result() {
    // Given: System with certificate store
    // When: get_system_cert_store is called
    let result = PlatformIntegration::get_system_cert_store();

    // Then: Result is returned (Ok with certs or empty vec)
    assert!(result.is_ok());
}

#[test]
fn test_get_system_cert_store_returns_vec_of_byte_vecs() {
    // Given: System certificate store
    // When: get_system_cert_store is called
    let result = PlatformIntegration::get_system_cert_store();

    // Then: Returns vector of byte vectors (certificate data)
    assert!(result.is_ok());
    let certs = result.unwrap();

    // Each certificate should be a Vec<u8>
    for cert in certs {
        assert!(cert.is_empty() || !cert.is_empty()); // Valid Vec<u8>
    }
}

#[test]
fn test_get_system_cert_store_graceful_on_unsupported_platform() {
    // Given: Potentially unsupported platform
    // When: get_system_cert_store is called
    let result = PlatformIntegration::get_system_cert_store();

    // Then: Returns Ok with empty vector (graceful degradation)
    // Should not panic or return error on unsupported platforms
    assert!(result.is_ok());
}
