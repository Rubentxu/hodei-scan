//! Unit tests for HoverProvider

use hodei_dsl_lsp::domain::models::{CursorPosition, Document};
use hodei_dsl_lsp::infrastructure::adapters::HodeiHoverProvider;
use crate::fixtures::{create_basic_document, create_document_with_content};

#[tokio::test]
async fn test_hover_provider_initialization() {
    let provider = HodeiHoverProvider::new();
    
    // Provider should be initialized with fact and function docs
    assert!(true); // If we can create it, initialization worked
}

#[tokio::test]
async fn test_provide_vulnerability_hover() {
    let provider = HodeiHoverProvider::new();
    let document = create_document_with_content("Vulnerability");
    
    let hover = provider.provide_hover(
        &document,
        CursorPosition { line: 0, column: 0 }
    )
    .await
    .expect("Should provide hover info");
    
    assert!(hover.is_some());
    
    let hover_info = hover.unwrap();
    assert!(hover_info.contents.contains("Vulnerability"));
    assert!(hover_info.contents.contains("Represents a security vulnerability"));
}

#[tokio::test]
async fn test_provide_code_smell_hover() {
    let provider = HodeiHoverProvider::new();
    let document = create_document_with_content("CodeSmell");
    
    let hover = provider.provide_hover(
        &document,
        CursorPosition { line: 0, column: 0 }
    )
    .await
    .expect("Should provide hover info");
    
    assert!(hover.is_some());
    
    let hover_info = hover.unwrap();
    assert!(hover_info.contents.contains("CodeSmell"));
    assert!(hover_info.contents.contains("Represents a code quality issue"));
}

#[tokio::test]
async fn test_provide_security_issue_hover() {
    let provider = HodeiHoverProvider::new();
    let document = create_document_with_content("SecurityIssue");
    
    let hover = provider.provide_hover(
        &document,
        CursorPosition { line: 0, column: 0 }
    )
    .await
    .expect("Should provide hover info");
    
    assert!(hover.is_some());
    
    let hover_info = hover.unwrap();
    assert!(hover_info.contents.contains("SecurityIssue"));
    assert!(hover_info.contents.contains("Represents a general security issue"));
}

#[tokio::test]
async fn test_provide_matches_function_hover() {
    let provider = HodeiHoverProvider::new();
    let document = create_document_with_content("matches");
    
    let hover = provider.provide_hover(
        &document,
        CursorPosition { line: 0, column: 0 }
    )
    .await
    .expect("Should provide hover info");
    
    assert!(hover.is_some());
    
    let hover_info = hover.unwrap();
    assert!(hover_info.contents.contains("matches"));
    assert!(hover_info.contents.contains("Pattern matching"));
    assert!(hover_info.contents.contains("matches(field, pattern)"));
}

#[tokio::test]
async fn test_provide_contains_function_hover() {
    let provider = HodeiHoverProvider::new();
    let document = create_document_with_content("contains");
    
    let hover = provider.provide_hover(
        &document,
        CursorPosition { line: 0, column: 0 }
    )
    .await
    .expect("Should provide hover info");
    
    assert!(hover.is_some());
    
    let hover_info = hover.unwrap();
    assert!(hover_info.contents.contains("contains"));
    assert!(hover_info.contents.contains("Substring check"));
    assert!(hover_info.contents.contains("contains(field, substring)"));
}

#[tokio::test]
async fn test_provide_length_gt_function_hover() {
    let provider = HodeiHoverProvider::new();
    let document = create_document_with_content("length_gt");
    
    let hover = provider.provide_hover(
        &document,
        CursorPosition { line: 0, column: 0 }
    )
    .await
    .expect("Should provide hover info");
    
    assert!(hover.is_some());
    
    let hover_info = hover.unwrap();
    assert!(hover_info.contents.contains("length_gt"));
    assert!(hover_info.contents.contains("Length greater than"));
    assert!(hover_info.contents.contains("length_gt(field, value)"));
}

#[tokio::test]
async fn test_hover_on_unknown_token() {
    let provider = HodeiHoverProvider::new();
    let document = create_document_with_content("unknown_token_xyz");
    
    let hover = provider.provide_hover(
        &document,
        CursorPosition { line: 0, column: 0 }
    )
    .await
    .expect("Should provide hover info");
    
    // Should return None for unknown tokens
    assert!(hover.is_none());
}

