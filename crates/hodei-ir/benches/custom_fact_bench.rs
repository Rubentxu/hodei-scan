//! Benchmarks for Custom FactTypes
//!
//! This benchmark suite measures the performance characteristics of Custom FactTypes
//! compared to standard FactTypes.

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use hodei_ir::*;
use std::collections::HashMap;

/// Simplified benchmark group for Custom FactType operations
fn custom_fact_benchmarks(c: &mut Criterion) {
    // Benchmark 1: Creating Custom FactTypes
    c.bench_function("custom_fact_creation", |b| {
        b.iter(|| {
            let mut data = HashMap::new();
            data.insert("name".to_string(), FactValue::String("test".to_string()));
            data.insert("count".to_string(), FactValue::Number(42.0));
            data.insert("active".to_string(), FactValue::Boolean(true));
            black_box(FactType::Custom {
                discriminant: "benchmark:test".to_string(),
                data,
            })
        })
    });

    // Benchmark 2: Field access in Custom FactTypes
    c.bench_function("custom_fact_field_access", |b| {
        let custom = create_test_fact();
        b.iter(|| black_box(custom.get_field("test_field")))
    });

    // Benchmark 3: Hashing performance
    c.bench_function("custom_fact_hashing", |b| {
        let custom = create_test_fact();
        b.iter(|| {
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            let mut hasher = DefaultHasher::new();
            custom.hash(&mut hasher);
            hasher.finish()
        })
    });

    // Benchmark 4: Cloning performance
    c.bench_function("custom_fact_cloning", |b| {
        let custom = create_test_fact();
        b.iter(|| black_box(custom.clone()))
    });

    // Benchmark 5: Pattern matching performance
    c.bench_function("pattern_matching", |b| {
        let custom = create_test_fact();
        b.iter(|| {
            black_box(match &custom {
                FactType::Custom { discriminant, .. } => discriminant == "benchmark:test",
                _ => false,
            })
        })
    });

    // Benchmark 6: Comparison with standard types
    c.bench_function("standard_vs_custom_creation", |b| {
        // Create standard Function fact
        b.iter(|| {
            black_box(FactType::Function {
                name: FunctionName("test_function".to_string()),
                complexity: 10,
                lines_of_code: 50,
            })
        })
    });

    // Benchmark 7: Schema validation (simplified)
    c.bench_function("schema_validation", |b| {
        use hodei_ir::PluginSchemaRegistry;

        let registry = PluginSchemaRegistry::new();

        // Setup schema
        let mut schema = CustomFactSchema::new("benchmark:test".to_string(), "1.0.0".to_string());
        schema.add_field("name".to_string(), FactValueType::String, true);
        schema.add_field("count".to_string(), FactValueType::Number, true);
        registry.register_schema(schema).unwrap();

        let test_fact = FactType::Custom {
            discriminant: "benchmark:test".to_string(),
            data: HashMap::from([
                ("name".to_string(), FactValue::String("test".to_string())),
                ("count".to_string(), FactValue::Number(42.0)),
            ]),
        };

        b.iter(|| black_box(registry.validate_custom_fact(&test_fact)))
    });
}

/// Helper function to create a test Custom FactType
fn create_test_fact() -> FactType {
    let mut data = HashMap::new();
    data.insert(
        "test_field".to_string(),
        FactValue::String("value".to_string()),
    );
    data.insert("number_field".to_string(), FactValue::Number(42.0));
    data.insert("bool_field".to_string(), FactValue::Boolean(true));

    FactType::Custom {
        discriminant: "benchmark:test".to_string(),
        data,
    }
}

criterion_group!(benches, custom_fact_benchmarks);
criterion_main!(benches);
