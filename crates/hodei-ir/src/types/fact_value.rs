//! FactValue - Dynamic value types for custom facts

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Enumeration of supported value types
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum FactValue {
    /// String value
    String(String),
    /// Numeric value (floating point for flexibility)
    Number(f64),
    /// Boolean value
    Boolean(bool),
    /// Array of values
    Array(Vec<FactValue>),
    /// Object with string keys and FactValue values
    Object(HashMap<String, FactValue>),
}

impl std::hash::Hash for FactValue {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // Hash the discriminant
        std::mem::discriminant(self).hash(state);

        match self {
            FactValue::String(s) => s.hash(state),
            FactValue::Number(n) => {
                // Hash f64 as raw bytes
                let bits = n.to_bits();
                bits.hash(state);
            }
            FactValue::Boolean(b) => b.hash(state),
            FactValue::Array(a) => {
                for item in a {
                    item.hash(state);
                }
            }
            FactValue::Object(o) => {
                for (k, v) in o {
                    k.hash(state);
                    v.hash(state);
                }
            }
        }
    }
}

impl FactValue {
    /// Get the type of this value
    pub fn get_type(&self) -> FactValueType {
        match self {
            FactValue::String(_) => FactValueType::String,
            FactValue::Number(_) => FactValueType::Number,
            FactValue::Boolean(_) => FactValueType::Boolean,
            FactValue::Array(_) => FactValueType::Array,
            FactValue::Object(_) => FactValueType::Object,
        }
    }

    /// Get the string value if this is a String variant
    pub fn as_string(&self) -> Option<&str> {
        match self {
            FactValue::String(s) => Some(s),
            _ => None,
        }
    }

    /// Get the number value if this is a Number variant
    pub fn as_number(&self) -> Option<f64> {
        match self {
            FactValue::Number(n) => Some(*n),
            _ => None,
        }
    }

    /// Get the boolean value if this is a Boolean variant
    pub fn as_boolean(&self) -> Option<bool> {
        match self {
            FactValue::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    /// Get the array value if this is an Array variant
    pub fn as_array(&self) -> Option<&Vec<FactValue>> {
        match self {
            FactValue::Array(a) => Some(a),
            _ => None,
        }
    }

    /// Get the object value if this is an Object variant
    pub fn as_object(&self) -> Option<&HashMap<String, FactValue>> {
        match self {
            FactValue::Object(o) => Some(o),
            _ => None,
        }
    }

    /// Create a string FactValue
    pub fn string<S: Into<String>>(value: S) -> Self {
        FactValue::String(value.into())
    }

    /// Create a number FactValue
    pub fn number(value: f64) -> Self {
        FactValue::Number(value)
    }

    /// Create a boolean FactValue
    pub fn boolean(value: bool) -> Self {
        FactValue::Boolean(value)
    }

    /// Create an empty array FactValue
    pub fn array() -> Self {
        FactValue::Array(Vec::new())
    }

    /// Create an empty object FactValue
    pub fn object() -> Self {
        FactValue::Object(HashMap::new())
    }
}

impl std::fmt::Display for FactValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FactValue::String(s) => write!(f, "{}", s),
            FactValue::Number(n) => write!(f, "{}", n),
            FactValue::Boolean(b) => write!(f, "{}", b),
            FactValue::Array(a) => write!(f, "{:?}", a),
            FactValue::Object(o) => write!(f, "{:?}", o),
        }
    }
}

/// Type information for FactValue
#[derive(Debug, Clone, PartialEq, Hash, serde::Serialize, serde:: Deserialize)]
pub enum FactValueType {
    String,
    Number,
    Boolean,
    Array,
    Object,
}

impl std::fmt::Display for FactValueType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FactValueType::String => write!(f, "string"),
            FactValueType::Number => write!(f, "number"),
            FactValueType::Boolean => write!(f, "boolean"),
            FactValueType::Array => write!(f, "array"),
            FactValueType::Object => write!(f, "object"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_fact_value_string() {
        let value = FactValue::string("test");
        assert_eq!(value.get_type(), FactValueType::String);
        assert_eq!(value.as_string(), Some("test"));
    }

    #[test]
    fn test_fact_value_number() {
        let value = FactValue::number(42.5);
        assert_eq!(value.get_type(), FactValueType::Number);
        assert_eq!(value.as_number(), Some(42.5));
    }

    #[test]
    fn test_fact_value_boolean() {
        let value = FactValue::boolean(true);
        assert_eq!(value.get_type(), FactValueType::Boolean);
        assert_eq!(value.as_boolean(), Some(true));
    }

    #[test]
    fn test_fact_value_array() {
        let value = FactValue::array();
        assert_eq!(value.get_type(), FactValueType::Array);
        assert_eq!(value.as_array(), Some(&Vec::new()));
    }

    #[test]
    fn test_fact_value_object() {
        let value = FactValue::object();
        assert_eq!(value.get_type(), FactValueType::Object);
        assert_eq!(value.as_object(), Some(&HashMap::new()));
    }

    #[test]
    fn test_complex_fact_value() {
        let mut obj = HashMap::new();
        obj.insert("key1".to_string(), FactValue::string("value1"));
        obj.insert("key2".to_string(), FactValue::number(42.0));

        let value = FactValue::Object(obj);
        assert_eq!(value.get_type(), FactValueType::Object);

        let obj_ref = value.as_object().unwrap();
        assert_eq!(obj_ref.get("key1").unwrap().as_string(), Some("value1"));
        assert_eq!(obj_ref.get("key2").unwrap().as_number(), Some(42.0));
    }

    #[test]
    fn test_fact_value_display() {
        assert_eq!(FactValue::string("test").to_string(), "test");
        assert_eq!(FactValue::number(42.5).to_string(), "42.5");
        assert_eq!(FactValue::boolean(true).to_string(), "true");
    }

    #[test]
    fn test_fact_value_type_display() {
        assert_eq!(FactValueType::String.to_string(), "string");
        assert_eq!(FactValueType::Number.to_string(), "number");
        assert_eq!(FactValueType::Boolean.to_string(), "boolean");
        assert_eq!(FactValueType::Array.to_string(), "array");
        assert_eq!(FactValueType::Object.to_string(), "object");
    }
}
