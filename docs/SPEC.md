# H2 Language Specification v0.3.1

## 1. Overview

H2 Language is a domain-specific language compatible with HOJ (Herbert Online Judge) H language,
extended with directives and multi-agent support.

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
| `IDENT` | `[a-z]` except `s,r,l` | Macro/function identifiers |
| `PARAM` | `[A-Z]` | Function parameters |
| `NUMBER` | `-?[0-9]+` | Integer literals (-255..255) |
| `AGENT_ID` | `[0-9]+` at line start followed by `:` | Agent identifier |
| Symbols | `: ( ) , + - =` | Punctuation |
| `NEWLINE` | `\n \| \r\n` | Line terminator |
| `SPACE` | `[ \t]+` | Whitespace |
| `COMMENT` | `#.* \| //.*` | Comment to end of line |

### 2.3 Lexical Rules

- **Longest match**: `-1` is read as a single `NUMBER` token.
- `SPACE` and `COMMENT` are ignored except as separators.
- `AGENT_ID` is valid only at line start AND immediately followed by `:`.

---

## 3. Syntax Grammar (EBNF)

### 3.1 Program Structure

```ebnf
program           ::= directives (agent_block | single_agent_block)

directives        ::= (SPACE? directive NEWLINE)*

directive         ::= directive_name '=' directive_value
directive_name    ::= 'MAX_STEP' | 'MAX_DEPTH' | 'MAX_MEMORY' | 'ON_LIMIT'
directive_value   ::= NUMBER | 'ERROR' | 'TRUNCATE'

(* Note: Leading spaces are allowed before directives *)

agent_block       ::= agent (NEWLINE agent)*
agent             ::= AGENT_ID ':' agent_body

single_agent_block ::= agent_body   (* treated as agent 0 *)

agent_body        ::= lines
lines             ::= (line NEWLINE)* line?
line              ::= (definition SPACE?)* expression?
```

> Note: Lines continue until the next `AGENT_ID` or EOF.
> Multiple lines' expressions are concatenated (NEWLINEs ignored).

### 3.2 Definitions

```ebnf
definition        ::= macro_def | func_def

macro_def         ::= IDENT ':' expression

func_def          ::= IDENT '(' param_list ')' ':' expression
param_list        ::= PARAM (',' PARAM)*
```

### 3.3 Expressions

```ebnf
expression        ::= term+

term              ::= COMMAND
                    | IDENT
                    | PARAM
                    | func_call

func_call         ::= IDENT '(' arg_list? ')'
arg_list          ::= argument (',' argument)*

argument          ::= cmd_expr
                    | NUMBER
                    | num_expr

cmd_expr          ::= expression        (* command sequence argument *)
num_expr          ::= PARAM ('+' | '-') NUMBER
```

---

## 4. Semantics

### 4.1 Evaluation Order (per agent)

1. Process `directives`
2. Register all `definitions` (in line order)
3. Concatenate all `expressions` (in line order) into one execution expression
4. Expand execution expression left-to-right, generating `s/r/l` sequence

### 4.2 Macro Expansion

- `IDENT` referencing a macro name is replaced with its body.
- Undefined macro reference: `E001`.

### 4.3 Function Expansion and Types

#### 4.3.1 Parameter Types (Required)

Each `PARAM` has one of two types at call time:

| Type | When | Description |
|------|------|-------------|
| **CmdSeq** | `argument` is `cmd_expr` | Command sequence |
| **Int** | `argument` is `NUMBER` or `num_expr` | Integer value |

**Usage constraints**:
- `PARAM` as `term` in `expression`: **CmdSeq only**
- `PARAM` in `num_expr` (`X-1`): **Int only**
- Type violation: `E008`

#### 4.3.2 Argument Binding and Arity

- Argument count must match `param_list`: otherwise `E003`
- Undefined function: `E002`

#### 4.3.3 Numeric Expression `num_expr`

