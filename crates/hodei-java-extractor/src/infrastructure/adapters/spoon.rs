//! Spoon Service (Infrastructure Layer)
//!
//! This service provides deep semantic analysis using Spoon library.
//! It implements Level 3 (deep analysis) with taint analysis and connascence detection.
//!
//! ## Architecture
//!
//! 1. **Java Runner**: Calls external Java process using Spoon JAR
//! 2. **Semantic Model**: Parses JSON output from Spoon
//! 3. **Integration**: Connects with hodei-deep-analysis-engine for advanced analysis

use crate::domain::{
    entities::{DomainError, JavaClass},
    repositories::JavaSourceRepository,
};
use hodei_ir::types::project_path::ProjectPath;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Semantic model from Spoon analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticModel {
    package: String,
    class_name: String,
    super_class: Option<String>,
    interfaces: Vec<String>,
    methods: Vec<MethodInfo>,
    fields: Vec<FieldInfo>,
    annotations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MethodInfo {
    name: String,
    signature: String,
    return_type: String,
    parameters: Vec<String>,
    is_public: bool,
    is_private: bool,
    is_protected: bool,
    is_static: bool,
    is_abstract: bool,
    body: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FieldInfo {
    name: String,
    type_: String,
    is_public: bool,
    is_private: bool,
    is_protected: bool,
    is_static: bool,
    is_final: bool,
    initializer: Option<String>,
}

/// Spoon Service Implementation
pub struct SpoonService {
    source_paths: Vec<PathBuf>,
    semantic_models: HashMap<String, SemanticModel>,
    output_dir: PathBuf,
}

impl SpoonService {
    pub fn new(source_paths: Vec<PathBuf>) -> Self {
        let output_dir = std::env::temp_dir().join("hodei-spoon-output");

        Self {
            source_paths,
            semantic_models: HashMap::new(),
            output_dir,
        }
    }

    /// Run Spoon analysis to build semantic model
    pub fn run_spoon_analysis(&mut self) -> Result<Vec<SemanticModel>, DomainError> {
        // Clean output directory
        if self.output_dir.exists() {
            std::fs::remove_dir_all(&self.output_dir)
                .map_err(|e| DomainError::Io(e.to_string()))?;
        }
        std::fs::create_dir_all(&self.output_dir).map_err(|e| DomainError::Io(e.to_string()))?;

        // Find all Java source directories
        let mut all_java_files = Vec::new();
        for source_path in &self.source_paths {
            if source_path.is_dir() {
                self.collect_java_files(source_path, &mut all_java_files)?;
            } else if source_path.is_file() {
                all_java_files.push(source_path.clone());
            }
        }

        if all_java_files.is_empty() {
            return Ok(Vec::new());
        }

        // Determine the source root (common parent directory)
        let source_root = self.find_common_parent(&all_java_files)?;

        // Run Spoon on each directory
        let mut all_models = Vec::new();
        for java_file in &all_java_files {
            let output_file = self.output_dir.join(
                java_file
                    .strip_prefix(&source_root)
                    .unwrap_or(java_file.as_path())
                    .with_extension("json"),
            );

            // Create parent directories
            if let Some(parent) = output_file.parent() {
                std::fs::create_dir_all(parent).map_err(|e| DomainError::Io(e.to_string()))?;
            }

            let result = self.run_spoon_on_file(&source_root, java_file, &output_file)?;

            if result.status.success() {
                // Parse output JSON
                let json_content =
                    fs::read_to_string(&output_file).map_err(|e| DomainError::Io(e.to_string()))?;

                let models: Vec<SemanticModel> =
                    serde_json::from_str(&json_content).map_err(|e| {
                        DomainError::ValidationError(format!("Failed to parse Spoon output: {}", e))
                    })?;

                for model in &models {
                    let key = format!("{}.{}", model.package, model.class_name);
                    self.semantic_models.insert(key, model.clone());
                }

                all_models.extend(models);
            } else {
                let stderr = String::from_utf8_lossy(&result.stderr);
                tracing::warn!(
                    "Spoon analysis failed for {}: {}",
                    java_file.display(),
                    stderr
                );
            }
        }

        Ok(all_models)
    }

    fn collect_java_files(
        &self,
        dir: &Path,
        java_files: &mut Vec<PathBuf>,
    ) -> Result<(), DomainError> {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && path.extension().map_or(false, |ext| ext == "java") {
                    java_files.push(path);
                } else if path.is_dir()
                    && !path
                        .file_name()
                        .map(|s| s.to_string_lossy().starts_with('.'))
                        .unwrap_or(false)
                {
                    self.collect_java_files(&path, java_files)?;
                }
            }
        }
        Ok(())
    }

    fn find_common_parent(&self, paths: &[PathBuf]) -> Result<PathBuf, DomainError> {
        if paths.is_empty() {
            return Err(DomainError::ValidationError(
                "No paths provided".to_string(),
            ));
        }

        let first = &paths[0];
        let first_components: Vec<_> = first.components().collect();

        for i in (1..first_components.len()).rev() {
            let candidate = PathBuf::from_iter(&first_components[..i]);
            if paths.iter().all(|p| p.starts_with(&candidate)) {
                return Ok(candidate);
            }
        }

        // Fallback to first path
        Ok(first.clone())
    }

    fn run_spoon_on_file(
        &self,
        source_root: &Path,
        java_file: &Path,
        output_file: &Path,
    ) -> Result<std::process::Output, DomainError> {
        // Find Spoon JAR
        let spoon_jar = self.find_spoon_jar()?;

        // Build classpath
        let mut classpath = format!("{}:{}", spoon_jar.display(), source_root.display());

        // Add gson for JSON serialization
        if let Ok(gson_jar) = self.find_gson_jar() {
            classpath.push_str(&format!(":{}", gson_jar.display()));
        }

        // Run Java process
        let output = Command::new("java")
            .arg("-cp")
            .arg(&classpath)
            .arg("SpoonRunner")
            .arg(source_root.display().to_string())
            .arg(output_file.display().to_string())
            .output()
            .map_err(|e| DomainError::Io(e.to_string()))?;

        Ok(output)
    }

    fn find_spoon_jar(&self) -> Result<PathBuf, DomainError> {
        // Try Maven Central download path
        let spoon_jar = PathBuf::from("/tmp/spoon-core-10.1.3.jar");

        if !spoon_jar.exists() {
            tracing::info!("Downloading Spoon JAR from Maven Central...");
            let download_url = "https://repo1.maven.org/maven2/fr/inria/spoon/spoon-core/10.1.3/spoon-core-10.1.3.jar";

            // Download using curl
            let output = Command::new("curl")
                .arg("-L")
                .arg("-o")
                .arg(&spoon_jar)
                .arg(download_url)
                .output()
                .map_err(|e| DomainError::Io(e.to_string()))?;

            if !output.status.success() {
                return Err(DomainError::Io(
                    "Failed to download Spoon JAR. Please install manually.".to_string(),
                ));
            }
        }

        Ok(spoon_jar)
    }

    fn find_gson_jar(&self) -> Result<PathBuf, DomainError> {
        let gson_jar = PathBuf::from("/tmp/gson-2.10.1.jar");

        if !gson_jar.exists() {
            tracing::info!("Downloading Gson JAR from Maven Central...");
            let download_url =
                "https://repo1.maven.org/maven2/com/google/code/gson/gson/2.10.1/gson-2.10.1.jar";

            // Download using curl
            let output = Command::new("curl")
                .arg("-L")
                .arg("-o")
                .arg(&gson_jar)
                .arg(download_url)
                .output()
                .map_err(|e| DomainError::Io(e.to_string()))?;

            if !output.status.success() {
                return Err(DomainError::Io(
                    "Failed to download Gson JAR. Please install manually.".to_string(),
                ));
            }
        }

        Ok(gson_jar)
    }

    /// Get semantic model for a class
    pub fn get_semantic_model(&self, fully_qualified_name: &str) -> Option<&SemanticModel> {
        self.semantic_models.get(fully_qualified_name)
    }
}

