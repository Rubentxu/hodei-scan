# hodei-engine

Rule evaluation engine for hodei-scan.

## Overview

This crate provides the high-performance rule evaluation engine that processes IR facts and generates findings.

## Key Components

- **IndexedFactStore**: Indexed storage for fast fact retrieval
- **RuleEngine**: Parallel rule evaluation engine
- **QueryPlanner**: Intelligent index selection for optimal queries

## Usage

```rust
use hodei_engine::RuleEngine;
```

## License

MIT
