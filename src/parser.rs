//! Recursive descent parser for H2 Language.
//!
//! Uses Peekable iterator with buffering for lookahead.

use crate::ast::{
    Agent, Arg, Definition, Directive, DirectiveValue, Expr, FuncDef, LimitConfig, MacroDef,
    OnLimitBehavior, Primitive, Program,
};
use crate::error::ParseError;
use crate::lexer::Lexer;
use crate::token::{Span, Token, TokenKind};
use std::iter::Peekable;
use std::vec::IntoIter;

/// Parser for H2 Language.
///
/// Uses Peekable iterator with a lookahead buffer for multi-token lookahead.
pub struct Parser {
    /// Token iterator
    tokens: Peekable<IntoIter<Token>>,
    /// Lookahead buffer for multi-token lookahead
    buffer: Vec<Token>,
    /// Last consumed token's span (for error reporting)
    last_span: Span,
}

impl Parser {
    /// Create a new parser for the given input.
    pub fn new(input: &str) -> Result<Self, ParseError> {
        let mut lexer = Lexer::new(input);
        let tokens = lexer
            .tokenize()
            .map_err(|e| ParseError::new(e.message, Span::new(0, 0, e.line, e.column)))?;

        let default_span = Span::new(0, 0, 1, 1);

        Ok(Self {
            tokens: tokens.into_iter().peekable(),
            buffer: Vec::new(),
            last_span: default_span,
        })
    }

    /// Fill buffer up to n elements if possible.
    fn fill_buffer(&mut self, n: usize) {
        while self.buffer.len() < n {
            if let Some(token) = self.tokens.next() {
                self.buffer.push(token);
            } else {
                break;
            }
        }
    }

    /// Peek at the current token (0th position).
    fn peek(&mut self) -> Option<&Token> {
        self.fill_buffer(1);
        self.buffer.first()
    }

    /// Peek at the nth token ahead (0-indexed).
    fn peek_nth(&mut self, n: usize) -> Option<&Token> {
        self.fill_buffer(n + 1);
        self.buffer.get(n)
    }

    /// Get current token kind, or EOF if none.
    fn current_kind(&mut self) -> TokenKind {
        self.peek()
            .map(|t| t.kind.clone())
            .unwrap_or(TokenKind::Eof)
    }

    /// Get current token span.
    fn current_span(&mut self) -> Span {
        self.peek().map(|t| t.span).unwrap_or(self.last_span)
    }

    /// Advance to next token, returning the consumed token.
    fn advance(&mut self) -> Option<Token> {
        // First try to take from buffer
        if !self.buffer.is_empty() {
            let token = self.buffer.remove(0);
            self.last_span = token.span;
            Some(token)
        } else {
            // Otherwise take from iterator
            let token = self.tokens.next();
            if let Some(ref t) = token {
                self.last_span = t.span;
            }
            token
        }
    }

    /// Check if current token matches the given kind.
    fn check(&mut self, kind: &TokenKind) -> bool {
        std::mem::discriminant(&self.current_kind()) == std::mem::discriminant(kind)
    }

    /// Check if we're at end of line or end of input.
    fn is_at_end_of_line(&mut self) -> bool {
        matches!(self.current_kind(), TokenKind::Newline | TokenKind::Eof)
    }

    /// Skip Space tokens.
    fn skip_space(&mut self) {
        while self.check(&TokenKind::Space) {
            self.advance();
        }
    }

    /// Expect a specific token kind.
    fn expect(&mut self, kind: &TokenKind) -> Result<Span, ParseError> {
        if self.check(kind) {
            let span = self.current_span();
            self.advance();
            Ok(span)
        } else {
            Err(ParseError::unexpected_token(
                kind.description(),
                self.current_kind().description(),
                self.current_span(),
            ))
        }
    }

