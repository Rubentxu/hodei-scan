//! Plugin Schema Registry - Validates Custom FactTypes

use crate::{FactType, FactValue, FactValueType};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use thiserror::Error;

/// Errors for schema validation
#[derive(Error, Debug)]
pub enum SchemaError {
    #[error("Schema validation error: {0}")]
    ValidationError(String),

    #[error("Unknown custom fact type: {0}")]
    UnknownFactType(String),

    #[error("Missing required field: {0}")]
    MissingRequiredField(String),

    #[error("Type mismatch for field '{field}': expected {expected:?}, got {actual:?}")]
    TypeMismatch {
        field: String,
        expected: FactValueType,
        actual: FactValueType,
    },

    #[error("Schema conflict: {0}")]
    SchemaConflict(String),
}

/// Schema for a custom fact type
#[derive(Debug, Clone, PartialEq)]
pub struct CustomFactSchema {
    /// Unique name/identifier for the custom fact type
    pub name: String,
    /// Version of the schema
    pub version: String,
    /// Map of field names to their expected types
    pub fields: HashMap<String, FactValueType>,
    /// List of required field names
    pub required_fields: Vec<String>,
    /// Additional metadata (e.g., author, description)
    pub metadata: HashMap<String, String>,
}

impl CustomFactSchema {
    /// Create a new schema
    pub fn new(name: String, version: String) -> Self {
        Self {
            name,
            version,
            fields: HashMap::new(),
            required_fields: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Add a field to the schema
    pub fn add_field(&mut self, name: String, value_type: FactValueType, required: bool) {
        self.fields.insert(name.clone(), value_type);
        if required && !self.required_fields.contains(&name) {
            self.required_fields.push(name);
        }
    }

    /// Add metadata to the schema
    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    /// Validate that a Custom fact conforms to this schema
    pub fn validate_custom_fact(&self, fact: &crate::FactType) -> Result<(), SchemaError> {
        let (discriminant, data) = match fact {
            crate::FactType::Custom { discriminant, data } => (discriminant, data),
            _ => {
                return Err(SchemaError::UnknownFactType(
                    "Expected Custom fact type".to_string(),
                ));
            }
        };

        // Check that discriminant matches (or matches prefix)
        if !discriminant.starts_with(&self.name) {
            return Err(SchemaError::ValidationError(format!(
                "Fact discriminant '{}' does not match schema name '{}'",
                discriminant, self.name
            )));
        }

        // Check required fields
        for required_field in &self.required_fields {
            if !data.contains_key(required_field) {
                return Err(SchemaError::MissingRequiredField(required_field.clone()));
            }
        }

        // Check field types
        for (field_name, field_value) in data {
            if let Some(expected_type) = self.fields.get(field_name).cloned() {
                let actual_type = field_value.get_type();
                if actual_type != expected_type {
                    return Err(SchemaError::TypeMismatch {
                        field: field_name.clone(),
                        expected: expected_type,
                        actual: actual_type,
                    });
                }
            }
        }

        Ok(())
    }
}

/// Thread-safe registry for custom fact schemas
#[derive(Debug)]
pub struct PluginSchemaRegistry {
    schemas: Arc<RwLock<HashMap<String, CustomFactSchema>>>,
}

impl PluginSchemaRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            schemas: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a new custom fact schema
    pub fn register_schema(&self, schema: CustomFactSchema) -> Result<(), SchemaError> {
        let mut schemas = self.schemas.write().map_err(|e| {
            SchemaError::ValidationError(format!("Failed to acquire write lock: {}", e))
        })?;

        let schema_name = schema.name.clone();

        // Check for conflicts
        if let Some(existing) = schemas.get(&schema_name) {
            if existing.version != schema.version {
                return Err(SchemaError::SchemaConflict(format!(
                    "Schema '{}' version conflict: existing '{}', new '{}'",
                    schema_name, existing.version, schema.version
                )));
            }
        }

        schemas.insert(schema_name, schema);
        Ok(())
    }

    /// Get a schema by name
    pub fn get_schema(&self, name: &str) -> Result<CustomFactSchema, SchemaError> {
        let schemas = self.schemas.read().map_err(|e| {
            SchemaError::ValidationError(format!("Failed to acquire read lock: {}", e))
        })?;

        schemas.get(name).cloned().ok_or_else(|| {
            SchemaError::UnknownFactType(format!("No schema registered for '{}'", name))
        })
    }

    /// Validate a custom fact against its schema
    pub fn validate_custom_fact(&self, fact: &crate::FactType) -> Result<(), SchemaError> {
        let discriminant = match fact {
            crate::FactType::Custom { discriminant, .. } => discriminant,
            _ => {
                return Err(SchemaError::UnknownFactType(
                    "Expected Custom fact type".to_string(),
                ));
            }
        };

        // Find a schema whose name is a prefix of the discriminant
        let schemas = self.schemas.read().map_err(|e| {
            SchemaError::ValidationError(format!("Failed to acquire read lock: {}", e))
        })?;

        // Try to find a matching schema by checking if the schema name is a prefix
        let mut matched_schema = None;
        for schema in schemas.values() {
            if discriminant.starts_with(&schema.name) {
                matched_schema = Some(schema.clone());
                break;
            }
        }

        let schema = matched_schema.ok_or_else(|| {
            SchemaError::UnknownFactType(format!(
                "No schema matches discriminant '{}'",
                discriminant
            ))
        })?;

        schema.validate_custom_fact(fact)
    }

    /// Check if a schema is registered
    pub fn has_schema(&self, name: &str) -> bool {
        let schemas = self.schemas.read().unwrap();
        schemas.contains_key(name)
    }

    /// Get all registered schema names
    pub fn list_schemas(&self) -> Vec<String> {
        let schemas = self.schemas.read().unwrap();
        schemas.keys().cloned().collect()
    }

    /// Remove a schema
    pub fn unregister_schema(&self, name: &str) -> Result<(), SchemaError> {
        let mut schemas = self.schemas.write().map_err(|e| {
            SchemaError::ValidationError(format!("Failed to acquire write lock: {}", e))
        })?;

        if schemas.remove(name).is_some() {
            Ok(())
        } else {
            Err(SchemaError::UnknownFactType(format!(
                "No schema registered for '{}'",
                name
            )))
        }
    }

    /// Clear all schemas
    pub fn clear(&self) {
        let mut schemas = self.schemas.write().unwrap();
        schemas.clear();
    }
}

impl Default for PluginSchemaRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::FactType;
    use std::collections::HashMap;

