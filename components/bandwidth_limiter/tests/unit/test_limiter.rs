//! Unit tests for BandwidthLimiter

use bandwidth_limiter::{BandwidthLimiter, NetworkCondition};
use std::time::Duration;

#[tokio::test]
async fn test_download_throttling_delays_transfer() {
    // Given a limiter with 1000 bytes/sec download limit
    let mut limiter = BandwidthLimiter::new();
    limiter.set_download_limit(Some(1000)); // 1000 bytes/sec

    // When throttling 1000 bytes
    let data = vec![0u8; 1000];
    let start = std::time::Instant::now();
    let delay = limiter.throttle_download(&data).await;
    let elapsed = start.elapsed();

    // Then delay should be approximately 1 second
    assert!(delay >= Duration::from_millis(900), "Delay too short: {:?}", delay);
    assert!(delay <= Duration::from_millis(1100), "Delay too long: {:?}", delay);
    assert!(elapsed >= Duration::from_millis(900), "Actual elapsed too short");
}

#[tokio::test]
async fn test_upload_throttling_delays_transfer() {
    // Given a limiter with 500 bytes/sec upload limit
    let mut limiter = BandwidthLimiter::new();
    limiter.set_upload_limit(Some(500));

    // When throttling 500 bytes
    let data = vec![0u8; 500];
    let start = std::time::Instant::now();
    let delay = limiter.throttle_upload(&data).await;
    let elapsed = start.elapsed();

    // Then delay should be approximately 1 second
    assert!(delay >= Duration::from_millis(900));
    assert!(delay <= Duration::from_millis(1100));
    assert!(elapsed >= Duration::from_millis(900));
}

#[tokio::test]
async fn test_latency_injection_adds_delay() {
    // Given a limiter with 100ms added latency
    let mut limiter = BandwidthLimiter::new();
    limiter.set_latency(Duration::from_millis(100));

    // When throttling download
    let data = vec![0u8; 100];
    let start = std::time::Instant::now();
    let delay = limiter.throttle_download(&data).await;
    let elapsed = start.elapsed();

    // Then delay should include 100ms latency
    assert!(delay >= Duration::from_millis(90));
    assert!(elapsed >= Duration::from_millis(90));
}

#[tokio::test]
async fn test_unlimited_bandwidth_has_minimal_delay() {
    // Given a limiter with no limits
    let limiter = BandwidthLimiter::new();

    // When throttling large data
    let data = vec![0u8; 1_000_000];
    let start = std::time::Instant::now();
    let delay = limiter.throttle_download(&data).await;
    let elapsed = start.elapsed();

    // Then delay should be minimal (only latency, which defaults to 0)
    assert!(delay < Duration::from_millis(10));
    assert!(elapsed < Duration::from_millis(10));
}

#[tokio::test]
fn test_network_condition_slow_2g_sets_correct_values() {
    // Given a limiter
    let mut limiter = BandwidthLimiter::new();

    // When applying Slow2G condition
    limiter.apply_condition(NetworkCondition::Slow2G);

    // Then limits should be: 50 Kbps = 6250 bytes/sec, 2000ms latency
    let stats = limiter.get_stats();
    assert_eq!(stats.download_limit, Some(6250)); // 50 Kbps = 50000 bits/sec = 6250 bytes/sec
    assert_eq!(stats.upload_limit, Some(6250));
    assert_eq!(stats.added_latency, Duration::from_millis(2000));
}

#[tokio::test]
fn test_network_condition_2g_sets_correct_values() {
    let mut limiter = BandwidthLimiter::new();
    limiter.apply_condition(NetworkCondition::G2);

    let stats = limiter.get_stats();
    assert_eq!(stats.download_limit, Some(31250)); // 250 Kbps = 31250 bytes/sec
    assert_eq!(stats.upload_limit, Some(31250));
    assert_eq!(stats.added_latency, Duration::from_millis(800));
}

#[tokio::test]
fn test_network_condition_3g_sets_correct_values() {
    let mut limiter = BandwidthLimiter::new();
    limiter.apply_condition(NetworkCondition::G3);

    let stats = limiter.get_stats();
    assert_eq!(stats.download_limit, Some(93750)); // 750 Kbps = 93750 bytes/sec
    assert_eq!(stats.upload_limit, Some(93750));
    assert_eq!(stats.added_latency, Duration::from_millis(200));
}

#[tokio::test]
fn test_network_condition_4g_sets_correct_values() {
    let mut limiter = BandwidthLimiter::new();
    limiter.apply_condition(NetworkCondition::G4);

    let stats = limiter.get_stats();
    assert_eq!(stats.download_limit, Some(500000)); // 4 Mbps = 500000 bytes/sec
    assert_eq!(stats.upload_limit, Some(500000));
    assert_eq!(stats.added_latency, Duration::from_millis(50));
}

