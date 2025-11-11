# hodei.toml Configuration Guide

The `hodei.toml` configuration file allows you to configure and customize extractors for hodei-scan. This guide covers all available configuration options.

## Table of Contents

- [Basic Configuration](#basic-configuration)
- [Extractor Configuration](#extractor-configuration)
- [Resource Limits](#resource-limits)
- [Configuration Inheritance](#configuration-inheritance)
- [Complete Examples](#complete-examples)

---

## Basic Configuration

### Minimal Configuration

```toml
[extractors]
enabled = true
```

### Full Configuration Structure

```toml
[extractors]
enabled = true
max_concurrent = 4
default_timeout = "30s"
global_memory_limit = 1_000_000_000  # 1GB
global_cpu_limit = 80  # 80% CPU
default_nice = 5  # CPU priority (-20 to 19)
default_io_priority = 2  # I/O priority (0-7, 0 is highest)

[[extractors.def]]
# Extractor definitions here...
```

---

## Extractor Configuration

### Simple Extractor

```toml
[[extractors.def]]
name = "ruff"
command = "ruff-to-hodei"
```

### Extractor with Arguments

```toml
[[extractors.def]]
name = "ruff"
command = "ruff"
args = ["--format", "json", "--output-format", "json"]
```

### Extractor with Custom Timeout

```toml
[[extractors.def]]
name = "sarif"
command = "sarif-to-hodei"
timeout = "60s"  # Override default timeout
```

### Extractor with Environment Variables

```toml
[[extractors.def]]
name = "ruff"
command = "ruff"
env = {
    RUST_BACKTRACE = "1",
    LOG_LEVEL = "debug",
    RUST_LOG = "hodei_scan=trace"
}
```

### Extractor with Resource Limits

```toml
[[extractors.def]]
name = "heavy-analyzer"
command = "analyzer"
memory_limit = 500_000_000  # 500MB
cpu_priority = 10  # Lower priority (nicer to other processes)
io_priority = 7  # Lowest I/O priority
```

### Complete Extractor Definition

```toml
[[extractors.def]]
name = "ruff"
command = "ruff-to-hodei"
args = ["--config", "ruff.toml"]
timeout = "30s"
env = { RUST_BACKTRACE = "1" }
memory_limit = 100_000_000  # 100MB
cpu_priority = 5
io_priority = 2
source = "ruff"  # Source tool name
```

---

## Resource Limits

### Global Limits (Applies to all extractors)

```toml
[extractors]
max_concurrent = 4  # Maximum number of extractors running simultaneously
default_timeout = "30s"  # Default timeout for all extractors
global_memory_limit = 1_000_000_000  # 1GB total memory limit
global_cpu_limit = 80  # 80% CPU limit
default_nice = 5  # Default CPU priority (-20 to 19)
default_io_priority = 2  # Default I/O priority (0-7)
```

### Per-Extractor Limits

```toml
[[extractors.def]]
name = "ruff"
command = "ruff"
memory_limit = 100_000_000  # 100MB for this extractor
cpu_priority = 10  # Lower priority
io_priority = 7  # Lowest I/O priority
```

### Timeout Formats

Supported timeout formats:
- `100ms` - milliseconds
- `5s` - seconds
- `2m` - minutes
- `1h` - hours

```toml
timeout = "30s"    # 30 seconds
timeout = "500ms"  # 500 milliseconds
timeout = "2m"     # 2 minutes
timeout = "1h"     # 1 hour
```

---

## Configuration Inheritance

You can have a global configuration and project-specific overrides.

### Global Config (~/.config/hodei/hodei.toml)

```toml
[extractors]
max_concurrent = 4
default_timeout = "30s"
default_nice = 5
default_io_priority = 2

[[extractors.def]]
name = "ruff"
command = "ruff-to-hodei"
```

### Project Config (./hodei.toml)

```toml
[extractors]
max_concurrent = 2  # Override global setting

[[extractors.def]]
name = "ruff"
command = "ruff-to-hodei"
timeout = "60s"  # Override default timeout

[[extractors.def]]
name = "sarif"
command = "sarif-to-hodei"
source = "semgrep"  # New extractor for this project
```

### How Inheritance Works

1. **Settings**: Project settings override global settings
2. **Extractors**:
   - Global extractors are inherited by all projects
   - Project can override global extractor settings
   - Project can add new extractors
   - Extractors are identified by name

---

## Complete Examples

### Python Project (Ruff + SARIF)

```toml
[extractors]
enabled = true
max_concurrent = 2
default_timeout = "30s"
default_nice = 5

[[extractors.def]]
name = "ruff"
command = "ruff-to-hodei"
args = ["--config", "ruff.toml"]
timeout = "60s"

[[extractors.def]]
name = "semgrep"
command = "sarif-to-hodei"
source = "semgrep"
timeout = "120s"
```

### Multi-Language Project

```toml
[extractors]
enabled = true
max_concurrent = 4
default_timeout = "30s"
default_nice = 3

[[extractors.def]]
name = "ruff-python"
command = "ruff-to-hodei"
source = "ruff"
args = ["--select", "ALL"]

[[extractors.def]]
name = "eslint-typescript"
command = "eslint-to-hodei"
source = "eslint"
timeout = "45s"

[[extractors.def]]
name = "gosec"
command = "gosec-to-hodei"
source = "gosec"
timeout = "60s"

[[extractors.def]]
name = "semgrep-java"
command = "sarif-to-hodei"
source = "semgrep"
timeout = "90s"
```

### CI/CD Optimized Config

```toml
[extractors]
enabled = true
max_concurrent = 8  # Use all CPU cores in CI
default_timeout = "10s"  # Faster timeouts in CI
default_nice = 10  # Lower priority to not interfere with build

[[extractors.def]]
name = "ruff"
command = "ruff-to-hodei"
timeout = "15s"
```

### High-Security Configuration

```toml
[extractors]
enabled = true
max_concurrent = 2  # Conservative concurrency
default_timeout = "300s"  # 5 minutes for thorough analysis
global_memory_limit = 2_000_000_000  # 2GB limit
global_cpu_limit = 50  # 50% CPU limit
default_nice = 15  # Very low priority

[[extractors.def]]
name = "semgrep"
command = "sarif-to-hodei"
source = "semgrep"
timeout = "300s"
memory_limit = 1_000_000_000  # 1GB per extractor
cpu_priority = 15
io_priority = 7  # Lowest I/O priority
```

### Development Configuration

```toml
[extractors]
enabled = true
max_concurrent = 1  # Sequential for easier debugging
default_timeout = "300s"  # No timeouts during development
default_nice = 0  # Highest priority
default_io_priority = 0  # Highest I/O priority

[[extractors.def]]
name = "ruff"
command = "ruff-to-hodei"
timeout = "300s"
env = {
    RUST_BACKTRACE = "1",
    LOG_LEVEL = "debug",
    RUST_LOG = "hodei_scan=debug"
}
```

---

## Validation Rules

The configuration is validated for the following:

### Global Settings

- `max_concurrent` must be > 0
- `default_timeout` must be in valid format (ms, s, m, h)
- `default_nice` must be between -20 and 19
- `default_io_priority` must be between 0 and 7
- `global_cpu_limit` must be between 0 and 100

### Extractor Settings

- `name` cannot be empty
- `command` cannot be empty
- `timeout` (if specified) must be in valid format
- `cpu_priority` (if specified) must be between -20 and 19
- `io_priority` (if specified) must be between 0 and 7

---

## Best Practices

1. **Use configuration inheritance**: Define global settings in `~/.config/hodei/hodei.toml`
2. **Set reasonable timeouts**: Default 30s is usually sufficient
3. **Monitor resource usage**: Check memory and CPU limits
4. **Use appropriate concurrency**: Match to your system capabilities
5. **Set I/O priorities**: Lower I/O priority for heavy analyzers
6. **Document custom extractors**: Add comments explaining purpose

---

## Troubleshooting

### Configuration Not Loading

1. Check file path: `./hodei.toml` or `~/.config/hodei/hodei.toml`
2. Verify TOML syntax with online validator
3. Check logs for parsing errors

### Extractor Not Running

1. Verify `enabled = true` in [extractors] section
2. Check extractor `name` and `command` are not empty
3. Ensure extractor binary is in PATH or use absolute path

### Timeout Errors

1. Increase `timeout` for the specific extractor
2. Increase `default_timeout` globally
3. Check if extractor is hanging

### Resource Limit Errors

1. Increase `global_memory_limit` if hitting memory limits
2. Reduce `max_concurrent` if hitting process limits
3. Check system ulimits: `ulimit -a`
