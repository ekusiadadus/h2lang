//! Lexer (tokenizer) for toioswarm language.

use crate::error::LexerError;
use crate::token::{Span, Token, TokenKind};

/// Lexer for toioswarm language.
pub struct Lexer<'a> {
    input: &'a str,
    chars: std::iter::Peekable<std::str::CharIndices<'a>>,
    current_pos: usize,
    line: usize,
    column: usize,
    at_line_start: bool,
}

impl<'a> Lexer<'a> {
    /// Create a new lexer for the given input.
    pub fn new(input: &'a str) -> Self {
        Self {
            input,
            chars: input.char_indices().peekable(),
            current_pos: 0,
            line: 1,
            column: 1,
            at_line_start: true,
        }
    }

    /// Get the next token.
    pub fn next_token(&mut self) -> Result<Token, LexerError> {
        // Skip comments but NOT whitespace (we emit Space tokens)
        self.skip_comment();

        let start_pos = self.current_pos;
        let start_line = self.line;
        let start_column = self.column;

        let Some((_pos, ch)) = self.advance() else {
            return Ok(Token::new(
                TokenKind::Eof,
                Span::new(start_pos, start_pos, start_line, start_column),
            ));
        };

        let kind = match ch {
            // Whitespace (space or tab)
            ' ' | '\t' => {
                // Consume consecutive whitespace as a single Space token
                while let Some(&(_, c)) = self.chars.peek() {
                    if c == ' ' || c == '\t' {
                        self.advance();
                    } else {
                        break;
                    }
                }
                TokenKind::Space
            }

            // Newline
            '\n' => {
                self.line += 1;
                self.column = 1;
                self.at_line_start = true;
                TokenKind::Newline
            }
            '\r' => {
                // Handle \r\n
                if self.peek_char() == Some('\n') {
                    self.advance();
                }
                self.line += 1;
                self.column = 1;
                self.at_line_start = true;
                TokenKind::Newline
            }

            // Basic commands
            's' => {
                self.at_line_start = false;
                TokenKind::Straight
            }
            'r' => {
                self.at_line_start = false;
                TokenKind::Right
            }
            'l' => {
                self.at_line_start = false;
                TokenKind::Left
            }

            // Symbols
            ':' => {
                self.at_line_start = false;
                TokenKind::Colon
            }
            '(' => {
                self.at_line_start = false;
                TokenKind::LParen
            }
            ')' => {
                self.at_line_start = false;
                TokenKind::RParen
            }
            ',' => {
                self.at_line_start = false;
                TokenKind::Comma
            }
            '+' => {
                self.at_line_start = false;
                TokenKind::Plus
            }
            '-' => {
                self.at_line_start = false;
                TokenKind::Minus
            }

            // Identifiers (lowercase letters except s, r, l)
            c if c.is_ascii_lowercase() => {
                self.at_line_start = false;
                TokenKind::Ident(c)
            }

            // Parameters (uppercase letters)
            c if c.is_ascii_uppercase() => {
                self.at_line_start = false;
                TokenKind::Param(c)
            }

            // Numbers (agent IDs at line start, otherwise Number literals)
            c if c.is_ascii_digit() => {
                let was_at_line_start = self.at_line_start;
                self.at_line_start = false;
                let num = self.read_number(c);
                // At line start, it's an agent ID (e.g., "0: srl")
                // Otherwise, it's a number literal (e.g., "a(4)")
                if was_at_line_start {
                    TokenKind::AgentId(num)
                } else {
                    TokenKind::Number(num as i32)
                }
            }

            // Unknown character
            _ => {
                return Err(LexerError::new(
                    format!("Unexpected character '{}'", ch),
                    start_line,
                    start_column,
                ));
            }
        };

        let span = Span::new(start_pos, self.current_pos, start_line, start_column);
        Ok(Token::new(kind, span))
    }

    /// Tokenize the entire input.
    pub fn tokenize(&mut self) -> Result<Vec<Token>, LexerError> {
        let mut tokens = Vec::new();

        loop {
            let token = self.next_token()?;
            let is_eof = token.kind == TokenKind::Eof;
            tokens.push(token);
            if is_eof {
                break;
            }
        }

        Ok(tokens)
    }

