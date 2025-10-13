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