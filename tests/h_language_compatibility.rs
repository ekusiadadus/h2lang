//! Comprehensive H Language (Herbert Online Judge) Compatibility Test Suite
//!
//! This test suite validates that hlang2 is fully compatible with the H language
//! specification used by Herbert Online Judge (HOJ).
//!
//! References:
//! - Herbert Online Judge: http://herbert.tealang.info/
//! - HOJ GitHub: https://github.com/quolc/hoj
//! - Codeforces discussion: https://codeforces.com/blog/entry/5579

use toioswarm_lang::compile_native;
use toioswarm_lang::output::CompileResult;

// =============================================================================
// Test Helpers
// =============================================================================

/// Compile source and return command sequence as string (e.g., "srl")
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
                    toioswarm_lang::output::CommandType::Straight => 's',
                    toioswarm_lang::output::CommandType::RotateRight => 'r',
                    toioswarm_lang::output::CommandType::RotateLeft => 'l',
                    toioswarm_lang::output::CommandType::Wait => 'w',
                })
                .collect();
            Ok(commands)
        }
        CompileResult::Error { errors } => {
            Err(errors.iter().map(|e| e.message.clone()).collect::<Vec<_>>().join("; "))
        }
    }
}

/// Compile source and return number of commands for agent 0
fn compile_to_count(source: &str) -> Result<usize, String> {
    compile_to_string(source).map(|s| s.len())
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

/// Assert that source fails to compile
fn assert_compile_error(source: &str) {
    match compile_native(source) {
        CompileResult::Success { .. } => {
            panic!("Source '{}' should have failed to compile", source);
        }
        CompileResult::Error { .. } => {
            // Expected
        }
    }
}

// =============================================================================
// 1. Basic Commands Tests (HOJ Specification)
// =============================================================================

mod basic_commands {
    use super::*;

    #[test]
    fn test_single_straight() {
        assert_compiles_to("0: s", "s");
    }

    #[test]
    fn test_single_right() {
        assert_compiles_to("0: r", "r");
    }

    #[test]
    fn test_single_left() {
        assert_compiles_to("0: l", "l");
    }

    #[test]
    fn test_basic_sequence_srl() {
        assert_compiles_to("0: srl", "srl");
    }

    #[test]
    fn test_basic_sequence_lrs() {
        assert_compiles_to("0: lrs", "lrs");
    }

    #[test]
    fn test_repeated_straight() {
        assert_compiles_to("0: ssss", "ssss");
    }

    #[test]
    fn test_repeated_right() {
        assert_compiles_to("0: rrrr", "rrrr");
    }

    #[test]
    fn test_repeated_left() {
        assert_compiles_to("0: llll", "llll");
    }

    #[test]
    fn test_long_sequence() {
        assert_compiles_to("0: ssssrssssrsssslssssl", "ssssrssssrsssslssssl");
    }

    #[test]
    fn test_all_combinations_short() {
        // All 3^3 = 27 combinations of 3 commands
        assert_compiles_to("0: sss", "sss");
        assert_compiles_to("0: ssr", "ssr");
        assert_compiles_to("0: ssl", "ssl");
        assert_compiles_to("0: srs", "srs");
        assert_compiles_to("0: srr", "srr");
        assert_compiles_to("0: srl", "srl");
        assert_compiles_to("0: sls", "sls");
        assert_compiles_to("0: slr", "slr");
        assert_compiles_to("0: sll", "sll");
        assert_compiles_to("0: rss", "rss");
        assert_compiles_to("0: rsr", "rsr");
        assert_compiles_to("0: rsl", "rsl");
        assert_compiles_to("0: rrs", "rrs");
        assert_compiles_to("0: rrr", "rrr");
        assert_compiles_to("0: rrl", "rrl");
        assert_compiles_to("0: rls", "rls");
        assert_compiles_to("0: rlr", "rlr");
        assert_compiles_to("0: rll", "rll");
        assert_compiles_to("0: lss", "lss");
        assert_compiles_to("0: lsr", "lsr");
        assert_compiles_to("0: lsl", "lsl");
        assert_compiles_to("0: lrs", "lrs");
        assert_compiles_to("0: lrr", "lrr");
        assert_compiles_to("0: lrl", "lrl");
        assert_compiles_to("0: lls", "lls");
        assert_compiles_to("0: llr", "llr");
        assert_compiles_to("0: lll", "lll");
    }

    #[test]
    fn test_empty_command_with_macro_only() {
        // Define a macro but don't use it - should result in empty commands
        let result = compile_to_string("0: x:ss");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "");
    }
}

