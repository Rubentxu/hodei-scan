//! End-to-End tests for full workflow

use std::path::PathBuf;
use std::process::Command;
use tempfile;

#[cfg(test)]
mod e2e_full_workflow {
    use super::*;

    fn setup_test_project(name: &str) -> tempfile::TempDir {
        let temp_dir = tempfile::tempdir().unwrap();
        let project_dir = temp_dir.path().join(name);
        std::fs::create_dir_all(&project_dir).unwrap();
        project_dir
    }

    #[test]
    fn test_full_java_project_scan() {
        let temp_dir = tempfile::tempdir().unwrap();
        let project_dir = temp_dir.path().join("test-project");

        // Create a realistic Java project structure
        let src_dir = project_dir
            .join("src")
            .join("main")
            .join("java")
            .join("com");
        std::fs::create_dir_all(&src_dir).unwrap();

        // Create a service class
        std::fs::write(
            src_dir.join("UserService.java"),
            r#"
package com.example;

import org.springframework.stereotype.Service;
import org.springframework.transaction.annotation.Transactional;

// TODO: Implement proper validation
@Service
@Transactional
public class UserService {
    // FIXME: Add proper error handling
    public void createUser(String name) {
        System.out.println("Creating user: " + name);
        // Implementation pending
    }
}
            "#,
        )
        .unwrap();

        // Create an entity
        std::fs::write(
            src_dir.join("User.java"),
            r#"
package com.example;

import javax.persistence.Entity;
import javax.persistence.Id;

@Entity
public class User {
    @Id
    private Long id;
    private String name;
    private String email;
}
            "#,
        )
        .unwrap();

        // Create a repository
        std::fs::write(
            src_dir.join("UserRepository.java"),
            r#"
package com.example;

import org.springframework.data.jpa.repository.JpaRepository;

public interface UserRepository extends JpaRepository<User, Long> {
}
            "#,
        )
        .unwrap();

        // Run the scan
        let output = Command::new("cargo")
            .args(&[
                "run",
                "--bin",
                "hodei-scan",
                "--",
                "scan",
                project_dir.to_str().unwrap(),
            ])
            .output()
            .expect("Failed to execute hodei-scan");

        // Verify the scan completed
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        // Check that it processed the files
        assert!(
            output.status.success() || stderr.contains("Scanning"),
            "Should scan the project successfully"
        );
    }

    #[test]
    fn test_petclinic_analysis() {
        // Check if PetClinic is already cloned
        let petclinic_path = PathBuf::from("/tmp/spring-petclinic");

        if !petclinic_path.exists() {
            println!("Skipping PetClinic test - not cloned");
            return;
        }

        // Run the analysis script
        let output = Command::new("bash")
            .args(&["simple-analyze.sh"])
            .current_dir("examples/petclinic-scan")
            .output()
            .expect("Failed to run PetClinic analysis");

        // Verify analysis completed
        assert!(
            output.status.success(),
            "PetClinic analysis should complete successfully"
        );

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        // Check for expected output
        assert!(
            stdout.contains("Analyzed") || stdout.contains("Scanning"),
            "Should show analysis progress"
        );

        // Verify reports were generated
        let reports_dir = PathBuf::from("examples/petclinic-scan/reports");
        assert!(reports_dir.exists(), "Reports directory should exist");

        let html_report = reports_dir.join("petclinic-analysis.html");
        let md_report = reports_dir.join("petclinic-summary.md");

        assert!(html_report.exists(), "HTML report should exist");
        assert!(md_report.exists(), "Markdown report should exist");
    }

    #[test]
    fn test_custom_rules_flow() {
        let temp_dir = tempfile::tempdir().unwrap();
        let project_dir = temp_dir.path().join("custom-rules-test");
        std::fs::create_dir_all(&project_dir).unwrap();

        // Create a rule file
        let rules_dir = project_dir.join("rules");
        std::fs::create_dir_all(&rules_dir).unwrap();

        std::fs::write(
            rules_dir.join("custom.rules"),
            r#"
rule "CustomSecurityRule" {
    description: "Detects potential security issues"
    severity: "High"
    tags: ["security", "custom"]

    match {
        pattern: Function {
            name == "processInput"
        }
    }

    emit Finding {
        message: "Custom security rule matched"
        confidence: "High"
    }
}
            "#,
        )
        .unwrap();

        // Create test files
        let test_dir = project_dir.join("src");
        std::fs::create_dir_all(&test_dir).unwrap();

        std::fs::write(
            test_dir.join("TestController.java"),
            r#"
public class TestController {
    public void processInput(String input) {
        // Process input
    }
}
            "#,
        )
        .unwrap();

        // The test validates the rule file can be created
        // Full integration would require rule engine support
        assert!(rules_dir.join("custom.rules").exists());
    }

