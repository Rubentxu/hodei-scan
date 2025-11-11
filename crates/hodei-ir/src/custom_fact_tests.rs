#[cfg(test)]
mod custom_fact_type_tests {
    use crate::{
        Confidence, FactId, FactType, FactTypeDiscriminant, FactTypeIndex, FactValue,
        FactValueType, Severity,
    };
    use std::collections::HashMap;

    #[test]
    fn test_create_custom_fact() {
        let custom = FactType::Custom {
            discriminant: "terraform::aws::insecure_s3_bucket".to_string(),
            data: HashMap::from([
                ("bucket_name".to_string(), FactValue::string("my-bucket")),
                ("acl".to_string(), FactValue::string("public-read")),
                ("public_access".to_string(), FactValue::boolean(true)),
            ]),
        };

        assert_eq!(custom.discriminant(), FactTypeDiscriminant::Custom);
        assert_eq!(
            custom.get_discriminant().unwrap(),
            "terraform::aws::insecure_s3_bucket"
        );
        assert_eq!(
            custom.get_field("acl").unwrap().as_string(),
            Some("public-read")
        );
        assert_eq!(
            custom.get_field("public_access").unwrap().as_boolean(),
            Some(true)
        );
    }

    #[test]
    fn test_custom_fact_with_nested_object() {
        let mut nested = HashMap::new();
        nested.insert("inner_key".to_string(), FactValue::string("inner_value"));

        let mut data = HashMap::new();
        data.insert("nested_object".to_string(), FactValue::Object(nested));
        data.insert("count".to_string(), FactValue::number(42.0));

        let custom = FactType::Custom {
            discriminant: "test::nested".to_string(),
            data,
        };

        let nested_value = custom.get_field("nested_object").unwrap();
        assert_eq!(nested_value.get_type(), FactValueType::Object);

        let nested_obj = nested_value.as_object().unwrap();
        assert_eq!(
            nested_obj.get("inner_key").unwrap().as_string(),
            Some("inner_value")
        );

        assert_eq!(custom.get_field("count").unwrap().as_number(), Some(42.0));
    }

    #[test]
    fn test_custom_fact_with_array() {
        let mut data = HashMap::new();
        data.insert(
            "tags".to_string(),
            FactValue::Array(vec![
                FactValue::string("tag1"),
                FactValue::string("tag2"),
                FactValue::string("tag3"),
            ]),
        );

        let custom = FactType::Custom {
            discriminant: "test::array".to_string(),
            data,
        };

        let array_value = custom.get_field("tags").unwrap();
        assert_eq!(array_value.get_type(), FactValueType::Array);

        let array = array_value.as_array().unwrap();
        assert_eq!(array.len(), 3);
        assert_eq!(array[0].as_string(), Some("tag1"));
        assert_eq!(array[1].as_string(), Some("tag2"));
        assert_eq!(array[2].as_string(), Some("tag3"));
    }

    #[test]
    fn test_custom_fact_discriminant_matching() {
        let custom1 = FactType::Custom {
            discriminant: "terraform::aws::insecure_s3_bucket".to_string(),
            data: HashMap::new(),
        };

        let custom2 = FactType::Custom {
            discriminant: "kubernetes::misconfiguration".to_string(),
            data: HashMap::new(),
        };

        assert_ne!(
            custom1.get_discriminant().unwrap(),
            custom2.get_discriminant().unwrap()
        );

        assert!(custom1.get_discriminant().unwrap().starts_with("terraform"));
        assert!(
            custom2
                .get_discriminant()
                .unwrap()
                .starts_with("kubernetes")
        );
    }

    #[test]
    fn test_custom_fact_type_index() {
        let mut index = FactTypeIndex::new();
        let custom1 = FactType::Custom {
            discriminant: "test::custom1".to_string(),
            data: HashMap::new(),
        };
        let custom2 = FactType::Custom {
            discriminant: "test::custom2".to_string(),
            data: HashMap::new(),
        };

        let fact_id1 = FactId::new();
        let fact_id2 = FactId::new();

        index.insert(FactTypeDiscriminant::Custom, fact_id1);
        index.insert(FactTypeDiscriminant::Custom, fact_id2);

        let custom_ids = index.get_by_type(FactTypeDiscriminant::Custom);
        assert_eq!(custom_ids.len(), 2);
        assert_eq!(index.count_by_type(FactTypeDiscriminant::Custom), 2);
    }

