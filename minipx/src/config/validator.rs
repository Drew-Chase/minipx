use crate::config::types::Config;
use crate::utils::validation::validate_hostname_chars;
use std::collections::BTreeSet;

impl Config {
    /// Check if SSL is enabled for any route
    /// FIXED: Previously always returned true - now properly checks routes
    pub fn is_ssl_enabled(&self) -> bool {
        for route in self.routes.values() {
            if route.is_ssl_enabled() {
                return true;
            }
        }
        false // Fixed: was previously hardcoded to return true
    }

    /// Validate email address format
    pub fn is_email_valid(&self) -> bool {
        let email = self.get_email();
        // very simple validation: one '@', no spaces, local and domain parts non-empty, domain contains '.'
        if email.is_empty() || email.contains(' ') {
            return false;
        }
        let parts: Vec<&str> = email.split('@').collect();
        if parts.len() != 2 {
            return false;
        }
        let (local, domain) = (parts[0], parts[1]);
        if local.is_empty() || domain.is_empty() {
            return false;
        }
        if !domain.contains('.') {
            return false;
        }
        // ensure domain is valid-ish
        Self::validate_domain(domain)
    }

    /// Validate domain name format for ACME certificate requests
    pub fn validate_domain(domain: &str) -> bool {
        // Disallow wildcard entries here; we cannot get wildcard certs with TLS-ALPN/HTTP-01
        if domain.starts_with("*.") {
            return false;
        }
        if domain.len() > 253 || !domain.contains('.') {
            return false;
        }
        // Only allow a-z, A-Z, 0-9, '-', '.'; labels 1..=63, cannot start/end with '-'
        if !validate_hostname_chars(domain) {
            return false;
        }
        if domain.ends_with('.') {
            return false;
        } // cannot end with a dot
        for label in domain.split('.') {
            if label.is_empty() || label.len() > 63 {
                return false;
            }
            if label.starts_with('-') || label.ends_with('-') {
                return false;
            }
        }
        true
    }

    /// Returns (valid_domains, invalid_domains) for ACME based on current routes.
    pub fn get_valid_domains_for_acme(&self) -> (Vec<String>, Vec<String>) {
        let mut valid_set: BTreeSet<String> = BTreeSet::new();
        let mut invalid: Vec<String> = Vec::new();
        for (domain, route) in &self.routes {
            if domain.starts_with("*.") {
                invalid.push(domain.clone());
                continue;
            }
            // Only consider routes that intend to serve HTTPS at the frontend
            if !route.is_ssl_enabled() {
                continue; // neither valid nor invalid; just not used for ACME
            }
            if Self::validate_domain(domain) {
                valid_set.insert(domain.clone());
            } else {
                invalid.push(domain.clone());
            }
        }
        (valid_set.into_iter().collect(), invalid)
    }

    /// True if this config can serve TLS for the specific host.
    pub fn can_serve_tls_for_host(&self, host: &str) -> bool {
        if !self.is_ssl_enabled() || !self.is_email_valid() {
            return false;
        }
        // Route must exist and be configured for HTTPS at the frontend
        if let Some(route) = self.lookup_host(host) {
            if !route.is_ssl_enabled() {
                return false;
            }
        } else {
            return false;
        }
        let (valid, _invalid) = self.get_valid_domains_for_acme();
        valid.iter().any(|d| d == host)
    }
}

#[cfg(test)]
mod tests {
    use crate::config::types::{Config, ProxyRoute};

    #[test]
    fn test_is_ssl_enabled_no_routes() {
        let config = Config::default();
        assert!(!config.is_ssl_enabled());
    }

    #[test]
    fn test_is_ssl_enabled_with_ssl_route() {
        let mut config = Config::default();
        config.routes.insert(
            "example.com".to_string(),
            ProxyRoute::new(
                "localhost".to_string(),
                "/api".to_string(),
                8080,
                true, // SSL enabled
                None,
                false,
            ),
        );
        assert!(config.is_ssl_enabled());
    }

