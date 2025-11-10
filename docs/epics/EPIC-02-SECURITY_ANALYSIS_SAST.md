# EPIC-02: Security Analysis (SAST)
## IR-Based Universal Security Rules with Cross-Language Detection

**Epic ID:** EPIC-02
**Version:** 2.0
**Created:** 2025-11-10
**Story Points:** 149
**Priority:** P0 (Critical)
**Status:** 🚧 In Progress

---

## 📋 Epic Overview

### Objective
Implement comprehensive SAST (Static Application Security Testing) using IR architecture to enable universal security rules across all supported languages. Target: 90%+ accuracy, <10% false positives, OWASP Top 10 2021 and CWE Top 25 2024 coverage.

### Key Deliverables
1. **OWASP Top 10 (2021) Rules** - 100% coverage with IR-based detection
2. **CWE Top 25 (2024) Rules** - Universal cross-language rules
3. **Taint Analysis Engine** - Cross-language data flow tracking
4. **Framework-Specific Rules** - React, Spring, Django, Flask support
5. **Cryptographic Validation** - Algorithm and key management checks
6. **Security Correlation** - Cross-domain analysis with SCA and Coverage

### Success Criteria
- [ ] OWASP Top 10 (2021): 100% coverage
- [ ] CWE Top 25 (2024): Universal rules via IR
- [ ] Framework-Specific: React, Spring, Django, Flask via IR facts
- [ ] Taint Analysis: Cross-language via IR
- [ ] Accuracy: >90% (vs 60-70% SonarQube)
- [ ] False Positives: <10% (vs 30-40% SonarQube)
- [ ] Analysis Speed: <10s for 1M LOC
- [ ] Language Support: JavaScript, TypeScript, Python, Go, Rust, Java, C#

---

## 🎯 User Stories & BDD Specifications

### US-01: As a Security Engineer, I want OWASP Top 10 2021 coverage via IR rules

**Priority:** P0
**Story Points:** 34
**Component:** OWASP Security Rules

#### BDD Specification (Gherkin)

```gherkin
Feature: OWASP Top 10 2021 Coverage

  Scenario: A01-Broken Access Control (IDOR detection)
    Given I have API endpoints with user IDs
    When the extractor processes route handlers
    Then it should detect missing authorization checks
    And generate AccessControl facts with missing checks

  Scenario: A02-Cryptographic Failures (weak crypto)
    Given I have cryptographic operations
    When the extractor processes crypto APIs
    Then it should detect:
      | algorithm         | severity  | requirement           |
      | MD5               | critical  | Use SHA-256+          |
      | SHA1              | high      | Use SHA-256+          |
      | DES               | critical  | Use AES-256+          |
      | ECB mode          | high      | Use CBC/GCM           |
      | RSA <2048 bits    | high      | Use RSA-2048+         |

  Scenario: A03-Injection (SQL, NoSQL, OS command)
    Given I have untrusted data reaching sinks
    When the extractor processes:
      | source        | sink         | sanitization     |
      | user input   | SQL query   | parameterized    |
      | request body | OS command | input validation |
      | URL param    | eval()     | NONE             |
    Then it should detect injection vulnerabilities
    And check for proper sanitization

  Scenario: A04-Insecure Design (business logic flaws)
    Given I have business logic
    When the extractor processes authentication flows
    Then it should detect:
      | flaw              | detection method            |
      | weak password     | < 8 chars, no complexity   |
      | missing MFA       | no second factor check     |
      | session flaws     | no expiration, weak tokens |

  Scenario: A05-Security Misconfiguration
    Given I have configuration files
    When the extractor processes:
      | config type | check for                    | severity |
      | CORS        | wildcard origins            | critical |
      | Headers     | missing security headers    | high     |
      | TLS         | weak ciphers, old versions  | critical |

  Scenario: A06-Vulnerable Components (SCA integration)
    Given I have dependencies
    When the extractor processes package manifests
    Then it should correlate SCA findings with SAST
    And generate VulnerableDependency facts

  Scenario: A07-Identity/Authentication Failures
    Given I have authentication code
    When the extractor processes auth handlers
    Then it should detect:
      | issue            | detection                       |
      | weak passwords   | no complexity requirements     |
      | missing lockout  | no brute force protection      |
      | weak sessions    | no secure session management   |

  Scenario: A08-Software/Data Integrity Failures
    Given I have update mechanisms
    When the extractor processes:
      | mechanism  | check for               | severity |
      | auto-update| no signature verification| critical |
      | plugins    | no integrity checks     | high     |

  Scenario: A09-Security Logging/Monitoring
    Given I have logging code
    When the extractor processes log statements
    Then it should verify:
      | check                  | requirement         |
      | sensitive data logging | must be masked      |
      | failed auth logging    | must be present     |
      | security events        | must be monitored   |

  Scenario: A10-Server-Side Request Forgery (SSRF)
    Given I have HTTP requests from user input
    When the extractor processes:
      | API              | check for              | severity |
      | fetch()          | URL validation          | high     |
      | urllib.request   | URL validation          | high     |
      | requests.get()   | URL validation          | high     |
    Then it should detect SSRF vulnerabilities
```

