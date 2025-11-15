use crate::BenchmarkResult;

/// Format benchmark results as a human-readable string
pub fn format_results(results: &[BenchmarkResult]) -> String {
    let mut output = String::new();

    output.push_str("Benchmark Results\n");
    output.push_str("=================\n\n");

    for result in results {
        output.push_str(&format!("Benchmark: {}\n", result.name));
        output.push_str(&format!("  Iterations: {}\n", result.iterations));
        output.push_str(&format!("  Total Time: {:.3} ms\n", result.total_time_ms));
        output.push_str(&format!("  Average Time: {:.3} ms\n", result.avg_time_ms));
        output.push_str(&format!("  Min Time: {:.3} ms\n", result.min_time_ms));
        output.push_str(&format!("  Max Time: {:.3} ms\n", result.max_time_ms));
        output.push_str(&format!("  Std Dev: {:.3} ms\n", result.std_dev_ms));
        output.push_str("\n");
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::BenchmarkResult;

    #[test]
    fn test_format_results() {
        let results = vec![
            BenchmarkResult {
                name: "test_bench".to_string(),
                iterations: 10,
                total_time_ms: 100.0,
                avg_time_ms: 10.0,
                min_time_ms: 8.0,
                max_time_ms: 12.0,
                std_dev_ms: 1.5,
            },
        ];

        let formatted = format_results(&results);

        assert!(formatted.contains("test_bench"));
        assert!(formatted.contains("10.0"));
    }
}
