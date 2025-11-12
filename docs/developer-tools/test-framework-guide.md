# Test Framework Guide

The hodei-test crate provides a comprehensive testing framework for hodei-scan rules.

## Test File Format

Test files use YAML format with the `.hodei.test` extension.

### Basic Structure

```yaml
# test_password_rules.hodei.test
rule: "password_strength.hodei"
language: "hodei-dsl"

cases:
  - name: "Strong password"
    code: |
      function validatePassword(pwd: string): boolean {
        if (pwd.length >= 12 && pwd.matches(/[A-Z]/) && pwd.matches(/[0-9]/)) {
          return true;
        }
        return false;
      }
    expected_findings: []

  - name: "Weak password - too short"
    code: |
      function validatePassword(pwd: string): boolean {
        return pwd.length >= 8;  // Too short!
      }
    expected_findings:
      - finding_type: "CodeSmell"
        severity: "Major"
        message: "Password too weak"
```

## Running Tests

### Using the CLI
```bash
# Run a single test file
hodei-test test-file --rule path/to/rule.hodei

# Run all tests in a directory
hodei-test test-dir --rule path/to/rule.hodei

# With verbose output
hodei-test test-file --rule path/to/rule.hodei --verbose

# Update snapshots
hodei-test test-file --rule path/to/rule.hodei --update-snapshots
```

### Using the Library
```rust
use hodei_test::{HodeiTestRunner, YamlTestConfigParser, FileSystemSnapshotRepository};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let parser = YamlTestConfigParser::new();
    let runner = /* create your runner */;
    let comparator = /* create your comparator */;
    
    let test_runner = HodeiTestRunner::new(parser, runner, comparator);
    
    let results = test_runner
        .run_test_file("tests/password_strength.hodei.test", "rules/password_strength.hodei")
        .await?;
    
    println!("Passed: {}/{}", results.passed_tests, results.total_tests);
    
    Ok(())
}
```

## Snapshot Testing

The framework includes snapshot testing to prevent regressions.

### Creating Snapshots
```bash
hodei-test test-file --rule path/to/rule.hodei --update-snapshots
```

This creates `.snap` files:
```
snapshots/
  password_strength_test.snap
```

### Verifying Snapshots
```bash
hodei-test test-file --rule path/to/rule.hodei --verify-snapshots
```

### Snapshot Format
```json
{
  "test_name": "password_strength_test",
  "findings": [
    {
      "finding_type": "CodeSmell",
      "severity": "Major",
      "message": "Password too weak"
    }
  ],
  "metadata": {
    "timestamp": "2025-01-01T00:00:00Z",
    "rule_version": "1.0.0"
  }
}
```

## Architecture

### Domain Layer
- `models.rs` - TestCase, TestResults, ExpectedFinding, etc.
- `ports.rs` - TestConfigParser, TestCaseRunner, ResultComparator, etc.
- `services.rs` - Test validation and processing services

### Application Layer
- `test_runner.rs` - HodeiTestRunner orchestration
- `snapshot.rs` - SnapshotManager for snapshot testing

### Infrastructure Layer
- `yaml_parser.rs` - YAML test file parser using serde_yml
- `file_system_snapshot_repo.rs` - File system snapshot storage

## Best Practices

### 1. Write Descriptive Test Names
```yaml
cases:
  - name: "Should detect weak password with length < 8"  # Good
  - name: "Test 1"  # Bad
```

### 2. Test Edge Cases
```yaml
cases:
  - name: "Empty password"
    code: ""
    expected_findings: []
    
  - name: "Very long password"
    code: "a".repeat(1000)
    expected_findings: []
    
  - name: "Password with special characters"
    code: "P@ssw0rd!123"
    expected_findings: []
```

### 3. Use Snapshot Testing for Complex Output
```rust
// When testing complex rule output, use snapshots
let results = test_runner.run_test_file("complex_rule.hodei.test", "rules/complex_rule.hodei").await?;
snapshot_manager.update_snapshots(&results).await?;
```

### 4. Organize Test Files
```
tests/
  password/
    password_strength.hodei
    password_strength.hodei.test
  authentication/
    auth_rules.hodei
    auth_rules.hodei.test
  common/
    shared_test_cases.hodei.test
```

### 5. CI Integration
```yaml
# .github/workflows/rules-test.yml
- name: Run Rule Tests
  run: |
    cargo test --all --package hodei-test
    
    # Run actual rule tests
    for test_file in tests/**/*.hodei.test; do
      hodei-test test-file --rule "${test_file%.test}.hodei" || exit 1
    done
```

## Extending the Framework

### Custom Comparator
```rust
pub struct CustomComparator;

#[async_trait::async_trait]
impl ResultComparator for CustomComparator {
    async fn compare(
        &self,
        actual: &[hodei_ir::Finding],
        expected: &[ExpectedFinding],
    ) -> Vec<AssertionResult> {
        // Custom comparison logic
        todo!()
    }
}
```

### Custom Parser
```rust
pub struct JsonTestConfigParser;

#[async_trait::async_trait]
impl TestConfigParser for JsonTestConfigParser {
    async fn parse_file(&self, path: &Path) -> Result<TestConfig> {
        let content = fs::read_to_string(path).await?;
        let config: TestConfig = serde_json::from_str(&content)?;
        Ok(config)
    }
}
```

## Troubleshooting

### Common Issues

1. **YAML parsing errors**
   - Check indentation (use spaces, not tabs)
   - Validate YAML syntax

2. **Test failures**
   - Check expected_findings match actual output
   - Use `--verbose` for detailed output
   - Update snapshots if output changed intentionally

3. **Snapshot mismatches**
   - Review the diff carefully
   - Update snapshots if change is intentional
   - Fix rule if change is unexpected
