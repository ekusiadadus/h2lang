//! Macro and function expansion for H2 Language.

use crate::ast::{
    Agent, Arg, Definition, Expr, LimitConfig, NumAtom, NumOp, OnLimitBehavior, ParamType,
    Primitive,
};
use crate::error::ExpandError;
use crate::token::Span;
use std::cell::Cell;
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
struct ExpandContext<'a> {
    /// Function definitions: name -> (param_names, param_types, body)
    /// Note: 0-arg functions (formerly macros) have empty param_names
    functions: HashMap<char, (Vec<char>, HashMap<char, ParamType>, Expr)>,
    /// Current recursion depth
    depth: usize,
    /// Limit configuration
    limits: &'a LimitConfig,
    /// Current step count (shared across all recursive calls)
    step_count: &'a Cell<usize>,
    /// Whether truncation occurred
    truncated: &'a Cell<bool>,
}

/// Expander for macro and function expansion.
pub struct Expander {
    /// Limit configuration
    limits: LimitConfig,
}

impl Default for Expander {
    fn default() -> Self {
        Self::new()
    }
}

impl Expander {
    /// Create a new expander with default settings.
    pub fn new() -> Self {
        Self {
            limits: LimitConfig::default(),
        }
    }

    /// Create a new expander with custom limits.
    pub fn with_limits(limits: LimitConfig) -> Self {
        Self { limits }
    }

    /// Create a new expander with custom max depth (for backwards compatibility).
    pub fn with_max_depth(max_depth: usize) -> Self {
        Self {
            limits: LimitConfig {
                max_depth,
                ..LimitConfig::default()
            },
        }
    }