// =============================================================================
// 2. Macro Compatibility Tests
// =============================================================================

mod macro_compatibility {
    use super::*;

    #[test]
    fn test_simple_macro() {
        // x:ss x -> ss
        assert_compiles_to("0: x:ss x", "ss");
    }

    #[test]
    fn test_macro_with_commands() {
        // x:ss xrx -> ssrss
        assert_compiles_to("0: x:ss xrx", "ssrss");
    }

    #[test]
    fn test_macro_multiple_uses() {
        // x:s xxx -> sss
        assert_compiles_to("0: x:s xxx", "sss");
    }

    #[test]
    fn test_macro_name_a() {
        assert_compiles_to("0: a:ss a", "ss");
    }

    #[test]
    fn test_macro_name_b() {
        assert_compiles_to("0: b:rr b", "rr");
    }

    #[test]
    fn test_macro_name_z() {
        assert_compiles_to("0: z:ll z", "ll");
    }

    #[test]
    fn test_multiple_macros() {
        // x:ss y:rr xy -> ssrr
        assert_compiles_to("0: x:ss y:rr xy", "ssrr");
    }

    #[test]
    fn test_macro_using_macro() {
        // a:s b:aa bbb -> aaaaaa -> ssssss
        assert_compiles_to("0: a:s b:aa bbb", "ssssss");
    }

    #[test]
    fn test_macro_complex_body() {
        // x:srl x -> srl
        assert_compiles_to("0: x:srl x", "srl");
    }

    #[test]
    fn test_macro_long_body() {
        // x:ssssrrrr x -> ssssrrrr
        assert_compiles_to("0: x:ssssrrrr x", "ssssrrrr");
    }

    #[test]
    fn test_three_macros() {
        // a:s b:r c:l abc -> srl
        assert_compiles_to("0: a:s b:r c:l abc", "srl");
    }

    #[test]
    fn test_macro_chain() {
        // a:s b:a c:b ccc -> aaa -> sss
        assert_compiles_to("0: a:s b:a c:b ccc", "sss");
    }

    #[test]
    fn test_undefined_macro_error() {
        // Using undefined macro should error
        assert_compile_error("0: x");
    }

    #[test]
    fn test_macro_names_near_primitives() {
        // t, u, v etc. should work as macro names
        assert_compiles_to("0: t:ss t", "ss");
        assert_compiles_to("0: u:rr u", "rr");
        assert_compiles_to("0: v:ll v", "ll");
    }

    #[test]
    fn test_macro_with_mixed_content() {
        // x:srl y:lrs xyx -> srllrssrl
        assert_compiles_to("0: x:srl y:lrs xyx", "srllrssrl");
    }
}

// =============================================================================
// 3. Function Compatibility Tests
// =============================================================================

mod function_compatibility {
    use super::*;

    #[test]
    fn test_simple_function_identity() {
        // f(X):X f(s) -> s
        assert_compiles_to("0: f(X):X f(s)", "s");
    }

    #[test]
    fn test_function_double() {
        // f(X):XX f(s) -> ss
        assert_compiles_to("0: f(X):XX f(s)", "ss");
    }

    #[test]
    fn test_function_triple() {
        // f(X):XXX f(s) -> sss
        assert_compiles_to("0: f(X):XXX f(s)", "sss");
    }

    #[test]
    fn test_function_quadruple() {
        // f(X):XXXX f(s) -> ssss
        assert_compiles_to("0: f(X):XXXX f(s)", "ssss");
    }

    #[test]
    fn test_function_with_prefix() {
        // f(X):sX f(r) -> sr
        assert_compiles_to("0: f(X):sX f(r)", "sr");
    }

    #[test]
    fn test_function_with_suffix() {
        // f(X):Xs f(r) -> rs
        assert_compiles_to("0: f(X):Xs f(r)", "rs");
    }

    #[test]
    fn test_function_with_prefix_and_suffix() {
        // f(X):sXr f(l) -> slr
        assert_compiles_to("0: f(X):sXr f(l)", "slr");
    }

    #[test]
    fn test_function_with_command_sequence_arg() {
        // f(X):XX f(srl) -> srlsrl
        assert_compiles_to("0: f(X):XX f(srl)", "srlsrl");
    }

    #[test]
    fn test_nested_function_call() {
        // f(X):XX f(f(s)) -> f(ss) -> ssss
        assert_compiles_to("0: f(X):XX f(f(s))", "ssss");
    }

