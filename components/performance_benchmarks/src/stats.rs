/// Calculate the mean (average) of a list of values
pub fn calculate_mean(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }

    let sum: f64 = values.iter().sum();
    sum / values.len() as f64
}

/// Calculate the standard deviation of a list of values
pub fn calculate_std_dev(values: &[f64]) -> f64 {
    if values.len() <= 1 {
        return 0.0;
    }

    let mean = calculate_mean(values);
    let variance = values.iter()
        .map(|value| {
            let diff = value - mean;
            diff * diff
        })
        .sum::<f64>() / values.len() as f64;

    variance.sqrt()
}

/// Find the minimum value in a list
pub fn find_min(values: &[f64]) -> f64 {
    values.iter()
        .copied()
        .fold(f64::INFINITY, f64::min)
}

/// Find the maximum value in a list
pub fn find_max(values: &[f64]) -> f64 {
    values.iter()
        .copied()
        .fold(f64::NEG_INFINITY, f64::max)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mean_calculation() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert!((calculate_mean(&values) - 3.0).abs() < 0.001);
    }

    #[test]
    fn test_std_dev_calculation() {
        let values = vec![2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0];
        let std_dev = calculate_std_dev(&values);
        assert!((std_dev - 2.0).abs() < 0.001);
    }

    #[test]
    fn test_find_min_max() {
        let values = vec![5.0, 2.0, 8.0, 1.0, 9.0];
        assert!((find_min(&values) - 1.0).abs() < 0.001);
        assert!((find_max(&values) - 9.0).abs() < 0.001);
    }
}
