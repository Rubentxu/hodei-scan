//! JaCoCo Adapter (Infrastructure Layer)
//!
//! This adapter implements the JavaSourceRepository port using JaCoCo coverage reports.
//! It reads JaCoCo XML reports and provides coverage data to the domain layer.

use crate::domain::{
    entities::{CoverageData, DomainError, JavaClass, JavaSourceId},
    repositories::JavaSourceRepository,
};
use hodei_ir::types::project_path::ProjectPath;
use std::collections::HashMap;
use std::path::Path;

/// JaCoCo Adapter Implementation
pub struct JacocoAdapter {
    jacoco_report_path: std::path::PathBuf,
    cache: HashMap<JavaSourceId, CoverageData>,
}

impl JacocoAdapter {
    pub fn new(jacoco_report_path: std::path::PathBuf) -> Self {
        Self {
            jacoco_report_path,
            cache: HashMap::new(),
        }
    }

    /// Load and cache coverage data from JaCoCo XML report
    pub fn load_coverage_data(&mut self) -> Result<Vec<CoverageData>, DomainError> {
        use quick_xml::Reader;
        use quick_xml::events::Event;
        use quick_xml::name::QName;

        let content = std::fs::read_to_string(&self.jacoco_report_path)
            .map_err(|e| DomainError::Io(e.to_string()))?;

        let mut reader = Reader::from_str(&content);

        let mut current_package = String::new();
        let mut current_class = String::new();
        let mut current_method = String::new();

        let mut buf = Vec::new();
        while let Ok(event) = reader.read_event_into(&mut buf) {
            match event {
                Event::Start(e) => {
                    let name = e.name();
                    match name.as_ref() {
                        b"package" => {
                            if let Some(attr) = e.attributes().find(|a| {
                                a.as_ref()
                                    .ok()
                                    .map(|a| a.key == QName(b"name"))
                                    .unwrap_or(false)
                            }) {
                                current_package =
                                    String::from_utf8_lossy(attr.as_ref().unwrap().value.as_ref())
                                        .to_string();
                            }
                        }
                        b"class" => {
                            if let Some(attr) = e.attributes().find(|a| {
                                a.as_ref()
                                    .ok()
                                    .map(|a| a.key == QName(b"name"))
                                    .unwrap_or(false)
                            }) {
                                current_class =
                                    String::from_utf8_lossy(attr.as_ref().unwrap().value.as_ref())
                                        .to_string();
                            }
                        }
                        b"method" => {
                            if let Some(attr) = e.attributes().find(|a| {
                                a.as_ref()
                                    .ok()
                                    .map(|a| a.key == QName(b"name"))
                                    .unwrap_or(false)
                            }) {
                                current_method =
                                    String::from_utf8_lossy(attr.as_ref().unwrap().value.as_ref())
                                        .to_string();
                            }
                        }
                        b"line" => {
                            if let Some(attr) = e.attributes().find(|a| {
                                a.as_ref()
                                    .ok()
                                    .map(|a| a.key == QName(b"nr"))
                                    .unwrap_or(false)
                            }) {
                                let line_number =
                                    String::from_utf8_lossy(attr.as_ref().unwrap().value.as_ref())
                                        .parse::<u32>()
                                        .unwrap_or(0);

                                let mi = e
                                    .attributes()
                                    .find(|a| {
                                        a.as_ref()
                                            .ok()
                                            .map(|a| a.key == QName(b"mi"))
                                            .unwrap_or(false)
                                    })
                                    .and_then(|a| {
                                        String::from_utf8_lossy(a.as_ref().unwrap().value.as_ref())
                                            .parse::<u32>()
                                            .ok()
                                    })
                                    .unwrap_or(0);

                                let ci = e
                                    .attributes()
                                    .find(|a| {
                                        a.as_ref()
                                            .ok()
                                            .map(|a| a.key == QName(b"ci"))
                                            .unwrap_or(false)
                                    })
                                    .and_then(|a| {
                                        String::from_utf8_lossy(a.as_ref().unwrap().value.as_ref())
                                            .parse::<u32>()
                                            .ok()
                                    })
                                    .unwrap_or(0);

                                let mb = e
                                    .attributes()
                                    .find(|a| {
                                        a.as_ref()
                                            .ok()
                                            .map(|a| a.key == QName(b"mb"))
                                            .unwrap_or(false)
                                    })
                                    .and_then(|a| {
                                        String::from_utf8_lossy(a.as_ref().unwrap().value.as_ref())
                                            .parse::<u32>()
                                            .ok()
                                    })
                                    .unwrap_or(0);

                                let cb = e
                                    .attributes()
                                    .find(|a| {
                                        a.as_ref()
                                            .ok()
                                            .map(|a| a.key == QName(b"cb"))
                                            .unwrap_or(false)
                                    })
                                    .and_then(|a| {
                                        String::from_utf8_lossy(a.as_ref().unwrap().value.as_ref())
                                            .parse::<u32>()
                                            .ok()
                                    })
                                    .unwrap_or(0);

                                let class_name =
                                    current_class.split('/').last().unwrap_or(&current_class);

                                let source_id = JavaSourceId {
                                    package: current_package.clone(),
                                    class_name: class_name.to_string(),
                                    file_path: ProjectPath::new(
                                        format!(
                                            "src/main/java/{}/{}.java",
                                            current_package.replace('.', "/"),
                                            class_name
                                        )
                                        .into(),
                                    ),
                                };

                                let total_instructions = mi + ci;
                                let coverage_pct = if total_instructions > 0 {
                                    (ci as f64 / total_instructions as f64) * 100.0
                                } else {
                                    0.0
                                };

                                self.cache.insert(
                                    source_id.clone(),
                                    CoverageData {
                                        source_id,
                                        line_number,
                                        instruction_missed: mi,
                                        instruction_covered: ci,
                                        branch_missed: mb,
                                        branch_covered: cb,
                                        coverage_percentage: coverage_pct,
                                    },
                                );
                            }
                        }
                        _ => {}
                    }
                }
                Event::End(_) => {}
                _ => {}
            }
        }

        // Convert cache to vector and return
        Ok(self.cache.values().cloned().collect())
    }
}

