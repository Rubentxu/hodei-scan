//! Sample IR data for testing

use hodei_ir::FindingSet;

/// Empty IR set
pub fn empty_ir() -> FindingSet {
    FindingSet { findings: Vec::new() }
}

/// Single finding IR
pub fn single_finding_ir() -> FindingSet {
    FindingSet {
        findings: vec![hodei_ir::Finding {
            fact_type: "Vulnerability".to_string(),
            message: "SQL injection detected".to_string(),
            location: Some("src/auth/login.js:42".to_string()),
            severity: Some("Critical".to_string()),
            metadata: std::collections::HashMap::new(),
        }],
    }
}

/// Multiple findings IR
pub fn multiple_findings_ir() -> FindingSet {
    let mut findings = Vec::new();
    
    findings.push(hodei_ir::Finding {
        fact_type: "Vulnerability".to_string(),
        message: "SQL injection detected".to_string(),
        location: Some("src/auth/login.js:42".to_string()),
        severity: Some("Critical".to_string()),
        metadata: {
            let mut meta = std::collections::HashMap::new();
            meta.insert("confidence".to_string(), "0.95".to_string());
            meta
        },
    });
    
    findings.push(hodei_ir::Finding {
        fact_type: "CodeSmell".to_string(),
        message: "Unused variable".to_string(),
        location: Some("src/utils/helpers.js:15".to_string()),
        severity: Some("Minor".to_string()),
        metadata: std::collections::HashMap::new(),
    });
    
    findings.push(hodei_ir::Finding {
        fact_type: "Vulnerability".to_string(),
        message: "Hardcoded password".to_string(),
        location: Some("src/config/db.js:23".to_string()),
        severity: Some("Major".to_string()),
        metadata: {
            let mut meta = std::collections::HashMap::new();
            meta.insert("cwe".to_string(), "CWE-798".to_string());
            meta
        },
    });
    
    FindingSet { findings }
}

/// IR with various fact types
pub fn varied_fact_types_ir() -> FindingSet {
    let mut findings = Vec::new();
    
    let fact_types = vec![
        "Vulnerability",
        "CodeSmell",
        "SecurityIssue",
        "PerformanceIssue",
    ];
    
    for (i, fact_type) in fact_types.iter().enumerate() {
        findings.push(hodei_ir::Finding {
            fact_type: fact_type.to_string(),
            message: format!("Finding {}", i),
            location: Some(format!("file{}.js:{}", i, i * 10)),
            severity: Some("Major".to_string()),
            metadata: std::collections::HashMap::new(),
        });
    }
    
    FindingSet { findings }
}

/// JSON representation for testing
pub const SAMPLE_JSON: &str = r#"{
  "findings": [
    {
      "fact_type": "Vulnerability",
      "message": "SQL injection detected",
      "location": "src/auth/login.js:42",
      "severity": "Critical",
      "metadata": {
        "confidence": "0.95"
      }
    },
    {
      "fact_type": "CodeSmell",
      "message": "Unused variable",
      "location": "src/utils/helpers.js:15",
      "severity": "Minor"
    }
  ]
}"#;

/// YAML representation for testing
pub const SAMPLE_YAML: &str = r#"findings:
  - fact_type: Vulnerability
    message: SQL injection detected
    location: src/auth/login.js:42
    severity: Critical
    metadata:
      confidence: "0.95"
  - fact_type: CodeSmell
    message: Unused variable
    location: src/utils/helpers.js:15
    severity: Minor
"#;

/// Creates IR with specific number of findings
pub fn create_ir_with_findings(num_findings: usize) -> FindingSet {
    let findings: Vec<hodei_ir::Finding> = (0..num_findings)
        .map(|i| hodei_ir::Finding {
            fact_type: if i % 2 == 0 {
                "Vulnerability".to_string()
            } else {
                "CodeSmell".to_string()
            },
            message: format!("Finding {}", i),
            location: Some(format!("file{}.js:{}", i, i)),
            severity: Some(if i % 3 == 0 {
                "Critical".to_string()
            } else {
                "Major".to_string()
            }),
            metadata: {
                let mut meta = std::collections::HashMap::new();
                meta.insert("index".to_string(), i.to_string());
                meta
            },
        })
        .collect();
    
    FindingSet { findings }
}

/// IR with unicode characters
pub fn unicode_ir() -> FindingSet {
    FindingSet {
        findings: vec![hodei_ir::Finding {
            fact_type: "Vulnerability".to_string(),
            message: "安全漏洞检测到".to_string(), // Chinese
            location: Some("文件.js:42".to_string()),
            severity: Some("Critical".to_string()),
            metadata: {
                let mut meta = std::collections::HashMap::new();
                meta.insert("language".to_string(), "zh-CN".to_string());
                meta
            },
        }],
    }
}