#### Implementation Tasks

**Task 1.1: Implement OWASP A01-A03 Rules**
- Create access control detection logic
- Implement cryptographic validation
- Build injection detection engine
- Add taint analysis for data flow

**Task 1.2: Implement OWASP A04-A07 Rules**
- Create business logic analysis
- Build configuration scanner
- Implement dependency correlation
- Add authentication testing

**Task 1.3: Implement OWASP A08-A10 Rules**
- Create integrity verification
- Build logging analyzer
- Implement SSRF detection

#### Test Suite (Unit Tests - 100% Coverage)

```rust
// tests/security/owasp_top10_test.rs

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_detect_sql_injection() {
        let code = r#"
            const userId = req.query.id;
            const query = `SELECT * FROM users WHERE id = ${userId}`;
            db.query(query);
        "#;
        
        let facts = extractor.extract(code, "vuln.js").await;
        let findings = security_engine.scan(&facts).await;
        
        let sql_injection = findings.find(|f| f.rule == "A03-001-SQL-INJECTION");
        assert!(sql_injection.is_some());
        assert_eq!(sql_injection.severity, Severity::CRITICAL);
    }

    #[tokio::test]
    async fn test_detect_weak_crypto() {
        let code = r#"
            const hash = crypto.createHash('md5').update(data).digest();
            const cipher = crypto.createCipher('des', key);
        "#;
        
        let facts = extractor.extract(code, "crypto.js").await;
        let findings = security_engine.scan(&facts).await;
        
        assert!(findings.iter().any(|f| f.rule == "A02-001-MD5-USAGE"));
        assert!(findings.iter().any(|f| f.rule == "A02-002-DES-USAGE"));
    }

    #[tokio::test]
    async fn test_validate_sanitization() {
        let code = r#"
            const userId = req.query.id;
            const sanitized = validator.escape(userId);
            const query = `SELECT * FROM users WHERE id = '${sanitized}'`;
        "#;
        
        let facts = extractor.extract(code, "safe.js").await;
        let findings = security_engine.scan(&facts).await;
        
        assert!(!findings.iter().any(|f| f.rule.contains("INJECTION")));
    }
}
```

---

### US-02: As a Security Researcher, I want CWE Top 25 2024 detection

**Priority:** P0
**Story Points:** 21
**Component:** CWE Security Rules

#### BDD Scenarios

