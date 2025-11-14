//! System proxy configuration detection

use network_errors::NetworkError;
use std::env;

/// System proxy configuration
///
/// Contains proxy settings read from environment variables.
#[derive(Debug, Clone, PartialEq)]
pub struct SystemProxyConfig {
    /// Whether proxy is enabled
    pub enabled: bool,
    /// HTTP proxy URL (from HTTP_PROXY environment variable)
    pub http_proxy: Option<String>,
    /// HTTPS proxy URL (from HTTPS_PROXY environment variable)
    pub https_proxy: Option<String>,
    /// Domains to bypass proxy (from NO_PROXY environment variable)
    pub no_proxy: Vec<String>,
}

/// Get system proxy configuration from environment variables
///
/// Reads the following environment variables:
/// - HTTP_PROXY: HTTP proxy server URL
/// - HTTPS_PROXY: HTTPS proxy server URL
/// - NO_PROXY: Comma-separated list of domains to bypass proxy
///
/// # Returns
///
/// Returns a `SystemProxyConfig` with proxy settings. If no proxy environment
/// variables are set, returns a disabled configuration.
pub fn get_system_proxy_config() -> Result<SystemProxyConfig, NetworkError> {
    let http_proxy = get_env_var("HTTP_PROXY");
    let https_proxy = get_env_var("HTTPS_PROXY");
    let no_proxy_str = env::var("NO_PROXY").ok();

    let no_proxy = parse_no_proxy(no_proxy_str.as_deref());

    let enabled = http_proxy.is_some() || https_proxy.is_some();

    Ok(SystemProxyConfig {
        enabled,
        http_proxy,
        https_proxy,
        no_proxy,
    })
}

/// Get environment variable value, treating empty strings as None
fn get_env_var(key: &str) -> Option<String> {
    env::var(key).ok().and_then(|v| {
        if v.trim().is_empty() {
            None
        } else {
            Some(v)
        }
    })
}

/// Parse NO_PROXY environment variable
///
/// Splits comma-separated list of domains and trims whitespace.
fn parse_no_proxy(no_proxy: Option<&str>) -> Vec<String> {
    match no_proxy {
        Some(s) if !s.trim().is_empty() => {
            s.split(',')
                .map(|domain| domain.trim().to_string())
                .filter(|domain| !domain.is_empty())
                .collect()
        }
        _ => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_no_proxy_empty() {
        assert_eq!(parse_no_proxy(None), Vec::<String>::new());
        assert_eq!(parse_no_proxy(Some("")), Vec::<String>::new());
        assert_eq!(parse_no_proxy(Some("   ")), Vec::<String>::new());
    }

    #[test]
    fn test_parse_no_proxy_single() {
        assert_eq!(
            parse_no_proxy(Some("localhost")),
            vec!["localhost".to_string()]
        );
    }

    #[test]
    fn test_parse_no_proxy_multiple() {
        assert_eq!(
            parse_no_proxy(Some("localhost,127.0.0.1,.example.com")),
            vec![
                "localhost".to_string(),
                "127.0.0.1".to_string(),
                ".example.com".to_string()
            ]
        );
    }

    #[test]
    fn test_parse_no_proxy_with_spaces() {
        assert_eq!(
            parse_no_proxy(Some("localhost, 127.0.0.1 , .example.com")),
            vec![
                "localhost".to_string(),
                "127.0.0.1".to_string(),
                ".example.com".to_string()
            ]
        );
    }

    #[test]
    fn test_get_env_var_empty_is_none() {
        env::set_var("TEST_EMPTY_VAR", "");
        assert_eq!(get_env_var("TEST_EMPTY_VAR"), None);
        env::remove_var("TEST_EMPTY_VAR");
    }

    #[test]
    fn test_get_env_var_whitespace_is_none() {
        env::set_var("TEST_WHITESPACE_VAR", "   ");
        assert_eq!(get_env_var("TEST_WHITESPACE_VAR"), None);
        env::remove_var("TEST_WHITESPACE_VAR");
    }

    #[test]
    fn test_get_env_var_value() {
        env::set_var("TEST_VALUE_VAR", "some_value");
        assert_eq!(get_env_var("TEST_VALUE_VAR"), Some("some_value".to_string()));
        env::remove_var("TEST_VALUE_VAR");
    }
}
