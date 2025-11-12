//! Pre-built security rules based on OWASP Top 10
//!
//! US-15.7: Biblioteca de Reglas de Seguridad OWASP Top 10

use crate::rules::{Rule, RuleSet};
use std::collections::HashMap;

/// OWASP Top 10 rule categories
#[derive(Debug, Clone)]
pub struct OWASPRuleCatalog {
    pub rules: HashMap<String, Rule>,
    pub by_category: HashMap<String, Vec<String>>,
}

impl OWASPRuleCatalog {
    /// Create new empty catalog
    pub fn new() -> Self {
        let mut catalog = Self {
            rules: HashMap::new(),
            by_category: HashMap::new(),
        };
        catalog.load_owasp_top_10_rules();
        catalog
    }

    /// Load OWASP Top 10 2021 rules
    fn load_owasp_top_10_rules(&mut self) {
        // A01: Broken Access Control
        self.add_rule(self.create_broken_access_control_rules());

        // A02: Cryptographic Failures
        self.add_rule(self.create_crypto_failures_rules());

        // A03: Injection (SQL, Command, XSS)
        self.add_rule(self.create_injection_rules());

        // A04: Insecure Design
        self.add_rule(self.create_insecure_design_rules());

        // A05: Security Misconfiguration
        self.add_rule(self.create_misconfiguration_rules());

        // A06: Vulnerable Components
        self.add_rule(self.create_vulnerable_components_rules());

        // A07: Identification and Authentication Failures
        self.add_rule(self.create_auth_failures_rules());

        // A08: Software and Data Integrity Failures
        self.add_rule(self.create_integrity_failures_rules());

        // A09: Logging and Monitoring Failures
        self.add_rule(self.create_logging_failures_rules());

        // A10: Server-Side Request Forgery
        self.add_rule(self.create_ssrf_rules());
    }

    /// Add a rule to catalog
    fn add_rule(&mut self, rule: Rule) {
        let category = rule
            .metadata
            .as_ref()
            .and_then(|m| m.category.clone())
            .unwrap_or_else(|| "uncategorized".to_string());

        self.rules.insert(rule.id.clone(), rule.clone());
        self.by_category
            .entry(category)
            .or_insert_with(Vec::new)
            .push(rule.id);
    }

    /// Get all rules
    pub fn get_all_rules(&self) -> RuleSet {
        let mut rule_set = RuleSet::new();
        for rule in self.rules.values() {
            rule_set.add_rule(rule.clone());
        }
        rule_set
    }

    /// Get rules by category
    pub fn get_rules_by_category(&self, category: &str) -> RuleSet {
        let mut rule_set = RuleSet::new();
        if let Some(rule_ids) = self.by_category.get(category) {
            for rule_id in rule_ids {
                if let Some(rule) = self.rules.get(rule_id) {
                    rule_set.add_rule(rule.clone());
                }
            }
        }
        rule_set
    }

    /// Get rules by severity
    pub fn get_rules_by_severity(&self, severity: &str) -> RuleSet {
        let mut rule_set = RuleSet::new();
        for rule in self.rules.values() {
            if let Some(metadata) = &rule.metadata {
                if metadata.severity == severity {
                    rule_set.add_rule(rule.clone());
                }
            }
        }
        rule_set
    }

    /// Create Broken Access Control rules (A01)
    fn create_broken_access_control_rules(&self) -> Rule {
        Rule {
            id: "OWASP-A01-001".to_string(),
            metadata: Some(crate::rules::RuleMetadata {
                name: "Direct Object Reference Without Authorization".to_string(),
                description: Some(
                    "Detects direct object references without proper access control checks"
                        .to_string(),
                ),
                severity: "critical".to_string(),
                confidence: "high".to_string(),
                category: Some("A01-BrokenAccessControl".to_string()),
                cwe: Some(vec!["639".to_string(), "862".to_string()]),
                owasp: Some(vec!["A01:2021".to_string()]),
            }),
            languages: vec![
                "python".to_string(),
                "javascript".to_string(),
                "java".to_string(),
            ],
            patterns: vec![
                crate::rules::Pattern {
                    pattern: "$USER.get($ID)".to_string(),
                    message: "Direct object reference without authorization check".to_string(),
                },
                crate::rules::Pattern {
                    pattern: "user[$ID]".to_string(),
                    message: "Potential direct object reference".to_string(),
                },
            ],
            where_clause: None,
            fix: None,
            tests: Some(vec![
                crate::rules::TestCase {
                    name: "Detects direct reference".to_string(),
                    code: "data = user.get(user_id)".to_string(),
                    should_match: true,
                },
                crate::rules::TestCase {
                    name: "No false positive with check".to_string(),
                    code: "if is_authorized(user_id): data = user.get(user_id)".to_string(),
                    should_match: false,
                },
            ]),
        }
    }

