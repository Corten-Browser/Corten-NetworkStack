# Bandwidth Limiter

Bandwidth throttling and network condition simulation for testing network-dependent code under various network conditions.

## Overview

The bandwidth_limiter component provides:
- **Download/Upload throttling**: Limit transfer speeds in bytes per second
- **Latency injection**: Add realistic network latency to operations
- **Network condition presets**: Simulate Slow2G, 2G, 3G, 4G, WiFi, and Offline
- **Custom conditions**: Define your own bandwidth and latency parameters
- **Usage tracking**: Monitor bytes transferred and connection duration

## Features

### Network Condition Presets

- **Offline**: 0 bandwidth (simulates no connectivity)
- **Slow2G**: 50 Kbps, 2000ms latency
- **2G**: 250 Kbps, 800ms latency
- **3G**: 750 Kbps, 200ms latency
- **4G**: 4 Mbps, 50ms latency
- **WiFi**: 30 Mbps, 10ms latency
- **Custom**: Define your own parameters

### Bandwidth Tracking

Tracks:
- Total bytes downloaded
- Total bytes uploaded
- Duration since tracking started
- Current bandwidth limits
- Added latency

## Usage

### Basic Example

```rust
use bandwidth_limiter::{BandwidthLimiter, NetworkCondition};
use std::time::Duration;

#[tokio::main]
async fn main() {
    let mut limiter = BandwidthLimiter::new();

    // Simulate 3G network
    limiter.apply_condition(NetworkCondition::G3);

    // Throttle a download
    let data = vec![0u8; 10000];
    let delay = limiter.throttle_download(&data).await;
    println!("Download delayed by: {:?}", delay);

    // Check statistics
    let stats = limiter.get_stats();
    println!("Downloaded: {} bytes", stats.bytes_received);
    println!("Current limit: {:?} bytes/sec", stats.download_limit);
}
```

### Custom Network Conditions

```rust
use bandwidth_limiter::{BandwidthLimiter, NetworkCondition};

let mut limiter = BandwidthLimiter::new();

// Define custom condition: 1 Mbps down, 256 Kbps up, 75ms latency
limiter.apply_condition(NetworkCondition::Custom {
    download_kbps: 1000,  // 1 Mbps
    upload_kbps: 256,      // 256 Kbps
    latency_ms: 75,
});
```

### Manual Configuration

```rust
use bandwidth_limiter::BandwidthLimiter;
use std::time::Duration;

let mut limiter = BandwidthLimiter::new();

// Set limits manually
limiter.set_download_limit(Some(125000)); // 1 Mbps in bytes/sec
limiter.set_upload_limit(Some(62500));    // 500 Kbps in bytes/sec
limiter.set_latency(Duration::from_millis(100));
```

### Unlimited Bandwidth

```rust
use bandwidth_limiter::BandwidthLimiter;

let mut limiter = BandwidthLimiter::new();

// Remove limits (unlimited bandwidth)
limiter.set_download_limit(None);
limiter.set_upload_limit(None);
```

## API Documentation

### BandwidthLimiter

Main struct for bandwidth limiting.

#### Methods

- `new() -> Self` - Create a new limiter with no limits
- `set_download_limit(&mut self, bytes_per_sec: Option<u64>)` - Set download limit
- `set_upload_limit(&mut self, bytes_per_sec: Option<u64>)` - Set upload limit
- `set_latency(&mut self, latency: Duration)` - Set additional latency
- `apply_condition(&mut self, condition: NetworkCondition)` - Apply preset condition
- `throttle_download(&self, bytes: &[u8]) -> Duration` - Throttle download (async)
- `throttle_upload(&self, bytes: &[u8]) -> Duration` - Throttle upload (async)
- `get_stats(&self) -> BandwidthStats` - Get current statistics
- `reset_stats(&mut self)` - Reset all statistics

### NetworkCondition

Enum for network condition presets.

#### Variants

