//! FTP response parsing

use network_errors::{NetworkError, NetworkResult};

/// FTP response structure
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FtpResponse {
    /// Response code (e.g., 220, 530)
    pub code: u16,
    /// Response message
    pub message: String,
    /// Whether this is a multi-line response
    pub is_multiline: bool,
}

impl FtpResponse {
    /// Check if response indicates success (2xx)
    pub fn is_success(&self) -> bool {
        self.code >= 200 && self.code < 300
    }

    /// Check if response indicates error (4xx or 5xx)
    pub fn is_error(&self) -> bool {
        self.code >= 400
    }

    /// Check if response is intermediate (3xx)
    pub fn is_intermediate(&self) -> bool {
        self.code >= 300 && self.code < 400
    }

    /// Check if response indicates continuation (1xx)
    pub fn is_continuation(&self) -> bool {
        self.code >= 100 && self.code < 200
    }
}

/// Parse FTP response from raw data
///
/// FTP responses follow the format:
/// - Single line: "CODE MESSAGE\r\n"
/// - Multi-line: "CODE-LINE1\r\nCODE-LINE2\r\nCODE MESSAGE\r\n"
pub fn parse_response(data: &str) -> NetworkResult<FtpResponse> {
    let lines: Vec<&str> = data.lines().collect();

    if lines.is_empty() {
        return Err(NetworkError::ProtocolError("Empty FTP response".to_string()));
    }

    let first_line = lines[0];

    // Parse response code (first 3 characters)
    if first_line.len() < 3 {
        return Err(NetworkError::ProtocolError("Invalid FTP response format".to_string()));
    }

    let code_str = &first_line[0..3];
    let code = code_str.parse::<u16>()
        .map_err(|_| NetworkError::ProtocolError("Invalid FTP response code".to_string()))?;

    // Check if multi-line (has '-' after code)
    let is_multiline = first_line.len() > 3 && first_line.chars().nth(3) == Some('-');

    // Extract message
    let message = if is_multiline {
        // Multi-line: collect all lines
        let mut msg_lines = Vec::new();
        for line in &lines {
            if line.len() > 4 {
                msg_lines.push(&line[4..]);
            }
        }
        msg_lines.join("\n")
    } else {
        // Single line: get text after space
        if first_line.len() > 4 {
            first_line[4..].to_string()
        } else {
            String::new()
        }
    };

    Ok(FtpResponse {
        code,
        message,
        is_multiline,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_response_classification() {
        assert!(FtpResponse { code: 220, message: "".into(), is_multiline: false }.is_success());
        assert!(FtpResponse { code: 331, message: "".into(), is_multiline: false }.is_intermediate());
        assert!(FtpResponse { code: 530, message: "".into(), is_multiline: false }.is_error());
        assert!(FtpResponse { code: 150, message: "".into(), is_multiline: false }.is_continuation());
    }
}
