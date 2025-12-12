//! H2 Language Specification v0.3.1 Conformance Tests
//!
//! These 18 tests verify conformance with the H2 Language Specification.
//! See docs/SPEC.md for the full specification.
//!
//! Test naming convention: t{NN}_{description}
//!
//! Note: Some tests are marked #[ignore] until the corresponding feature
//! (directives, type checking, etc.) is implemented.

use h2lang::compile_native;
use h2lang::output::CompileResult;

// =============================================================================
// Test Helpers
// =============================================================================

/// Compile source and return command sequence as string for agent 0
fn compile_to_string(source: &str) -> Result<String, String> {
    match compile_native(source) {
        CompileResult::Success { program } => {
            if program.agents.is_empty() {
                return Ok(String::new());
            }
            let commands: String = program.agents[0]
                .commands
                .iter()
                .map(|c| match c.command_type {
                    h2lang::output::CommandType::Straight => 's',
                    h2lang::output::CommandType::RotateRight => 'r',
                    h2lang::output::CommandType::RotateLeft => 'l',
                    h2lang::output::CommandType::Wait => 'w',
                })
                .collect();
            Ok(commands)
        }
        CompileResult::Error { errors } => {
            let msg = errors
                .iter()
                .map(|e| e.message.clone())
                .collect::<Vec<_>>()
                .join("; ");
            Err(msg)
        }
    }
}

/// Check if error message contains expected error code pattern
fn error_contains(result: &Result<String, String>, pattern: &str) -> bool {
    match result {
        Err(msg) => msg.to_lowercase().contains(&pattern.to_lowercase()),
        Ok(_) => false,
    }
}

/// Assert successful compilation with expected output
fn assert_compiles_to(source: &str, expected: &str, test_id: &str) {
    let result = compile_to_string(source);
    match result {
        Ok(actual) => {
            assert_eq!(
                actual, expected,
                "[{}] Expected '{}', got '{}'",
                test_id, expected, actual
            );
        }
        Err(e) => {
            panic!(
                "[{}] Expected success '{}', got error: {}",
                test_id, expected, e
            );
        }
    }
}

/// Assert compilation error
fn assert_compile_error(source: &str, test_id: &str) {
    let result = compile_to_string(source);
    assert!(
        result.is_err(),
        "[{}] Expected error, got success: {:?}",
        test_id,
        result
    );
}

// =============================================================================
// T01-T03: Basic Commands and Macros
// =============================================================================

/// T01: Simple commands
/// Input: srl
/// Expected: Agent0 = srl
#[test]
fn t01_simple_commands() {
    assert_compiles_to("srl", "srl", "T01");
}

/// T02: Macro definition and usage
/// Input: a:ssss\naa
/// Expected: Agent0 = ssssssss
#[test]
fn t02_macro() {
    assert_compiles_to("a:ssss\naa", "ssssssss", "T02");
}

/// T03: Undefined macro reference
/// Input: b
/// Expected: Error E001
#[test]
fn t03_undefined_macro() {
    assert_compile_error("b", "T03");
    let result = compile_to_string("b");
    assert!(
        error_contains(&result, "undefined") || error_contains(&result, "macro"),
        "[T03] Error should mention undefined macro"
    );
}

// =============================================================================
// T04-T06: Function Basics
// =============================================================================

/// T04: Function with CmdSeq argument
/// Input: f(X):XXX\nf(sr)
/// Expected: Agent0 = srsrsr
#[test]
fn t04_function_cmdseq() {
    assert_compiles_to("f(X):XXX\nf(sr)", "srsrsr", "T04");
}

/// T05: Undefined function reference
/// Input: f(s)
/// Expected: Error E002
#[test]
fn t05_undefined_function() {
    assert_compile_error("f(s)", "T05");
    let result = compile_to_string("f(s)");
    assert!(
        error_contains(&result, "undefined") || error_contains(&result, "function"),
        "[T05] Error should mention undefined function"
    );
}

/// T06: Argument count mismatch
/// Input: f(X,Y):XY\nf(s)
/// Expected: Error E003
#[test]
fn t06_argument_count_mismatch() {
    assert_compile_error("f(X,Y):XY\nf(s)", "T06");
    let result = compile_to_string("f(X,Y):XY\nf(s)");
    assert!(
        error_contains(&result, "E003"),
        "[T06] Error should mention E003"
    );
}

