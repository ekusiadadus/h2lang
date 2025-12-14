//! Byte count tests for HOJ golf scoring compatibility
//!
//! These tests verify the byte counting functionality according to HOJ specification.
//! See docs/SPEC.md Appendix B for the full specification.
//!
//! **Important**: count_bytes_native validates syntax first.
//! Invalid H2 programs return an error, not a byte count.

use h2lang::count_bytes_native;

// =============================================================================
// Helper macro for concise testing
// =============================================================================

macro_rules! assert_bytes {
    ($source:expr, $expected:expr) => {
        assert_eq!(
            count_bytes_native($source),
            Ok($expected),
            "Failed for source: {:?}",
            $source
        );
    };
}

macro_rules! assert_syntax_error {
    ($source:expr) => {
        assert!(
            count_bytes_native($source).is_err(),
            "Expected syntax error for: {:?}",
            $source
        );
    };
}

// =============================================================================
// Basic Examples from SPEC.md
// =============================================================================

#[test]
fn test_basic_example_1() {
    // a:sa a → a + s + a + a = 4 bytes
    assert_bytes!("a:sa a", 4);
}

#[test]
fn test_basic_example_2() {
    // f(X):sa(X-1) f(10) → f + X + s + a + X + 1 + f + 10 = 8 bytes
    assert_bytes!("f(X):sa(X-1) f(10)", 8);
}

#[test]
fn test_basic_example_3() {
    // 0: x:ss xrx → x + s + s + x + r + x = 6 bytes (agent id not counted)
    assert_bytes!("0: x:ss xrx", 6);
}

#[test]
fn test_basic_example_4() {
    // Two-param function with both CmdSeq types
    // f(X,Y):XYf(X,Y) f(s,r) → f + X + Y + X + Y + f + X + Y + f + s + r = 11 bytes
    assert_bytes!("f(X,Y):XYf(X,Y) f(s,r)", 11);
}

// =============================================================================
// Command Counting
// =============================================================================

#[test]
fn test_commands_only() {
    // Bare commands without agent id (treated as agent 0)
    assert_bytes!("s", 1);
    assert_bytes!("srl", 3);
    assert_bytes!("ssssrrrrlllll", 13);
}

#[test]
fn test_commands_with_spaces() {
    assert_bytes!("s r l", 3);
    assert_bytes!("s  r  l", 3);
}

// =============================================================================
// Function Definition and Call
// =============================================================================

#[test]
fn test_zero_arg_function() {
    // Zero-arg function definition and call
    // x:ss x → x + s + s + x = 4 bytes
    assert_bytes!("x:ss x", 4);
}

#[test]
fn test_function_with_empty_parens() {
    // Function defined with empty parentheses
    // f():ss f() → f + s + s + f = 4 bytes
    assert_bytes!("f():ss f()", 4);
}

#[test]
fn test_single_param_function() {
    // f(X):XX f(s) → f + X + X + X + f + s = 6 bytes
    assert_bytes!("f(X):XX f(s)", 6);
}

#[test]
fn test_two_param_function() {
    // f(X,Y):XY f(s,r) → f + X + Y + X + Y + f + s + r = 8 bytes
    assert_bytes!("f(X,Y):XY f(s,r)", 8);
}

// =============================================================================
// Numeric Arguments
// =============================================================================

#[test]
fn test_single_digit_number() {
    // a(X):sa(X-1) a(5) → a + X + s + a + X + 1 + a + 5 = 8 bytes
    assert_bytes!("a(X):sa(X-1) a(5)", 8);
}

#[test]
fn test_multi_digit_number() {
    // Multi-digit number counts as 1 byte (not per digit)
    // a(X):sa(X-1) a(100) → a + X + s + a + X + 1 + a + 100 = 8 bytes
    assert_bytes!("a(X):sa(X-1) a(100)", 8);
}

#[test]
fn test_large_number() {
    // Large numbers still count as 1 byte
    // a(X):sa(X-1) a(999999) → 8 bytes
    assert_bytes!("a(X):sa(X-1) a(999999)", 8);
}

// =============================================================================
// Punctuation (Not Counted)
// =============================================================================

#[test]
fn test_colon_not_counted() {
    // Colon is not counted
    // x:s → x + s = 2 bytes
    assert_bytes!("x:s", 2);
}

#[test]
fn test_parentheses_not_counted() {
    // Parentheses are not counted
    // f(X):X f(s) → f + X + X + f + s = 5 bytes
    assert_bytes!("f(X):X f(s)", 5);
}

#[test]
fn test_comma_not_counted() {
    // Comma is not counted
    // f(X,Y):XY f(s,r) → f + X + Y + X + Y + f + s + r = 8 bytes
    assert_bytes!("f(X,Y):XY f(s,r)", 8);
}

#[test]
fn test_plus_minus_not_counted() {
    // Plus and minus operators are not counted
    // f(X):sf(X-1) f(5) → f + X + s + f + X + 1 + f + 5 = 8 bytes
    assert_bytes!("f(X):sf(X-1) f(5)", 8);
    // f(X):sf(X+1-2) f(5) → f + X + s + f + X + 1 + 2 + f + 5 = 9 bytes
    assert_bytes!("f(X):sf(X+1-2) f(5)", 9);
}

// =============================================================================
// Whitespace (Not Counted)
// =============================================================================

#[test]
fn test_space_not_counted() {
    assert_bytes!("x:ss x", 4);
    assert_bytes!("x:ss  x", 4);
    assert_bytes!("x:ss   x", 4);
}

#[test]
fn test_tab_not_counted() {
    assert_bytes!("x:ss\tx", 4);
}