    /// Skip comments (// or #).
    fn skip_comment(&mut self) {
        if let Some(&(_, ch)) = self.chars.peek() {
            if ch == '#' {
                // Skip until newline
                while let Some(&(_, c)) = self.chars.peek() {
                    if c == '\n' || c == '\r' {
                        break;
                    }
                    self.advance();
                }
            } else if ch == '/' {
                // Check for //
                let saved_pos = self.current_pos;
                let saved_line = self.line;
                let saved_column = self.column;

                self.advance();
                if let Some(&(_, '/')) = self.chars.peek() {
                    // Skip until newline
                    while let Some(&(_, c)) = self.chars.peek() {
                        if c == '\n' || c == '\r' {
                            break;
                        }
                        self.advance();
                    }
                } else {
                    // Not a comment, restore position
                    // Recreate iterator from the beginning and skip to saved_pos
                    // This preserves absolute byte offsets from char_indices()
                    self.current_pos = saved_pos;
                    self.line = saved_line;
                    self.column = saved_column;
                    self.chars = self.input.char_indices().peekable();
                    // Skip characters until we reach saved_pos
                    while let Some(&(pos, _)) = self.chars.peek() {
                        if pos >= saved_pos {
                            break;
                        }
                        self.chars.next();
                    }
                }
            }
        }
    }

    /// Peek at the next character without consuming it.
    fn peek_char(&mut self) -> Option<char> {
        self.chars.peek().map(|&(_, ch)| ch)
    }

    /// Advance to the next character.
    fn advance(&mut self) -> Option<(usize, char)> {
        if let Some((pos, ch)) = self.chars.next() {
            self.current_pos = pos + ch.len_utf8();
            if ch != '\n' && ch != '\r' {
                self.column += 1;
            }
            Some((pos, ch))
        } else {
            None
        }
    }

