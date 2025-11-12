//! Integration tests for Custom FactTypes end-to-end workflows
//!
//! This test suite validates the complete lifecycle of Custom FactTypes:
//! - Schema registration and validation
//! - Custom fact creation and manipulation
//! - IR serialization and deserialization
//! - Version migration
//! - Real-world plugin scenarios

use hodei_ir::*;
use std::collections::HashMap;
use std::path::PathBuf;

/// Test complete workflow: register schema, create facts, validate
#[test]
fn test_end_to_end_custom_fact_workflow() {
    // Step 1: Create registry and register schema
    let registry = PluginSchemaRegistry::new();

    let mut schema = CustomFactSchema::new("terraform::aws".to_string(), "1.0.0".to_string());
    schema.add_field("resource_type".to_string(), FactValueType::String, true);
    schema.add_field("resource_name".to_string(), FactValueType::String, true);
    schema.add_field("severity".to_string(), FactValueType::String, true);
    schema.add_field("tags".to_string(), FactValueType::Array, false);
    schema.add_metadata("plugin_author".to_string(), "Hodei Team".to_string());

    assert!(registry.register_schema(schema).is_ok());

    // Step 2: Create custom facts
    let mut fact_data = HashMap::new();
    fact_data.insert(
        "resource_type".to_string(),
        FactValue::String("s3_bucket".to_string()),
    );
    fact_data.insert(
        "resource_name".to_string(),
        FactValue::String("my-public-bucket".to_string()),
    );
    fact_data.insert(
        "severity".to_string(),
        FactValue::String("high".to_string()),
    );
    fact_data.insert(
        "tags".to_string(),
        FactValue::Array(vec![
            FactValue::String("public".to_string()),
            FactValue::String("unencrypted".to_string()),
        ]),
    );

    let custom_fact = FactType::Custom {
        discriminant: "terraform::aws::insecure_s3_bucket".to_string(),
        data: fact_data,
    };

    // Step 3: Validate against schema
    assert!(
        registry.validate_custom_fact(&custom_fact).is_ok(),
        "Validation should pass for valid custom fact"
    );

    // Step 4: Create IR with custom facts
    let location = SourceLocation::new(
        ProjectPath::new(PathBuf::from("main.tf")),
        LineNumber::new(10).unwrap(),
        Some(ColumnNumber::new(1).unwrap()),
        LineNumber::new(15).unwrap(),
        Some(ColumnNumber::new(1).unwrap()),
    );

    let provenance = Provenance::new(
        ExtractorId::Custom,
        "terraform-plugin-1.0.0".to_string(),
        Confidence::HIGH,
    );

    let fact = Fact::new(custom_fact.clone(), location, provenance);

    let metadata = ProjectMetadata::new(
        "terraform-project".to_string(),
        "1.0.0".to_string(),
        ProjectPath::new(PathBuf::from("/project")),
    );

    let ir = IntermediateRepresentation {
        facts: vec![fact],
        metadata,
        schema_version: "3.3.0".to_string(),
    };

    // Step 5: Serialize and deserialize
    let json = serde_json::to_string(&ir).expect("Serialization should succeed");
    let deserialized: IntermediateRepresentation =
        serde_json::from_str(&json).expect("Deserialization should succeed");

    assert_eq!(deserialized.facts.len(), 1);
    assert_eq!(deserialized.schema_version, "3.3.0");

    // Step 6: Validate deserialized custom fact
    match &deserialized.facts[0].fact_type {
        FactType::Custom { discriminant, data } => {
            assert_eq!(discriminant, "terraform::aws::insecure_s3_bucket");
            assert_eq!(
                data.get("resource_type").unwrap().as_string(),
                Some("s3_bucket")
            );
            assert!(
                registry
                    .validate_custom_fact(&deserialized.facts[0].fact_type)
                    .is_ok()
            );
        }
        _ => panic!("Expected Custom fact type"),
    }
}

