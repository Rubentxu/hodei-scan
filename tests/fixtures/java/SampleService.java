package com.example.service;

import org.springframework.stereotype.Service;
import org.springframework.transaction.annotation.Transactional;

/**
 * Sample service with various code quality issues for testing
 */
@Service
@Transactional
public class SampleService {

    // TODO: Implement proper validation
    public void validateInput(String input) {
        // FIXME: Add null check
        System.out.println("Processing: " + input);
    }

    // Long method - should be refactored
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
    }

    // Hardcoded credentials - security issue
    private static final String API_KEY = "sk-1234567890abcdef";
    private static final String PASSWORD = "admin123";

    public void connect() {
        // Connection logic
    }
}
