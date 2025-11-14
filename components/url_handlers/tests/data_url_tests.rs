//! Unit tests for data URL parsing

use url_handlers::DataUrlHandler;

#[test]
fn test_is_data_url() {
    assert!(DataUrlHandler::is_data_url("data:text/plain,Hello"));
    assert!(DataUrlHandler::is_data_url("data:image/png;base64,ABC"));
    assert!(!DataUrlHandler::is_data_url("http://example.com"));
    assert!(!DataUrlHandler::is_data_url("file:///path"));
}

#[test]
fn test_parse_data_url_with_base64() {
    let url = "data:text/plain;base64,SGVsbG8gV29ybGQ=";
    let result = DataUrlHandler::parse(url);
    assert!(result.is_ok(), "Base64 data URL should parse");

    let data = result.unwrap();
    assert_eq!(data.mime_type, "text/plain");
    assert_eq!(String::from_utf8(data.data).unwrap(), "Hello World");
    assert_eq!(data.charset, None);
}

#[test]
fn test_parse_data_url_plain_text() {
    let url = "data:text/plain,Hello%20World";
    let result = DataUrlHandler::parse(url);
    assert!(result.is_ok(), "Plain text data URL should parse");

    let data = result.unwrap();
    assert_eq!(data.mime_type, "text/plain");
    assert_eq!(String::from_utf8(data.data).unwrap(), "Hello World");
}

#[test]
fn test_parse_data_url_with_charset() {
    let url = "data:text/plain;charset=utf-8;base64,SGVsbG8gV29ybGQ=";
    let result = DataUrlHandler::parse(url);
    assert!(result.is_ok(), "Data URL with charset should parse");

    let data = result.unwrap();
    assert_eq!(data.mime_type, "text/plain");
    assert_eq!(data.charset, Some("utf-8".to_string()));
    assert_eq!(String::from_utf8(data.data).unwrap(), "Hello World");
}

#[test]
fn test_parse_data_url_default_mime_type() {
    let url = "data:,Hello";
    let result = DataUrlHandler::parse(url);
    assert!(
        result.is_ok(),
        "Data URL with default MIME type should parse"
    );

    let data = result.unwrap();
    // Default MIME type is "text/plain" (charset "US-ASCII" is implicit per RFC 2397)
    assert_eq!(data.mime_type, "text/plain");
    assert_eq!(String::from_utf8(data.data).unwrap(), "Hello");
}

#[test]
fn test_parse_data_url_custom_mime_type() {
    let url = "data:application/json;base64,eyJ0ZXN0IjoidmFsdWUifQ==";
    let result = DataUrlHandler::parse(url);
    assert!(
        result.is_ok(),
        "Data URL with custom MIME type should parse"
    );

    let data = result.unwrap();
    assert_eq!(data.mime_type, "application/json");
    assert_eq!(String::from_utf8(data.data).unwrap(), r#"{"test":"value"}"#);
}

#[test]
fn test_parse_invalid_data_url() {
    let url = "http://example.com";
    let result = DataUrlHandler::parse(url);
    assert!(result.is_err(), "HTTP URL should fail to parse as data URL");
}

#[test]
fn test_parse_data_url_missing_comma() {
    let url = "data:text/plainHello";
    let result = DataUrlHandler::parse(url);
    assert!(result.is_err(), "Data URL without comma should fail");
}

#[test]
fn test_parse_data_url_invalid_base64() {
    let url = "data:text/plain;base64,!!!invalid!!!";
    let result = DataUrlHandler::parse(url);
    assert!(result.is_err(), "Invalid base64 should fail");
}

#[test]
fn test_parse_data_url_with_special_characters() {
    let url = "data:text/plain,Hello%20%26%20Goodbye";
    let result = DataUrlHandler::parse(url);
    assert!(
        result.is_ok(),
        "Data URL with URL-encoded chars should parse"
    );

    let data = result.unwrap();
    assert_eq!(String::from_utf8(data.data).unwrap(), "Hello & Goodbye");
}

#[test]
fn test_parse_empty_data_url() {
    let url = "data:,";
    let result = DataUrlHandler::parse(url);
    assert!(result.is_ok(), "Empty data URL should parse");

    let data = result.unwrap();
    assert_eq!(data.data, b"");
}

#[test]
fn test_parse_data_url_binary() {
    // Simple binary data (3 bytes: 0x01, 0x02, 0x03)
    let url = "data:application/octet-stream;base64,AQID";
    let result = DataUrlHandler::parse(url);
    assert!(result.is_ok(), "Binary data URL should parse");

    let data = result.unwrap();
    assert_eq!(data.mime_type, "application/octet-stream");
    assert_eq!(data.data, vec![1, 2, 3]);
}
