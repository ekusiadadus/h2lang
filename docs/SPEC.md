# H2 Language Specification v0.5.0

## 1. Overview

H2 Language is a domain-specific language for robot control, inspired by HOJ (Herbert Online Judge) H language,
with extensions for multi-agent support and execution limits.

**Design Philosophy**:
- Simple, unambiguous syntax
- Predictable type system with compile-time checking
- Safe execution with configurable limits

---

## 2. Lexical Structure

### 2.1 Character Sets

```
letter_lower   ::= 'a'..'z'
letter_upper   ::= 'A'..'Z'
digit          ::= '0'..'9'
command        ::= 's' | 'r' | 'l'
identifier     ::= letter_lower - command   (* a-z except s, r, l *)
parameter      ::= letter_upper             (* A-Z *)
```

### 2.2 Tokens

| Token | Pattern | Description |
|-------|---------|-------------|
| `COMMAND` | `s \| r \| l` | Basic movement commands |
| `IDENT` | `[a-z]` except `s,r,l` | Function identifiers |
| `PARAM` | `[A-Z]` | Function parameters |
| `NUMBER` | `[0-9]+` | Non-negative integer literals |
| `AGENT_ID` | `[0-9]+` at line start **immediately** followed by `:` | Agent identifier |
| Symbols | `: ( ) , + -` | Punctuation |
| `NEWLINE` | `\n \| \r\n` | Line terminator |
| `SPACE` | `[ \t]+` | Whitespace (H2 extension) |
| `COMMENT` | `#...` or `//...` | Comment to end of line (H2 extension) |

### 2.3 Lexical Rules

- **Longest match**: `12` is read as a single `NUMBER(12)` token.
- **AGENT_ID**: Valid only at line start AND **immediately** followed by `:` (no spaces).
  - `0:` → `AGENT_ID(0)`
  - `0 :` → `NUMBER(0)`, `SPACE`, `COLON` (NOT AgentId)
- **SPACE**: Whitespace between tokens is allowed (H2 extension, not HOJ-compatible).
- **COMMENT**: `#` or `//` starts a comment until end of line (H2 extension).

---

## 3. Syntax Grammar (EBNF)

### 3.1 Program Structure

```ebnf
program           ::= directives agent_content

directives        ::= (SPACE? directive NEWLINE)*

directive         ::= directive_name '=' directive_value
directive_name    ::= 'MAX_STEP' | 'MAX_DEPTH' | 'ON_LIMIT'
directive_value   ::= NUMBER | 'ERROR' | 'TRUNCATE'

agent_content     ::= agent_block | single_agent_block

agent_block       ::= agent (NEWLINE agent)*
agent             ::= AGENT_ID agent_body

single_agent_block ::= agent_body   (* treated as agent 0 *)
```

### 3.2 Agent Body (H2 Mode)

```ebnf
(* H2 Mode: definitions and expressions can be mixed *)
agent_body        ::= statement*

statement         ::= definition | term

definition        ::= func_def

func_def          ::= IDENT '(' param_list? ')' ':' expression
                    | IDENT ':' expression

param_list        ::= PARAM (',' PARAM)*
```

**H2 Mode Features**:
- Definitions and expressions can be **mixed** on the same line or across lines.
- No "last line = main" constraint.
- `f():ss` (0-arg with parentheses) is valid and equivalent to `f:ss`.

### 3.3 Expressions

```ebnf
expression        ::= term+

term              ::= COMMAND
                    | IDENT '(' arg_list? ')'   (* function call *)
                    | IDENT                      (* 0-arg function call *)
                    | PARAM                      (* parameter reference *)

arg_list          ::= argument (',' argument)*

argument          ::= num_expr                   (* numeric expression *)
                    | cmd_expr                   (* command expression *)

cmd_expr          ::= term+
```

**Important**: `IDENT` alone (e.g., `x`) is a **0-arg function call**, not a special "macro reference".

### 3.4 Numeric Expressions

```ebnf
num_expr          ::= num_atom (('+' | '-') num_atom)*

num_atom          ::= NUMBER
                    | PARAM              (* must be Int-typed parameter *)
```

**Parsing Rule**: If an argument starts with `NUMBER` or `PARAM` followed by `+`/`-`, it is parsed as `num_expr`.

---

## 4. Semantics

### 4.1 Evaluation Order

1. Parse directives and build limit configuration
2. Parse all definitions and register in function table
3. Parse main expression (all non-definition terms)
4. Expand main expression, generating command sequence

