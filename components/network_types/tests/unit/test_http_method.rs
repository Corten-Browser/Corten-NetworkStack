use network_types::HttpMethod;

#[test]
fn test_http_method_variants_exist() {
    // Given the HTTP method enum
    // When all standard HTTP methods are defined
    // Then each method variant should be accessible
    let _ = HttpMethod::Get;
    let _ = HttpMethod::Head;
    let _ = HttpMethod::Post;
    let _ = HttpMethod::Put;
    let _ = HttpMethod::Delete;
    let _ = HttpMethod::Connect;
    let _ = HttpMethod::Options;
    let _ = HttpMethod::Trace;
    let _ = HttpMethod::Patch;
}

#[test]
fn test_http_method_debug() {
    // Given an HTTP method
    // When debug formatted
    // Then it should produce readable output
    let method = HttpMethod::Get;
    let debug_str = format!("{:?}", method);
    assert!(debug_str.contains("Get"));
}

#[test]
fn test_http_method_clone() {
    // Given an HTTP method
    // When cloned
    // Then the clone should equal the original
    let method = HttpMethod::Post;
    let cloned = method;
    assert_eq!(method, cloned);
}

#[test]
fn test_http_method_copy() {
    // Given an HTTP method
    // When copied
    // Then both values should be usable independently
    let method1 = HttpMethod::Get;
    let method2 = method1; // Copy trait allows this
    assert_eq!(method1, method2);
}

#[test]
fn test_http_method_partial_eq() {
    // Given two HTTP methods
    // When compared for equality
    // Then same variants should be equal
    // And different variants should not be equal
    assert_eq!(HttpMethod::Get, HttpMethod::Get);
    assert_eq!(HttpMethod::Post, HttpMethod::Post);
    assert_ne!(HttpMethod::Get, HttpMethod::Post);
    assert_ne!(HttpMethod::Put, HttpMethod::Delete);
}

#[test]
fn test_http_method_all_variants() {
    // Given all HTTP method variants
    // When checking they are distinct
    // Then each should be unique
    let methods = [
        HttpMethod::Get,
        HttpMethod::Head,
        HttpMethod::Post,
        HttpMethod::Put,
        HttpMethod::Delete,
        HttpMethod::Connect,
        HttpMethod::Options,
        HttpMethod::Trace,
        HttpMethod::Patch,
    ];

    // Verify we have 9 methods
    assert_eq!(methods.len(), 9);

    // Verify each is distinct
    for (i, method1) in methods.iter().enumerate() {
        for (j, method2) in methods.iter().enumerate() {
            if i == j {
                assert_eq!(method1, method2);
            } else {
                assert_ne!(method1, method2);
            }
        }
    }
}
