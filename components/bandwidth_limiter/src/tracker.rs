//! Bandwidth usage tracking

use std::time::Instant;

/// Tracks bandwidth usage over time
#[derive(Debug)]
pub struct BandwidthTracker {
    /// Total bytes sent (uploaded)
    pub bytes_sent: u64,
    /// Total bytes received (downloaded)
    pub bytes_received: u64,
    /// When tracking started
    pub start_time: Instant,
}

impl BandwidthTracker {
    /// Create a new bandwidth tracker
    pub fn new() -> Self {
        Self {
            bytes_sent: 0,
            bytes_received: 0,
            start_time: Instant::now(),
        }
    }

    /// Record downloaded bytes
    pub fn record_download(&mut self, bytes: u64) {
        self.bytes_received += bytes;
    }

    /// Record uploaded bytes
    pub fn record_upload(&mut self, bytes: u64) {
        self.bytes_sent += bytes;
    }

    /// Get elapsed time since tracking started
    pub fn elapsed_secs(&self) -> f64 {
        self.start_time.elapsed().as_secs_f64()
    }

    /// Reset all tracking statistics
    pub fn reset(&mut self) {
        self.bytes_sent = 0;
        self.bytes_received = 0;
        self.start_time = Instant::now();
    }
}

impl Default for BandwidthTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tracker_new() {
        let tracker = BandwidthTracker::new();
        assert_eq!(tracker.bytes_sent, 0);
        assert_eq!(tracker.bytes_received, 0);
    }

    #[test]
    fn test_record_download() {
        let mut tracker = BandwidthTracker::new();
        tracker.record_download(1000);
        tracker.record_download(500);
        assert_eq!(tracker.bytes_received, 1500);
    }

    #[test]
    fn test_record_upload() {
        let mut tracker = BandwidthTracker::new();
        tracker.record_upload(250);
        tracker.record_upload(750);
        assert_eq!(tracker.bytes_sent, 1000);
    }

    #[test]
    fn test_elapsed_time() {
        let tracker = BandwidthTracker::new();
        std::thread::sleep(std::time::Duration::from_millis(10));
        let elapsed = tracker.elapsed_secs();
        assert!(elapsed >= 0.01);
    }

    #[test]
    fn test_reset() {
        let mut tracker = BandwidthTracker::new();
        tracker.record_download(1000);
        tracker.record_upload(500);

        tracker.reset();

        assert_eq!(tracker.bytes_sent, 0);
        assert_eq!(tracker.bytes_received, 0);
    }
}