/// Test multiple plugin schemas coexisting
#[test]
fn test_multiple_plugin_schemas() {
    let registry = PluginSchemaRegistry::new();

    // Register Terraform plugin schema
    let mut terraform_schema =
        CustomFactSchema::new("terraform::aws".to_string(), "1.0.0".to_string());
    terraform_schema.add_field("resource".to_string(), FactValueType::String, true);
    registry.register_schema(terraform_schema).unwrap();

    // Register Kubernetes plugin schema
    let mut k8s_schema =
        CustomFactSchema::new("kubernetes::security".to_string(), "2.0.0".to_string());
    k8s_schema.add_field("kind".to_string(), FactValueType::String, true);
    k8s_schema.add_field("namespace".to_string(), FactValueType::String, true);
    registry.register_schema(k8s_schema).unwrap();

    // Register Docker plugin schema
    let mut docker_schema = CustomFactSchema::new("docker::image".to_string(), "1.5.0".to_string());
    docker_schema.add_field("image_name".to_string(), FactValueType::String, true);
    docker_schema.add_field("vulnerabilities".to_string(), FactValueType::Array, false);
    registry.register_schema(docker_schema).unwrap();

    assert_eq!(registry.list_schemas().len(), 3);
    assert!(registry.has_schema("terraform::aws"));
    assert!(registry.has_schema("kubernetes::security"));
    assert!(registry.has_schema("docker::image"));

    // Validate facts from different plugins
    let terraform_fact = FactType::Custom {
        discriminant: "terraform::aws::example".to_string(),
        data: HashMap::from([("resource".to_string(), FactValue::String("s3".to_string()))]),
    };

    let k8s_fact = FactType::Custom {
        discriminant: "kubernetes::security::pod_misconfiguration".to_string(),
        data: HashMap::from([
            ("kind".to_string(), FactValue::String("Pod".to_string())),
            (
                "namespace".to_string(),
                FactValue::String("default".to_string()),
            ),
        ]),
    };

    assert!(registry.validate_custom_fact(&terraform_fact).is_ok());
    assert!(registry.validate_custom_fact(&k8s_fact).is_ok());
}

/// Test IR version migration with custom facts
#[test]
fn test_migration_with_custom_facts() {
    let location = SourceLocation::new(
        ProjectPath::new(PathBuf::from("test.tf")),
        LineNumber::new(1).unwrap(),
        None,
        LineNumber::new(1).unwrap(),
        None,
    );

    let provenance = Provenance::new(
        ExtractorId::Custom,
        "test-plugin".to_string(),
        Confidence::MEDIUM,
    );

    let custom_fact = FactType::Custom {
        discriminant: "test::plugin::finding".to_string(),
        data: HashMap::from([
            (
                "severity".to_string(),
                FactValue::String("high".to_string()),
            ),
            ("confidence".to_string(), FactValue::Number(0.9)),
        ]),
    };

    let fact = Fact::new(custom_fact, location, provenance);

    let metadata = ProjectMetadata::new(
        "test-project".to_string(),
        "1.0.0".to_string(),
        ProjectPath::new(PathBuf::from("/test")),
    );

    // Create IR with v3.2.0 schema
    let ir_v32 = IntermediateRepresentation {
        facts: vec![fact.clone()],
        metadata: metadata.clone(),
        schema_version: "3.2.0".to_string(),
    };

    // Migrate to v3.3.0
    let ir_v33 = migrate_ir_version(&ir_v32).expect("Migration should succeed");

    assert_eq!(ir_v33.schema_version, "3.3.0");
    assert_eq!(ir_v33.facts.len(), 1);

    // Verify custom fact is preserved
    match &ir_v33.facts[0].fact_type {
        FactType::Custom { discriminant, data } => {
            assert_eq!(discriminant, "test::plugin::finding");
            assert_eq!(data.get("severity").unwrap().as_string(), Some("high"));
            assert_eq!(data.get("confidence").unwrap().as_number(), Some(0.9));
        }
        _ => panic!("Expected Custom fact type after migration"),
    }
}

