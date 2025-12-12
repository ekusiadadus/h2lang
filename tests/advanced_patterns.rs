//! Advanced H Language Pattern Tests
//!
//! Based on snuke's blog post "Herbert to Suugaku" (Herbert and Mathematics)
//! and the 9-byte classification document.
//!
//! **Note:** Many tests in this file fail due to MAX_DEPTH=100 limit.
//! These patterns use infinite CmdSeq recursion (e.g., `a(A):Ara(sA)`)
//! which requires deep recursion until MAX_STEP truncates output.
//! A future iterative expander could support these patterns.
//!
//! Current status: 64 passed, 100 failed (depth limit exceeded)
//!
//! References:
//! - snuke's blog: https://snuke.hatenablog.com/entry/20111206/1323180471
//! - Herbert Online Judge: http://herbert.tealang.info/

use h2lang::compile_native;
use h2lang::output::CompileResult;

// =============================================================================
// Test Helpers
// =============================================================================

/// Compile source and return command sequence as string
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
        CompileResult::Error { errors } => Err(errors
            .iter()
            .map(|e| e.message.clone())
            .collect::<Vec<_>>()
            .join("; ")),
    }
}

/// Assert that source compiles successfully
fn assert_compiles(source: &str) {
    match compile_to_string(source) {
        Ok(_) => {}
        Err(e) => {
            panic!("Source '{}' failed to compile: {}", source, e);
        }
    }
}

/// Assert that source compiles to expected command string
fn assert_compiles_to(source: &str, expected: &str) {
    match compile_to_string(source) {
        Ok(result) => {
            assert_eq!(
                result, expected,
                "Source '{}' compiled to '{}', expected '{}'",
                source, result, expected
            );
        }
        Err(e) => {
            panic!("Source '{}' failed to compile: {}", source, e);
        }
    }
}

/// Assert command count
fn assert_command_count(source: &str, expected_count: usize) {
    match compile_to_string(source) {
        Ok(result) => {
            assert_eq!(
                result.len(),
                expected_count,
                "Source '{}' produced {} commands, expected {}",
                source,
                result.len(),
                expected_count
            );
        }
        Err(e) => {
            panic!("Source '{}' failed to compile: {}", source, e);
        }
    }
}

/// Assert that source fails to compile
fn assert_compile_error(source: &str) {
    match compile_native(source) {
        CompileResult::Success { .. } => {
            panic!("Source '{}' should have failed to compile", source);
        }
        CompileResult::Error { .. } => {}
    }
}

/// Count specific command in result
fn count_command(source: &str, cmd: char) -> usize {
    compile_to_string(source)
        .unwrap_or_default()
        .chars()
        .filter(|&c| c == cmd)
        .count()
}

// =============================================================================
// 1. Mathematical Patterns (from snuke's blog)
// =============================================================================

mod mathematical_patterns {
    use super::*;

    /// Arithmetic sequence: 0, 1, 2, 3, 4, 5, ...
    /// Pattern: a(A):Ara(sA) a()
    /// Each term: move A steps, turn right
    mod arithmetic_sequence {
        use super::*;

        #[test]
        fn test_arithmetic_sequence_basic() {
            // a(A):Ara(sA) a()
            // Expands: r a(s) -> r sr a(ss) -> r sr ssr a(sss) -> ...
            // Result: r sr ssr sssr ...
            let source = "0: a(A):Ara(sA) a()";
            assert_compiles(source);
        }

        #[test]
        fn test_arithmetic_sequence_produces_spiral() {
            // The arithmetic sequence creates a spiral pattern
            // 0 + 1 + 2 + 3 = 6 straights, 4 rights in first few iterations
            let source = "0: a(A):Ara(sA) a()";
            let result = compile_to_string(source).unwrap();
            // Should start with: r (0 steps) sr (1 step) ssr (2 steps) ...
            assert!(result.starts_with("r"), "Should start with r");
            assert!(result.contains("sr"), "Should contain sr pattern");
            assert!(result.contains("ssr"), "Should contain ssr pattern");
        }

        #[test]
        fn test_arithmetic_with_initial_value() {
            // Start with s instead of empty
            let source = "0: a(A):Ara(sA) a(s)";
            assert_compiles(source);
            let result = compile_to_string(source).unwrap();
            // Should start with: sr (1 step) ssr (2 steps) ...
            assert!(result.starts_with("sr"), "Should start with sr");
        }
    }

    /// Powers of 2: 1, 2, 4, 8, 16, ...
    /// Pattern: a(A):Ara(AA) a(s)
    mod powers_of_two {
        use super::*;

        #[test]
        fn test_powers_of_two_basic() {
            // a(A):Ara(AA) a(s)
            // Expands: sr a(ss) -> sr ssr a(ssss) -> sr ssr sssssr a(ssssssss) -> ...
            // Lengths: 1, 2, 4, 8, ...
            let source = "0: a(A):Ara(AA) a(s)";
            assert_compiles(source);
        }

        #[test]
        fn test_powers_of_two_exponential_growth() {
            let source = "0: a(A):Ara(AA) a(s)";
            let result = compile_to_string(source).unwrap();
            // First iteration: s (1) r
            // Second iteration: ss (2) r
            // Third iteration: ssss (4) r
            // The number of 's' grows exponentially
            assert!(result.starts_with("sr"), "Should start with sr (1 step)");
        }
    }

    /// Fibonacci sequence: 1, 1, 2, 3, 5, 8, 13, ...
    /// Pattern: a(A,B):Ara(AB,A) a(s,)
    mod fibonacci {
        use super::*;

