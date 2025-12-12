# H2 Language Specification v0.4.0

## 1. Overview

H2 Language is a domain-specific language **fully compatible** with HOJ (Herbert Online Judge) H language,
extended with directives and multi-agent support.

**Compatibility Goal**: Any program accepted by HOJ Ruby implementation should produce
identical output in H2 (within execution limits).

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
| `AGENT_ID` | `[0-9]+` at line start followed by `:` | Agent identifier (H2 extension) |
| Symbols | `: ( ) , + -` | Punctuation |
| `NEWLINE` | `\n \| \r\n` | Line terminator |

**HOJ Compatibility Notes:**
- HOJ does **not** support spaces, tabs, or comments in source code.
- HOJ does **not** support negative number literals (`-1`); use `0-1` instead.
- H2 accepts `SPACE`, `COMMENT`, and `-NUMBER` as **extensions** (not HOJ-compatible).

### 2.3 Lexical Rules

- **Longest match**: `12` is read as a single `NUMBER(12)` token.
- `AGENT_ID` is valid only at line start AND immediately followed by `:`.
- In HOJ-compatible mode: unknown characters cause immediate rejection.

---

## 3. Syntax Grammar (EBNF)

### 3.1 Program Structure

```ebnf
(* H2 Extended Mode *)
program           ::= directives agent_content

directives        ::= (SPACE? directive NEWLINE)*

directive         ::= directive_name '=' directive_value
directive_name    ::= 'MAX_STEP' | 'MAX_DEPTH' | 'MAX_MEMORY' | 'ON_LIMIT'
directive_value   ::= NUMBER | 'ERROR' | 'TRUNCATE'

agent_content     ::= agent_block | single_agent_block

agent_block       ::= agent (NEWLINE agent)*
agent             ::= AGENT_ID ':' agent_body

single_agent_block ::= agent_body   (* treated as agent 0 *)
```

### 3.2 Agent Body (HOJ-Compatible Structure)

```ebnf
(* HOJ Structure: last line = main expression, previous lines = definitions *)
agent_body        ::= (definition NEWLINE)* main_expression

definition        ::= func_def

func_def          ::= IDENT '(' param_list? ')' ':' expression
                    | IDENT ':' expression    (* 0-arg function, same as macro *)

param_list        ::= PARAM (',' PARAM)*

main_expression   ::= expression
```

**Critical HOJ Compatibility:**
- Each line before the last is **exactly one function definition**.
- The **last line** is the **main execution expression**.
- Empty lines are ignored.
- A 0-argument function (`x:ss`) is semantically identical to what H2 v0.3 called "macro".

### 3.3 Expressions

```ebnf
expression        ::= term+

term              ::= COMMAND
                    | IDENT              (* 0-arg function call *)
                    | PARAM              (* parameter reference, CmdSeq type only *)
                    | func_call

func_call         ::= IDENT '(' arg_list? ')'
arg_list          ::= argument (',' argument)*

argument          ::= num_expr           (* if starts with NUMBER or evaluates to Int *)
                    | cmd_expr           (* otherwise, CmdSeq *)

cmd_expr          ::= term+              (* command sequence argument *)
```

### 3.4 Numeric Expressions (HOJ-Compatible)

```ebnf
(* HOJ supports full arithmetic: 12-3+4, X-1+2, etc. *)
num_expr          ::= num_atom (('+' | '-') num_atom)*

num_atom          ::= NUMBER
                    | PARAM              (* must be Int-typed parameter *)
```

**HOJ Behavior:**
- If an argument **starts with a digit**, it is parsed as `num_expr`.
- After parameter substitution, if the result starts with a digit, it becomes numeric.
- Example: `X` bound to `5` in `X-1` → `5-1` → `4`

---

## 4. Semantics

### 4.1 Evaluation Order (per agent)

1. Process `directives` (H2 extension)
2. Parse all lines: definitions (all but last) + main expression (last line)
3. Register all function definitions
4. Expand main expression left-to-right, generating `s/r/l` sequence

### 4.2 Function Definitions (Unified Model)

**HOJ treats all definitions uniformly:**
- `x:ss` is a 0-argument function (H2 v0.3 called this "macro")
- `f(X):XXX` is a 1-argument function
- `g(A,B):AB` is a 2-argument function

**Same-name collision is forbidden** (only one definition per identifier).

### 4.3 Type System (Fixed Per Definition)

#### 4.3.1 Parameter Types Are Fixed

