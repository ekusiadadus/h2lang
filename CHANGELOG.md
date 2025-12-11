# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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

[Unreleased]: https://github.com/ekusiadadus/h2lang/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/ekusiadadus/h2lang/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/ekusiadadus/h2lang/releases/tag/v0.1.0
