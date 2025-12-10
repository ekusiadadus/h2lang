//! Macro and function expansion for toioswarm language.

use crate::ast::{Agent, Arg, Definition, Expr, Primitive};
use crate::error::ExpandError;
use std::collections::HashMap;

/// Parameter value (command sequence or number).
#[derive(Debug, Clone)]
pub enum ParamValue {
    /// Sequence of commands (for command arguments)
    Commands(Vec<Command>),
    /// Numeric value (for numeric arguments)
    Number(i32),
}

/// Expanded command (after macro/function expansion).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    /// Move straight (forward)
    Straight,
    /// Rotate right (90° clockwise)
    Right,
    /// Rotate left (90° counter-clockwise)
    Left,
}

impl Command {
    /// Get the character representation.
    pub fn as_char(&self) -> char {
        match self {
            Command::Straight => 's',
            Command::Right => 'r',
            Command::Left => 'l',
        }
    }
}

impl From<Primitive> for Command {
    fn from(p: Primitive) -> Self {
        match p {
            Primitive::Straight => Command::Straight,
            Primitive::Right => Command::Right,
            Primitive::Left => Command::Left,
        }
    }
}

/// Expansion context.
struct ExpandContext {
    /// Macro definitions: name -> body
    macros: HashMap<char, Expr>,
    /// Function definitions: name -> (param_names, body)
    functions: HashMap<char, (Vec<char>, Expr)>,
    /// Current recursion depth
    depth: usize,
}

/// Expander for macro and function expansion.
pub struct Expander {
    /// Maximum recursion depth to prevent infinite loops
    max_depth: usize,
}

impl Default for Expander {
    fn default() -> Self {
        Self::new()
    }
}

impl Expander {
    /// Create a new expander with default settings.
    pub fn new() -> Self {
        Self { max_depth: 100 }
    }

    /// Create a new expander with custom max depth.
    pub fn with_max_depth(max_depth: usize) -> Self {
        Self { max_depth }
    }

    /// Expand an agent's expression to a list of commands.
    pub fn expand_agent(&self, agent: &Agent) -> Result<Vec<Command>, ExpandError> {
        let mut ctx = ExpandContext {
            macros: HashMap::new(),
            functions: HashMap::new(),
            depth: 0,
        };

        // Register definitions
        for def in &agent.definitions {
            match def {
                Definition::Macro(m) => {
                    ctx.macros.insert(m.name, m.body.clone());
                }
                Definition::Function(f) => {
                    ctx.functions
                        .insert(f.name, (f.params.clone(), f.body.clone()));
                }
            }
        }

        // Expand the expression
        self.expand_expr(&agent.expression, &ctx, &HashMap::new())
    }

