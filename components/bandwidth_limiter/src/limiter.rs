//! Bandwidth limiter implementation

use crate::conditions::NetworkCondition;
use crate::tracker::BandwidthTracker;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::time::sleep;

/// Bandwidth limiter with throttling capabilities
///
/// Controls download and upload speeds, adds latency, and tracks bandwidth usage.
#[derive(Debug, Clone)]
pub struct BandwidthLimiter {
    state: Arc<Mutex<LimiterState>>,
}

#[derive(Debug)]
struct LimiterState {
    download_limit: Option<u64>, // bytes per second
    upload_limit: Option<u64>,   // bytes per second
    added_latency: Duration,
    tracker: BandwidthTracker,
    last_download_time: Option<Instant>,
    last_upload_time: Option<Instant>,
}

/// Bandwidth statistics
#[derive(Debug, Clone, PartialEq)]
pub struct BandwidthStats {
    /// Current download limit in bytes per second (None = unlimited)
    pub download_limit: Option<u64>,
    /// Current upload limit in bytes per second (None = unlimited)
    pub upload_limit: Option<u64>,
    /// Added latency
    pub added_latency: Duration,
    /// Total bytes sent (uploaded)
    pub bytes_sent: u64,
    /// Total bytes received (downloaded)
    pub bytes_received: u64,
    /// Duration in seconds since tracking started
    pub duration_secs: f64,
}