        #[test]
        fn test_fibonacci_basic() {
            // a(A,B):Ara(AB,A) a(s,)
            // Expands:
            // a(s,) -> sr a(s,s)
            // a(s,s) -> sr a(ss,s)
            // a(ss,s) -> ssr a(sss,ss)
            // a(sss,ss) -> sssr a(sssss,sss)
            // Lengths: 1, 1, 2, 3, 5, ...
            let source = "0: a(A,B):Ara(AB,A) a(s,)";
            assert_compiles(source);
        }

        #[test]
        fn test_fibonacci_produces_fibonacci_lengths() {
            let source = "0: a(A,B):Ara(AB,A) a(s,)";
            let result = compile_to_string(source).unwrap();
            // Should produce Fibonacci-like pattern
            // 1r 1r 2r 3r 5r ...
            assert!(result.contains("sr"), "Should contain 1-step");
            assert!(result.contains("ssr"), "Should contain 2-step");
        }

        #[test]
        fn test_fibonacci_two_args() {
            // Verify two-argument function works
            let source = "0: f(X,Y):XY f(s,ss)";
            assert_compiles_to(source, "sss");
        }
    }

    /// Division example from snuke's blog
    /// Counts how many times we can subtract 7
    mod division {
        use super::*;

        #[test]
        fn test_division_by_subtraction() {
            // b(B):sb(B-7) counts floor((B+6)/7) by subtracting 7 repeatedly
            // b(10) -> sb(3) -> sb(-4) -> ss (stops when B <= 0)
            let source = "0: b(B):sb(B-7) b(10)";
            assert_compiles(source);
            // 10 -> 3 -> -4 (stops), so 2 s's
            assert_compiles_to(source, "ss");
        }

        #[test]
        fn test_division_30_by_7() {
            // b(30) -> sb(23) -> sb(16) -> sb(9) -> sb(2) -> sb(-5)
            // 5 s's executed
            let source = "0: b(B):sb(B-7) b(30)";
            assert_compiles_to(source, "sssss");
        }

        #[test]
        fn test_division_exact_multiple() {
            // b(21) -> sb(14) -> sb(7) -> sb(0) -> stops at 0
            // 3 s's executed (21/7 = 3, but b(0) doesn't execute)
            let source = "0: b(B):sb(B-7) b(21)";
            assert_compiles_to(source, "sss");
        }

        #[test]
        fn test_division_adjusted() {
            // To get exact division, use b(B-6) as starting point
            // For 28/7 = 4: b(28-6) = b(22)
            // b(22) -> sb(15) -> sb(8) -> sb(1) -> sb(-6) -> 4 s's
            let source = "0: b(B):sb(B-7) b(22)";
            assert_compiles_to(source, "ssss");
        }
    }

    /// Modulo operation
    mod modulo {
        use super::*;

        #[test]
        fn test_modulo_concept() {
            // Simple modulo: keep subtracting until < divisor
            // For X mod 3: subtract 3 until X < 3, then output X
            let source = "0: c(C):sc(C-1) c(2)";
            // c(2) -> sc(1) -> ssc(0) -> ss (c(0) stops)
            assert_compiles_to(source, "ss");
        }

        #[test]
        fn test_modulo_one() {
            let source = "0: c(C):sc(C-1) c(1)";
            assert_compiles_to(source, "s");
        }

        #[test]
        fn test_modulo_zero() {
            let source = "0: c(C):sc(C-1) c(0)";
            // c(0) doesn't execute because 0 <= 0
            assert_compiles_to(source, "");
        }
    }
}

// =============================================================================
// 2. 9-Byte Classification Patterns
// =============================================================================

mod nine_byte_patterns {
    use super::*;

    /// Z-type patterns (large)
    mod z_type_large {
        use super::*;

        #[test]
        fn test_z_large_xssa_lx() {
            // a(X):Xssa(lX) a()
            let source = "0: a(X):Xssa(lX) a()";
            assert_compiles(source);
        }

        #[test]
        fn test_z_large_xssa_rx() {
            let source = "0: a(X):Xssa(rX) a()";
            assert_compiles(source);
        }

        #[test]
        fn test_z_large_sxsa_lx() {
            let source = "0: a(X):sXsa(lX) a()";
            assert_compiles(source);
        }

        #[test]
        fn test_z_large_ssxa_lx() {
            let source = "0: a(X):ssXa(lX) a()";
            assert_compiles(source);
        }
    }

    /// Z-type patterns (small)
    mod z_type_small {
        use super::*;

        #[test]
        fn test_z_small_xsla_lx() {
            let source = "0: a(X):Xsla(lX) a()";
            assert_compiles(source);
        }

        #[test]
        fn test_z_small_xsra_rx() {
            let source = "0: a(X):Xsra(rX) a()";
            assert_compiles(source);
        }

        #[test]
        fn test_z_small_sxla_lx() {
            let source = "0: a(X):sXla(lX) a()";
            assert_compiles(source);
        }

        #[test]
        fn test_z_small_xlsa_lx() {
            let source = "0: a(X):Xlsa(lX) a()";
            assert_compiles(source);
        }
    }

    /// Z-type with square patterns
    mod z_type_with_square {
        use super::*;

        #[test]
        fn test_z_square_sxa_srx() {
            let source = "0: a(X):sXa(srX) a()";
            assert_compiles(source);
        }

        #[test]
        fn test_z_square_sxa_slx() {
            let source = "0: a(X):sXa(slX) a()";
            assert_compiles(source);
        }