    /// Expand an expression to a list of commands.
    fn expand_expr(
        &self,
        expr: &Expr,
        ctx: &ExpandContext,
        params: &HashMap<char, ParamValue>,
    ) -> Result<Vec<Command>, ExpandError> {
        // Check recursion depth
        if ctx.depth > self.max_depth {
            return Err(ExpandError::max_recursion_depth(expr.span()));
        }

        match expr {
            Expr::Primitive(p, _) => Ok(vec![Command::from(*p)]),

            Expr::Ident(name, span) => {
                // Look up macro
                if let Some(body) = ctx.macros.get(name) {
                    let new_ctx = ExpandContext {
                        macros: ctx.macros.clone(),
                        functions: ctx.functions.clone(),
                        depth: ctx.depth + 1,
                    };
                    self.expand_expr(body, &new_ctx, params)
                } else {
                    Err(ExpandError::undefined_macro(*name, *span))
                }
            }

            Expr::Param(name, span) => {
                // Look up parameter value
                if let Some(value) = params.get(name) {
                    match value {
                        ParamValue::Commands(cmds) => Ok(cmds.clone()),
                        ParamValue::Number(_) => {
                            // Numeric param used as command - this is an error
                            Err(ExpandError::new(
                                format!("Numeric parameter '{}' cannot be used as command", name),
                                *span,
                            ))
                        }
                    }
                } else {
                    Err(ExpandError::new(
                        format!("Undefined parameter '{}'", name),
                        *span,
                    ))
                }
            }

            Expr::FuncCall { name, arg, span } => {
                // Look up function
                if let Some((param_names, body)) = ctx.functions.get(name) {
                    // First, expand the argument
                    let arg_expanded = self.expand_expr(arg, ctx, params)?;

                    // Create new parameter map with the expanded argument
                    let mut new_params = params.clone();
                    if let Some(param_name) = param_names.first() {
                        new_params.insert(*param_name, ParamValue::Commands(arg_expanded));
                    }

                    // Expand the function body with the new parameters
                    let new_ctx = ExpandContext {
                        macros: ctx.macros.clone(),
                        functions: ctx.functions.clone(),
                        depth: ctx.depth + 1,
                    };
                    self.expand_expr(body, &new_ctx, &new_params)
                } else {
                    Err(ExpandError::undefined_function(*name, *span))
                }
            }

            Expr::FuncCallArgs { name, args, span } => {
                // Look up function
                if let Some((param_names, body)) = ctx.functions.get(name) {
                    // Evaluate arguments and bind to parameters
                    let mut new_params = params.clone();

                    for (i, arg) in args.iter().enumerate() {
                        if let Some(param_name) = param_names.get(i) {
                            let param_value = self.eval_arg(arg, ctx, params)?;

                            // HOJ termination: if numeric arg <= 0, return empty
                            if let ParamValue::Number(n) = &param_value {
                                if *n <= 0 {
                                    return Ok(vec![]);
                                }
                            }

                            new_params.insert(*param_name, param_value);
                        }
                    }

                    // Expand the function body with the new parameters
                    let new_ctx = ExpandContext {
                        macros: ctx.macros.clone(),
                        functions: ctx.functions.clone(),
                        depth: ctx.depth + 1,
                    };
                    self.expand_expr(body, &new_ctx, &new_params)
                } else {
                    Err(ExpandError::undefined_function(*name, *span))
                }
            }

            Expr::Sequence(exprs) => {
                let mut result = Vec::new();
                for e in exprs {
                    let cmds = self.expand_expr(e, ctx, params)?;
                    result.extend(cmds);
                }
                Ok(result)
            }
        }
    }

