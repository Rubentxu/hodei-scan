//! Source location for facts

use super::*;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SourceLocation {
    pub file: ProjectPath,
    pub start_line: LineNumber,
    pub start_column: Option<ColumnNumber>,
    pub end_line: LineNumber,
    pub end_column: Option<ColumnNumber>,
}

impl std::fmt::Display for SourceLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let col_start = self
            .start_column
            .map(|c| format!(":{}", c.get()))
            .unwrap_or_else(|| "".to_string());
        let col_end = self
            .end_column
            .map(|c| format!(":{}", c.get()))
            .unwrap_or_else(|| "".to_string());

        write!(
            f,
            "{}:{}-{}{}",
            self.file.as_str(),
            self.start_line.get(),
            self.end_line.get(),
            if col_start == col_end {
                col_start
            } else {
                format!("{}{}", col_start, col_end)
            }
        )
    }
}

impl SourceLocation {
    pub fn new(
        file: ProjectPath,
        start_line: LineNumber,
        start_column: Option<ColumnNumber>,
        end_line: LineNumber,
        end_column: Option<ColumnNumber>,
    ) -> Self {
        Self {
            file,
            start_line,
            start_column,
            end_line,
            end_column,
        }
    }

    pub fn span(&self) -> (LineNumber, LineNumber) {
        (self.start_line, self.end_line)
    }
}

impl Default for SourceLocation {
    fn default() -> Self {
        Self {
            file: ProjectPath::new(std::path::PathBuf::from("unknown")),
            start_line: LineNumber::new(1).unwrap(),
            start_column: None,
            end_line: LineNumber::new(1).unwrap(),
            end_column: None,
        }
    }
}