    #[test]
    fn test_custom_fact_serialization() {
        let custom = FactType::Custom {
            discriminant: "test::serialization".to_string(),
            data: HashMap::from([
                ("string_field".to_string(), FactValue::string("test")),
                ("number_field".to_string(), FactValue::number(123.45)),
                ("bool_field".to_string(), FactValue::boolean(true)),
            ]),
        };

        // Serialize to JSON
        let json = serde_json::to_string(&custom).unwrap();
        let deserialized: FactType = serde_json::from_str(&json).unwrap();

        match &deserialized {
            FactType::Custom { discriminant, data } => {
                assert_eq!(discriminant, "test::serialization");
                assert_eq!(data.get("string_field").unwrap().as_string(), Some("test"));
                assert_eq!(data.get("number_field").unwrap().as_number(), Some(123.45));
                assert_eq!(data.get("bool_field").unwrap().as_boolean(), Some(true));
            }
            _ => panic!("Expected Custom fact type"),
        }
    }

    #[test]
    fn test_custom_fact_complex_data_structure() {
        let mut nested_array = Vec::new();
        nested_array.push(FactValue::number(1.0));
        nested_array.push(FactValue::number(2.0));
        nested_array.push(FactValue::number(3.0));

        let mut nested_object = HashMap::new();
        nested_object.insert(
            "nested".to_string(),
            FactValue::String("nested_value".to_string()),
        );

        let mut complex_data = HashMap::new();
        complex_data.insert("array".to_string(), FactValue::Array(nested_array));
        complex_data.insert("object".to_string(), FactValue::Object(nested_object));
        complex_data.insert("string".to_string(), FactValue::String("test".to_string()));
        complex_data.insert("number".to_string(), FactValue::Number(42.0));
        complex_data.insert("bool".to_string(), FactValue::Boolean(false));

        let custom = FactType::Custom {
            discriminant: "test::complex".to_string(),
            data: complex_data,
        };

        // Verify all fields
        let array = custom.get_field("array").unwrap().as_array().unwrap();
        assert_eq!(array.len(), 3);

        let object = custom.get_field("object").unwrap().as_object().unwrap();
        assert_eq!(
            object.get("nested").unwrap().as_string(),
            Some("nested_value")
        );

        assert_eq!(
            custom.get_field("string").unwrap().as_string(),
            Some("test")
        );
        assert_eq!(custom.get_field("number").unwrap().as_number(), Some(42.0));
        assert_eq!(custom.get_field("bool").unwrap().as_boolean(), Some(false));
    }

    #[test]
    fn test_custom_fact_equality() {
        let mut data1 = HashMap::new();
        data1.insert("key".to_string(), FactValue::string("value"));

        let mut data2 = HashMap::new();
        data2.insert("key".to_string(), FactValue::string("value"));

        let custom1 = FactType::Custom {
            discriminant: "test::equal".to_string(),
            data: data1,
        };

        let custom2 = FactType::Custom {
            discriminant: "test::equal".to_string(),
            data: data2,
        };

        assert_eq!(custom1, custom2);
    }

    #[test]
    fn test_custom_fact_inequality_different_discriminant() {
        let custom1 = FactType::Custom {
            discriminant: "test::type1".to_string(),
            data: HashMap::new(),
        };

        let custom2 = FactType::Custom {
            discriminant: "test::type2".to_string(),
            data: HashMap::new(),
        };

        assert_ne!(custom1, custom2);
    }

    #[test]
    fn test_custom_fact_inequality_different_data() {
        let mut data1 = HashMap::new();
        data1.insert("key".to_string(), FactValue::string("value1"));

        let mut data2 = HashMap::new();
        data2.insert("key".to_string(), FactValue::string("value2"));

        let custom1 = FactType::Custom {
            discriminant: "test::same".to_string(),
            data: data1,
        };

        let custom2 = FactType::Custom {
            discriminant: "test::same".to_string(),
            data: data2,
        };

        assert_ne!(custom1, custom2);
    }

    #[test]
    fn test_custom_fact_get_field_nonexistent() {
        let custom = FactType::Custom {
            discriminant: "test::nonexistent".to_string(),
            data: HashMap::from([("existing".to_string(), FactValue::string("value"))]),
        };

        assert_eq!(custom.get_field("nonexistent"), None);
        assert_eq!(custom.get_field(""), None);
    }

    #[test]
    fn test_custom_fact_get_discriminant_none() {
        let vulnerability = FactType::Vulnerability {
            cwe_id: None,
            owasp_category: None,
            severity: Severity::Major,
            cvss_score: None,
            description: "test".to_string(),
            confidence: Confidence::HIGH,
        };

        assert_eq!(vulnerability.get_discriminant(), None);
        assert_eq!(vulnerability.get_field("any_field"), None);
    }

    #[test]
    fn test_custom_fact_with_many_fields() {
        let mut data = HashMap::new();
        for i in 0..100 {
            data.insert(format!("field_{}", i), FactValue::number(i as f64));
        }

        let custom = FactType::Custom {
            discriminant: "test::many_fields".to_string(),
            data,
        };

        assert_eq!(custom.get_field("field_0").unwrap().as_number(), Some(0.0));
        assert_eq!(
            custom.get_field("field_99").unwrap().as_number(),
            Some(99.0)
        );
        assert_eq!(custom.get_field("field_100"), None);
    }
}
