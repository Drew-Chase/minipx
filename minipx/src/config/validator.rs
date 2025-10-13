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