    /// Parse the entire program.
    ///
    /// Structure: `directives* (agent_block | single_agent_block)`
    ///
    /// Supports two modes:
    /// 1. **With agent prefix**: `0: srl` - traditional multi-agent syntax
    /// 2. **Without agent prefix**: `srl` - single agent mode (defaults to agent 0)
    pub fn parse_program(&mut self) -> Result<Program, ParseError> {
        // Skip leading newlines
        while self.check(&TokenKind::Newline) {
            self.advance();
        }

        // Parse directives first
        let directives = self.parse_directives()?;
        let limits = Self::build_limit_config(&directives)?;

        // Skip newlines after directives
        while self.check(&TokenKind::Newline) {
            self.advance();
        }

        // Check if we're at EOF (empty program or directives only)
        if self.check(&TokenKind::Eof) {
            return Ok(Program {
                directives,
                limits,
                agents: Vec::new(),
            });
        }

        // Parse agents
        let mut agents = Vec::new();

        // Determine mode: check if first token is AgentId
        let has_agent_prefix = matches!(self.current_kind(), TokenKind::AgentId(_));

        if has_agent_prefix {
            // Traditional multi-agent mode
            while !self.check(&TokenKind::Eof) {
                let agent = self.parse_agent_line_with_prefix()?;
                agents.push(agent);

                // Skip newlines between agents
                while self.check(&TokenKind::Newline) {
                    self.advance();
                }
            }
        } else {
            // Single agent mode (no prefix)
            // All lines are treated as the same agent (agent 0)
            let agent = self.parse_agent_line_without_prefix_multiline()?;
            agents.push(agent);
        }

        Ok(Program {
            directives,
            limits,
            agents,
        })
    }

    /// Parse directives at the beginning of the program.
    ///
    /// Directives are lines like `MAX_STEP=1000` or `ON_LIMIT=ERROR`
    fn parse_directives(&mut self) -> Result<Vec<Directive>, ParseError> {
        let mut directives = Vec::new();

        loop {
            // Skip newlines
            while self.check(&TokenKind::Newline) {
                self.advance();
            }

            // Check if this is a directive
            if let TokenKind::Directive(name) = self.current_kind() {
                let start_span = self.current_span();
                self.advance();

                // Expect '='
                self.expect(&TokenKind::Equals)?;

                // Parse directive value (number or directive value like ERROR/TRUNCATE)
                let value = self.parse_directive_value()?;

                let end_span = self.current_span();
                let span = Span::new(
                    start_span.start,
                    end_span.end,
                    start_span.line,
                    start_span.column,
                );

                directives.push(Directive { name, value, span });

                // Expect newline or EOF after directive
                if !self.check(&TokenKind::Newline) && !self.check(&TokenKind::Eof) {
                    return Err(ParseError::unexpected_token(
                        "newline or end of input",
                        self.current_kind().description(),
                        self.current_span(),
                    ));
                }
            } else {
                // Not a directive, stop parsing directives
                break;
            }
        }

        Ok(directives)
    }

    /// Parse directive value (number or ERROR/TRUNCATE)
    fn parse_directive_value(&mut self) -> Result<DirectiveValue, ParseError> {
        match self.current_kind() {
            TokenKind::Number(n) => {
                self.advance();
                Ok(DirectiveValue::Number(n as i64))
            }
            TokenKind::DirectiveValue(s) => {
                self.advance();
                Ok(DirectiveValue::String(s))
            }
            _ => Err(ParseError::unexpected_token(
                "number or ERROR/TRUNCATE",
                self.current_kind().description(),
                self.current_span(),
            )),
        }
    }

