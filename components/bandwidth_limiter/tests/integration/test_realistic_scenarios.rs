//! Integration tests for realistic bandwidth limiting scenarios

use bandwidth_limiter::{BandwidthLimiter, NetworkCondition};
use std::time::Duration;

#[tokio::test]
async fn test_realistic_slow_2g_download() {
    // Simulate downloading a 10KB file on Slow 2G
    let mut limiter = BandwidthLimiter::new();
    limiter.apply_condition(NetworkCondition::Slow2G);

    let file_size = 10240; // 10 KB
    let data = vec![0u8; file_size];

    let start = std::time::Instant::now();
    limiter.throttle_download(&data).await;
    let elapsed = start.elapsed();

    // Slow 2G: 6250 bytes/sec, so 10KB should take ~1.6 seconds + 2s latency = ~3.6s
    assert!(elapsed >= Duration::from_millis(3500));
    assert!(elapsed <= Duration::from_millis(4000));
}

#[tokio::test]
async fn test_realistic_4g_download() {
    // Simulate downloading a 1MB file on 4G
    let mut limiter = BandwidthLimiter::new();
    limiter.apply_condition(NetworkCondition::G4);

    let file_size = 1_048_576; // 1 MB
    let data = vec![0u8; file_size];

    let start = std::time::Instant::now();
    limiter.throttle_download(&data).await;
    let elapsed = start.elapsed();

    // 4G: 500000 bytes/sec, so 1MB should take ~2.1 seconds + 50ms latency
    assert!(elapsed >= Duration::from_millis(2000));
    assert!(elapsed <= Duration::from_millis(2500));
}

#[tokio::test]
async fn test_switching_network_conditions() {
    // Test switching from WiFi to Offline
    let mut limiter = BandwidthLimiter::new();

    // Start with WiFi
    limiter.apply_condition(NetworkCondition::WiFi);
    let data = vec![0u8; 1000];
    let delay1 = limiter.throttle_download(&data).await;
    assert!(delay1 < Duration::from_millis(50)); // WiFi is fast

    // Switch to Offline
    limiter.apply_condition(NetworkCondition::Offline);
    let delay2 = limiter.throttle_download(&data).await;
    // Offline should have effectively infinite delay (or very long)
    // But our implementation should handle this gracefully
}

#[tokio::test]
async fn test_mixed_upload_download_traffic() {
    // Simulate realistic mixed traffic
    let mut limiter = BandwidthLimiter::new();
    limiter.apply_condition(NetworkCondition::G3);

    let download_data = vec![0u8; 5000];
    let upload_data = vec![0u8; 2000];

    limiter.throttle_download(&download_data).await;
    limiter.throttle_upload(&upload_data).await;

    let stats = limiter.get_stats();
    assert_eq!(stats.bytes_received, 5000);
    assert_eq!(stats.bytes_sent, 2000);
}

#[tokio::test]
async fn test_custom_condition_for_specific_use_case() {
    // Custom condition: 1 Mbps download, 256 Kbps upload, 75ms latency
    let mut limiter = BandwidthLimiter::new();
    limiter.apply_condition(NetworkCondition::Custom {
        download_kbps: 1000, // 1 Mbps
        upload_kbps: 256,
        latency_ms: 75,
    });

    let stats = limiter.get_stats();
    assert_eq!(stats.download_limit, Some(125000)); // 1 Mbps = 125000 bytes/sec
    assert_eq!(stats.upload_limit, Some(32000)); // 256 Kbps = 32000 bytes/sec
    assert_eq!(stats.added_latency, Duration::from_millis(75));
}

#[tokio::test]
async fn test_statistics_over_time() {
    // Verify stats track correctly over multiple operations
    let mut limiter = BandwidthLimiter::new();
    limiter.set_download_limit(Some(10000)); // Fast enough to not slow down test

    for _ in 0..5 {
        let data = vec![0u8; 1000];
        limiter.throttle_download(&data).await;
    }

    let stats = limiter.get_stats();
    assert_eq!(stats.bytes_received, 5000);
    assert!(stats.duration_secs > 0.0);
}

#[tokio::test]
async fn test_removing_limits() {
    // Test that setting limits to None removes throttling
    let mut limiter = BandwidthLimiter::new();
    limiter.set_download_limit(Some(100)); // Very slow
    limiter.set_download_limit(None); // Remove limit

    let data = vec![0u8; 10000];
    let start = std::time::Instant::now();
    limiter.throttle_download(&data).await;
    let elapsed = start.elapsed();

    // Should be fast with no limit
    assert!(elapsed < Duration::from_millis(50));
}