#[tokio::test]
fn test_network_condition_wifi_sets_correct_values() {
    let mut limiter = BandwidthLimiter::new();
    limiter.apply_condition(NetworkCondition::WiFi);

    let stats = limiter.get_stats();
    assert_eq!(stats.download_limit, Some(3750000)); // 30 Mbps = 3750000 bytes/sec
    assert_eq!(stats.upload_limit, Some(3750000));
    assert_eq!(stats.added_latency, Duration::from_millis(10));
}

#[tokio::test]
fn test_network_condition_offline_sets_zero_bandwidth() {
    let mut limiter = BandwidthLimiter::new();
    limiter.apply_condition(NetworkCondition::Offline);

    let stats = limiter.get_stats();
    assert_eq!(stats.download_limit, Some(0));
    assert_eq!(stats.upload_limit, Some(0));
}

#[tokio::test]
fn test_network_condition_custom_sets_correct_values() {
    let mut limiter = BandwidthLimiter::new();
    limiter.apply_condition(NetworkCondition::Custom {
        download_kbps: 100,
        upload_kbps: 50,
        latency_ms: 150,
    });

    let stats = limiter.get_stats();
    assert_eq!(stats.download_limit, Some(12500)); // 100 Kbps = 12500 bytes/sec
    assert_eq!(stats.upload_limit, Some(6250)); // 50 Kbps = 6250 bytes/sec
    assert_eq!(stats.added_latency, Duration::from_millis(150));
}

#[tokio::test]
async fn test_statistics_tracking() {
    // Given a limiter
    let limiter = BandwidthLimiter::new();

    // When throttling some data
    let download_data = vec![0u8; 1000];
    let upload_data = vec![0u8; 500];

    limiter.throttle_download(&download_data).await;
    limiter.throttle_upload(&upload_data).await;

    // Then stats should track the bytes
    let stats = limiter.get_stats();
    assert_eq!(stats.bytes_received, 1000);
    assert_eq!(stats.bytes_sent, 500);
    assert!(stats.duration_secs > 0.0);
}

#[tokio::test]
async fn test_combined_bandwidth_and_latency() {
    // Given a limiter with both bandwidth limit and latency
    let mut limiter = BandwidthLimiter::new();
    limiter.set_download_limit(Some(1000)); // 1000 bytes/sec
    limiter.set_latency(Duration::from_millis(100));

    // When throttling 1000 bytes
    let data = vec![0u8; 1000];
    let start = std::time::Instant::now();
    let delay = limiter.throttle_download(&data).await;
    let elapsed = start.elapsed();

    // Then delay should be ~1 second (bandwidth) + 100ms (latency)
    assert!(delay >= Duration::from_millis(1000));
    assert!(elapsed >= Duration::from_millis(1000));
}

#[tokio::test]
async fn test_multiple_throttle_calls_accumulate() {
    // Given a limiter with 1000 bytes/sec limit
    let mut limiter = BandwidthLimiter::new();
    limiter.set_download_limit(Some(1000));

    // When throttling multiple times
    let data = vec![0u8; 500];
    limiter.throttle_download(&data).await;

    let start = std::time::Instant::now();
    limiter.throttle_download(&data).await;
    let elapsed = start.elapsed();

    // Then second call should account for accumulated bytes
    // First 500 bytes take 0.5s, second 500 bytes should wait until 1s from start
    assert!(elapsed >= Duration::from_millis(400));
}

#[tokio::test]
fn test_set_download_limit() {
    let mut limiter = BandwidthLimiter::new();
    limiter.set_download_limit(Some(5000));

    let stats = limiter.get_stats();
    assert_eq!(stats.download_limit, Some(5000));
}

#[tokio::test]
fn test_set_upload_limit() {
    let mut limiter = BandwidthLimiter::new();
    limiter.set_upload_limit(Some(3000));

    let stats = limiter.get_stats();
    assert_eq!(stats.upload_limit, Some(3000));
}

#[tokio::test]
fn test_set_latency() {
    let mut limiter = BandwidthLimiter::new();
    limiter.set_latency(Duration::from_millis(250));

    let stats = limiter.get_stats();
    assert_eq!(stats.added_latency, Duration::from_millis(250));
}

#[tokio::test]
async fn test_zero_byte_transfer_has_minimal_delay() {
    let limiter = BandwidthLimiter::new();
    let data = vec![];

    let delay = limiter.throttle_download(&data).await;

    assert_eq!(delay, Duration::ZERO);
}
