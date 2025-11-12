#!/bin/bash
# Mock extractor script that returns sample IR
# Reads configuration from stdin and outputs JSON IR

# Read stdin (ignore errors if no input)
cat > /dev/null 2>&1

# Generate sample IR
cat << 'EOF'
{
  "schema_version": "1.0.0",
  "facts": [
    {
      "id": "fact-1",
      "fact_type": {
        "type": "CodeSmell",
        "smell_type": "unused_variable",
        "severity": "Minor"
      },
      "message": "Unused variable detected",
      "location": {
        "file": "test.py",
        "start_line": 10,
        "start_column": 5,
        "end_line": 10,
        "end_column": 10
      },
      "provenance": {
        "extractor": "sarif-adapter",
        "version": "1.0.0",
        "confidence": 0.9,
        "extracted_at": "2024-01-01T00:00:00Z"
      }
    },
    {
      "id": "fact-2",
      "fact_type": {
        "type": "Vulnerability",
        "cwe_ids": [89],
        "security_severity": 0.8
      },
      "message": "SQL injection vulnerability",
      "location": {
        "file": "test.py",
        "start_line": 20,
        "start_column": 10,
        "end_line": 20,
        "end_column": 25
      },
      "provenance": {
        "extractor": "sarif-adapter",
        "version": "1.0.0",
        "confidence": 0.9,
        "extracted_at": "2024-01-01T00:00:00Z"
      }
    }
  ]
}
EOF
