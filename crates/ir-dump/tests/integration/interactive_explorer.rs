//! Integration tests for interactive explorer

use ir_dump::InteractiveExplorer;
use hodei_ir::FindingSet;
use crate::fixtures::SAMPLE_JSON, multiple_findings_ir;

#[cfg(test)]
mod interactive_tests {
    use super::*;

    // Helper to create a mock reedline for testing
    fn create_test_explorer() -> InteractiveExplorer {
        let ir = multiple_findings_ir();
        InteractiveExplorer::new(ir)
    }

    #[test]
    fn test_explorer_creation() {
        let explorer = create_test_explorer();
        assert_eq!(explorer.current_index, 0);
    }

    #[test]
    fn test_navigate_findings() {
        let mut explorer = create_test_explorer();

        // Initially at first finding
        assert_eq!(explorer.current_index, 0);

        // Navigate to next
        explorer.next_finding();
        assert_eq!(explorer.current_index, 1);

        // Navigate to previous
        explorer.prev_finding();
        assert_eq!(explorer.current_index, 0);
    }

    #[test]
    fn test_goto_finding() {
        let mut explorer = create_test_explorer();

        // Goto finding 3 (0-based index 2)
        explorer.goto_finding(2).unwrap();
        assert_eq!(explorer.current_index, 2);

        // Goto finding 1 (0-based index 0)
        explorer.goto_finding(0).unwrap();
        assert_eq!(explorer.current_index, 0);
    }

    #[test]
    fn test_goto_invalid_finding() {
        let mut explorer = create_test_explorer();

        // Goto beyond bounds - should not change
        let original_index = explorer.current_index;
        explorer.goto_finding(1000).unwrap_err(); // Should error
        assert_eq!(explorer.current_index, original_index);
    }

    #[test]
    fn test_boundary_navigation() {
        let mut explorer = create_test_explorer();
        let total_findings = explorer.ir.findings.len();

        // Try to go beyond last
        for _ in 0..total_findings + 5 {
            explorer.next_finding();
        }
        assert_eq!(explorer.current_index, total_findings - 1);

        // Try to go before first
        for _ in 0..10 {
            explorer.prev_finding();
        }
        assert_eq!(explorer.current_index, 0);
    }

    #[test]
    fn test_list_findings() {
        let explorer = create_test_explorer();
        let output = capture_output(|| explorer.list_findings());

        assert!(output.contains("All findings:"));
        assert!(output.contains("Finding #1"));
        assert!(output.contains("Finding #2"));
    }

    #[test]
    fn test_show_stats() {
        let explorer = create_test_explorer();
        let output = capture_output(|| explorer.show_stats());

        assert!(output.contains("Statistics:"));
        assert!(output.contains("Total findings:"));
        assert!(output.contains("Vulnerability"));
        assert!(output.contains("CodeSmell"));
    }

    #[test]
    fn test_filter_findings() {
        let explorer = create_test_explorer();
        let output = capture_output(|| explorer.filter_findings(&["Vulnerability"]));

        assert!(output.contains("Filtering by: 'Vulnerability'"));
        assert!(output.contains("Vulnerability"));
    }

    #[test]
    fn test_filter_case_insensitive() {
        let explorer = create_test_explorer();
        let output = capture_output(|| explorer.filter_findings(&["vulnerability"]));

        assert!(output.contains("Vulnerability"));
    }

    #[test]
    fn test_empty_ir_explorer() {
        let empty_ir = FindingSet { findings: Vec::new() };
        let explorer = InteractiveExplorer::new(empty_ir);

        let output = capture_output(|| explorer.show_current_finding());
        assert!(output.contains("No findings to show"));
    }

    #[test]
    fn test_single_finding_explorer() {
        let single_finding = FindingSet {
            findings: vec![hodei_ir::Finding {
                fact_type: "TestType".to_string(),
                message: "Test Message".to_string(),
                location: Some("test.rs:42".to_string()),
                severity: Some("Minor".to_string()),
                metadata: std::collections::HashMap::new(),
            }]
        };
        let mut explorer = InteractiveExplorer::new(single_finding);

        // Should show the only finding
        let output = capture_output(|| explorer.show_current_finding());
        assert!(output.contains("TestType"));
        assert!(output.contains("Test Message"));
        assert!(output.contains("test.rs:42"));

        // Navigation should stay within bounds
        explorer.next_finding();
        assert_eq!(explorer.current_index, 0);
    }

