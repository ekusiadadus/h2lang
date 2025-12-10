//! Token definitions for the H2 Language lexer.
//!
//! This module defines the token types produced by the lexer and consumed by the parser.
//! Tokens represent the smallest meaningful units in H2 source code, such as commands,
//! identifiers, numbers, and punctuation.
//!
//! # Token Categories
//!
//! - **Commands**: `s` (straight), `r` (right), `l` (left)
//! - **Identifiers**: Lowercase letters (a-z, excluding s/r/l) for macros and functions
//! - **Parameters**: Uppercase letters (A-Z) for function parameters
//! - **Numbers**: Integer literals for agent IDs and numeric arguments
//! - **Punctuation**: `:`, `(`, `)`, `,`, `+`, `-`
//! - **Whitespace**: Spaces, tabs, newlines
//!
//! # Example
//!
//! The source `0: f(X):XX f(s)` produces tokens:
//!
//! ```text
//! AgentId(0), Colon, Space, Ident('f'), LParen, Param('X'), RParen,
//! Colon, Param('X'), Param('X'), Space, Ident('f'), LParen,
//! Straight, RParen, Eof
//! ```

use std::fmt;

/// Position information for a token in the source code.
///
/// Spans track both byte offsets and line/column positions for error reporting
/// and source mapping.
///
/// # Fields
///
/// - `start`: Byte offset from the beginning of the source (0-based)
/// - `end`: Byte offset of the character after this token
/// - `line`: Line number (1-based)
/// - `column`: Column number (1-based)
///
/// # Example
///
/// ```rust
/// use h2lang::token::Span;
///
/// let span = Span::new(0, 3, 1, 1);
/// assert_eq!(span.start, 0);
/// assert_eq!(span.line, 1);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Span {
    /// Start position (byte offset from beginning of source).
    pub start: usize,
    /// End position (byte offset, exclusive).
    pub end: usize,
    /// Line number (1-based).
    pub line: usize,
    /// Column number (1-based).
    pub column: usize,
}