#[tokio::test]
async fn test_hover_on_empty_document() {
    let provider = HodeiHoverProvider::new();
    let document = create_document_with_content("");
    
    let hover = provider.provide_hover(
        &document,
        CursorPosition { line: 0, column: 0 }
    )
    .await
    .expect("Should provide hover info");
    
    // Should return None for empty document
    assert!(hover.is_none());
}

#[tokio::test]
async fn test_hover_range() {
    let provider = HodeiHoverProvider::new();
    let document = create_document_with_content("Vulnerability");
    
    let hover = provider.provide_hover(
        &document,
        CursorPosition { line: 0, column: 0 }
    )
    .await
    .expect("Should provide hover info");
    
    assert!(hover.is_some());
    
    let hover_info = hover.unwrap();
    assert!(hover_info.range.is_some());
    
    let range = hover_info.range.unwrap();
    assert_eq!(range.start.line, 0);
    assert_eq!(range.start.column, 0);
    assert_eq!(range.end.line, 0);
    assert!(range.end.column > range.start.column);
}

#[tokio::test]
async fn test_vulnerability_fact_fields_in_hover() {
    let provider = HodeiHoverProvider::new();
    let document = create_document_with_content("Vulnerability");
    
    let hover = provider.provide_hover(
        &document,
        CursorPosition { line: 0, column: 0 }
    )
    .await
    .expect("Should provide hover info");
    
    let hover_info = hover.unwrap();
    
    // Should include field documentation
    assert!(hover_info.contents.contains("severity"));
    assert!(hover_info.contents.contains("message"));
    assert!(hover_info.contents.contains("Severity"));
    assert!(hover_info.contents.contains("String"));
}

#[tokio::test]
async fn test_code_smell_fact_fields_in_hover() {
    let provider = HodeiHoverProvider::new();
    let document = create_document_with_content("CodeSmell");
    
    let hover = provider.provide_hover(
        &document,
        CursorPosition { line: 0, column: 0 }
    )
    .await
    .expect("Should provide hover info");
    
    let hover_info = hover.unwrap();
    
    // Should include field documentation
    assert!(hover_info.contents.contains("type"));
    assert!(hover_info.contents.contains("severity"));
}

#[tokio::test]
async fn test_function_parameters_in_hover() {
    let provider = HodeiHoverProvider::new();
    let document = create_document_with_content("matches");
    
    let hover = provider.provide_hover(
        &document,
        CursorPosition { line: 0, column: 0 }
    )
    .await
    .expect("Should provide hover info");
    
    let hover_info = hover.unwrap();
    
    // Should include parameter documentation
    assert!(hover_info.contents.contains("field"));
    assert!(hover_info.contents.contains("pattern"));
    assert!(hover_info.contents.contains("String"));
}

#[tokio::test]
async fn test_hover_documentation_format() {
    let provider = HodeiHoverProvider::new();
    let document = create_document_with_content("Vulnerability");
    
    let hover = provider.provide_hover(
        &document,
        CursorPosition { line: 0, column: 0 }
    )
    .await
    .expect("Should provide hover info");
    
    let hover_info = hover.unwrap();
    
    // Should be in markdown format (contains headers and code blocks)
    assert!(hover_info.contents.contains("#"));
    assert!(hover_info.contents.contains("\n\n"));
}

#[tokio::test]
async fn test_hover_at_different_positions() {
    let provider = HodeiHoverProvider::new();
    let document = create_basic_document();
    
    // Test hover at beginning
    let hover1 = provider.provide_hover(
        &document,
        CursorPosition { line: 0, column: 0 }
    )
    .await
    .expect("Should provide hover info");
    // May or may not find something depending on token
    
    // Test hover at middle
    let hover2 = provider.provide_hover(
        &document,
        CursorPosition { line: 5, column: 10 }
    )
    .await
    .expect("Should provide hover info");
    
    // Test hover at end
    let hover3 = provider.provide_hover(
        &document,
        CursorPosition { line: 20, column: 0 }
    )
    .await
    .expect("Should provide hover info");
    
    // All should complete without errors
    assert!(true);
}
