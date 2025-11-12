# Examples

This directory contains practical examples demonstrating how to use the hodei-scan developer tools.

## Getting Started Examples

### 01-basic-rule
Simple rule with autocompletion and validation.

**Files:**
- `basic_rule.hodei` - A simple security rule
- `README.md` - Explains the example

**Concepts:**
- Rule syntax
- Fact types
- LSP autocompletion

**Try it:**
```bash
# Open in VS Code with extension
code basic_rule.hodei

# See autocompletion by typing "Vuln<TAB>"
# Hover over keywords to see documentation
```

### 02-rule-with-patterns
Rule with pattern matching and functions.

**Files:**
- `password_validation.hodei` - Password validation rule
- `password_validation.test.hodei` - Test cases

**Concepts:**
- Pattern matching
- Function calls
- Test file format

**Try it:**
```bash
# Run tests
hodei-test test-file --rule password_validation.hodei

# See test results
```

### 03-complex-analysis
Multi-rule analysis with IR inspection.

**Files:**
- `rules/` - Directory with multiple rules
- `test_suite.hodei.test` - Comprehensive test suite
- `analyze.sh` - Shell script for analysis

**Concepts:**
- Multiple rules
- IR comparison
- Shell scripting

**Try it:**
```bash
# Run full analysis
./analyze.sh

# Dump IR
ir-dump --input output.json --interactive

# Compare with baseline
ir-dump --input-1 baseline.json --input-2 output.json --diff
```

## LSP Examples

### lsp-01-basic-completion
Demonstrates basic autocompletion features.

**Features:**
- Fact type completions
- Field completions
- Function completions

### lsp-02-smart-context
Shows contextual intelligence.

**Features:**
- Context-aware completions
- Trigger character behavior
- Snippet support

### lsp-03-hover-documentation
Displays hover documentation.

**Features:**
- Fact type documentation
- Function documentation
- Parameter hints

## Test Framework Examples

### test-01-simple
Basic test case example.

```yaml
rule: "simple_rule.hodei"
language: "hodei-dsl"

cases:
  - name: "Basic test"
    code: "function test() { return true; }"
    expected_findings: []
```

### test-02-snapshot-testing
Demonstrates snapshot testing.

```yaml
rule: "complex_rule.hodei"
language: "hodei-dsl"

cases:
  - name: "Complex scenario"
    code: |
      function complex() {
        // Complex logic here
      }
    expected_findings:
      - finding_type: "Vulnerability"
        severity: "Major"
        message: "Complex logic detected"
```

Commands:
```bash
# Create snapshots
hodei-test test-file --update-snapshots

# Verify snapshots
hodei-test test-file --verify-snapshots
```

### test-03-custom-comparator
Shows custom result comparator.

**Files:**
- `custom_test.hodei.test`
- `custom_comparator.rs` - Custom comparator implementation

## IR Dump Examples

### ir-01-basic-dump
Basic IR inspection.

```bash
# Visual format
ir-dump --input facts.json

# JSON format
ir-dump --input facts.json --format json

# YAML format
ir-dump --input facts.json --format yaml
```

### ir-02-filtering
Demonstrates filtering capabilities.

```bash
# Filter by type
ir-dump --input facts.json --filter "type=Vulnerability"

# Filter by location
ir-dump --input facts.json --filter "location=src/auth"

# Chain filters
ir-dump --input facts.json --filter "type=Vulnerability,location=src/auth"
```

### ir-03-interactive-mode
Interactive REPL exploration.

```bash
ir-dump --input facts.json --interactive
```

Commands:
```
> list              # List all findings
> goto 5            # Jump to finding 5
> filter CodeSmell  # Filter by type
> stats             # Show statistics
> show              # Show current finding
> quit              # Exit
```

## VS Code Extension Examples

### vscode-01-setup
Complete setup walkthrough.

**Topics:**
- Installing the extension
- Configuring LSP server
- Using keybindings

### vscode-02-commands
Using extension commands.

**Commands:**
- Test Rule
- Dump IR
- Show Documentation

### vscode-03-customization
Customizing the extension.

**Topics:**
- Adding custom commands
- Configuring settings
- Custom syntax highlighting

## Integration Examples

### integration-01-ci-pipeline
CI/CD integration example.

**Files:**
- `.github/workflows/rules-test.yml` - GitHub Actions workflow
- `Makefile` - Build automation
- `scripts/test-all.sh` - Test runner script

### integration-02-vscode-tasks
VS Code Tasks integration.

**Files:**
- `.vscode/tasks.json` - VS Code tasks
- `run-tests.sh` - Task implementation

### integration-03-debugging-setup
Complete debugging setup.

**Files:**
- `.vscode/launch.json` - Debugger configuration
- `debug-helper.js` - Debug utilities
- `README.md` - Debugging guide

## Real-World Examples

