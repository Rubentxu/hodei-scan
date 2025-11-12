//! Project fixtures for E2E testing
//!
//! These fixtures contain complete project structures for testing

/// Small Java project with minimal issues
pub const SMALL_PROJECT: &[(&str, &str)] = &[
    (
        "pom.xml",
        r#"
<project xmlns="http://maven.apache.org/POM/4.0.0">
    <modelVersion>4.0.0</modelVersion>
    <groupId>com.example</groupId>
    <artifactId>small-project</artifactId>
    <version>1.0.0</version>
</project>
        "#,
    ),
    (
        "src/main/java/com/example/SimpleService.java",
        r#"
package com.example;

public class SimpleService {
    public void process() {
        // Simple implementation
    }
}
        "#,
    ),
];

/// Medium Java project with various issues
pub const MEDIUM_PROJECT: &[(&str, &str)] = &[
    (
        "pom.xml",
        r#"
<project xmlns="http://maven.apache.org/POM/4.0.0">
    <modelVersion>4.0.0</modelVersion>
    <groupId>com.example</groupId>
    <artifactId>medium-project</artifactId>
    <version>1.0.0</version>
</project>
        "#,
    ),
    (
        "src/main/java/com/example/Service.java",
        r#"
package com.example;

public class Service {
    // TODO: Implement validation
    public void process(String input) {
        // FIXME: Add null check
        System.out.println(input);
    }
}
        "#,
    ),
    (
        "src/main/java/com/example/Controller.java",
        r#"
package com.example;

public class Controller {
    public void handle() {
        // TODO: Implement
        System.out.println("Handling request");
    }
}
        "#,
    ),
];

/// Spring Boot project structure
pub const SPRING_BOOT_PROJECT: &[(&str, &str)] = &[
    (
        "pom.xml",
        r#"
<project xmlns="http://maven.apache.org/POM/4.0.0">
    <modelVersion>4.0.0</modelVersion>
    <parent>
        <groupId>org.springframework.boot</groupId>
        <artifactId>spring-boot-starter-parent</artifactId>
        <version>3.0.0</version>
    </parent>
    <groupId>com.example</groupId>
    <artifactId>spring-boot-app</artifactId>
</project>
        "#,
    ),
    (
        "src/main/java/com/example/Application.java",
        r#"
package com.example;

import org.springframework.boot.SpringApplication;
import org.springframework.boot.autoconfigure.SpringBootApplication;

@SpringBootApplication
public class Application {
    public static void main(String[] args) {
        SpringApplication.run(Application.class, args);
    }
}
        "#,
    ),
    (
        "src/main/java/com/example/controller/UserController.java",
        r#"
package com.example.controller;

import org.springframework.web.bind.annotation.*;

@RestController
@RequestMapping("/api/users")
public class UserController {
    // TODO: Add validation
    @PostMapping
    public void createUser(@RequestBody String user) {
        // TODO: Implement user creation
        System.out.println("Creating: " + user);
    }

    @GetMapping("/{id}")
    public String getUser(@PathVariable Long id) {
        // FIXME: Add error handling
        return "User " + id;
    }
}
        "#,
    ),
    (
        "src/main/java/com/example/service/UserService.java",
        r#"
package com.example.service;

import org.springframework.stereotype.Service;

@Service
public class UserService {
    // Hardcoded password - security issue
    private static final String PASSWORD = "admin123";

    public void save(String user) {
        System.out.println("Saving: " + user);
    }
}
        "#,
    ),
];

/// Project with security vulnerabilities
pub const SECURITY_VULNERABLE_PROJECT: &[(&str, &str)] = &[
    (
        "pom.xml",
        r#"
<project xmlns="http://maven.apache.org/POM/4.0.0">
    <modelVersion>4.0.0</modelVersion>
    <groupId>com.example</groupId>
    <artifactId>security-vulnerable</artifactId>
    <version>1.0.0</version>
</project>
        "#,
    ),
    (
        "src/main/java/com/example/SqlInjection.java",
        r#"
package com.example;

import java.sql.Connection;
import java.sql.Statement;

public class SqlInjection {
    private Connection connection;

    // Vulnerable to SQL injection
    public void findUser(String username) throws Exception {
        Statement stmt = connection.createStatement();
        String query = "SELECT * FROM users WHERE username = '" + username + "'";
        stmt.executeQuery(query);
    }
}
        "#,
    ),
    (
        "src/main/java/com/example/XssExample.java",
        r#"
package com.example;

public class XssExample {
    public String renderComment(String comment) {
        // Potential XSS
        return "<div>" + comment + "</div>";
    }
}
        "#,
    ),
    (
        "src/main/java/com/example/Credentials.java",
        r#"
package com.example;

public class Credentials {
    // Hardcoded credentials - critical security issue
    private static final String API_KEY = "sk-1234567890abcdef";
    private static final String PASSWORD = "admin123";
    private static final String SECRET_TOKEN = "secret-token-xyz";
}
        "#,
    ),
];

