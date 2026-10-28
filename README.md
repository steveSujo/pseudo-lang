# pseudo-lang (psudo)

A Rust implementation of the Lox language from "Crafting Interpreters" with a Lean 4 formal verification companion and **Language Server Protocol (LSP) support**.

## Features

- **Variables**: `let x = 10`, `x = x + 1`
- **Control flow**: `if/else`, `while`, `for` loops with indentation-based blocks (Python-style)
- **Data types**: Numbers (f32), Booleans, Strings, Nil
- **Operators**: Arithmetic (+, -, *, /), Comparison (==, !=, <, >, <=, >=), Logical (!)
- **String concatenation**: `"a" + "b"` → `"ab"` (also supports `"str" + num`)
- **Print statements**: `print expression`
- **Classes & Inheritance**: `class Name`, `class Derived < Base`, `super.method()`
- **Functions & Closures**: `fn name(params) => expr` or `fn name(params) { body }`
- **REPL**: Interactive mode with `./target/release/psudo -i`
- **LSP Server**: `./target/release/psudo --lsp` — hover, completion, diagnostics, semantic tokens
- **pIR (Pseudo Intermediate Representation)**: `./target/release/psudo --pir -f scripts/hello.pir` — LLM-optimized hyper-compressed bytecode-style text format with single-character opcodes (`v`, `=`, `p`, `?`, `w`, `f`, `r`, `c`, `m`) and strict prefix notation (`(+ a b)`, `(add 2 3)`)

## Quick Start

```bash
# Build
cargo build --release

# Run standard file
./target/release/psudo -f scripts/hello.psudo

# Run pIR (Pseudo Intermediate Representation) file
./target/release/psudo --pir -f scripts/hello.pir

```bash
# Build
cargo build --release

# Run file (using built artifact)
./target/release/psudo -f scripts/hello.psudo

# REPL (using built artifact)
./target/release/psudo -i

# Language Server Protocol (for editors like VS Code)
./target/release/psudo --lsp

# Show tokens
./target/release/psudo -f scripts/hello.psudo -t

# Show AST
./target/release/psudo -f scripts/hello.psudo -a

# Show syntax reference
./target/release/psudo -s

# Run all test scripts (Windows)
scripts/run_all.bat

# Run all test scripts (Unix)
./scripts/run_all.sh
```

*Note: On Windows, use `target\release\psudo.exe` instead of `./target/release/psudo`*

## Example

```psudo
let x = 10
let y = 5
print x + y        // 15
print x * y        // 50

if x > y
    print "x is greater"
else
    print "y is greater"

for let i = 0
i < 3
i = i + 1
    print i

// Classes and inheritance
class Animal
    fn speak()
        print "Some sound"

class Dog < Animal
    fn speak()
        super.speak()
        print "Woof!"

let d = Dog()
d.speak()

// Functions with arrow syntax
fn add(a, b) => a + b
print add(2, 3)

// Closure
fn makeAdder(x)
    fn add(y) => x + y
    return add