// =============================================================================
// T07-T09: Numeric Counter Recursion
// =============================================================================

/// T07: Numeric counter recursion (termination)
/// Input: a(X):sa(X-1)\na(4)
/// Expected: Agent0 = ssss
#[test]
fn t07_numeric_counter_recursion() {
    assert_compiles_to("a(X):sa(X-1)\na(4)", "ssss", "T07");
}

/// T08: Numeric counter with 0 (immediate empty)
/// Input: a(X):sa(X-1)\na(0)
/// Expected: Agent0 = (empty)
#[test]
fn t08_numeric_counter_zero() {
    assert_compiles_to("a(X):sa(X-1)\na(0)", "", "T08");
}

/// T09: Numeric counter with negative (immediate empty)
/// Input: a(X):sa(X-1)\na(-1)
/// Expected: Agent0 = (empty)
#[test]
fn t09_numeric_counter_negative() {
    assert_compiles_to("a(X):sa(X-1)\na(-1)", "", "T09");
}

// =============================================================================
// T10-T12: Multiple Arguments and Numeric Expressions
// =============================================================================

/// T10: Multiple arguments (Int + CmdSeq + CmdSeq)
/// Input: a(N,X,Y):XYa(N-1,X,Y)\na(3,s,r)
/// Expected: Agent0 = srsrsr
///
/// Note: N is Int type (counter), X and Y are CmdSeq type
#[test]
fn t10_multiple_args() {
    assert_compiles_to("a(N,X,Y):XYa(N-1,X,Y)\na(3,s,r)", "srsrsr", "T10");
}

/// T11: Numeric argument (simple)
/// Input: a(X):sa(X-1)\na(2)
/// Expected: Agent0 = ss
#[test]
fn t11_numeric_argument() {
    assert_compiles_to("a(X):sa(X-1)\na(2)", "ss", "T11");
}

/// T12: Numeric value out of range
/// Input: a(X):s\na(256)
/// Expected: Error E007
#[test]
fn t12_numeric_range_exceeded() {
    assert_compile_error("a(X):s\na(256)", "T12");
    let result = compile_to_string("a(X):s\na(256)");
    assert!(
        error_contains(&result, "E007"),
        "[T12] Error should mention E007"
    );
}

// =============================================================================
// T13-T14: Type Errors
// =============================================================================

/// T13: Type error - Int type PARAM used as term
/// Input: f(X):X\nf(3)
/// Expected: Error E008
///
/// X is bound to Int (3), but used as term in expression (requires CmdSeq)
#[test]
fn t13_type_error_int_as_term() {
    assert_compile_error("f(X):X\nf(3)", "T13");
    let result = compile_to_string("f(X):X\nf(3)");
    assert!(
        error_contains(&result, "E008"),
        "[T13] Error should mention E008"
    );
}

/// T14: Type error - CmdSeq type PARAM used in num_expr
/// Input: a(X):a(X-1)\na(sr)
/// Expected: Error E008
///
/// X is bound to CmdSeq (sr), but X-1 requires Int type
#[test]
fn t14_type_error_cmdseq_in_numexpr() {
    assert_compile_error("a(X):a(X-1)\na(sr)", "T14");
    let result = compile_to_string("a(X):a(X-1)\na(sr)");
    assert!(
        error_contains(&result, "E008"),
        "[T14] Error should mention E008"
    );
}

// =============================================================================
// T15-T17: Directives
// =============================================================================

/// T15: Unknown directive
/// Input: MAX_STEPS=10\ns
/// Expected: Error E009
///
/// Note: MAX_STEPS (with S) is invalid, should be MAX_STEP
#[test]
fn t15_unknown_directive() {
    assert_compile_error("MAX_STEPS=10\ns", "T15");
    let result = compile_to_string("MAX_STEPS=10\ns");
    assert!(
        error_contains(&result, "E009") || error_contains(&result, "unknown"),
        "[T15] Error should mention E009 or unknown directive"
    );
}

/// T16: MAX_STEP exceeded with ON_LIMIT=ERROR (default)
/// Input: MAX_STEP=3\na(X):sa(X-1)\na(10)
/// Expected: Error E004
#[test]
fn t16_max_step_exceeded_error() {
    assert_compile_error("MAX_STEP=3\na(X):sa(X-1)\na(10)", "T16");
    let result = compile_to_string("MAX_STEP=3\na(X):sa(X-1)\na(10)");
    assert!(
        error_contains(&result, "E004") || error_contains(&result, "MAX_STEP"),
        "[T16] Error should mention E004 or MAX_STEP"
    );
}