    /// Build LimitConfig from parsed directives.
    fn build_limit_config(directives: &[Directive]) -> Result<LimitConfig, ParseError> {
        let mut config = LimitConfig::default();

        for directive in directives {
            match directive.name.as_str() {
                "MAX_STEP" => {
                    if let DirectiveValue::Number(n) = &directive.value {
                        if *n < 1 || *n > 10_000_000 {
                            return Err(ParseError::new(
                                format!(
                                    "MAX_STEP value {} out of range (1..10000000) (E009)",
                                    n
                                ),
                                directive.span,
                            ));
                        }
                        config.max_step = *n as usize;
                    } else {
                        return Err(ParseError::new(
                            "MAX_STEP requires a numeric value (E009)",
                            directive.span,
                        ));
                    }
                }
                "MAX_DEPTH" => {
                    if let DirectiveValue::Number(n) = &directive.value {
                        if *n < 1 || *n > 10_000 {
                            return Err(ParseError::new(
                                format!("MAX_DEPTH value {} out of range (1..10000) (E009)", n),
                                directive.span,
                            ));
                        }
                        config.max_depth = *n as usize;
                    } else {
                        return Err(ParseError::new(
                            "MAX_DEPTH requires a numeric value (E009)",
                            directive.span,
                        ));
                    }
                }
                "MAX_MEMORY" => {
                    if let DirectiveValue::Number(n) = &directive.value {
                        if *n < 1 || *n > 10_000_000 {
                            return Err(ParseError::new(
                                format!(
                                    "MAX_MEMORY value {} out of range (1..10000000) (E009)",
                                    n
                                ),
                                directive.span,
                            ));
                        }
                        config.max_memory = *n as usize;
                    } else {
                        return Err(ParseError::new(
                            "MAX_MEMORY requires a numeric value (E009)",
                            directive.span,
                        ));
                    }
                }
                "ON_LIMIT" => {
                    if let DirectiveValue::String(s) = &directive.value {
                        match s.as_str() {
                            "ERROR" => config.on_limit = OnLimitBehavior::Error,
                            "TRUNCATE" => config.on_limit = OnLimitBehavior::Truncate,
                            _ => {
                                return Err(ParseError::new(
                                    format!(
                                        "ON_LIMIT value '{}' invalid, expected ERROR or TRUNCATE (E009)",
                                        s
                                    ),
                                    directive.span,
                                ));
                            }
                        }
                    } else {
                        return Err(ParseError::new(
                            "ON_LIMIT requires ERROR or TRUNCATE (E009)",
                            directive.span,
                        ));
                    }
                }
                _ => {
                    return Err(ParseError::new(
                        format!("Unknown directive '{}' (E009)", directive.name),
                        directive.span,
                    ));
                }
            }
        }

        Ok(config)
    }

    /// Parse agent with prefix: `agent_id ':' statement_list`
    ///
    /// Parses until EOF or the next agent ID is encountered.
    /// Supports multi-line code for a single agent.
    fn parse_agent_line_with_prefix(&mut self) -> Result<Agent, ParseError> {
        let start_span = self.current_span();

        // Parse agent ID
        let id = match self.current_kind() {
            TokenKind::AgentId(n) => n,
            _ => {
                return Err(ParseError::unexpected_token(
                    "agent ID",
                    self.current_kind().description(),
                    self.current_span(),
                ));
            }
        };
        self.advance();

        // Expect ':'
        self.expect(&TokenKind::Colon)?;

        // Skip space after colon
        self.skip_space();

        // Parse statement list across multiple lines until next AgentId or EOF
        let (definitions, expression) = self.parse_statement_list_multiline()?;

        let end_span = self.current_span();
        let span = Span::new(
            start_span.start,
            end_span.end,
            start_span.line,
            start_span.column,
        );

        Ok(Agent {
            id,
            definitions,
            expression,
            span,
        })
    }

    /// Parse multiple lines without prefix as a single agent (agent 0).
    ///
    /// This is used for single-agent programs where all lines belong to agent 0.
    /// Continues parsing until EOF, treating newlines as whitespace separators.
    fn parse_agent_line_without_prefix_multiline(&mut self) -> Result<Agent, ParseError> {
        let start_span = self.current_span();

        // Default to agent 0
        let id = 0;

        // Parse statement list across multiple lines
        let (definitions, expression) = self.parse_statement_list_multiline()?;

        let end_span = self.current_span();
        let span = Span::new(
            start_span.start,
            end_span.end,
            start_span.line,
            start_span.column,
        );

        Ok(Agent {
            id,
            definitions,
            expression,
            span,
        })
    }

    /// Parse statement list across multiple lines until EOF or next AgentId.
    ///
    /// This is used for single-agent programs or multi-line agent definitions.
    fn parse_statement_list_multiline(&mut self) -> Result<(Vec<Definition>, Expr), ParseError> {
        let mut definitions = Vec::new();
        let mut expression_terms = Vec::new();

        loop {
            // Skip spaces and newlines
            while self.check(&TokenKind::Space) || self.check(&TokenKind::Newline) {
                self.advance();
            }

            // Check for end conditions
            if self.check(&TokenKind::Eof) {
                break;
            }

            // Check if we hit a new agent ID (for multi-agent mode)
            if matches!(self.current_kind(), TokenKind::AgentId(_)) {
                break;
            }

            // Try to parse a definition (lookahead required)
            if let Some(def) = self.try_parse_definition()? {
                definitions.push(def);
            } else {
                // Parse expression term
                let term = self.parse_term()?;
                expression_terms.push(term);
            }
        }

        let expression = if expression_terms.is_empty() {
            Expr::Sequence(vec![])
        } else if expression_terms.len() == 1 {
            expression_terms.pop().unwrap()
        } else {
            Expr::Sequence(expression_terms)
        };

        Ok((definitions, expression))
    }