    /// Create Cryptographic Failures rules (A02)
    fn create_crypto_failures_rules(&self) -> Rule {
        Rule {
            id: "OWASP-A02-001".to_string(),
            metadata: Some(crate::rules::RuleMetadata {
                name: "Use of Weak Cryptographic Hash".to_string(),
                description: Some(
                    "Detects use of weak or outdated cryptographic hash functions".to_string(),
                ),
                severity: "major".to_string(),
                confidence: "high".to_string(),
                category: Some("A02-CryptoFailures".to_string()),
                cwe: Some(vec!["327".to_string()]),
                owasp: Some(vec!["A02:2021".to_string()]),
            }),
            languages: vec!["python".to_string(), "javascript".to_string()],
            patterns: vec![
                crate::rules::Pattern {
                    pattern: "hashlib.md5".to_string(),
                    message: "MD5 is cryptographically broken".to_string(),
                },
                crate::rules::Pattern {
                    pattern: "hashlib.sha1".to_string(),
                    message: "SHA1 is cryptographically weak".to_string(),
                },
                crate::rules::Pattern {
                    pattern: "CryptoJS.MD5".to_string(),
                    message: "MD5 is cryptographically broken".to_string(),
                },
            ],
            where_clause: None,
            fix: Some(crate::rules::Fix {
                template: "Use SHA-256 or stronger hash functions".to_string(),
                message: "Replace with cryptographically secure hash function".to_string(),
            }),
            tests: Some(vec![
                crate::rules::TestCase {
                    name: "Detects MD5 usage".to_string(),
                    code: "import hashlib; hashlib.md5(b'data')".to_string(),
                    should_match: true,
                },
                crate::rules::TestCase {
                    name: "Detects SHA1 usage".to_string(),
                    code: "hashlib.sha1(b'data')".to_string(),
                    should_match: true,
                },
            ]),
        }
    }

    /// Create Injection rules (A03)
    fn create_injection_rules(&self) -> Rule {
        Rule {
            id: "OWASP-A03-001".to_string(),
            metadata: Some(crate::rules::RuleMetadata {
                name: "SQL Injection via String Concatenation".to_string(),
                description: Some("Detects SQL injection vulnerabilities through string concatenation or formatting".to_string()),
                severity: "critical".to_string(),
                confidence: "high".to_string(),
                category: Some("A03-Injection".to_string()),
                cwe: Some(vec!["89".to_string()]),
                owasp: Some(vec!["A03:2021".to_string()]),
            }),
            languages: vec!["python".to_string(), "javascript".to_string(), "java".to_string()],
            patterns: vec![
                crate::rules::Pattern {
                    pattern: "execute($SQL + $VAR)".to_string(),
                    message: "SQL injection via string concatenation".to_string(),
                },
                crate::rules::Pattern {
                    pattern: "execute(f\"...{$VAR}...\")".to_string(),
                    message: "SQL injection via f-string formatting".to_string(),
                },
                crate::rules::Pattern {
                    pattern: "execute($SQL % $VAR)".to_string(),
                    message: "SQL injection via % formatting".to_string(),
                },
            ],
            where_clause: None,
            fix: Some(crate::rules::Fix {
                template: "Use parameterized queries".to_string(),
                message: "Replace with parameterized queries to prevent SQL injection".to_string(),
            }),
            tests: Some(vec![
                crate::rules::TestCase {
                    name: "Detects concatenation SQL injection".to_string(),
                    code: "db.execute('SELECT * FROM users WHERE id = ' + user_id)".to_string(),
                    should_match: true,
                },
                crate::rules::TestCase {
                    name: "Detects f-string SQL injection".to_string(),
                    code: "db.execute(f'SELECT * FROM users WHERE id = {user_id}')".to_string(),
                    should_match: true,
                },
                crate::rules::TestCase {
                    name: "No false positive for safe query".to_string(),
                    code: "db.execute('SELECT * FROM users WHERE id = ?', (user_id,))".to_string(),
                    should_match: false,
                },
            ]),
        }
    }