    #[test]
    fn test_is_ssl_enabled_without_ssl_route() {
        let mut config = Config::default();
        config.routes.insert(
            "example.com".to_string(),
            ProxyRoute::new(
                "localhost".to_string(),
                "/api".to_string(),
                8080,
                false, // SSL disabled
                None,
                false,
            ),
        );
        assert!(!config.is_ssl_enabled());
    }

    #[test]
    fn test_is_email_valid() {
        let mut config = Config::default();

        // Invalid emails
        config.set_email("".to_string());
        assert!(!config.is_email_valid());

        config.set_email("invalid".to_string());
        assert!(!config.is_email_valid());

        config.set_email("test@".to_string());
        assert!(!config.is_email_valid());

        config.set_email("@example.com".to_string());
        assert!(!config.is_email_valid());

        config.set_email("test @example.com".to_string());
        assert!(!config.is_email_valid());

        config.set_email("test@invalid".to_string());
        assert!(!config.is_email_valid());

        // Valid emails
        config.set_email("test@example.com".to_string());
        assert!(config.is_email_valid());

        config.set_email("admin@sub.example.com".to_string());
        assert!(config.is_email_valid());

        config.set_email("user.name+tag@example.co.uk".to_string());
        assert!(config.is_email_valid());
    }

    #[test]
    fn test_validate_domain_valid() {
        assert!(Config::validate_domain("example.com"));
        assert!(Config::validate_domain("sub.example.com"));
        assert!(Config::validate_domain("api.v2.example.com"));
        assert!(Config::validate_domain("test-123.example.com"));
        assert!(Config::validate_domain("a.b.c.d.example.com"));
    }

    #[test]
    fn test_validate_domain_invalid() {
        // Wildcards not allowed for ACME
        assert!(!Config::validate_domain("*.example.com"));

        // No dot (must be FQDN-like)
        assert!(!Config::validate_domain("localhost"));

        // Ends with dot
        assert!(!Config::validate_domain("example.com."));

        // Empty
        assert!(!Config::validate_domain(""));

        // Too long label (>63 chars)
        assert!(!Config::validate_domain(&format!("{}.com", "a".repeat(64))));

        // Label starts/ends with dash
        assert!(!Config::validate_domain("-example.com"));
        assert!(!Config::validate_domain("example-.com"));

        // Invalid characters
        assert!(!Config::validate_domain("exam_ple.com"));
        assert!(!Config::validate_domain("exam ple.com"));
    }

    #[test]
    fn test_get_valid_domains_for_acme() {
        let mut config = Config::default();
        config.set_email("admin@example.com".to_string());

        // Add valid SSL-enabled route
        config.routes.insert("api.example.com".to_string(), ProxyRoute::new("localhost".to_string(), "/api".to_string(), 8080, true, None, false));

        // Add wildcard route (should be invalid for ACME)
        config.routes.insert("*.example.com".to_string(), ProxyRoute::new("localhost".to_string(), "/".to_string(), 8080, true, None, false));

        // Add non-SSL route (should be ignored)
        config.routes.insert("nossl.example.com".to_string(), ProxyRoute::new("localhost".to_string(), "/".to_string(), 8080, false, None, false));

        // Add invalid domain
        config.routes.insert("localhost".to_string(), ProxyRoute::new("localhost".to_string(), "/".to_string(), 8080, true, None, false));

        let (valid, invalid) = config.get_valid_domains_for_acme();

        assert_eq!(valid.len(), 1);
        assert!(valid.contains(&"api.example.com".to_string()));

        assert_eq!(invalid.len(), 2);
        assert!(invalid.contains(&"*.example.com".to_string()));
        assert!(invalid.contains(&"localhost".to_string()));
    }

    #[test]
    fn test_can_serve_tls_for_host() {
        let mut config = Config::default();
        config.set_email("admin@example.com".to_string());

        // Add SSL-enabled route
        config.routes.insert("api.example.com".to_string(), ProxyRoute::new("localhost".to_string(), "/api".to_string(), 8080, true, None, false));

        assert!(config.can_serve_tls_for_host("api.example.com"));
        assert!(!config.can_serve_tls_for_host("other.example.com"));

        // Disable SSL
        config.routes.get_mut("api.example.com").unwrap().ssl_enable = false;
        assert!(!config.can_serve_tls_for_host("api.example.com"));
    }
}