**Unlike H2 v0.3, HOJ determines parameter types at definition time, not call time.**

Each parameter in a function definition has exactly one type:

| Type | Description |
|------|-------------|
| **CmdSeq** | Command sequence (s/r/l combinations) |
| **Int** | Integer value (-255..255) |

#### 4.3.2 Type Inference Rules

Type is inferred by analyzing how each parameter is used in the function body:

| Usage | Inferred Type |
|-------|---------------|
| `PARAM` as term in expression (e.g., `X` alone) | CmdSeq |
| `PARAM` in `num_expr` (e.g., `X-1`, `X+2`) | Int |

**Conflict = Error**: If a parameter is used both ways, it's a type error.

#### 4.3.3 Call-Site Type Checking

At call site, argument type must match parameter type:

| Parameter Type | Valid Argument |
|----------------|----------------|
| CmdSeq | `cmd_expr` (command sequence) |
| Int | `num_expr` (numeric expression) |

Type mismatch: `E008`

### 4.4 Numeric Expression Evaluation

```
num_expr ::= num_atom (('+' | '-') num_atom)*
```

Evaluation:
1. Substitute all `PARAM` with their bound Int values
2. Evaluate left-to-right: `12-3+4` → `9+4` → `13`
3. Range check: `-255 ≤ result ≤ 255`, otherwise `E007`

### 4.5 Numeric Termination Condition

**If ANY Int-typed parameter has value `<= 0`, the function call returns empty.**

This is checked **after** argument evaluation:

```
a(X):sa(X-1) a(4)
→ s a(3) → ss a(2) → sss a(1) → ssss a(0)
→ ssss (empty)   # X=0 <= 0, returns empty
```

CmdSeq-typed parameters do not affect this check.

### 4.6 Function Expansion

1. Match arguments to parameters by position
2. Check arity: mismatch → `E003`
3. Type-check each argument against parameter type
4. Check numeric termination condition
5. Substitute parameters in body and expand recursively

---

## 5. Execution Limits

### 5.1 Directives (H2 Extension)

| Name | Type | Default | Range/Values |
|------|------|---------|--------------|
| `MAX_STEP` | int | 1,000,000 | 1..10,000,000 |
| `MAX_DEPTH` | int | 100 | 1..10,000 |
| `MAX_MEMORY` | int | 1,000,000 | 1..10,000,000 |
| `ON_LIMIT` | enum | *see below* | ERROR / TRUNCATE |

**ON_LIMIT Default Behavior:**
- **No directives specified**: `TRUNCATE` (HOJ compatibility mode)
- **Any directive specified**: `ERROR` (strict mode)

### 5.2 Step Counting (HOJ vs H2)

**HOJ counts differently than H2 v0.3:**

| Implementation | What is counted |
|----------------|-----------------|
| HOJ Ruby | Every token dequeued during expansion (including function names) |
| H2 (simplified) | Only generated `s/r/l` commands |

For strict HOJ compatibility, H2 should count expansion operations.
Current H2 uses simplified counting (commands only).

### 5.3 Limit Exceeded Behavior

| ON_LIMIT | Behavior |
|----------|----------|
| `ERROR` | Return corresponding error (E004/E005/E006) and stop |
| `TRUNCATE` | Return commands generated so far and stop |

---

## 6. Multi-Agent Extension (H2 Only)

### 6.1 Agent Output

- Each agent independently expands to a command sequence.
- Final result: command sequence per agent.

### 6.2 Parallel Timeline

- Timesteps: `t = 0, 1, 2, ...`
- Agent `i` executes `cmd[i][t]` if available
- If sequence exhausted: **no-op**
- Timeline length: `max_i len(cmd[i])`

### 6.3 Single Agent Mode

`0:` prefix can be omitted for single-agent programs:

```
# These are equivalent:
0: f(X):XXXX f(sssr)
f(X):XXXX f(sssr)
```

---

## 7. Error Codes

| Code | Description |
|------|-------------|
| E001 | Undefined function (was: undefined macro) |
| E002 | Undefined function |
| E003 | Argument count mismatch |
| E004 | MAX_STEP exceeded |
| E005 | MAX_DEPTH exceeded |
| E006 | MAX_MEMORY exceeded |
| E007 | Numeric value out of range (±255) |
| E008 | Type error (CmdSeq/Int mismatch) |
| E009 | Invalid directive (H2 extension) |

---

## 8. HOJ Compatibility Summary

### 8.1 Fully Compatible Features