impl BandwidthLimiter {
    /// Create a new bandwidth limiter with no limits
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(LimiterState {
                download_limit: None,
                upload_limit: None,
                added_latency: Duration::ZERO,
                tracker: BandwidthTracker::new(),
                last_download_time: None,
                last_upload_time: None,
            })),
        }
    }

    /// Set download bandwidth limit in bytes per second
    ///
    /// # Arguments
    /// * `bytes_per_sec` - Limit in bytes per second, or None for unlimited
    pub fn set_download_limit(&mut self, bytes_per_sec: Option<u64>) {
        let mut state = self.state.lock().unwrap();
        state.download_limit = bytes_per_sec;
    }

    /// Set upload bandwidth limit in bytes per second
    ///
    /// # Arguments
    /// * `bytes_per_sec` - Limit in bytes per second, or None for unlimited
    pub fn set_upload_limit(&mut self, bytes_per_sec: Option<u64>) {
        let mut state = self.state.lock().unwrap();
        state.upload_limit = bytes_per_sec;
    }

    /// Set additional latency to inject
    ///
    /// # Arguments
    /// * `latency` - Latency to add to each operation
    pub fn set_latency(&mut self, latency: Duration) {
        let mut state = self.state.lock().unwrap();
        state.added_latency = latency;
    }

    /// Apply a preset network condition
    ///
    /// # Arguments
    /// * `condition` - Network condition to apply
    pub fn apply_condition(&mut self, condition: NetworkCondition) {
        let mut state = self.state.lock().unwrap();
        state.download_limit = condition.download_bytes_per_sec();
        state.upload_limit = condition.upload_bytes_per_sec();
        state.added_latency = condition.latency();
    }

    /// Throttle a download operation
    ///
    /// Simulates downloading the given bytes, applying bandwidth limits and latency.
    /// Returns the total delay applied.
    ///
    /// # Arguments
    /// * `bytes` - Data being downloaded
    pub async fn throttle_download(&self, bytes: &[u8]) -> Duration {
        let byte_count = bytes.len() as u64;

        if byte_count == 0 {
            return Duration::ZERO;
        }

        let (bandwidth_delay, latency) = {
            let mut state = self.state.lock().unwrap();

            // Record the download
            state.tracker.record_download(byte_count);

            // Calculate bandwidth delay
            let bandwidth_delay = if let Some(limit) = state.download_limit {
                if limit == 0 {
                    // Offline mode - extremely long delay
                    Duration::from_secs(365 * 24 * 60 * 60) // 1 year (effectively infinite)
                } else {
                    // Calculate time needed to transfer these bytes
                    let transfer_time = Duration::from_secs_f64(byte_count as f64 / limit as f64);

                    // If we have a last download time, check if we need to wait
                    let now = Instant::now();
                    if let Some(last_time) = state.last_download_time {
                        let time_since_last = now.duration_since(last_time);
                        if time_since_last < transfer_time {
                            // Need to wait for the bandwidth window
                            transfer_time - time_since_last
                        } else {
                            // Enough time has passed
                            Duration::ZERO
                        }
                    } else {
                        // First download
                        transfer_time
                    }
                }
            } else {
                Duration::ZERO
            };

            state.last_download_time = Some(Instant::now() + bandwidth_delay);

            (bandwidth_delay, state.added_latency)
        };

        // Apply the delays
        let total_delay = bandwidth_delay + latency;
        if total_delay > Duration::ZERO {
            sleep(total_delay).await;
        }

        total_delay
    }

    /// Throttle an upload operation
    ///
    /// Simulates uploading the given bytes, applying bandwidth limits and latency.
    /// Returns the total delay applied.
    ///
    /// # Arguments
    /// * `bytes` - Data being uploaded
    pub async fn throttle_upload(&self, bytes: &[u8]) -> Duration {
        let byte_count = bytes.len() as u64;

        if byte_count == 0 {
            return Duration::ZERO;
        }

        let (bandwidth_delay, latency) = {
            let mut state = self.state.lock().unwrap();

            // Record the upload
            state.tracker.record_upload(byte_count);

            // Calculate bandwidth delay
            let bandwidth_delay = if let Some(limit) = state.upload_limit {
                if limit == 0 {
                    // Offline mode
                    Duration::from_secs(365 * 24 * 60 * 60)
                } else {
                    let transfer_time = Duration::from_secs_f64(byte_count as f64 / limit as f64);

                    let now = Instant::now();
                    if let Some(last_time) = state.last_upload_time {
                        let time_since_last = now.duration_since(last_time);
                        if time_since_last < transfer_time {
                            transfer_time - time_since_last
                        } else {
                            Duration::ZERO
                        }
                    } else {
                        transfer_time
                    }
                }
            } else {
                Duration::ZERO
            };

            state.last_upload_time = Some(Instant::now() + bandwidth_delay);

            (bandwidth_delay, state.added_latency)
        };

        // Apply the delays
        let total_delay = bandwidth_delay + latency;
        if total_delay > Duration::ZERO {
            sleep(total_delay).await;
        }

        total_delay
    }

    /// Get current bandwidth statistics
    pub fn get_stats(&self) -> BandwidthStats {
        let state = self.state.lock().unwrap();
        BandwidthStats {
            download_limit: state.download_limit,
            upload_limit: state.upload_limit,
            added_latency: state.added_latency,
            bytes_sent: state.tracker.bytes_sent,
            bytes_received: state.tracker.bytes_received,
            duration_secs: state.tracker.elapsed_secs(),
        }
    }

    /// Reset bandwidth statistics
    pub fn reset_stats(&mut self) {
        let mut state = self.state.lock().unwrap();
        state.tracker.reset();
        state.last_download_time = None;
        state.last_upload_time = None;
    }
}

impl Default for BandwidthLimiter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_limiter_has_no_limits() {
        let limiter = BandwidthLimiter::new();
        let stats = limiter.get_stats();
        assert_eq!(stats.download_limit, None);
        assert_eq!(stats.upload_limit, None);
        assert_eq!(stats.added_latency, Duration::ZERO);
    }

    #[test]
    fn test_set_limits() {
        let mut limiter = BandwidthLimiter::new();
        limiter.set_download_limit(Some(1000));
        limiter.set_upload_limit(Some(500));
        limiter.set_latency(Duration::from_millis(100));

        let stats = limiter.get_stats();
        assert_eq!(stats.download_limit, Some(1000));
        assert_eq!(stats.upload_limit, Some(500));
        assert_eq!(stats.added_latency, Duration::from_millis(100));
    }

    #[tokio::test]
    async fn test_zero_bytes_has_no_delay() {
        let limiter = BandwidthLimiter::new();
        let delay = limiter.throttle_download(&[]).await;
        assert_eq!(delay, Duration::ZERO);
    }
}