### 4.2 Function Definitions (Unified Model)

All definitions are functions:

| Syntax | Params | Example |
|--------|--------|---------|
| `x:ss` | 0 | 0-arg function |
| `f():ss` | 0 | 0-arg function (explicit) |
| `f(X):XXX` | 1 | 1-arg function |
| `g(A,B):AB` | 2 | 2-arg function |

**Constraints**:
- Same-name collision is **forbidden** (one definition per identifier).
- There is no separate "macro" concept; `x:ss` is simply a 0-arg function.

### 4.3 Type System

#### 4.3.1 Parameter Types

Each parameter has exactly one type, determined at **definition time**:

| Type | Description | Value Range |
|------|-------------|-------------|
| `CmdSeq` | Command sequence | Any sequence of s/r/l |
| `Int` | Integer value | -255..255 |

#### 4.3.2 Type Inference Rules (Definition Time)

Type is inferred by analyzing parameter usage in the function body:

| Usage | Inferred Type |
|-------|---------------|
| `PARAM` as term (e.g., `X` alone in body) | CmdSeq |
| `PARAM` in num_expr (e.g., `X-1`, `X+2`) | Int |

**Type Conflict**: If a parameter is used both ways → **E010** (definition-time error).

```
# Error: X used as both CmdSeq and Int
f(X):Xf(X-1)   # E010: Type conflict for parameter 'X'
```

#### 4.3.3 Type Storage

```rust
struct FuncDef {
    name: char,
    params: Vec<char>,
    param_types: HashMap<char, ParamType>,  // Inferred at definition
    body: Expr,
    span: Span,
}
```

#### 4.3.4 Call-Site Type Checking

At call site, argument type must match parameter type:

| Parameter Type | Valid Argument |
|----------------|----------------|
| CmdSeq | Command expression (terms) |
| Int | Numeric expression |

Type mismatch → **E008**.

```
f(X):XX f(sr)     # OK: X is CmdSeq, sr is CmdSeq
f(X):sf(X-1) f(3) # OK: X is Int, 3 is Int
f(X):XX f(3)      # E008: X is CmdSeq, but 3 is Int
f(X):sf(X-1) f(sr)# E008: X is Int, but sr is CmdSeq
```

### 4.4 Function Call Semantics

#### 4.4.1 Arity Check

Argument count must match parameter count, with one exception:

```
f(X,Y):XY f(s)    # E003: expects 2, got 1
f(X,Y):XY f(s,r,l)# E003: expects 2, got 3
```

**HOJ Compatibility - Empty Call Exception**:

When `f()` is called on a function with parameters, default values are bound:
- `CmdSeq` parameters → empty sequence
- `Int` parameters → 0 (triggers ≤0 termination)

```
a(X):X a()        # OK: X=empty → returns empty
a(X):Xrra(sX) a() # OK: generates rr s rr ss rr sss rr ...
a(X):sa(X-1) a()  # OK: X=0 → immediate termination → empty
```

This is a known HOJ technique for generating increasing sequences from empty.

#### 4.4.2 Numeric Termination

If ANY Int-typed parameter has value `<= 0`, the function call returns empty:

```
a(X):sa(X-1) a(4)
→ s a(3) → ss a(2) → sss a(1) → ssss a(0)
→ ssss (empty)   # X=0 terminates
```

#### 4.4.3 Expansion

1. Check arity: mismatch → E003
2. Type-check each argument: mismatch → E008
3. Check numeric termination: any Int param ≤ 0 → return empty
4. Substitute parameters and expand body recursively

### 4.5 Numeric Expression Evaluation

```
num_expr ::= num_atom (('+' | '-') num_atom)*
```

1. Substitute all `PARAM` with their bound Int values
2. Evaluate left-to-right: `12-3+4` → `9+4` → `13`
3. Range check at each step: `-255 ≤ result ≤ 255`, otherwise E007

---

## 5. Execution Limits

### 5.1 Directives

| Name | Type | Default | Range |
|------|------|---------|-------|
| `MAX_STEP` | int | 1,000,000 | 1..10,000,000 |
| `MAX_DEPTH` | int | 100 | 1..10,000 |
| `ON_LIMIT` | enum | TRUNCATE | ERROR / TRUNCATE |

**Note**: `MAX_MEMORY` is reserved for future use (not implemented in v0.5.0).

### 5.2 ON_LIMIT Behavior