    /// Create Insecure Design rules (A04)
    fn create_insecure_design_rules(&self) -> Rule {
        Rule {
            id: "OWASP-A04-001".to_string(),
            metadata: Some(crate::rules::RuleMetadata {
                name: "Missing Rate Limiting".to_string(),
                description: Some(
                    "Detects missing rate limiting on authentication endpoints".to_string(),
                ),
                severity: "major".to_string(),
                confidence: "medium".to_string(),
                category: Some("A04-InsecureDesign".to_string()),
                cwe: Some(vec!["307".to_string()]),
                owasp: Some(vec!["A04:2021".to_string()]),
            }),
            languages: vec![
                "python".to_string(),
                "javascript".to_string(),
                "java".to_string(),
            ],
            patterns: vec![
                crate::rules::Pattern {
                    pattern: "def login".to_string(),
                    message: "Login endpoint without explicit rate limiting".to_string(),
                },
                crate::rules::Pattern {
                    pattern: "def authenticate".to_string(),
                    message: "Authentication endpoint without rate limiting".to_string(),
                },
            ],
            where_clause: None,
            fix: Some(crate::rules::Fix {
                template: "Implement rate limiting (e.g., 5 attempts per minute)".to_string(),
                message: "Add rate limiting to prevent brute force attacks".to_string(),
            }),
            tests: None,
        }
    }

    /// Create Security Misconfiguration rules (A05)
    fn create_misconfiguration_rules(&self) -> Rule {
        Rule {
            id: "OWASP-A05-001".to_string(),
            metadata: Some(crate::rules::RuleMetadata {
                name: "Debug Mode Enabled in Production".to_string(),
                description: Some(
                    "Detects debug mode or development features in production code".to_string(),
                ),
                severity: "major".to_string(),
                confidence: "high".to_string(),
                category: Some("A05-SecurityMisconfiguration".to_string()),
                cwe: Some(vec!["489".to_string()]),
                owasp: Some(vec!["A05:2021".to_string()]),
            }),
            languages: vec!["python".to_string(), "javascript".to_string()],
            patterns: vec![
                crate::rules::Pattern {
                    pattern: "DEBUG = True".to_string(),
                    message: "Debug mode enabled (production security risk)".to_string(),
                },
                crate::rules::Pattern {
                    pattern: "app.run(debug=True)".to_string(),
                    message: "Debug mode enabled in server startup".to_string(),
                },
            ],
            where_clause: None,
            fix: Some(crate::rules::Fix {
                template: "Set DEBUG = False in production".to_string(),
                message: "Disable debug mode in production environments".to_string(),
            }),
            tests: Some(vec![crate::rules::TestCase {
                name: "Detects debug flag".to_string(),
                code: "DEBUG = True".to_string(),
                should_match: true,
            }]),
        }
    }

    /// Create Vulnerable Components rules (A06)
    fn create_vulnerable_components_rules(&self) -> Rule {
        Rule {
            id: "OWASP-A06-001".to_string(),
            metadata: Some(crate::rules::RuleMetadata {
                name: "Use of Outdated Package Version".to_string(),
                description: Some("Detects use of packages with known vulnerabilities".to_string()),
                severity: "critical".to_string(),
                confidence: "high".to_string(),
                category: Some("A06-VulnerableComponents".to_string()),
                cwe: Some(vec!["1104".to_string()]),
                owasp: Some(vec!["A06:2021".to_string()]),
            }),
            languages: vec![
                "python".to_string(),
                "javascript".to_string(),
                "java".to_string(),
            ],
            patterns: vec![
                crate::rules::Pattern {
                    pattern: "django==2.2".to_string(),
                    message: "Django 2.2 has known vulnerabilities (EOL)".to_string(),
                },
                crate::rules::Pattern {
                    pattern: "requests==2.6.0".to_string(),
                    message: "Old version of requests library".to_string(),
                },
            ],
            where_clause: None,
            fix: Some(crate::rules::Fix {
                template: "Update to latest secure version".to_string(),
                message: "Update package to latest secure version".to_string(),
            }),
            tests: None,
        }
    }

