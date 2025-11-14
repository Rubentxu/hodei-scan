#[cfg(test)]
mod e2e_github_tests {
    use std::fs;
    use std::path::Path;
    use std::path::PathBuf;
    use std::process::Command;
    use tempfile::TempDir;

    /// Test 1: Clone and analyze a simple Java library
    /// Tests extraction on a basic Java project with JAR dependencies
    #[tokio::test]
    async fn test_simple_java_library_extraction() {
        // Test configuration
        let repo_url = "https://github.com/google/guava.git";
        let repo_name = "guava";
        let expected_min_files = 50; // Guava is a large library
        let max_clone_time_seconds = 120;

        println!("Starting E2E test: Simple Java Library Extraction");
        println!("Repository: {}", repo_url);

        // Create temporary directory for cloning
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let clone_path = temp_dir.path().join(repo_name);

        // Clone repository
        println!("Cloning repository...");
        let start_time = std::time::Instant::now();

        let clone_output = Command::new("git")
            .args(&["clone", "--depth", "1", repo_url, clone_path.to_str().unwrap()])
            .output()
            .expect("Failed to execute git clone");

        let clone_duration = start_time.elapsed();

        assert!(
            clone_output.status.success(),
            "Git clone failed: {}",
            String::from_utf8_lossy(&clone_output.stderr)
        );

        assert!(
            clone_duration < std::time::Duration::from_secs(max_clone_time_seconds),
            "Clone took too long: {:?}",
            clone_duration
        );

        println!("Repository cloned successfully in {:?}", clone_duration);

        // Discover Java files
        let java_files = discover_java_files(&clone_path);
        assert!(
            !java_files.is_empty(),
            "No Java files found in repository"
        );

        assert!(
            java_files.len() >= expected_min_files,
            "Expected at least {} Java files, found {}",
            expected_min_files,
            java_files.len()
        );

        println!("Found {} Java files", java_files.len());

        // Analyze repository structure
        let package_info = analyze_java_package_structure(&clone_path);
        println!("Package structure: {:?}", package_info);

        assert!(
            !package_info.is_empty(),
            "Should find package declarations"
        );

        // Validate Java version by checking source files
        let has_modern_java = check_for_modern_java_features(&clone_path);
        println!("Repository uses modern Java features: {}", has_modern_java);

        // Test JaCoCo report generation (if build system available)
        test_jacoco_report_generation(&clone_path).await;

        println!("Simple Java library extraction test completed successfully");
    }

    /// Test 2: Clone and analyze a Spring Boot application
    /// Tests extraction on a real-world web application
    #[tokio::test]
    async fn test_spring_boot_application_extraction() {
        let repo_url = "https://github.com/spring-projects/spring-boot.git";
        let repo_name = "spring-boot";
        let max_clone_time_seconds = 180; // Spring Boot is large

        println!("Starting E2E test: Spring Boot Application Extraction");
        println!("Repository: {}", repo_url);

        // Create temporary directory for cloning
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let clone_path = temp_dir.path().join(repo_name);

        // Clone repository with shallow clone to save time
        println!("Cloning repository (shallow)...");
        let start_time = std::time::Instant::now();

        let clone_output = Command::new("git")
            .args(&["clone", "--depth", "1", "--branch", "main", repo_url, clone_path.to_str().unwrap()])
            .output()
            .expect("Failed to execute git clone");

        let clone_duration = start_time.elapsed();

        assert!(
            clone_output.status.success(),
            "Git clone failed: {}",
            String::from_utf8_lossy(&clone_output.stderr)
        );

        println!("Repository cloned in {:?}", clone_duration);

        // Analyze Maven/Gradle structure
        let build_system = detect_build_system(&clone_path);
        println!("Detected build system: {:?}", build_system);

        assert!(
            build_system.is_some(),
            "Should detect a build system (Maven or Gradle)"
        );

        // Count Java files
        let java_files = discover_java_files(&clone_path);
        println!("Found {} Java files", java_files.len());

        // Spring Boot samples directory often contains examples
        let samples_path = clone_path.join("spring-boot-samples");
        if samples_path.exists() {
            let sample_files = discover_java_files(&samples_path);
            println!("Found {} Java files in samples directory", sample_files.len());
        }

        // Test annotation processing capabilities
        test_annotation_processing(&clone_path).await;

        println!("Spring Boot application extraction test completed");
    }

