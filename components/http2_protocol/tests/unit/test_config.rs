//! Unit tests for Http2Config

use http2_protocol::Http2Config;

#[test]
fn test_http2_config_default_values() {
    //! Given: Default Http2Config is created
    //! When: Configuration is accessed
    //! Then: Default values match HTTP/2 specification

    // When
    let config = Http2Config::default();

    // Then
    assert_eq!(config.max_concurrent_streams(), 100);
    assert_eq!(config.initial_window_size(), 65_535);
    assert_eq!(config.max_frame_size(), 16_384);
    assert!(!config.enable_push());
}

#[test]
fn test_http2_config_custom_values() {
    //! Given: Http2Config is created with custom values
    //! When: Configuration is accessed
    //! Then: Custom values are returned

    // Given
    let config = Http2Config::new()
        .with_max_concurrent_streams(200)
        .with_initial_window_size(131_072)
        .with_max_frame_size(32_768)
        .with_push_enabled(true);

    // When/Then
    assert_eq!(config.max_concurrent_streams(), 200);
    assert_eq!(config.initial_window_size(), 131_072);
    assert_eq!(config.max_frame_size(), 32_768);
    assert!(config.enable_push());
}

#[test]
fn test_http2_config_max_concurrent_streams_validation() {
    //! Given: Http2Config is created with invalid max_concurrent_streams
    //! When: Configuration is validated
    //! Then: Returns error for invalid values

    // Given
    let config = Http2Config::new().with_max_concurrent_streams(0);

    // When
    let result = config.validate();

    // Then
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("max_concurrent_streams must be greater than 0"));
}

#[test]
fn test_http2_config_initial_window_size_validation() {
    //! Given: Http2Config with initial_window_size exceeding max
    //! When: Configuration is validated
    //! Then: Returns error for window size exceeding 2^31-1

    // Given
    let config = Http2Config::new().with_initial_window_size(u32::MAX);

    // When
    let result = config.validate();

    // Then
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("initial_window_size"));
}

#[test]
fn test_http2_config_max_frame_size_validation() {
    //! Given: Http2Config with invalid max_frame_size
    //! When: Configuration is validated
    //! Then: Returns error for frame size outside valid range

    // Given - too small
    let config_small = Http2Config::new().with_max_frame_size(16_383);

    // When
    let result_small = config_small.validate();

    // Then
    assert!(result_small.is_err());

    // Given - too large
    let config_large = Http2Config::new().with_max_frame_size(16_777_216);

    // When
    let result_large = config_large.validate();

    // Then
    assert!(result_large.is_err());
}

#[test]
fn test_http2_config_builder_pattern() {
    //! Given: Http2Config is built using builder pattern
    //! When: Multiple configurations are chained
    //! Then: All values are correctly set

    // Given/When
    let config = Http2Config::new()
        .with_max_concurrent_streams(150)
        .with_initial_window_size(98_304)
        .with_max_frame_size(24_576)
        .with_push_enabled(false);

    // Then
    assert_eq!(config.max_concurrent_streams(), 150);
    assert_eq!(config.initial_window_size(), 98_304);
    assert_eq!(config.max_frame_size(), 24_576);
    assert!(!config.enable_push());
}

#[test]
fn test_http2_config_clone() {
    //! Given: Http2Config is created
    //! When: Configuration is cloned
    //! Then: Clone has identical values

    // Given
    let config = Http2Config::new()
        .with_max_concurrent_streams(175)
        .with_push_enabled(true);

    // When
    let cloned = config.clone();

    // Then
    assert_eq!(
        config.max_concurrent_streams(),
        cloned.max_concurrent_streams()
    );
    assert_eq!(config.enable_push(), cloned.enable_push());
}