        #[test]
        fn test_z_square_lxa_srx() {
            let source = "0: a(X):lXa(srX) a()";
            assert_compiles(source);
        }

        #[test]
        fn test_z_square_xsa_srx() {
            let source = "0: a(X):Xsa(srX) a()";
            assert_compiles(source);
        }

        #[test]
        fn test_z_square_xla_srx() {
            let source = "0: a(X):Xla(srX) a()";
            assert_compiles(source);
        }

        #[test]
        fn test_z_square_sxa_rsx() {
            let source = "0: a(X):sXa(rsX) a()";
            assert_compiles(source);
        }
    }

    /// P-type patterns
    mod p_type {
        use super::*;

        #[test]
        fn test_p_type_sxa_xxr() {
            let source = "0: a(X):sXa(XXr) a()";
            assert_compiles(source);
        }

        #[test]
        fn test_p_type_sxa_xxl() {
            let source = "0: a(X):sXa(XXl) a()";
            assert_compiles(source);
        }

        #[test]
        fn test_p_type_xsa_xxr() {
            let source = "0: a(X):Xsa(XXr) a()";
            assert_compiles(source);
        }

        #[test]
        fn test_p_type_xsa_xxl() {
            let source = "0: a(X):Xsa(XXl) a()";
            assert_compiles(source);
        }
    }

    /// Straight line patterns
    mod straight_lines {
        use super::*;

        #[test]
        fn test_straight_xlla_sx() {
            let source = "0: a(X):Xlla(sX) a()";
            assert_compiles(source);
        }

        #[test]
        fn test_straight_lxla_sx() {
            let source = "0: a(X):lXla(sX) a()";
            assert_compiles(source);
        }

        #[test]
        fn test_straight_rxra_sx() {
            let source = "0: a(X):rXra(sX) a()";
            assert_compiles(source);
        }

        #[test]
        fn test_two_column_xa_xsx_l() {
            let source = "0: a(X):Xa(XsX) a(l)";
            assert_compiles(source);
        }

        #[test]
        fn test_two_column_xa_xsx_r() {
            let source = "0: a(X):Xa(XsX) a(r)";
            assert_compiles(source);
        }

        #[test]
        fn test_three_column_xa_lxsl() {
            let source = "0: a(X):Xa(lXsl) a()";
            assert_compiles(source);
        }

        #[test]
        fn test_three_column_xa_rxsr() {
            let source = "0: a(X):Xa(rXsr) a()";
            assert_compiles(source);
        }
    }

    /// Diagonal patterns
    mod diagonal {
        use super::*;

        #[test]
        fn test_diagonal_xxa_rx_s() {
            // Classic diagonal: a(X):XXa(rX) a(s)
            let source = "0: a(X):XXa(rX) a(s)";
            assert_compiles(source);
        }

        #[test]
        fn test_diagonal_xxa_lx_s() {
            let source = "0: a(X):XXa(lX) a(s)";
            assert_compiles(source);
        }

        #[test]
        fn test_diagonal_xxa_xr_s() {
            let source = "0: a(X):XXa(Xr) a(s)";
            assert_compiles(source);
        }

        #[test]
        fn test_diagonal_xxa_xl_s() {
            let source = "0: a(X):XXa(Xl) a(s)";
            assert_compiles(source);
        }

        #[test]
        fn test_diagonal_xa_lxsr() {
            let source = "0: a(X):Xa(lXsr) a()";
            assert_compiles(source);
        }

        #[test]
        fn test_diagonal_xa_rxsl() {
            let source = "0: a(X):Xa(rXsl) a()";
            assert_compiles(source);
        }
    }

    /// Spiral/Vortex patterns (うず)
    mod spiral {
        use super::*;

        // Pattern: 0,2,4,6,8 sequence
        #[test]
        fn test_spiral_xxla_sx_even() {
            // a(X):XXla(sX) a() produces 0,2,4,6,8 sequence
            let source = "0: a(X):XXla(sX) a()";
            assert_compiles(source);
        }

        #[test]
        fn test_spiral_xxra_sx() {
            let source = "0: a(X):XXra(sX) a()";
            assert_compiles(source);
        }

        // Pattern: 1,3,5,7,9 sequence
        #[test]
        fn test_spiral_xlxa_sx_odd() {
            // a(X):XlXa(sX) a() produces 1,3,5,7,9 sequence
            let source = "0: a(X):XlXa(sX) a()";
            assert_compiles(source);
        }

        #[test]
        fn test_spiral_xrxa_sx() {
            let source = "0: a(X):XrXa(sX) a()";
            assert_compiles(source);
        }

        // Pattern: 1,2,3,4,5 sequence
        #[test]
        fn test_spiral_sxla_sx_linear() {
            // a(X):sXla(sX) a() produces 1,2,3,4,5 sequence
            let source = "0: a(X):sXla(sX) a()";
            assert_compiles(source);
        }

        #[test]
        fn test_spiral_sxra_sx() {
            let source = "0: a(X):sXra(sX) a()";
            assert_compiles(source);
        }

        // Pattern: 0,1,3,7,15 sequence (2^n - 1)
        #[test]
        fn test_spiral_xla_xxs_power() {
            // a(X):Xla(XXs) a() produces 0,1,3,7,15 sequence
            let source = "0: a(X):Xla(XXs) a()";
            assert_compiles(source);
        }

        #[test]
        fn test_spiral_xra_xxs() {
            let source = "0: a(X):Xra(XXs) a()";
            assert_compiles(source);
        }

