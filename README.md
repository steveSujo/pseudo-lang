# pseudo-lang (psudo)

A Rust implementation of the Lox language from "Crafting Interpreters" with a Lean 4 formal verification companion.

## Features

- **Variables**: `let x = 10;`, `x = x + 1;`
- **Control flow**: `if/else`, `while`, `for` loops with blocks `{ ... }`
- **Data types**: Numbers (f32), Booleans, Strings, Nil
- **Operators**: Arithmetic (+, -, *, /), Comparison (==, !=, <, >, <=, >=), Logical (!)
- **String concatenation**: `"a" + "b"` → `"ab"`
- **Print statements**: `print expression;`
- **REPL**: Interactive mode with `cargo run -- -i`

## Quick Start

```bash
# Build
cargo build

# Run file
cargo run -- -f scripts/hello.psudo

# REPL
cargo run -- -i

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
│   └── error_handler.rs   # Error types
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
├── lean/                  # Lean 4 formal verification
│   ├── Spec.lean          # Formal specification
│   ├── Verification.lean  # Verification theorems
│   ├── PsudoVerification.lean
│   ├── lakefile.toml
│   └── lean-toolchain
├── Cargo.toml
└── LICENSE.md
```

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
| Lean spec | ✅ Complete |
| Lean proofs | ⚠️ Partial (24/30) |

## License

MIT License - see [LICENSE.md](LICENSE.md)