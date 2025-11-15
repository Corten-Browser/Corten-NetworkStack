//! FTP command formatting

/// Format USER command
pub fn format_user(username: &str) -> String {
    format!("USER {}\r\n", username)
}

/// Format PASS command
pub fn format_pass(password: &str) -> String {
    format!("PASS {}\r\n", password)
}

/// Format PASV command (passive mode)
pub fn format_pasv() -> String {
    "PASV\r\n".to_string()
}

/// Format PORT command (active mode)
pub fn format_port(address: &str) -> String {
    format!("PORT {}\r\n", address)
}

/// Format LIST command
pub fn format_list(path: Option<&str>) -> String {
    match path {
        Some(p) => format!("LIST {}\r\n", p),
        None => "LIST\r\n".to_string(),
    }
}

/// Format RETR command (download)
pub fn format_retr(filename: &str) -> String {
    format!("RETR {}\r\n", filename)
}

/// Format STOR command (upload)
pub fn format_stor(filename: &str) -> String {
    format!("STOR {}\r\n", filename)
}

/// Format QUIT command
pub fn format_quit() -> String {
    "QUIT\r\n".to_string()
}

/// Format TYPE command (set transfer type)
pub fn format_type(type_code: &str) -> String {
    format!("TYPE {}\r\n", type_code)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_formatting() {
        assert_eq!(format_user("test"), "USER test\r\n");
        assert_eq!(format_pass("pwd"), "PASS pwd\r\n");
        assert_eq!(format_pasv(), "PASV\r\n");
        assert_eq!(format_quit(), "QUIT\r\n");
        assert_eq!(format_list(None), "LIST\r\n");
        assert_eq!(format_list(Some("/path")), "LIST /path\r\n");
    }
}
