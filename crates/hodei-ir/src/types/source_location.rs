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

impl fmt::Display for SourceLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}:{}:{}",
            self.file.as_str(),
            self.start_line.get(),
            self.start_column.map(|c| c.get()).unwrap_or(0)
        )
    }
}
