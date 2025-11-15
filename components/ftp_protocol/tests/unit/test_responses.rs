//! Unit tests for FTP response parsing

use ftp_protocol::responses::{FtpResponse, parse_response};

#[test]
fn test_parse_single_line_response() {
    // Given a single-line FTP response
    let response_data = "220 Service ready\r\n";

    // When parsing the response
    let result = parse_response(response_data);

    // Then it should be parsed correctly
    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.code, 220);
    assert_eq!(response.message, "Service ready");
    assert!(!response.is_multiline);
}

#[test]
fn test_parse_multiline_response() {
    // Given a multi-line FTP response
    let response_data = "220-Welcome to FTP server\r\n220-This is line 2\r\n220 End of message\r\n";

    // When parsing the response
    let result = parse_response(response_data);

    // Then it should be parsed correctly
    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.code, 220);
    assert!(response.is_multiline);
    assert!(response.message.contains("Welcome"));
}

#[test]
fn test_parse_error_response() {
    // Given an error response (5xx)
    let response_data = "530 Not logged in\r\n";

    // When parsing the response
    let result = parse_response(response_data);

    // Then it should be parsed correctly
    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.code, 530);
    assert_eq!(response.message, "Not logged in");
}

#[test]
fn test_response_is_success() {
    // Given a success response (2xx)
    let response = FtpResponse {
        code: 200,
        message: "OK".to_string(),
        is_multiline: false,
    };

    // When checking if it's a success
    let is_success = response.is_success();

    // Then it should be true
    assert!(is_success);
}

#[test]
fn test_response_is_error() {
    // Given an error response (5xx)
    let response = FtpResponse {
        code: 500,
        message: "Error".to_string(),
        is_multiline: false,
    };

    // When checking if it's an error
    let is_error = response.is_error();

    // Then it should be true
    assert!(is_error);
}

#[test]
fn test_response_is_intermediate() {
    // Given an intermediate response (3xx)
    let response = FtpResponse {
        code: 331,
        message: "Password required".to_string(),
        is_multiline: false,
    };

    // When checking if it's intermediate
    let is_intermediate = response.is_intermediate();

    // Then it should be true
    assert!(is_intermediate);
}

#[test]
fn test_parse_invalid_response() {
    // Given an invalid response
    let response_data = "Invalid\r\n";

    // When parsing the response
    let result = parse_response(response_data);

    // Then it should return an error
    assert!(result.is_err());
}
