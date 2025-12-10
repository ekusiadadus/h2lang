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
                    // This is tricky with iterators, so we'll handle '/' as an error later
                    self.current_pos = saved_pos;
                    self.line = saved_line;
                    self.column = saved_column;
                    // Recreate iterator from current position
                    self.chars = self.input[saved_pos..].char_indices().peekable();
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
}