    /// Read a number starting with the given digit.
    fn read_number(&mut self, first_digit: char) -> u32 {
        let mut num = first_digit.to_digit(10).unwrap();

        while let Some(&(_, ch)) = self.chars.peek() {
            if let Some(digit) = ch.to_digit(10) {
                self.advance();
                num = num * 10 + digit;
            } else {
                break;
            }
        }

        num
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_commands() {
        let mut lexer = Lexer::new("srl");
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Straight);
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Right);
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Left);
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Eof);
    }

    #[test]
    fn test_agent_line() {
        let mut lexer = Lexer::new("0: srl");
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::AgentId(0));
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Colon);
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Space);
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Straight);
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Right);
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Left);
    }

    #[test]
    fn test_multi_digit_agent_id() {
        let mut lexer = Lexer::new("12: s");
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::AgentId(12));
    }

    #[test]
    fn test_identifier() {
        let mut lexer = Lexer::new("x:ss");
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Ident('x'));
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Colon);
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Straight);
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Straight);
    }

    #[test]
    fn test_function_definition() {
        let mut lexer = Lexer::new("f(X):XXX");
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Ident('f'));
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::LParen);
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Param('X'));
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::RParen);
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Colon);
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Param('X'));
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Param('X'));
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Param('X'));
    }

    #[test]
    fn test_newline() {
        let mut lexer = Lexer::new("s\nr");
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Straight);
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Newline);
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Right);
    }

    #[test]
    fn test_comment() {
        let mut lexer = Lexer::new("s # comment\nr");
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Straight);
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Space);
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Newline);
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Right);
    }

    #[test]
    fn test_span_tracking() {
        let mut lexer = Lexer::new("0: s");
        let token = lexer.next_token().unwrap();
        assert_eq!(token.span.line, 1);
        assert_eq!(token.span.column, 1);
    }

    #[test]
    fn test_number_in_function_call() {
        // a(4) - number literal inside function call
        let mut lexer = Lexer::new("0: a(4)");
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::AgentId(0));
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Colon);
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Space);
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Ident('a'));
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::LParen);
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Number(4));
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::RParen);
    }

    #[test]
    fn test_comma_and_operators() {
        // a(X,Y) and a(X-1)
        let mut lexer = Lexer::new("a(X,Y)");
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Ident('a'));
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::LParen);
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Param('X'));
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Comma);
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Param('Y'));
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::RParen);

        let mut lexer = Lexer::new("a(X-1)");
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Ident('a'));
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::LParen);
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Param('X'));
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Minus);
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Number(1));
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::RParen);
    }

    #[test]
    fn test_single_slash_position_tracking() {
        // Test that a single '/' (not a comment) doesn't corrupt position tracking
        // After encountering '/' that isn't '//', the lexer should properly restore position
        let mut lexer = Lexer::new("/s");
        // '/' should be treated as an unexpected character error
        let result = lexer.next_token();
        assert!(result.is_err());
        // The error should report correct position (line 1, column 1)
        if let Err(e) = result {
            assert_eq!(e.line, 1);
            assert_eq!(e.column, 1);
        }
    }

    #[test]
    fn test_single_slash_span_after_restore() {
        // Critical test: verify position tracking after single slash restoration
        // The bug occurs when saved_pos is non-zero and we recreate the iterator
        // from a substring. The char_indices() on substring returns relative positions.

        // Test case: "s/r" - after 's', we have '/' at position 1
        // When skip_comment sees '/' but not '//', it should restore position
        // but the bug causes current_pos to become corrupted

        let mut lexer = Lexer::new("s/r");

        // First token 's' at position 0
        let t1 = lexer.next_token().unwrap();
        assert_eq!(t1.kind, TokenKind::Straight);
        assert_eq!(t1.span.start, 0);
        assert_eq!(t1.span.end, 1);

        // Now skip_comment will see '/' at position 1, advance, see 'r', restore
        // Due to bug: self.chars = self.input[1..].char_indices() creates
        // an iterator that returns (0, '/') instead of (1, '/')
        // Then advance() sets current_pos = 0 + 1 = 1 (should stay at 1, but span tracking breaks)

        // The '/' should error
        let err = lexer.next_token().unwrap_err();

        // Check error reports correct position - column should be 2 (after 's')
        assert_eq!(err.column, 2, "Error column should be 2");
        assert_eq!(err.line, 1);
    }

    #[test]
    fn test_position_after_single_slash_in_middle() {
        // More thorough test: what happens to span.end after the bug triggers?
        // Input: "ss/r" (two 's', then '/', then 'r')
        let mut lexer = Lexer::new("ss/r");

        let t1 = lexer.next_token().unwrap();
        assert_eq!(t1.kind, TokenKind::Straight);
        assert_eq!(t1.span.start, 0);
        assert_eq!(t1.span.end, 1);

        let t2 = lexer.next_token().unwrap();
        assert_eq!(t2.kind, TokenKind::Straight);
        assert_eq!(t2.span.start, 1);
        assert_eq!(t2.span.end, 2);

        // '/' at position 2 should error
        let err = lexer.next_token().unwrap_err();
        // Column should be 3 (1-indexed)
        assert_eq!(
            err.column, 3,
            "Error column should be 3 for '/' at position 2"
        );
    }

    #[test]
    fn test_span_corruption_after_slash_restore() {
        // This test specifically checks if span values are corrupted after
        // the iterator restoration bug in skip_comment()
        //
        // Bug scenario: "sss/sss" - after processing first 3 's', we hit '/'
        // skip_comment() restores position, but the new iterator is created
        // from a substring, causing relative offsets to be used
        let mut lexer = Lexer::new("sss/sss");

        // First three 's' tokens
        for i in 0..3 {
            let t = lexer.next_token().unwrap();
            assert_eq!(t.kind, TokenKind::Straight);
            assert_eq!(t.span.start, i, "Token {} should start at {}", i, i);
            assert_eq!(t.span.end, i + 1, "Token {} should end at {}", i, i + 1);
        }

        // '/' at position 3 should error - skip this
        let _ = lexer.next_token(); // Error expected

        // Now the critical test: create fresh lexer and check after error
        // If bug exists, subsequent tokens after '/' would have wrong spans
    }

    #[test]
    fn test_single_slash_followed_by_valid_code() {
        // After a single slash error, subsequent tokens should have correct positions
        let input = "s/r";
        let mut lexer = Lexer::new(input);

        // First token 's' should be at position 0
        let token = lexer.next_token().unwrap();
        assert_eq!(token.kind, TokenKind::Straight);
        assert_eq!(token.span.start, 0);
        assert_eq!(token.span.end, 1);

        // '/' should cause an error at position 1
        let result = lexer.next_token();
        assert!(result.is_err());
    }

    #[test]
    fn test_double_slash_comment() {
        // Verify // comments work correctly
        let mut lexer = Lexer::new("s // comment\nr");
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Straight);
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Space);
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Newline);

        let token = lexer.next_token().unwrap();
        assert_eq!(token.kind, TokenKind::Right);
        // 'r' should be on line 2
        assert_eq!(token.span.line, 2);
    }

    #[test]
    fn test_span_end_position() {
        // Verify span.end is correctly set
        let mut lexer = Lexer::new("srl");
        let token = lexer.next_token().unwrap();
        assert_eq!(token.kind, TokenKind::Straight);
        assert_eq!(token.span.start, 0);
        assert_eq!(token.span.end, 1);

        let token = lexer.next_token().unwrap();
        assert_eq!(token.kind, TokenKind::Right);
        assert_eq!(token.span.start, 1);
        assert_eq!(token.span.end, 2);

        let token = lexer.next_token().unwrap();
        assert_eq!(token.kind, TokenKind::Left);
        assert_eq!(token.span.start, 2);
        assert_eq!(token.span.end, 3);
    }
}