```gherkin
Feature: CWE Top 25 2024 Coverage

  Scenario: CWE-787 Out-of-bounds Write
    Given I have array access
    When the extractor processes array operations
    Then it should detect unsafe array access without bounds checking

  Scenario: CWE-79 Cross-site Scripting (XSS)
    Given I have untrusted output
    When the extractor processes:
      | sink           | context   | detection           |
      | innerHTML      | HTML      | missing escape      |
      | document.write | HTML      | missing encode      |
      | eval()         | JS        | always vulnerable   |
    Then it should detect XSS vulnerabilities

  Scenario: CWE-89 SQL Injection
    Given I have database queries
    When the extractor processes:
      | query type | detection                            |
      | string concat | missing parameterized      |
      | template literal | missing sanitization |
      | dynamic SQL | missing validation           |
    Then it should detect SQL injection

  Scenario: CWE-20 Improper Input Validation
    Given I have input parameters
    When the extractor processes user inputs
    Then it should verify:
      | check type   | requirement       |
      | type         | validate data type|
      | range        | validate bounds   |
      | format       | validate pattern  |

  Scenario: CWE-125 Out-of-bounds Read
    Given I have buffer operations
    When the extractor processes:
      | operation | check for        | severity |
      | memcpy    | size validation  | high     |
      | array[]   | bounds checking  | high     |
      | pointer   | null check       | high     |
    Then it should detect unsafe reads

  Scenario: CWE-78 OS Command Injection
    Given I have system calls
    When the extractor processes:
      | API      | detection                  |
      | exec()   | unvalidated input          |
      | system() | missing shell escaping     |
      | eval()   | command execution          |
    Then it should detect command injection

  Scenario: CWE-416 Use After Free
    Given I have memory management
    When the extractor processes:
      | language | check for           |
      | C/C++    | free without null   |
      | Rust     | use after drop      |
    Then it should detect use-after-free

  Scenario: CWE-22 Path Traversal
    Given I have file operations
    When the extractor processes:
      | API        | check for         | severity |
      | open()     | "../" in path     | high     |
      | file()     | absolute path     | medium   |
      | include()  | directory change  | high     |
    Then it should detect path traversal

  Scenario: CWE-352 CSRF
    Given I have state-changing requests
    When the extractor processes:
      | check          | requirement           |
      | CSRF token     | must be validated     |
      | SameSite       | must be set           |
      | Origin header  | must be checked       |
    Then it should detect CSRF vulnerabilities

  Scenario: CWE-434 Unrestricted Upload
    Given I have file upload
    When the extractor processes upload handlers
    Then it should verify:
      | check           | requirement         |
      | file type       | must be validated   |
      | file size       | must be limited     |
      | file content    | must be scanned     |

  Scenario: CWE-862 Missing Authorization
    Given I have protected resources
    When the extractor processes route handlers
    Then it should verify:
      | check          | requirement           |
      | auth required  | must be checked       |
      | role check     | must be validated     |
      | ownership      | must be verified      |
```

#### Test Suite

```rust
#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_cwe_79_xss_detection() {
        let code = r#"
            document.getElementById('output').innerHTML = userInput;
        "#;
        
        let facts = extractor.extract(code, "xss.js").await;
        let findings = security_engine.scan(&facts).await;
        
        assert!(findings.iter().any(|f| f.cwe_id == "CWE-79"));
        assert_eq!(findings[0].severity, Severity::HIGH);
    }

    #[tokio::test]
    async fn test_cwe_862_missing_authorization() {
        let code = r#"
            app.get('/admin/users', (req, res) => {
                res.json(getAllUsers());
            });
        "#;
        
        let facts = extractor.extract(code, "auth.js").await;
        let findings = security_engine.scan(&facts).await;
        
        assert!(findings.iter().any(|f| f.cwe_id == "CWE-862"));
    }
}
```

---

### US-03: As a Developer, I want cross-language taint analysis via IR

**Priority:** P0
**Story Points:** 13
**Component:** Taint Analysis Engine

#### BDD Scenarios

