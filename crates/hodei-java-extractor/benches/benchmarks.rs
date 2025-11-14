//! Performance Benchmarks for Java Extractor
//!
//! This module contains performance benchmarks for all three extraction levels.

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use hodei_java_extractor::{ExtractionLevel, JacocoAdapter, JavaAnalysisConfig};
use std::path::PathBuf;

fn bench_jacoco_parsing(c: &mut Criterion) {
    let mut adapter = JacocoAdapter::new(PathBuf::from(
        "/home/rubentxu/Proyectos/rust/hodei-scan/test-java-project/jacoco.xml",
    ));

    c.bench_function("jacoco_parse_xml", |b| {
        b.iter(|| adapter.load_coverage_data())
    });
}

fn bench_jacoco_large_file_parsing(c: &mut Criterion) {
    // Create a temporary large XML file for benchmarking
    let temp_dir = std::env::temp_dir();
    let large_jacoco_file = temp_dir.join("large-jacoco.xml");

    // Generate a large JaCoCo XML with many classes
    let mut xml_content = String::from(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<report name="Large JaCoCo Report">
"#,
    );

    for i in 0..100 {
        xml_content.push_str(&format!(
            r#"  <package name="com/example{}">
    <class name="com/example{}/TestClass{}" sourcefilename="TestClass{}.java">
      <method name="testMethod" desc="()V" line="1">
        <line nr="1" mi="0" ci="10" mb="0" cb="2"/>
        <line nr="2" mi="5" ci="5" mb="1" cb="1"/>
      </method>
    </class>
  </package>
"#,
            i, i, i, i
        ));
    }

    xml_content.push_str("</report>\n");

    std::fs::write(&large_jacoco_file, &xml_content).unwrap();

    let mut adapter = JacocoAdapter::new(large_jacoco_file.clone());

    c.bench_function("jacoco_parse_large_xml", |b| {
        b.iter(|| adapter.load_coverage_data())
    });

    // Cleanup
    std::fs::remove_file(&large_jacoco_file).ok();
}

fn bench_jacoco_adapter_creation(c: &mut Criterion) {
    let jacoco_path =
        PathBuf::from("/home/rubentxu/Proyectos/rust/hodei-scan/test-java-project/jacoco.xml");

    c.bench_function("jacoco_adapter_creation", |b| {
        b.iter(|| JacocoAdapter::new(black_box(jacoco_path.clone())))
    });
}

fn bench_memory_usage_jacoco(c: &mut Criterion) {
    let mut adapter = JacocoAdapter::new(PathBuf::from(
        "/home/rubentxu/Proyectos/rust/hodei-scan/test-java-project/jacoco.xml",
    ));

    // Benchmark memory allocation during parsing
    c.bench_function("jacoco_memory_allocation", |b| {
        b.iter(|| {
            let result = adapter.load_coverage_data();
            black_box(result);
        })
    });
}

fn bench_extraction_level_enum_operations(c: &mut Criterion) {
    c.bench_function("extraction_level_as_str", |b| {
        b.iter(|| {
            black_box(ExtractionLevel::Level1.as_str());
            black_box(ExtractionLevel::Level2.as_str());
            black_box(ExtractionLevel::Level3.as_str());
        })
    });

    c.bench_function("extraction_level_from_str", |b| {
        b.iter(|| {
            black_box(ExtractionLevel::from_str("level1"));
            black_box(ExtractionLevel::from_str("level2"));
            black_box(ExtractionLevel::from_str("level3"));
        })
    });
}

fn bench_config_creation(c: &mut Criterion) {
    c.bench_function("java_analysis_config_creation", |b| {
        b.iter(|| {
            let config = JavaAnalysisConfig {
                level: ExtractionLevel::Level1,
                source_paths: vec![PathBuf::from(
                    "/home/rubentxu/Proyectos/rust/hodei-scan/test-java-project/src",
                )],
                include_packages: vec![],
                exclude_packages: vec![],
                enable_cache: true,
            };
            black_box(config);
        })
    });
}

fn benchmark_group(c: &mut Criterion) {
    bench_jacoco_parsing(c);
    bench_jacoco_large_file_parsing(c);
    bench_jacoco_adapter_creation(c);
    bench_memory_usage_jacoco(c);
    bench_extraction_level_enum_operations(c);
    bench_config_creation(c);
}

criterion_group!(benches, benchmark_group);
criterion_main!(benches);
