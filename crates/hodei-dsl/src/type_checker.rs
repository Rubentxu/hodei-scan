//! Type checker for DSL rules

use crate::ast::*;
use crate::error::{TypeError, TypeResult};
use std::collections::HashMap;

/// Type checker for DSL rules
pub struct TypeChecker {
    fact_schemas: HashMap<String, FactSchema>,
}

/// Schema for fact types
#[derive(Debug, Clone)]
pub struct FactSchema {
    pub fields: HashMap<String, FieldType>,
}

/// Field types
#[derive(Debug, Clone, PartialEq)]
pub enum FieldType {
    String,
    Number,
    Boolean,
    SourceLocation,
    Confidence,
    Array(Box<FieldType>),
    Optional(Box<FieldType>),
}

impl TypeChecker {
    /// Create a new type checker with predefined fact schemas
    pub fn new() -> Self {
        let mut fact_schemas = HashMap::new();

        // Define schemas for all fact types
        fact_schemas.insert("TaintSink".to_string(), Self::taint_sink_schema());
        fact_schemas.insert("TaintSource".to_string(), Self::taint_source_schema());
        fact_schemas.insert("UncoveredLine".to_string(), Self::uncovered_line_schema());
        fact_schemas.insert("Vulnerability".to_string(), Self::vulnerability_schema());
        fact_schemas.insert("Dependency".to_string(), Self::dependency_schema());
        fact_schemas.insert("License".to_string(), Self::license_schema());
        fact_schemas.insert("CodeSmell".to_string(), Self::code_smell_schema());
        fact_schemas.insert(
            "DependencyVulnerability".to_string(),
            Self::dep_vuln_schema(),
        );
        fact_schemas.insert("LowTestCoverage".to_string(), Self::low_coverage_schema());
        fact_schemas.insert("CoverageStats".to_string(), Self::coverage_stats_schema());
        fact_schemas.insert("Sanitization".to_string(), Self::sanitization_schema());
        fact_schemas.insert("UnsafeCall".to_string(), Self::unsafe_call_schema());
        fact_schemas.insert("CryptographicOperation".to_string(), Self::crypto_schema());
        fact_schemas.insert("Function".to_string(), Self::function_schema());
        fact_schemas.insert("Variable".to_string(), Self::variable_schema());

        Self { fact_schemas }
    }

    /// Type check a rule definition
    pub fn check_rule(&self, rule: &RuleDef) -> TypeResult<()> {
        // Build symbol table with bindings
        let mut symbols = HashMap::new();
        for pattern in &rule.match_block.patterns {
            if let Some(schema) = self.fact_schemas.get(&pattern.fact_type) {
                symbols.insert(pattern.binding.clone(), schema.clone());
            } else {
                return Err(TypeError::UnknownFactType(pattern.fact_type.clone()));
            }
        }

        // Type check where clause
        if let Some(expr) = &rule.match_block.where_clause {
            self.check_expr(expr, &symbols)?;
        }

        // Type check emit block
        self.check_emit_block(&rule.emit_block, &symbols)?;

        Ok(())
    }

    fn check_expr(
        &self,
        expr: &Expr,
        symbols: &HashMap<String, FactSchema>,
    ) -> TypeResult<FieldType> {
        match expr {
            Expr::Literal(lit) => Ok(self.literal_type(lit)),

            Expr::Path(path) => self.resolve_path(path, symbols),

            Expr::Binary { left, op, right } => {
                let left_ty = self.check_expr(left, symbols)?;
                let right_ty = self.check_expr(right, symbols)?;

                // Validate operator compatibility
                match op {
                    BinaryOp::Eq | BinaryOp::Ne => {
                        if left_ty != right_ty {
                            return Err(TypeError::TypeMismatch {
                                expected: format!("{:?}", left_ty),
                                found: format!("{:?}", right_ty),
                            });
                        }
                        Ok(FieldType::Boolean)
                    }
                    BinaryOp::Lt | BinaryOp::Gt | BinaryOp::Le | BinaryOp::Ge => {
                        if !matches!(left_ty, FieldType::Number)
                            || !matches!(right_ty, FieldType::Number)
                        {
                            return Err(TypeError::ExpectedNumber);
                        }
                        Ok(FieldType::Boolean)
                    }
                    BinaryOp::And | BinaryOp::Or => {
                        if !matches!(left_ty, FieldType::Boolean)
                            || !matches!(right_ty, FieldType::Boolean)
                        {
                            return Err(TypeError::ExpectedBoolean);
                        }
                        Ok(FieldType::Boolean)
                    }
                }
            }

            Expr::FunctionCall { name, args } => self.check_function_call(name, args, symbols),
        }
    }