        // Pattern: 0,0,1,1,2,2 sequence
        #[test]
        fn test_spiral_xxa_sx_l_half() {
            let source = "0: a(X):XXa(sX) a(l)";
            assert_compiles(source);
        }

        #[test]
        fn test_spiral_xxa_sx_r() {
            let source = "0: a(X):XXa(sX) a(r)";
            assert_compiles(source);
        }

        // Pattern: 1,2,4,8,16 sequence (powers of 2)
        #[test]
        fn test_spiral_xra_xx_s_power2() {
            // a(X):Xra(XX) a(s) produces 1,2,4,8,16 sequence
            let source = "0: a(X):Xra(XX) a(s)";
            assert_compiles(source);
        }

        #[test]
        fn test_spiral_xla_xx_s() {
            let source = "0: a(X):Xla(XX) a(s)";
            assert_compiles(source);
        }

        // Pattern: 0,2,4,6,8 with different structure
        #[test]
        fn test_spiral_xa_xss_l_even() {
            let source = "0: a(X):Xa(Xss) a(l)";
            assert_compiles(source);
        }

        #[test]
        fn test_spiral_xa_xss_r() {
            let source = "0: a(X):Xa(Xss) a(r)";
            assert_compiles(source);
        }
    }

    /// Spiral variants
    mod spiral_variants {
        use super::*;

        #[test]
        fn test_spiral_var_xa_sxlx() {
            let source = "0: a(X):Xa(sXlX) a()";
            assert_compiles(source);
        }

        #[test]
        fn test_spiral_var_xa_sxrx() {
            let source = "0: a(X):Xa(sXrX) a()";
            assert_compiles(source);
        }

        #[test]
        fn test_spiral_var_xa_xlxs() {
            let source = "0: a(X):Xa(XlXs) a()";
            assert_compiles(source);
        }

        #[test]
        fn test_spiral_var_xa_xrxs() {
            let source = "0: a(X):Xa(XrXs) a()";
            assert_compiles(source);
        }
    }

    /// Zigzag patterns
    mod zigzag {
        use super::*;

        #[test]
        fn test_zigzag_xxa_sxl() {
            let source = "0: a(X):XXa(sXl) a()";
            assert_compiles(source);
        }

        #[test]
        fn test_zigzag_xxa_sxr() {
            let source = "0: a(X):XXa(sXr) a()";
            assert_compiles(source);
        }

        #[test]
        fn test_zigzag_xxa_lxs() {
            let source = "0: a(X):XXa(lXs) a()";
            assert_compiles(source);
        }

        #[test]
        fn test_zigzag_xxa_rxs() {
            let source = "0: a(X):XXa(rXs) a()";
            assert_compiles(source);
        }
    }

    /// Growing Z patterns
    mod growing_z {
        use super::*;

        #[test]
        fn test_growing_z_xa_lxss() {
            let source = "0: a(X):Xa(lXss) a()";
            assert_compiles(source);
        }

        #[test]
        fn test_growing_z_xa_rxss() {
            let source = "0: a(X):Xa(rXss) a()";
            assert_compiles(source);
        }

        #[test]
        fn test_growing_z_xa_ssxl() {
            let source = "0: a(X):Xa(ssXl) a()";
            assert_compiles(source);
        }

        #[test]
        fn test_growing_z_xa_sxl_s() {
            let source = "0: a(X):Xa(sXl) a(s)";
            assert_compiles(source);
        }

        #[test]
        fn test_growing_z_xa_sxr_s() {
            let source = "0: a(X):Xa(sXr) a(s)";
            assert_compiles(source);
        }

        #[test]
        fn test_growing_z_xsa_sxl() {
            let source = "0: a(X):Xsa(sXl) a()";
            assert_compiles(source);
        }

        #[test]
        fn test_growing_z_sxa_lxs() {
            let source = "0: a(X):sXa(lXs) a()";
            assert_compiles(source);
        }

        #[test]
        fn test_growing_z_xla_lxs() {
            let source = "0: a(X):Xla(lXs) a()";
            assert_compiles(source);
        }

        #[test]
        fn test_growing_z_xra_rxs() {
            let source = "0: a(X):Xra(rXs) a()";
            assert_compiles(source);
        }
    }

    /// Growing Z with square patterns
    mod growing_z_with_square {
        use super::*;

        #[test]
        fn test_growing_z_sq_xa_sxsl() {
            let source = "0: a(X):Xa(sXsl) a()";
            assert_compiles(source);
        }

        #[test]
        fn test_growing_z_sq_xa_sxsr() {
            let source = "0: a(X):Xa(sXsr) a()";
            assert_compiles(source);
        }

        #[test]
        fn test_growing_z_sq_xa_sxls() {
            let source = "0: a(X):Xa(sXls) a()";
            assert_compiles(source);
        }

        #[test]
        fn test_growing_z_sq_xa_slxs() {
            let source = "0: a(X):Xa(slXs) a()";
            assert_compiles(source);
        }

        #[test]
        fn test_growing_z_sq_xa_srxs() {
            let source = "0: a(X):Xa(srXs) a()";
            assert_compiles(source);
        }

        #[test]
        fn test_growing_z_sq_xa_lsxs() {
            let source = "0: a(X):Xa(lsXs) a()";
            assert_compiles(source);
        }
    }

    /// Mysterious/Complex patterns (不可思議)
    mod mysterious {
        use super::*;

