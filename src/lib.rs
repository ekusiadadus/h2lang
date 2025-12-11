//! # H2 Language Compiler
//!
//! `h2lang` is a compiler for the H2 programming language, which is fully compatible
//! with the [Herbert Online Judge (HOJ)](https://github.com/quolc/hoj) H language
//! specification while extending it with multi-agent support for robot swarm control.
//!
//! This crate contains no unsafe code.
//!
//! ## Overview
//!
//! H2 Language is a domain-specific language designed for controlling robots through
//! simple movement commands. It supports:
//!
//! - **Basic Commands**: `s` (straight/forward), `r` (right turn), `l` (left turn)
//! - **Macros**: Reusable command sequences defined with lowercase letters
//! - **Functions**: Parameterized procedures with recursion support
//! - **Multi-Agent**: Control multiple robots simultaneously (H2 extension)
//!
//! ## Quick Start
//!
//! ```rust
//! use h2lang::compile_native;
//! use h2lang::output::CompileResult;
//!
//! // Compile a simple program
//! let result = compile_native("0: srl");
//!
//! match result {
//!     CompileResult::Success { program } => {
//!         println!("Compiled {} agents", program.agents.len());
//!         println!("Total steps: {}", program.max_steps);
//!     }
//!     CompileResult::Error { errors } => {
//!         for err in errors {
//!             eprintln!("Error: {}", err.message);
//!         }
//!     }
//! }
//! ```
//!
//! ## Language Syntax
//!
//! ### Basic Commands
//!
//! | Command | Description |
//! |---------|-------------|
//! | `s` | Move forward one step |
//! | `r` | Rotate 90° clockwise |
//! | `l` | Rotate 90° counter-clockwise |
//!
//! ### Agent Definition
//!
//! ```text
//! agent_id: commands
//! 0: srl        // Agent 0: straight, right, left
//! 1: llss       // Agent 1: left, left, straight, straight
//! ```
//!
//! ### Macros
//!
//! ```text
//! x:ss          // Define macro 'x' as 'ss'
//! xrx           // Expands to: ssrss
//! ```
//!
//! ### Functions
//!
//! ```text
//! f(X):XXX f(s)           // Repeats argument 3 times → sss
//! a(X,Y):Ya(X-1,Y) a(4,s) // Recursive with multiple params → ssss
//! ```
//!
//! ## WebAssembly Support
//!
//! This crate compiles to WebAssembly for use in browsers:
//!
//! ```javascript
//! import init, { compile, validate, version } from 'h2lang';
//!
//! await init();
//! const result = compile('0: srl');
//! console.log(result);
//! ```
//!
//! ## Module Structure
//!
//! - [`ast`]: Abstract Syntax Tree definitions
//! - [`lexer`]: Tokenizer for source code
//! - [`parser`]: Recursive descent parser
//! - [`expander`]: Macro and function expansion
//! - [`scheduler`]: Multi-agent parallel scheduling
//! - [`output`]: JSON-serializable output structures
//! - [`token`]: Token type definitions
//! - [`error`]: Error types for compilation stages
//!
//! ## References
//!
//! - [Herbert Online Judge](http://herbert.tealang.info/)
//! - [HOJ GitHub Repository](https://github.com/quolc/hoj)
//! - [Codeforces Discussion](https://codeforces.com/blog/entry/5579)

#![doc(html_root_url = "https://docs.rs/h2lang/0.2.0")]
#![forbid(unsafe_code)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
// TODO: Re-enable once all public APIs are documented
// #![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]

pub mod ast;
pub mod error;
pub mod expander;
pub mod lexer;
pub mod output;
pub mod parser;
pub mod scheduler;
pub mod token;

use expander::Expander;
use output::{CompileResult, CompiledProgram};
use parser::Parser;
use scheduler::Scheduler;
use wasm_bindgen::prelude::*;

// =============================================================================
// WebAssembly API
// =============================================================================

/// Initializes the panic hook for better error messages in WebAssembly.
///
/// This function is automatically called when the WASM module is loaded.
/// It sets up `console_error_panic_hook` to provide readable panic messages
/// in the browser console instead of cryptic WASM error codes.
///
/// # Example
///
/// This is called automatically, but can be invoked manually if needed:
///
/// ```ignore
/// h2lang::init();
/// ```
#[wasm_bindgen(start)]
pub fn init() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