```gherkin
Feature: Cross-Language Taint Analysis

  Scenario: Track taint from JavaScript to Python
    Given I have a multi-language project
    When data flows from:
      """
      JavaScript: req.query.userId → API call
      Python: receives userId → database query
      """
    Then IR should track taint across language boundary
    And security rules should apply to both languages

  Scenario: Taint propagation through function calls
    Given I have taint source
    When the taint flows through:
      | source   | transformations           | sink        |
      | input    | sanitize() -> toString()  | SQL query   |
      | query    | escapeHtml() -> concat()  | innerHTML   |
    Then IR should track transformation effectiveness
    And mark as sanitized if transformation is effective

  Scenario: Taint through data structures
    Given I have taint in data structures
    When the extractor processes:
      | structure | check for      | tracking         |
      | array     | tainted index  | track each item  |
      | object    | tainted key    | track properties |
      | map       | key/value pair | track both       |
    Then IR should track taint through structures

  Scenario: Taint with inter-procedural analysis
    Given I have taint source
    When the taint flows through function calls:
      """
      function processInput(userInput) {
        return sanitize(userInput);
      }
      
      const safe = processInput(req.query.id);
      db.query(`SELECT * FROM users WHERE id = '${safe}'`);
      """
    Then IR should:
      - Track taint at source
      - Mark as sanitized after sanitize()
      - Apply to sink with sanitized taint
```

#### Implementation Tasks

**Task 3.1: Build Taint Tracking Engine**
- Create taint source identification
- Implement taint propagation logic
- Add taint sink detection
- Build transformation tracking

**Task 3.2: Cross-Language Taint Flow**
- Track taint across language boundaries
- Handle API calls between languages
- Maintain taint context in IR facts
- Enable cross-language correlation

#### Test Suite

```rust
#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_taint_propagation() {
        let code = r#"
            const userId = req.query.id;  // Taint source
            const query = `SELECT * FROM users WHERE id = ${userId}`;  // Taint sink
        "#;
        
        let facts = extractor.extract(code, "taint.js").await;
        let taint_flow = find_taint_flow(&facts);
        
        assert!(taint_flow.is_some());
        assert_eq!(taint_flow.sanitized, false);
    }

    #[tokio::test]
    async fn test_taint_sanitization() {
        let code = r#"
            const userId = req.query.id;
            const sanitized = validator.escape(userId);
            const query = `SELECT * FROM users WHERE id = ${sanitized}`;
        "#;
        
        let facts = extractor.extract(code, "sanitized.js").await;
        let taint_flow = find_taint_flow(&facts);
        
        assert!(taint_flow.is_some());
        assert_eq!(taint_flow.sanitized, true);
    }
}
```

---

### US-04: As a Security Engineer, I want framework-specific rule detection

**Priority:** P0
**Story Points:** 13
**Component:** Framework Detection Engine

#### BDD Scenarios

```gherkin
Feature: Framework-Specific Security Rules

  Scenario: React Security Rules
    Given I have React components
    When the extractor processes:
      | API/Pattern                    | check for                | severity |
      | dangerouslySetInnerHTML        | missing validation       | critical |
      | eval() in JSX                  | always vulnerable        | critical |
      | innerHTML without encoding     | XSS vulnerability        | high     |
      | state updates without sanitization| XSS in state          | high     |

  Scenario: Spring Security Rules
    Given I have Spring Boot application
    When the extractor processes:
      | component         | check for                      | severity |
      | @RequestMapping   | missing @PreAuthorize          | high     |
      | JdbcTemplate      | missing parameterized queries  | critical |
      | ModelAttribute    | missing @Valid                 | medium   |

  Scenario: Django Security Rules
    Given I have Django application
    When the extractor processes:
      | component       | check for                   | severity |
      | QuerySet        | missing .get() validation   | high     |
      | raw() queries   | missing parameterization    | critical |
      | template tags   | missing auto-escaping check | high     |

  Scenario: Flask Security Rules
    Given I have Flask application
    When the extractor processes:
      | API/Pattern          | check for              | severity |
      | render_template      | missing context        | high     |
      | request.json         | missing validation     | medium   |
      | session manipulation | missing CSRF           | high     |

  Scenario: Express.js Security Rules
    Given I have Express application
    When the extractor processes:
      | middleware/API     | check for                | severity |
      | express.json()     | missing size limit       | high     |
      | helmet()           | missing security headers | medium   |
      | user input to response| missing sanitization  | high     |

  Scenario: .NET Security Rules
    Given I have ASP.NET application
    When the extractor processes:
      | component     | check for                    | severity |
      | Controller    | missing [ValidateAntiForgeryToken] | high |
      | SqlCommand    | missing parameters           | critical |
      | View          | missing HTML encoding        | high     |
```