- Only `PARAM ± NUMBER` form (e.g., `X-1`, `Y+2`)
- Result is Int type
- Range: `-255 ≤ n ≤ 255`, otherwise `E007`

#### 4.3.4 Numeric Termination Condition

**If ANY Int-typed parameter has value `<= 0`, the entire function call
returns empty (generates nothing).**

CmdSeq-typed parameters do not affect this check.

Example:
```
a(X):sa(X-1) a(4)
→ s a(3) → s s a(2) → s s s a(1) → s s s s a(0)
→ s s s s (empty)   # a(0): X=0 <= 0, returns empty
→ ssss
```

---

## 5. Execution Limits

### 5.1 Directives

| Name | Type | Default | Range/Values |
|------|------|---------|--------------|
| `MAX_STEP` | int | 1,000,000 | 1..10,000,000 |
| `MAX_DEPTH` | int | 100 | 1..10,000 |
| `MAX_MEMORY` | int | 1,000,000 | 1..10,000,000 |
| `ON_LIMIT` | enum | *see below* | ERROR / TRUNCATE |

**ON_LIMIT Default Behavior:**
- **No directives specified**: `TRUNCATE` (HOJ compatibility mode)
- **Any directive specified**: `ERROR` (spec-compliant mode)

This allows existing HOJ programs to work without modification (infinite patterns
truncate naturally), while programs using directives get strict error checking.

### 5.2 What is Counted

| Limit | Definition |
|-------|------------|
| `MAX_STEP` | Total generated `s/r/l` commands (all agents combined) |
| `MAX_DEPTH` | Function call nesting depth (per agent) |
| `MAX_MEMORY` | Intermediate representation size during expansion (per agent) |

**MAX_MEMORY definition** (for implementation consistency):
- Count each token in expansion queue/stack/AST as 1 byte
- `COMMAND/IDENT/PARAM/NUMBER/symbol` = 1 byte each

### 5.3 Limit Exceeded Behavior

| ON_LIMIT | Behavior |
|----------|----------|
| `ERROR` | Return corresponding error (E004/E005/E006) and stop |
| `TRUNCATE` | Return commands generated so far and stop |

---

## 6. Multi-Agent Extension

### 6.1 Agent Output

- Each agent independently expands to a command sequence.
- Final result: command sequence per agent.

### 6.2 Parallel Timeline (Execution Model)

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

Multi-line single agent:
```
a:ssrs
aaaaaaaaaa
```

---

## 7. Error Codes

| Code | Description |
|------|-------------|
| E001 | Undefined macro |
| E002 | Undefined function |
| E003 | Argument count mismatch |
| E004 | MAX_STEP exceeded |
| E005 | MAX_DEPTH exceeded |
| E006 | MAX_MEMORY exceeded |
| E007 | Numeric value out of range (±255) |
| E008 | Syntax/type/semantic error |
| E009 | Invalid directive (unknown or invalid value) |

---

## 8. Byte Count (HOJ Compatibility)

### 8.1 Calculation Rules

- Alphabet: 1 character = 1 byte
- Number: 1 number = 1 byte (regardless of digits)

### 8.2 Example

```
a:sa a(4)
# Byte count: a + s + a + a + 4 = 5 bytes (excluding : and parentheses for counting)
```

---

## 9. Conformance Tests

See `tests/spec_conformance.rs` for the 18 conformance tests.

### Test Summary

| ID | Description | Expected |
|----|-------------|----------|
| T01 | Simple commands | `srl` |
| T02 | Macro | `ssssssss` |
| T03 | Undefined macro | E001 |
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
| T18 | Multi-agent | 3 agents, timeline=3 |

---

## 10. References

- [Herbert Online Judge](http://herbert.tealang.info/)
- [HOJ GitHub Repository](https://github.com/quolc/hoj)
- [snuke's blog - Herbert and Mathematics](https://snuke.hatenablog.com/entry/20111206/1323180471)
