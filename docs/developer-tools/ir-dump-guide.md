# IR Dump Tool Guide

The ir-dump crate provides a powerful CLI tool for inspecting Intermediate Representation (IR) data from hodei-scan.

## Installation

```bash
cargo install --path crates/ir-dump
```

## Usage

### Basic Commands

```bash
# Dump IR to visual format (default)
ir-dump --input facts.json

# Dump IR to JSON
ir-dump --input facts.json --format json

# Dump IR to YAML
ir-dump --input facts.yaml --format yaml

# Interactive mode
ir-dump --input facts.json --interactive

# Filter by fact type
ir-dump --input facts.json --filter "type=Vulnerability"

# Compare two IR files
ir-dump --input-1 facts_v1.json --input-2 facts_v2.json --diff
```

## Output Formats

### Visual Format (Default)
Human-readable ASCII tree structure:
```
IR Structure:
============================================================

Finding #1
------------------------------------------------------------
Fact Type: Vulnerability
Message: SQL injection vulnerability in user input
Location: src/auth/login.js:42

Finding #2
------------------------------------------------------------
Fact Type: CodeSmell
Message: Unused variable
Location: src/utils/helpers.js:15

Total findings: 2
```

### JSON Format
Machine-readable JSON:
```json
{
  "facts": [
    {
      "fact_type": "Vulnerability",
      "message": "SQL injection vulnerability in user input",
      "location": "src/auth/login.js:42",
      "severity": "Critical",
      "metadata": {
        "confidence": "0.95"
      }
    }
  ]
}
```

### YAML Format
Human and machine-readable YAML:
```yaml
facts:
  - fact_type: Vulnerability
    message: SQL injection vulnerability in user input
    location: src/auth/login.js:42
    severity: Critical
```

## Interactive Explorer

The interactive REPL mode provides a powerful way to explore IR data:

```bash
ir-dump --input facts.json --interactive
```

### Available Commands

```
help          Show available commands
show          Show current finding
next          Move to next finding
prev          Move to previous finding
goto N        Jump to finding N (1-based)
list          List all findings
filter TYPE   Filter by fact type
stats         Show statistics
quit          Exit explorer
```

### Example Session
```
hodei-scan IR Explorer
Type 'help' for available commands
Type 'quit' to exit

[IR 1]> list
All findings:
  [1] Vulnerability - SQL injection in login
  [2] CodeSmell - Unused variable
  [3] Vulnerability - Hardcoded password

[IR 1]> goto 3
[IR 3]> show

============================================================
Finding #3 (of 3)
============================================================
Fact Type: Vulnerability
Message: Hardcoded password detected
Location: src/config/db.js:23
============================================================

[IR 3]> filter Vulnerability
Filtering by: 'Vulnerability'
------------------------------------------------------------
[1] SQL injection in login - src/auth/login.js:42
[3] Hardcoded password - src/config/db.js:23

[IR 3]> stats

Statistics:
============================================================
Total findings: 3

By fact type:
  Vulnerability: 2
  CodeSmell: 1
============================================================

[IR 3]> quit
Goodbye!
```

## Architecture

### Components

1. **IRReader** - Reads IR from various formats
   - JSON (serde_json)
   - YAML (serde_yml)
   - Cap'n Proto (capnp) - future

2. **IRFormatter** - Formats IR to output
   - Visual: ASCII tree
   - JSON: Structured data
   - YAML: Human-readable

3. **InteractiveExplorer** - REPL for exploring IR
   - Uses reedline for terminal I/O
   - Command parsing and execution
   - State management

### Usage in Code

```rust
use ir_dump::{IRReader, IRFormatter, Format};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let reader = IRReader::new();
    let formatter = IRFormatter::new();
    
    // Read IR
    let ir = reader.read(Path::new("facts.json")).await?;
    
    // Format to visual
    let output = formatter.format(&ir, &Format::Visual)?;
    println!("{}", output);
    
    Ok(())
}
```

## Filtering

### Filter by Fact Type
```bash
ir-dump --input facts.json --filter "type=Vulnerability"
```

### Filter by Message
```bash
ir-dump --input facts.json --filter "message=SQL"
```

### Filter by Location
```bash
ir-dump --input facts.json --filter "location=src/auth"
```

## Comparison Mode

Compare two IR files to see differences:

```bash
ir-dump --input-1 facts_v1.json --input-2 facts_v2.json --diff
```

Output:
```
Comparing IR files:
  File 1: facts_v1.json (5 findings)
  File 2: facts_v2.json (7 findings)

Difference: 2 findings
```

## Integration

### With VS Code Extension
Use from the extension:
1. Right-click on a .hodei file
2. Select "hodei-scan: Dump IR"
3. Choose format and output location

### With CI/CD
```bash
# Validate IR output
ir-dump --input facts.json --format json > facts_output.json

# Check for critical vulnerabilities
ir-dump --input facts.json --filter "type=Vulnerability" | grep -i critical
```

### With Scripts
```bash
#!/bin/bash
# check_findings.sh

INPUT=$1
CRITICAL=$(ir-dump --input "$INPUT" --filter "type=Vulnerability" | grep -c "Critical" || echo "0")

if [ "$CRITICAL" -gt 0 ]; then
  echo "Found $CRITICAL critical vulnerabilities!"
  exit 1
fi

echo "No critical vulnerabilities found"
```

## Best Practices

### 1. Use Visual Format for Quick Inspection
```bash
ir-dump --input facts.json  # Default visual format
```

### 2. Use JSON for Scripting
```bash
ir-dump --input facts.json --format json | jq '.facts[] | select(.severity == "Critical")'
```

### 3. Use Interactive Mode for Deep Analysis
```bash
ir-dump --input facts.json --interactive
```

### 4. Filter Early to Reduce Output
```bash
ir-dump --input facts.json --filter "type=Vulnerability" --format visual
```

### 5. Compare After Rule Updates
```bash
# Before update
hodei-scan scan src/ --output facts_v1.json
ir-dump --input facts_v1.json --format json --output facts_v1.json

# After update
hodei-scan scan src/ --output facts_v2.json
ir-dump --input facts_v2.json --format json --output facts_v2.json

# Compare
ir-dump --input-1 facts_v1.json --input-2 facts_v2.json --diff
```

## Extending

### Custom Formatter
```rust
pub struct CustomFormatter;

impl IRFormatter {
    pub fn format_custom(&self, ir: &FindingSet) -> Result<String, String> {
        // Custom formatting logic
        todo!()
    }
}
```

### Custom Filter
```rust
pub struct CustomFilter;

impl CustomFilter {
    pub fn apply(&self, ir: &FindingSet, filter: &str) -> Result<FindingSet, String> {
        // Custom filtering logic
        todo!()
    }
}
```
