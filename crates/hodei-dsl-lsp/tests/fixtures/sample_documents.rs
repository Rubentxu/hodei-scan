//! Sample documents for testing
//!
//! Provides various test documents with different scenarios

use hodei_dsl_lsp::domain::models::Document;

/// Sample document with basic rule
pub const BASIC_RULE: &str = r#"
rule password_strength {
  when {
    function validatePassword(pwd: string): boolean {
      return pwd.length >= 8;
    }
  }
  then {
    emit CodeSmell {
      type: "weak_password",
      severity: "Major"
    };
  }
}
"#;

/// Document with incomplete syntax
pub const INCOMPLETE_SYNTAX: &str = r#"
rule incomplete_rule {
  when {
    function test() {
      return true
    }
"#;

/// Document with unknown fact type
pub const UNKNOWN_FACT_TYPE: &str = r#"
rule test_unknown_fact {
  when {
    emit UnknownFact {
      field: "value"
    };
  }
}
"#;

/// Document with multiple fact types
pub const MULTIPLE_FACT_TYPES: &str = r#"
rule multiple_facts {
  when {
    emit Vulnerability {
      severity: "Critical",
      message: "SQL injection detected"
    };
    
    emit CodeSmell {
      type: "unused_variable",
      severity: "Minor"
    };
    
    emit SecurityIssue {
      category: "Authentication",
      severity: "Major"
    };
  }
}
"#;

/// Document with various functions
pub const WITH_FUNCTIONS: &str = r#"
rule function_usage {
  when {
    function checkInput(input: string): boolean {
      if (matches(input, /^[a-z]+$/)) {
        return contains(input, "test");
      }
      return length_gt(input, 10);
    }
  }
}
"#;

/// Empty document
pub const EMPTY_DOCUMENT: &str = "";

/// Document with only comments
pub const ONLY_COMMENTS: &str = r#"
// This is a comment
// Another comment
/* 
   Block comment
*/
"#;

/// Creates a test document with basic rule
pub fn create_basic_document() -> Document {
    Document {
        uri: "file:///test/rule.hodei".to_string(),
        content: BASIC_RULE.to_string(),
        version: 1,
    }
}

/// Creates a document with specific content
pub fn create_document_with_content(content: &str) -> Document {
    Document {
        uri: "file:///test/document.hodei".to_string(),
        content: content.to_string(),
        version: 1,
    }
}

/// Creates a document with specific version
pub fn create_document_with_version(content: &str, version: i32) -> Document {
    Document {
        uri: format!("file:///test/document_v{}.hodei", version),
        content: content.to_string(),
        version,
    }
}