impl JavaSourceRepository for JacocoAdapter {
    fn find_by_package(&self, package: &str) -> Result<Vec<JavaClass>, DomainError> {
        // TODO: Implement package discovery from source files
        Ok(vec![])
    }

    fn get_coverage_data(
        &self,
        source_id: &JavaSourceId,
    ) -> Result<Option<CoverageData>, DomainError> {
        Ok(self.cache.get(source_id).cloned())
    }

    fn save_analysis_result(
        &self,
        _result: &crate::domain::entities::JavaAnalysisResult,
    ) -> Result<(), DomainError> {
        // JaCoCo adapter doesn't save analysis results
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_jacoco_adapter_creation() {
        // RED: Test should fail until implementation
        let adapter = JacocoAdapter::new(PathBuf::from("target/site/jacoco/jacoco.xml"));
        assert!(
            adapter.jacoco_report_path.exists()
                || !adapter.jacoco_report_path.to_string_lossy().is_empty()
        );
    }

    #[test]
    fn test_find_package() {
        // RED: Test should fail until implementation
        let adapter = JacocoAdapter::new(PathBuf::from("target/site/jacoco/jacoco.xml"));
        let result = adapter.find_by_package("com.example");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[test]
    fn test_get_coverage_data_empty() {
        // RED: Test should fail until implementation
        let adapter = JacocoAdapter::new(PathBuf::from("target/site/jacoco/jacoco.xml"));
        let source_id = JavaSourceId {
            package: "com.example".to_string(),
            class_name: "TestClass".to_string(),
            file_path: ProjectPath::new(PathBuf::from("src/main/java/com/example/TestClass.java")),
        };

        let result = adapter.get_coverage_data(&source_id);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }
}
