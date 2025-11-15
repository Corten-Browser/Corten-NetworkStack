//! Unit tests for FTP command formatting

use ftp_protocol::commands::{format_user, format_pass, format_pasv, format_list, format_retr, format_stor, format_quit};

#[test]
fn test_format_user_command() {
    // Given a username
    let username = "testuser";

    // When formatting USER command
    let command = format_user(username);

    // Then it should be formatted correctly
    assert_eq!(command, "USER testuser\r\n");
}

#[test]
fn test_format_pass_command() {
    // Given a password
    let password = "secret123";

    // When formatting PASS command
    let command = format_pass(password);

    // Then it should be formatted correctly
    assert_eq!(command, "PASS secret123\r\n");
}

#[test]
fn test_format_pasv_command() {
    // When formatting PASV command
    let command = format_pasv();

    // Then it should be formatted correctly
    assert_eq!(command, "PASV\r\n");
}

#[test]
fn test_format_list_command_no_path() {
    // Given no path
    let path = None;

    // When formatting LIST command
    let command = format_list(path);

    // Then it should be formatted correctly
    assert_eq!(command, "LIST\r\n");
}

#[test]
fn test_format_list_command_with_path() {
    // Given a path
    let path = Some("/home/user");

    // When formatting LIST command
    let command = format_list(path);

    // Then it should be formatted correctly
    assert_eq!(command, "LIST /home/user\r\n");
}

#[test]
fn test_format_retr_command() {
    // Given a filename
    let filename = "file.txt";

    // When formatting RETR command
    let command = format_retr(filename);

    // Then it should be formatted correctly
    assert_eq!(command, "RETR file.txt\r\n");
}

#[test]
fn test_format_stor_command() {
    // Given a filename
    let filename = "upload.txt";

    // When formatting STOR command
    let command = format_stor(filename);

    // Then it should be formatted correctly
    assert_eq!(command, "STOR upload.txt\r\n");
}

#[test]
fn test_format_quit_command() {
    // When formatting QUIT command
    let command = format_quit();

    // Then it should be formatted correctly
    assert_eq!(command, "QUIT\r\n");
}
