//! Common validation utilities shared across modules

/// Validate that a port number is in valid range (1-65535) and not restricted (80, 443)
pub fn validate_custom_port(port: u16) -> Result<(), String> {
    if port == 0 {
        return Err("Port must be between 1 and 65535".to_string());
    }
    if port == 80 || port == 443 {
        return Err("Port cannot be 80 or 443 (reserved for HTTP/HTTPS)".to_string());
    }
    Ok(())
}

/// Validate that a port number is in valid range (1-65535)
pub fn validate_port_range(port: u16) -> Result<(), String> {
    if port == 0 {
        return Err("Port must be between 1 and 65535".to_string());
    }
    Ok(())
}

/// Check if a string is empty or only whitespace
pub fn is_empty_or_whitespace(s: &str) -> bool {
    s.trim().is_empty()
}

/// Validate that a hostname/domain doesn't contain invalid characters
pub fn validate_hostname_chars(hostname: &str) -> bool {
    !hostname.is_empty()
        && hostname.chars().all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '*')
        && !hostname.starts_with('-')
        && !hostname.ends_with('-')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_custom_port_valid() {
        assert!(validate_custom_port(8080).is_ok());
        assert!(validate_custom_port(3000).is_ok());
        assert!(validate_custom_port(9090).is_ok());
        assert!(validate_custom_port(1).is_ok());
        assert!(validate_custom_port(65535).is_ok());
    }

    #[test]
    fn test_validate_custom_port_reserved() {
        assert!(validate_custom_port(80).is_err());
        assert!(validate_custom_port(443).is_err());
        assert_eq!(
            validate_custom_port(80).unwrap_err(),
            "Port cannot be 80 or 443 (reserved for HTTP/HTTPS)"
        );
    }

    #[test]
    fn test_validate_custom_port_invalid() {
        assert!(validate_custom_port(0).is_err());
        assert_eq!(
            validate_custom_port(0).unwrap_err(),
            "Port must be between 1 and 65535"
        );
    }

    #[test]
    fn test_validate_port_range_valid() {
        assert!(validate_port_range(1).is_ok());
        assert!(validate_port_range(80).is_ok());
        assert!(validate_port_range(443).is_ok());
        assert!(validate_port_range(8080).is_ok());
        assert!(validate_port_range(65535).is_ok());
    }

    #[test]
    fn test_validate_port_range_invalid() {
        assert!(validate_port_range(0).is_err());
        assert_eq!(
            validate_port_range(0).unwrap_err(),
            "Port must be between 1 and 65535"
        );
    }

    #[test]
    fn test_is_empty_or_whitespace() {
        assert!(is_empty_or_whitespace(""));
        assert!(is_empty_or_whitespace("   "));
        assert!(is_empty_or_whitespace("\t"));
        assert!(is_empty_or_whitespace("\n"));
        assert!(is_empty_or_whitespace("  \t\n  "));
        assert!(!is_empty_or_whitespace("hello"));
        assert!(!is_empty_or_whitespace("  hello  "));
    }

    #[test]
    fn test_validate_hostname_chars_valid() {
        assert!(validate_hostname_chars("example.com"));
        assert!(validate_hostname_chars("sub.example.com"));
        assert!(validate_hostname_chars("*.example.com"));
        assert!(validate_hostname_chars("api-v2.example.com"));
        assert!(validate_hostname_chars("123.456.789.com"));
        assert!(validate_hostname_chars("a"));
    }

    #[test]
    fn test_validate_hostname_chars_invalid() {
        assert!(!validate_hostname_chars(""));
        assert!(!validate_hostname_chars("-example.com")); // starts with dash
        assert!(!validate_hostname_chars("example.com-")); // ends with dash
        assert!(!validate_hostname_chars("exam ple.com")); // contains space
        assert!(!validate_hostname_chars("exam_ple.com")); // contains underscore
        assert!(!validate_hostname_chars("exam@ple.com")); // contains @
    }
}