impl Span {
    /// Creates a new span with the given positions.
    ///
    /// # Arguments
    ///
    /// * `start` - Byte offset of the first character
    /// * `end` - Byte offset after the last character
    /// * `line` - Line number (1-based)
    /// * `column` - Column number (1-based)
    ///
    /// # Example
    ///
    /// ```rust
    /// use h2lang::token::Span;
    ///
    /// let span = Span::new(10, 15, 2, 5);
    /// assert_eq!(span.start, 10);
    /// assert_eq!(span.end, 15);
    /// ```
    #[inline]
    pub fn new(start: usize, end: usize, line: usize, column: usize) -> Self {
        Self {
            start,
            end,
            line,
            column,
        }
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

/// Token type enumeration for H2 Language.
///
/// Each variant represents a distinct lexical element in H2 source code.
/// The lexer produces a stream of these tokens for the parser to consume.
///
/// # Categories
///
/// ## Literals
/// - [`AgentId`](TokenKind::AgentId): Numeric agent identifier (e.g., `0`, `1`, `12`)
/// - [`Number`](TokenKind::Number): Integer literal (e.g., `4`, `-1`)
///
/// ## Identifiers and Parameters
/// - [`Ident`](TokenKind::Ident): Lowercase letter (a-z, excluding s/r/l)
/// - [`Param`](TokenKind::Param): Uppercase letter (A-Z)
///
/// ## Commands
/// - [`Straight`](TokenKind::Straight): `s` command
/// - [`Right`](TokenKind::Right): `r` command
/// - [`Left`](TokenKind::Left): `l` command
///
/// ## Punctuation
/// - [`Colon`](TokenKind::Colon): `:`
/// - [`LParen`](TokenKind::LParen): `(`
/// - [`RParen`](TokenKind::RParen): `)`
/// - [`Comma`](TokenKind::Comma): `,`
/// - [`Plus`](TokenKind::Plus): `+`
/// - [`Minus`](TokenKind::Minus): `-`
///
/// ## Control
/// - [`Space`](TokenKind::Space): Whitespace (space or tab)
/// - [`Newline`](TokenKind::Newline): Line terminator
/// - [`Eof`](TokenKind::Eof): End of input
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    // -------------------------------------------------------------------------
    // Literals
    // -------------------------------------------------------------------------
    /// Agent ID (e.g., `0`, `1`, `12`).
    ///
    /// Agent IDs appear at the start of lines and identify which robot
    /// the following commands apply to.
    AgentId(u32),

    /// Lowercase identifier (`a`-`z`, excluding `s`, `r`, `l`).
    ///
    /// Used for macro names and function names.
    Ident(char),

    /// Uppercase parameter (`A`-`Z`).
    ///
    /// Used in function definitions and calls to represent parameters.
    Param(char),

    // -------------------------------------------------------------------------
    // Basic Commands
    // -------------------------------------------------------------------------
    /// `s` - Straight (move forward one step).
    Straight,

    /// `r` - Right (rotate 90° clockwise).
    Right,

    /// `l` - Left (rotate 90° counter-clockwise).
    Left,

    // -------------------------------------------------------------------------
    // Numeric Literals
    // -------------------------------------------------------------------------
    /// Number literal (e.g., `4`, `12`, `255`).
    ///
    /// Used for numeric function arguments.
    Number(i32),

    // -------------------------------------------------------------------------
    // Punctuation
    // -------------------------------------------------------------------------
    /// `:` - Colon (used in definitions).
    Colon,

    /// `(` - Left parenthesis.
    LParen,

    /// `)` - Right parenthesis.
    RParen,

    /// `,` - Comma (parameter separator).
    Comma,

    /// `+` - Plus (numeric expression).
    Plus,

    /// `-` - Minus (numeric expression).
    Minus,

    // -------------------------------------------------------------------------
    // Control Tokens
    // -------------------------------------------------------------------------
    /// Whitespace (one or more spaces or tabs).
    Space,

    /// Newline (`\n` or `\r\n`).
    Newline,

    /// End of input.
    Eof,
}

impl TokenKind {
    /// Returns a human-readable description of the token kind.
    ///
    /// Used for error messages to describe what token was expected or found.
    ///
    /// # Example
    ///
    /// ```rust
    /// use h2lang::token::TokenKind;
    ///
    /// assert_eq!(TokenKind::Straight.description(), "'s'");
    /// assert_eq!(TokenKind::AgentId(0).description(), "agent ID");
    /// ```
    pub fn description(&self) -> &'static str {
        match self {
            TokenKind::AgentId(_) => "agent ID",
            TokenKind::Ident(_) => "identifier",
            TokenKind::Param(_) => "parameter",
            TokenKind::Straight => "'s'",
            TokenKind::Right => "'r'",
            TokenKind::Left => "'l'",
            TokenKind::Number(_) => "number",
            TokenKind::Colon => "':'",
            TokenKind::LParen => "'('",
            TokenKind::RParen => "')'",
            TokenKind::Comma => "','",
            TokenKind::Plus => "'+'",
            TokenKind::Minus => "'-'",
            TokenKind::Space => "space",
            TokenKind::Newline => "newline",
            TokenKind::Eof => "end of input",
        }
    }

    /// Returns `true` if this token is a command (`s`, `r`, or `l`).
    ///
    /// # Example
    ///
    /// ```rust
    /// use h2lang::token::TokenKind;
    ///
    /// assert!(TokenKind::Straight.is_command());
    /// assert!(!TokenKind::Ident('f').is_command());
    /// ```
    #[inline]
    pub fn is_command(&self) -> bool {
        matches!(
            self,
            TokenKind::Straight | TokenKind::Right | TokenKind::Left
        )
    }

    /// Returns `true` if this token is whitespace (space or newline).
    ///
    /// # Example
    ///
    /// ```rust
    /// use h2lang::token::TokenKind;
    ///
    /// assert!(TokenKind::Space.is_whitespace());
    /// assert!(TokenKind::Newline.is_whitespace());
    /// assert!(!TokenKind::Straight.is_whitespace());
    /// ```
    #[inline]
    pub fn is_whitespace(&self) -> bool {
        matches!(self, TokenKind::Space | TokenKind::Newline)
    }
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenKind::AgentId(n) => write!(f, "{}", n),
            TokenKind::Ident(c) => write!(f, "{}", c),
            TokenKind::Param(c) => write!(f, "{}", c),
            TokenKind::Straight => write!(f, "s"),
            TokenKind::Right => write!(f, "r"),
            TokenKind::Left => write!(f, "l"),
            TokenKind::Number(n) => write!(f, "{}", n),
            TokenKind::Colon => write!(f, ":"),
            TokenKind::LParen => write!(f, "("),
            TokenKind::RParen => write!(f, ")"),
            TokenKind::Comma => write!(f, ","),
            TokenKind::Plus => write!(f, "+"),
            TokenKind::Minus => write!(f, "-"),
            TokenKind::Space => write!(f, " "),
            TokenKind::Newline => write!(f, "\\n"),
            TokenKind::Eof => write!(f, "EOF"),
        }
    }
}