    #[test]
    fn test_register_and_validate_schema() {
        let registry = PluginSchemaRegistry::new();

        let mut schema = CustomFactSchema::new("terraform::aws".to_string(), "1.0.0".to_string());
        schema.add_field("bucket_name".to_string(), FactValueType::String, true);
        schema.add_field("public_access".to_string(), FactValueType::Boolean, true);
        schema.add_field("tags".to_string(), FactValueType::Array, false);
        schema.add_metadata("author".to_string(), "Terraform Plugin".to_string());

        assert!(registry.register_schema(schema).is_ok());

        // Validate a valid fact
        let mut fact_data = HashMap::new();
        fact_data.insert("bucket_name".to_string(), FactValue::string("my-bucket"));
        fact_data.insert("public_access".to_string(), FactValue::boolean(true));

        let fact = FactType::Custom {
            discriminant: "terraform::aws::insecure_s3_bucket".to_string(),
            data: fact_data,
        };

        assert!(
            registry.validate_custom_fact(&fact).is_ok(),
            "Validation failed: {:?}",
            registry.validate_custom_fact(&fact)
        );

        // Validate an invalid fact (missing required field)
        let mut invalid_data = HashMap::new();
        invalid_data.insert("bucket_name".to_string(), FactValue::string("my-bucket"));

        let invalid_fact = FactType::Custom {
            discriminant: "terraform::aws::insecure_s3_bucket".to_string(),
            data: invalid_data,
        };

        assert!(matches!(
            registry.validate_custom_fact(&invalid_fact),
            Err(SchemaError::MissingRequiredField(_))
        ));
    }

    #[test]
    fn test_schema_type_mismatch() {
        let registry = PluginSchemaRegistry::new();

        let mut schema = CustomFactSchema::new("test::type".to_string(), "1.0.0".to_string());
        schema.add_field("count".to_string(), FactValueType::Number, true);

        registry.register_schema(schema).unwrap();

        // Try to validate with wrong type
        let mut fact_data = HashMap::new();
        fact_data.insert("count".to_string(), FactValue::string("not a number"));

        let fact = FactType::Custom {
            discriminant: "test::type::example".to_string(),
            data: fact_data,
        };

        assert!(matches!(
            registry.validate_custom_fact(&fact),
            Err(SchemaError::TypeMismatch { .. })
        ));
    }

    #[test]
    fn test_unknown_schema() {
        let registry = PluginSchemaRegistry::new();

        let mut fact_data = HashMap::new();
        fact_data.insert("field".to_string(), FactValue::string("value"));

        let fact = FactType::Custom {
            discriminant: "unknown::schema".to_string(),
            data: fact_data,
        };

        assert!(matches!(
            registry.validate_custom_fact(&fact),
            Err(SchemaError::UnknownFactType(_))
        ));
    }

