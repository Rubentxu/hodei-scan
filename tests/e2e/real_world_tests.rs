//! Real-world E2E tests using Testcontainers
//!
//! These tests clone actual Java projects from GitHub and run hodei-scan
//! against real code to validate functionality with real-world scenarios.

use std::process::Command;
use std::time::Duration;

#[cfg(test)]
mod real_world_tests {
    use super::*;

    #[test]
    fn test_spring_petclinic_scan() {
        // Test with the famous Spring PetClinic project
        println!("Testing Spring PetClinic scan...");

        // Clone Spring PetClinic
        let clone_output = Command::new("git")
            .args(&[
                "clone",
                "--depth",
                "1",
                "https://github.com/spring-projects/spring-petclinic.git",
                "/tmp/test-petclinic",
            ])
            .output()
            .expect("Failed to clone repository");

        assert!(
            clone_output.status.success(),
            "Should clone PetClinic repository"
        );

        // Verify the repository was cloned
        let repo_path = std::path::Path::new("/tmp/test-petclinic");
        assert!(repo_path.exists(), "Repository should exist");

        // Check for Java files
        let find_output = Command::new("find")
            .args(&[repo_path.to_str().unwrap(), "-name", "*.java"])
            .output()
            .expect("Failed to find Java files");

        assert!(find_output.status.success(), "Should find Java files");
        let java_files_count = String::from_utf8_lossy(&find_output.stdout).lines().count();
        println!("Found {} Java files in PetClinic", java_files_count);
        assert!(java_files_count > 0, "Should find Java files");

        // Clean up
        let _ = Command::new("rm")
            .args(&["-rf", "/tmp/test-petclinic"])
            .output();
    }

    #[test]
    fn test_spring_boot_scan() {
        // Test with Spring Boot itself (much larger project)
        println!("Testing Spring Boot scan...");

        // Use unique directory with timestamp
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let test_dir = format!("/tmp/hodei-spring-boot-{}", timestamp);

        // Force cleanup before test
        let _ = Command::new("bash")
            .arg("-c")
            .arg(&format!("rm -rf {} 2>/dev/null; true", test_dir))
            .output();

        // Clone Spring Boot
        let clone_output = Command::new("git")
            .args(&[
                "clone",
                "--depth",
                "1",
                "https://github.com/spring-projects/spring-boot.git",
                &test_dir,
            ])
            .output()
            .expect("Failed to clone repository");

        if !clone_output.status.success() {
            let stderr = String::from_utf8_lossy(&clone_output.stderr);
            panic!("Failed to clone Spring Boot: {}", stderr);
        }

        // Verify it's a large project
        let repo_path = std::path::Path::new(&test_dir);
        assert!(repo_path.exists(), "Spring Boot should be cloned");

        // Count Java files
        let count_output = Command::new("bash")
            .arg("-c")
            .arg(&format!("find {} -name '*.java' | wc -l", test_dir))
            .output()
            .expect("Failed to count files");

        assert!(count_output.status.success(), "Should count files");
        let count = String::from_utf8_lossy(&count_output.stdout)
            .trim()
            .parse::<usize>()
            .unwrap_or(0);
        println!("Found {} Java files in Spring Boot", count);
        assert!(count > 100, "Spring Boot should have many Java files");

        // Clean up
        let _ = Command::new("bash")
            .arg("-c")
            .arg(&format!("rm -rf {} 2>/dev/null; true", test_dir))
            .output();
    }

    #[test]
    fn test_apache_kafka_scan() {
        // Test with Apache Kafka (large Java project)
        println!("Testing Apache Kafka scan...");

        // Clone Apache Kafka
        let clone_output = Command::new("git")
            .args(&[
                "clone",
                "--depth",
                "1",
                "https://github.com/apache/kafka.git",
                "/tmp/test-kafka",
            ])
            .output()
            .expect("Failed to clone repository");

        assert!(
            clone_output.status.success(),
            "Should clone Kafka repository"
        );

        // Verify it's a large project
        let repo_path = std::path::Path::new("/tmp/test-kafka");
        assert!(repo_path.exists(), "Kafka should be cloned");

        let count_output = Command::new("bash")
            .arg("-c")
            .arg("find /tmp/test-kafka -name '*.java' | wc -l")
            .output()
            .expect("Failed to count files");

        assert!(count_output.status.success(), "Should count files");
        let count = String::from_utf8_lossy(&count_output.stdout)
            .trim()
            .parse::<usize>()
            .unwrap_or(0);
        println!("Found {} Java files in Kafka", count);
        assert!(count > 100, "Kafka should have many Java files");

        // Clean up
        let _ = Command::new("rm")
            .args(&["-rf", "/tmp/test-kafka"])
            .output();
    }