#### Test Suite

```rust
#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_react_dangerous_html() {
        let code = r#"
            function Component() {
                return <div dangerouslySetInnerHTML={{__html: userContent}} />;
            }
        "#;
        
        let facts = extractor.extract(code, "react.jsx").await;
        let findings = security_engine.scan(&facts).await;
        
        assert!(findings.iter().any(|f| 
            f.rule == "REACT-001-DANGEROUS-INNER-HTML"
        ));
    }

    #[tokio::test]
    async fn test_spring_authorization() {
        let code = r#"
            @GetMapping("/admin/users")
            public List<User> getUsers() {
                return userService.findAll();
            }
        "#;
        
        let facts = extractor.extract(code, "UserController.java").await;
        let findings = security_engine.scan(&facts).await;
        
        assert!(findings.iter().any(|f| 
            f.rule == "SPRING-001-MISSING-PREAUTHORIZE"
        ));
    }
}
```

---

### US-05: As a CISO, I want <10% false positive rate for security findings

**Priority:** P0
**Story Points:** 8
**Component:** False Positive Reduction

#### BDD Scenarios

```gherkin
Feature: Low False Positive Rate

  Scenario: Accurate sanitization detection
    Given I have potentially vulnerable code
    When the code uses:
      | method              | context    | should detect | confidence |
      | parameterized query | SQL        | NO            | high       |
      | html.escape()       | HTML       | NO            | high       |
      | validator.escape()  | HTML       | NO            | high       |
      | built-in escaping   | template   | NO            | high       |
    Then security engine should not flag as vulnerable

  Scenario: Trusted data source detection
    Given I have data sources
    When the data comes from:
      | source              | trust level | detection |
      | config file         | trusted     | NO flag    |
      | hardcoded constant  | trusted     | NO flag    |
      | database (trusted)  | trusted     | NO flag    |
      | user input          | untrusted   | FLAG       |
    Then engine should distinguish trusted sources

  Scenario: Context-aware detection
    Given I have sink contexts
    When the extractor processes:
      | context        | check for       | threshold      |
      | HTML           | encoding needed | 100% encoding  |
      | attribute      | encoding needed | 100% encoding  |
      | URL parameter  | encoding needed | 100% encoding  |
      | CSS            | encoding needed | 100% encoding  |
    Then engine should be context-aware

  Scenario: Validation override
    Given I have validation comments
    When the code has:
      | comment                     | effect           |
      | // validate: SQL injection  | suppress finding |
      | # nosec B608               | suppress finding |
      | /* secure: using ORM */     | suppress finding |
    Then engine should respect validation comments
```

---

## 🏗️ Technical Implementation

### IR Security Facts Schema

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityFact {
    // Taint tracking
    TaintSource {
        source: String,
        location: CodeLocation,
        trust_level: TrustLevel,
    },
    TaintSink {
        sink: String,
        location: CodeLocation,
        context: String,
    },
    TaintFlow {
        source: String,
        sink: String,
        path: Vec<CodeLocation>,
        sanitized: bool,
    },
    
    // Input validation
    InputValidation {
        input: String,
        location: CodeLocation,
        validation_type: ValidationType,
        effective: bool,
    },
    
    // Cryptographic operations
    CryptographicOperation {
        algorithm: String,
        key_length: Option<u32>,
        location: CodeLocation,
        secure: bool,
        issue: Option<String>,
    },
    
    // Authentication/Authorization
    AuthCheck {
        location: CodeLocation,
        check_type: AuthCheckType,
        effective: bool,
    },
    
    // Security headers/config
    SecurityHeader {
        header: String,
        value: String,
        location: CodeLocation,
        present: bool,
    },
    
    // OWASP Top 10 mapping
    OwaspTop10 {
        category: String,
        rule_id: String,
        location: CodeLocation,
        severity: Severity,
    },
    
    // CWE mapping
    Cwe {
        id: String,
        name: String,
        location: CodeLocation,
        severity: Severity,
    },
}
```

### Security Rule Engine

```rust
pub struct SecurityRule {
    pub id: String,
    pub name: String,
    pub category: String, // OWASP/CWE
    pub severity: Severity,
    pub language_agnostic: bool,
    pub patterns: Vec<RulePattern>,
}

