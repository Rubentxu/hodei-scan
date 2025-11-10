# hodei-dsl

DSL parser for hodei-scan rules.

## Overview

This crate provides a Cedar-like DSL for defining security and quality rules that can be evaluated against IR facts.

## Key Components

- **RuleParser**: PEG-based parser for the rule DSL
- **AST**: Type-safe abstract syntax tree for rules
- **QualityGates**: Metric-based quality gate definitions

## Usage

```rust
use hodei_dsl::RuleParser;

let rule = RuleParser::parse_rule(r#"
    rule sql_injection_uncovered {
        severity: critical
        condition: FactExists {
            fact_type: TaintSource,
            and: FactExists {
                fact_type: TaintSink {
                    category: SqlQuery
                }
            }
        }
    }
"#)?;
```

## License

MIT