    /// Create Authentication Failures rules (A07)
    fn create_auth_failures_rules(&self) -> Rule {
        Rule {
            id: "OWASP-A07-001".to_string(),
            metadata: Some(crate::rules::RuleMetadata {
                name: "Default or Weak Credentials".to_string(),
                description: Some("Detects use of default or weak passwords".to_string()),
                severity: "critical".to_string(),
                confidence: "high".to_string(),
                category: Some("A07-AuthenticationFailures".to_string()),
                cwe: Some(vec!["521".to_string()]),
                owasp: Some(vec!["A07:2021".to_string()]),
            }),
            languages: vec![
                "python".to_string(),
                "javascript".to_string(),
                "java".to_string(),
            ],
            patterns: vec![
                crate::rules::Pattern {
                    pattern: "password = \"admin\"".to_string(),
                    message: "Default admin password detected".to_string(),
                },
                crate::rules::Pattern {
                    pattern: "password = \"password\"".to_string(),
                    message: "Weak default password detected".to_string(),
                },
            ],
            where_clause: None,
            fix: Some(crate::rules::Fix {
                template: "Use strong, unique passwords".to_string(),
                message: "Replace with strong, unique password".to_string(),
            }),
            tests: Some(vec![crate::rules::TestCase {
                name: "Detects default password".to_string(),
                code: "password = \"admin\"".to_string(),
                should_match: true,
            }]),
        }
    }

    /// Create Software and Data Integrity Failures rules (A08)
    fn create_integrity_failures_rules(&self) -> Rule {
        Rule {
            id: "OWASP-A08-001".to_string(),
            metadata: Some(crate::rules::RuleMetadata {
                name: "Unsigned/Unverified Data Deserialization".to_string(),
                description: Some(
                    "Detects unsafe deserialization without integrity checks".to_string(),
                ),
                severity: "critical".to_string(),
                confidence: "high".to_string(),
                category: Some("A08-IntegrityFailures".to_string()),
                cwe: Some(vec!["502".to_string()]),
                owasp: Some(vec!["A08:2021".to_string()]),
            }),
            languages: vec![
                "python".to_string(),
                "javascript".to_string(),
                "java".to_string(),
            ],
            patterns: vec![
                crate::rules::Pattern {
                    pattern: "pickle.loads($DATA)".to_string(),
                    message: "Unsafe deserialization with pickle".to_string(),
                },
                crate::rules::Pattern {
                    pattern: "eval($USER_INPUT)".to_string(),
                    message: "Code injection via eval()".to_string(),
                },
            ],
            where_clause: None,
            fix: Some(crate::rules::Fix {
                template: "Use safe deserialization with integrity checks".to_string(),
                message: "Replace with safe deserialization".to_string(),
            }),
            tests: Some(vec![crate::rules::TestCase {
                name: "Detects pickle deserialization".to_string(),
                code: "pickle.loads(data)".to_string(),
                should_match: true,
            }]),
        }
    }

    /// Create Logging and Monitoring Failures rules (A09)
    fn create_logging_failures_rules(&self) -> Rule {
        Rule {
            id: "OWASP-A09-001".to_string(),
            metadata: Some(crate::rules::RuleMetadata {
                name: "Missing Security Event Logging".to_string(),
                description: Some(
                    "Detects missing logging for security-critical events".to_string(),
                ),
                severity: "minor".to_string(),
                confidence: "medium".to_string(),
                category: Some("A09-LoggingFailures".to_string()),
                cwe: Some(vec!["778".to_string()]),
                owasp: Some(vec!["A09:2021".to_string()]),
            }),
            languages: vec![
                "python".to_string(),
                "javascript".to_string(),
                "java".to_string(),
            ],
            patterns: vec![crate::rules::Pattern {
                pattern: "def login".to_string(),
                message: "Login function without security event logging".to_string(),
            }],
            where_clause: None,
            fix: Some(crate::rules::Fix {
                template: "Add security event logging".to_string(),
                message: "Log authentication events for security monitoring".to_string(),
            }),
            tests: None,
        }
    }