impl SecurityRule {
    pub async fn evaluate(&self, facts: &[SecurityFact]) -> Vec<SecurityFinding> {
        let mut findings = Vec::new();
        
        // Pattern matching
        for pattern in &self.patterns {
            if let Some(match_result) = pattern.match_facts(facts) {
                findings.push(SecurityFinding {
                    rule_id: self.id.clone(),
                    name: self.name.clone(),
                    severity: self.severity,
                    location: match_result.location,
                    confidence: match_result.confidence,
                    details: match_result.details,
                });
            }
        }
        
        findings
    }
}
```

---

## 📊 Performance Benchmarks

### Target Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| **Analysis Speed** | <10s for 1M LOC | Real projects |
| **False Positive Rate** | <10% | User feedback |
| **Detection Accuracy** | >90% | Test suites |
| **Memory Usage** | <1GB | 1M LOC |
| **Language Coverage** | 7 languages | All supported |

### Benchmarking

```bash
# Security analysis benchmarks
cargo bench --features security -- security_analysis

# Compare against SonarQube
./scripts/benchmark_sonarqube.sh

# Generate accuracy report
./scripts/security_accuracy_report.sh
```

---

## 🧪 Test Strategy

### Test Coverage Requirements

| Component | Coverage Target | Test Types |
|-----------|----------------|------------|
| **OWASP Rules** | 100% | Unit + Integration |
| **CWE Rules** | 100% | Unit + Integration |
| **Taint Analysis** | 100% | Unit + Integration |
| **Framework Detection** | 100% | Unit + Integration |
| **False Positives** | 100% | E2E tests |

### Test Execution

```bash
# Run all security tests
cargo test --features security

# Run OWASP tests
cargo test owasp_top10

# Run CWE tests
cargo test cwe_top25

# Run taint analysis tests
cargo test taint_analysis

# Run false positive tests
cargo test false_positive
```

---

## ✅ Definition of Done

### Code Quality
- [ ] All OWASP Top 10 2021 rules implemented
- [ ] All CWE Top 25 2024 rules implemented
- [ ] 100% test coverage
- [ ] Framework-specific rules for 5+ frameworks
- [ ] Cross-language taint analysis working

### Performance
- [ ] Analysis <10s for 1M LOC
- [ ] False positives <10%
- [ ] Detection accuracy >90%
- [ ] Memory usage <1GB

### Security
- [ ] No security vulnerabilities in code
- [ ] All findings reviewed
- [ ] Documentation complete

---

## 📝 Commit Validation Requirements

```bash
feat(epic-02): implement comprehensive SAST with IR-based security rules

- Implement OWASP Top 10 2021 coverage (100% rules implemented)
- Implement CWE Top 25 2024 detection (all rules)
- Build cross-language taint analysis engine
- Add framework-specific rules (React, Spring, Django, Flask, Express, .NET)
- Implement cryptographic validation (weak algorithms, key management)
- Add security correlation with SCA and coverage
- Achieve >90% detection accuracy
- Achieve <10% false positive rate
- Analysis speed: <10s for 1M LOC
- Test coverage: 100% for all security components
- 7 languages supported: JavaScript, TypeScript, Python, Go, Rust, Java, C#

Validation:
- All OWASP Top 10 2021 rules pass tests
- All CWE Top 25 2024 rules pass tests
- Taint analysis working cross-language
- Framework detection working for all major frameworks
- Performance benchmarks: PASS
- False positive rate: <10%
- Code quality: PASS (no clippy warnings)

Closes: EPIC-02
```

---

**Epic Owner:** Security Engineering Team
**Reviewers:** Architecture Team, Security Team, Performance Team
**Status:** 🚧 In Progress
**Next Steps:** Begin Phase 1 - OWASP Top 10 2021 Implementation

---

**Copyright © 2025 hodei-scan. All rights reserved.**