    fn resolve_path(
        &self,
        path: &Path,
        symbols: &HashMap<String, FactSchema>,
    ) -> TypeResult<FieldType> {
        if path.segments.is_empty() {
            return Err(TypeError::UndefinedVariable("empty".to_string()));
        }

        let first = &path.segments[0];
        let schema = symbols
            .get(first)
            .ok_or_else(|| TypeError::UndefinedVariable(first.clone()))?;

        // Simple field access - return the field type directly
        // In a full implementation, we'd recursively check each segment
        if path.segments.len() == 1 {
            // Return the schema type (treating each fact as having a simple type)
            return Ok(FieldType::SourceLocation);
        }

        // For multi-segment paths, just return SourceLocation as a placeholder
        Ok(FieldType::SourceLocation)
    }

    fn check_function_call(
        &self,
        name: &str,
        args: &[Expr],
        _symbols: &HashMap<String, FactSchema>,
    ) -> TypeResult<FieldType> {
        // Built-in functions
        match name {
            "count" => Ok(FieldType::Number),
            "distance" => Ok(FieldType::Number),
            "reachable" => Ok(FieldType::Boolean),
            _ => Err(TypeError::UnknownFunction(name.to_string())),
        }
    }

    fn check_emit_block(
        &self,
        emit_block: &EmitBlock,
        _symbols: &HashMap<String, FactSchema>,
    ) -> TypeResult<()> {
        // For now, just validate that metadata values are valid types
        for (_key, value) in &emit_block.metadata {
            match value {
                Literal::String(_) | Literal::Number(_) | Literal::Boolean(_) | Literal::Null => {
                    // Valid
                }
            }
        }
        Ok(())
    }

    fn literal_type(&self, lit: &Literal) -> FieldType {
        match lit {
            Literal::String(_) => FieldType::String,
            Literal::Number(_) => FieldType::Number,
            Literal::Boolean(_) => FieldType::Boolean,
            Literal::Null => FieldType::Optional(Box::new(FieldType::String)),
        }
    }

    // Schema definitions for each fact type
    fn taint_sink_schema() -> FactSchema {
        let mut fields = HashMap::new();
        fields.insert("location".to_string(), FieldType::SourceLocation);
        fields.insert("func".to_string(), FieldType::String);
        fields.insert("consumes_flow".to_string(), FieldType::String);
        fields.insert("category".to_string(), FieldType::String);
        fields.insert("severity".to_string(), FieldType::String);
        FactSchema { fields }
    }

    fn taint_source_schema() -> FactSchema {
        let mut fields = HashMap::new();
        fields.insert("location".to_string(), FieldType::SourceLocation);
        fields.insert("var".to_string(), FieldType::String);
        fields.insert("flow_id".to_string(), FieldType::String);
        fields.insert("source_type".to_string(), FieldType::String);
        fields.insert("confidence".to_string(), FieldType::Confidence);
        FactSchema { fields }
    }

    fn uncovered_line_schema() -> FactSchema {
        let mut fields = HashMap::new();
        fields.insert("location".to_string(), FieldType::SourceLocation);
        fields.insert("coverage".to_string(), FieldType::String);
        FactSchema { fields }
    }

    fn vulnerability_schema() -> FactSchema {
        let mut fields = HashMap::new();
        fields.insert("location".to_string(), FieldType::SourceLocation);
        fields.insert(
            "cwe_id".to_string(),
            FieldType::Optional(Box::new(FieldType::String)),
        );
        fields.insert(
            "owasp_category".to_string(),
            FieldType::Optional(Box::new(FieldType::String)),
        );
        fields.insert("severity".to_string(), FieldType::String);
        fields.insert(
            "cvss_score".to_string(),
            FieldType::Optional(Box::new(FieldType::Number)),
        );
        fields.insert("description".to_string(), FieldType::String);
        FactSchema { fields }
    }