| Value | Behavior |
|-------|----------|
| `ERROR` | Return error (E004/E005) and stop |
| `TRUNCATE` | Return commands generated so far |

**Default**: `TRUNCATE` (for HOJ compatibility).

### 5.3 Step Counting

H2 counts generated commands (`s`, `r`, `l`) only.

---

## 6. Multi-Agent Extension

### 6.1 Agent Definition

```
0: x:ss xx       # Agent 0
1: srl           # Agent 1
2: f(X):XX f(s)  # Agent 2
```

### 6.2 Single Agent Mode

When no `AGENT_ID:` prefix is present, the entire program is agent 0:

```
# These are equivalent:
0: f(X):XXXX f(sssr)
f(X):XXXX f(sssr)
```

### 6.3 Parallel Timeline

- Each agent executes independently
- Timestep `t`: each agent executes `cmd[agent][t]`
- If sequence exhausted: no-op
- Timeline length: `max(len(cmd[agent]))`

---

## 7. Error Codes

| Code | Description | When |
|------|-------------|------|
| E001 | Undefined function | 0-arg call to undefined function |
| E002 | Undefined function | N-arg call to undefined function |
| E003 | Argument count mismatch | `f(X,Y)` called with wrong arity |
| E004 | MAX_STEP exceeded | Step count > MAX_STEP |
| E005 | MAX_DEPTH exceeded | Recursion depth > MAX_DEPTH |
| E007 | Numeric out of range | Value outside -255..255 |
| E008 | Type error | CmdSeq/Int mismatch at call site |
| E009 | Invalid directive | Unknown directive name/value |
| E010 | Type conflict | Parameter used as both CmdSeq and Int |

**Note**: E006 (MAX_MEMORY exceeded) is reserved for future use.

---

## 8. Compatibility Notes

### 8.1 HOJ Compatibility

| Feature | HOJ | H2 |
|---------|-----|-----|
| Commands `s`, `r`, `l` | ✓ | ✓ |
| Function definitions | ✓ | ✓ |
| 0-arg functions | ✓ | ✓ |
| Numeric recursion | ✓ | ✓ |
| ≤0 termination | ✓ | ✓ |
| Complex num_expr | ✓ | ✓ |
| Fixed parameter types | ✓ | ✓ |

### 8.2 H2 Extensions

| Feature | Description |
|---------|-------------|
| Multi-agent | `0:`, `1:`, etc. |
| Directives | `MAX_STEP`, `MAX_DEPTH`, `ON_LIMIT` |
| Whitespace | Spaces allowed between tokens |
| Comments | `#` and `//` |
| Mixed structure | Definitions and expressions can be mixed |

### 8.3 Breaking Changes from v0.4.0

1. **AgentId requires immediate `:`**: `0 :` is no longer AgentId
2. **No `a()` special case**: Empty args always requires 0-param function
3. **E010 added**: Type conflict error for definition-time checking
4. **MAX_MEMORY removed**: Reserved for future implementation

---

## 9. Conformance Tests

See `tests/spec_conformance.rs` and `tests/hoj_conformance.rs`.

---

## 10. References

- [Herbert Online Judge](http://herbert.tealang.info/)
- [HOJ GitHub Repository](https://github.com/quolc/hoj)

---

## Appendix A: Migration from v0.4.0

### Breaking Changes

1. **AgentId lexing**: `0 :` is now `NUMBER SPACE COLON`, not `AGENT_ID`
2. **Arity check strict**: `f()` on function with params is E003 (no special case)
3. **Type conflict error**: E010 added for definition-time type conflicts
4. **MAX_MEMORY removed**: Directive parsing will reject it

### AST Changes

```rust
// v0.5.0 FuncDef with type information
struct FuncDef {
    name: char,
    params: Vec<char>,
    param_types: HashMap<char, ParamType>,  // NEW
    body: Expr,
    span: Span,
}

// v0.5.0 unified Definition (no Macro variant)
enum Definition {
    Function(FuncDef),
}

// Expr::Ident removed, use FuncCallArgs with empty args
enum Expr {
    Primitive(Primitive, Span),
    Param(char, Span),
    FuncCallArgs { name: char, args: Vec<Arg>, span: Span },
    Sequence(Vec<Expr>),
}
```

### Error Handling

```rust
// New error
ExpandError::type_conflict(param: char, span: Span) -> Self {
    // E010: Parameter 'X' used as both CmdSeq and Int
}
```