- `Offline` - No connectivity (0 bandwidth)
- `Slow2G` - 50 Kbps, 2000ms latency
- `G2` - 250 Kbps, 800ms latency
- `G3` - 750 Kbps, 200ms latency
- `G4` - 4 Mbps, 50ms latency
- `WiFi` - 30 Mbps, 10ms latency
- `Custom { download_kbps, upload_kbps, latency_ms }` - Custom parameters

### BandwidthStats

Statistics structure returned by `get_stats()`.

#### Fields

- `download_limit: Option<u64>` - Current download limit in bytes/sec (None = unlimited)
- `upload_limit: Option<u64>` - Current upload limit in bytes/sec (None = unlimited)
- `added_latency: Duration` - Additional latency
- `bytes_sent: u64` - Total bytes uploaded
- `bytes_received: u64` - Total bytes downloaded
- `duration_secs: f64` - Duration since tracking started in seconds

### BandwidthTracker

Internal tracker for bandwidth usage (exposed for advanced use).

#### Fields

- `bytes_sent: u64` - Total bytes sent
- `bytes_received: u64` - Total bytes received
- `start_time: Instant` - When tracking started

#### Methods

- `new() -> Self` - Create new tracker
- `record_download(&mut self, bytes: u64)` - Record downloaded bytes
- `record_upload(&mut self, bytes: u64)` - Record uploaded bytes
- `elapsed_secs(&self) -> f64` - Get elapsed seconds
- `reset(&mut self)` - Reset all statistics

## Testing

### Running Tests

```bash
# Run all tests
cargo test

# Run unit tests only
cargo test --lib

# Run integration tests only
cargo test --test integration

# Run with coverage
cargo tarpaulin --out Html
```

### Test Coverage

The component includes comprehensive test coverage:

**Unit Tests** (`tests/unit/test_limiter.rs`):
- Download throttling delays transfer
- Upload throttling delays transfer
- Latency injection adds delay
- Unlimited bandwidth has minimal delay
- Network condition presets set correct values
- Custom conditions work correctly
- Statistics tracking
- Combined bandwidth and latency
- Multiple throttle calls accumulate

**Integration Tests** (`tests/integration/test_realistic_scenarios.rs`):
- Realistic Slow2G download scenario
- Realistic 4G download scenario
- Switching network conditions
- Mixed upload/download traffic
- Custom conditions for specific use cases
- Statistics over time
- Removing limits

## Implementation Details

### Throttling Algorithm

The limiter calculates the time needed to transfer data based on the bandwidth limit:

```rust
transfer_time = bytes_to_transfer / bytes_per_second
```

It tracks the last operation time and ensures proper spacing between operations to maintain the specified bandwidth limit.

### Latency Injection

Latency is added to every operation, simulating network round-trip time:

```rust
total_delay = bandwidth_delay + added_latency
```

### Thread Safety

The `BandwidthLimiter` uses `Arc<Mutex<...>>` internally, making it safe to clone and use across async tasks.

## Dependencies

- `tokio` - Async runtime for sleep operations
- `network_types` - Network type definitions
- `network_errors` - Error handling

## Architecture

```
BandwidthLimiter
├── LimiterState (internal)
│   ├── download_limit: Option<u64>
│   ├── upload_limit: Option<u64>
│   ├── added_latency: Duration
│   ├── tracker: BandwidthTracker
│   ├── last_download_time: Option<Instant>
│   └── last_upload_time: Option<Instant>
└── Public API
    ├── set_download_limit()
    ├── set_upload_limit()
    ├── set_latency()
    ├── apply_condition()
    ├── throttle_download()
    ├── throttle_upload()
    └── get_stats()
```

## Performance Considerations

- **Minimal overhead**: When no limits are set, operations complete with minimal delay
- **Accurate timing**: Uses tokio::time::sleep for precise delays
- **Lock contention**: Mutex held only for short durations during state updates
- **Memory efficient**: Minimal memory footprint (~100 bytes per instance)

## License

MIT OR Apache-2.0
