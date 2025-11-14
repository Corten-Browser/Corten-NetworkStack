use performance_benchmarks::stats::{calculate_mean, calculate_std_dev, find_min, find_max};

#[test]
fn test_calculate_mean_simple() {
    let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
    let mean = calculate_mean(&values);
    assert!((mean - 3.0).abs() < 0.001);
}

#[test]
fn test_calculate_mean_single_value() {
    let values = vec![42.0];
    let mean = calculate_mean(&values);
    assert!((mean - 42.0).abs() < 0.001);
}

#[test]
fn test_calculate_mean_decimal_values() {
    let values = vec![1.5, 2.5, 3.5];
    let mean = calculate_mean(&values);
    assert!((mean - 2.5).abs() < 0.001);
}

#[test]
fn test_calculate_std_dev_no_variance() {
    let values = vec![5.0, 5.0, 5.0, 5.0];
    let std_dev = calculate_std_dev(&values);
    assert!((std_dev - 0.0).abs() < 0.001);
}

#[test]
fn test_calculate_std_dev_with_variance() {
    let values = vec![2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0];
    let std_dev = calculate_std_dev(&values);
    // Expected std dev is 2.0
    assert!((std_dev - 2.0).abs() < 0.001);
}

#[test]
fn test_calculate_std_dev_single_value() {
    let values = vec![10.0];
    let std_dev = calculate_std_dev(&values);
    assert!((std_dev - 0.0).abs() < 0.001);
}

#[test]
fn test_find_min() {
    let values = vec![5.0, 2.0, 8.0, 1.0, 9.0];
    let min = find_min(&values);
    assert!((min - 1.0).abs() < 0.001);
}

#[test]
fn test_find_min_negative_values() {
    let values = vec![-5.0, -2.0, -8.0, -1.0];
    let min = find_min(&values);
    assert!((min - (-8.0)).abs() < 0.001);
}

#[test]
fn test_find_min_single_value() {
    let values = vec![42.0];
    let min = find_min(&values);
    assert!((min - 42.0).abs() < 0.001);
}

#[test]
fn test_find_max() {
    let values = vec![5.0, 2.0, 8.0, 1.0, 9.0];
    let max = find_max(&values);
    assert!((max - 9.0).abs() < 0.001);
}

#[test]
fn test_find_max_negative_values() {
    let values = vec![-5.0, -2.0, -8.0, -1.0];
    let max = find_max(&values);
    assert!((max - (-1.0)).abs() < 0.001);
}

#[test]
fn test_find_max_single_value() {
    let values = vec![42.0];
    let max = find_max(&values);
    assert!((max - 42.0).abs() < 0.001);
}
