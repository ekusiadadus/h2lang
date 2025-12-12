//! Abstract Syntax Tree definitions for H2 Language.

use crate::token::Span;

// =============================================================================
// Directives and Limits
// =============================================================================

/// Behavior when execution limit is exceeded.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OnLimitBehavior {
    /// Return error and stop (default)
    #[default]
    Error,
    /// Truncate output and return what was generated so far
    Truncate,
}

/// Execution limit configuration.
#[derive(Debug, Clone)]
pub struct LimitConfig {
    /// Maximum number of generated commands (s/r/l)
    /// Default: 1,000,000
    pub max_step: usize,
    /// Maximum recursion depth
    /// Default: 100
    pub max_depth: usize,
    /// Maximum memory usage during expansion (in bytes)
    /// Default: 1,000,000
    pub max_memory: usize,
    /// Behavior when limit is exceeded
    pub on_limit: OnLimitBehavior,
}

impl Default for LimitConfig {
    fn default() -> Self {
        Self {
            max_step: 1_000_000,
            // Depth limit to prevent Rust stack overflow
            // Spec default is 100; safe for deep nested calls
            max_depth: 100,
            max_memory: 1_000_000,
            // HOJ compatibility: truncate by default instead of error
            on_limit: OnLimitBehavior::Truncate,
        }
    }
}

/// A single directive (e.g., MAX_STEP=1000).
#[derive(Debug, Clone)]
pub struct Directive {
    /// Directive name (e.g., "MAX_STEP")
    pub name: String,
    /// Directive value (number or string like "ERROR"/"TRUNCATE")
    pub value: DirectiveValue,
    /// Source location
    pub span: Span,
}

/// Value of a directive.
#[derive(Debug, Clone)]
pub enum DirectiveValue {
    /// Numeric value
    Number(i64),
    /// String value (for ON_LIMIT)
    String(String),
}

/// Basic command primitive.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Primitive {
    /// 's' - move straight (forward)
    Straight,
    /// 'r' - rotate right (90° clockwise)
    Right,
    /// 'l' - rotate left (90° counter-clockwise)
    Left,
}

impl Primitive {
    /// Get the character representation.
    pub fn as_char(&self) -> char {
        match self {
            Primitive::Straight => 's',
            Primitive::Right => 'r',
            Primitive::Left => 'l',
        }
    }
}

/// Function/Procedure argument (at call site).
#[derive(Debug, Clone)]
pub enum Arg {
    /// Command expression (e.g., `srl`, `XXX`)
    Command(Expr),
    /// Number literal (e.g., `4`, `12`)
    Number(i32, Span),
    /// Numeric expression with parameter (e.g., `X-1`, `X+2`)
    NumExpr {
        param: char,
        offset: i32,
        span: Span,
    },
}

impl Arg {
    /// Get the span of this argument.
    pub fn span(&self) -> Span {
        match self {
            Arg::Command(expr) => expr.span(),
            Arg::Number(_, span) => *span,
            Arg::NumExpr { span, .. } => *span,
        }
    }
}

/// Expression (before expansion).
#[derive(Debug, Clone)]
pub enum Expr {
    /// Basic command
    Primitive(Primitive, Span),
    /// Identifier reference (macro expansion target)
    Ident(char, Span),
    /// Parameter reference (inside function body)
    Param(char, Span),
    /// Function call with single command argument (legacy)
    FuncCall {
        name: char,
        arg: Box<Expr>,
        span: Span,
    },
    /// Function call with multiple/typed arguments (HOJ-compatible)
    FuncCallArgs {
        name: char,
        args: Vec<Arg>,
        span: Span,
    },
    /// Sequence of expressions
    Sequence(Vec<Expr>),
}

impl Expr {
    /// Get the span of this expression.
    pub fn span(&self) -> Span {
        match self {
            Expr::Primitive(_, span) => *span,
            Expr::Ident(_, span) => *span,
            Expr::Param(_, span) => *span,
            Expr::FuncCall { span, .. } => *span,
            Expr::FuncCallArgs { span, .. } => *span,
            Expr::Sequence(exprs) => {
                if exprs.is_empty() {
                    Span::default()
                } else {
                    let first = exprs.first().unwrap().span();
                    let last = exprs.last().unwrap().span();
                    Span::new(first.start, last.end, first.line, first.column)
                }
            }
        }
    }

    /// Check if this is an empty sequence.
    pub fn is_empty(&self) -> bool {
        matches!(self, Expr::Sequence(exprs) if exprs.is_empty())
    }
}

/// Macro definition.
#[derive(Debug, Clone)]
pub struct MacroDef {
    /// Macro name (single lowercase letter)
    pub name: char,
    /// Macro body
    pub body: Expr,
    /// Source location
    pub span: Span,
}

/// Function definition.
#[derive(Debug, Clone)]
pub struct FuncDef {
    /// Function name (single lowercase letter)
    pub name: char,
    /// Parameter names (uppercase letters)
    pub params: Vec<char>,
    /// Function body
    pub body: Expr,
    /// Source location
    pub span: Span,
}

/// Definition (macro or function).
#[derive(Debug, Clone)]
pub enum Definition {
    Macro(MacroDef),
    Function(FuncDef),
}

impl Definition {
    /// Get the name of this definition.
    pub fn name(&self) -> char {
        match self {
            Definition::Macro(m) => m.name,
            Definition::Function(f) => f.name,
        }
    }

    /// Get the span of this definition.
    pub fn span(&self) -> Span {
        match self {
            Definition::Macro(m) => m.span,
            Definition::Function(f) => f.span,
        }
    }
}

/// Agent definition.
#[derive(Debug, Clone)]
pub struct Agent {
    /// Agent ID (0, 1, 2, ...)
    pub id: u32,
    /// Macro and function definitions
    pub definitions: Vec<Definition>,
    /// Expression to execute
    pub expression: Expr,
    /// Source location
    pub span: Span,
}

/// Program (collection of agents).
#[derive(Debug, Clone)]
pub struct Program {
    /// Directives (e.g., MAX_STEP=1000)
    pub directives: Vec<Directive>,
    /// Execution limit configuration (derived from directives)
    pub limits: LimitConfig,
    /// List of agents
    pub agents: Vec<Agent>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primitive_char() {
        assert_eq!(Primitive::Straight.as_char(), 's');
        assert_eq!(Primitive::Right.as_char(), 'r');
        assert_eq!(Primitive::Left.as_char(), 'l');
    }

    #[test]
    fn test_expr_span() {
        let span = Span::new(0, 1, 1, 1);
        let expr = Expr::Primitive(Primitive::Straight, span);
        assert_eq!(expr.span().start, 0);
        assert_eq!(expr.span().end, 1);
    }

    #[test]
    fn test_sequence_span() {
        let span1 = Span::new(0, 1, 1, 1);
        let span2 = Span::new(1, 2, 1, 2);
        let expr = Expr::Sequence(vec![
            Expr::Primitive(Primitive::Straight, span1),
            Expr::Primitive(Primitive::Right, span2),
        ]);
        let combined = expr.span();
        assert_eq!(combined.start, 0);
        assert_eq!(combined.end, 2);
    }
}