/// Compiles H2 source code to a structured program.
///
/// This is the main entry point for WebAssembly usage. It takes H2 source code
/// as input and returns a JSON-serializable result containing either the
/// compiled program or compilation errors.
///
/// # Arguments
///
/// * `source` - The H2 source code to compile
///
/// # Returns
///
/// A [`JsValue`] containing a serialized [`CompileResult`]:
/// - On success: `{ "status": "success", "program": { ... } }`
/// - On error: `{ "status": "error", "errors": [ ... ] }`
///
/// # Example (JavaScript)
///
/// ```javascript
/// const result = compile('0: f(X):XXXX f(sssr)');
/// if (result.status === 'success') {
///     console.log('Commands:', result.program.agents[0].commands);
/// }
/// ```
///
/// # Errors
///
/// Returns an error result for:
/// - Syntax errors (malformed source code)
/// - Undefined macro/function references
/// - Invalid agent ID format
/// - Maximum recursion depth exceeded
#[wasm_bindgen]
pub fn compile(source: &str) -> JsValue {
    let result = compile_internal(source);
    serde_wasm_bindgen::to_value(&result).unwrap_or(JsValue::NULL)
}

/// Validates H2 source code without full compilation.
///
/// Performs lexical and syntactic analysis without macro/function expansion.
/// Useful for quick syntax checking in editors or IDEs.
///
/// # Arguments
///
/// * `source` - The H2 source code to validate
///
/// # Returns
///
/// A [`JsValue`] containing a unified response format:
/// - On success: `{ "status": "ok", "valid": true }`
/// - On error: `{ "status": "error", "errors": [ ... ] }`
///
/// # Example (JavaScript)
///
/// ```javascript
/// const result = validate('0: srl');
/// if (result.status === 'ok') {
///     console.log('Syntax is valid');
/// } else {
///     console.error('Errors:', result.errors);
/// }
/// ```
#[wasm_bindgen]
pub fn validate(source: &str) -> JsValue {
    let mut parser = match Parser::new(source) {
        Ok(p) => p,
        Err(e) => {
            return serde_wasm_bindgen::to_value(&CompileResult::Error {
                errors: vec![e.into()],
            })
            .unwrap_or(JsValue::NULL)
        }
    };

    match parser.parse_program() {
        Ok(_) => {
            let result = serde_json::json!({ "status": "ok", "valid": true });
            serde_wasm_bindgen::to_value(&result).unwrap_or(JsValue::NULL)
        }
        Err(e) => serde_wasm_bindgen::to_value(&CompileResult::Error {
            errors: vec![e.into()],
        })
        .unwrap_or(JsValue::NULL),
    }
}

/// Retrieves a specific step from a compiled program.
///
/// Extracts timeline information for a specific execution step, useful for
/// step-by-step visualization or debugging.
///
/// # Arguments
///
/// * `program_json` - JSON string of a [`CompileResult`]
/// * `step` - Step number (0-based index)
///
/// # Returns
///
/// A [`JsValue`] containing the [`TimelineEntry`](output::TimelineEntry) for
/// the specified step, or `null` if the step doesn't exist.
///
/// # Example (JavaScript)
///
/// ```javascript
/// const compiled = compile('0: srl');
/// const step0 = get_step(JSON.stringify(compiled), 0);
/// console.log('Step 0 commands:', step0.agent_commands);
/// ```
#[wasm_bindgen]
pub fn get_step(program_json: &str, step: usize) -> JsValue {
    let result: Result<CompileResult, _> = serde_json::from_str(program_json);

    match result {
        Ok(CompileResult::Success { program }) => {
            if let Some(entry) = program.timeline.get(step) {
                serde_wasm_bindgen::to_value(entry).unwrap_or(JsValue::NULL)
            } else {
                JsValue::NULL
            }
        }
        _ => JsValue::NULL,
    }
}

/// Returns the version of the H2 Language compiler.
///
/// # Returns
///
/// The version string from `Cargo.toml` (e.g., "0.1.0").
///
/// # Example (JavaScript)
///
/// ```javascript
/// console.log('H2 Language Compiler v' + version());
/// ```
#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

