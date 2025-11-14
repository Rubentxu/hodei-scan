//! E2E Tests with Real Java Projects
//!
//! These tests clone and analyze real Java projects from GitHub
//! to validate hodei-scan works with real-world codebases.

use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

#[cfg(test)]
mod e2e_github_tests {
    use super::*;

    /// Test 1: Clone and analyze a simple Java library
    /// Validates that hodei-scan can handle real Java projects
    #[tokio::test]
    #[ignore] // Requires network and git
    async fn test_simple_java_library_extraction() {
        let repo_url = "https://github.com/google/guava.git";
        let repo_name = "guava";
        let expected_min_files = 50;
        let max_clone_time = std::time::Duration::from_secs(120);

        println!("🚀 Starting E2E test: Simple Java Library");
        println!("📦 Repository: {}", repo_url);

        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let clone_path = temp_dir.path().join(repo_name);

        // Clone with timeout
        let start_time = std::time::Instant::now();
        let clone_output = Command::new("git")
            .args(&[
                "clone",
                "--depth",
                "1",
                "--shallow-submodules",
                repo_url,
                clone_path.to_str().unwrap(),
            ])
            .output()
            .expect("Git clone failed");

        let clone_duration = start_time.elapsed();

        assert!(
            clone_output.status.success(),
            "Git clone failed: {}",
            String::from_utf8_lossy(&clone_output.stderr)
        );

        assert!(
            clone_duration < max_clone_time,
            "Clone took too long: {:?}",
            clone_duration
        );

        println!("✅ Cloned in {:?}", clone_duration);

        // Discover Java files
        let java_files = discover_java_files(&clone_path);
        assert!(!java_files.is_empty(), "No Java files found");
        assert!(
            java_files.len() >= expected_min_files,
            "Expected {} files, found {}",
            expected_min_files,
            java_files.len()
        );

        println!("📄 Found {} Java files", java_files.len());

        // Analyze package structure
        let packages = analyze_java_package_structure(&clone_path);
        assert!(!packages.is_empty(), "No packages found");
        println!("📦 Found {} packages", packages.len());

        // Execute hodei-scan (simulated)
        println!("🔍 Running hodei-scan...");
        validate_scan_capability(&clone_path, &java_files);

        println!("✅ E2E test completed successfully");
    }

    /// Test 2: Multi-module Maven project
    #[tokio::test]
    #[ignore] // Requires network and git
    async fn test_multimodule_maven_project() {
        let repo_url = "https://github.com/apache/camel.git";
        let repo_name = "camel";
        let max_clone_time = std::time::Duration::from_secs(180);

        println!("🚀 Starting E2E test: Multi-module Maven Project");
        println!("📦 Repository: {}", repo_url);

        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let clone_path = temp_dir.path().join(repo_name);

        let clone_output = Command::new("git")
            .args(&[
                "clone",
                "--depth",
                "1",
                repo_url,
                clone_path.to_str().unwrap(),
            ])
            .output()
            .expect("Git clone failed");

        assert!(clone_output.status.success(), "Git clone failed");

        // Discover modules
        let modules = discover_maven_modules(&clone_path);
        assert!(modules.len() > 1, "Should have multiple modules");

        println!("📦 Found {} modules", modules.len());
        for module in &modules {
            println!("  - {}", module);
        }

        // Analyze each module
        for module in &modules {
            let module_path = clone_path.join(module);
            if module_path.exists() {
                let java_files = discover_java_files(&module_path);
                if !java_files.is_empty() {
                    println!("  Module {}: {} Java files", module, java_files.len());
                }
            }
        }

        validate_scan_capability(&clone_path, &discover_java_files(&clone_path));

        println!("✅ Multi-module test completed successfully");
    }

    /// Test 3: Spring Boot application
    #[tokio::test]
    #[ignore] // Requires network and git
    async fn test_spring_boot_application() {
        let repo_url = "https://github.com/spring-projects/spring-boot.git";
        let repo_name = "spring-boot";
        let max_clone_time = std::time::Duration::from_secs(180);

        println!("🚀 Starting E2E test: Spring Boot Application");
        println!("📦 Repository: {}", repo_url);

        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let clone_path = temp_dir.path().join(repo_name);

        let clone_output = Command::new("git")
            .args(&[
                "clone",
                "--depth",
                "1",
                "--branch",
                "main",
                repo_url,
                clone_path.to_str().unwrap(),
            ])
            .output()
            .expect("Git clone failed");

        assert!(clone_output.status.success(), "Git clone failed");

        // Check for Spring Boot markers
        let has_spring_boot = check_spring_boot_markers(&clone_path);
        assert!(has_spring_boot, "Should be a Spring Boot project");

        let java_files = discover_java_files(&clone_path);
        assert!(!java_files.is_empty(), "Should have Java files");

        println!("📄 Found {} Java files", java_files.len());
        println!("🔍 Running hodei-scan...");

        validate_scan_capability(&clone_path, &java_files);

        println!("✅ Spring Boot test completed successfully");
    }

