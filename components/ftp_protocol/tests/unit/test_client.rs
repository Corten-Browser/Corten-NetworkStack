//! Unit tests for FTP client

use ftp_protocol::{FtpClient, FtpConfig, FtpMode};
use std::time::Duration;

#[test]
fn test_ftp_client_creation() {
    // Given an FTP config
    let config = FtpConfig {
        timeout: Duration::from_secs(30),
        passive_mode: true,
    };

    // When creating an FTP client
    let client = FtpClient::new(config);

    // Then it should be created successfully
    assert_eq!(client.config.timeout, Duration::from_secs(30));
    assert_eq!(client.config.passive_mode, true);
}

#[test]
fn test_ftp_mode_passive() {
    // Given passive mode
    let mode = FtpMode::Passive;

    // Then it should match
    assert!(matches!(mode, FtpMode::Passive));
}

#[test]
fn test_ftp_mode_active() {
    // Given active mode
    let mode = FtpMode::Active;

    // Then it should match
    assert!(matches!(mode, FtpMode::Active));
}

#[test]
fn test_default_config() {
    // When creating default config
    let config = FtpConfig::default();

    // Then it should have sensible defaults
    assert_eq!(config.timeout, Duration::from_secs(60));
    assert_eq!(config.passive_mode, true);
}
