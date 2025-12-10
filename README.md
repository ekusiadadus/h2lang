# H2 Language

[![CI](https://github.com/ekusiadadus/h2lang/actions/workflows/ci.yml/badge.svg)](https://github.com/ekusiadadus/h2lang/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![WebAssembly](https://img.shields.io/badge/wasm-supported-blueviolet.svg)](https://webassembly.org/)
[![Crates.io](https://img.shields.io/crates/v/h2lang.svg)](https://crates.io/crates/h2lang)
[![Documentation](https://docs.rs/h2lang/badge.svg)](https://docs.rs/h2lang)

<!-- Uncomment after publishing to npm -->
<!-- [![npm](https://img.shields.io/npm/v/h2lang.svg)](https://www.npmjs.com/package/h2lang) -->

**H2 Language** (h2lang) is a programming language compiler fully compatible with the [Herbert Online Judge (HOJ)](https://github.com/quolc/hoj) H language specification, extended with multi-agent support for robot swarm control.

## Features

- **HOJ Compatible** - Full support for Herbert Online Judge syntax including macros, functions, and recursion
- **Multi-Agent Support** - Control multiple robots simultaneously with parallel scheduling
- **WebAssembly Ready** - Runs in browsers via wasm-bindgen
- **Zero Dependencies Runtime** - Lightweight compiled output
- **Type-Safe** - Written in Rust with comprehensive error handling
- **Well Documented** - Extensive documentation following Rust API guidelines

## Quick Start

```h2
# Single robot drawing a square
0: f(X):XXXX f(sssr)

# Two robots moving in parallel
0: srl
1: lrs

# Recursive pattern with numeric argument
0: a(X):sra(X-1) a(4)
```

## Installation

### From crates.io (Rust)

```bash
cargo add h2lang
```

### From npm (WebAssembly)

```bash
npm install h2lang
```

### From Source

```bash
# Clone the repository
git clone https://github.com/ekusiadadus/h2lang.git
cd h2lang

# Build WebAssembly package
wasm-pack build --target web --out-dir pkg

# Or build native library
cargo build --release

# Run tests
cargo test
```

### Requirements

- Rust 1.70+
- wasm-pack (for WebAssembly builds)
- Node.js 16+ (for npm usage)

## Language Specification

### Basic Commands

| Command | Description | Action |
|---------|-------------|--------|
| `s` | **S**traight | Move forward one step |
| `r` | **R**ight | Rotate 90° clockwise |
| `l` | **L**eft | Rotate 90° counter-clockwise |

### Agent Definition

Each line defines commands for a specific robot (agent):

```h2
agent_id: commands

# Examples
0: srl        # Agent 0: straight, right, left
1: llss       # Agent 1: left, left, straight, straight
```

### Macros

Define reusable command sequences with single lowercase letters:

```h2
# Syntax: name:body
x:ss          # Define macro 'x' as 'ss'
xrx           # Expands to: ssrss

# Full example
0: x:sssr xrxrxrx   # Square pattern using macro
```

### Functions

Functions support parameters (uppercase letters) and recursion:

```h2
# Syntax: name(PARAMS):body
f(X):XXX f(s)           # Repeats argument 3 times → sss
f(X):XXXX f(sssr)       # Square pattern → sssrsssrsssrsssr

# Multiple parameters
a(X,Y):Ya(X-1,Y) a(4,s) # Repeat 's' four times → ssss

# Numeric arguments with recursion
a(X):sra(X-1) a(4)      # Spiral pattern (terminates when X ≤ 0)
```

### Numeric Expressions

Functions support numeric arguments and arithmetic:

```h2
a(X):sa(X-1) a(4)       # X decrements: 4→3→2→1→0(stop)
a(X):sa(X+1) a(-2)      # X increments: -2→-1→0(stop)
```

**Termination Rule**: When a numeric argument is ≤ 0, the function returns empty (recursion stops).

### Comments

```h2
# This is a comment
0: srl  # Inline comment
// C-style comments also work
```

### Implementation Notes

#### Whitespace Handling

- **Agent ID**: Agent IDs must appear at the start of a line. Leading spaces/tabs before the agent ID are permitted and treated as line-start context.
- **Function/Macro Definitions**: No whitespace is allowed between the identifier and `(` in function definitions (e.g., `f(X):...` is valid, `f (X):...` is not).
- **Spaces**: Spaces and tabs between tokens are generally ignored except where they affect line-start detection.

#### Recursion and Termination

- **Maximum Recursion Depth**: The expander has a maximum recursion depth of **100** to prevent stack overflow from deeply nested macro/function calls. Exceeding this limit results in an expansion error.
- **Numeric Termination**: When any numeric argument becomes ≤ 0, the function call returns an empty sequence (no commands). This applies to all numeric parameters in the function.
- **Expansion Limit**: There is no explicit limit on the total number of expanded commands, but deeply recursive patterns may hit the depth limit first.

#### Error Handling

Compilation errors include:
- **Line and column information** for precise error location
- **Expected vs. found tokens** for parse errors
- **Undefined macro/function references**
- **Maximum recursion depth exceeded**

## Examples

### Drawing Shapes

```h2
# Square (4 sides)
0: f(X):XXXX f(sssr)

# Triangle (3 sides)
0: f(X):XXX f(ssssrr)

# Spiral
0: a(X):sra(X-1) a(8)
```

### Multi-Robot Choreography

```h2
# Two robots moving in mirror pattern
0: srlsrl
1: slrslr

# Three robots with different patterns
0: f(X):XXXX f(sr)
1: f(X):XXXX f(sl)
2: ssssssss
```

### Complex Recursion

```h2
# Nested function calls
0: f(X):XX f(f(s))      # f(s)=ss, f(ss)=ssss → 4 commands

# Parameterized repetition
0: a(X,Y):Ya(X-1,Y) a(3,sr)  # srsrsr (repeat 'sr' 3 times)
```

## API Reference

### Rust (Native)

```rust
use h2lang::compile_native;
use h2lang::output::CompileResult;

let result = compile_native("0: srl\n1: lrs");

match result {
    CompileResult::Success { program } => {
        println!("Agents: {}", program.agents.len());
        println!("Max steps: {}", program.max_steps);
        for entry in &program.timeline {
            println!("Step {}: {:?}", entry.step, entry.agent_commands);
        }
    }
    CompileResult::Error { errors } => {
        for err in errors {
            eprintln!("Error at {}:{}: {}", err.line, err.column, err.message);
        }
    }
}
```

### JavaScript/TypeScript (WebAssembly)

```javascript
import init, { compile, validate, version } from 'h2lang';

await init();

// Compile source code
const result = compile('0: srl');
if (result.status === 'success') {
  console.log(result.program.timeline);  // Parallel execution timeline
  console.log(result.program.agents);    // Per-agent command lists
}

// Validate without compiling
const validation = validate('0: srl');
console.log(validation.valid);  // true or false

// Get compiler version
console.log(version());  // "0.1.0"
```

### Output Format

The compiler produces a JSON structure:

```json
{
  "status": "success",
  "program": {
    "agents": [
      {
        "id": 0,
        "commands": [
          {"type": "straight", "steps": 1},
          {"type": "rotate_right", "angle": 90},
          {"type": "rotate_left", "angle": -90}
        ]
      }
    ],
    "max_steps": 3,
    "timeline": [
      {
        "step": 0,
        "agent_commands": [
          {"agent_id": 0, "command": {"type": "straight", "steps": 1}}
        ]
      }
    ]
  }
}
```

## Building from Source

### WebAssembly Build

```bash
# Install wasm-pack if not already installed
cargo install wasm-pack

# Build for web
npm run build
# or
wasm-pack build --target web --out-dir pkg

# Build for Node.js
wasm-pack build --target nodejs --out-dir pkg
```

### Native Build

```bash
# Debug build
cargo build

# Release build
cargo build --release

# Run tests
cargo test

# Generate documentation
cargo doc --no-deps --open
```

## Testing

```bash
# Run all tests (241 tests)
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test module
cargo test basic_commands

# WebAssembly tests (requires Chrome)
wasm-pack test --headless --chrome
```

## Project Structure

```
h2lang/
├── .github/
│   ├── workflows/
│   │   └── ci.yml              # CI pipeline (fmt, clippy, test, wasm)
│   ├── ISSUE_TEMPLATE/         # Issue templates
│   └── PULL_REQUEST_TEMPLATE.md
├── src/
│   ├── lib.rs          # Main entry point, WASM bindings
│   ├── lexer.rs        # Tokenizer
│   ├── parser.rs       # Recursive descent parser
│   ├── ast.rs          # Abstract Syntax Tree definitions
│   ├── expander.rs     # Macro/function expansion
│   ├── scheduler.rs    # Multi-agent parallel scheduling
│   ├── output.rs       # JSON output structures
│   ├── token.rs        # Token definitions
│   └── error.rs        # Error types
├── tests/
│   └── h_language_compatibility.rs  # 145 HOJ compatibility tests
├── Cargo.toml          # Rust dependencies
├── package.json        # npm configuration
├── rust-toolchain.toml # Rust toolchain configuration
├── CONTRIBUTING.md     # Contribution guidelines
├── CODE_OF_CONDUCT.md  # Community guidelines (Contributor Covenant)
├── CHANGELOG.md        # Version history
└── LICENSE             # MIT License
```

### Architecture

```
Source Code → Lexer → Parser → AST → Expander → Scheduler → Output
     ↓          ↓        ↓       ↓        ↓          ↓         ↓
   "0:srl"   Tokens   Parse   Tree   Commands   Timeline    JSON
                      Tree          (expanded)  (parallel)
```

## Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

### Quick Contribution Guide

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Write tests for new functionality
4. Ensure all tests pass (`cargo test`)
5. Format code (`cargo fmt`)
6. Run linter (`cargo clippy`)
7. Commit with clear messages following [Conventional Commits](https://www.conventionalcommits.org/)
8. Open a Pull Request

### Development Commands

```bash
# Format code
cargo fmt

# Run linter
cargo clippy -- -D warnings

# Run all checks before PR
cargo fmt --check && cargo clippy -- -D warnings && cargo test
```

## Community

- [GitHub Issues](https://github.com/ekusiadadus/h2lang/issues) - Bug reports and feature requests
- [GitHub Discussions](https://github.com/ekusiadadus/h2lang/discussions) - Questions and ideas

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Related Links

- [Herbert Online Judge](http://herbert.tealang.info/) - Original HOJ platform
- [HOJ GitHub](https://github.com/quolc/hoj) - HOJ source code
- [Codeforces Discussion](https://codeforces.com/blog/entry/5579) - Community discussion
- [wasm-bindgen](https://rustwasm.github.io/wasm-bindgen/) - Rust/WebAssembly bindings

## Acknowledgments

- Herbert Online Judge by [@quolc](https://github.com/quolc)
- Microsoft ImagineCup for the original Herbert game concept
- The Rust community for excellent tooling