    #[test]
    fn test_deeply_nested_function() {
        // f(X):XX f(f(f(s))) -> f(f(ss)) -> f(ssss) -> ssssssss
        assert_compiles_to("0: f(X):XX f(f(f(s)))", "ssssssss");
    }

    #[test]
    fn test_two_parameter_function() {
        // f(X,Y):XY f(s,r) -> sr
        assert_compiles_to("0: f(X,Y):XY f(s,r)", "sr");
    }

    #[test]
    fn test_two_parameter_function_reversed() {
        // f(X,Y):YX f(s,r) -> rs
        assert_compiles_to("0: f(X,Y):YX f(s,r)", "rs");
    }

    #[test]
    fn test_three_parameter_function() {
        // f(X,Y,Z):XYZ f(s,r,l) -> srl
        assert_compiles_to("0: f(X,Y,Z):XYZ f(s,r,l)", "srl");
    }

    #[test]
    fn test_function_parameter_repeated() {
        // f(X,Y):XXYY f(s,r) -> ssrr
        assert_compiles_to("0: f(X,Y):XXYY f(s,r)", "ssrr");
    }

    #[test]
    fn test_function_different_names() {
        assert_compiles_to("0: a(X):XX a(s)", "ss");
        assert_compiles_to("0: b(X):XX b(s)", "ss");
        assert_compiles_to("0: g(X):XX g(s)", "ss");
        assert_compiles_to("0: h(X):XX h(s)", "ss");
    }

    #[test]
    fn test_function_with_macro() {
        // x:ss f(X):XX f(x) -> f(ss) -> ssss
        assert_compiles_to("0: x:ss f(X):XX f(x)", "ssss");
    }

    #[test]
    fn test_undefined_function_error() {
        assert_compile_error("0: f(s)");
    }

    #[test]
    fn test_hoj_square_pattern() {
        // HOJ classic: f(X):XXXX f(sssr) draws a square
        // sssr repeated 4 times = sssrsssrsssrsssr (16 commands)
        assert_compiles_to("0: f(X):XXXX f(sssr)", "sssrsssrsssrsssr");
    }

    #[test]
    fn test_hoj_triangle_pattern() {
        // f(X):XXX f(ssr) -> ssrssrssr (9 commands)
        assert_compiles_to("0: f(X):XXX f(ssr)", "ssrssrssr");
    }
}

// =============================================================================
// 4. Numeric Arguments Tests (HOJ Extension)
// =============================================================================

mod numeric_arguments {
    use super::*;

    #[test]
    fn test_numeric_simple_4() {
        // a(X):sa(X-1) a(4) -> s + s + s + s = ssss
        assert_compiles_to("0: a(X):sa(X-1) a(4)", "ssss");
    }

    #[test]
    fn test_numeric_simple_3() {
        assert_compiles_to("0: a(X):sa(X-1) a(3)", "sss");
    }

    #[test]
    fn test_numeric_simple_2() {
        assert_compiles_to("0: a(X):sa(X-1) a(2)", "ss");
    }

    #[test]
    fn test_numeric_simple_1() {
        // a(1) -> s + a(0) = s (a(0) terminates)
        assert_compiles_to("0: a(X):sa(X-1) a(1)", "s");
    }

    #[test]
    fn test_numeric_zero_terminates() {
        // a(0) should produce nothing
        assert_compiles_to("0: a(X):sa(X-1) a(0)", "");
    }

    #[test]
    fn test_numeric_negative_terminates() {
        // a(-1) should produce nothing
        assert_compiles_to("0: a(X):sa(X-1) a(-1)", "");
    }

    #[test]
    fn test_numeric_negative_2() {
        assert_compiles_to("0: a(X):sa(X-1) a(-2)", "");
    }

    #[test]
    fn test_numeric_with_turn() {
        // a(X):sra(X-1) a(4) -> sr sr sr sr = srsrsrsr
        assert_compiles_to("0: a(X):sra(X-1) a(4)", "srsrsrsr");
    }

    #[test]
    fn test_numeric_spiral() {
        // a(X):sla(X-1) a(4) -> sl sl sl sl = slslslsl
        assert_compiles_to("0: a(X):sla(X-1) a(4)", "slslslsl");
    }

    #[test]
    fn test_numeric_with_two_params() {
        // a(X,Y):Ya(X-1,Y) a(4,s) -> s + s + s + s = ssss
        assert_compiles_to("0: a(X,Y):Ya(X-1,Y) a(4,s)", "ssss");
    }

    #[test]
    fn test_numeric_two_params_different_command() {
        // a(X,Y):Ya(X-1,Y) a(4,r) -> rrrr
        assert_compiles_to("0: a(X,Y):Ya(X-1,Y) a(4,r)", "rrrr");
    }

