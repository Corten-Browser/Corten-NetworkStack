use crate::{BenchmarkConfig, BenchmarkResult};
use crate::stats::{calculate_mean, calculate_std_dev, find_min, find_max};
use std::time::Instant;

pub struct BenchmarkRunner {
    pub config: BenchmarkConfig,
}

impl BenchmarkRunner {
    pub fn new(config: BenchmarkConfig) -> Self {
        Self { config }
    }

    pub async fn run_benchmark<F, Fut>(&self, name: &str, bench_fn: F) -> BenchmarkResult
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = ()>,
    {
        // Run warmup iterations (not measured)
        for _ in 0..self.config.warmup_iterations {
            bench_fn().await;
        }

        // Measure iterations
        let mut times = Vec::with_capacity(self.config.iterations);

        for _ in 0..self.config.iterations {
            let start = Instant::now();
            bench_fn().await;
            let elapsed = start.elapsed();
            times.push(elapsed.as_secs_f64() * 1000.0); // Convert to milliseconds
        }

        // Calculate statistics
        let total_time_ms = times.iter().sum();
        let avg_time_ms = calculate_mean(&times);
        let min_time_ms = find_min(&times);
        let max_time_ms = find_max(&times);
        let std_dev_ms = calculate_std_dev(&times);

        BenchmarkResult {
            name: name.to_string(),
            iterations: self.config.iterations,
            total_time_ms,
            avg_time_ms,
            min_time_ms,
            max_time_ms,
            std_dev_ms,
        }
    }

    pub fn compare_results(&self, baseline: &BenchmarkResult, current: &BenchmarkResult) -> f64 {
        // Calculate percentage change
        // Positive = regression (slower)
        // Negative = improvement (faster)
        if baseline.avg_time_ms == 0.0 {
            return 0.0;
        }

        ((current.avg_time_ms - baseline.avg_time_ms) / baseline.avg_time_ms) * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_benchmark_runner() {
        let config = BenchmarkConfig {
            iterations: 5,
            warmup_iterations: 2,
        };

        let runner = BenchmarkRunner::new(config);

        let result = runner.run_benchmark("test", || async {
            sleep(Duration::from_micros(100)).await;
        }).await;

        assert_eq!(result.iterations, 5);
        assert!(result.avg_time_ms > 0.0);
    }

    #[test]
    fn test_compare_results() {
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
            avg_time_ms: 15.0,
            min_time_ms: 13.0,
            max_time_ms: 17.0,
            std_dev_ms: 2.0,
        };

        let change = runner.compare_results(&baseline, &current);
        assert!((change - 50.0).abs() < 0.01); // 50% regression
    }
}