/// A token with its kind and source position.
///
/// Tokens are the output of the lexer and input to the parser. Each token
/// contains its type ([`TokenKind`]) and its position in the source code
/// ([`Span`]).
///
/// # Example
///
/// ```rust
/// use h2lang::token::{Token, TokenKind, Span};
///
/// let token = Token::new(
///     TokenKind::Straight,
///     Span::new(0, 1, 1, 1)
/// );
///
/// assert_eq!(token.kind, TokenKind::Straight);
/// assert_eq!(token.span.line, 1);
/// ```
#[derive(Debug, Clone)]
pub struct Token {
    /// The type of this token.
    pub kind: TokenKind,
    /// Position information in the source code.
    pub span: Span,
}

impl Token {
    /// Creates a new token with the given kind and span.
    ///
    /// # Arguments
    ///
    /// * `kind` - The type of token
    /// * `span` - Position information
    ///
    /// # Example
    ///
    /// ```rust
    /// use h2lang::token::{Token, TokenKind, Span};
    ///
    /// let token = Token::new(TokenKind::Right, Span::new(5, 6, 1, 6));
    /// assert_eq!(token.kind, TokenKind::Right);
    /// ```
    #[inline]
    pub fn new(kind: TokenKind, span: Span) -> Self {
        Self { kind, span }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} at {}", self.kind, self.span)
    }
}

// =============================================================================
// Unit Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_span_creation() {
        let span = Span::new(0, 5, 1, 1);
        assert_eq!(span.start, 0);
        assert_eq!(span.end, 5);
        assert_eq!(span.line, 1);
        assert_eq!(span.column, 1);
    }

    #[test]
    fn test_span_display() {
        let span = Span::new(0, 5, 3, 7);
        assert_eq!(format!("{}", span), "3:7");
    }

    #[test]
    fn test_token_kind_description() {
        assert_eq!(TokenKind::Straight.description(), "'s'");
        assert_eq!(TokenKind::Right.description(), "'r'");
        assert_eq!(TokenKind::Left.description(), "'l'");
        assert_eq!(TokenKind::AgentId(0).description(), "agent ID");
        assert_eq!(TokenKind::Number(42).description(), "number");
    }

    #[test]
    fn test_token_kind_is_command() {
        assert!(TokenKind::Straight.is_command());
        assert!(TokenKind::Right.is_command());
        assert!(TokenKind::Left.is_command());
        assert!(!TokenKind::Ident('f').is_command());
        assert!(!TokenKind::Number(1).is_command());
    }

    #[test]
    fn test_token_kind_is_whitespace() {
        assert!(TokenKind::Space.is_whitespace());
        assert!(TokenKind::Newline.is_whitespace());
        assert!(!TokenKind::Straight.is_whitespace());
        assert!(!TokenKind::Eof.is_whitespace());
    }

    #[test]
    fn test_token_kind_display() {
        assert_eq!(format!("{}", TokenKind::Straight), "s");
        assert_eq!(format!("{}", TokenKind::AgentId(5)), "5");
        assert_eq!(format!("{}", TokenKind::Ident('f')), "f");
    }

    #[test]
    fn test_token_creation() {
        let token = Token::new(TokenKind::Straight, Span::new(0, 1, 1, 1));
        assert_eq!(token.kind, TokenKind::Straight);
        assert_eq!(token.span.start, 0);
    }

    #[test]
    fn test_token_display() {
        let token = Token::new(TokenKind::Right, Span::new(5, 6, 2, 3));
        assert_eq!(format!("{}", token), "r at 2:3");
    }
}
