//! Java code fixtures for testing
//!
//! These fixtures contain sample Java code with various code quality issues

/// Java code with multiple issues (TODOs, FIXMEs, System.out.println, hardcoded credentials)
pub const MULTIPLE_ISSUES_JAVA: &str = r#"
package com.example.service;

import org.springframework.stereotype.Service;

@Service
public class ProblematicService {

    // TODO: Implement proper validation
    public void validateInput(String input) {
        // FIXME: Add null check - crashes on null
        System.out.println("Processing: " + input);
    }

    // Hardcoded credentials - security issue
    private static final String API_KEY = "sk-1234567890abcdef";
    private static final String PASSWORD = "admin123";

    public void connect() {
        // TODO: Implement connection logic
        System.out.println("Connecting...");
    }
}
"#;

/// Java code with only TODOs
pub const TODO_ONLY_JAVA: &str = r#"
package com.example.service;

public class TodoOnlyService {
    // TODO: Implement method
    public void test() {
    }

    // TODO: Add documentation
    // TODO: Add error handling
    public void anotherMethod() {
    }
}
"#;

/// Java code with only FIXMEs
pub const FIXME_ONLY_JAVA: &str = r#"
package com.example.service;

public class FixmeOnlyService {
    // FIXME: This method crashes on null input
    public void test(String input) {
        System.out.println(input.length());
    }

    // FIXME: Hardcoded value should be configurable
    private static final int MAX_RETRIES = 3;
}
"#;

/// Java code with only System.out.println
pub const SYSOUT_ONLY_JAVA: &str = r#"
package com.example.service;

public class SysoutService {
    public void test() {
        System.out.println("Debug message 1");
        System.out.println("Debug message 2");
        System.out.println("Debug message 3");
    }
}
"#;

/// Clean Java code without issues
pub const CLEAN_JAVA: &str = r#"
package com.example.service;

import org.springframework.stereotype.Service;

@Service
public class CleanService {

    public void validateInput(String input) {
        if (input == null) {
            throw new IllegalArgumentException("Input cannot be null");
        }
        // Process input
    }

    public void connect(String apiKey) {
        // Connection logic using injected configuration
    }
}
"#;

/// Spring Boot controller with issues
pub const SPRING_CONTROLLER_JAVA: &str = r#"
package com.example.controller;

import org.springframework.web.bind.annotation.*;

@RestController
@RequestMapping("/api/users")
public class UserController {

    @PostMapping
    // TODO: Add validation
    public void createUser(@RequestBody String user) {
        // TODO: Implement user creation
        System.out.println("Creating user: " + user);
    }

    @GetMapping("/{id}")
    public String getUser(@PathVariable Long id) {
        // FIXME: Add error handling
        return "User " + id;
    }
}
"#;

/// Java class with very long method (code smell)
pub const LONG_METHOD_JAVA: &str = r#"
package com.example.service;

public class LongMethodService {

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
    }
}
"#;

/// Java class with SQL injection vulnerability
pub const SQL_INJECTION_JAVA: &str = r#"
package com.example.repository;

import java.sql.Connection;
import java.sql.Statement;

public class UserRepository {

    private Connection connection;

    // Vulnerable to SQL injection
    public void findUser(String username) throws Exception {
        Statement stmt = connection.createStatement();
        // Unsafe query construction
        String query = "SELECT * FROM users WHERE username = '" + username + "'";
        stmt.executeQuery(query);
    }
}
"#;

/// Java class with XSS vulnerability
pub const XSS_JAVA: &str = r#"
package com.example.controller;

import org.springframework.web.bind.annotation.*;

@RestController
public class CommentController {

    @PostMapping("/comment")
    public String postComment(@RequestParam String comment) {
        // Potential XSS vulnerability
        return "<div>" + comment + "</div>";
    }
}
"#;

/// Java class with deprecated API usage
pub const DEPRECATED_API_JAVA: &str = r#"
package com.example.service;

@Deprecated
public class DeprecatedService {
    @Deprecated
    public void oldMethod() {
        // Use newMethod() instead
    }

    public void newMethod() {
        // Replacement method
    }
}
"#;

/// Java class with complex nested structure
pub const NESTED_CLASSES_JAVA: &str = r#"
package com.example.service;

public class OuterClass {

    private class InnerClass {
        private class DeepInnerClass {
            public void test() {
                System.out.println("Deep nesting");
            }
        }
    }

    public void method() {
        class LocalClass {
            public void localMethod() {
                // Local class implementation
            }
        }
    }
}
"#;

/// Complete Spring Boot application structure
pub const COMPLETE_SPRING_APP: &[(&str, &str)] = &[
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
        System.out.println("Creating: " + user);
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
    // FIXME: Add proper error handling
    public void save(String user) {
        System.out.println("Saving: " + user);
    }
}
        "#,
    ),
];
