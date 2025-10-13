use log::warn;

/// Trim trailing slashes from a path and warn if any were removed
/// This utility consolidates the duplicated path trimming logic from config route operations
pub fn trim_trailing_slash(path: String) -> String {
    if path.ends_with('/') {
        let trimmed = path.trim_end_matches('/').to_string();
        warn!("Path should not end with '/', will be stripped: {}", trimmed);
        trimmed
    } else {
        path
    }
}

/// Validate that a path doesn't end with slash and return the cleaned path
pub fn validate_and_clean_path(path: String) -> String {
    trim_trailing_slash(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trim_trailing_slash_with_slash() {
        assert_eq!(trim_trailing_slash("/api/v1/".to_string()), "/api/v1");
        assert_eq!(trim_trailing_slash("/test/".to_string()), "/test");
        assert_eq!(trim_trailing_slash("/".to_string()), "");
        assert_eq!(trim_trailing_slash("path///".to_string()), "path");
    }

    #[test]
    fn test_trim_trailing_slash_without_slash() {
        assert_eq!(trim_trailing_slash("/api/v1".to_string()), "/api/v1");
        assert_eq!(trim_trailing_slash("/test".to_string()), "/test");
        assert_eq!(trim_trailing_slash("path".to_string()), "path");
        assert_eq!(trim_trailing_slash("".to_string()), "");
    }

    #[test]
    fn test_validate_and_clean_path() {
        assert_eq!(validate_and_clean_path("/api/v1/".to_string()), "/api/v1");
        assert_eq!(validate_and_clean_path("/api/v1".to_string()), "/api/v1");
        assert_eq!(validate_and_clean_path("".to_string()), "");
    }
}