    #[test]
    fn test_numeric_two_params_sequence() {
        // a(X,Y):Ya(X-1,Y) a(3,sr) -> sr sr sr = srsrsr
        assert_compiles_to("0: a(X,Y):Ya(X-1,Y) a(3,sr)", "srsrsr");
    }

    #[test]
    fn test_numeric_two_params_zero() {
        // a(X,Y):Ya(X-1,Y) a(0,s) -> empty
        assert_compiles_to("0: a(X,Y):Ya(X-1,Y) a(0,s)", "");
    }

    #[test]
    fn test_numeric_increment() {
        // a(X):sa(X+1) with negative start should terminate at 0
        // a(-3) -> a(-2) -> a(-1) -> a(0) = empty (all terminate)
        // Wait, this will cause infinite loop if X increases!
        // Let's test with upper bound check
        // Actually in HOJ, only X <= 0 terminates, so X+1 from negative will eventually hit 0
        // a(-2) -> s + a(-1) but -2 <= 0 so terminates immediately
        assert_compiles_to("0: a(X):sa(X+1) a(-2)", "");
    }

    #[test]
    fn test_numeric_large_value() {
        // a(X):sa(X-1) a(10) -> 10 s's
        let result = compile_to_count("0: a(X):sa(X-1) a(10)");
        assert_eq!(result.unwrap(), 10);
    }

    #[test]
    fn test_numeric_20() {
        let result = compile_to_count("0: a(X):sa(X-1) a(20)");
        assert_eq!(result.unwrap(), 20);
    }

    #[test]
    fn test_numeric_with_prefix() {
        // a(X):rsa(X-1) a(3) -> rs rs rs = rsrsrs
        assert_compiles_to("0: a(X):rsa(X-1) a(3)", "rsrsrs");
    }

    #[test]
    fn test_numeric_with_suffix() {
        // a(X):a(X-1)s a(3) -> (a(2)s)s = ((a(1)s)s)s = (((a(0)s)s)s) = sss
        // Wait, order matters: a(X):a(X-1)s a(3)
        // a(3) -> a(2)s -> (a(1)s)s -> ((a(0)s)s)s -> ((s)s)s -> sss
        // Actually a(0) = empty, so a(1) = a(0)s = s, a(2) = a(1)s = ss, a(3) = a(2)s = sss
        assert_compiles_to("0: a(X):a(X-1)s a(3)", "sss");
    }
}

// =============================================================================
// 5. Termination and Recursion Depth Tests
// =============================================================================

mod termination {
    use super::*;

    #[test]
    fn test_recursion_terminates_at_zero() {
        // Base case: f(0) should return empty
        assert_compiles_to("0: f(X):sf(X-1) f(0)", "");
    }

    #[test]
    fn test_recursion_terminates_at_negative() {
        assert_compiles_to("0: f(X):sf(X-1) f(-5)", "");
    }

    #[test]
    fn test_mutual_recursion_style() {
        // Two functions calling each other (with numeric termination)
        // This tests that the expander handles complex call patterns
        // a(X):sb(X-1) b(X):ra(X-1) a(4) -> s + b(3) -> s + r + a(2) -> s + r + s + b(1) -> s + r + s + r + a(0)
        // = srsr (4 commands)
        assert_compiles_to("0: a(X):sb(X-1) b(X):ra(X-1) a(4)", "srsr");
    }

    #[test]
    fn test_recursion_depth_moderate() {
        // Test recursion with depth 50
        let result = compile_to_count("0: a(X):sa(X-1) a(50)");
        assert_eq!(result.unwrap(), 50);
    }

    #[test]
    fn test_deep_nested_function_calls() {
        // f(X):XX f(f(f(f(s))))
        // f(s) = ss (2)
        // f(f(s)) = f(ss) = ssss (4)
        // f(f(f(s))) = f(ssss) = ssssssss (8)
        // f(f(f(f(s)))) = f(ssssssss) = 16
        let result = compile_to_count("0: f(X):XX f(f(f(f(s))))");
        assert_eq!(result.unwrap(), 16);
    }

    #[test]
    fn test_exponential_growth_bounded() {
        // f(X):XX creates exponential growth
        // f(f(f(f(f(s))))) = 2^5 = 32 commands
        let result = compile_to_count("0: f(X):XX f(f(f(f(f(s)))))");
        assert_eq!(result.unwrap(), 32);
    }
}

// =============================================================================
// 6. Error Case Tests
// =============================================================================

