//! Data URL parsing and handling
//!
//! Implements RFC 2397 data URLs with support for:
//! - Plain text data
//! - Base64 encoded data
//! - MIME type extraction
//! - Character set detection

use network_errors::NetworkError;

/// Data URL parsed data structure
///
/// Contains the parsed components of a data URL including MIME type,
/// character set, and decoded data.
#[derive(Debug, Clone, PartialEq)]
pub struct DataUrlData {
    /// MIME type of the data (e.g., "text/plain", "image/png")
    pub mime_type: String,
    /// Decoded binary data
    pub data: Vec<u8>,
    /// Character set if specified (e.g., "utf-8")
    pub charset: Option<String>,
}

/// Data URL handler
///
/// Provides parsing and validation for data: URLs according to RFC 2397.
#[derive(Debug, Clone, Copy)]
pub struct DataUrlHandler;

impl DataUrlHandler {
    /// Check if a URL is a data URL
    ///
    /// # Examples
    ///
    /// ```
    /// use url_handlers::DataUrlHandler;
    ///
    /// assert!(DataUrlHandler::is_data_url("data:text/plain,Hello"));
    /// assert!(!DataUrlHandler::is_data_url("http://example.com"));
    /// ```
    pub fn is_data_url(url: &str) -> bool {
        url.starts_with("data:")
    }

    /// Parse a data URL into its components
    ///
    /// Supports both plain text and base64 encoded data.
    /// Default MIME type is "text/plain" if not specified.
    ///
    /// # Arguments
    ///
    /// * `url` - The data URL to parse
    ///
    /// # Returns
    ///
    /// * `Ok(DataUrlData)` - Successfully parsed data
    /// * `Err(NetworkError)` - Invalid URL format or decoding error
    ///
    /// # Examples
    ///
    /// ```
    /// use url_handlers::DataUrlHandler;
    ///
    /// // Plain text
    /// let data = DataUrlHandler::parse("data:text/plain,Hello").unwrap();
    /// assert_eq!(data.mime_type, "text/plain");
    /// assert_eq!(String::from_utf8(data.data).unwrap(), "Hello");
    ///
    /// // Base64 encoded
    /// let data = DataUrlHandler::parse("data:text/plain;base64,SGVsbG8=").unwrap();
    /// assert_eq!(String::from_utf8(data.data).unwrap(), "Hello");
    /// ```
    pub fn parse(url: &str) -> Result<DataUrlData, NetworkError> {
        // Verify it's a data URL
        if !Self::is_data_url(url) {
            return Err(NetworkError::InvalidUrl(
                "Not a data URL (must start with 'data:')".to_string(),
            ));
        }

        // Strip "data:" prefix
        let url = &url[5..];

        // Find the comma separator
        let comma_pos = url.find(',').ok_or_else(|| {
            NetworkError::InvalidUrl("Data URL missing comma separator".to_string())
        })?;

        // Split into metadata and data
        let metadata = &url[..comma_pos];
        let data_part = &url[comma_pos + 1..];

        // Parse metadata (MIME type, charset, base64 flag)
        let (mime_type, charset, is_base64) = Self::parse_metadata(metadata);

        // Decode data
        let decoded_data = if is_base64 {
            Self::decode_base64(data_part)?
        } else {
            Self::decode_plain(data_part)?
        };

        Ok(DataUrlData {
            mime_type,
            data: decoded_data,
            charset,
        })
    }

    /// Parse metadata section of data URL
    ///
    /// Returns (mime_type, charset, is_base64)
    fn parse_metadata(metadata: &str) -> (String, Option<String>, bool) {
        if metadata.is_empty() {
            return ("text/plain".to_string(), None, false);
        }

        let parts: Vec<&str> = metadata.split(';').collect();
        let mut mime_type = "text/plain".to_string();
        let mut charset = None;
        let mut is_base64 = false;

        for (i, part) in parts.iter().enumerate() {
            let part = part.trim();
            if i == 0 && !part.is_empty() && !part.starts_with("charset=") && part != "base64" {
                // First part is MIME type
                mime_type = part.to_string();
            } else if let Some(stripped) = part.strip_prefix("charset=") {
                // Character set
                charset = Some(stripped.to_string());
            } else if part == "base64" {
                // Base64 encoding flag
                is_base64 = true;
            }
        }

        (mime_type, charset, is_base64)
    }

    /// Decode base64 encoded data
    fn decode_base64(data: &str) -> Result<Vec<u8>, NetworkError> {
        use base64::{engine::general_purpose::STANDARD, Engine};

        STANDARD
            .decode(data)
            .map_err(|e| NetworkError::InvalidUrl(format!("Invalid base64 encoding: {}", e)))
    }

    /// Decode plain (URL-encoded) data
    fn decode_plain(data: &str) -> Result<Vec<u8>, NetworkError> {
        // URL decode the data
        let decoded = Self::url_decode(data)?;
        Ok(decoded.into_bytes())
    }

    /// URL decode a string
    fn url_decode(s: &str) -> Result<String, NetworkError> {
        let mut result = String::new();
        let mut chars = s.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '%' {
                // Decode percent-encoded character
                let hex: String = chars.by_ref().take(2).collect();
                if hex.len() != 2 {
                    return Err(NetworkError::InvalidUrl(
                        "Invalid percent encoding".to_string(),
                    ));
                }
                let byte = u8::from_str_radix(&hex, 16).map_err(|_| {
                    NetworkError::InvalidUrl("Invalid percent encoding hex value".to_string())
                })?;
                result.push(byte as char);
            } else if ch == '+' {
                // Plus signs decode to spaces
                result.push(' ');
            } else {
                result.push(ch);
            }
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_data_url() {
        assert!(DataUrlHandler::is_data_url("data:text/plain,test"));
        assert!(!DataUrlHandler::is_data_url("http://example.com"));
    }

    #[test]
    fn test_parse_plain_text() {
        let data = DataUrlHandler::parse("data:text/plain,Hello").unwrap();
        assert_eq!(data.mime_type, "text/plain");
        assert_eq!(String::from_utf8(data.data).unwrap(), "Hello");
    }

    #[test]
    fn test_parse_base64() {
        let data = DataUrlHandler::parse("data:text/plain;base64,SGVsbG8=").unwrap();
        assert_eq!(data.mime_type, "text/plain");
        assert_eq!(String::from_utf8(data.data).unwrap(), "Hello");
    }

    #[test]
    fn test_parse_with_charset() {
        let data = DataUrlHandler::parse("data:text/plain;charset=utf-8,Hello").unwrap();
        assert_eq!(data.charset, Some("utf-8".to_string()));
    }

    #[test]
    fn test_parse_invalid_url() {
        assert!(DataUrlHandler::parse("http://example.com").is_err());
    }

    #[test]
    fn test_url_decode() {
        let result = DataUrlHandler::url_decode("Hello%20World").unwrap();
        assert_eq!(result, "Hello World");

        let result = DataUrlHandler::url_decode("Hello+World").unwrap();
        assert_eq!(result, "Hello World");
    }
}