### real-world-01-password-validation
Production-ready password validation rule.

**Files:**
- `password_rule.hodei` - The rule
- `password_tests.hodei.test` - Comprehensive tests
- `README.md` - Rule documentation

**Features:**
- Multiple test scenarios
- Edge case handling
- Performance considerations

### real-world-02-sql-injection-detection
SQL injection detection rule.

**Files:**
- `sql_injection_rule.hodei` - Detection rule
- `sql_test_suite.hodei.test` - Test suite
- `test_cases/` - Sample vulnerable code

**Features:**
- Pattern matching
- False positive reduction
- Confidence scoring

### real-world-03-authentication-rules
Authentication security rules.

**Files:**
- `auth_rules.hodei` - Multiple auth rules
- `auth_tests.hodei.test` - Test suite
- `examples/` - Code examples

**Features:**
- Rule composition
- Dependency handling
- Documentation

## Tutorial Examples

### tutorial-01-first-rule
Step-by-step tutorial: Creating your first rule.

**Duration:** 30 minutes

**Topics:**
1. Setting up the environment
2. Writing your first rule
3. Testing the rule
4. Using the LSP
5. Publishing the rule

### tutorial-02-advanced-patterns
Advanced pattern matching tutorial.

**Duration:** 45 minutes

**Topics:**
1. Complex patterns
2. Custom functions
3. Rule composition
4. Performance tuning

### tutorial-03-test-driven-development
Test-driven development for rules.

**Duration:** 60 minutes

**Topics:**
1. Writing tests first
2. Implementing rules
3. Refactoring
4. Maintaining tests

## Utility Scripts

### build-all.sh
Builds all tools and extensions.

```bash
#!/bin/bash
set -e

echo "Building hodei-dsl-lsp..."
cargo build --package hodei-dsl-lsp

echo "Building hodei-test..."
cargo build --package hodei-test

echo "Building ir-dump..."
cargo build --package ir-dump

echo "Compiling VS Code extension..."
cd extensions/vscode-hodei-dsl
npm run compile
cd -

echo "All tools built successfully!"
```

### run-all-tests.sh
Runs all tests across tools.

```bash
#!/bin/bash
set -e

echo "Running LSP tests..."
cargo test --package hodei-dsl-lsp

echo "Running test framework tests..."
cargo test --package hodei-test

echo "Running IR dump tests..."
cargo test --package ir-dump

echo "Running extension tests..."
cd extensions/vscode-hodei-dsl
npm test
cd -

echo "All tests passed!"
```

### setup-dev.sh
Sets up development environment.

```bash
#!/bin/bash
set -e

echo "Installing Rust dependencies..."
cargo fetch

echo "Building all crates..."
cargo build --all

echo "Installing tools..."
cargo install --path crates/hodei-dsl-lsp
cargo install --path crates/hodei-test
cargo install --path crates/ir-dump

echo "Setting up VS Code extension..."
cd extensions/vscode-hodei-dsl
npm install
npm run compile
cd -

echo "Development environment ready!"
```

## Data Files

### sample-rules/
Sample rule files for testing.

- `password_strength.hodei`
- `sql_injection.hodei`
- `xss_prevention.hodei`
- `auth_best_practices.hodei`

### sample-ir/
Sample IR files.

- `vulnerabilities.json`
- `code_smells.json`
- `mixed_findings.yaml`

### test-cases/
Vulnerable code samples for testing.

- `sql_injection_samples/`
- `xss_samples/`
- `auth_samples/`

## How to Use Examples

1. **Start with basics**
   ```bash
   cd examples/01-basic-rule
   code .
   ```

2. **Read the README**
   Each example has a detailed README explaining concepts.

3. **Try the commands**
   Follow the commands in the README.

4. **Experiment**
   Modify the examples to learn by doing.

5. **Check solutions**
   Solution files available in `solutions/` directory.

## Contributing Examples

To contribute a new example:

1. Create a new directory in `examples/`
2. Add README.md with explanation
3. Include all necessary files
4. Test the example thoroughly
5. Submit a pull request

Example structure:
```
examples/
├── new-example/
│   ├── README.md
│   ├── example.hodei
│   ├── example.test.hodei
│   ├── solution/ (optional)
│   └── assets/ (optional)
```

## Best Practices

### For Rule Examples
- Keep rules simple and focused
- Include comprehensive tests
- Document all features used
- Provide comments in code

### For Test Examples
- Cover edge cases
- Use descriptive test names
- Include snapshot tests
- Show both passing and failing tests

### For Documentation
- Start with overview
- Provide step-by-step instructions
- Include expected output
- Link to relevant guides

## Support

If you encounter issues with examples:
1. Check the README for common issues
2. Review error messages carefully
3. Verify all dependencies are installed
4. Open an issue on GitHub

## License

All examples are released under the same MIT license as the main project.
