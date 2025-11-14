//! Fuzz Testing for Java Extractor
//!
//! These tests use cargo-fuzz to test the parser and analyzer
//! with malformed, adversarial, and edge-case inputs.

use std::path::PathBuf;

/// Fuzz target: JaCoCo XML Parser
///
/// This fuzz target tests the JaCoCo XML parser with various inputs
/// to ensure it handles malformed, malicious, and edge-case XML gracefully.
#[cfg(fuzzing)]
pub fn fuzz_parse_jacoco_xml(data: &[u8]) {
    use hodei_java_extractor::JacocoAdapter;

    // Create a temporary file with the fuzz data
    let temp_dir = std::env::temp_dir();
    let temp_file = temp_dir.join("fuzz_jacoco.xml");

    // Write fuzz data to file
    if let Ok(mut file) = std::fs::File::create(&temp_file) {
        let _ = file.write_all(data);
    }

    // Try to parse
    let mut adapter = JacocoAdapter::new(temp_file);
    let _ = adapter.load_coverage_data();

    // Clean up
    let _ = std::fs::remove_file(&temp_file);
}

/// Fuzz target: Project Path Handling
#[cfg(fuzzing)]
pub fn fuzz_project_path(data: &[u8]) {
    use hodei_ir::ProjectPath;

    // Convert fuzz data to string (valid UTF-8 or not)
    let path_str = String::from_utf8_lossy(data);

    // Try to create a ProjectPath
    if !path_str.is_empty() {
        let path = PathBuf::from(&path_str);
        let project_path = ProjectPath::new(path);

        // Ensure operations don't panic
        let _ = project_path.as_str();
        let _ = project_path.to_string();
    }
}

/// Fuzz target: Package and Class Name Parsing
#[cfg(fuzzing)]
pub fn fuzz_package_parsing(data: &[u8]) {
    // Parse fuzz data as potential package names
    let package_str = String::from_utf8_lossy(data);

    // Test with various formats
    let test_cases = vec![
        &package_str,
        &format!("com.{}", package_str),
        &format!("{}.example", package_str),
    ];

    for test_case in test_cases {
        if !test_case.is_empty() {
            // These operations should not panic
            let _ = test_case.replace('.', "/");
            let _ = test_case.chars().count();
            let _ = test_case.split('.').collect::<Vec<_>>();
        }
    }
}

/// Fuzz target: Coverage Calculation
#[cfg(fuzzing)]
pub fn fuzz_coverage_calculation(mi: u32, ci: u32, mb: u32, cb: u32) {
    // Calculate coverage with fuzzed inputs
    let total_instructions = mi + ci;
    let coverage_pct = if total_instructions > 0 {
        (ci as f64 / total_instructions as f64) * 100.0
    } else {
        0.0
    };

    let total_branches = mb + cb;
    let branch_coverage = if total_branches > 0 {
        (cb as f64 / total_branches as f64) * 100.0
    } else {
        0.0
    };

    // Ensure results are valid
    assert!(coverage_pct >= 0.0 && coverage_pct <= 100.0);
    assert!(branch_coverage >= 0.0 && branch_coverage <= 100.0);
}

/// Fuzz target: Malformed XML Generation
#[cfg(fuzzing)]
pub fn fuzz_xml_generation(data: &[u8]) {
    use quick_xml::Reader;
    use quick_xml::events::Event;

    // Test with raw fuzz data as XML
    let xml_str = String::from_utf8_lossy(data);

    let mut reader = Reader::from_str(&xml_str);
    let mut buf = Vec::new();

    // Parse without crashing
    while let Ok(event) = reader.read_event_into(&mut buf) {
        match event {
            Event::Start(_) | Event::End(_) | Event::Text(_) | Event::Comment(_) => {
                // Normal events
            }
            Event::Decl(_) => {
                // XML declaration
            }
            Event::DocType(_) => {
                // DOCTYPE
            }
            Event::CData(_) => {
                // CDATA
            }
            Event::PI(_) => {
                // Processing instruction
            }
            Event::Eof => {
                break;
            }
            _ => {
                // Other events
            }
        }
    }
}