// =============================================================================
// Native Rust API
// =============================================================================

/// Compiles H2 source code and returns the result (native Rust API).
///
/// This is the main entry point for native Rust usage. Unlike the WASM
/// [`compile`] function, this returns a strongly-typed [`CompileResult`]
/// directly.
///
/// # Arguments
///
/// * `source` - The H2 source code to compile
///
/// # Returns
///
/// A [`CompileResult`] enum containing either:
/// - `Success { program }` - The compiled program with timeline
/// - `Error { errors }` - A list of compilation errors
///
/// # Examples
///
/// ## Basic Compilation
///
/// ```rust
/// use h2lang::compile_native;
/// use h2lang::output::CompileResult;
///
/// let result = compile_native("0: srl");
/// match result {
///     CompileResult::Success { program } => {
///         assert_eq!(program.agents.len(), 1);
///         assert_eq!(program.max_steps, 3);
///     }
///     CompileResult::Error { errors } => {
///         panic!("Compilation failed: {:?}", errors);
///     }
/// }
/// ```
///
/// ## Multiple Agents
///
/// ```rust
/// use h2lang::compile_native;
/// use h2lang::output::CompileResult;
///
/// let result = compile_native("0: srl\n1: lrs");
/// if let CompileResult::Success { program } = result {
///     assert_eq!(program.agents.len(), 2);
///     // Timeline contains parallel execution info
///     assert_eq!(program.timeline[0].agent_commands.len(), 2);
/// }
/// ```
///
/// ## Using Macros and Functions
///
/// ```rust
/// use h2lang::compile_native;
/// use h2lang::output::CompileResult;
///
/// // Define macro 'x' and function 'f', then use them
/// let source = "0: x:ss f(X):XXX xf(r)x";
/// let result = compile_native(source);
///
/// if let CompileResult::Success { program } = result {
///     // x = ss, f(r) = rrr
///     // xf(r)x = ss + rrr + ss = ssrrrss (7 commands)
///     assert_eq!(program.agents[0].commands.len(), 7);
/// }
/// ```
///
/// # Errors
///
/// Returns `CompileResult::Error` for:
/// - **Lexer errors**: Invalid characters, malformed tokens
/// - **Parser errors**: Syntax errors, unexpected tokens
/// - **Expansion errors**: Undefined macros/functions, infinite recursion
pub fn compile_native(source: &str) -> CompileResult {
    compile_internal(source)
}

/// Internal compilation implementation shared by WASM and native APIs.
///
/// # Pipeline
///
/// 1. **Lexing**: Source → Tokens
/// 2. **Parsing**: Tokens → AST
/// 3. **Expansion**: AST → Expanded Commands (macro/function resolution)
/// 4. **Scheduling**: Commands → Parallel Timeline
/// 5. **Output**: Timeline → JSON-serializable structures
fn compile_internal(source: &str) -> CompileResult {
    // Phase 1: Parse source code into AST
    let mut parser = match Parser::new(source) {
        Ok(p) => p,
        Err(e) => {
            return CompileResult::Error {
                errors: vec![e.into()],
            }
        }
    };

    let program = match parser.parse_program() {
        Ok(p) => p,
        Err(e) => {
            return CompileResult::Error {
                errors: vec![e.into()],
            }
        }
    };

    // Phase 2: Expand macros and functions for each agent
    let expander = Expander::new();
    let mut expanded_agents = Vec::new();

    for agent in &program.agents {
        match expander.expand_agent(agent) {
            Ok(commands) => {
                expanded_agents.push((agent.id, commands));
            }
            Err(e) => {
                return CompileResult::Error {
                    errors: vec![e.into()],
                }
            }
        }
    }

    // Phase 3: Schedule parallel execution across agents
    let timeline = Scheduler::schedule(&expanded_agents);

    // Phase 4: Convert to JSON-serializable output format
    let compiled = CompiledProgram::from_expanded(&expanded_agents, timeline);

    CompileResult::Success { program: compiled }
}

