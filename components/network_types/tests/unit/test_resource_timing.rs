use network_types::ResourceTiming;

#[test]
fn test_resource_timing_creation() {
    // Given timing values for a resource
    // When a ResourceTiming structure is created
    // Then it should contain all timing information
    let timing = ResourceTiming {
        start_time: 0.0,
        redirect_start: 1.0,
        redirect_end: 2.0,
        fetch_start: 3.0,
        domain_lookup_start: 4.0,
        domain_lookup_end: 5.0,
        connect_start: 6.0,
        connect_end: 7.0,
        secure_connection_start: 8.0,
        request_start: 9.0,
        response_start: 10.0,
        response_end: 11.0,
        transfer_size: 1024,
        encoded_body_size: 512,
        decoded_body_size: 768,
    };

    assert_eq!(timing.start_time, 0.0);
    assert_eq!(timing.redirect_start, 1.0);
    assert_eq!(timing.redirect_end, 2.0);
    assert_eq!(timing.fetch_start, 3.0);
    assert_eq!(timing.domain_lookup_start, 4.0);
    assert_eq!(timing.domain_lookup_end, 5.0);
    assert_eq!(timing.connect_start, 6.0);
    assert_eq!(timing.connect_end, 7.0);
    assert_eq!(timing.secure_connection_start, 8.0);
    assert_eq!(timing.request_start, 9.0);
    assert_eq!(timing.response_start, 10.0);
    assert_eq!(timing.response_end, 11.0);
    assert_eq!(timing.transfer_size, 1024);
    assert_eq!(timing.encoded_body_size, 512);
    assert_eq!(timing.decoded_body_size, 768);
}

#[test]
fn test_resource_timing_debug() {
    // Given a ResourceTiming structure
    // When debug formatted
    // Then it should produce readable output
    let timing = ResourceTiming {
        start_time: 0.0,
        redirect_start: 0.0,
        redirect_end: 0.0,
        fetch_start: 0.0,
        domain_lookup_start: 0.0,
        domain_lookup_end: 0.0,
        connect_start: 0.0,
        connect_end: 0.0,
        secure_connection_start: 0.0,
        request_start: 0.0,
        response_start: 0.0,
        response_end: 0.0,
        transfer_size: 0,
        encoded_body_size: 0,
        decoded_body_size: 0,
    };

    let debug_str = format!("{:?}", timing);
    assert!(debug_str.contains("ResourceTiming"));
}

#[test]
fn test_resource_timing_clone() {
    // Given a ResourceTiming structure
    // When cloned
    // Then the clone should have same values
    let timing = ResourceTiming {
        start_time: 1.5,
        redirect_start: 2.0,
        redirect_end: 3.0,
        fetch_start: 4.0,
        domain_lookup_start: 5.0,
        domain_lookup_end: 6.0,
        connect_start: 7.0,
        connect_end: 8.0,
        secure_connection_start: 9.0,
        request_start: 10.0,
        response_start: 11.0,
        response_end: 12.0,
        transfer_size: 2048,
        encoded_body_size: 1024,
        decoded_body_size: 1536,
    };

    let cloned = timing.clone();
    assert_eq!(cloned.start_time, timing.start_time);
    assert_eq!(cloned.transfer_size, timing.transfer_size);
    assert_eq!(cloned.encoded_body_size, timing.encoded_body_size);
}

#[test]
fn test_resource_timing_serde() {
    // Given a ResourceTiming structure
    // When serialized to JSON
    // Then it should serialize all fields
    // And deserialization should restore the structure
    let timing = ResourceTiming {
        start_time: 1.0,
        redirect_start: 2.0,
        redirect_end: 3.0,
        fetch_start: 4.0,
        domain_lookup_start: 5.0,
        domain_lookup_end: 6.0,
        connect_start: 7.0,
        connect_end: 8.0,
        secure_connection_start: 9.0,
        request_start: 10.0,
        response_start: 11.0,
        response_end: 12.0,
        transfer_size: 1024,
        encoded_body_size: 512,
        decoded_body_size: 768,
    };

    let json = serde_json::to_string(&timing).unwrap();
    let deserialized: ResourceTiming = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.start_time, timing.start_time);
    assert_eq!(deserialized.transfer_size, timing.transfer_size);
}