    /// Try to parse a definition (macro or function).
    /// Returns None if not a definition, without consuming tokens.
    fn try_parse_definition(&mut self) -> Result<Option<Definition>, ParseError> {
        // Definition starts with identifier
        let name = match self.current_kind() {
            TokenKind::Ident(c) => c,
            _ => return Ok(None),
        };

        // Check what follows the identifier using lookahead
        let next_kind = self.peek_nth(1).map(|t| t.kind.clone());

        match next_kind {
            Some(TokenKind::Colon) => {
                // Macro definition: `name ':' expression`
                let def = self.parse_macro_def(name)?;
                Ok(Some(Definition::Macro(def)))
            }
            Some(TokenKind::LParen) => {
                // Could be function definition or function call
                // Function definition has: name '(' PARAM ')' ':'
                if self.is_function_definition() {
                    let def = self.parse_function_def(name)?;
                    Ok(Some(Definition::Function(def)))
                } else {
                    Ok(None)
                }
            }
            _ => Ok(None),
        }
    }

    /// Check if the current position is a function definition.
    /// A function definition has the pattern: ident '(' PARAM (',' PARAM)* ')' ':'
    /// Where PARAM is an uppercase letter (TokenKind::Param).
    fn is_function_definition(&mut self) -> bool {
        // Pattern: ident '(' PARAM (',' PARAM)* ')' ':'
        // peek(0): ident (current)
        // peek(1): '('
        // peek(2+): PARAM, optionally followed by ',' PARAM
        // then: ')' ':'

        let t1 = self.peek_nth(1);
        if !matches!(t1.map(|t| &t.kind), Some(TokenKind::LParen)) {
            return false;
        }

        // Scan forward to find ')' and check if ':' follows
        let mut pos = 2;
        loop {
            let token = self.peek_nth(pos);
            match token.map(|t| &t.kind) {
                Some(TokenKind::Param(_)) => {
                    pos += 1;
                }
                Some(TokenKind::Comma) => {
                    pos += 1;
                }
                Some(TokenKind::RParen) => {
                    // Found ')' - check if ':' follows
                    let next = self.peek_nth(pos + 1);
                    return matches!(next.map(|t| &t.kind), Some(TokenKind::Colon));
                }
                _ => {
                    // Not a valid function definition pattern
                    return false;
                }
            }

            // Safety limit to prevent infinite loop
            if pos > 20 {
                return false;
            }
        }
    }

    /// Parse macro definition: `name ':' expression`
    fn parse_macro_def(&mut self, name: char) -> Result<MacroDef, ParseError> {
        let start_span = self.current_span();

        // Advance past name
        self.advance();

        // Expect ':'
        self.expect(&TokenKind::Colon)?;

        // Note: Do NOT skip space here - body ends at space
        // Parse body expression (until space, next definition, or end of line)
        let body = self.parse_expression_until_definition()?;

        let span = Span::new(
            start_span.start,
            body.span().end,
            start_span.line,
            start_span.column,
        );

        Ok(MacroDef { name, body, span })
    }

    /// Parse function definition: `name '(' param (',' param)* ')' ':' expression`
    fn parse_function_def(&mut self, name: char) -> Result<FuncDef, ParseError> {
        let start_span = self.current_span();

        // Advance past name
        self.advance();

        // Expect '('
        self.expect(&TokenKind::LParen)?;

        // Parse parameters (comma-separated)
        let mut params = Vec::new();
        loop {
            // Expect parameter
            let param = match self.current_kind() {
                TokenKind::Param(p) => p,
                _ => {
                    return Err(ParseError::unexpected_token(
                        "parameter (uppercase letter)",
                        self.current_kind().description(),
                        self.current_span(),
                    ));
                }
            };
            self.advance();
            params.push(param);

            // Check for comma (more params) or RParen (end of params)
            if self.check(&TokenKind::Comma) {
                self.advance();
            } else {
                break;
            }
        }

        // Expect ')'
        self.expect(&TokenKind::RParen)?;

        // Expect ':'
        self.expect(&TokenKind::Colon)?;

        // Note: Do NOT skip space here - body ends at space
        // Parse body expression (until space, next definition, or end of line)
        let body = self.parse_expression_until_definition()?;

        let span = Span::new(
            start_span.start,
            body.span().end,
            start_span.line,
            start_span.column,
        );

        Ok(FuncDef {
            name,
            params,
            body,
            span,
        })
    }