mod error_cases {
    use super::*;

    #[test]
    fn test_undefined_macro() {
        assert_compile_error("0: x");
    }

    #[test]
    fn test_undefined_function() {
        assert_compile_error("0: f(s)");
    }

    #[test]
    fn test_syntax_error_unmatched_paren() {
        assert_compile_error("0: f(X:X f(s)");
    }

    #[test]
    fn test_syntax_error_unmatched_paren_right() {
        assert_compile_error("0: f(X):X f(s))");
    }

    #[test]
    fn test_invalid_character() {
        assert_compile_error("0: !");
    }

    #[test]
    fn test_missing_agent_id() {
        assert_compile_error("srl");
    }

    #[test]
    fn test_missing_colon() {
        assert_compile_error("0 srl");
    }

    #[test]
    fn test_undefined_parameter() {
        // Function using undefined parameter
        assert_compile_error("0: f(X):Y f(s)");
    }

    #[test]
    fn test_multiple_undefined_macros() {
        assert_compile_error("0: xy");
    }

    #[test]
    fn test_function_without_definition() {
        assert_compile_error("0: g(s)");
    }

    #[test]
    fn test_empty_function_call() {
        // f() with no args when f expects one
        // This might parse differently, let's see
        let result = compile_native("0: f(X):X f()");
        // Should either error or produce empty
        match result {
            CompileResult::Error { .. } => { /* OK */ }
            CompileResult::Success { program } => {
                // If it succeeds, the commands should be empty
                assert!(program.agents[0].commands.is_empty());
            }
        }
    }
}

// =============================================================================
// 7. Whitespace and Newline Tests
// =============================================================================

mod whitespace {
    use super::*;

    #[test]
    fn test_space_after_colon() {
        assert_compiles_to("0: srl", "srl");
    }

    #[test]
    fn test_no_space_after_colon() {
        assert_compiles_to("0:srl", "srl");
    }

    #[test]
    fn test_multiple_spaces() {
        assert_compiles_to("0:   srl", "srl");
    }

    #[test]
    fn test_space_between_macro_and_commands() {
        assert_compiles_to("0: x:ss x", "ss");
    }

    #[test]
    fn test_tab_character() {
        assert_compiles_to("0:\tsrl", "srl");
    }

    #[test]
    fn test_space_in_macro_definition() {
        // Space after macro body ends the definition
        assert_compiles_to("0: x:ss xrx", "ssrss");
    }

    #[test]
    fn test_multiple_agents_newline() {
        let result = compile_native("0: s\n1: r");
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
    fn test_trailing_newline() {
        assert_compiles_to("0: srl\n", "srl");
    }

    #[test]
    fn test_multiple_trailing_newlines() {
        assert_compiles_to("0: srl\n\n\n", "srl");
    }

    #[test]
    fn test_leading_newlines() {
        assert_compiles_to("\n\n0: srl", "srl");
    }

    #[test]
    fn test_crlf_line_endings() {
        assert_compiles_to("0: srl\r\n", "srl");
    }
}

// =============================================================================
// 8. Comment Tests
// =============================================================================

mod comments {
    use super::*;

    #[test]
    fn test_hash_comment() {
        assert_compiles_to("0: srl # comment", "srl");
    }

    #[test]
    fn test_hash_comment_whole_line() {
        assert_compiles_to("# this is a comment\n0: srl", "srl");
    }

    #[test]
    fn test_double_slash_comment() {
        assert_compiles_to("0: srl // comment", "srl");
    }

    #[test]
    fn test_comment_with_macro() {
        assert_compiles_to("0: x:ss x # macro test", "ss");
    }

    #[test]
    fn test_multiple_comment_lines() {
        assert_compiles_to("# comment 1\n# comment 2\n0: srl", "srl");
    }
}

// =============================================================================
// 9. Multi-Agent Tests (hlang2 Extension)
// =============================================================================

mod multi_agent {
    use super::*;

    #[test]
    fn test_two_agents() {
        let result = compile_native("0: srl\n1: lrs");
        match result {
            CompileResult::Success { program } => {
                assert_eq!(program.agents.len(), 2);
                assert_eq!(program.agents[0].id, 0);
                assert_eq!(program.agents[1].id, 1);
            }
            CompileResult::Error { errors } => {
                panic!("Failed: {:?}", errors);
            }
        }
    }

    #[test]
    fn test_three_agents() {
        let result = compile_native("0: srl\n1: lrs\n2: rsl");
        match result {
            CompileResult::Success { program } => {
                assert_eq!(program.agents.len(), 3);
            }
            CompileResult::Error { errors } => {
                panic!("Failed: {:?}", errors);
            }
        }
    }