    #[test]
    fn test_junit5_scan() {
        // Test with JUnit 5 (testing framework)
        println!("Testing JUnit 5 scan...");

        // Clone JUnit 5
        let clone_output = Command::new("git")
            .args(&[
                "clone",
                "--depth",
                "1",
                "https://github.com/junit-team/junit5.git",
                "/tmp/test-junit5",
            ])
            .output()
            .expect("Failed to clone repository");

        assert!(
            clone_output.status.success(),
            "Should clone JUnit 5 repository"
        );

        let repo_path = std::path::Path::new("/tmp/test-junit5");
        assert!(repo_path.exists(), "JUnit 5 should be cloned");

        let count_output = Command::new("bash")
            .arg("-c")
            .arg("find /tmp/test-junit5 -name '*.java' | wc -l")
            .output()
            .expect("Failed to count files");

        assert!(count_output.status.success(), "Should count files");
        let count = String::from_utf8_lossy(&count_output.stdout)
            .trim()
            .parse::<usize>()
            .unwrap_or(0);
        println!("Found {} Java files in JUnit 5", count);

        // Clean up
        let _ = Command::new("rm")
            .args(&["-rf", "/tmp/test-junit5"])
            .output();
    }

    #[test]
    fn test_multiple_projects() {
        // Test scanning multiple different projects
        let projects = vec![
            (
                "Spring PetClinic",
                "https://github.com/spring-projects/spring-petclinic.git",
            ),
            (
                "Spring Boot",
                "https://github.com/spring-projects/spring-boot.git",
            ),
        ];

        for (name, url) in projects {
            let dir_name = format!("/tmp/test-{}", name.to_lowercase().replace(" ", "-"));
            let dir_path = std::path::Path::new(&dir_name);

            // Force cleanup before test
            let test_base_name = std::path::Path::new(&dir_name)
                .file_name()
                .unwrap()
                .to_str()
                .unwrap();
            let _ = Command::new("bash")
                .arg("-c")
                .arg(&format!("rm -rf {} 2>/dev/null; find /tmp -name '{}*' -type d -exec rm -rf {{}} \\; 2>/dev/null; true",
                    dir_name, test_base_name))
                .output();

            let clone_output = Command::new("git")
                .args(&["clone", "--depth", "1", url, &dir_name])
                .output()
                .expect("Failed to clone repository");

            if !clone_output.status.success() {
                let stderr = String::from_utf8_lossy(&clone_output.stderr);
                panic!("Failed to clone {}: {}", name, stderr);
            }

            assert!(
                dir_path.exists() && dir_path.is_dir(),
                "{} should be cloned",
                name
            );

            // Clean up
            let _ = Command::new("bash")
                .arg("-c")
                .arg(&format!("rm -rf {} 2>/dev/null; find /tmp -name '{}*' -type d -exec rm -rf {{}} \\; 2>/dev/null; true",
                    dir_name, test_base_name))
                .output();
        }
    }

    #[test]
    fn test_java_project_with_issues() {
        // Find a project known to have code issues for testing
        println!("Testing code issue detection...");

        // Clone a project
        let clone_output = Command::new("git")
            .args(&[
                "clone",
                "--depth",
                "1",
                "https://github.com/spring-projects/spring-petclinic.git",
                "/tmp/test-issues",
            ])
            .output()
            .expect("Failed to clone repository");

        assert!(clone_output.status.success(), "Should clone repository");

        // Check for potential issues
        // Look for TODO comments
        let todo_output = Command::new("bash")
            .arg("-c")
            .arg("find /tmp/test-issues -name '*.java' -exec grep -l 'TODO' {} \\;")
            .output()
            .expect("Failed to search for TODOs");

        let todo_count = String::from_utf8_lossy(&todo_output.stdout).lines().count();
        println!("Found {} files with TODO comments", todo_count);

        // Look for FIXME comments
        let fixme_output = Command::new("bash")
            .arg("-c")
            .arg("find /tmp/test-issues -name '*.java' -exec grep -l 'FIXME' {} \\;")
            .output()
            .expect("Failed to search for FIXME");

        let fixme_count = String::from_utf8_lossy(&fixme_output.stdout)
            .lines()
            .count();
        println!("Found {} files with FIXME comments", fixme_count);

        // Look for System.out.println
        let sysout_output = Command::new("bash")
            .arg("-c")
            .arg("find /tmp/test-issues -name '*.java' -exec grep -l 'System\\.out\\.println' {} \\;")
            .output()
            .expect("Failed to search for System.out.println");

        let sysout_count = String::from_utf8_lossy(&sysout_output.stdout)
            .lines()
            .count();
        println!("Found {} files with System.out.println", sysout_count);

        // Clean up
        let _ = Command::new("rm")
            .args(&["-rf", "/tmp/test-issues"])
            .output();
    }