let add5 = makeAdder(5)
print add5(10)  // 15
```

## Project Structure

```
pseudo-lang/
├── src/                    # Rust interpreter
│   ├── main.rs            # CLI entry point
│   ├── lib.rs             # Public API, tokens
│   ├── lexer.rs           # Lexical analysis
│   ├── parser.rs          # Parsing + interpretation
│   ├── error_handler.rs   # Error types
│   └── lsp/               # Language Server Protocol
│       ├── mod.rs         # LSP module exports
│       └── backend.rs     # LSP backend implementation
├── scripts/               # Test scripts
│   ├── hello.psudo
│   ├── arithmetic.psudo
│   ├── control_flow.psudo
│   ├── strings.psudo
│   ├── booleans.psudo
│   ├── variables.psudo
│   ├── blocks.psudo
│   ├── complex.psudo
│   ├── functions.psudo
│   ├── fs_test.psudo
│   ├── objects.psudo
│   ├── arrays.psudo
│   ├── run_all.bat
│   └── run_all.sh
├── tests/                 # Integration tests
│   ├── integration_tests.rs
│   └── lsp_tests.rs       # LSP test suite
├── lean/                  # Lean 4 formal verification
│   ├── Spec.lean          # Formal specification
│   ├── Verification.lean  # Verification theorems
│   ├── PsudoVerification.lean
│   ├── lakefile.toml
│   └── lean-toolchain
├── Cargo.toml
└── LICENSE.md
```

## Language Server Protocol (LSP)

The LSP server provides IDE support for pseudo-lang:

```bash
# Start LSP server (stdin/stdout)
./target/release/psudo --lsp
```

### Supported LSP Features

| Feature | Status |
|---------|--------|
| Initialize/Shutdown | ✅ |
| Text Document Sync (did_open, did_change) | ✅ |
| Hover (variables, keywords, operators) | ✅ |
| Completion (keywords, built-ins, variables) | ✅ |
| Diagnostics (syntax errors, undefined variables) | ✅ |
| Semantic Tokens | ✅ |

### Editor Integration (VS Code)

Create `.vscode/settings.json`:
```json
{
  "pseudo-lang.lsp.enabled": true
}
```
Or use a generic LSP client configuration pointing to `cargo run -- --lsp`.

## Formal Verification (Lean 4)

The `lean/` directory contains a formal specification and verification of the interpreter:

```bash
cd lean
lake build
```

**Proven theorems (24):**
- Literal value preservation
- Binary/unary operations correctness
- Comparison operations
- Truthiness semantics
- Error handling (division by zero, type errors)

**Pending (6, marked `sorry`):** Full evaluation proofs for expressions, statements, and control flow (require Mathlib tactics).

## Development

```bash
# Format
cargo fmt

# Lint
cargo clippy

# Test
cargo test

# Run LSP tests
cargo test lsp

# Lean verification
cd lean && lake build
```

## Language Reference

### Literals
- Numbers: `1`, `3.14`, `-5`
- Strings: `"hello"`, `"world"`
- Booleans: `true`, `false`
- Nil: `nil`

### Operators
| Precedence | Operators | Associativity |
|------------|-----------|---------------|
| Highest    | `-` (unary), `!` | Right |
|            | `*`, `/` | Left |
|            | `+`, `-` | Left |
|            | `<`, `>`, `<=`, `>=` | Left |
|            | `==`, `!=` | Left |
| Lowest     | `=` (assignment) | Right |

### Statements
- `print expr`
- `let name = expr`
- `name = expr`
- `if cond\n    ...` (indented block)
- `else\n    ...` (indented block)
- `while cond\n    ...` (indented block)
- `for init\ncond\ninc\n    ...` (indented block)
- Statements terminated by newlines (semicolons optional for backward compatibility)

### Functions
- `fn name(params) => expr` — expression body
- `fn name(params)\n    stmt\n    ...` — block body
- `fn name(params) =>\n    stmt\n    ...` — indented block body after `=>`
- `return expr` — return from function

### Classes
- `class Name\n    fn method()...` — class definition
- `class Derived < Base\n    fn method()...` — inheritance
- `super.method()` — call superclass method
- `this` — reference to current instance

### Built-in Functions
| Function | Description |
|----------|-------------|
| `clock()` | Current time in seconds |
| `len(x)` | Length of string/array/object |
| `str(x)`, `num(x)`, `bool(x)` | Type conversion |
| `readFile(path)` | Read file (sandboxed) |
| `writeFile(path, content)` | Write file (sandboxed) |
| `listDir(path)` | List directory |
| `exists(path)` | Check if path exists |
| `isFile(path)`, `isDir(path)` | Path type checks |
| `mkdir(path)` | Create directory |
| `push(arr, x)`, `pop(arr)`, `slice(arr, start, end?)` | Array ops |
| `keys(obj)`, `values(obj)`, `has(obj, key)` | Object ops |

### Comments
- `// line comment`
- `/* block comment */` (supports nesting)

### Statements
Terminated by newlines. Semicolons optional for backward compatibility.
## Status

| Component | Status |
|-----------|--------|
| Lexer | ✅ Complete |
| Parser | ✅ Complete |
| Interpreter | ✅ Complete |
| REPL | ✅ Complete |
| Error handling | ✅ Complete |
| LSP Server | ✅ Complete |
| Lean spec | ✅ Complete |
| Lean proofs | ⚠️ Partial (24/30) |
| MIR & Borrow Checker | ✅ Complete |
| Cranelift Codegen | 🚧 Deferred (cranelift 0.109 API migration needed) |
## License