    /// Create Server-Side Request Forgery rules (A10)
    fn create_ssrf_rules(&self) -> Rule {
        Rule {
            id: "OWASP-A10-001".to_string(),
            metadata: Some(crate::rules::RuleMetadata {
                name: "Unvalidated URL Request".to_string(),
                description: Some(
                    "Detects server-side requests without proper URL validation".to_string(),
                ),
                severity: "major".to_string(),
                confidence: "high".to_string(),
                category: Some("A10-SSRF".to_string()),
                cwe: Some(vec!["918".to_string()]),
                owasp: Some(vec!["A10:2021".to_string()]),
            }),
            languages: vec![
                "python".to_string(),
                "javascript".to_string(),
                "java".to_string(),
            ],
            patterns: vec![
                crate::rules::Pattern {
                    pattern: "requests.get($URL)".to_string(),
                    message: "HTTP request without URL validation".to_string(),
                },
                crate::rules::Pattern {
                    pattern: "urllib.request.urlopen($URL)".to_string(),
                    message: "URL request without validation".to_string(),
                },
            ],
            where_clause: None,
            fix: Some(crate::rules::Fix {
                template: "Validate and sanitize URL before request".to_string(),
                message: "Add URL validation to prevent SSRF attacks".to_string(),
            }),
            tests: Some(vec![crate::rules::TestCase {
                name: "Detects unvalidated request".to_string(),
                code: "requests.get(user_url)".to_string(),
                should_match: true,
            }]),
        }
    }

    /// Get catalog statistics
    pub fn get_stats(&self) -> OWASPCatalogStats {
        let mut by_severity = HashMap::new();
        for rule in self.rules.values() {
            if let Some(metadata) = &rule.metadata {
                *by_severity.entry(metadata.severity.clone()).or_insert(0) += 1;
            }
        }

        OWASPCatalogStats {
            total_rules: self.rules.len(),
            by_category: self.by_category.len(),
            by_severity,
        }
    }
}

impl Default for OWASPRuleCatalog {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about the OWASP catalog
#[derive(Debug, Clone)]
pub struct OWASPCatalogStats {
    pub total_rules: usize,
    pub by_category: usize,
    pub by_severity: HashMap<String, usize>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_catalog_creation() {
        let catalog = OWASPRuleCatalog::new();
        assert!(catalog.rules.len() > 0);
    }

    #[test]
    fn test_get_all_rules() {
        let catalog = OWASPRuleCatalog::new();
        let rule_set = catalog.get_all_rules();
        assert!(rule_set.rules().len() > 0);
    }

    #[test]
    fn test_get_rules_by_category() {
        let catalog = OWASPRuleCatalog::new();
        let rule_set = catalog.get_rules_by_category("A03-Injection");
        assert!(rule_set.rules().len() > 0);
    }

    #[test]
    fn test_get_rules_by_severity() {
        let catalog = OWASPRuleCatalog::new();
        let rule_set = catalog.get_rules_by_severity("critical");
        assert!(rule_set.rules().len() > 0);
    }

    #[test]
    fn test_catalog_stats() {
        let catalog = OWASPRuleCatalog::new();
        let stats = catalog.get_stats();
        assert!(stats.total_rules > 0);
        assert!(stats.by_category > 0);
        assert!(stats.by_severity.len() > 0);
    }

    #[test]
    fn test_sql_injection_rule() {
        let catalog = OWASPRuleCatalog::new();
        let rule = catalog.rules.get("OWASP-A03-001").unwrap();

        assert_eq!(rule.id, "OWASP-A03-001");
        assert!(rule.metadata.is_some());
        assert!(rule.patterns.len() > 0);
        assert!(rule.tests.is_some());
    }

    #[test]
    fn test_crypto_rule() {
        let catalog = OWASPRuleCatalog::new();
        let rule = catalog.rules.get("OWASP-A02-001").unwrap();

        assert!(rule.patterns.iter().any(|p| p.pattern.contains("md5")));
    }
}