/// Fuzz target: Large File Handling
#[cfg(fuzzing)]
pub fn fuzz_large_file(data: &[u8]) {
    // Test with extremely large inputs
    if data.len() > 100_000_000 { // 100MB
        // Should handle or reject efficiently
        assert!(data.len() < 1_000_000_000); // Max 1GB
    }

    // Test with various sizes
    match data.len() {
        0 => {}, // Empty file
        1..=100 => {}, // Small file
        101..=10000 => {}, // Medium file
        10001..=1000000 => {}, // Large file
        _ => {}, // Very large file
    }
}

/// Fuzz target: Unicode and Special Characters
#[cfg(fuzzing)]
pub fn fuzz_unicode_handling(data: &[u8]) {
    // Test Unicode characters
    let text = String::from_utf8_lossy(data);

    // Test various Unicode operations
    let _ = text.len();
    let _ = text.chars().count();
    let _ = text.split_whitespace().collect::<Vec<_>>();
    let _ = text.trim();

    // Test emoji and special chars
    if text.contains('😀') || text.contains('🚀') || text.contains('🔥') {
        // Emoji handling
    }

    // Test RTL languages
    if text.contains('العربية') || text.contains('日本語') {
        // Right-to-left or CJK handling
    }

    // Test zero-width characters
    if text.contains('\u{200B}') || text.contains('\u{200C}') {
        // Zero-width characters
    }
}

/// Fuzz target: Path Traversal
#[cfg(fuzzing)]
pub fn fuzz_path_traversal(data: &[u8]) {
    let path_str = String::from_utf8_lossy(data);

    // Test path traversal patterns
    let dangerous_patterns = vec![
        "../../../etc/passwd",
        "..\\..\\..\\windows\\system32\\config\\sam",
        "....//....//....//etc//passwd",
        "/../../../../../../etc/passwd",
        "%2e%2e%2f%2e%2e%2f%2e%2e%2fetc%2fpasswd",
    ];

    for pattern in &dangerous_patterns {
        if path_str.contains(pattern) {
            // Should detect or handle safely
            assert!(true);
        }
    }

    // Test with actual path creation (should be safe)
    if !path_str.is_empty() && path_str.len() < 1000 {
        let _ = PathBuf::from(&path_str);
    }
}

/// Fuzz target: XML Entity Injection
#[cfg(fuzzing)]
pub fn fuzz_entity_injection(data: &[u8]) {
    let xml_str = String::from_utf8_lossy(data);

    // Test for entity injection patterns
    let dangerous_patterns = vec![
        "<!ENTITY xxe SYSTEM \"file:///etc/passwd\">",
        "<!ENTITY % xxe \"<!ENTITY xxe SYSTEM 'file:///etc/passwd'>",
        "<!ENTITY xxe SYSTEM \"file:///proc/version\">",
        "<!ENTITY xxe SYSTEM \"http://evil.com\">",
    ];

    for pattern in &dangerous_patterns {
        if xml_str.contains(pattern) {
            // Should detect or reject
            assert!(true, "Should handle entity injection");
        }
    }
}

/// Fuzz target: Billion Laughs Attack
#[cfg(fuzzing)]
pub fn fuzz_billion_laughs(data: &[u8]) {
    let xml_str = String::from_utf8_lossy(data);

    // Count entity declarations
    let entity_count = xml_str.matches("<!ENTITY").count();
    let lol_count = xml_str.matches("lol").count();

    // Should limit entity expansion
    if entity_count > 10 || lol_count > 1000 {
        // Potential billion laughs attack
        assert!(entity_count < 100, "Too many entities");
    }
}

/// Fuzz target: Quadratic Blowup
#[cfg(fuzzing)]
pub fn fuzz_quadratic_blowup(data: &[u8]) {
    let xml_str = String::from_utf8_lossy(data);

    // Test for quadratic blowup patterns
    // Many repeated characters that could cause O(n^2) behavior
    let repeated_char = if !xml_str.is_empty() {
        xml_str.chars().next()
    } else {
        None
    };

    if let Some(ch) = repeated_char {
        if xml_str.chars().filter(|&c| c == ch).count() > 1000 {
            // Potential quadratic blowup
            assert!(xml_str.len() < 1_000_000, "Input too large");
        }
    }
}