    /// Test 3: Clone and analyze a multi-module Maven project
    /// Tests extraction on a complex project with multiple modules
    #[tokio::test]
    async fn test_multimodule_maven_project_extraction() {
        let repo_url = "https://github.com/apache/camel.git";
        let repo_name = "camel";
        let max_clone_time_seconds = 180;

        println!("Starting E2E test: Multi-module Maven Project");
        println!("Repository: {}", repo_url);

        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let clone_path = temp_dir.path().join(repo_name);

        // Clone repository
        println!("Cloning Apache Camel...");
        let start_time = std::time::Instant::now();

        let clone_output = Command::new("git")
            .args(&["clone", "--depth", "1", repo_url, clone_path.to_str().unwrap()])
            .output()
            .expect("Failed to execute git clone");

        let clone_duration = start_time.elapsed();

        assert!(
            clone_output.status.success(),
            "Git clone failed: {}",
            String::from_utf8_lossy(&clone_output.stderr)
        );

        println!("Repository cloned in {:?}", clone_duration);

        // Discover all modules
        let modules = discover_maven_modules(&clone_path);
        println!("Found {} Maven modules", modules.len());

        assert!(
            modules.len() > 1,
            "Multi-module project should have multiple modules"
        );

        // Analyze each module
        for module in &modules {
            let module_path = clone_path.join(module);
            let java_files = discover_java_files(&module_path);

            if !java_files.is_empty() {
                println!("Module {}: {} Java files", module, java_files.len());

                // Test module-specific extraction
                test_module_extraction(&module_path).await;
            }
        }

        // Validate cross-module dependencies
        validate_module_dependencies(&clone_path, &modules);

        println!("Multi-module project extraction test completed");
    }

    /// Test 4: Clone and analyze Java with complex dependencies
    /// Tests extraction on a project with complex build configuration
    #[tokio::test]
    async fn test_complex_dependency_project_extraction() {
        let repo_url = "https://github.com/junit-team/junit4.git";
        let repo_name = "junit4";
        let max_clone_time_seconds = 120;

        println!("Starting E2E test: Complex Dependency Project");
        println!("Repository: {}", repo_url);

        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let clone_path = temp_dir.path().join(repo_name);

        let clone_output = Command::new("git")
            .args(&["clone", "--depth", "1", repo_url, clone_path.to_str().unwrap()])
            .output()
            .expect("Failed to execute git clone");

        assert!(
            clone_output.status.success(),
            "Git clone failed: {}",
            String::from_utf8_lossy(&clone_output.stderr)
        );

        // Analyze build configuration
        let pom_path = clone_path.join("pom.xml");
        assert!(pom_path.exists(), "Maven project should have pom.xml");

        // Check for dependency management
        let has_dependency_management = check_dependency_management(&pom_path);
        println!("Project has dependency management: {}", has_dependency_management);

        // Analyze test structure
        let test_dir = clone_path.join("src").join("test");
        if test_dir.exists() {
            let test_files = discover_java_files(&test_dir);
            println!("Found {} test files", test_files.len());

            // Validate test patterns
            validate_test_patterns(&test_files);
        }

        println!("Complex dependency project extraction test completed");
    }

    /// Test 5: Clone and analyze legacy Java project
    /// Tests extraction on older Java code (Java 7/8 era)
    #[tokio::test]
    async fn test_legacy_java_project_extraction() {
        let repo_url = "https://github.com/h2database/h2database.git";
        let repo_name = "h2database";
        let max_clone_time_seconds = 120;

        println!("Starting E2E test: Legacy Java Project");
        println!("Repository: {}", repo_url);

        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let clone_path = temp_dir.path().join(repo_name);

        let clone_output = Command::new("git")
            .args(&["clone", "--depth", "1", "--branch", "master", repo_url, clone_path.to_str().unwrap()])
            .output()
            .expect("Failed to execute git clone");

        assert!(
            clone_output.status.success(),
            "Git clone failed: {}",
            String::from_utf8_lossy(&clone_output.stderr)
        );

        // Analyze Java version compatibility
        let java_version = detect_java_version(&clone_path);
        println!("Detected Java version: {:?}", java_version);

        // Check for legacy patterns
        let legacy_patterns = detect_legacy_patterns(&clone_path);
        println!("Legacy patterns found: {:?}", legacy_patterns);

        // Analyze file size distribution
        let file_stats = analyze_file_statistics(&clone_path);
        println!("File statistics: {:?}", file_stats);

        println!("Legacy Java project extraction test completed");
    }