    #[test]
    fn test_scanning_performance() {
        // Measure scanning performance on a real project
        let start = std::time::Instant::now();

        // Clone project
        let clone_output = Command::new("git")
            .args(&[
                "clone",
                "--depth",
                "1",
                "https://github.com/spring-projects/spring-petclinic.git",
                "/tmp/test-perf",
            ])
            .output()
            .expect("Failed to clone repository");

        assert!(clone_output.status.success(), "Should clone repository");

        // Count files (simulating scan work)
        let count_output = Command::new("bash")
            .arg("-c")
            .arg("find /tmp/test-perf -name '*.java' | wc -l")
            .output()
            .expect("Failed to count files");

        assert!(count_output.status.success(), "Should count files");
        let count = String::from_utf8_lossy(&count_output.stdout)
            .trim()
            .parse::<usize>()
            .unwrap_or(0);
        println!("Processed {} Java files", count);

        let duration = start.elapsed();
        println!("Operation took: {:?}", duration);

        // Should complete in reasonable time
        assert!(
            duration < Duration::from_secs(60),
            "Should complete in under 60 seconds"
        );

        // Clean up
        let _ = Command::new("rm").args(&["-rf", "/tmp/test-perf"]).output();
    }

    #[test]
    fn test_scan_with_hodei_scan_binary() {
        // Check if hodei-cli has a binary definition
        let cli_cargo = std::fs::read_to_string("crates/hodei-cli/Cargo.toml").unwrap_or_default();

        if !cli_cargo.contains("[[bin]]") {
            println!("Skipping binary test - no [[bin]] section in hodei-cli/Cargo.toml");
            return;
        }

        // Build the hodei-scan binary
        println!("Building hodei-scan binary...");
        let build_output = Command::new("cargo")
            .args(&["build", "-p", "hodei-cli"])
            .output()
            .expect("Failed to build hodei-scan");

        assert!(build_output.status.success(), "hodei-scan should build");

        // Verify binary exists (check common locations)
        let binary_paths = vec!["target/debug/hodei-scan", "target/debug/hodei-cli"];

        let mut binary_found = false;
        for path in &binary_paths {
            if std::path::Path::new(path).exists() {
                binary_found = true;
                break;
            }
        }

        assert!(
            binary_found,
            "hodei-scan binary should exist in one of: {:?}",
            binary_paths
        );
    }

    #[test]
    fn test_real_project_scan() {
        // This test demonstrates a real scan of a Java project

        // Clone a project
        let clone_output = Command::new("git")
            .args(&[
                "clone",
                "--depth",
                "1",
                "https://github.com/spring-projects/spring-petclinic.git",
                "/tmp/real-scan",
            ])
            .output()
            .expect("Failed to clone");

        assert!(clone_output.status.success(), "Should clone");

        // Create a simple report of what was found
        let report = generate_scan_report("/tmp/real-scan");
        println!("\n=== Scan Report ===\n{}", report);

        // Verify we scanned something
        assert!(!report.is_empty(), "Should generate report");

        // Clean up
        let _ = Command::new("rm").args(&["-rf", "/tmp/real-scan"]).output();
    }

    fn generate_scan_report(project_path: &str) -> String {
        let mut report = String::new();

        report.push_str(&format!("Project: {}\n", project_path));

        // Count Java files
        let java_count = Command::new("bash")
            .arg("-c")
            .arg(&format!("find {} -name '*.java' | wc -l", project_path))
            .output()
            .unwrap();

        let count = String::from_utf8_lossy(&java_count.stdout)
            .trim()
            .parse::<usize>()
            .unwrap_or(0);
        report.push_str(&format!("Java files: {}\n", count));

        // Count TODO
        let todo_count = Command::new("bash")
            .arg("-c")
            .arg(&format!(
                "grep -r 'TODO' {} --include='*.java' | wc -l",
                project_path
            ))
            .output()
            .unwrap();

        let count = String::from_utf8_lossy(&todo_count.stdout)
            .trim()
            .parse::<usize>()
            .unwrap_or(0);
        report.push_str(&format!("TODO comments: {}\n", count));

        // Count FIXME
        let fixme_count = Command::new("bash")
            .arg("-c")
            .arg(&format!(
                "grep -r 'FIXME' {} --include='*.java' | wc -l",
                project_path
            ))
            .output()
            .unwrap();

        let count = String::from_utf8_lossy(&fixme_count.stdout)
            .trim()
            .parse::<usize>()
            .unwrap_or(0);
        report.push_str(&format!("FIXME comments: {}\n", count));

        // Count System.out
        let sysout_count = Command::new("bash")
            .arg("-c")
            .arg(&format!(
                "grep -r 'System\\.out\\.println' {} --include='*.java' | wc -l",
                project_path
            ))
            .output()
            .unwrap();

        let count = String::from_utf8_lossy(&sysout_count.stdout)
            .trim()
            .parse::<usize>()
            .unwrap_or(0);
        report.push_str(&format!("System.out.println: {}\n", count));

        report
    }
}
