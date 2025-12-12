//! Error types for H2 Language compiler.

use crate::token::Span;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Lexer error.
#[derive(Debug, Clone)]
pub struct LexerError {
    pub message: String,
    pub line: usize,
    pub column: usize,
}

impl LexerError {
    pub fn new(message: impl Into<String>, line: usize, column: usize) -> Self {
        Self {
            message: message.into(),
            line,
            column,
        }
    }
}

impl fmt::Display for LexerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Lexer error at line {}, column {}: {}",
            self.line, self.column, self.message
        )
    }
}

impl std::error::Error for LexerError {}

/// Parser error.
#[derive(Debug, Clone)]
pub struct ParseError {
    pub message: String,
    pub line: usize,
    pub column: usize,
    pub expected: Option<String>,
    pub found: Option<String>,
}

impl ParseError {
    pub fn new(message: impl Into<String>, span: Span) -> Self {
        Self {
            message: message.into(),
            line: span.line,
            column: span.column,
            expected: None,
            found: None,
        }
    }

    pub fn with_expected(mut self, expected: impl Into<String>) -> Self {
        self.expected = Some(expected.into());
        self
    }

    pub fn with_found(mut self, found: impl Into<String>) -> Self {
        self.found = Some(found.into());
        self
    }

    pub fn unexpected_token(expected: &str, found: &str, span: Span) -> Self {
        Self {
            message: "Unexpected token".to_string(),
            line: span.line,
            column: span.column,
            expected: Some(expected.to_string()),
            found: Some(found.to_string()),
        }
    }

    pub fn unexpected_eof(expected: &str, span: Span) -> Self {
        Self {
            message: "Unexpected end of input".to_string(),
            line: span.line,
            column: span.column,
            expected: Some(expected.to_string()),
            found: Some("end of input".to_string()),
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Parse error at line {}, column {}: {}",
            self.line, self.column, self.message
        )?;
        if let Some(ref expected) = self.expected {
            write!(f, "\n  Expected: {}", expected)?;
        }
        if let Some(ref found) = self.found {
            write!(f, "\n  Found: {}", found)?;
        }
        Ok(())
    }
}

impl std::error::Error for ParseError {}

/// Expansion error (macro/function expansion).
#[derive(Debug, Clone)]
pub struct ExpandError {
    pub message: String,
    pub line: usize,
    pub column: usize,
}

impl ExpandError {
    pub fn new(message: impl Into<String>, span: Span) -> Self {
        Self {
            message: message.into(),
            line: span.line,
            column: span.column,
        }
    }

    pub fn undefined_macro(name: char, span: Span) -> Self {
        Self {
            message: format!("Undefined macro '{}'", name),
            line: span.line,
            column: span.column,
        }
    }

    pub fn undefined_function(name: char, span: Span) -> Self {
        Self {
            message: format!("Undefined function '{}'", name),
            line: span.line,
            column: span.column,
        }
    }

    pub fn max_recursion_depth(span: Span) -> Self {
        Self {
            message: "Maximum recursion depth exceeded".to_string(),
            line: span.line,
            column: span.column,
        }
    }

    /// E004: MAX_STEP limit exceeded
    pub fn max_step_exceeded(limit: usize, span: Span) -> Self {
        Self {
            message: format!("[E004] MAX_STEP limit ({}) exceeded", limit),
            line: span.line,
            column: span.column,
        }
    }
}

impl fmt::Display for ExpandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Expansion error at line {}, column {}: {}",
            self.line, self.column, self.message
        )
    }
}

impl std::error::Error for ExpandError {}

/// Compile error for output (JSON serializable).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompileError {
    pub line: usize,
    pub column: usize,
    pub message: String,
}

impl From<LexerError> for CompileError {
    fn from(e: LexerError) -> Self {
        Self {
            line: e.line,
            column: e.column,
            message: e.message,
        }
    }
}

impl From<ParseError> for CompileError {
    fn from(e: ParseError) -> Self {
        let mut message = e.message;
        if let Some(ref expected) = e.expected {
            message.push_str(&format!(" (expected: {})", expected));
        }
        if let Some(ref found) = e.found {
            message.push_str(&format!(" (found: {})", found));
        }
        Self {
            line: e.line,
            column: e.column,
            message,
        }
    }
}

impl From<ExpandError> for CompileError {
    fn from(e: ExpandError) -> Self {
        Self {
            line: e.line,
            column: e.column,
            message: e.message,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexer_error_display() {
        let err = LexerError::new("Unexpected character", 1, 5);
        assert!(err.to_string().contains("line 1"));
        assert!(err.to_string().contains("column 5"));
    }

    #[test]
    fn test_parse_error_conversion() {
        let span = Span::new(0, 1, 2, 3);
        let err = ParseError::unexpected_token("'s'", "'x'", span);
        let compile_err: CompileError = err.into();
        assert_eq!(compile_err.line, 2);
        assert_eq!(compile_err.column, 3);
    }
}