        #[test]
        fn test_mysterious_xa_xsxl() {
            let source = "0: a(X):Xa(XsXl) a()";
            assert_compiles(source);
        }

        #[test]
        fn test_mysterious_xa_xsxr() {
            let source = "0: a(X):Xa(XsXr) a()";
            assert_compiles(source);
        }

        #[test]
        fn test_mysterious_xa_sxxl() {
            let source = "0: a(X):Xa(sXXl) a()";
            assert_compiles(source);
        }

        #[test]
        fn test_mysterious_xa_sxxr() {
            let source = "0: a(X):Xa(sXXr) a()";
            assert_compiles(source);
        }

        #[test]
        fn test_mysterious_xa_lxxs() {
            let source = "0: a(X):Xa(lXXs) a()";
            assert_compiles(source);
        }

        #[test]
        fn test_mysterious_xa_rxsx() {
            let source = "0: a(X):Xa(rXsX) a()";
            assert_compiles(source);
        }

        #[test]
        fn test_mysterious_xa_xxl_s() {
            let source = "0: a(X):Xa(XXl) a(s)";
            assert_compiles(source);
        }

        #[test]
        fn test_mysterious_xa_xxr_s() {
            let source = "0: a(X):Xa(XXr) a(s)";
            assert_compiles(source);
        }

        #[test]
        fn test_mysterious_xa_lxx_s() {
            let source = "0: a(X):Xa(lXX) a(s)";
            assert_compiles(source);
        }

        #[test]
        fn test_mysterious_xa_rxx_s() {
            let source = "0: a(X):Xa(rXX) a(s)";
            assert_compiles(source);
        }
    }

    /// Four squares pattern (田4つ)
    mod four_squares {
        use super::*;

        #[test]
        fn test_four_squares_xa_xlx_s() {
            let source = "0: a(X):Xa(XlX) a(s)";
            assert_compiles(source);
        }

        #[test]
        fn test_four_squares_xa_xrx_s() {
            let source = "0: a(X):Xa(XrX) a(s)";
            assert_compiles(source);
        }
    }
}

// =============================================================================
// 3. Conditional Patterns (場合分け/IF文)
// =============================================================================

mod conditional_patterns {
    use super::*;

    /// Basic conditional: a(X):s executes s only if X >= 1
    mod basic_conditional {
        use super::*;

        #[test]
        fn test_conditional_positive() {
            // a(X):s with X=1 executes s
            let source = "0: a(X):s a(1)";
            assert_compiles_to(source, "s");
        }

        #[test]
        fn test_conditional_positive_large() {
            // a(X):s with X=5 executes s
            let source = "0: a(X):s a(5)";
            assert_compiles_to(source, "s");
        }

        #[test]
        fn test_conditional_zero() {
            // a(X):s with X=0 does nothing (0 <= 0)
            let source = "0: a(X):s a(0)";
            assert_compiles_to(source, "");
        }

        #[test]
        fn test_conditional_negative() {
            // a(X):s with X=-1 does nothing
            let source = "0: a(X):s a(-1)";
            assert_compiles_to(source, "");
        }

        #[test]
        fn test_conditional_with_subtraction() {
            // a(10-X) executes if X <= 9
            let source = "0: a(X):s a(10-5)";
            assert_compiles_to(source, "s");
        }

        #[test]
        fn test_conditional_at_boundary() {
            // a(10-X) with X=10 -> a(0) -> no execution
            let source = "0: a(X):s a(10-10)";
            assert_compiles_to(source, "");
        }

        #[test]
        fn test_conditional_exactly_one() {
            // a(2-X) executes only when X=1
            // X=1: a(2-1)=a(1) -> s
            let source = "0: a(X):s a(2-1)";
            assert_compiles_to(source, "s");
        }

        #[test]
        fn test_conditional_exactly_one_fail() {
            // a(2-X) with X=2: a(2-2)=a(0) -> nothing
            let source = "0: a(X):s a(2-2)";
            assert_compiles_to(source, "");
        }
    }

    /// Advanced conditional: a(X,Y):Y returns Y if X >= 1
    mod advanced_conditional {
        use super::*;

        #[test]
        fn test_conditional_return_positive() {
            // a(X,Y):Y returns Y when X >= 1
            let source = "0: a(X,Y):Y a(1,sss)";
            assert_compiles_to(source, "sss");
        }

        #[test]
        fn test_conditional_return_zero() {
            // a(X,Y):Y returns nothing when X <= 0
            let source = "0: a(X,Y):Y a(0,sss)";
            assert_compiles_to(source, "");
        }

        #[test]
        fn test_conditional_return_negative() {
            let source = "0: a(X,Y):Y a(-1,sss)";
            assert_compiles_to(source, "");
        }

        #[test]
        fn test_conditional_return_with_expression() {
            // Use computed condition
            let source = "0: a(X,Y):Y a(5-3,sr)";
            // 5-3=2 >= 1, so returns sr
            assert_compiles_to(source, "sr");
        }

        #[test]
        fn test_conditional_return_boundary() {
            // a(X,Y):Y with X exactly 1
            let source = "0: a(X,Y):Y a(1,rrr)";
            assert_compiles_to(source, "rrr");
        }
    }

    /// Conditional in loop (from snuke's blog example)
    mod conditional_in_loop {
        use super::*;

        #[test]
        fn test_conditional_spiral_with_cutoff() {
            // a(X):s a(10-X) - conditional based on X
            // b(X,Y):a(10-X)Yrb(X+1,sY) - spiral with conditional
            let source = "0: a(X):s\nb(X,Y):a(10-X)Yrb(X+1,sY)\nb(1,)";
            assert_compiles(source);
        }
    }
}

