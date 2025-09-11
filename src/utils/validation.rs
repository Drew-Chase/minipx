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