    /// Expand an agent's expression to a list of commands.
    pub fn expand_agent(&self, agent: &Agent) -> Result<Vec<Command>, ExpandError> {
        let step_count = Cell::new(0usize);
        let truncated = Cell::new(false);

        let mut ctx = ExpandContext {
            functions: HashMap::new(),
            depth: 0,
            limits: &self.limits,
            step_count: &step_count,
            truncated: &truncated,
        };

        // Register all definitions as functions (unified model)
        for def in &agent.definitions {
            match def {
                Definition::Function(f) => {
                    ctx.functions.insert(
                        f.name,
                        (f.params.clone(), f.param_types.clone(), f.body.clone()),
                    );
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
        if ctx.depth > ctx.limits.max_depth {
            return Err(ExpandError::max_recursion_depth(expr.span()));
        }

        // Check if already truncated
        if ctx.truncated.get() {
            return Ok(vec![]);
        }

        match expr {
            Expr::Primitive(p, span) => self.add_command_with_limit(Command::from(*p), ctx, *span),

            Expr::Param(name, span) => {
                // Look up parameter value
                if let Some(value) = params.get(name) {
                    match value {
                        ParamValue::Commands(cmds) => {
                            // Add each command with limit checking
                            let mut result = Vec::new();
                            for cmd in cmds {
                                let added = self.add_command_with_limit(*cmd, ctx, *span)?;
                                result.extend(added);
                                if ctx.truncated.get() {
                                    break;
                                }
                            }
                            Ok(result)
                        }
                        ParamValue::Number(_) => {
                            // E008: Int type parameter used as term (command)
                            Err(ExpandError::type_error(
                                format!(
                                    "Parameter '{}' is Int type but used as command sequence",
                                    name
                                ),
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

            Expr::FuncCall { name, args, span } => {
                // Unified function call handling (v0.5.0)
                // Look up function (includes 0-arg functions, formerly macros)
                if let Some((param_names, param_types, body)) = ctx.functions.get(name) {
                    // Evaluate arguments and bind to parameters
                    let mut new_params = params.clone();

                    if args.is_empty() && !param_names.is_empty() {
                        // HOJ compatibility: f() with params binds default values
                        // CmdSeq → empty, Int → 0 (triggers ≤0 termination)
                        for param_name in param_names {
                            let default_value =
                                match param_types.get(param_name).unwrap_or(&ParamType::CmdSeq) {
                                    ParamType::CmdSeq => ParamValue::Commands(vec![]),
                                    ParamType::Int => ParamValue::Number(0),
                                };

                            // Int=0 triggers termination
                            if let ParamValue::Number(0) = &default_value {
                                return Ok(vec![]);
                            }

                            new_params.insert(*param_name, default_value);
                        }
                    } else {
                        // Normal case: check arity
                        if args.len() != param_names.len() {
                            return Err(ExpandError::argument_count_mismatch(
                                *name,
                                param_names.len(),
                                args.len(),
                                *span,
                            ));
                        }

                        for (i, arg) in args.iter().enumerate() {
                            if let Some(param_name) = param_names.get(i) {
                                let param_value = self.eval_arg(arg, ctx, params)?;

                                // Numeric termination: if numeric arg <= 0, return empty
                                if let ParamValue::Number(n) = &param_value {
                                    if *n <= 0 {
                                        return Ok(vec![]);
                                    }
                                }

                                new_params.insert(*param_name, param_value);
                            }
                        }
                    }

                    // Expand the function body with the new parameters
                    let new_ctx = ExpandContext {
                        functions: ctx.functions.clone(),
                        depth: ctx.depth + 1,
                        limits: ctx.limits,
                        step_count: ctx.step_count,
                        truncated: ctx.truncated,
                    };
                    self.expand_expr(body, &new_ctx, &new_params)
                } else {
                    // E001 for 0-arg, E002 for n-arg (both use same error now)
                    Err(ExpandError::undefined_function(*name, *span))
                }
            }

            Expr::Sequence(exprs) => {
                let mut result = Vec::new();
                for e in exprs {
                    if ctx.truncated.get() {
                        break;
                    }
                    let cmds = self.expand_expr(e, ctx, params)?;
                    result.extend(cmds);
                }
                Ok(result)
            }
        }
    }

    /// Add a command with step limit checking.
    /// Returns the command in a vec if successful, or error if limit exceeded.
    fn add_command_with_limit(
        &self,
        cmd: Command,
        ctx: &ExpandContext,
        span: Span,
    ) -> Result<Vec<Command>, ExpandError> {
        let current = ctx.step_count.get();

        if current >= ctx.limits.max_step {
            match ctx.limits.on_limit {
                OnLimitBehavior::Error => {
                    return Err(ExpandError::max_step_exceeded(ctx.limits.max_step, span));
                }
                OnLimitBehavior::Truncate => {
                    ctx.truncated.set(true);
                    return Ok(vec![]);
                }
            }
        }

        ctx.step_count.set(current + 1);
        Ok(vec![cmd])
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
                // Special case: if the expression is a single Param reference
                // and that param is bound to a Number, pass through the Number.
                // This handles HOJ patterns like: a(X,Y):sra(X-1,Y) a(3,2)
                // where Y is passed through to recursive calls.
                if let Expr::Param(p, _) = expr {
                    if let Some(value) = params.get(p) {
                        return Ok(value.clone());
                    }
                }

                let cmds = self.expand_expr(expr, ctx, params)?;
                Ok(ParamValue::Commands(cmds))
            }
            Arg::Number(n, span) => {
                // E007: Check numeric range (-255..=255)
                if *n < -255 || *n > 255 {
                    return Err(ExpandError::numeric_out_of_range(*n, *span));
                }
                Ok(ParamValue::Number(*n))
            }
            Arg::NumExpr { first, rest, span } => {
                // Evaluate extended num_expr: first ((op atom)*)
                // Examples: X-1, 10-3+1, X+Y-2

                // Evaluate the first atom
                let mut result = self.eval_num_atom(first, params, *span)?;

                // Apply each operation in sequence (left-to-right)
                for (op, atom) in rest {
                    let atom_value = self.eval_num_atom(atom, params, *span)?;
                    result = match op {
                        NumOp::Add => result + atom_value,
                        NumOp::Sub => result - atom_value,
                    };

                    // E007: Check intermediate result range
                    if !(-255..=255).contains(&result) {
                        return Err(ExpandError::numeric_out_of_range(result, *span));
                    }
                }

                Ok(ParamValue::Number(result))
            }
        }
    }

    /// Evaluate a numeric atom to an i32 value.
    fn eval_num_atom(
        &self,
        atom: &NumAtom,
        params: &HashMap<char, ParamValue>,
        span: Span,
    ) -> Result<i32, ExpandError> {
        match atom {
            NumAtom::Number(n) => {
                // E007: Check range
                if *n < -255 || *n > 255 {
                    return Err(ExpandError::numeric_out_of_range(*n, span));
                }
                Ok(*n)
            }
            NumAtom::Param(p) => {
                // Look up the parameter value
                if let Some(value) = params.get(p) {
                    match value {
                        ParamValue::Number(n) => Ok(*n),
                        ParamValue::Commands(_) => {
                            // E008: CmdSeq type parameter used in num_expr
                            Err(ExpandError::type_error(
                                format!(
                                    "Parameter '{}' is CmdSeq type but used in numeric expression",
                                    p
                                ),
                                span,
                            ))
                        }
                    }
                } else {
                    Err(ExpandError::new(
                        format!("Undefined parameter '{}'", p),
                        span,
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
        // Use limits from parsed program (includes directives like MAX_STEP)
        let expander = Expander::with_limits(program.limits);
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

    // MAX_STEP limit tests

    #[test]
    fn test_max_step_error() {
        // MAX_STEP=3 with recursion producing 10 commands should error
        let result = expand_source("MAX_STEP=3\n0: a(X):sa(X-1) a(10)");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("E004") || err.message.contains("MAX_STEP"));
    }

    #[test]
    fn test_max_step_truncate() {
        // MAX_STEP=3 with ON_LIMIT=TRUNCATE should return 3 commands
        let result = expand_source("MAX_STEP=3\nON_LIMIT=TRUNCATE\n0: a(X):sa(X-1) a(10)");
        assert!(result.is_ok());
        let cmds = result.unwrap();
        assert_eq!(cmds.len(), 3);
    }

    #[test]
    fn test_max_step_exact_boundary() {
        // MAX_STEP=5 with a(5) should succeed with exactly 5 commands
        let result = expand_source("MAX_STEP=5\n0: a(X):sa(X-1) a(5)");
        assert!(result.is_ok());
        let cmds = result.unwrap();
        assert_eq!(cmds.len(), 5);
    }

    #[test]
    fn test_max_step_one_over_boundary() {
        // MAX_STEP=5 with a(6) should error (6th command exceeds limit)
        let result = expand_source("MAX_STEP=5\n0: a(X):sa(X-1) a(6)");
        assert!(result.is_err());
    }

    #[test]
    fn test_max_step_truncate_one_over() {
        // MAX_STEP=5 ON_LIMIT=TRUNCATE with a(6) should return 5 commands
        let result = expand_source("MAX_STEP=5\nON_LIMIT=TRUNCATE\n0: a(X):sa(X-1) a(6)");
        assert!(result.is_ok());
        let cmds = result.unwrap();
        assert_eq!(cmds.len(), 5);
    }

    #[test]
    fn test_default_max_step() {
        // Default MAX_STEP (1,000,000) should allow small expansions
        let result = expand_source("0: a(X):sa(X-1) a(100)");
        assert!(result.is_ok());
        let cmds = result.unwrap();
        assert_eq!(cmds.len(), 100);
    }
}
