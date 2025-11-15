//! FTP Protocol Implementation
//!
//! Basic FTP client supporting:
//! - Control connection (port 21)
//! - Passive mode (PASV)
//! - Active mode (PORT)
//! - Authentication (USER, PASS)
//! - File operations (LIST, RETR, STOR)

use std::time::Duration;

pub mod responses;
pub mod commands;
pub mod client;

pub use client::FtpClient;
pub use responses::FtpResponse;

/// FTP configuration
#[derive(Debug, Clone)]
pub struct FtpConfig {
    /// Connection timeout
    pub timeout: Duration,
    /// Use passive mode (default: true)
    pub passive_mode: bool,
}

impl Default for FtpConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(60),
            passive_mode: true,
        }
    }
}

/// FTP transfer mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FtpMode {
    /// Active mode (server connects to client)
    Active,
    /// Passive mode (client connects to server)
    Passive,
}