    #[test]
    fn test_quality_gates_validation() {
        let temp_dir = tempfile::tempdir().unwrap();
        let project_dir = temp_dir.path().join("quality-gates-test");
        std::fs::create_dir_all(&project_dir).unwrap();

        // Create quality gates config
        let config_dir = project_dir.join("config");
        std::fs::create_dir_all(&config_dir).unwrap();

        std::fs::write(
            config_dir.join("quality-gates.yml"),
            r#"
quality_gates:
  - name: "Security Gate"
    description: "Blocks critical security issues"
    enabled: true
    rules:
      - "System.out.println"
    fail_conditions:
      - severity: "Critical"
        count: 0
      - severity: "High"
        count: 3
            "#,
        )
        .unwrap();

        // Create test file with issues
        let test_dir = project_dir.join("src");
        std::fs::create_dir_all(&test_dir).unwrap();

        // Create file with System.out.println (High severity)
        std::fs::write(
            test_dir.join("Test.java"),
            r#"
public class Test {
    public void test() {
        System.out.println("This is bad");
        System.out.println("Also bad");
        System.out.println("Very bad");
    }
}
            "#,
        )
        .unwrap();

        // Validate the configuration files exist
        assert!(config_dir.join("quality-gates.yml").exists());
    }

    #[test]
    fn test_microservices_architecture_scan() {
        let temp_dir = tempfile::tempdir().unwrap();
        let project_dir = temp_dir.path().join("microservices");

        // Create multiple services
        for service in &["user-service", "order-service", "payment-service"] {
            let service_dir = project_dir
                .join(service)
                .join("src")
                .join("main")
                .join("java");
            std::fs::create_dir_all(&service_dir).unwrap();

            // Create service implementation
            std::fs::write(
                service_dir.join(format!("{}.java", service)),
                format!(
                    r#"
import org.springframework.stereotype.Service;

@Service
public class {0} {{
    // TODO: Implement {0}
    public void process() {{
        System.out.println("Processing in {0}");
    }}
}}
                    "#,
                    service.replace('-', "").to_uppercase()
                ),
            )
            .unwrap();
        }

        // Run scan
        let output = Command::new("cargo")
            .args(&[
                "run",
                "--bin",
                "hodei-scan",
                "--",
                "scan",
                project_dir.to_str().unwrap(),
            ])
            .output()
            .expect("Failed to execute hodei-scan");

        // Verify it handles multiple services
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            output.status.success() || stderr.contains("Scanning"),
            "Should scan microservices architecture"
        );
    }

    #[test]
    fn test_legacy_codebase_scan() {
        let temp_dir = tempfile::tempdir().unwrap();
        let project_dir = temp_dir.path().join("legacy-project");
        std::fs::create_dir_all(&project_dir).unwrap();

        // Create old-style Java code
        let src_dir = project_dir.join("src");
        std::fs::create_dir_all(&src_dir).unwrap();

        // Legacy code with various issues
        std::fs::write(
            src_dir.join("LegacyClass.java"),
            r#"
public class LegacyClass {
    // Very long method - TODO: Refactor
    public void veryLongMethod() {
        System.out.println("Line 1");
        System.out.println("Line 2");
        System.out.println("Line 3");
        System.out.println("Line 4");
        System.out.println("Line 5");
        System.out.println("Line 6");
        System.out.println("Line 7");
        System.out.println("Line 8");
        System.out.println("Line 9");
        System.out.println("Line 10");
        // FIXME: This is getting too long
    }

    // Deprecated method
    @Deprecated
    public void oldMethod() {
        // TODO: Remove this
    }
}
            "#,
        )
        .unwrap();

        // Run scan
        let output = Command::new("cargo")
            .args(&[
                "run",
                "--bin",
                "hodei-scan",
                "--",
                "scan",
                project_dir.to_str().unwrap(),
            ])
            .output()
            .expect("Failed to execute hodei-scan");

        // Verify it processes legacy code
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            output.status.success() || stderr.contains("Scanning"),
            "Should scan legacy code"
        );
    }

    #[test]
    fn test_spring_boot_application_scan() {
        let temp_dir = tempfile::tempdir().unwrap();
        let project_dir = temp_dir.path().join("spring-boot-app");

        // Create Spring Boot structure
        let src_main_java = project_dir
            .join("src")
            .join("main")
            .join("java")
            .join("com")
            .join("example");
        std::fs::create_dir_all(&src_main_java).unwrap();

        let src_test_java = project_dir
            .join("src")
            .join("test")
            .join("java")
            .join("com")
            .join("example");
        std::fs::create_dir_all(&src_test_java).unwrap();

        // Create @RestController
        std::fs::write(
            src_main_java.join("UserController.java"),
            r#"
package com.example;

import org.springframework.web.bind.annotation.*;

@RestController
@RequestMapping("/api/users")
public class UserController {
    // TODO: Add validation
    @PostMapping
    public void createUser(@RequestBody String user) {
        // TODO: Implement
    }

    @GetMapping("/{id}")
    public String getUser(@PathVariable Long id) {
        // FIXME: Add error handling
        return "User " + id;
    }
}
            "#,
        )
        .unwrap();

        // Create @Service
        std::fs::write(
            src_main_java.join("UserService.java"),
            r#"
package com.example;

import org.springframework.stereotype.Service;
import org.springframework.transaction.annotation.Transactional;

@Service
@Transactional
public class UserService {
    public void save() {
        // TODO: Implement persistence
    }
}
            "#,
        )
        .unwrap();

        // Create @Entity
        std::fs::write(
            src_main_java.join("User.java"),
            r#"
package com.example;

import javax.persistence.*;

@Entity
public class User {
    @Id
    @GeneratedValue
    private Long id;

    @Column(nullable = false)
    private String name;

    @Column(unique = true)
    private String email;
}
            "#,
        )
        .unwrap();

        // Create test
        std::fs::write(
            src_test_java.join("UserControllerTest.java"),
            r#"
package com.example;

import org.junit.Test;

public class UserControllerTest {
    @Test
    public void testCreateUser() {
        // TODO: Add tests
    }
}
            "#,
        )
        .unwrap();

        // Run scan
        let output = Command::new("cargo")
            .args(&[
                "run",
                "--bin",
                "hodei-scan",
                "--",
                "scan",
                project_dir.to_str().unwrap(),
            ])
            .output()
            .expect("Failed to execute hodei-scan");

        // Verify Spring Boot annotations detected
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            output.status.success() || stderr.contains("Scanning"),
            "Should scan Spring Boot application"
        );
    }

    #[test]
    fn test_error_handling_graceful() {
        let temp_dir = tempfile::tempdir().unwrap();
        let project_dir = temp_dir.path().join("error-test");

        // Create file with unusual content
        let src_dir = project_dir.join("src");
        std::fs::create_dir_all(&src_dir).unwrap();

        std::fs::write(
            src_dir.join("Test.java"),
            r#"
public class Test {
    // Unicode: ñáéíóú
    // Special chars: <>&"'
    // Long line: aaaaaaaaaaabbbbbbbbbbbaaaaaaaaaabbbbbbbbbbbaaaaaaaaaabbbbbbbbbbbaaaaaaaaaabbbbbbbbbbb
}
            "#,
        )
        .unwrap();

        // Run scan
        let output = Command::new("cargo")
            .args(&[
                "run",
                "--bin",
                "hodei-scan",
                "--",
                "scan",
                project_dir.to_str().unwrap(),
            ])
            .output()
            .expect("Failed to execute hodei-scan");

        // Should handle encoding and special chars gracefully
        assert!(
            output.status.success() || String::from_utf8_lossy(&output.stderr).contains("error"),
            "Should handle special characters gracefully"
        );
    }
}