    /// Parse expression until we hit Space, definition start, or end of line.
    /// This is used for parsing definition bodies.
    fn parse_expression_until_definition(&mut self) -> Result<Expr, ParseError> {
        let mut terms = Vec::new();

        while !self.is_at_end_of_line() {
            // Space terminates definition body
            if self.check(&TokenKind::Space) {
                break;
            }

            // Check if this looks like a definition start
            if let TokenKind::Ident(_) = self.current_kind() {
                // Check for macro definition: ident ':'
                if matches!(self.peek_nth(1).map(|t| &t.kind), Some(TokenKind::Colon)) {
                    break;
                }
                // Check for function definition: ident '(' PARAM ')' ':'
                if self.is_function_definition() {
                    break;
                }
            }

            let term = self.parse_term()?;
            terms.push(term);
        }

        Ok(if terms.is_empty() {
            Expr::Sequence(vec![])
        } else if terms.len() == 1 {
            terms.pop().unwrap()
        } else {
            Expr::Sequence(terms)
        })
    }

    /// Parse a single term.
    fn parse_term(&mut self) -> Result<Expr, ParseError> {
        let span = self.current_span();

        match self.current_kind() {
            TokenKind::Straight => {
                self.advance();
                Ok(Expr::Primitive(Primitive::Straight, span))
            }
            TokenKind::Right => {
                self.advance();
                Ok(Expr::Primitive(Primitive::Right, span))
            }
            TokenKind::Left => {
                self.advance();
                Ok(Expr::Primitive(Primitive::Left, span))
            }
            TokenKind::Ident(name) => {
                // Check if this is a function call
                if matches!(self.peek_nth(1).map(|t| &t.kind), Some(TokenKind::LParen)) {
                    self.parse_function_call(name)
                } else {
                    // Just an identifier (macro reference)
                    self.advance();
                    Ok(Expr::Ident(name, span))
                }
            }
            TokenKind::Param(name) => {
                self.advance();
                Ok(Expr::Param(name, span))
            }
            TokenKind::LParen => {
                // Grouped expression
                self.advance();
                let expr = self.parse_expression_until(&TokenKind::RParen)?;
                self.expect(&TokenKind::RParen)?;
                Ok(expr)
            }
            _ => Err(ParseError::unexpected_token(
                "'s', 'r', 'l', identifier, or '('",
                self.current_kind().description(),
                span,
            )),
        }
    }

    /// Parse function call with typed arguments: `name '(' args ')'`
    /// Arguments can be:
    /// - Command expression (e.g., srl, XXX)
    /// - Number literal (e.g., 4)
    /// - Numeric expression (e.g., X-1, X+2)
    fn parse_function_call(&mut self, name: char) -> Result<Expr, ParseError> {
        let start_span = self.current_span();

        // Advance past name
        self.advance();

        // Expect '('
        self.expect(&TokenKind::LParen)?;

        // Parse arguments
        let args = self.parse_function_args()?;

        // Expect ')'
        let end_span = self.current_span();
        self.expect(&TokenKind::RParen)?;

        let span = Span::new(
            start_span.start,
            end_span.end,
            start_span.line,
            start_span.column,
        );

        // Use FuncCallArgs for HOJ-compatible calls
        Ok(Expr::FuncCallArgs { name, args, span })
    }

    /// Parse function arguments separated by commas.
    fn parse_function_args(&mut self) -> Result<Vec<Arg>, ParseError> {
        let mut args = Vec::new();

        if self.check(&TokenKind::RParen) {
            return Ok(args); // Empty args
        }

        loop {
            let arg = self.parse_function_arg()?;
            args.push(arg);

            if self.check(&TokenKind::Comma) {
                self.advance();
            } else {
                break;
            }
        }

        Ok(args)
    }