/// T17: MAX_STEP exceeded with ON_LIMIT=TRUNCATE
/// Input: MAX_STEP=3\nON_LIMIT=TRUNCATE\na(X):sa(X-1)\na(10)
/// Expected: Agent0 = sss
#[test]
fn t17_max_step_exceeded_truncate() {
    assert_compiles_to(
        "MAX_STEP=3\nON_LIMIT=TRUNCATE\na(X):sa(X-1)\na(10)",
        "sss",
        "T17",
    );
}

// =============================================================================
// T18: Multi-Agent
// =============================================================================

/// T18: Multi-agent (expansion results + timeline length)
/// Input: 0: srl\n1: ss\n2: l
/// Expected:
///   - Agent0 = srl
///   - Agent1 = ss
///   - Agent2 = l
///   - Timeline length = 3 (longest is Agent0 with 3)
#[test]
fn t18_multi_agent() {
    let result = compile_native("0: srl\n1: ss\n2: l");

    match result {
        CompileResult::Success { program } => {
            // Check agent count
            assert_eq!(program.agents.len(), 3, "[T18] Expected 3 agents");

            // Check Agent 0
            let agent0: String = program.agents[0]
                .commands
                .iter()
                .map(|c| match c.command_type {
                    h2lang::output::CommandType::Straight => 's',
                    h2lang::output::CommandType::RotateRight => 'r',
                    h2lang::output::CommandType::RotateLeft => 'l',
                    h2lang::output::CommandType::Wait => 'w',
                })
                .collect();
            assert_eq!(agent0, "srl", "[T18] Agent0 should be 'srl'");

            // Check Agent 1
            let agent1: String = program.agents[1]
                .commands
                .iter()
                .map(|c| match c.command_type {
                    h2lang::output::CommandType::Straight => 's',
                    h2lang::output::CommandType::RotateRight => 'r',
                    h2lang::output::CommandType::RotateLeft => 'l',
                    h2lang::output::CommandType::Wait => 'w',
                })
                .collect();
            assert_eq!(agent1, "ss", "[T18] Agent1 should be 'ss'");

            // Check Agent 2
            let agent2: String = program.agents[2]
                .commands
                .iter()
                .map(|c| match c.command_type {
                    h2lang::output::CommandType::Straight => 's',
                    h2lang::output::CommandType::RotateRight => 'r',
                    h2lang::output::CommandType::RotateLeft => 'l',
                    h2lang::output::CommandType::Wait => 'w',
                })
                .collect();
            assert_eq!(agent2, "l", "[T18] Agent2 should be 'l'");

            // Check timeline length
            assert_eq!(
                program.max_steps, 3,
                "[T18] Timeline length should be 3 (max of 3, 2, 1)"
            );
        }
        CompileResult::Error { errors } => {
            panic!(
                "[T18] Expected success, got error: {:?}",
                errors.iter().map(|e| &e.message).collect::<Vec<_>>()
            );
        }
    }
}

// =============================================================================
// Additional Conformance Tests (Edge Cases)
// =============================================================================

/// Additional: Empty program
#[test]
fn additional_empty_program() {
    let result = compile_native("");
    match result {
        CompileResult::Success { program } => {
            assert!(
                program.agents.is_empty(),
                "Empty program should have no agents"
            );
        }
        CompileResult::Error { .. } => {
            // Empty program may be an error in some implementations
        }
    }
}

/// Additional: Single agent mode (no prefix)
#[test]
fn additional_single_agent_no_prefix() {
    assert_compiles_to("srl", "srl", "Additional-NoPrefix");
}

/// Additional: Multiline single agent
#[test]
fn additional_multiline_single_agent() {
    assert_compiles_to("a:ss\naaa", "ssssss", "Additional-Multiline");
}

/// Additional: Nested function calls
#[test]
fn additional_nested_function() {
    assert_compiles_to("f(X):XX\nf(f(s))", "ssss", "Additional-Nested");
}

/// Additional: Deep recursion (within default limit)
#[test]
fn additional_deep_recursion() {
    // a(50) should produce 50 s's
    assert_compiles_to(
        "a(X):sa(X-1)\na(50)",
        &"s".repeat(50),
        "Additional-DeepRecursion",
    );
}