#[test]
fn test_newline_not_counted() {
    assert_bytes!("x:ss\nx", 4);
    assert_bytes!("x:ss\r\nx", 4);
}

// =============================================================================
// Comments (Not Counted)
// =============================================================================

#[test]
fn test_hash_comment_not_counted() {
    assert_bytes!("srl # this is a comment", 3);
    assert_bytes!("srl #comment with letters abc", 3);
}

#[test]
fn test_double_slash_comment_not_counted() {
    assert_bytes!("srl // this is a comment", 3);
    assert_bytes!("srl //comment with numbers 123", 3);
}

#[test]
fn test_multiline_with_comments() {
    let source = "x:ss # define x\nxrx // use x";
    assert_bytes!(source, 6); // x + s + s + x + r + x
}

// =============================================================================
// Directives (Not Counted)
// =============================================================================

#[test]
fn test_directive_not_counted() {
    assert_bytes!("MAX_STEP=100\nsrl", 3);
}

#[test]
fn test_multiple_directives_not_counted() {
    let source = "MAX_STEP=1000\nMAX_DEPTH=50\nON_LIMIT=ERROR\nsrl";
    assert_bytes!(source, 3);
}

#[test]
fn test_directive_with_truncate() {
    let source = "ON_LIMIT=TRUNCATE\nf(X):sf(X-1) f(10)";
    assert_bytes!(source, 8);
}

// =============================================================================
// Agent ID (Not Counted)
// =============================================================================

#[test]
fn test_agent_id_not_counted() {
    assert_bytes!("0:srl", 3);
    assert_bytes!("1:srl", 3);
    assert_bytes!("99:srl", 3);
}

#[test]
fn test_multiple_agents() {
    let source = "0:srl\n1:lrs";
    assert_bytes!(source, 6); // s + r + l + l + r + s
}

#[test]
fn test_agent_with_definitions() {
    let source = "0: x:ss xrx\n1: y:rr yly";
    assert_bytes!(source, 12); // x+s+s+x+r+x + y+r+r+y+l+y
}

// =============================================================================
// Complex Examples
// =============================================================================

#[test]
fn test_hoj_square_pattern() {
    // Classic HOJ square pattern
    // f(X):XXXX f(sssr) → f + X + X + X + X + X + f + s + s + s + r = 11 bytes
    assert_bytes!("f(X):XXXX f(sssr)", 11);
}

#[test]
fn test_recursive_function() {
    // Numeric recursion (X is Int type)
    // a(X):sa(X-1) a(4) → a + X + s + a + X + 1 + a + 4 = 8 bytes
    assert_bytes!("a(X):sa(X-1) a(4)", 8);
}

#[test]
fn test_nested_function_calls() {
    // f(X):XX g(Y):YYY f(g(s))
    // → f + X + X + X + g + Y + Y + Y + Y + f + g + s = 12 bytes
    assert_bytes!("f(X):XX g(Y):YYY f(g(s))", 12);
}

#[test]
fn test_complex_numeric_expression() {
    // f(X):sf(X-1+2-3) f(10) → f + X + s + f + X + 1 + 2 + 3 + f + 10 = 10 bytes
    assert_bytes!("f(X):sf(X-1+2-3) f(10)", 10);
}

// =============================================================================
// Edge Cases (Valid Syntax)
// =============================================================================

#[test]
fn test_empty_input() {
    // Empty input is valid (no agents, no commands)
    assert_bytes!("", 0);
}

#[test]
fn test_only_whitespace() {
    assert_bytes!("   \t\n  ", 0);
}

#[test]
fn test_only_comments() {
    assert_bytes!("# just a comment", 0);
    assert_bytes!("// another comment", 0);
}

#[test]
fn test_only_directives() {
    // Directive followed by newline (no commands)
    assert_bytes!("MAX_STEP=100\n", 0);
}

#[test]
fn test_single_command() {
    assert_bytes!("s", 1);
    assert_bytes!("r", 1);
    assert_bytes!("l", 1);
}

// =============================================================================
// HOJ Blog Examples (snuke's patterns)
// =============================================================================

#[test]
fn test_snuke_basic_recursion() {
    // Basic recursive pattern
    // a(X):sa(X-1) a(4) → a + X + s + a + X + 1 + a + 4 = 8 bytes
    assert_bytes!("a(X):sa(X-1) a(4)", 8);
}

#[test]
fn test_snuke_double_param() {
    // Two parameter function
    // a(X,Y):Ya(X-1,Y) a(4,s) → a + X + Y + Y + a + X + 1 + Y + a + 4 + s = 11 bytes
    assert_bytes!("a(X,Y):Ya(X-1,Y) a(4,s)", 11);
}

#[test]
fn test_concatenated_params() {
    // Concatenated parameters in function args (AA, AB pattern)
    // f(A,B):ABf(A,B) f(s,r) → f + A + B + A + B + f + A + B + f + s + r = 11 bytes
    assert_bytes!("f(A,B):ABf(A,B) f(s,r)", 11);
}

// =============================================================================
// Syntax Error Cases (Parse-time errors)
// =============================================================================

#[test]
fn test_invalid_directive_error() {
    // Unknown directive name
    assert_syntax_error!("INVALID_DIRECTIVE=100");
}

#[test]
fn test_mismatched_parens_error() {
    assert_syntax_error!("f(X:X");
}

#[test]
fn test_type_conflict_error() {
    // Parameter used as both CmdSeq and Int (E010)
    assert_syntax_error!("f(X):Xf(X-1)");
}

#[test]
fn test_only_punctuation_error() {
    // Just punctuation without valid program structure
    assert_syntax_error!(":(),+-");
}
