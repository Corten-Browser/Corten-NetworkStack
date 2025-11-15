pub mod stats;
mod runner;
mod formatter;

pub use runner::BenchmarkRunner;

/// Configuration for benchmark execution
#[derive(Debug, Clone)]
pub struct BenchmarkConfig {
    /// Number of iterations to run for measurement
    pub iterations: usize,
    /// Number of warmup iterations (not measured)
    pub warmup_iterations: usize,
}

/// Result of a benchmark run
#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    /// Name of the benchmark
    pub name: String,
    /// Number of iterations executed
    pub iterations: usize,
    /// Total time in milliseconds
    pub total_time_ms: f64,
    /// Average time per iteration in milliseconds
    pub avg_time_ms: f64,
    /// Minimum time in milliseconds
    pub min_time_ms: f64,
    /// Maximum time in milliseconds
    pub max_time_ms: f64,
    /// Standard deviation in milliseconds
    pub std_dev_ms: f64,
}

impl BenchmarkRunner {
    /// Format multiple benchmark results as a human-readable string
    pub fn format_results(&self, results: &[BenchmarkResult]) -> String {
        formatter::format_results(results)
    }
}
