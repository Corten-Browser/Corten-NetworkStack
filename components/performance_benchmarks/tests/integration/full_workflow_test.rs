use performance_benchmarks::{BenchmarkRunner, BenchmarkConfig};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_complete_benchmark_workflow() {
    // Create configuration
    let config = BenchmarkConfig {
        iterations: 20,
        warmup_iterations: 5,
    };

    let runner = BenchmarkRunner::new(config);

    // Run multiple benchmarks
    let result1 = runner.run_benchmark("fast_operation", || async {
        sleep(Duration::from_micros(10)).await;
    }).await;

    let result2 = runner.run_benchmark("slow_operation", || async {
        sleep(Duration::from_micros(100)).await;
    }).await;

    // Verify results
    assert_eq!(result1.iterations, 20);
    assert_eq!(result2.iterations, 20);

    // Slow operation should take longer
    assert!(result2.avg_time_ms > result1.avg_time_ms);

    // Format results
    let formatted = runner.format_results(&[result1, result2]);
    assert!(formatted.contains("fast_operation"));
    assert!(formatted.contains("slow_operation"));
}

#[tokio::test]
async fn test_regression_detection_workflow() {
    let config = BenchmarkConfig {
        iterations: 10,
        warmup_iterations: 2,
    };

    let runner = BenchmarkRunner::new(config);

    // Run baseline with 1ms sleep
    let baseline = runner.run_benchmark("operation", || async {
        sleep(Duration::from_millis(1)).await;
    }).await;

    // Simulate optimization (faster operation - 500us)
    let optimized = runner.run_benchmark("operation_optimized", || async {
        sleep(Duration::from_micros(500)).await;
    }).await;

    // Simulate regression (slower operation - 2ms)
    let regressed = runner.run_benchmark("operation_regressed", || async {
        sleep(Duration::from_millis(2)).await;
    }).await;

    // Compare with baseline
    let optimized_change = runner.compare_results(&baseline, &optimized);
    let regressed_change = runner.compare_results(&baseline, &regressed);

    // Optimized should show improvement (negative change)
    // With more realistic timings, this should be reliably negative
    assert!(optimized_change < -10.0, "Expected significant improvement, got {}%", optimized_change);

    // Regressed should show degradation (positive change)
    // With 2ms vs 1ms, this should be close to +100%
    assert!(regressed_change > 10.0, "Expected significant regression, got {}%", regressed_change);
}

#[tokio::test]
async fn test_warmup_iterations_exist() {
    let config = BenchmarkConfig {
        iterations: 5,
        warmup_iterations: 2,
    };

    let runner = BenchmarkRunner::new(config);

    let result = runner.run_benchmark("warmup_test", || async {
        sleep(Duration::from_micros(10)).await;
    }).await;

    // Only measured iterations (5) should be counted in results
    assert_eq!(result.iterations, 5);
    assert!(result.avg_time_ms > 0.0);
}

#[tokio::test]
async fn test_zero_warmup_iterations() {
    let config = BenchmarkConfig {
        iterations: 10,
        warmup_iterations: 0,
    };

    let runner = BenchmarkRunner::new(config);

    let result = runner.run_benchmark("no_warmup", || async {
        sleep(Duration::from_micros(20)).await;
    }).await;

    assert_eq!(result.iterations, 10);
    assert!(result.avg_time_ms > 0.0);
}

#[tokio::test]
async fn test_statistical_accuracy() {
    let config = BenchmarkConfig {
        iterations: 100,
        warmup_iterations: 10,
    };

    let runner = BenchmarkRunner::new(config);

    let result = runner.run_benchmark("stats_test", || async {
        sleep(Duration::from_micros(50)).await;
    }).await;

    // With consistent sleep, std dev should be relatively small
    // Std dev should be less than 50% of mean for consistent operations
    assert!(result.std_dev_ms < result.avg_time_ms * 0.5);

    // Min and max should be reasonably close for consistent operations
    let range = result.max_time_ms - result.min_time_ms;
    assert!(range < result.avg_time_ms * 2.0);
}
