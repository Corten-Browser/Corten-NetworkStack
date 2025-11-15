use performance_benchmarks::{BenchmarkRunner, BenchmarkConfig, BenchmarkResult};
use std::time::Duration;
use tokio::time::sleep;

#[test]
fn test_benchmark_config_creation() {
    let config = BenchmarkConfig {
        iterations: 100,
        warmup_iterations: 10,
    };

    assert_eq!(config.iterations, 100);
    assert_eq!(config.warmup_iterations, 10);
}

#[test]
fn test_benchmark_runner_creation() {
    let config = BenchmarkConfig {
        iterations: 50,
        warmup_iterations: 5,
    };

    let _runner = BenchmarkRunner::new(config);
    // Should not panic
}

#[tokio::test]
async fn test_run_benchmark_basic() {
    let config = BenchmarkConfig {
        iterations: 10,
        warmup_iterations: 2,
    };

    let runner = BenchmarkRunner::new(config);

    let result = runner.run_benchmark("test_bench", || async {
        sleep(Duration::from_micros(100)).await;
    }).await;

    assert_eq!(result.name, "test_bench");
    assert_eq!(result.iterations, 10);
    assert!(result.total_time_ms > 0.0);
    assert!(result.avg_time_ms > 0.0);
    assert!(result.min_time_ms > 0.0);
    assert!(result.max_time_ms > 0.0);
    assert!(result.std_dev_ms >= 0.0);
}

#[tokio::test]
async fn test_benchmark_min_max_ordering() {
    let config = BenchmarkConfig {
        iterations: 5,
        warmup_iterations: 1,
    };

    let runner = BenchmarkRunner::new(config);

    let result = runner.run_benchmark("ordering_test", || async {
        sleep(Duration::from_micros(50)).await;
    }).await;

    // Min should be <= max
    assert!(result.min_time_ms <= result.max_time_ms);
    // Avg should be between min and max
    assert!(result.avg_time_ms >= result.min_time_ms);
    assert!(result.avg_time_ms <= result.max_time_ms);
}

#[tokio::test]
async fn test_benchmark_total_time_calculation() {
    let config = BenchmarkConfig {
        iterations: 5,
        warmup_iterations: 0,
    };

    let runner = BenchmarkRunner::new(config);

    let result = runner.run_benchmark("total_time_test", || async {
        sleep(Duration::from_micros(100)).await;
    }).await;

    // Total time should be approximately iterations * avg_time
    let expected_total = result.iterations as f64 * result.avg_time_ms;
    let diff = (result.total_time_ms - expected_total).abs();
    assert!(diff < 0.1); // Allow small floating point difference
}

#[test]
fn test_format_results() {
    let config = BenchmarkConfig {
        iterations: 10,
        warmup_iterations: 2,
    };

    let runner = BenchmarkRunner::new(config);

    let results = vec![
        BenchmarkResult {
            name: "bench1".to_string(),
            iterations: 10,
            total_time_ms: 100.0,
            avg_time_ms: 10.0,
            min_time_ms: 8.0,
            max_time_ms: 12.0,
            std_dev_ms: 1.5,
        },
        BenchmarkResult {
            name: "bench2".to_string(),
            iterations: 10,
            total_time_ms: 200.0,
            avg_time_ms: 20.0,
            min_time_ms: 18.0,
            max_time_ms: 22.0,
            std_dev_ms: 2.0,
        },
    ];

    let formatted = runner.format_results(&results);

    // Should contain benchmark names
    assert!(formatted.contains("bench1"));
    assert!(formatted.contains("bench2"));
    // Should contain timing information
    assert!(formatted.contains("10.0"));
    assert!(formatted.contains("20.0"));
}

#[test]
fn test_compare_results_no_change() {
    let config = BenchmarkConfig {
        iterations: 10,
        warmup_iterations: 2,
    };

    let runner = BenchmarkRunner::new(config);

    let baseline = BenchmarkResult {
        name: "test".to_string(),
        iterations: 10,
        total_time_ms: 100.0,
        avg_time_ms: 10.0,
        min_time_ms: 8.0,
        max_time_ms: 12.0,
        std_dev_ms: 1.5,
    };

    let current = BenchmarkResult {
        name: "test".to_string(),
        iterations: 10,
        total_time_ms: 100.0,
        avg_time_ms: 10.0,
        min_time_ms: 8.0,
        max_time_ms: 12.0,
        std_dev_ms: 1.5,
    };

    let change = runner.compare_results(&baseline, &current);
    assert!((change - 0.0).abs() < 0.01); // Should be 0% change
}

#[test]
fn test_compare_results_improvement() {
    let config = BenchmarkConfig {
        iterations: 10,
        warmup_iterations: 2,
    };

    let runner = BenchmarkRunner::new(config);

    let baseline = BenchmarkResult {
        name: "test".to_string(),
        iterations: 10,
        total_time_ms: 100.0,
        avg_time_ms: 10.0,
        min_time_ms: 8.0,
        max_time_ms: 12.0,
        std_dev_ms: 1.5,
    };

    let current = BenchmarkResult {
        name: "test".to_string(),
        iterations: 10,
        total_time_ms: 50.0,
        avg_time_ms: 5.0, // 50% faster
        min_time_ms: 4.0,
        max_time_ms: 6.0,
        std_dev_ms: 0.8,
    };

    let change = runner.compare_results(&baseline, &current);
    assert!((change - (-50.0)).abs() < 0.01); // Should be -50% (improvement)
}

#[test]
fn test_compare_results_regression() {
    let config = BenchmarkConfig {
        iterations: 10,
        warmup_iterations: 2,
    };

    let runner = BenchmarkRunner::new(config);

    let baseline = BenchmarkResult {
        name: "test".to_string(),
        iterations: 10,
        total_time_ms: 100.0,
        avg_time_ms: 10.0,
        min_time_ms: 8.0,
        max_time_ms: 12.0,
        std_dev_ms: 1.5,
    };

    let current = BenchmarkResult {
        name: "test".to_string(),
        iterations: 10,
        total_time_ms: 150.0,
        avg_time_ms: 15.0, // 50% slower
        min_time_ms: 13.0,
        max_time_ms: 17.0,
        std_dev_ms: 2.0,
    };

    let change = runner.compare_results(&baseline, &current);
    assert!((change - 50.0).abs() < 0.01); // Should be 50% (regression)
}
