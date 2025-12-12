# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.5.1] - 2025-12-12

### Fixed

- **HOJ compatibility: Empty call exception restored**
  - `a(X):X a()` now works (binds X to empty CmdSeq)
  - `a(X):Xrra(sX) a()` generates increasing sequences from empty
  - `a(X):sa(X-1) a()` terminates immediately (X=0)
  - Default values: CmdSeq → empty, Int → 0

### Changed

- Expander now stores `param_types` for HOJ-compatible default binding

## [0.5.0] - 2025-12-12

### Breaking Changes

- **Unified function model** - Macros are now 0-arg functions
  - `x:ss` is parsed as a 0-arg function, not a separate "macro"
  - `Definition::Macro` removed from AST
  - `Expr::Ident` removed, replaced by `Expr::FuncCall { args: vec![] }`
- **Strict arity checking** - `a()` on function with params is now E003
  - Previous behavior: `a(X):X a()` returned empty (special case)
  - New behavior: `a(X):X a()` returns E003 (arity mismatch)
- **AgentId requires immediate colon**
  - `0:` → `AGENT_ID(0)`
  - `0 :` → `NUMBER(0)`, `SPACE`, `COLON` (no longer AgentId)

### Added

- **Parameter type inference at definition time** - Types inferred from body usage
  - `f(X):XX` → X is CmdSeq (used as term)
  - `f(X):sf(X-1)` → X is Int (used in num_expr)
- **E010 error code** - Type conflict when parameter used as both CmdSeq and Int
- **`f():ss` syntax** - 0-arg functions with explicit empty parentheses
- **Specification v0.5.0** - Complete language specification

### Changed

- AST simplified: `Expr::FuncCall { name, args, span }` unified model
- `FuncDef` now includes `param_types: HashMap<char, ParamType>`
- Expander uses single `functions` map instead of separate `macros` map

## [0.3.0] - 2025-12-12

### Added

- **Extended numeric expressions** - HOJ-compatible chained arithmetic in function arguments
  ```h2
  a(X):sa(X-1) a(10-3+1)   # evaluates to 8
  a(X):sa(X-1) a(5+5-2)    # evaluates to 8
  ```
- **MAX_STEP directive** - Limit generated commands to prevent runaway expansions
  ```h2
  MAX_STEP=1000
  ON_LIMIT=TRUNCATE
  a(X):sa(X-1) a(999999)   # truncates at 1000 commands
  ```
- **MAX_DEPTH directive** - Limit recursion depth (default: 100)
- **ON_LIMIT directive** - Control behavior when limits exceeded (ERROR or TRUNCATE)
- **E003 error code** - Argument count mismatch validation
- **E007 error code** - Numeric range validation (-255..255)
- **E008 error code** - Type checking for Int vs CmdSeq parameters
- **HOJ conformance tests** - 31 tests for HOJ Ruby implementation compatibility

### Fixed

- **AgentId detection** - Numbers at line start now require `:` to be AgentId
- **Directive parsing** - Leading spaces before directives now handled correctly
- **Parameter passthrough** - Numeric parameters properly passed in recursive calls
- **Stack safety** - Improved handling of deep recursion patterns

### Changed

- Parser buffer changed from `Vec` to `VecDeque` for O(1) lookahead performance
- Specification updated to v0.4.0 with HOJ compatibility documentation

## [0.2.1] - 2025-12-11

### Fixed

- **Multiline support for single-agent programs** - Programs without `0:` prefix can now span multiple lines
  ```h2
  # This now works (was previously an error)
  a: ssrs
  aaaaaaaaaa
  ```
- **Multiline support for multi-agent programs** - Each agent's code continues until the next agent ID
  ```h2
  0: a:ssrs
  aaaa        # still part of agent 0
  1: srl      # agent 1 starts here
  ```

## [0.2.0] - 2025-12-11

### Added

- **Optional agent prefix for single-agent programs** - Programs with a single agent can now omit the `0:` prefix
  ```h2
  # Before (still works)
  0: f(X):XXXX f(sssr)

  # After (also works)
  f(X):XXXX f(sssr)
  ```
- Multi-agent programs still require explicit agent IDs (`0:`, `1:`, etc.)

### Changed

- Parser now auto-detects single-agent vs multi-agent mode based on first token

## [0.1.0] - 2025-12-10

### Added

- Initial release of H2 Language compiler
- Full compatibility with Herbert Online Judge (HOJ) H language specification
- Lexer with support for basic commands (`s`, `r`, `l`), macros, and functions
- Recursive descent parser for H language syntax
- AST (Abstract Syntax Tree) representation
- Macro and function expansion with parameter substitution
- Numeric argument support with arithmetic expressions (`X-1`, `X+1`)
- Multi-agent support for controlling multiple robots simultaneously
- Parallel scheduling system for multi-agent execution
- WebAssembly (WASM) compilation target via wasm-bindgen
- JSON output format for compiled programs
- Comprehensive error handling with line/column information
- 217 unit tests including 145 HOJ compatibility tests

### Technical Details

- Written in Rust 1.70+
- Zero-dependency runtime
- Optimized WASM build with LTO enabled
- TypeScript type definitions included

[Unreleased]: https://github.com/ekusiadadus/h2lang/compare/v0.5.1...HEAD
[0.5.1]: https://github.com/ekusiadadus/h2lang/compare/v0.5.0...v0.5.1
[0.5.0]: https://github.com/ekusiadadus/h2lang/compare/v0.3.0...v0.5.0
[0.3.0]: https://github.com/ekusiadadus/h2lang/compare/v0.2.1...v0.3.0
[0.2.1]: https://github.com/ekusiadadus/h2lang/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/ekusiadadus/h2lang/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/ekusiadadus/h2lang/releases/tag/v0.1.0
