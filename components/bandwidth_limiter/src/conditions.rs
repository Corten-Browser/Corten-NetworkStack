//! Network condition presets for common network types

use std::time::Duration;

/// Network condition presets
///
/// Provides realistic network condition simulations for testing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkCondition {
    /// Offline - no network connectivity
    Offline,

    /// Slow 2G - 50 Kbps, 2000ms latency
    Slow2G,

    /// 2G - 250 Kbps, 800ms latency
    G2,

    /// 3G - 750 Kbps, 200ms latency
    G3,

    /// 4G - 4 Mbps, 50ms latency
    G4,

    /// WiFi - 30 Mbps, 10ms latency
    WiFi,

    /// Custom network condition
    Custom {
        /// Download speed in Kbps (kilobits per second)
        download_kbps: u32,
        /// Upload speed in Kbps (kilobits per second)
        upload_kbps: u32,
        /// Latency in milliseconds
        latency_ms: u32,
    },
}

impl NetworkCondition {
    /// Get download limit in bytes per second for this condition
    pub fn download_bytes_per_sec(&self) -> Option<u64> {
        match self {
            NetworkCondition::Offline => Some(0),
            NetworkCondition::Slow2G => Some(kbps_to_bytes_per_sec(50)),
            NetworkCondition::G2 => Some(kbps_to_bytes_per_sec(250)),
            NetworkCondition::G3 => Some(kbps_to_bytes_per_sec(750)),
            NetworkCondition::G4 => Some(kbps_to_bytes_per_sec(4000)), // 4 Mbps = 4000 Kbps
            NetworkCondition::WiFi => Some(kbps_to_bytes_per_sec(30000)), // 30 Mbps = 30000 Kbps
            NetworkCondition::Custom { download_kbps, .. } => {
                Some(kbps_to_bytes_per_sec(*download_kbps))
            }
        }
    }

    /// Get upload limit in bytes per second for this condition
    pub fn upload_bytes_per_sec(&self) -> Option<u64> {
        match self {
            NetworkCondition::Offline => Some(0),
            NetworkCondition::Slow2G => Some(kbps_to_bytes_per_sec(50)),
            NetworkCondition::G2 => Some(kbps_to_bytes_per_sec(250)),
            NetworkCondition::G3 => Some(kbps_to_bytes_per_sec(750)),
            NetworkCondition::G4 => Some(kbps_to_bytes_per_sec(4000)),
            NetworkCondition::WiFi => Some(kbps_to_bytes_per_sec(30000)),
            NetworkCondition::Custom { upload_kbps, .. } => Some(kbps_to_bytes_per_sec(*upload_kbps)),
        }
    }

    /// Get latency for this condition
    pub fn latency(&self) -> Duration {
        match self {
            NetworkCondition::Offline => Duration::ZERO,
            NetworkCondition::Slow2G => Duration::from_millis(2000),
            NetworkCondition::G2 => Duration::from_millis(800),
            NetworkCondition::G3 => Duration::from_millis(200),
            NetworkCondition::G4 => Duration::from_millis(50),
            NetworkCondition::WiFi => Duration::from_millis(10),
            NetworkCondition::Custom { latency_ms, .. } => Duration::from_millis(*latency_ms as u64),
        }
    }
}

/// Convert Kbps (kilobits per second) to bytes per second
///
/// 1 Kbps = 1000 bits/sec = 125 bytes/sec
fn kbps_to_bytes_per_sec(kbps: u32) -> u64 {
    (kbps as u64) * 1000 / 8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kbps_conversion() {
        assert_eq!(kbps_to_bytes_per_sec(8), 1000); // 8 Kbps = 1000 bytes/sec
        assert_eq!(kbps_to_bytes_per_sec(1000), 125000); // 1 Mbps = 125000 bytes/sec
    }

    #[test]
    fn test_slow_2g_values() {
        let condition = NetworkCondition::Slow2G;
        assert_eq!(condition.download_bytes_per_sec(), Some(6250)); // 50 Kbps
        assert_eq!(condition.upload_bytes_per_sec(), Some(6250));
        assert_eq!(condition.latency(), Duration::from_millis(2000));
    }

    #[test]
    fn test_custom_condition() {
        let condition = NetworkCondition::Custom {
            download_kbps: 100,
            upload_kbps: 50,
            latency_ms: 150,
        };
        assert_eq!(condition.download_bytes_per_sec(), Some(12500));
        assert_eq!(condition.upload_bytes_per_sec(), Some(6250));
        assert_eq!(condition.latency(), Duration::from_millis(150));
    }
}
