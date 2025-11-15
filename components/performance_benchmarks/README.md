# Performance Benchmarks

Performance benchmarking framework for the Corten NetworkStack.

## Overview

This component provides a flexible benchmarking framework for measuring and analyzing the performance of asynchronous operations. It supports configurable iterations, warmup runs, statistical analysis, and regression detection.

## Features

- **Configurable Benchmarking**: Set custom iteration counts and warmup runs
- **Statistical Analysis**: Automatic calculation of mean, min, max, and standard deviation
- **Warmup Iterations**: Exclude warmup runs from measurements for more accurate results
- **Regression Detection**: Compare benchmark results to detect performance improvements or regressions
- **Result Formatting**: Human-readable output of benchmark results

## Usage

### Basic Benchmarking

```rust
use performance_benchmarks::{BenchmarkRunner, BenchmarkConfig};
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    // Create configuration
    let config = BenchmarkConfig {
        iterations: 100,
        warmup_iterations: 10,
    };

    let runner = BenchmarkRunner::new(config);

    // Run benchmark
    let result = runner.run_benchmark("my_operation", || async {
        // Your async operation here
        sleep(Duration::from_millis(1)).await;
    }).await;

    // Print results
    println!("Benchmark: {}", result.name);
    println!("  Average: {:.3} ms", result.avg_time_ms);
    println!("  Min: {:.3} ms", result.min_time_ms);
    println!("  Max: {:.3} ms", result.max_time_ms);
    println!("  Std Dev: {:.3} ms", result.std_dev_ms);
}
```

### Multiple Benchmarks with Formatting

```rust
use performance_benchmarks::{BenchmarkRunner, BenchmarkConfig};

#[tokio::main]
async fn main() {
    let config = BenchmarkConfig {
        iterations: 50,
        warmup_iterations: 5,
    };

    let runner = BenchmarkRunner::new(config);

    // Run multiple benchmarks
    let results = vec![
        runner.run_benchmark("fast_operation", || async {
            // Fast operation
        }).await,
        runner.run_benchmark("slow_operation", || async {
            // Slow operation
        }).await,
    ];

    // Format and print all results
    let formatted = runner.format_results(&results);
    println!("{}", formatted);
}
```

### Regression Detection

```rust
use performance_benchmarks::{BenchmarkRunner, BenchmarkConfig};

#[tokio::main]
async fn main() {
    let config = BenchmarkConfig {
        iterations: 100,
        warmup_iterations: 10,
    };

    let runner = BenchmarkRunner::new(config);

    // Run baseline benchmark
    let baseline = runner.run_benchmark("baseline", || async {
        // Original implementation
    }).await;

    // Run new implementation
    let current = runner.run_benchmark("optimized", || async {
        // Optimized implementation
    }).await;

    // Compare results
    let change = runner.compare_results(&baseline, &current);

    if change < 0.0 {
        println!("Performance improved by {:.1}%", -change);
    } else if change > 0.0 {
        println!("Performance regressed by {:.1}%", change);
    } else {
        println!("No significant change");
    }
}
```

## API

### `BenchmarkConfig`

Configuration for benchmark execution.

**Fields:**
- `iterations: usize` - Number of iterations to run for measurement
- `warmup_iterations: usize` - Number of warmup iterations (not measured)

### `BenchmarkRunner`

Main benchmarking runner.

**Methods:**
- `new(config: BenchmarkConfig) -> Self` - Create new benchmark runner
- `run_benchmark<F, Fut>(&self, name: &str, bench_fn: F) -> BenchmarkResult` - Run benchmark
- `format_results(&self, results: &[BenchmarkResult]) -> String` - Format results as text
- `compare_results(&self, baseline: &BenchmarkResult, current: &BenchmarkResult) -> f64` - Compare two results

### `BenchmarkResult`

Result of a benchmark run.

**Fields:**
- `name: String` - Benchmark name
- `iterations: usize` - Number of iterations executed
- `total_time_ms: f64` - Total time in milliseconds
- `avg_time_ms: f64` - Average time per iteration in milliseconds
- `min_time_ms: f64` - Minimum time in milliseconds
- `max_time_ms: f64` - Maximum time in milliseconds
- `std_dev_ms: f64` - Standard deviation in milliseconds

## Statistical Functions

The `stats` module provides low-level statistical functions:

- `calculate_mean(values: &[f64]) -> f64` - Calculate mean of values
- `calculate_std_dev(values: &[f64]) -> f64` - Calculate standard deviation
- `find_min(values: &[f64]) -> f64` - Find minimum value
- `find_max(values: &[f64]) -> f64` - Find maximum value

## Testing

Run tests with:

```bash
cargo test
```

Test coverage includes:
- Unit tests for statistical calculations
- Unit tests for benchmark runner
- Integration tests for complete workflows
- Regression detection tests

## Implementation Details

### Warmup Iterations

Warmup iterations are executed before measurement begins to ensure:
- JIT compilation is complete
- Caches are warmed up
- The system reaches a steady state

Warmup iterations are NOT included in the timing measurements.

### Statistical Analysis

- **Mean**: Average of all measured iteration times
- **Min/Max**: Fastest and slowest iterations
- **Standard Deviation**: Measure of timing variability
  - Lower std dev = more consistent performance
  - Higher std dev = less consistent performance

### Regression Detection

The `compare_results` method returns percentage change:
- **Negative**: Performance improved (faster)
- **Positive**: Performance regressed (slower)
- **Zero**: No significant change

Formula: `((current.avg - baseline.avg) / baseline.avg) * 100`

## Dependencies

- `tokio` - Async runtime and timing utilities

## License

Part of the Corten NetworkStack project.