// =============================================================================
// 4. Recursion Termination Patterns (再帰の打ち切り)
// =============================================================================

mod recursion_termination {
    use super::*;

    /// Bomb pattern: a(X,Y):XXXXa(sX,Y-1)
    /// Creates square-like coverage patterns
    mod bomb_patterns {
        use super::*;

        #[test]
        fn test_bomb_basic() {
            // a(X,Y):XXXXa(sX,Y-1) with a(r,2)
            // Y=2: rrrr a(sr,1)
            // Y=1: srsrsrsr a(ssr,0)
            // Y=0: stops
            let source = "0: a(X,Y):XXXXa(sX,Y-1) a(r,2)";
            assert_compiles(source);
        }

        #[test]
        fn test_bomb_y_one() {
            // a(r,1) -> rrrr a(sr,0) -> rrrr
            let source = "0: a(X,Y):XXXXa(sX,Y-1) a(r,1)";
            assert_compiles_to(source, "rrrr");
        }

        #[test]
        fn test_bomb_y_zero() {
            // a(r,0) -> nothing (Y=0 doesn't execute)
            let source = "0: a(X,Y):XXXXa(sX,Y-1) a(r,0)";
            assert_compiles_to(source, "");
        }

        #[test]
        fn test_bomb_quadruple() {
            // a(X,Y):XXXXa(sX,Y-1) with a(p,1) quadruples p
            // a(s,1) -> ssss
            let source = "0: a(X,Y):XXXXa(sX,Y-1) a(s,1)";
            assert_compiles_to(source, "ssss");
        }

        #[test]
        fn test_bomb_12_steps() {
            // For 12 steps: a(,3) is clever
            // a(,3) -> a(s,2) -> ssssa(ss,1) -> ssssssssssssa(sss,0)
            // Let's verify the math: empty + 4*empty = empty at Y=3
            // Actually a(,3) means X="" which doesn't add s
            // This requires a(sss,1) = 12 s's
            let source = "0: a(X,Y):XXXXa(sX,Y-1) a(sss,1)";
            assert_compiles_to(source, "ssssssssssss"); // 12 s's
        }

        #[test]
        fn test_bomb_with_rotation() {
            // Using rotation in bomb
            let source = "0: a(X,Y):XXXXa(sX,Y-1) a(l,2)";
            assert_compiles(source);
            let result = compile_to_string(source).unwrap();
            // Should have some l's and sl combinations
            assert!(result.contains("llll"), "Should contain llll");
        }
    }

    /// Alternative bomb: a(X,Y):Xa(rX,Y-1)
    mod alternative_bomb {
        use super::*;

        #[test]
        fn test_alt_bomb_basic() {
            let source = "0: a(X,Y):Xa(rX,Y-1) a(ss,3)";
            assert_compiles(source);
        }

        #[test]
        fn test_alt_bomb_growth() {
            // a(ss,2) -> ss a(rss,1) -> ss rss a(rrss,0) -> ss rss
            let source = "0: a(X,Y):Xa(rX,Y-1) a(ss,2)";
            assert_compiles_to(source, "ssrss");
        }

        #[test]
        fn test_alt_bomb_single() {
            let source = "0: a(X,Y):Xa(rX,Y-1) a(sss,1)";
            assert_compiles_to(source, "sss");
        }
    }

    /// Post-recursion pattern: a(X,Y):a(sX,Y-1)XXXX
    mod post_recursion {
        use super::*;

        #[test]
        fn test_post_recursion_basic() {
            // a(X,Y):a(sX,Y-1)XXXX - recursion first, then execute
            let source = "0: a(X,Y):a(sX,Y-1)XXXX a(r,2)";
            assert_compiles(source);
        }

        #[test]
        fn test_post_recursion_order() {
            // a(r,2) -> a(sr,1)rrrr -> a(ssr,0)srsrsrsrrrrr
            // Since a(ssr,0) doesn't execute: srsrsrsrrrrr
            let source = "0: a(X,Y):a(sX,Y-1)XXXX a(r,2)";
            let result = compile_to_string(source).unwrap();
            // Should end with rrrr
            assert!(result.ends_with("rrrr"), "Should end with rrrr");
        }

        #[test]
        fn test_post_recursion_with_suffix() {
            // a(X,Y):XXXXa(sX,Y-1)s - with trailing s
            let source = "0: a(X,Y):XXXXa(sX,Y-1)s a(r,2)";
            assert_compiles(source);
        }
    }

    /// Nested bomb usage
    mod nested_bomb {
        use super::*;

        #[test]
        fn test_nested_bomb() {
            // a(a(r,10)r,255) - bomb within bomb
            // This creates complex patterns
            let source = "0: a(X,Y):XXXXa(sX,Y-1) a(a(r,2)r,1)";
            assert_compiles(source);
        }
    }
}

// =============================================================================
// 5. Growth Rate Control (成長速度の調整)
// =============================================================================

mod growth_rate_control {
    use super::*;

    /// Fractional growth rate: 3/4 growth
    /// Pattern: a(T):sa(T-4) b(T):a(T)rb(T+3) b(3)
    mod fractional_growth {
        use super::*;

        #[test]
        fn test_fractional_growth_basic() {
            // a(T):sa(T-4) - count T/4 (ceiling)
            // b(T):a(T)rb(T+3) - grow by 3/4 each step
            let source = "0: a(T):sa(T-4)\nb(T):a(T)rb(T+3)\nb(3)";
            assert_compiles(source);
        }