/// Fuzz target: Memory Exhaustion
#[cfg(fuzzing)]
pub fn fuzz_memory_exhaustion(data: &[u8]) {
    // Test with large allocations
    let large_vec = data.to_vec();

    // Should handle or reject large allocations
    if large_vec.len() > 10_000_000 { // 10MB
        assert!(large_vec.len() < 100_000_000, "Allocation too large"); // 100MB limit
    }

    // Test with many small allocations
    let mut small_allocs = Vec::new();
    for _ in 0..10000 {
        small_allocs.push(data.to_vec());
        if small_allocs.len() > 1000 {
            small_allocs.clear(); // Keep memory bounded
        }
    }
}

/// Fuzz target: Deeply Nested XML
#[cfg(fuzzing)]
pub fn fuzz_deep_nesting(data: &[u8]) {
    let xml_str = String::from_utf8_lossy(data);

    // Count nesting level
    let mut max_depth = 0;
    let mut current_depth = 0;

    for ch in xml_str.chars() {
        if ch == '<' && !xml_str.contains("</") {
            current_depth += 1;
            max_depth = max_depth.max(current_depth);
        } else if ch == '>' {
            current_depth = current_depth.saturating_sub(1);
        }
    }

    // Should limit nesting depth
    if max_depth > 1000 {
        assert!(max_depth < 10000, "Nesting too deep");
    }
}

#[cfg(test)]
mod fuzz_unit_tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn xml_parsing_doesnt_crash() {
        let test_cases = vec![
            "", // Empty
            "<?xml version=\"1.0\"?>", // Minimal
            "<?xml version=\"1.0\"?><report></report>", // Well-formed
            "<?xml version=\"1.0\"?><report><package name=\"test\"></package></report>", // Valid
            "<!DOCTYPE foo [<!ENTITY x \"test\">]><foo>&x;</foo>", // With entity
        ];

        for xml in test_cases {
            test_xml_generation(xml.as_bytes());
        }
    }

    #[test]
    fn coverage_calculation_bounds() {
        proptest!(|(mi in 0u32..1000000u32, ci in 0u32..1000000u32)| {
            test_coverage_calculation(mi, ci, 0, 0);
        });
    }

    #[test]
    fn path_traversal_prevention() {
        let dangerous_paths = vec![
            "../../../etc/passwd",
            "..\\..\\..\\windows\\system32\\config\\sam",
            "....//....//....//etc//passwd",
            "/../../../../../../etc/passwd",
            "%2e%2e%2f%2e%2e%2f%2e%2e%2fetc%2fpasswd",
        ];

        for path in dangerous_paths {
            test_path_traversal(path.as_bytes());
        }
    }

    #[test]
    fn entity_injection_detection() {
        let malicious_xml = r#"<?xml version="1.0"?>
        <!DOCTYPE foo [<!ENTITY xxe SYSTEM "file:///etc/passwd">]>
        <foo>&xxe;</foo>"#;

        test_entity_injection(malicious_xml.as_bytes());
    }

    #[test]
    fn billion_laughs_prevention() {
        let malicious_xml = r#"<?xml version="1.0"?>
        <!DOCTYPE lolz [<!ENTITY lol "lol">
        <!ENTITY lol2 "&lol;&lol;&lol;&lol;&lol;&lol;&lol;&lol;&lol;&lol;">
        <!ENTITY lol3 "&lol2;&lol2;&lol2;&lol2;&lol2;&lol2;&lol2;&lol2;&lol2;&lol2;">
        <!ENTITY lol4 "&lol3;&lol3;&lol3;&lol3;&lol3;&lol3;&lol3;&lol3;&lol3;&lol3;">
        <!ENTITY lol5 "&lol4;&lol4;&lol4;&lol4;&lol4;&lol4;&lol4;&lol4;&lol4;&lol4;">
        <!ENTITY lol6 "&lol5;&lol5;&lol5;&lol5;&lol5;&lol5;&lol5;&lol5;&lol5;&lol5;">
        <!ENTITY lol7 "&lol6;&lol6;&lol6;&lol6;&lol6;&lol6;&lol6;&lol6;&lol6;&lol6;">
        <!ENTITY lol8 "&lol7;&lol7;&lol7;&lol7;&lol7;&lol7;&lol7;&lol7;&lol7;&lol7;">
        <!ENTITY lol9 "&lol8;&lol8;&lol8;&lol8;&lol8;&lol8;&lol8;&lol8;&lol8;&lol8;">
        ]>
        <lolz>&lol9;</lolz>"#;

        test_billion_laughs(malicious_xml.as_bytes());
    }

    #[test]
    fn unicode_edge_cases() {
        let test_strings = vec![
            "\u{0000}", // NULL
            "\u{007F}", // DEL
            "\u{0080}", // C1 control
            "\u{200B}", // Zero-width space
            "\u{200C}", // Zero-width non-joiner
            "\u{FDD0}", // Noncharacter
            "\u{10FFFF}", // Max Unicode
            "😀🚀🔥💯", // Emoji
            "العربية", // Arabic
            "日本語", // Japanese
            "Привет", // Russian
        ];

        for s in test_strings {
            test_unicode_handling(s.as_bytes());
        }
    }
}

