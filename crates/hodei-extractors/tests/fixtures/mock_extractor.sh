#!/bin/bash
# Mock extractor script that returns sample IR
# Reads configuration from stdin and outputs JSON IR

# Read stdin (ignore errors if no input)
cat > /dev/null 2>&1

# Generate sample IR
cat << 'EOF'
{
  "schema_version": "3.3.0",
  "metadata": {
    "name": "Mock Project",
    "version": "1.0.0",
    "root_path": "/"
  },
  "facts": []
}
EOF