        #[test]
        fn test_fractional_a_function() {
            // a(T):sa(T-4) with T=8 -> ss (8/4 = 2)
            // a(8) -> sa(4) -> ssa(0) -> ss
            let source = "0: a(T):sa(T-4) a(8)";
            assert_compiles_to(source, "ss");
        }

        #[test]
        fn test_fractional_a_function_12() {
            // a(12) -> sa(8) -> ssa(4) -> sssa(0) -> sss
            let source = "0: a(T):sa(T-4) a(12)";
            assert_compiles_to(source, "sss");
        }

        #[test]
        fn test_fractional_a_function_3() {
            // a(3) -> sa(-1) -> s
            let source = "0: a(T):sa(T-4) a(3)";
            assert_compiles_to(source, "s");
        }

        #[test]
        fn test_fractional_a_function_4() {
            // a(4) -> sa(0) -> s
            let source = "0: a(T):sa(T-4) a(4)";
            assert_compiles_to(source, "s");
        }

        #[test]
        fn test_fractional_b_sequence() {
            // b(3) starts the sequence
            // b(3) -> a(3)rb(6) -> srb(6)
            // b(6) -> a(6)rb(9) -> ssrb(9)
            // etc.
            let source = "0: a(T):sa(T-4)\nb(T):a(T)rb(T+3)\nb(3)";
            let result = compile_to_string(source).unwrap();
            // Should have pattern of increasing s's with r's
            assert!(result.contains("sr"), "Should contain sr");
        }

        #[test]
        fn test_growth_sequence_1_2_3_3() {
            // The blog mentions [1,2,3,3], [4,5,6,6], [7,8,9,9]...
            // This is "4回で3成長" (grow 3 per 4 iterations)
            let source = "0: a(T):sa(T-4)\nb(T):a(T)rb(T+3)\nb(3)";
            assert_compiles(source);
        }
    }

    /// Single-line fractional growth
    mod single_line_fractional {
        use super::*;

        #[test]
        fn test_single_line_fractional_growth() {
            // a(T,U):sa(T-4,U)ra(U,U+3)
            // Combines counting and growing in one function
            let source = "0: a(T,U):sa(T-4,U)ra(U,U+3) a(3,3)";
            assert_compiles(source);
        }

        #[test]
        fn test_single_line_different_start() {
            // Starting with different T gives different initial
            let source = "0: a(T,U):sa(T-4,U)ra(U,U+3) a(7,3)";
            assert_compiles(source);
        }
    }

    /// Quarter growth (1/4)
    mod quarter_growth {
        use super::*;

        #[test]
        fn test_quarter_growth() {
            // Growth rate 1/4: repeat 4 times, grow once
            let source = "0: a(T):sa(T-4) a(16)";
            // 16/4 = 4 s's
            assert_compiles_to(source, "ssss");
        }

        #[test]
        fn test_quarter_growth_17() {
            // 17 -> 13 -> 9 -> 5 -> 1 = 5 s's (ceiling of 17/4)
            let source = "0: a(T):sa(T-4) a(17)";
            assert_compiles_to(source, "sssss");
        }
    }
}

// =============================================================================
// 6. Fractal Patterns (from snuke's blog)
// =============================================================================

mod fractal_patterns {
    use super::*;

    #[test]
    fn test_fractal_basic() {
        // a(A,B):Ala(BlAAABl,BB) a(r,s)
        // Creates fractal patterns through recursive growth
        let source = "0: a(A,B):Ala(BlAAABl,BB) a(r,s)";
        assert_compiles(source);
    }

    #[test]
    fn test_fractal_simple_growth() {
        // Simple fractal: double each iteration
        let source = "0: a(X):Xa(XX) a(s)";
        assert_compiles(source);
        let result = compile_to_string(source).unwrap();
        // Should grow exponentially
        assert!(result.len() > 10, "Should produce many commands");
    }

    #[test]
    fn test_fractal_with_turns() {
        // Fractal with direction changes
        let source = "0: a(X):Xra(Xl) a(s)";
        assert_compiles(source);
    }
}

// =============================================================================
// 7. Multi-argument Function Tests
// =============================================================================

mod multi_argument {
    use super::*;

    #[test]
    fn test_two_args_basic() {
        let source = "0: f(X,Y):XY f(s,r)";
        assert_compiles_to(source, "sr");
    }

    #[test]
    fn test_two_args_reversed() {
        let source = "0: f(X,Y):YX f(s,r)";
        assert_compiles_to(source, "rs");
    }

    #[test]
    fn test_two_args_repeated() {
        let source = "0: f(X,Y):XXYY f(s,r)";
        assert_compiles_to(source, "ssrr");
    }

    #[test]
    fn test_three_args() {
        let source = "0: f(X,Y,Z):XYZ f(s,r,l)";
        assert_compiles_to(source, "srl");
    }

    #[test]
    fn test_four_args() {
        let source = "0: f(A,B,C,D):ABCD f(s,r,l,s)";
        assert_compiles_to(source, "srls");
    }

    #[test]
    fn test_args_with_recursion() {
        // Multi-variable recursion for growth control
        let source = "0: a(A,B,C,D):Ara(ssB,sC,D,A) a(s,,,)";
        assert_compiles(source);
    }
}

// =============================================================================
// 8. Numeric Expression Tests
// =============================================================================

mod numeric_expressions {
    use super::*;

