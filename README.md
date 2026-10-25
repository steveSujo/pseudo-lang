# pseudo-lang (psudo)

A Rust implementation of the Lox language from "Crafting Interpreters" with a Lean 4 formal verification companion and **Language Server Protocol (LSP) support**.

## Features

- **Variables**: `let x = 10;`, `x = x + 1;`
- **Control flow**: `if/else`, `while`, `for` loops with blocks `{ ... }`
- **Data types**: Numbers (f32), Booleans, Strings, Nil
- **Operators**: Arithmetic (+, -, *, /), Comparison (==, !=, <, >, <=, >=), Logical (!)
- **String concatenation**: `"a" + "b"` → `"ab"`
- **Print statements**: `print expression;`
- **REPL**: Interactive mode with `cargo run -- -i`
- **LSP Server**: `cargo run -- --lsp` — hover, completion, diagnostics, semantic tokens

## Quick Start

```bash
# Build
cargo build

# Run file
cargo run -- -f scripts/hello.psudo

# REPL
cargo run -- -i

# Language Server Protocol (for editors like VS Code)
cargo run -- --lsp

# Run all test scripts (Windows)
scripts/run_all.bat

# Run all test scripts (Unix)
./scripts/run_all.sh
```

## Example

```lox
let x = 10;
let y = 5;
print x + y;        // 15
print x * y;        // 50

if (x > y) {
    print "x is greater";
} else {
    print "y is greater";
}

for (let i = 0; i < 3; i = i + 1) {
    print i;
}
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
cargo run -- --lsp
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
- `print expr;`
- `let name = expr;`
- `name = expr;`
- `if (cond) { ... } else { ... }`
- `while (cond) { ... }`
- `for (init; cond; inc) { ... }`
- `{ stmt; stmt; }` (block)

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

## License

MIT License - see [LICENSE.md](LICENSE.md)