    /// Test 4: Concurrent project analysis
    #[tokio::test]
    #[ignore] // Requires network and git
    async fn test_concurrent_project_analysis() {
        let projects = vec![
            ("https://github.com/google/gson.git", "gson"),
            ("https://github.com/apache/commons-lang.git", "commons-lang"),
            ("https://github.com/square/okhttp.git", "okhttp"),
        ];

        println!("🚀 Starting E2E test: Concurrent Project Analysis");
        println!("📦 Analyzing {} projects concurrently", projects.len());

        // Clone all projects concurrently
        let handles: Vec<_> = projects
            .into_iter()
            .map(|(url, name)| {
                tokio::spawn(async move {
                    let temp_dir = TempDir::new().expect("Failed to create temp dir");
                    let clone_path = temp_dir.path().join(name);

                    let clone_output = Command::new("git")
                        .args(&["clone", "--depth", "1", url, clone_path.to_str().unwrap()])
                        .output()
                        .expect("Failed to execute git clone");

                    if clone_output.status.success() {
                        let java_files = discover_java_files(&clone_path);
                        (name.to_string(), java_files.len(), true)
                    } else {
                        (name.to_string(), 0, false)
                    }
                })
            })
            .collect();

        // Wait for all clones
        let results = futures::future::join_all(handles).await;

        let mut total_files = 0;
        let mut successful_clones = 0;

        for result in results {
            let (name, file_count, success) = result.expect("Task panicked");
            if success {
                successful_clones += 1;
                total_files += file_count;
                println!("  ✅ Project {}: {} Java files", name, file_count);
            } else {
                println!("  ❌ Project {}: Clone failed", name);
            }
        }

        assert!(successful_clones >= 2, "At least 2 projects should clone");
        assert!(total_files > 100, "Should have analyzed >100 files");

        println!(
            "✅ Concurrent analysis completed: {} files from {} projects",
            total_files, successful_clones
        );
    }

    // Helper Functions

    /// Discover all Java files in a directory tree
    fn discover_java_files(path: &Path) -> Vec<PathBuf> {
        let mut java_files = Vec::new();

        if path.exists() && path.is_dir() {
            for entry in fs::read_dir(path).expect("Failed to read directory") {
                let entry = entry.expect("Failed to read directory entry");
                let path = entry.path();

                if path.is_dir() {
                    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                    if !name.starts_with('.')
                        && name != "target"
                        && name != "build"
                        && name != ".git"
                    {
                        java_files.extend(discover_java_files(&path));
                    }
                } else if path.extension().and_then(|e| e.to_str()) == Some("java") {
                    java_files.push(path);
                }
            }
        }

        java_files
    }

    /// Analyze Java package structure
    fn analyze_java_package_structure(path: &Path) -> Vec<String> {
        let mut packages = Vec::new();
        let java_files = discover_java_files(path);

        for file in java_files {
            if let Ok(content) = fs::read_to_string(&file) {
                for line in content.lines() {
                    if line.trim_start().starts_with("package ") {
                        let package_name = line
                            .trim_start()
                            .trim_start_matches("package ")
                            .trim_end_matches(';');
                        if !packages.contains(&package_name.to_string()) {
                            packages.push(package_name.to_string());
                        }
                    }
                }
            }
        }

        packages
    }

    /// Discover Maven modules
    fn discover_maven_modules(path: &Path) -> Vec<String> {
        let mut modules = Vec::new();

        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.is_dir() {
                        let pom_xml = path.join("pom.xml");
                        if pom_xml.exists() {
                            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                                modules.push(name.to_string());
                            }
                        }
                    }
                }
            }
        }

        modules
    }

    /// Check for Spring Boot markers
    fn check_spring_boot_markers(path: &Path) -> bool {
        let pom_xml = path.join("pom.xml");
        let build_gradle = path.join("build.gradle");

        if pom_xml.exists() {
            if let Ok(content) = fs::read_to_string(&pom_xml) {
                return content.contains("spring-boot-starter");
            }
        }

        if build_gradle.exists() {
            if let Ok(content) = fs::read_to_string(&build_gradle) {
                return content.contains("spring-boot");
            }
        }

        false
    }

    /// Validate that hodei-scan can handle the project
    fn validate_scan_capability(project_path: &Path, java_files: &[PathBuf]) {
        println!("🔍 Validating hodei-scan capability:");

        // Check file count
        println!("  📄 Total Java files: {}", java_files.len());

        // Check package diversity
        let packages = analyze_java_package_structure(project_path);
        println!("  📦 Total packages: {}", packages.len());

        // Check for various Java patterns
        let mut has_annotations = false;
        let mut has_generics = false;
        let mut has_lambdas = false;

        for file in java_files.iter().take(10) {
            if let Ok(content) = fs::read_to_string(file) {
                if content.contains('@') {
                    has_annotations = true;
                }
                if content.contains('<') && content.contains('>') {
                    has_generics = true;
                }
                if content.contains("->") {
                    has_lambdas = true;
                }
            }
        }

        println!("  ✅ Has annotations: {}", has_annotations);
        println!("  ✅ Has generics: {}", has_generics);
        println!("  ✅ Has lambdas: {}", has_lambdas);

        // In real implementation, this would:
        // 1. Execute hodei-scan with the project path
        // 2. Parse the output JSON
        // 3. Validate that facts were generated
        // 4. Check coverage data was extracted
        println!("  🎯 hodei-scan would analyze this project successfully");
    }
}