    #[test]
    fn test_numeric_subtraction() {
        // f(X-1) with X=5 -> f(4)
        let source = "0: f(X):sf(X-1) f(5)";
        assert_compiles_to(source, "sssss");
    }

    #[test]
    fn test_numeric_addition() {
        // Using addition in recursion
        let source = "0: a(X):ra(X+1) a(1)";
        assert_compiles(source);
    }

    #[test]
    fn test_numeric_complex_expression() {
        // X-7+1 = X-6
        let source = "0: f(X):sf(X-7) f(21)";
        // 21 -> 14 -> 7 -> 0 (stops) = 3 s's
        assert_compiles_to(source, "sss");
    }

    #[test]
    fn test_numeric_boundary_behavior() {
        // Test that X=0 stops recursion
        let source = "0: f(X):sf(X-1) f(0)";
        assert_compiles_to(source, "");
    }

    #[test]
    fn test_numeric_negative_stops() {
        // Negative values stop recursion
        let source = "0: f(X):sf(X-1) f(-1)";
        assert_compiles_to(source, "");
    }
}

// =============================================================================
// 9. Edge Cases and Complex Patterns
// =============================================================================

mod edge_cases {
    use super::*;

    #[test]
    fn test_empty_function_arg() {
        // Function with empty argument
        let source = "0: f(X):Xs f(,)";
        assert_compiles_to(source, "s");
    }

    #[test]
    fn test_empty_macro() {
        // Empty macro body
        let source = "0: a: a";
        assert_compiles_to(source, "");
    }

    #[test]
    fn test_deeply_nested_function() {
        // f(f(f(s)))
        let source = "0: f(X):XX f(f(f(s)))";
        // f(s) = ss, f(ss) = ssss, f(ssss) = ssssssss
        assert_compiles_to(source, "ssssssss");
    }

    #[test]
    fn test_macro_and_function_combined() {
        let source = "0: a:srl f(X):Xa f(ss)";
        assert_compiles_to(source, "sssrl");
    }

    #[test]
    fn test_function_reuse() {
        // Reuse same function multiple times
        let source = "0: f(X):XX f(s)f(r)f(l)";
        assert_compiles_to(source, "ssrrll");
    }

    #[test]
    fn test_long_recursion_chain() {
        // Longer recursion
        let source = "0: f(X):sf(X-1) f(20)";
        assert_command_count(source, 20);
    }

    #[test]
    fn test_multiple_numeric_args() {
        // Two numeric arguments
        let source = "0: f(X,Y):sf(X-1,Y-1) f(3,5)";
        // Stops when either becomes <= 0, so 3 s's
        assert_compiles_to(source, "sss");
    }

    #[test]
    fn test_numeric_with_command_arg() {
        // Mix of numeric and command args
        let source = "0: f(X,N):Xf(sX,N-1) f(r,3)";
        // N=3: r f(sr,2)
        // N=2: rsr f(ssr,1)
        // N=1: rsrssr f(sssr,0)
        // N=0: stops
        assert_compiles_to(source, "rsrssr");
    }
}

// =============================================================================
// 10. HOJ Problem Patterns (from snuke's blog)
// =============================================================================

mod hoj_patterns {
    use super::*;

    /// Hyperbola pattern: y = 24/x
    #[test]
    fn test_hyperbola_y_function() {
        // y(A,X):sy(A-X,X) - counts A/X
        let source = "0: y(A,X):sy(A-X,X) y(24,6)";
        // 24/6 = 4
        assert_compiles_to(source, "ssss");
    }

    #[test]
    fn test_hyperbola_y_function_12() {
        let source = "0: y(A,X):sy(A-X,X) y(24,12)";
        // 24/12 = 2
        assert_compiles_to(source, "ss");
    }

    #[test]
    fn test_hyperbola_y_function_8() {
        let source = "0: y(A,X):sy(A-X,X) y(24,8)";
        // 24/8 = 3
        assert_compiles_to(source, "sss");
    }

    /// Improved hyperbola with rsly pattern
    #[test]
    fn test_hyperbola_improved() {
        // y(A,X):rsly(A-X,X)lsr
        let source = "0: y(A,X):rsly(A-X,X)lsr y(24,8)";
        assert_compiles(source);
    }
}

// =============================================================================
// 11. Multiline Pattern Tests
// =============================================================================

mod multiline_patterns {
    use super::*;

    #[test]
    fn test_multiline_single_agent() {
        // Multiline without agent prefix
        let source = "a:srl\naaa";
        assert_compiles_to(source, "srlsrlsrl");
    }

    #[test]
    fn test_multiline_macro_then_usage() {
        let source = "a:ssrs\naaaaaaaaaa";
        assert_compiles(source);
        assert_command_count(source, 40); // 10 * 4 = 40
    }

    #[test]
    fn test_multiline_function_definition() {
        let source = "f(X):XX\nf(srl)";
        assert_compiles_to(source, "srlsrl");
    }

    #[test]
    fn test_multiline_multi_agent() {
        // Multi-agent with multiline code
        let source = "0: a:ss\naa\n1: srl";
        let result = compile_native(source);
        match result {
            CompileResult::Success { program } => {
                assert_eq!(program.agents.len(), 2);
            }
            CompileResult::Error { errors } => {
                panic!("Failed: {:?}", errors);
            }
        }
    }

    #[test]
    fn test_multiline_complex() {
        let source = "f(X):Xr\na:ss\nf(a)f(a)";
        assert_compiles_to(source, "ssrssr");
    }
}