    /// Parse a single function argument.
    fn parse_function_arg(&mut self) -> Result<Arg, ParseError> {
        let span = self.current_span();

        match self.current_kind() {
            TokenKind::Minus => {
                // Negative number: -N
                self.advance();
                if let TokenKind::Number(n) = self.current_kind() {
                    let end_span = self.current_span();
                    self.advance();
                    let neg_span = Span::new(span.start, end_span.end, span.line, span.column);
                    Ok(Arg::Number(-n, neg_span))
                } else {
                    Err(ParseError::unexpected_token(
                        "number after '-'",
                        self.current_kind().description(),
                        self.current_span(),
                    ))
                }
            }
            TokenKind::Number(n) => {
                self.advance();
                Ok(Arg::Number(n, span))
            }
            TokenKind::Param(p) => {
                // Check if this is a numeric expression (e.g., X-1, X+2)
                let next = self.peek_nth(1).map(|t| t.kind.clone());
                match next {
                    Some(TokenKind::Plus) | Some(TokenKind::Minus) => {
                        self.parse_numeric_expr(p, span)
                    }
                    _ => {
                        // Just a parameter reference as command arg
                        self.advance();
                        Ok(Arg::Command(Expr::Param(p, span)))
                    }
                }
            }
            _ => {
                // Parse as command expression
                let expr = self.parse_arg_expression()?;
                Ok(Arg::Command(expr))
            }
        }
    }

    /// Parse numeric expression: `PARAM ('+' | '-') NUMBER`
    fn parse_numeric_expr(&mut self, param: char, start_span: Span) -> Result<Arg, ParseError> {
        self.advance(); // consume param

        let is_plus = self.check(&TokenKind::Plus);
        self.advance(); // consume + or -

        let num = match self.current_kind() {
            TokenKind::Number(n) => n,
            _ => {
                return Err(ParseError::unexpected_token(
                    "number",
                    self.current_kind().description(),
                    self.current_span(),
                ));
            }
        };
        let end_span = self.current_span();
        self.advance();

        let offset = if is_plus { num } else { -num };
        let span = Span::new(
            start_span.start,
            end_span.end,
            start_span.line,
            start_span.column,
        );

        Ok(Arg::NumExpr {
            param,
            offset,
            span,
        })
    }

    /// Parse argument expression (until comma or rparen).
    fn parse_arg_expression(&mut self) -> Result<Expr, ParseError> {
        let mut terms = Vec::new();

        while !self.check(&TokenKind::RParen)
            && !self.check(&TokenKind::Comma)
            && !self.is_at_end_of_line()
        {
            let term = self.parse_term()?;
            terms.push(term);
        }

        Ok(if terms.is_empty() {
            Expr::Sequence(vec![])
        } else if terms.len() == 1 {
            terms.pop().unwrap()
        } else {
            Expr::Sequence(terms)
        })
    }

