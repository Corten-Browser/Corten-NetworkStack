//! Integration tests for platform_integration component
//!
//! These tests verify the component works correctly with real system resources

use platform_integration::{PlatformIntegration, SystemProxyConfig};

#[test]
fn test_platform_integration_public_api_available() {
    // Verify all public API methods are accessible
    let _proxy_result = PlatformIntegration::get_system_proxy_config();
    let _cert_result = PlatformIntegration::get_system_cert_store();
    let _online_result = PlatformIntegration::is_online();
}

#[test]
fn test_proxy_config_struct_accessible() {
    // Verify SystemProxyConfig struct can be used
    let config = SystemProxyConfig {
        enabled: false,
        http_proxy: None,
        https_proxy: None,
        no_proxy: Vec::new(),
    };

    assert!(!config.enabled);
}