/// Test complex nested data structures in custom facts
#[test]
fn test_complex_nested_custom_fact() {
    let registry = PluginSchemaRegistry::new();

    let mut schema = CustomFactSchema::new("security::scan".to_string(), "1.0.0".to_string());
    schema.add_field("scan_result".to_string(), FactValueType::Object, true);
    schema.add_field("findings".to_string(), FactValueType::Array, true);
    registry.register_schema(schema).unwrap();

    // Create nested structure
    let mut scan_result = HashMap::new();
    scan_result.insert(
        "status".to_string(),
        FactValue::String("completed".to_string()),
    );
    scan_result.insert("duration_ms".to_string(), FactValue::Number(1234.5));
    scan_result.insert("success".to_string(), FactValue::Boolean(true));

    let findings = vec![
        FactValue::String("CVE-2023-1234".to_string()),
        FactValue::String("CVE-2023-5678".to_string()),
    ];

    let custom_fact = FactType::Custom {
        discriminant: "security::scan::result".to_string(),
        data: HashMap::from([
            ("scan_result".to_string(), FactValue::Object(scan_result)),
            ("findings".to_string(), FactValue::Array(findings)),
        ]),
    };

    assert!(registry.validate_custom_fact(&custom_fact).is_ok());

    // Verify field access
    let scan_obj = custom_fact.get_field("scan_result").unwrap();
    assert_eq!(scan_obj.get_type(), FactValueType::Object);

    let findings_arr = custom_fact.get_field("findings").unwrap();
    assert_eq!(findings_arr.get_type(), FactValueType::Array);
    assert_eq!(findings_arr.as_array().unwrap().len(), 2);
}

/// Test fact type indexing with custom facts
#[test]
fn test_custom_fact_indexing() {
    let mut index = FactTypeIndex::new();

    // Create multiple custom facts
    for i in 0..10 {
        let custom = FactType::Custom {
            discriminant: format!("plugin::type{}", i),
            data: HashMap::new(),
        };

        let location = SourceLocation::new(
            ProjectPath::new(PathBuf::from("test.rs")),
            LineNumber::new(i + 1).unwrap(),
            None,
            LineNumber::new(i + 1).unwrap(),
            None,
        );

        let provenance = Provenance::new(ExtractorId::Custom, "test".to_string(), Confidence::HIGH);

        let fact = Fact::new(custom, location, provenance);
        index.insert(fact.fact_type.discriminant(), fact.id);
    }

    // Verify all custom facts are indexed
    let custom_ids = index.get_by_type(FactTypeDiscriminant::Custom);
    assert_eq!(custom_ids.len(), 10);
    assert_eq!(index.count_by_type(FactTypeDiscriminant::Custom), 10);
}

/// Test schema validation errors
#[test]
fn test_validation_error_scenarios() {
    let registry = PluginSchemaRegistry::new();

    let mut schema = CustomFactSchema::new("validation::test".to_string(), "1.0.0".to_string());
    schema.add_field("required_field".to_string(), FactValueType::String, true);
    schema.add_field("number_field".to_string(), FactValueType::Number, true);
    registry.register_schema(schema).unwrap();

    // Test 1: Missing required field
    let missing_field = FactType::Custom {
        discriminant: "validation::test::example".to_string(),
        data: HashMap::from([("number_field".to_string(), FactValue::Number(42.0))]),
    };

    assert!(matches!(
        registry.validate_custom_fact(&missing_field),
        Err(SchemaError::MissingRequiredField(_))
    ));

    // Test 2: Type mismatch
    let wrong_type = FactType::Custom {
        discriminant: "validation::test::example".to_string(),
        data: HashMap::from([
            (
                "required_field".to_string(),
                FactValue::String("valid".to_string()),
            ),
            (
                "number_field".to_string(),
                FactValue::String("not a number".to_string()),
            ),
        ]),
    };

    assert!(matches!(
        registry.validate_custom_fact(&wrong_type),
        Err(SchemaError::TypeMismatch { .. })
    ));

    // Test 3: Unknown schema
    let unknown_schema = FactType::Custom {
        discriminant: "unknown::plugin::fact".to_string(),
        data: HashMap::new(),
    };

    assert!(matches!(
        registry.validate_custom_fact(&unknown_schema),
        Err(SchemaError::UnknownFactType(_))
    ));
}