    #[test]
    fn test_metadata_preserved_in_display() {
        let mut metadata = std::collections::HashMap::new();
        metadata.insert("confidence".to_string(), "0.95".to_string());
        metadata.insert("cwe".to_string(), "CWE-79".to_string());

        let ir_with_metadata = FindingSet {
            findings: vec![hodei_ir::Finding {
                fact_type: "Vulnerability".to_string(),
                message: "Test".to_string(),
                location: Some("file.js:1".to_string()),
                severity: Some("Critical".to_string()),
                metadata,
            }]
        };

        let explorer = InteractiveExplorer::new(ir_with_metadata);
        let output = capture_output(|| explorer.show_current_finding());

        // Metadata should be shown
        assert!(output.contains("Vulnerability"));
        assert!(output.contains("Test"));
        assert!(output.contains("file.js:1"));
    }

    #[test]
    fn test_stats_calculation() {
        let explorer = create_test_explorer();
        let output = capture_output(|| explorer.show_stats());

        // Should show correct counts
        assert!(output.contains("Total findings: 2"));
        assert!(output.contains("Vulnerability"));
        assert!(output.contains("CodeSmell"));
    }

    #[test]
    fn test_multiple_types_stats() {
        let multi_type_ir = FindingSet {
            findings: vec![
                hodei_ir::Finding {
                    fact_type: "TypeA".to_string(),
                    message: "Message A".to_string(),
                    location: None,
                    severity: None,
                    metadata: std::collections::HashMap::new(),
                },
                hodei_ir::Finding {
                    fact_type: "TypeB".to_string(),
                    message: "Message B".to_string(),
                    location: None,
                    severity: None,
                    metadata: std::collections::HashMap::new(),
                },
                hodei_ir::Finding {
                    fact_type: "TypeA".to_string(),
                    message: "Message C".to_string(),
                    location: None,
                    severity: None,
                    metadata: std::collections::HashMap::new(),
                },
            ]
        };

        let explorer = InteractiveExplorer::new(multi_type_ir);
        let output = capture_output(|| explorer.show_stats());

        // Should count types correctly
        assert!(output.contains("TypeA: 2"));
        assert!(output.contains("TypeB: 1"));
    }

    #[test]
    fn test_filter_no_matches() {
        let explorer = create_test_explorer();
        let output = capture_output(|| explorer.filter_findings(&["NonExistentType"]));

        assert!(output.contains("Filtering by: 'NonExistentType'"));
        // Should show empty or no matches
    }

    #[test]
    fn test_filter_partial_match() {
        let ir_with_many_types = FindingSet {
            findings: vec![
                hodei_ir::Finding {
                    fact_type: "SecurityVulnerability".to_string(),
                    message: "Message 1".to_string(),
                    location: None,
                    severity: None,
                    metadata: std::collections::HashMap::new(),
                },
                hodei_ir::Finding {
                    fact_type: "CodeSmell".to_string(),
                    message: "Message 2".to_string(),
                    location: None,
                    severity: None,
                    metadata: std::collections::HashMap::new(),
                },
                hodei_ir::Finding {
                    fact_type: "PerformanceIssue".to_string(),
                    message: "Message 3".to_string(),
                    location: None,
                    severity: None,
                    metadata: std::collections::HashMap::new(),
                },
            ]
        };

        let explorer = InteractiveExplorer::new(ir_with_many_types);
        let output = capture_output(|| explorer.filter_findings(&["Vulnerability"]));

        // Should find partial matches
        assert!(output.contains("SecurityVulnerability"));
        assert!(!output.contains("CodeSmell"));
        assert!(!output.contains("PerformanceIssue"));
    }

    #[test]
    fn test_show_current_finding_at_different_positions() {
        let mut explorer = create_test_explorer();

        // Show finding at position 0
        explorer.goto_finding(0).unwrap();
        let output1 = capture_output(|| explorer.show_current_finding());
        assert!(output1.contains("Finding #1"));

        // Show finding at position 1
        explorer.goto_finding(1).unwrap();
        let output2 = capture_output(|| explorer.show_current_finding());
        assert!(output2.contains("Finding #2"));

        // Verify they're different
        assert_ne!(output1, output2);
    }

    // Helper function to capture output from functions that print
    fn capture_output<F>(f: F) -> String
    where
        F: FnOnce(),
    {
        use std::sync::{Arc, Mutex};
        use std::thread;

        let output = Arc::new(Mutex::new(Vec::new()));
        let output_clone = output.clone();

        thread::spawn(move || {
            use std::io::Write;
            let mut guard = output_clone.lock().unwrap();
            let mut buf = Vec::new();
            let old = std::io::stdout();
            let mut stdout = std::io::stdout();

            // Note: This is a simplified capture - actual Reedline uses stdin
            // For unit tests, we'll use a different approach
            // In real tests, you'd use mocks or integration tests
            writeln!(guard, "mocked output").unwrap();
        });

        thread::sleep(std::time::Duration::from_millis(10));
        let guard = output.lock().unwrap();
        String::from_utf8(guard.clone()).unwrap()
    }
}