/// Project with code quality issues
pub const CODE_QUALITY_PROJECT: &[(&str, &str)] = &[
    (
        "pom.xml",
        r#"
<project xmlns="http://maven.apache.org/POM/4.0.0">
    <modelVersion>4.0.0</modelVersion>
    <groupId>com.example</groupId>
    <artifactId>code-quality-issues</artifactId>
    <version>1.0.0</version>
</project>
        "#,
    ),
    (
        "src/main/java/com/example/LongMethod.java",
        r#"
package com.example;

public class LongMethod {
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
        System.out.println("Line 11");
        System.out.println("Line 12");
        System.out.println("Line 13");
        System.out.println("Line 14");
        System.out.println("Line 15");
        System.out.println("Line 16");
    }
}
        "#,
    ),
    (
        "src/main/java/com/example/TodoComments.java",
        r#"
package com.example;

public class TodoComments {
    // TODO: Implement this feature
    public void feature1() {
    }

    // TODO: Add documentation
    // TODO: Handle edge cases
    public void feature2() {
    }

    // FIXME: This method crashes on null
    public void feature3(String input) {
        System.out.println(input.length());
    }
}
        "#,
    ),
    (
        "src/main/java/com/example/DebugStatements.java",
        r#"
package com.example;

public class DebugStatements {
    public void process() {
        System.out.println("Debug: starting process");
        System.out.println("Debug: processing data");
        System.out.println("Debug: finished");
    }
}
        "#,
    ),
];

/// Multi-module Maven project
pub const MULTIMODULE_PROJECT: &[(&str, &str)] = &[
    (
        "pom.xml",
        r#"
<project xmlns="http://maven.apache.org/POM/4.0.0">
    <modelVersion>4.0.0</modelVersion>
    <groupId>com.example</groupId>
    <artifactId>multimodule-project</artifactId>
    <version>1.0.0</version>
    <modules>
        <module>module-a</module>
        <module>module-b</module>
    </modules>
</project>
        "#,
    ),
    (
        "module-a/pom.xml",
        r#"
<project xmlns="http://maven.apache.org/POM/4.0.0">
    <modelVersion>4.0.0</modelVersion>
    <parent>
        <groupId>com.example</groupId>
        <artifactId>multimodule-project</artifactId>
        <version>1.0.0</version>
    </parent>
    <artifactId>module-a</artifactId>
</project>
        "#,
    ),
    (
        "module-a/src/main/java/com/example/ModuleAService.java",
        r#"
package com.example;

public class ModuleAService {
    // TODO: Implement
    public void methodA() {
    }
}
        "#,
    ),
    (
        "module-b/pom.xml",
        r#"
<project xmlns="http://maven.apache.org/POM/4.0.0">
    <modelVersion>4.0.0</modelVersion>
    <parent>
        <groupId>com.example</groupId>
        <artifactId>multimodule-project</artifactId>
        <version>1.0.0</version>
    </parent>
    <artifactId>module-b</artifactId>
</project>
        "#,
    ),
    (
        "module-b/src/main/java/com/example/ModuleBService.java",
        r#"
package com.example;

public class ModuleBService {
    // FIXME: Fix this
    public void methodB() {
        System.out.println("Debug");
    }
}
        "#,
    ),
];

/// Legacy Java project (Java 8 style)
pub const LEGACY_PROJECT: &[(&str, &str)] = &[
    (
        "pom.xml",
        r#"
<project xmlns="http://maven.apache.org/POM/4.0.0">
    <modelVersion>4.0.0</modelVersion>
    <groupId>com.example</groupId>
    <artifactId>legacy-project</artifactId>
    <version>1.0.0</version>
    <properties>
        <maven.compiler.source>1.8</maven.compiler.source>
        <maven.compiler.target>1.8</maven.compiler.target>
    </properties>
</project>
        "#,
    ),
    (
        "src/main/java/com/example/LegacyService.java",
        r#"
package com.example;

@Deprecated
public class LegacyService {
    @Deprecated
    public void oldMethod() {
        // TODO: Remove this deprecated method
        System.out.println("Old implementation");
    }

    public void newMethod() {
        // Modern implementation
    }
}
        "#,
    ),
];

/// Test project with JUnit tests
pub const TEST_PROJECT: &[(&str, &str)] = &[
    (
        "pom.xml",
        r#"
<project xmlns="http://maven.apache.org/POM/4.0.0">
    <modelVersion>4.0.0</modelVersion>
    <groupId>com.example</groupId>
    <artifactId>test-project</artifactId>
    <version>1.0.0</version>
</project>
        "#,
    ),
    (
        "src/main/java/com/example/Service.java",
        r#"
package com.example;

public class Service {
    public int add(int a, int b) {
        return a + b;
    }
}
        "#,
    ),
    (
        "src/test/java/com/example/ServiceTest.java",
        r#"
package com.example;

import org.junit.Test;

public class ServiceTest {
    @Test
    public void testAdd() {
        Service service = new Service();
        // TODO: Add assertions
        service.add(1, 2);
    }
}
        "#,
    ),
];

/// Project with all types of issues
pub const ALL_ISSUES_PROJECT: &[(&str, &str)] = &[
    (
        "pom.xml",
        r#"
<project xmlns="http://maven.apache.org/POM/4.0.0">
    <modelVersion>4.0.0</modelVersion>
    <groupId>com.example</groupId>
    <artifactId>all-issues</artifactId>
    <version>1.0.0</version>
</project>
        "#,
    ),
    (
        "src/main/java/com/example/AllIssues.java",
        r#"
package com.example;

import java.sql.Connection;
import java.sql.Statement;

public class AllIssues {
    private static final String PASSWORD = "admin123";

    // TODO: Implement validation
    // FIXME: Add null check
    public void method(String input) throws Exception {
        // System.out.println for debugging
        System.out.println("Input: " + input);

        // SQL injection vulnerability
        Statement stmt = connection.createStatement();
        String query = "SELECT * FROM table WHERE id = '" + input + "'";
        stmt.executeQuery(query);

        // XSS vulnerability
        String output = "<div>" + input + "</div>";

        // Very long method
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
        System.out.println("Line 11");
        System.out.println("Line 12");
        System.out.println("Line 13");
        System.out.println("Line 14");
        System.out.println("Line 15");
        System.out.println("Line 16");
    }
}
        "#,
    ),
];