    #[test]
    fn test_agents_different_lengths() {
        let result = compile_native("0: ssssss\n1: rr");
        match result {
            CompileResult::Success { program } => {
                assert_eq!(program.max_steps, 6);
                assert_eq!(program.agents[0].commands.len(), 6);
                assert_eq!(program.agents[1].commands.len(), 2);
            }
            CompileResult::Error { errors } => {
                panic!("Failed: {:?}", errors);
            }
        }
    }

    #[test]
    fn test_timeline_parallel() {
        let result = compile_native("0: sr\n1: ls");
        match result {
            CompileResult::Success { program } => {
                // Step 0 should have commands from both agents
                assert_eq!(program.timeline[0].agent_commands.len(), 2);
                // Step 1 should also have commands from both agents
                assert_eq!(program.timeline[1].agent_commands.len(), 2);
            }
            CompileResult::Error { errors } => {
                panic!("Failed: {:?}", errors);
            }
        }
    }

    #[test]
    fn test_agent_with_macro() {
        let result = compile_native("0: x:ss xrx\n1: y:rr yly");
        match result {
            CompileResult::Success { program } => {
                assert_eq!(program.agents[0].commands.len(), 5); // ssrss
                assert_eq!(program.agents[1].commands.len(), 5); // rrlrr
            }
            CompileResult::Error { errors } => {
                panic!("Failed: {:?}", errors);
            }
        }
    }

    #[test]
    fn test_agent_id_10() {
        let result = compile_native("10: srl");
        match result {
            CompileResult::Success { program } => {
                assert_eq!(program.agents[0].id, 10);
            }
            CompileResult::Error { errors } => {
                panic!("Failed: {:?}", errors);
            }
        }
    }

    #[test]
    fn test_non_sequential_agent_ids() {
        let result = compile_native("0: s\n5: r\n10: l");
        match result {
            CompileResult::Success { program } => {
                assert_eq!(program.agents.len(), 3);
                assert_eq!(program.agents[0].id, 0);
                assert_eq!(program.agents[1].id, 5);
                assert_eq!(program.agents[2].id, 10);
            }
            CompileResult::Error { errors } => {
                panic!("Failed: {:?}", errors);
            }
        }
    }
}

// =============================================================================
// 10. HOJ Real Problem Pattern Tests
// =============================================================================

mod hoj_patterns {
    use super::*;

    #[test]
    fn test_hoj_simple_line() {
        // Most basic: move in a straight line
        // Problem type: reach target in N steps
        assert_compiles_to("0: ssss", "ssss");
    }

    #[test]
    fn test_hoj_turn_and_go() {
        // Turn right and go
        assert_compiles_to("0: rss", "rss");
    }

    #[test]
    fn test_hoj_square_small() {
        // 2x2 square: sr sr sr sr
        assert_compiles_to("0: f(X):XXXX f(sr)", "srsrsrsr");
    }

    #[test]
    fn test_hoj_square_medium() {
        // 3-step square: sssr sssr sssr sssr
        let result = compile_to_count("0: f(X):XXXX f(sssr)");
        assert_eq!(result.unwrap(), 16);
    }

    #[test]
    fn test_hoj_triangle() {
        // Triangle with 120-degree approximation
        // f(X):XXX f(ssr) = ssr ssr ssr
        assert_compiles_to("0: f(X):XXX f(ssr)", "ssrssrssr");
    }

    #[test]
    fn test_hoj_zigzag() {
        // Zigzag pattern: sr sl sr sl
        assert_compiles_to("0: f(X):XX f(srsl)", "srslsrsl");
    }

    #[test]
    fn test_hoj_spiral_out() {
        // Spiral outward: increasing steps
        // This is a classic HOJ pattern
        // s r ss r sss r ssss r
        // Using numeric: a(X):repeat(X,s)ra(X+1) but we need repeat function
        // Simpler: manual spiral
        assert_compiles_to("0: srsslsssrssssl", "srsslsssrssssl");
    }

    #[test]
    fn test_hoj_repeat_pattern() {
        // Repeat a command N times using recursion
        // a(X):sa(X-1) a(5) = sssss
        assert_compiles_to("0: a(X):sa(X-1) a(5)", "sssss");
    }

    #[test]
    fn test_hoj_repeat_sequence() {
        // Repeat a sequence N times
        // a(X,Y):Ya(X-1,Y) a(4,srl) = srl srl srl srl
        assert_compiles_to("0: a(X,Y):Ya(X-1,Y) a(4,srl)", "srlsrlsrlsrl");
    }