// =============================================================================
// Unit Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_simple() {
        let result = compile_internal("0: srl");
        match result {
            CompileResult::Success { program } => {
                assert_eq!(program.agents.len(), 1);
                assert_eq!(program.agents[0].id, 0);
                assert_eq!(program.agents[0].commands.len(), 3);
                assert_eq!(program.max_steps, 3);
                assert_eq!(program.timeline.len(), 3);
            }
            CompileResult::Error { errors } => {
                panic!("Compilation failed: {:?}", errors);
            }
        }
    }

    #[test]
    fn test_compile_multiple_agents() {
        let result = compile_internal("0: srl\n1: lrs");
        match result {
            CompileResult::Success { program } => {
                assert_eq!(program.agents.len(), 2);
                assert_eq!(program.max_steps, 3);
                // Each step should have 2 agent commands
                assert_eq!(program.timeline[0].agent_commands.len(), 2);
            }
            CompileResult::Error { errors } => {
                panic!("Compilation failed: {:?}", errors);
            }
        }
    }

    #[test]
    fn test_compile_with_macro() {
        let result = compile_internal("0: x:ss xrx");
        match result {
            CompileResult::Success { program } => {
                // x:ss xrx -> ssrss (5 commands)
                assert_eq!(program.agents[0].commands.len(), 5);
            }
            CompileResult::Error { errors } => {
                panic!("Compilation failed: {:?}", errors);
            }
        }
    }

    #[test]
    fn test_compile_with_function() {
        let result = compile_internal("0: f(X):XXX f(s)");
        match result {
            CompileResult::Success { program } => {
                // f(X):XXX f(s) -> sss (3 commands)
                assert_eq!(program.agents[0].commands.len(), 3);
            }
            CompileResult::Error { errors } => {
                panic!("Compilation failed: {:?}", errors);
            }
        }
    }

    #[test]
    fn test_compile_nested_function() {
        let result = compile_internal("0: f(X):XX f(f(s))");
        match result {
            CompileResult::Success { program } => {
                // f(X):XX f(f(s)) -> f(ss) -> ssss (4 commands)
                assert_eq!(program.agents[0].commands.len(), 4);
            }
            CompileResult::Error { errors } => {
                panic!("Compilation failed: {:?}", errors);
            }
        }
    }

    #[test]
    fn test_compile_error_undefined_macro() {
        let result = compile_internal("0: x");
        match result {
            CompileResult::Success { .. } => {
                panic!("Expected error for undefined macro");
            }
            CompileResult::Error { errors } => {
                assert!(!errors.is_empty());
                assert!(errors[0].message.contains("Undefined macro"));
            }
        }
    }

    #[test]
    fn test_compile_error_syntax() {
        let result = compile_internal("0: (");
        match result {
            CompileResult::Success { .. } => {
                panic!("Expected syntax error");
            }
            CompileResult::Error { errors } => {
                assert!(!errors.is_empty());
            }
        }
    }

    #[test]
    fn test_timeline_parallel_execution() {
        let result = compile_internal("0: srl\n1: lrs");
        match result {
            CompileResult::Success { program } => {
                // Step 0: agent 0 -> s, agent 1 -> l
                assert_eq!(program.timeline[0].step, 0);
                let step0_cmds: Vec<_> = program.timeline[0]
                    .agent_commands
                    .iter()
                    .map(|c| (c.agent_id, &c.command.command_type))
                    .collect();

                // Both agents should have commands at step 0
                assert!(step0_cmds.iter().any(|(id, _)| *id == 0));
                assert!(step0_cmds.iter().any(|(id, _)| *id == 1));
            }
            CompileResult::Error { errors } => {
                panic!("Compilation failed: {:?}", errors);
            }
        }
    }

    #[test]
    fn test_complex_hoj_example() {
        // From HOJ tutorial: square pattern
        let result = compile_internal("0: f(X):XXXX f(sssr)");
        match result {
            CompileResult::Success { program } => {
                // f(X):XXXX f(sssr) -> sssrsssrsssrsssr (16 commands)
                assert_eq!(program.agents[0].commands.len(), 16);
            }
            CompileResult::Error { errors } => {
                panic!("Compilation failed: {:?}", errors);
            }
        }
    }
}