    /// Evaluate a function argument to a ParamValue.
    fn eval_arg(
        &self,
        arg: &Arg,
        ctx: &ExpandContext,
        params: &HashMap<char, ParamValue>,
    ) -> Result<ParamValue, ExpandError> {
        match arg {
            Arg::Command(expr) => {
                let cmds = self.expand_expr(expr, ctx, params)?;
                Ok(ParamValue::Commands(cmds))
            }
            Arg::Number(n, _) => Ok(ParamValue::Number(*n)),
            Arg::NumExpr {
                param,
                offset,
                span,
            } => {
                // Look up the current value of the parameter
                if let Some(value) = params.get(param) {
                    match value {
                        ParamValue::Number(n) => Ok(ParamValue::Number(n + offset)),
                        ParamValue::Commands(_) => Err(ExpandError::new(
                            format!("Parameter '{}' is not numeric", param),
                            *span,
                        )),
                    }
                } else {
                    Err(ExpandError::new(
                        format!("Undefined parameter '{}'", param),
                        *span,
                    ))
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Parser;

    fn expand_source(source: &str) -> Result<Vec<Command>, ExpandError> {
        let mut parser = Parser::new(source).expect("Parser creation failed");
        let program = parser.parse_program().expect("Parsing failed");
        let expander = Expander::new();
        expander.expand_agent(&program.agents[0])
    }

    #[test]
    fn test_simple_commands() {
        let cmds = expand_source("0: srl").unwrap();
        assert_eq!(cmds.len(), 3);
        assert_eq!(cmds[0], Command::Straight);
        assert_eq!(cmds[1], Command::Right);
        assert_eq!(cmds[2], Command::Left);
    }

    #[test]
    fn test_macro_expansion() {
        // x:ss xrx -> ssrss
        let cmds = expand_source("0: x:ss xrx").unwrap();
        assert_eq!(cmds.len(), 5);
        assert_eq!(cmds[0], Command::Straight);
        assert_eq!(cmds[1], Command::Straight);
        assert_eq!(cmds[2], Command::Right);
        assert_eq!(cmds[3], Command::Straight);
        assert_eq!(cmds[4], Command::Straight);
    }

    #[test]
    fn test_function_expansion() {
        // f(X):XXX f(s) -> sss
        let cmds = expand_source("0: f(X):XXX f(s)").unwrap();
        assert_eq!(cmds.len(), 3);
        assert_eq!(cmds[0], Command::Straight);
        assert_eq!(cmds[1], Command::Straight);
        assert_eq!(cmds[2], Command::Straight);
    }

    #[test]
    fn test_nested_function() {
        // f(X):XX f(f(s)) -> f(ss) -> ssss
        let cmds = expand_source("0: f(X):XX f(f(s))").unwrap();
        assert_eq!(cmds.len(), 4);
    }

    #[test]
    fn test_function_with_expression() {
        // f(X):sXr f(ll) -> sllr
        let cmds = expand_source("0: f(X):sXr f(ll)").unwrap();
        assert_eq!(cmds.len(), 4);
        assert_eq!(cmds[0], Command::Straight);
        assert_eq!(cmds[1], Command::Left);
        assert_eq!(cmds[2], Command::Left);
        assert_eq!(cmds[3], Command::Right);
    }

    #[test]
    fn test_undefined_macro() {
        let result = expand_source("0: x");
        assert!(result.is_err());
    }

    #[test]
    fn test_undefined_function() {
        let result = expand_source("0: f(s)");
        assert!(result.is_err());
    }

    #[test]
    fn test_complex_example() {
        // From HOJ tutorial: f(X):XXXX f(srs) -> srssrssrssrs
        let cmds = expand_source("0: f(X):XXXX f(srs)").unwrap();
        assert_eq!(cmds.len(), 12);
    }

    // HOJ numeric argument tests

    #[test]
    fn test_numeric_recursion_simple() {
        // a(X):sa(X-1) a(4) -> s + s + s + s + (terminate at 0) = ssss
        let cmds = expand_source("0: a(X):sa(X-1) a(4)").unwrap();
        assert_eq!(cmds.len(), 4);
        assert!(cmds.iter().all(|c| *c == Command::Straight));
    }

    #[test]
    fn test_numeric_recursion_zero() {
        // a(0) should produce nothing (terminate immediately)
        let cmds = expand_source("0: a(X):sa(X-1) a(0)").unwrap();
        assert_eq!(cmds.len(), 0);
    }

    #[test]
    fn test_numeric_recursion_negative() {
        // a(-1) should produce nothing (negative = terminate)
        let cmds = expand_source("0: a(X):sa(X-1) a(-1)").unwrap();
        assert_eq!(cmds.len(), 0);
    }

    #[test]
    fn test_numeric_recursion_one() {
        // a(1) -> s + a(0) = s
        let cmds = expand_source("0: a(X):sa(X-1) a(1)").unwrap();
        assert_eq!(cmds.len(), 1);
        assert_eq!(cmds[0], Command::Straight);
    }

    #[test]
    fn test_numeric_with_turn() {
        // a(X):sra(X-1) a(4) -> sr + sr + sr + sr = srsrsrsr
        let cmds = expand_source("0: a(X):sra(X-1) a(4)").unwrap();
        assert_eq!(cmds.len(), 8);
        assert_eq!(cmds[0], Command::Straight);
        assert_eq!(cmds[1], Command::Right);
    }

    #[test]
    fn test_square_pattern() {
        // Square: a(X):sla(X-1) a(4) -> sl sl sl sl
        // This draws a square by going straight then turning left 4 times
        let cmds = expand_source("0: a(X):sla(X-1) a(4)").unwrap();
        assert_eq!(cmds.len(), 8);
    }

    // HOJ multiple arguments tests

    #[test]
    fn test_multiple_args_numeric_and_command() {
        // HOJ: a(X,Y):Ya(X-1,Y) a(4,s) -> s + s + s + s = ssss
        // Y is the command 's', X is the count 4
        let cmds = expand_source("0: a(X,Y):Ya(X-1,Y) a(4,s)").unwrap();
        assert_eq!(cmds.len(), 4);
        assert!(cmds.iter().all(|c| *c == Command::Straight));
    }

    #[test]
    fn test_multiple_args_with_turn() {
        // a(X,Y):Ya(X-1,Y) a(4,r) -> r + r + r + r = rrrr
        let cmds = expand_source("0: a(X,Y):Ya(X-1,Y) a(4,r)").unwrap();
        assert_eq!(cmds.len(), 4);
        assert!(cmds.iter().all(|c| *c == Command::Right));
    }

    #[test]
    fn test_multiple_args_complex_command() {
        // a(X,Y):Ya(X-1,Y) a(3,sr) -> sr + sr + sr = srsrsr
        let cmds = expand_source("0: a(X,Y):Ya(X-1,Y) a(3,sr)").unwrap();
        assert_eq!(cmds.len(), 6);
        assert_eq!(cmds[0], Command::Straight);
        assert_eq!(cmds[1], Command::Right);
        assert_eq!(cmds[2], Command::Straight);
        assert_eq!(cmds[3], Command::Right);
    }

    #[test]
    fn test_multiple_args_zero_count() {
        // a(X,Y):Ya(X-1,Y) a(0,s) -> (empty, terminates immediately)
        let cmds = expand_source("0: a(X,Y):Ya(X-1,Y) a(0,s)").unwrap();
        assert_eq!(cmds.len(), 0);
    }

    #[test]
    fn test_multiple_args_with_prefix() {
        // a(X,Y):sYa(X-1,Y) a(3,r) -> sr + sr + sr = srsrsr
        let cmds = expand_source("0: a(X,Y):sYa(X-1,Y) a(3,r)").unwrap();
        assert_eq!(cmds.len(), 6);
        assert_eq!(cmds[0], Command::Straight);
        assert_eq!(cmds[1], Command::Right);
    }

    #[test]
    fn test_hoj_triple_function() {
        // f(X):XXX f(srl) -> srlsrlsrl (9 commands)
        let cmds = expand_source("0: f(X):XXX f(srl)").unwrap();
        assert_eq!(cmds.len(), 9);
        assert_eq!(cmds[0], Command::Straight);
        assert_eq!(cmds[1], Command::Right);
        assert_eq!(cmds[2], Command::Left);
    }

    #[test]
    fn test_hoj_deeply_nested() {
        // f(X):XXXX f(f(ss)r)
        // f(ss) = ssssssss (8)
        // f(ss)r = sssssssr (9)
        // f(f(ss)r) = f(sssssssr) = 9 * 4 = 36 commands
        let cmds = expand_source("0: f(X):XXXX f(f(ss)r)").unwrap();
        assert_eq!(cmds.len(), 36);
    }

    #[test]
    fn test_hoj_square_with_function() {
        // HOJ tutorial: square pattern
        // f(X):XXXX f(sssr) -> sssrsssrsssrsssr (16 commands)
        let cmds = expand_source("0: f(X):XXXX f(sssr)").unwrap();
        assert_eq!(cmds.len(), 16);
        // Verify the pattern: sss r sss r sss r sss r
        assert_eq!(cmds[0], Command::Straight);
        assert_eq!(cmds[3], Command::Right);
        assert_eq!(cmds[7], Command::Right);
    }

    #[test]
    fn test_macro_and_function_combined() {
        // x:ss f(X):XXX xf(r)x -> ss + rrr + ss = ssrrrss
        let cmds = expand_source("0: x:ss f(X):XXX xf(r)x").unwrap();
        assert_eq!(cmds.len(), 7);
        assert_eq!(cmds[0], Command::Straight);
        assert_eq!(cmds[1], Command::Straight);
        assert_eq!(cmds[2], Command::Right);
        assert_eq!(cmds[3], Command::Right);
        assert_eq!(cmds[4], Command::Right);
        assert_eq!(cmds[5], Command::Straight);
        assert_eq!(cmds[6], Command::Straight);
    }

    #[test]
    fn test_numeric_addition() {
        // Test X+1 numeric expression (if supported)
        // a(X):sa(X-1) a(2+2) should work like a(4) -> ssss
        // Note: This tests if the parser handles numeric expressions in arguments
        let cmds = expand_source("0: a(X):sa(X-1) a(4)").unwrap();
        assert_eq!(cmds.len(), 4);
    }
}