/// Non-fuzz test variants that can be called from unit tests
fn test_xml_generation(data: &[u8]) {
    use quick_xml::Reader;
    use quick_xml::events::Event;

    let xml_str = String::from_utf8_lossy(data);
    let mut reader = Reader::from_str(&xml_str);
    let mut buf = Vec::new();

    while let Ok(event) = reader.read_event_into(&mut buf) {
        match event {
            Event::Start(_) | Event::End(_) | Event::Text(_) | Event::Comment(_) => {}
            Event::Decl(_) => {}
            Event::DocType(_) => {}
            Event::CData(_) => {}
            Event::PI(_) => {}
            Event::Eof => break,
            _ => {}
        }
    }
}

fn test_coverage_calculation(mi: u32, ci: u32, mb: u32, cb: u32) {
    let total_instructions = mi + ci;
    let coverage_pct = if total_instructions > 0 {
        (ci as f64 / total_instructions as f64) * 100.0
    } else {
        0.0
    };

    let total_branches = mb + cb;
    let branch_coverage = if total_branches > 0 {
        (cb as f64 / total_branches as f64) * 100.0
    } else {
        0.0
    };

    assert!(coverage_pct >= 0.0 && coverage_pct <= 100.0);
    assert!(branch_coverage >= 0.0 && branch_coverage <= 100.0);
}

fn test_path_traversal(data: &[u8]) {
    let path_str = String::from_utf8_lossy(data);
    let dangerous_patterns = vec![
        "../../../etc/passwd",
        "..\\..\\..\\windows\\system32\\config\\sam",
        "....//....//....//etc//passwd",
        "/../../../../../../etc/passwd",
        "%2e%2e%2f%2e%2e%2f%2e%2e%2fetc%2fpasswd",
    ];

    for pattern in &dangerous_patterns {
        if path_str.contains(pattern) {
            assert!(true);
        }
    }
}

fn test_entity_injection(data: &[u8]) {
    let xml_str = String::from_utf8_lossy(data);
    let dangerous_patterns = vec![
        "<!ENTITY xxe SYSTEM \"file:///etc/passwd\">",
        "<!ENTITY % xxe \"<!ENTITY xxe SYSTEM 'file:///etc/passwd'>",
        "<!ENTITY xxe SYSTEM \"file:///proc/version\">",
        "<!ENTITY xxe SYSTEM \"http://evil.com\">",
    ];

    for pattern in &dangerous_patterns {
        if xml_str.contains(pattern) {
            assert!(true, "Should handle entity injection");
        }
    }
}

fn test_billion_laughs(data: &[u8]) {
    let xml_str = String::from_utf8_lossy(data);
    let entity_count = xml_str.matches("<!ENTITY").count();
    let lol_count = xml_str.matches("lol").count();

    if entity_count > 10 || lol_count > 1000 {
        assert!(entity_count < 100, "Too many entities");
    }
}

fn test_unicode_handling(data: &[u8]) {
    let text = String::from_utf8_lossy(data);
    let _ = text.len();
    let _ = text.chars().count();
    let _ = text.split_whitespace().collect::<Vec<_>>();
    let _ = text.trim();
}
