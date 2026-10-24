# AGENTS.md - pseudo-lang Project

## Project Overview
**pseudo-lang** (package: `psudo`) - A Rust implementation of the Lox language from "Crafting Interpreters" with a Lean 4 formal verification companion.

## Repository Structure
```
pseudo-lang/
├── src/                    # Rust interpreter source
│   ├── main.rs            # CLI entry point
│   ├── lib.rs             # Library root, token definitions
│   ├── lexer.rs           # Lexical analysis
│   ├── parser.rs          # Parsing + interpretation (visitor pattern)
│   └── error_handler.rs   # Error types
├── test/                  # Test files
│   ├── valid_test.txt
│   └── comprehensive_test.txt
├── tests/                 # Integration tests
│   └── integration_tests.rs
├── lean/                  # Lean 4 formal verification
│   └── lean-verification/
│       ├── lean-toolchain
│       ├── leanpkg.toml
│       └── Psudo/
│           ├── Spec.lean          # Formal specification
│           └── Verification.lean  # Verification theorems
├── Cargo.toml
├── Cargo.lock
├── README.md
├── LICENSE.md
├── CHANGELOG.md
└── .gitignore
```

## Language Features (Implemented)
- **Variables**: `let x = 10;`, `x = x + 1;`
- **Control flow**: `if/else`, `while`, `for` loops with blocks `{ ... }`
- **Data types**: Numbers (f32), Booleans, Strings
- **Operators**: Arithmetic (+, -, *, /), Comparison (==, !=, <, >, <=, >=), Logical (!)
- **String concatenation**: `"a" + "b"` → `"ab"`
- **Print statements**: `print expression;`

## Architecture
- **Lexer**: Single-pass scanner with error recovery
- **Parser**: Recursive descent with Pratt-style precedence
- **AST**: Expression/Statement enums with visitor pattern
- **Interpreter**: Tree-walking with `HashMap<String, Value>` environment
- **Error handling**: Result-based with error propagation

## Build & Test Commands
```bash
# Build
cargo build

# Run tests
cargo test

# Run interpreter
cargo run -- -i                    # Interactive REPL
cargo run -- -f test/file.txt      # Execute file

# Lean verification (requires Lean 4)
cd lean/lean-verification
lake build
```

## Conventions

### Rust Code Style
- Use `cargo fmt` and `cargo clippy`
- Prefer `Result<T, E>` over `unwrap()`/`expect()`
- Visitor pattern for AST traversal
- Public API in `lib.rs`, binary in `main.rs`

### Lean 4 Formal Verification
- Specification in `Psudo/Spec.lean`
- Theorems in `Psudo/Verification.lean`
- Use `partial def` for potentially non-terminating functions
- Prove safety properties (no division by zero, type preservation, etc.)

### Testing
- Integration tests in `tests/` (run with `cargo test`)
- Test files in `test/` for manual verification
- Each test file should end statements with `;`

## Agent Guidelines

### When Working on Rust Code
1. **Read first**: Check `src/lib.rs` for public API, `src/parser.rs` for interpreter logic
2. **Follow patterns**: Visitor trait in `parser.rs`, error handling in `error_handler.rs`
3. **Test**: Add test cases in `tests/integration_tests.rs` and `test/`
4. **Verify**: Run `cargo build`, `cargo test`, `cargo fmt`, `cargo clippy`

### When Working on Lean 4
1. **Spec first**: Update `Psudo/Spec.lean` before proofs
2. **Use `partial def`**: For recursive functions that may not terminate
3. **Prove incrementally**: Start with simple lemmas (literals, binary ops)
4. **Check**: Run `lake build` in `lean/lean-verification/`

### When Adding Language Features
1. Add token type in `lib.rs`
2. Handle in lexer (`lexer.rs`)
3. Parse in parser (`parser.rs` - add to expression/statement hierarchy)
4. Interpret in visitor impl (`Interpret` in `parser.rs`)
5. Add test case in `tests/integration_tests.rs`
6. Update Lean spec if verification needed

## File Ownership
| File | Purpose | Owner |
|------|---------|-------|
| `src/main.rs` | CLI, REPL | Rust agent |
| `src/lib.rs` | Public API, tokens | Rust agent |
| `src/lexer.rs` | Lexical analysis | Rust agent |
| `src/parser.rs` | Parsing + interpretation | Rust agent |
| `src/error_handler.rs` | Errors | Rust agent |
| `lean/lean-verification/Psudo/Spec.lean` | Formal spec | Lean agent |
| `lean/lean-verification/Psudo/Verification.lean` | Proofs | Lean agent |
| `tests/integration_tests.rs` | Integration tests | Rust agent |

## Do Not
- Modify `Cargo.lock` directly
- Commit `target/` directory
- Use `unwrap()` in production code paths
- Skip tests when adding features

## Lean 4 Verification Status
- [x] Specification complete
- [x] Basic theorems (literals, binary ops, truthiness)
- [ ] Division by zero safety
- [ ] Type preservation
- [ ] Progress/termination for loops
- [ ] Environment consistency
- [ ] Full interpreter correctness

## Dependencies
- Rust: 2021 edition, no external deps (stdlib only)
- Lean: 4.10.0+, std library