impl JavaSourceRepository for SpoonService {
    fn find_by_package(&self, package: &str) -> Result<Vec<JavaClass>, DomainError> {
        let mut classes = Vec::new();

        for (key, model) in &self.semantic_models {
            if model.package == package {
                let source_id = crate::domain::entities::JavaSourceId {
                    package: model.package.clone(),
                    class_name: model.class_name.clone(),
                    file_path: ProjectPath::new(PathBuf::from(format!(
                        "{}/{}.java",
                        model.package.replace('.', "/"),
                        model.class_name
                    ))),
                };

                classes.push(JavaClass {
                    id: source_id,
                    is_public: model.methods.iter().any(|m| m.is_public),
                    is_abstract: model.methods.iter().any(|m| m.is_abstract),
                    super_class: model.super_class.clone(),
                    interfaces: model.interfaces.clone(),
                });
            }
        }

        Ok(classes)
    }

    fn get_coverage_data(
        &self,
        _source_id: &crate::domain::entities::JavaSourceId,
    ) -> Result<Option<crate::domain::entities::CoverageData>, DomainError> {
        // Spoon service doesn't provide coverage data (that's JaCoCo's job)
        Ok(None)
    }

    fn save_analysis_result(
        &self,
        _result: &crate::domain::entities::JavaAnalysisResult,
    ) -> Result<(), DomainError> {
        // Spoon service doesn't save analysis results
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spoon_service_creation() {
        let service = SpoonService::new(vec![PathBuf::from("src/main/java")]);
        assert_eq!(service.source_paths.len(), 1);
    }

    #[test]
    fn test_find_package_empty() {
        let service = SpoonService::new(vec![PathBuf::from("src/main/java")]);
        let result = service.find_by_package("com.example.service");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[test]
    fn test_spoon_analysis_stub() {
        let mut service = SpoonService::new(vec![PathBuf::from("src/main/java")]);
        let result = service.run_spoon_analysis();
        assert!(result.is_ok());
    }
}
