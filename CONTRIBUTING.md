# Contributing to H2 Language

Thank you for your interest in contributing to H2 Language (h2lang)! This document provides guidelines and instructions for contributing.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [How to Contribute](#how-to-contribute)
- [Development Setup](#development-setup)
- [Pull Request Process](#pull-request-process)
- [Style Guidelines](#style-guidelines)

## Code of Conduct

This project adheres to the [Contributor Covenant Code of Conduct](CODE_OF_CONDUCT.md). By participating, you are expected to uphold this code.

## Getting Started

1. Fork the repository
2. Clone your fork: `git clone https://github.com/YOUR_USERNAME/h2lang.git`
3. Add the upstream remote: `git remote add upstream https://github.com/ekusiadadus/h2lang.git`
4. Create a new branch: `git checkout -b feature/your-feature-name`

## How to Contribute

### Reporting Bugs

- Check existing issues to avoid duplicates
- Use the [bug report template](.github/ISSUE_TEMPLATE/bug_report.md)
- Provide clear reproduction steps
- Include your environment details (OS, Rust version, browser)

### Suggesting Features

- Use the [feature request template](.github/ISSUE_TEMPLATE/feature_request.md)
- Explain the use case and expected behavior
- Consider H language compatibility requirements

### Contributing Code

1. Ensure your code follows the project's style guidelines
2. Write tests for new functionality
3. Update documentation as needed
4. Submit a pull request

## Development Setup

### Prerequisites

- Rust 1.70+ with `wasm32-unknown-unknown` target
- wasm-pack
- Node.js 18+ (for web playground)

### Building

```bash
# Build the WebAssembly module
wasm-pack build --target web

# Run tests
cargo test

# Run H language compatibility tests
cargo test --test h_language_compatibility
```

### Testing

We follow Test-Driven Development (TDD) principles:

1. Write tests first
2. Implement the feature
3. Refactor as needed

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run tests with output
cargo test -- --nocapture
```

## Pull Request Process

1. **Create a feature branch** from `main`
2. **Write tests** for your changes
3. **Ensure all tests pass**: `cargo test`
4. **Format your code**: `cargo fmt`
5. **Check for lint errors**: `cargo clippy`
6. **Update documentation** if needed
7. **Submit a pull request** using the PR template

### PR Requirements

- [ ] Tests pass locally
- [ ] Code is formatted (`cargo fmt`)
- [ ] No clippy warnings (`cargo clippy`)
- [ ] Documentation updated (if applicable)
- [ ] Commit messages follow conventions

## Style Guidelines

### Rust Code

- Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `cargo fmt` for formatting
- Use `cargo clippy` for linting
- Write documentation comments for public APIs

### Commit Messages

Follow the [Conventional Commits](https://www.conventionalcommits.org/) specification:

```
type(scope): description

[optional body]

[optional footer]
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, etc.)
- `refactor`: Code refactoring
- `test`: Adding or updating tests
- `chore`: Maintenance tasks

Examples:
```
feat(lexer): add support for nested blocks
fix(interpreter): correct Hello World output encoding
docs(readme): update installation instructions
test(parser): add edge case tests for expressions
```

## Questions?

If you have questions, feel free to:
- Open a [discussion](https://github.com/ekusiadadus/h2lang/discussions)
- Create an issue with the `question` label

Thank you for contributing!