    #[test]
    fn test_hoj_nested_squares() {
        // Nested squares: outer and inner
        // f(X):XXXX f(f(s)r)
        // f(s) = ssss, f(s)r = ssssr
        // f(f(s)r) = ssssrssssrssssrsssssrssssr... wait
        // Let's simplify: f(X):XX f(sr) = srsr (4 commands)
        assert_compiles_to("0: f(X):XX f(sr)", "srsr");
    }

    #[test]
    fn test_hoj_l_shape() {
        // L-shape: go forward, turn, go forward
        assert_compiles_to("0: sssrss", "sssrss");
    }

    #[test]
    fn test_hoj_comb_pattern() {
        // Comb: repeatedly go and return
        // srrs srrs srrs = going right, returning left
        assert_compiles_to("0: f(X):XXX f(srrs)", "srrssrrssrrs");
    }

    #[test]
    fn test_hoj_star_pattern() {
        // Star-like: go out and back multiple directions
        // This creates a + shape
        assert_compiles_to("0: ssllssllssllssll", "ssllssllssllssll");
    }

    #[test]
    fn test_hoj_double_loop() {
        // Double loop using two functions
        // f(X):XXXX g(X):XX f(g(s))
        // g(s) = ss
        // f(ss) = ssssssss (8)
        assert_compiles_to("0: f(X):XXXX g(X):XX f(g(s))", "ssssssss");
    }

    #[test]
    fn test_hoj_parameterized_square() {
        // Square with variable side length using numeric parameter
        // side(N):s side(N-1) square:side(3)r side(3)r side(3)r side(3)r
        // This is complex; let's use simpler version
        // a(X):sa(X-1) f(Y):a(Y)r a(Y)r a(Y)r a(Y)r but this won't work directly

        // Simpler: direct numeric square
        // a(X,Y):Ya(X-1,Y) a(3,s) = sss
        // Then manually: a(3,s)r a(3,s)r a(3,s)r a(3,s)r
        // But we can't call a() multiple times easily

        // Let's test a pattern that does work:
        // f(X):XXXX creates 4 copies, combine with numeric repeat
        assert_compiles_to("0: a(X):sa(X-1) f(Y):YYYY f(a(3)r)", "sssrsssrsssrsssr");
    }

    #[test]
    fn test_hoj_complex_recursion() {
        // Complex: f(X):XXX with g(X):sXr
        // g(s) = ssr, f(g(s)) = f(ssr) = ssrssrssr (9)
        assert_compiles_to("0: f(X):XXX g(X):sXr f(g(s))", "ssrssrssr");
    }
}

// =============================================================================
// 11. Edge Cases and Boundary Tests
// =============================================================================

mod edge_cases {
    use super::*;

    #[test]
    fn test_single_command_single_agent() {
        assert_compiles_to("0: s", "s");
    }

    #[test]
    fn test_very_long_direct_commands() {
        // 100 straight commands
        let source = format!("0: {}", "s".repeat(100));
        let result = compile_to_count(&source);
        assert_eq!(result.unwrap(), 100);
    }

    #[test]
    fn test_many_macros() {
        // Define many macros and use them
        let source = "0: a:s b:r c:l d:ss e:rr f:ll abcdef";
        let result = compile_to_string(source);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "srlssrrll");
    }

    #[test]
    fn test_macro_shadowing_in_sequence() {
        // Multiple definitions of same macro - last one wins? Or first?
        // In HOJ, typically first definition is used
        // Our implementation might differ - let's test
        let result = compile_to_string("0: a:s a:r a");
        // This behavior may vary; just ensure it doesn't crash
        assert!(result.is_ok());
    }

    #[test]
    fn test_function_with_empty_result() {
        // f(X):f(X-1) f(1) -> f(0) -> empty
        assert_compiles_to("0: f(X):f(X-1) f(1)", "");
    }

    #[test]
    fn test_numeric_boundary_1() {
        assert_compiles_to("0: f(X):sf(X-1) f(1)", "s");
    }

    #[test]
    fn test_numeric_boundary_0() {
        assert_compiles_to("0: f(X):sf(X-1) f(0)", "");
    }

    #[test]
    fn test_numeric_boundary_minus_1() {
        assert_compiles_to("0: f(X):sf(X-1) f(-1)", "");
    }

    #[test]
    fn test_deeply_nested_macros() {
        // a -> b -> c -> d -> s
        let source = "0: a:b b:c c:d d:s a";
        assert_compiles_to(source, "s");
    }