/// Test real-world plugin scenario: Terraform security scanner
#[test]
fn test_terraform_plugin_scenario() {
    let registry = PluginSchemaRegistry::new();

    // Register Terraform security findings schema
    let mut schema = CustomFactSchema::new("terraform::security".to_string(), "2.1.0".to_string());
    schema.add_field("rule_id".to_string(), FactValueType::String, true);
    schema.add_field("resource_type".to_string(), FactValueType::String, true);
    schema.add_field("resource_name".to_string(), FactValueType::String, true);
    schema.add_field("severity".to_string(), FactValueType::String, true);
    schema.add_field("description".to_string(), FactValueType::String, true);
    schema.add_field("remediation".to_string(), FactValueType::String, false);
    schema.add_field("references".to_string(), FactValueType::Array, false);
    schema.add_metadata(
        "plugin_name".to_string(),
        "terraform-security-scanner".to_string(),
    );
    schema.add_metadata("plugin_version".to_string(), "2.1.0".to_string());

    registry.register_schema(schema).unwrap();

    // Create multiple findings
    let findings = vec![
        (
            "TF-001",
            "s3_bucket",
            "public-data-bucket",
            "CRITICAL",
            "S3 bucket allows public access",
            Some("Add bucket policy to restrict access"),
        ),
        (
            "TF-002",
            "security_group",
            "web-sg",
            "HIGH",
            "Security group allows unrestricted SSH access",
            Some("Restrict SSH to specific IPs"),
        ),
        (
            "TF-003",
            "iam_policy",
            "admin-policy",
            "MEDIUM",
            "IAM policy grants excessive permissions",
            None,
        ),
    ];

    let mut ir_facts = Vec::new();

    for (i, (rule_id, res_type, res_name, severity, desc, remediation)) in
        findings.iter().enumerate()
    {
        let mut data = HashMap::new();
        data.insert(
            "rule_id".to_string(),
            FactValue::String(rule_id.to_string()),
        );
        data.insert(
            "resource_type".to_string(),
            FactValue::String(res_type.to_string()),
        );
        data.insert(
            "resource_name".to_string(),
            FactValue::String(res_name.to_string()),
        );
        data.insert(
            "severity".to_string(),
            FactValue::String(severity.to_string()),
        );
        data.insert(
            "description".to_string(),
            FactValue::String(desc.to_string()),
        );

        if let Some(rem) = remediation {
            data.insert(
                "remediation".to_string(),
                FactValue::String(rem.to_string()),
            );
        }

        let custom_fact = FactType::Custom {
            discriminant: format!("terraform::security::{}", rule_id.to_lowercase()),
            data,
        };

        // Validate each fact
        assert!(
            registry.validate_custom_fact(&custom_fact).is_ok(),
            "Validation failed for finding {}",
            rule_id
        );

        let location = SourceLocation::new(
            ProjectPath::new(PathBuf::from("main.tf")),
            LineNumber::new((i * 10 + 1) as u32).unwrap(),
            None,
            LineNumber::new((i * 10 + 5) as u32).unwrap(),
            None,
        );

        let provenance = Provenance::new(
            ExtractorId::Custom,
            "terraform-security-scanner-2.1.0".to_string(),
            Confidence::HIGH,
        );

        ir_facts.push(Fact::new(custom_fact, location, provenance));
    }

    // Create IR
    let metadata = ProjectMetadata::new(
        "infrastructure-repo".to_string(),
        "1.0.0".to_string(),
        ProjectPath::new(PathBuf::from("/infrastructure")),
    );

    let ir = IntermediateRepresentation {
        facts: ir_facts,
        metadata,
        schema_version: "3.3.0".to_string(),
    };

    assert_eq!(ir.facts.len(), 3);

    // Verify all facts are valid custom facts
    for fact in &ir.facts {
        assert_eq!(fact.fact_type.discriminant(), FactTypeDiscriminant::Custom);
        assert!(registry.validate_custom_fact(&fact.fact_type).is_ok());
    }
}

/// Test performance: bulk validation of custom facts
#[test]
fn test_bulk_custom_fact_validation() {
    let registry = PluginSchemaRegistry::new();

    let mut schema = CustomFactSchema::new("bulk::test".to_string(), "1.0.0".to_string());
    schema.add_field("id".to_string(), FactValueType::Number, true);
    schema.add_field("name".to_string(), FactValueType::String, true);
    registry.register_schema(schema).unwrap();

    // Create and validate 1000 facts
    let facts: Vec<FactType> = (0..1000)
        .map(|i| FactType::Custom {
            discriminant: "bulk::test::item".to_string(),
            data: HashMap::from([
                ("id".to_string(), FactValue::Number(i as f64)),
                ("name".to_string(), FactValue::String(format!("item_{}", i))),
            ]),
        })
        .collect();

    // Validate all facts
    for fact in &facts {
        assert!(registry.validate_custom_fact(fact).is_ok());
    }
}