    fn dependency_schema() -> FactSchema {
        let mut fields = HashMap::new();
        fields.insert("name".to_string(), FieldType::String);
        fields.insert("version".to_string(), FieldType::String);
        fields.insert("path".to_string(), FieldType::String);
        FactSchema { fields }
    }

    fn license_schema() -> FactSchema {
        let mut fields = HashMap::new();
        fields.insert("dependency".to_string(), FieldType::String);
        fields.insert("license_type".to_string(), FieldType::String);
        fields.insert("compatible".to_string(), FieldType::Boolean);
        FactSchema { fields }
    }

    fn code_smell_schema() -> FactSchema {
        let mut fields = HashMap::new();
        fields.insert("location".to_string(), FieldType::SourceLocation);
        fields.insert("smell_type".to_string(), FieldType::String);
        fields.insert("severity".to_string(), FieldType::String);
        fields.insert("description".to_string(), FieldType::String);
        FactSchema { fields }
    }

    fn dep_vuln_schema() -> FactSchema {
        let mut fields = HashMap::new();
        fields.insert("dependency".to_string(), FieldType::String);
        fields.insert("vulnerability_id".to_string(), FieldType::String);
        fields.insert("severity".to_string(), FieldType::String);
        fields.insert("description".to_string(), FieldType::String);
        FactSchema { fields }
    }

    fn low_coverage_schema() -> FactSchema {
        let mut fields = HashMap::new();
        fields.insert("file".to_string(), FieldType::String);
        fields.insert("percentage".to_string(), FieldType::Number);
        fields.insert("total_lines".to_string(), FieldType::Number);
        fields.insert("covered_lines".to_string(), FieldType::Number);
        FactSchema { fields }
    }

    fn coverage_stats_schema() -> FactSchema {
        let mut fields = HashMap::new();
        fields.insert("scope".to_string(), FieldType::String);
        fields.insert("path".to_string(), FieldType::String);
        fields.insert("line_coverage".to_string(), FieldType::Number);
        fields.insert("branch_coverage".to_string(), FieldType::Number);
        FactSchema { fields }
    }

    fn sanitization_schema() -> FactSchema {
        let mut fields = HashMap::new();
        fields.insert("location".to_string(), FieldType::SourceLocation);
        fields.insert("method".to_string(), FieldType::String);
        fields.insert("sanitizes_flow".to_string(), FieldType::String);
        fields.insert("effective".to_string(), FieldType::Boolean);
        fields.insert("confidence".to_string(), FieldType::Confidence);
        FactSchema { fields }
    }

    fn unsafe_call_schema() -> FactSchema {
        let mut fields = HashMap::new();
        fields.insert("location".to_string(), FieldType::SourceLocation);
        fields.insert("function_name".to_string(), FieldType::String);
        fields.insert("reason".to_string(), FieldType::String);
        fields.insert("severity".to_string(), FieldType::String);
        FactSchema { fields }
    }

    fn crypto_schema() -> FactSchema {
        let mut fields = HashMap::new();
        fields.insert("location".to_string(), FieldType::SourceLocation);
        fields.insert("algorithm".to_string(), FieldType::String);
        fields.insert(
            "key_length".to_string(),
            FieldType::Optional(Box::new(FieldType::Number)),
        );
        fields.insert("secure".to_string(), FieldType::Boolean);
        fields.insert(
            "recommendation".to_string(),
            FieldType::Optional(Box::new(FieldType::String)),
        );
        FactSchema { fields }
    }

    fn function_schema() -> FactSchema {
        let mut fields = HashMap::new();
        fields.insert("location".to_string(), FieldType::SourceLocation);
        fields.insert("name".to_string(), FieldType::String);
        fields.insert("arity".to_string(), FieldType::Number);
        FactSchema { fields }
    }

    fn variable_schema() -> FactSchema {
        let mut fields = HashMap::new();
        fields.insert("location".to_string(), FieldType::SourceLocation);
        fields.insert("name".to_string(), FieldType::String);
        fields.insert("var_type".to_string(), FieldType::String);
        FactSchema { fields }
    }
}

impl Default for TypeChecker {
    fn default() -> Self {
        Self::new()
    }
}
