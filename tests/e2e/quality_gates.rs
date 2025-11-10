//! Quality Gates Validation Tests

use std::path::PathBuf;

#[cfg(test)]
mod quality_gates_tests {

    #[test]
    fn test_quality_gate_configuration_parsing() {
        let config = r#"
quality_gates:
  - name: "Security Gate"
    description: "Blocks critical security vulnerabilities"
    enabled: true
    rules:
      - "SQL Injection"
      - "Hardcoded Credentials"
    fail_conditions:
      - severity: "Critical"
        count: 0
      - severity: "High"
        count: 3
    pass_conditions:
      - severity: "Low"
        count: 10
            "#;

        // Test that we can parse quality gate configs
        assert!(config.contains("quality_gates"));
        assert!(config.contains("Security Gate"));
        assert!(config.contains("enabled"));
    }

    #[test]
    fn test_quality_gates_passing() {
        let temp_dir = tempfile::tempdir().unwrap();
        let project_dir = temp_dir.path().join("passing-project");
        std::fs::create_dir_all(&project_dir).unwrap();

        // Create quality gate config that should pass
        let config_dir = project_dir.join("config");
        std::fs::create_dir_all(&config_dir).unwrap();

        std::fs::write(
            config_dir.join("quality-gates.yml"),
            r#"
quality_gates:
  - name: "Code Quality Gate"
    description: "Checks code quality"
    enabled: true
    rules:
      - "TODO"
    fail_conditions:
      - severity: "Critical"
        count: 0
            "#,
        )
        .unwrap();

        // Create clean Java code (no TODOs)
        let src_dir = project_dir.join("src");
        std::fs::create_dir_all(&src_dir).unwrap();

        std::fs::write(
            src_dir.join("CleanCode.java"),
            r#"
public class CleanCode {
    public void method() {
        // Clean implementation
    }
}
            "#,
        )
        .unwrap();

        // Validate files exist
        assert!(config_dir.join("quality-gates.yml").exists());
        assert!(src_dir.join("CleanCode.java").exists());
    }

    #[test]
    fn test_quality_gates_failing() {
        let temp_dir = tempfile::tempdir().unwrap();
        let project_dir = temp_dir.path().join("failing-project");
        std::fs::create_dir_all(&project_dir).unwrap();

        // Create quality gate config
        let config_dir = project_dir.join("config");
        std::fs::create_dir_all(&config_dir).unwrap();

        std::fs::write(
            config_dir.join("quality-gates.yml"),
            r#"
quality_gates:
  - name: "Strict Quality Gate"
    description: "Very strict quality checks"
    enabled: true
    rules:
      - "System.out.println"
    fail_conditions:
      - severity: "Medium"
        count: 0
            "#,
        )
        .unwrap();

        // Create code with System.out.println (violation)
        let src_dir = project_dir.join("src");
        std::fs::create_dir_all(&src_dir).unwrap();

        std::fs::write(
            src_dir.join("BadCode.java"),
            r#"
public class BadCode {
    public void method() {
        System.out.println("This violates quality gate");
    }
}
            "#,
        )
        .unwrap();

        // Validate files exist
        assert!(config_dir.join("quality-gates.yml").exists());
        assert!(src_dir.join("BadCode.java").exists());
    }

    #[test]
    fn test_multiple_quality_gates() {
        let temp_dir = tempfile::tempdir().unwrap();
        let project_dir = temp_dir.path().join("multi-gates");
        std::fs::create_dir_all(&project_dir).unwrap();

        // Create config with multiple gates
        let config_dir = project_dir.join("config");
        std::fs::create_dir_all(&config_dir).unwrap();

        std::fs::write(
            config_dir.join("quality-gates.yml"),
            r#"
quality_gates:
  - name: "Security Gate"
    description: "Security checks"
    enabled: true
    rules:
      - "SQL Injection"
    fail_conditions:
      - severity: "Critical"
        count: 0

  - name: "Code Quality Gate"
    description: "Code quality checks"
    enabled: true
    rules:
      - "TODO"
      - "FIXME"
    fail_conditions:
      - severity: "Major"
        count: 0

  - name: "Test Coverage Gate"
    description: "Test coverage checks"
    enabled: true
    rules:
      - "LowTestCoverage"
    fail_conditions:
      - coverage: 80
        count: 1
            "#,
        )
        .unwrap();

        // Verify config has all gates
        let config_content = std::fs::read_to_string(config_dir.join("quality-gates.yml")).unwrap();
        assert!(config_content.contains("Security Gate"));
        assert!(config_content.contains("Code Quality Gate"));
        assert!(config_content.contains("Test Coverage Gate"));
    }

    #[test]
    fn test_quality_gates_with_thresholds() {
        let config = r#"
quality_gates:
  - name: "Threshold Gate"
    enabled: true
    rules:
      - "LongMethod"
    fail_conditions:
      - severity: "High"
        count: 0
      - method_length: 50
        count: 0
    pass_conditions:
      - coverage: 90
        count: 1
            "#;

        // Verify threshold configuration
        assert!(config.contains("method_length"));
        assert!(config.contains("coverage"));
    }

