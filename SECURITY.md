# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |

## Reporting a Vulnerability

We take the security of H2 Language seriously. If you believe you have found a security vulnerability, please report it to us as described below.

### How to Report

**Please do not report security vulnerabilities through public GitHub issues.**

Instead, please report them via email to: **security@example.com**

You should receive a response within 48 hours. If for some reason you do not, please follow up via email to ensure we received your original message.

Please include the following information in your report:

- Type of issue (e.g., buffer overflow, infinite loop, resource exhaustion)
- Full paths of source file(s) related to the issue
- Location of the affected source code (tag/branch/commit or direct URL)
- Any special configuration required to reproduce the issue
- Step-by-step instructions to reproduce the issue
- Proof-of-concept or exploit code (if possible)
- Impact of the issue, including how an attacker might exploit it

### What to Expect

- **Initial Response**: Within 48 hours
- **Status Update**: Within 7 days
- **Resolution Timeline**: Typically within 90 days

### Disclosure Policy

- We will acknowledge receipt of your vulnerability report
- We will confirm the vulnerability and determine its impact
- We will release a fix as soon as possible, depending on complexity
- We will publicly disclose the vulnerability after a fix is available

### Safe Harbor

We consider security research conducted in accordance with this policy to be:

- Authorized concerning any applicable anti-hacking laws
- Authorized concerning any relevant anti-circumvention laws
- Exempt from restrictions in our Terms of Service that would interfere with conducting security research

We will not pursue civil action or initiate a complaint to law enforcement for accidental, good faith violations of this policy.

## Security Considerations for H2 Language

### Recursion Limits

The H2 Language compiler has a maximum recursion depth of 100 to prevent stack overflow from deeply nested macro/function expansions. If you find a way to bypass this limit, please report it.

### Resource Limits

When using H2 Language in untrusted environments (e.g., web playgrounds), consider:

- **Expansion limits**: Large programs may produce many commands
- **Memory usage**: Complex recursive patterns may consume significant memory
- **Execution time**: Some patterns may take longer to compile

### WebAssembly Security

The WASM build runs in a sandboxed environment. However:

- Input validation should be performed before passing to the compiler
- Output size should be monitored to prevent memory exhaustion
- Consider implementing timeouts for compilation in web environments

## Acknowledgments

We thank the following individuals for responsibly disclosing security issues:

- *No acknowledgments yet*
