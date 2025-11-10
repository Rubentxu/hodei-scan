# hodei-cli

CLI tool for hodei-scan.

## Overview

This crate provides the command-line interface for running code analysis with hodei-scan.

## Key Components

- **Command Parser**: CLI argument parsing and validation
- **Analysis Runner**: Orchestrates the full analysis pipeline
- **Output Formatters**: Multiple output formats (JSON, SARIF, HTML)

## Usage

```bash
# Analyze a project
hodei scan --project /path/to/project

# Run with custom rules
hodei scan --rules /path/to/rules.d --output sarif

# Check quality gates
hodei gate --threshold coverage:80
```

## License

MIT