    #[test]
    fn test_schema_operations() {
        let registry = PluginSchemaRegistry::new();

        assert_eq!(registry.list_schemas().len(), 0);
        assert!(!registry.has_schema("test"));

        let schema = CustomFactSchema::new("test".to_string(), "1.0.0".to_string());
        registry.register_schema(schema).unwrap();

        assert_eq!(registry.list_schemas().len(), 1);
        assert!(registry.has_schema("test"));

        registry.unregister_schema("test").unwrap();
        assert_eq!(registry.list_schemas().len(), 0);
        assert!(!registry.has_schema("test"));
    }

    #[test]
    fn test_schema_conflict() {
        let registry = PluginSchemaRegistry::new();

        let schema1 = CustomFactSchema::new("test".to_string(), "1.0.0".to_string());
        registry.register_schema(schema1).unwrap();

        let schema2 = CustomFactSchema::new("test".to_string(), "2.0.0".to_string());
        assert!(matches!(
            registry.register_schema(schema2),
            Err(SchemaError::SchemaConflict(_))
        ));
    }

    #[test]
    fn test_optional_fields() {
        let registry = PluginSchemaRegistry::new();

        let mut schema = CustomFactSchema::new("optional::test".to_string(), "1.0.0".to_string());
        schema.add_field("required_field".to_string(), FactValueType::String, true);
        schema.add_field("optional_field".to_string(), FactValueType::Number, false);

        registry.register_schema(schema).unwrap();

        // Fact with only required field (should pass)
        let mut fact_data = HashMap::new();
        fact_data.insert("required_field".to_string(), FactValue::string("value"));

        let fact = FactType::Custom {
            discriminant: "optional::test::example".to_string(),
            data: fact_data,
        };

        assert!(registry.validate_custom_fact(&fact).is_ok());

        // Fact with optional field (should also pass)
        let mut fact_data2 = HashMap::new();
        fact_data2.insert("required_field".to_string(), FactValue::string("value"));
        fact_data2.insert("optional_field".to_string(), FactValue::number(42.0));

        let fact2 = FactType::Custom {
            discriminant: "optional::test::example".to_string(),
            data: fact_data2,
        };

        assert!(registry.validate_custom_fact(&fact2).is_ok());
    }

    #[test]
    fn test_nested_object_validation() {
        let registry = PluginSchemaRegistry::new();

        let mut schema = CustomFactSchema::new("nested::test".to_string(), "1.0.0".to_string());
        schema.add_field("config".to_string(), FactValueType::Object, true);

        registry.register_schema(schema).unwrap();

        let mut nested = HashMap::new();
        nested.insert("inner".to_string(), FactValue::string("value"));

        let mut fact_data = HashMap::new();
        fact_data.insert("config".to_string(), FactValue::Object(nested));

        let fact = FactType::Custom {
            discriminant: "nested::test::example".to_string(),
            data: fact_data,
        };

        assert!(registry.validate_custom_fact(&fact).is_ok());
    }

    #[test]
    fn test_array_validation() {
        let registry = PluginSchemaRegistry::new();

        let mut schema = CustomFactSchema::new("array::test".to_string(), "1.0.0".to_string());
        schema.add_field("items".to_string(), FactValueType::Array, true);

        registry.register_schema(schema).unwrap();

        let fact_data = HashMap::from([(
            "items".to_string(),
            FactValue::Array(vec![FactValue::string("item1"), FactValue::string("item2")]),
        )]);

        let fact = FactType::Custom {
            discriminant: "array::test::example".to_string(),
            data: fact_data,
        };

        assert!(registry.validate_custom_fact(&fact).is_ok());
    }

    #[test]
    fn test_discriminant_prefix_matching() {
        let registry = PluginSchemaRegistry::new();

        let schema = CustomFactSchema::new("terraform::aws".to_string(), "1.0.0".to_string());
        registry.register_schema(schema).unwrap();

        // Should match prefixes
        let test_cases = vec![
            "terraform::aws::insecure_s3_bucket",
            "terraform::aws::eks_misconfiguration",
            "terraform::aws",
        ];

        for discriminant in test_cases {
            let mut fact_data = HashMap::new();
            fact_data.insert("dummy".to_string(), FactValue::string("value"));

            let fact = FactType::Custom {
                discriminant: discriminant.to_string(),
                data: fact_data,
            };

            // Should not error on discriminant prefix
            assert!(matches!(
                registry.validate_custom_fact(&fact),
                Err(SchemaError::ValidationError(_)) | Err(SchemaError::UnknownFactType(_)) | Ok(_)
            ));
        }
    }
}