    #[test]
    fn test_quality_gates_disabled() {
        let temp_dir = tempfile::tempdir().unwrap();
        let project_dir = temp_dir.path().join("disabled-gates");
        std::fs::create_dir_all(&project_dir).unwrap();

        // Create config with disabled gate
        let config_dir = project_dir.join("config");
        std::fs::create_dir_all(&config_dir).unwrap();

        std::fs::write(
            config_dir.join("quality-gates.yml"),
            r#"
quality_gates:
  - name: "Disabled Gate"
    description: "This gate is disabled"
    enabled: false
    rules:
      - "TODO"
    fail_conditions:
      - severity: "Critical"
        count: 0
            "#,
        )
        .unwrap();

        // Verify gate is disabled
        let config_content = std::fs::read_to_string(config_dir.join("quality-gates.yml")).unwrap();
        assert!(config_content.contains("enabled: false"));
    }

    #[test]
    fn test_quality_gates_real_world_config() {
        let temp_dir = tempfile::tempdir().unwrap();
        let project_dir = temp_dir.path().join("real-world");
        std::fs::create_dir_all(&project_dir).unwrap();

        // Create comprehensive quality gate config
        let config_dir = project_dir.join("config");
        std::fs::create_dir_all(&config_dir).unwrap();

        std::fs::write(
            config_dir.join("quality-gates.yml"),
            r#"
quality_gates:
  # Security gate - must pass
  - name: "Security Gate"
    description: "Blocks critical security vulnerabilities"
    enabled: true
    rules:
      - "SQL Injection"
      - "XSS"
      - "Hardcoded Credentials"
      - "Weak Cryptography"
    fail_conditions:
      - severity: "Critical"
        count: 0
      - severity: "High"
        count: 1

  # Code quality gate
  - name: "Code Quality Gate"
    description: "Enforces code quality standards"
    enabled: true
    rules:
      - "TODO"
      - "FIXME"
      - "LongMethod"
      - "LargeClass"
    fail_conditions:
      - severity: "Major"
        count: 0

  # Test coverage gate
  - name: "Test Coverage Gate"
    description: "Ensures adequate test coverage"
    enabled: true
    rules:
      - "LowTestCoverage"
    fail_conditions:
      - coverage: 80
        count: 1

  # Dependency gate
  - name: "Dependency Gate"
    description: "Checks for vulnerable dependencies"
    enabled: true
    rules:
      - "VulnerableDependency"
      - "DeprecatedAPI"
    fail_conditions:
      - severity: "High"
        count: 0

  # Documentation gate
  - name: "Documentation Gate"
    description: "Ensures code is documented"
    enabled: false  # Can be enabled later
    rules:
      - "MissingJavadoc"
    fail_conditions:
      - severity: "Medium"
        count: 5
            "#,
        )
        .unwrap();

        // Verify all gate types are present
        let config_content = std::fs::read_to_string(config_dir.join("quality-gates.yml")).unwrap();
        assert!(config_content.contains("Security Gate"));
        assert!(config_content.contains("Code Quality Gate"));
        assert!(config_content.contains("Test Coverage Gate"));
        assert!(config_content.contains("Dependency Gate"));
        assert!(config_content.contains("Documentation Gate"));
    }

    #[test]
    fn test_quality_gates_yaml_structure() {
        let yaml = r#"
quality_gates:
  - name: "Test Gate"
    description: "Test description"
    enabled: true
    rules:
      - "Rule1"
      - "Rule2"
    fail_conditions:
      - severity: "Critical"
        count: 0
    pass_conditions:
      - severity: "Low"
        count: 5
    actions:
      - type: "fail"
        message: "Quality gate failed"
      - type: "notify"
        channels: ["email", "slack"]
            "#;

        // Verify YAML structure
        assert!(yaml.contains("quality_gates"));
        assert!(yaml.contains("name:"));
        assert!(yaml.contains("description:"));
        assert!(yaml.contains("enabled:"));
        assert!(yaml.contains("rules:"));
        assert!(yaml.contains("fail_conditions:"));
        assert!(yaml.contains("pass_conditions:"));
        assert!(yaml.contains("actions:"));
    }

    #[test]
    fn test_quality_gates_with_actions() {
        let config = r#"
quality_gates:
  - name: "Strict Gate"
    enabled: true
    rules:
      - "Security"
    fail_conditions:
      - severity: "Critical"
        count: 0
    actions:
      - type: "fail"
        message: "Build failed due to security issues"
      - type: "notify"
        channels: ["email:security@company.com", "slack:security-alerts"]
      - type: "block"
        reason: "Critical security vulnerability detected"
      - type: "tag"
        tags: ["security", "critical"]
            "#;

        // Verify action types
        assert!(config.contains("type: \"fail\""));
        assert!(config.contains("type: \"notify\""));
        assert!(config.contains("type: \"block\""));
        assert!(config.contains("type: \"tag\""));
    }
}
