//! Simple parser for Hodei Scan rule DSL
//!
//! This module provides a basic parser for the Hodei Scan rule DSL
//! without external dependencies. It uses regex-based parsing for
//! simplicity and reliability.

use crate::ast::*;
use crate::error::{ParseError, ParseResult};
use regex::Regex;
use std::collections::HashMap;

/// Parse a complete rule file
pub fn parse_file(input: &str) -> ParseResult<RuleFile> {
    let mut rules = Vec::new();

    // Find all rule start positions
    let rule_start_pattern = Regex::new(r"rule\s+(\w+)\s*\{").unwrap();
    let matches: Vec<_> = rule_start_pattern.find_iter(input).collect();

    if matches.is_empty() {
        return Ok(RuleFile { rules });
    }

    // For each match, find the matching closing brace
    for (i, start_match) in matches.iter().enumerate() {
        let start = start_match.start();
        let brace_pos = start_match.end() - 1; // Position of the opening {

        // Find the matching closing brace
        let mut depth = 1;
        let mut end = input.len();
        let mut pos = brace_pos + 1;

        while pos < input.len() && depth > 0 {
            let c = input.chars().nth(pos).unwrap();
            if c == '{' {
                depth += 1;
            } else if c == '}' {
                depth -= 1;
                if depth == 0 {
                    end = pos + 1;
                    break;
                }
            }
            pos += 1;
        }

        // Extract the rule text and parse it
        let rule_text = &input[start..end];
        let rule = parse_rule_text(rule_text)?;
        rules.push(rule);
    }

    Ok(RuleFile { rules })
}

/// Parse a single rule from text
fn parse_rule_text(text: &str) -> ParseResult<RuleDef> {
    // Extract rule name
    let name_pattern = Regex::new(r"rule\s+(\w+)").unwrap();
    let name = name_pattern
        .captures(text)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str().to_string())
        .ok_or(ParseError::msg("Failed to extract rule name"))?;

    // Extract description
    let desc_pattern = Regex::new(r#"description:\s*"([^"]*)""#).unwrap();
    let description = desc_pattern
        .captures(text)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str().to_string());

    // Extract severity
    let severity_pattern = Regex::new(r#"severity:\s*"([^"]+)""#).unwrap();
    let severity = severity_pattern
        .captures(text)
        .and_then(|caps| caps.get(1))
        .map(|m| match m.as_str() {
            "Critical" => Severity::Critical,
            "High" => Severity::High,
            "Medium" => Severity::Medium,
            "Low" => Severity::Low,
            "Info" => Severity::Info,
            _ => Severity::Medium,
        })
        .unwrap_or(Severity::Medium);

    // Extract message template
    let message_pattern = Regex::new(r#"message:\s*"([^"]*)""#).unwrap();
    let message_template = message_pattern
        .captures(text)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str().to_string())
        .unwrap_or_else(|| "Finding".to_string());

    // Extract confidence
    let confidence_pattern = Regex::new(r#"confidence:\s*"([^"]+)""#).unwrap();
    let confidence = confidence_pattern
        .captures(text)
        .and_then(|caps| caps.get(1))
        .map(|m| match m.as_str() {
            "High" => Confidence::High,
            "Medium" => Confidence::Medium,
            "Low" => Confidence::Low,
            _ => Confidence::Medium,
        })
        .unwrap_or(Confidence::Medium);

    // Extract patterns (simple pattern matching)
    let pattern_pattern = Regex::new(r"(\w+):\s*(\w+)").unwrap();
    let mut patterns = Vec::new();
    for cap in pattern_pattern.find_iter(text) {
        if let Some(captures) = pattern_pattern.captures(cap.as_str()) {
            let fact_type = captures.get(1).unwrap().as_str().to_string();
            let binding = captures.get(2).unwrap().as_str().to_string();

            // Skip keywords (check fact_type)
            if ![
                "rule",
                "description",
                "severity",
                "match",
                "emit",
                "message",
                "confidence",
            ]
            .contains(&fact_type.as_str())
            {
                patterns.push(Pattern {
                    binding,
                    fact_type,
                    conditions: vec![],
                    span: Span {
                        start: cap.start(),
                        end: cap.end(),
                    },
                });
            }
        }
    }

    Ok(RuleDef {
        name,
        metadata: Metadata {
            description,
            severity,
            tags: vec![],
        },
        match_block: MatchBlock {
            patterns,
            where_clause: None,
        },
        emit_block: EmitBlock {
            message_template,
            confidence,
            metadata: HashMap::new(),
        },
        span: Span {
            start: 0,
            end: text.len(),
        },
    })
}

/// Convenience function to parse a single rule
pub fn parse_rule(input: &str) -> ParseResult<RuleDef> {
    let file = parse_file(input)?;
    if file.rules.len() == 1 {
        Ok(file.rules[0].clone())
    } else {
        Err(ParseError::msg("Expected a single rule"))
    }
}