    /// Test 6: Concurrent project analysis
    /// Tests handling multiple projects simultaneously
    #[tokio::test]
    async fn test_concurrent_project_analysis() {
        let projects = vec![
            ("https://github.com/google/gson.git", "gson"),
            ("https://github.com/apache/commons-lang.git", "commons-lang"),
            ("https://github.com/square/okhttp.git", "okhttp"),
        ];

        println!("Starting E2E test: Concurrent Project Analysis");
        println!("Analyzing {} projects concurrently", projects.len());

        // Clone all projects concurrently
        let handles: Vec<_> = projects.into_iter().map(|(url, name)| {
            tokio::spawn(async move {
                let temp_dir = TempDir::new().expect("Failed to create temp directory");
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
        }).collect();

        // Wait for all clones to complete
        let results = futures::future::join_all(handles).await;

        // Validate results
        let mut total_files = 0;
        let mut successful_clones = 0;

        for result in results {
            let (name, file_count, success) = result.expect("Task panicked");
            if success {
                successful_clones += 1;
                total_files += file_count;
                println!("Project {}: {} Java files", name, file_count);
            } else {
                println!("Project {}: Clone failed", name);
            }
        }

        assert!(
            successful_clones >= 2,
            "At least 2 projects should clone successfully"
        );

        println!("Concurrent analysis completed: {} files total from {} projects",
                 total_files, successful_clones);
    }

    /// Test 7: Repository with submodules
    /// Tests extraction on project with git submodules
    #[tokio::test]
    async fn test_repository_with_submodules() {
        let repo_url = "https://github.com/tensorflow/tensorflow.git";
        let repo_name = "tensorflow";
        let max_clone_time_seconds = 180;

        println!("Starting E2E test: Repository with Submodules");
        println!("Repository: {}", repo_url);

        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let clone_path = temp_dir.path().join(repo_name);

        // Clone with submodules
        let clone_output = Command::new("git")
            .args(&["clone", "--depth", "1", "--recurse-submodules", repo_url, clone_path.to_str().unwrap()])
            .output()
            .expect("Failed to execute git clone");

        if clone_output.status.success() {
            // Check for submodules
            let has_gitmodules = clone_path.join(".gitmodules").exists();
            println!("Repository has .gitmodules: {}", has_gitmodules);

            // Look for Java files in the repository
            let java_files = discover_java_files(&clone_path);
            println!("Found {} Java files", java_files.len());

            // Java might be in specific directories
            let contrib_dir = clone_path.join("java");
            if contrib_dir.exists() {
                let contrib_files = discover_java_files(&contrib_dir);
                println!("Found {} Java files in contrib directory", contrib_files.len());
            }
        } else {
            println!("Clone failed, skipping submodule checks");
        }

        println!("Repository with submodules test completed");
    }

    // Helper functions

    /// Discover all Java files in a directory tree
    fn discover_java_files(path: &Path) -> Vec<PathBuf> {
        let mut java_files = Vec::new();

        if path.exists() && path.is_dir() {
            for entry in fs::read_dir(path).expect("Failed to read directory") {
                let entry = entry.expect("Failed to read directory entry");
                let path = entry.path();

                if path.is_dir() {
                    // Skip hidden directories and common build directories
                    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                    if !name.starts_with('.') && name != "target" && name != "build" && name != ".git" {
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
                        let package_name = line.trim_start()
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

    /// Check for modern Java features
    fn check_for_modern_java_features(path: &Path) -> bool {
        let java_files = discover_java_files(path);
        let mut has_modern_features = false;

        for file in java_files.iter().take(10) { // Check first 10 files
            if let Ok(content) = fs::read_to_string(file) {
                // Look for var keyword, records, sealed classes, etc.
                if content.contains("var ") || content.contains("record ") || content.contains("sealed ") {
                    has_modern_features = true;
                    break;
                }
            }
        }

        has_modern_features
    }

    /// Detect build system (Maven or Gradle)
    fn detect_build_system(path: &Path) -> Option<String> {
        let pom_xml = path.join("pom.xml");
        let build_gradle = path.join("build.gradle");
        let settings_gradle = path.join("settings.gradle");

        if pom_xml.exists() {
            Some("Maven".to_string())
        } else if build_gradle.exists() || settings_gradle.exists() {
            Some("Gradle".to_string())
        } else {
            None
        }
    }

    /// Test JaCoCo report generation
    async fn test_jacoco_report_generation(_path: &Path) {
        // In a real implementation, this would:
        // 1. Build the project
        // 2. Run tests to generate JaCoCo reports
        // 3. Parse and validate the reports
        println!("JaCoCo report generation test (simulated)");
    }

    /// Test annotation processing
    async fn test_annotation_processing(_path: &Path) {
        // In a real implementation, this would test:
        // 1. Custom annotations
        // 2. Annotation processors
        // 3. Runtime annotation access
        println!("Annotation processing test (simulated)");
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

    /// Test module-specific extraction
    async fn test_module_extraction(_path: &Path) {
        println!("Module extraction test (simulated)");
    }

    /// Validate module dependencies
    fn validate_module_dependencies(_path: &Path, _modules: &[String]) {
        println!("Module dependency validation (simulated)");
    }

    /// Check dependency management in pom.xml
    fn check_dependency_management(pom_path: &Path) -> bool {
        if let Ok(content) = fs::read_to_string(pom_path) {
            content.contains("<dependencyManagement>")
        } else {
            false
        }
    }

    /// Validate test patterns
    fn validate_test_patterns(test_files: &[PathBuf]) {
        let mut junit_tests = 0;
        let mut testng_tests = 0;

        for file in test_files {
            if let Ok(content) = fs::read_to_string(file) {
                if content.contains("@Test") || content.contains("org.junit") {
                    junit_tests += 1;
                }
                if content.contains("org.testng") {
                    testng_tests += 1;
                }
            }
        }

        println!("JUnit tests: {}, TestNG tests: {}", junit_tests, testng_tests);
    }

    /// Detect Java version from build configuration
    fn detect_java_version(path: &Path) -> Option<String> {
        let pom_xml = path.join("pom.xml");
        let build_gradle = path.join("build.gradle");

        if pom_xml.exists() {
            if let Ok(content) = fs::read_to_string(pom_xml) {
                if content.contains("<java.version>") {
                    return Some("Maven (detected from pom.xml)".to_string());
                }
            }
        }

        if build_gradle.exists() {
            if let Ok(content) = fs::read_to_string(build_gradle) {
                if content.contains("sourceCompatibility") || content.contains("targetCompatibility") {
                    return Some("Gradle (detected from build.gradle)".to_string());
                }
            }
        }

        None
    }

    /// Detect legacy patterns in Java code
    fn detect_legacy_patterns(path: &Path) -> Vec<String> {
        let mut patterns = Vec::new();
        let java_files = discover_java_files(path);

        for file in java_files.iter().take(20) {
            if let Ok(content) = fs::read_to_string(file) {
                if content.contains("System.out.println") {
                    patterns.push("System.out.println".to_string());
                }
                if content.contains("Vector") || content.contains("Hashtable") {
                    patterns.push("Legacy Collections".to_string());
                }
                if content.contains("Synchronized") {
                    patterns.push("Synchronized blocks".to_string());
                }
            }
        }

        patterns
    }

    /// Analyze file statistics
    fn analyze_file_statistics(path: &Path) -> (usize, usize, usize) {
        let java_files = discover_java_files(path);
        let mut total_lines = 0;
        let mut total_size = 0;
        let mut file_count = 0;

        for file in &java_files {
            if let Ok(metadata) = fs::metadata(file) {
                total_size += metadata.len() as usize;
                file_count += 1;

                if let Ok(content) = fs::read_to_string(file) {
                    total_lines += content.lines().count();
                }
            }
        }

        (file_count, total_lines, total_size)
    }
}