    #[test]
    fn test_function_calling_macro() {
        // f(X):aX a:s f(r) -> sr
        assert_compiles_to("0: a:s f(X):aX f(r)", "sr");
    }

    #[test]
    fn test_macro_using_function() {
        // This is tricky: macro defined before function
        // a:f(s) f(X):XX a -> f(s) -> ss
        assert_compiles_to("0: f(X):XX a:f(s) a", "ss");
    }

    #[test]
    fn test_parameter_name_x() {
        assert_compiles_to("0: f(X):X f(s)", "s");
    }

    #[test]
    fn test_parameter_name_y() {
        assert_compiles_to("0: f(Y):Y f(s)", "s");
    }

    #[test]
    fn test_parameter_name_a() {
        assert_compiles_to("0: f(A):A f(s)", "s");
    }

    #[test]
    fn test_parameter_name_z() {
        assert_compiles_to("0: f(Z):Z f(s)", "s");
    }

    #[test]
    fn test_all_uppercase_params() {
        // Use various uppercase letters as parameters
        assert_compiles_to("0: f(A,B,C):ABC f(s,r,l)", "srl");
    }
}

// =============================================================================
// 12. Regression Tests (Known Issues)
// =============================================================================

mod regression {
    use super::*;

    #[test]
    fn test_function_not_mistaken_for_definition() {
        // f(s) should be a function call, not a definition attempt
        assert_compile_error("0: f(s)"); // undefined function
    }

    #[test]
    fn test_macro_body_ends_at_space() {
        // x:ss yyy should define x as "ss", not "ss yyy"
        // But "yyy" would be undefined macros
        assert_compile_error("0: x:ss yyy");
    }

    #[test]
    fn test_number_at_start_is_agent_id() {
        // "12: s" should be agent 12 with command s
        let result = compile_native("12: s");
        match result {
            CompileResult::Success { program } => {
                assert_eq!(program.agents[0].id, 12);
            }
            CompileResult::Error { errors } => {
                panic!("Failed: {:?}", errors);
            }
        }
    }

    #[test]
    fn test_number_in_function_is_numeric_arg() {
        // "a(4)" should pass 4 as numeric argument
        assert_compiles_to("0: a(X):sa(X-1) a(4)", "ssss");
    }

    #[test]
    fn test_complex_nested_call() {
        // f(X):XX g(Y):YYY f(g(s))
        // g(s) = sss
        // f(sss) = ssssss
        assert_compiles_to("0: f(X):XX g(Y):YYY f(g(s))", "ssssss");
    }

    #[test]
    fn test_parameter_in_recursive_call() {
        // a(X,Y):Ya(X-1,Y) a(2,sr)
        // a(2,sr) -> sr + a(1,sr) -> sr + sr + a(0,sr) -> srsr
        assert_compiles_to("0: a(X,Y):Ya(X-1,Y) a(2,sr)", "srsr");
    }
}

// =============================================================================
// 13. Stress Tests (Performance/Limits)
// =============================================================================

mod stress {
    use super::*;

    #[test]
    fn test_recursion_depth_80() {
        // Test near the default max_depth of 100
        let result = compile_to_count("0: a(X):sa(X-1) a(80)");
        assert_eq!(result.unwrap(), 80);
    }

    #[test]
    fn test_many_agents_10() {
        let source: String = (0..10).map(|i| format!("{}: s", i)).collect::<Vec<_>>().join("\n");
        let result = compile_native(&source);
        match result {
            CompileResult::Success { program } => {
                assert_eq!(program.agents.len(), 10);
            }
            CompileResult::Error { errors } => {
                panic!("Failed: {:?}", errors);
            }
        }
    }

    #[test]
    fn test_deeply_nested_function_8_levels() {
        // f(X):XX with 8 levels of nesting
        // f(f(f(f(f(f(f(f(s)))))))) = 2^8 = 256 commands
        let result = compile_to_count("0: f(X):XX f(f(f(f(f(f(f(f(s))))))))");
        assert_eq!(result.unwrap(), 256);
    }

    #[test]
    fn test_combined_macro_function_complexity() {
        // Mix of macros and functions
        let source = "0: a:sr b:rs f(X):XXX g(Y):YY af(g(b))a";
        // g(b) = g(rs) = rsrs
        // f(g(b)) = f(rsrs) = rsrsrsrsrsrs (12)
        // a f(...) a = sr + 12 + sr = 16
        let result = compile_to_count(source);
        assert_eq!(result.unwrap(), 16);
    }
}