| Feature | HOJ | H2 |
|---------|-----|-----|
| Commands `s`, `r`, `l` | ✓ | ✓ |
| Function definitions | ✓ | ✓ |
| 0-arg functions (macros) | ✓ | ✓ |
| Numeric recursion | ✓ | ✓ |
| ≤0 termination | ✓ | ✓ |
| Multi-digit numbers | ✓ | ✓ |
| Arithmetic `+`, `-` | ✓ | ✓ |
| Complex num_expr `12-3+4` | ✓ | ✓ (v0.4+) |
| Fixed parameter types | ✓ | ✓ (v0.4+) |

### 8.2 H2 Extensions (Not in HOJ)

| Feature | Description |
|---------|-------------|
| `AGENT_ID:` | Multi-agent programs |
| Directives | `MAX_STEP`, `ON_LIMIT`, etc. |
| Spaces | Whitespace in source |
| Comments | `#` and `//` |
| Negative literals | `-1` as single token |

### 8.3 Program Structure Difference

**HOJ:**
```
line1: definition
line2: definition
line3: main expression (LAST LINE)
```

**H2 (compatible mode):**
```
line1: definition
line2: definition
line3: main expression (LAST LINE)
```

**H2 (extended mode - single line):**
```
definition definition expression
```

---

## 9. Conformance Tests

See `tests/spec_conformance.rs` and `tests/hoj_conformance.rs`.

### 9.1 Basic Tests (T01-T18)

| ID | Description | Expected |
|----|-------------|----------|
| T01 | Simple commands | `srl` |
| T02 | 0-arg function (macro) | `ssssssss` |
| T03 | Undefined function | E001/E002 |
| T04 | Function (CmdSeq arg) | `srsrsr` |
| T05 | Undefined function | E002 |
| T06 | Argument count mismatch | E003 |
| T07 | Numeric counter recursion | `ssss` |
| T08 | Numeric counter 0 | (empty) |
| T09 | Numeric counter negative | (empty) |
| T10 | Multiple args (Int + CmdSeq) | `srsrsr` |
| T11 | Numeric argument | `ss` |
| T12 | Numeric range exceeded | E007 |
| T13 | Type error (Int as term) | E008 |
| T14 | Type error (CmdSeq in num_expr) | E008 |
| T15 | Unknown directive | E009 |
| T16 | MAX_STEP exceeded (ERROR) | E004 |
| T17 | MAX_STEP exceeded (TRUNCATE) | `sss` |
| T18 | Multi-agent | 3 agents |

### 9.2 HOJ-Specific Tests (T19-T24)

| ID | Description | Input | Expected |
|----|-------------|-------|----------|
| T19 | Complex num_expr | `a(X):sa(X-1) a(10-3+1)` | `ssssssss` (8) |
| T20 | Multi-term num_expr | `a(X):sa(X-1) a(5+5-2)` | `ssssssss` (8) |
| T21 | Param in num_expr chain | `a(X,Y):sa(X-1,Y-1) a(3,3)` | see test |
| T22 | Last-line-main structure | multiline | see test |
| T23 | Type inference (CmdSeq) | `f(X):XX f(sr)` | `srsr` |
| T24 | Type inference (Int) | `f(X):sf(X-1) f(3)` | `sss` |

---

## 10. References

- [Herbert Online Judge](http://herbert.tealang.info/)
- [HOJ GitHub Repository](https://github.com/quolc/hoj)
- [snuke's blog - Herbert and Mathematics](https://snuke.hatenablog.com/entry/20111206/1323180471)

---

## Appendix A: Migration from v0.3.1

### Breaking Changes

1. **`macro_def` removed**: Use 0-arg `func_def` instead (`x:ss`)
2. **`num_expr` expanded**: Now supports `num_atom (('+' | '-') num_atom)*`
3. **Parameter types fixed**: Types inferred at definition, checked at call
4. **E001 unified**: Now refers to "undefined function" (was "undefined macro")

### Code Changes Required

```rust
// Before (v0.3.1)
enum Definition {
    Macro(MacroDef),
    Function(FuncDef),
}

// After (v0.4.0)
enum Definition {
    Function(FuncDef),  // 0-arg function replaces Macro
}
```

### AST Changes

```rust
// Before
Arg::NumExpr { param, offset, span }  // only PARAM±NUMBER

// After
Arg::NumExpr { atoms: Vec<NumAtom>, ops: Vec<Op>, span }  // full arithmetic
```