    /// Parse expression until a specific token.
    fn parse_expression_until(&mut self, end_token: &TokenKind) -> Result<Expr, ParseError> {
        let mut terms = Vec::new();

        while !self.check(end_token) && !self.is_at_end_of_line() {
            // Skip spaces within expressions (e.g., function arguments)
            self.skip_space();
            if self.check(end_token) || self.is_at_end_of_line() {
                break;
            }
            let term = self.parse_term()?;
            terms.push(term);
        }

        Ok(if terms.is_empty() {
            Expr::Sequence(vec![])
        } else if terms.len() == 1 {
            terms.pop().unwrap()
        } else {
            Expr::Sequence(terms)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_program() {
        let mut parser = Parser::new("0: srl").unwrap();
        let program = parser.parse_program().unwrap();

        assert_eq!(program.agents.len(), 1);
        assert_eq!(program.agents[0].id, 0);
    }

    #[test]
    fn test_multiple_agents() {
        let mut parser = Parser::new("0: srl\n1: lrs").unwrap();
        let program = parser.parse_program().unwrap();

        assert_eq!(program.agents.len(), 2);
        assert_eq!(program.agents[0].id, 0);
        assert_eq!(program.agents[1].id, 1);
    }

    #[test]
    fn test_macro_definition() {
        let mut parser = Parser::new("0: x:ss xrx").unwrap();
        let program = parser.parse_program().unwrap();

        assert_eq!(program.agents[0].definitions.len(), 1);
        assert_eq!(program.agents[0].definitions[0].name(), 'x');
    }

    #[test]
    fn test_function_definition() {
        let mut parser = Parser::new("0: f(X):XXX f(s)").unwrap();
        let program = parser.parse_program().unwrap();

        assert_eq!(program.agents[0].definitions.len(), 1);
        if let Definition::Function(f) = &program.agents[0].definitions[0] {
            assert_eq!(f.name, 'f');
            assert_eq!(f.params, vec!['X']);
        } else {
            panic!("Expected function definition");
        }
    }

    #[test]
    fn test_nested_function_call() {
        let mut parser = Parser::new("0: f(X):XXX f(f(s))").unwrap();
        let program = parser.parse_program().unwrap();

        // Should parse successfully
        assert_eq!(program.agents.len(), 1);
    }

    #[test]
    fn test_empty_expression() {
        let mut parser = Parser::new("0: x:ss").unwrap();
        let program = parser.parse_program().unwrap();

        // Macro definition only, empty expression
        assert_eq!(program.agents[0].definitions.len(), 1);
        assert!(program.agents[0].expression.is_empty());
    }

    #[test]
    fn test_function_call_not_definition() {
        // f(s) should be parsed as a function call, not a definition
        // because 's' is lowercase (not a Param)
        let mut parser = Parser::new("0: f(X):X f(s)").unwrap();
        let program = parser.parse_program().unwrap();

        // One function definition, one expression (function call)
        assert_eq!(program.agents[0].definitions.len(), 1);
        assert!(!program.agents[0].expression.is_empty());
    }

    #[test]
    fn test_multiple_definitions() {
        let mut parser = Parser::new("0: x:ss f(X):XX xf(s)").unwrap();
        let program = parser.parse_program().unwrap();

        // Two definitions: macro x and function f
        assert_eq!(program.agents[0].definitions.len(), 2);
    }

    #[test]
    fn test_numeric_argument() {
        // HOJ: a(4) - numeric argument
        let mut parser = Parser::new("0: a(X):sa(X-1) a(4)").unwrap();
        let program = parser.parse_program().unwrap();

        assert_eq!(program.agents[0].definitions.len(), 1);
        // Expression should be a function call with numeric arg
        if let Expr::FuncCallArgs { name, args, .. } = &program.agents[0].expression {
            assert_eq!(*name, 'a');
            assert_eq!(args.len(), 1);
            assert!(matches!(args[0], Arg::Number(4, _)));
        } else {
            panic!("Expected FuncCallArgs");
        }
    }

    #[test]
    fn test_numeric_expression_arg() {
        // HOJ: a(X-1) - numeric expression argument
        let mut parser = Parser::new("0: a(X-1)").unwrap();
        let program = parser.parse_program().unwrap();

        if let Expr::FuncCallArgs { args, .. } = &program.agents[0].expression {
            assert_eq!(args.len(), 1);
            if let Arg::NumExpr { param, offset, .. } = &args[0] {
                assert_eq!(*param, 'X');
                assert_eq!(*offset, -1);
            } else {
                panic!("Expected NumExpr");
            }
        } else {
            panic!("Expected FuncCallArgs");
        }
    }

    #[test]
    fn test_numeric_expression_plus() {
        // HOJ: a(X+2)
        let mut parser = Parser::new("0: a(X+2)").unwrap();
        let program = parser.parse_program().unwrap();

        if let Expr::FuncCallArgs { args, .. } = &program.agents[0].expression {
            if let Arg::NumExpr { param, offset, .. } = &args[0] {
                assert_eq!(*param, 'X');
                assert_eq!(*offset, 2);
            } else {
                panic!("Expected NumExpr");
            }
        } else {
            panic!("Expected FuncCallArgs");
        }
    }

    #[test]
    fn test_multiple_arguments() {
        // HOJ: a(4,s) - multiple arguments
        let mut parser = Parser::new("0: a(4,s)").unwrap();
        let program = parser.parse_program().unwrap();

        if let Expr::FuncCallArgs { args, .. } = &program.agents[0].expression {
            assert_eq!(args.len(), 2);
            assert!(matches!(args[0], Arg::Number(4, _)));
            if let Arg::Command(Expr::Primitive(Primitive::Straight, _)) = &args[1] {
                // OK
            } else {
                panic!("Expected Command(Straight)");
            }
        } else {
            panic!("Expected FuncCallArgs");
        }
    }

    #[test]
    fn test_complex_hoj_example() {
        // HOJ: a(X,Y):Ya(X-1,Y) a(4,s)
        let mut parser = Parser::new("0: a(X,Y):Ya(X-1,Y) a(4,s)").unwrap();
        let program = parser.parse_program().unwrap();

        // Should have one function definition
        assert_eq!(program.agents[0].definitions.len(), 1);
        // Expression should be function call with two args
        if let Expr::FuncCallArgs { name, args, .. } = &program.agents[0].expression {
            assert_eq!(*name, 'a');
            assert_eq!(args.len(), 2);
        } else {
            panic!("Expected FuncCallArgs");
        }
    }

    #[test]
    fn test_command_expression_arg() {
        // HOJ: f(srl) - command expression argument
        let mut parser = Parser::new("0: f(srl)").unwrap();
        let program = parser.parse_program().unwrap();

        if let Expr::FuncCallArgs { args, .. } = &program.agents[0].expression {
            assert_eq!(args.len(), 1);
            if let Arg::Command(Expr::Sequence(exprs)) = &args[0] {
                assert_eq!(exprs.len(), 3);
            } else {
                panic!("Expected Command(Sequence)");
            }
        } else {
            panic!("Expected FuncCallArgs");
        }
    }

    // =============================================================================
    // Agent Prefix Optional Tests (Single Agent)
    // =============================================================================

    #[test]
    fn test_no_agent_prefix_simple() {
        // Single agent without "0:" prefix
        let mut parser = Parser::new("srl").unwrap();
        let program = parser.parse_program().unwrap();

        assert_eq!(program.agents.len(), 1);
        assert_eq!(program.agents[0].id, 0);
    }

    #[test]
    fn test_no_agent_prefix_with_macro() {
        // Macro definition without agent prefix
        let mut parser = Parser::new("x:ss xrx").unwrap();
        let program = parser.parse_program().unwrap();

        assert_eq!(program.agents.len(), 1);
        assert_eq!(program.agents[0].id, 0);
        assert_eq!(program.agents[0].definitions.len(), 1);
    }

    #[test]
    fn test_no_agent_prefix_with_function() {
        // Function definition without agent prefix
        let mut parser = Parser::new("f(X):XXXX f(sssr)").unwrap();
        let program = parser.parse_program().unwrap();

        assert_eq!(program.agents.len(), 1);
        assert_eq!(program.agents[0].id, 0);
        assert_eq!(program.agents[0].definitions.len(), 1);
    }

    #[test]
    fn test_no_agent_prefix_multiline() {
        // Multiple lines without agent prefix are allowed
        // All lines are treated as the same agent (agent 0)
        let mut parser = Parser::new("srl\nlrs").unwrap();
        let program = parser.parse_program().unwrap();

        assert_eq!(program.agents.len(), 1);
        assert_eq!(program.agents[0].id, 0);
        // srl + lrs = 6 commands total
    }

    #[test]
    fn test_no_agent_prefix_macro_multiline() {
        // Macro definition on one line, usage on next line
        let mut parser = Parser::new("a:ssrs\naaaaaaaaaa").unwrap();
        let program = parser.parse_program().unwrap();

        assert_eq!(program.agents.len(), 1);
        assert_eq!(program.agents[0].id, 0);
        assert_eq!(program.agents[0].definitions.len(), 1);
    }

    #[test]
    fn test_mixed_prefix_and_no_prefix() {
        // If first line has prefix, it's multi-agent mode
        let mut parser = Parser::new("0: srl\n1: lrs").unwrap();
        let program = parser.parse_program().unwrap();
        assert_eq!(program.agents.len(), 2);
    }

    #[test]
    fn test_agent_with_multiline_code() {
        // Agent 0 has code spanning multiple lines
        // Agent 1 has single line
        let mut parser = Parser::new("0: a:ssrs\naaaa\n1: srl").unwrap();
        let program = parser.parse_program().unwrap();

        assert_eq!(program.agents.len(), 2);
        assert_eq!(program.agents[0].id, 0);
        assert_eq!(program.agents[0].definitions.len(), 1); // macro a
        assert_eq!(program.agents[1].id, 1);
    }

    #[test]
    fn test_agent_with_multiline_code_trailing() {
        // Agent 0 code, then Agent 1 code with trailing lines
        let mut parser = Parser::new("0: srl\n1: a:ss\naaaa").unwrap();
        let program = parser.parse_program().unwrap();

        assert_eq!(program.agents.len(), 2);
        assert_eq!(program.agents[0].id, 0);
        assert_eq!(program.agents[1].id, 1);
        assert_eq!(program.agents[1].definitions.len(), 1); // macro a
    }
}