MIT License - see [LICENSE.md](LICENSE.md)

## Project Structure

```
pseudo-lang/
├── src/                    # Rust interpreter
│   ├── main.rs            # CLI entry point
│   ├── lib.rs             # Public API, tokens
│   ├── lexer.rs           # Lexical analysis
│   ├── parser.rs          # Parsing + interpretation
│   ├── error_handler.rs   # Error types
│   └── lsp/               # Language Server Protocol
│       ├── mod.rs         # LSP module exports
│       └── backend.rs     # LSP backend implementation
├── scripts/               # Test scripts
│   ├── hello.psudo
│   ├── arithmetic.psudo
│   ├── control_flow.psudo
│   ├── strings.psudo
│   ├── booleans.psudo
│   ├── variables.psudo
│   ├── blocks.psudo
│   ├── complex.psudo
│   ├── run_all.bat
│   └── run_all.sh
├── tests/                 # Integration tests
│   ├── integration_tests.rs
│   └── lsp_tests.rs       # LSP test suite
├── lean/                  # Lean 4 formal verification
│   ├── Spec.lean          # Formal specification
│   ├── Verification.lean  # Verification theorems
│   ├── PsudoVerification.lean
│   ├── lakefile.toml
│   └── lean-toolchain
├── Cargo.toml
└── LICENSE.md
```

## Language Server Protocol (LSP)

The LSP server provides IDE support for pseudo-lang:

```bash
# Start LSP server (stdin/stdout)
./target/release/psudo --lsp
```

### Supported LSP Features

| Feature | Status |
|---------|--------|
| Initialize/Shutdown | ✅ |
| Text Document Sync (did_open, did_change) | ✅ |
| Hover (variables, keywords, operators) | ✅ |
| Completion (keywords, built-ins, variables) | ✅ |
| Diagnostics (syntax errors, undefined variables) | ✅ |
| Semantic Tokens | ✅ |

### Editor Integration (VS Code)

Create `.vscode/settings.json`:
```json
{
  "pseudo-lang.lsp.enabled": true
}
```
Or use a generic LSP client configuration pointing to `cargo run -- --lsp`.

## Formal Verification (Lean 4)

The `lean/` directory contains a formal specification and verification of the interpreter:

```bash
cd lean
lake build
```

**Proven theorems (24):**
- Literal value preservation
- Binary/unary operations correctness
- Comparison operations
- Truthiness semantics
- Error handling (division by zero, type errors)

**Pending (6, marked `sorry`):** Full evaluation proofs for expressions, statements, and control flow (require Mathlib tactics).

## Development

```bash
# Format
cargo fmt

# Lint
cargo clippy

# Test
cargo test

# Run LSP tests
cargo test lsp

# Lean verification
cd lean && lake build
```

## Language Reference

### Literals
- Numbers: `1`, `3.14`, `-5`
- Strings: `"hello"`, `"world"`
- Booleans: `true`, `false`
- Nil: `nil`

### Operators
| Precedence | Operators | Associativity |
|------------|-----------|---------------|
| Highest    | `-` (unary), `!` | Right |
|            | `*`, `/` | Left |
|            | `+`, `-` | Left |
|            | `<`, `>`, `<=`, `>=` | Left |
|            | `==`, `!=` | Left |
| Lowest     | `=` (assignment) | Right |

### Statements
- `print expr`
- `let name = expr`
- `name = expr`
- `if cond\n    ...` (indented block)
- `else\n    ...` (indented block)
- `while cond\n    ...` (indented block)
- `for init\ncond\ninc\n    ...` (indented block)
- Statements terminated by newlines (semicolons optional for backward compatibility)
## Status

| Component | Status |
|-----------|--------|
| Lexer | ✅ Complete |
| Parser | ✅ Complete |
| Interpreter | ✅ Complete |
| REPL | ✅ Complete |
| Error handling | ✅ Complete |
| LSP Server | ✅ Complete |
| Lean spec | ✅ Complete |
| Lean proofs | ⚠️ Partial (24/30) |
| MIR & Borrow Checker | ✅ Complete |
| Cranelift Codegen | 🚧 Deferred (cranelift 0.109 API migration needed) |
## License

MIT License - see [LICENSE.md](LICENSE.md)