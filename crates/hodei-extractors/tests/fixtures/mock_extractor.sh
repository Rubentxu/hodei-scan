#!/bin/bash
# Mock extractor script that returns sample IR
# Reads configuration from stdin and outputs JSON IR
# Usage: bash mock_extractor.sh [extractor_id]

# Get extractor ID from command line argument
EXTRACTOR_ID="${1:-unknown}"

# Read stdin and extract project_path using grep
STDIN=$(cat)
PROJECT_PATH=$(echo "$STDIN" | grep -o '"project_path":"[^"]*"' | cut -d'"' -f4)

# Generate unique IDs - combine PID, extractor ID hash, and sequential numbers
# This ensures uniqueness even with parallel execution
PID=$(printf "%d" $$)

# Create a simple hash from the extractor ID (sum of character codes)
EXTRACTOR_HASH=0
for ((i=0; i<${#EXTRACTOR_ID}; i++)); do
    EXTRACTOR_HASH=$((EXTRACTOR_HASH + $(printf "%d" "'${EXTRACTOR_ID:i:1}")))
done
EXTRACTOR_DIGIT=$((EXTRACTOR_HASH % 10))

# Combine PID mod 10 and extractor hash mod 10 for more uniqueness
PID_DIGIT=$((PID % 10))
UNIQUE_DIGIT=$(((PID_DIGIT + EXTRACTOR_DIGIT) % 10))

# Base has 9 chars, add UNIQUE_DIGIT + sequential number to make 12 chars total
FACT1_ID="550e8400-e29b-41d4-a716-4466554400${UNIQUE_DIGIT}1"
FACT2_ID="550e8400-e29b-41d4-b716-4466554400${UNIQUE_DIGIT}2"

# Construct the test file path - use project_path/test.py
if [ -n "$PROJECT_PATH" ]; then
    TEST_FILE_PATH="$PROJECT_PATH/test.py"
else
    TEST_FILE_PATH="test.py"
fi

# Generate sample IR with 2 facts using unique UUIDs per run
cat << EOF
{
  "schema_version": "3.3.0",
  "metadata": {
    "name": "Mock Project",
    "version": "1.0.0",
    "root_path": {
      "path": "/"
    }
  },
  "facts": [
    {
      "id": "$FACT1_ID",
      "fact_type": {
        "CodeSmell": {
          "smell_type": "test_smell",
          "severity": "Minor"
        }
      },
      "message": "Test code smell",
      "location": {
        "file": {
          "path": "$TEST_FILE_PATH"
        },
        "start_line": 1,
        "start_column": 1,
        "end_line": 1,
        "end_column": 10
      },
      "provenance": {
        "extractor": "Custom",
        "version": "1.0.0",
        "confidence": 0.9,
        "extracted_at": "2025-01-15T10:30:00Z"
      }
    },
    {
      "id": "$FACT2_ID",
      "fact_type": {
        "Vulnerability": {
          "cwe_id": "CWE-79",
          "owasp_category": "A03:2021",
          "severity": "Critical",
          "cvss_score": 9.8,
          "description": "Cross-site scripting",
          "confidence": 0.95
        }
      },
      "message": "XSS vulnerability in user input",
      "location": {
        "file": {
          "path": "$TEST_FILE_PATH"
        },
        "start_line": 10,
        "start_column": 5,
        "end_line": 10,
        "end_column": 20
      },
      "provenance": {
        "extractor": "Custom",
        "version": "1.0.0",
        "confidence": 0.95,
        "extracted_at": "2025-01-15T10:30:00Z"
      }
    }
  ]
}
EOF
