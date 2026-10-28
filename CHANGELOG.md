# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Classes and Inheritance support (Phase 2)
  - `class Name` with method definitions
  - `class Derived < Base` inheritance syntax
  - `super.method()` calls
  - `LoxClass`, `LoxInstance`, `LoxFunction` runtime support
  - Method binding with `this` reference
- Resolver with LSP Integration (Phase 1)
  - Static scope analysis with `src/resolver.rs`
  - Variable resolution with depth tracking
  - LSP diagnostics for undefined variables, duplicate declarations, use-before-init
  - Real-time diagnostics on `did_open` and `did_change`
- String + number concatenation: `"str" + num` -> `"strnum"`
- `fn name(params) =>` syntax with indented block body support
- Classes and methods in LSP (completions, hover, semantic tokens)

### Changed
- **Breaking**: Whitespace & Punctuation Diet grammar (indentation-based blocks)
  - Removed `;`, `{`, `}`, `:` syntax
  - Python-style indentation blocks
  - Keywords: `let`, `fn`, `class`, `if`, `else`, `while`, `for`, `return`, `super`, `this`
- **Breaking**: Cranelift codegen deferred due to API incompatibility with cranelift 0.109
  - `CodegenContext::new`, `compile_program`, `emit_object` now `todo!()` 
  - Codegen tests marked `#[ignore]`
  - TargetSpec architecture matching fixed for target_lexicon 0.12
  - Migration needed for cranelift-native, cranelift-object, target-lexicon APIs
- Updated built-in function names to camelCase: `readFile`, `writeFile`, `listDir`, `exists`, `isFile`, `isDir`, `mkdir`, `len`, `push`, `pop`, `slice`, `keys`, `values`, `has`, `str`, `num`, `bool`, `clock`
- String concatenation now supports `"str" + num` and `num + "str"`

### Fixed
- Parser error recovery for multiple statements after class definitions
- Indentation handling for blank lines
- Function declaration parsing with `=>` and indented block body
- Variable resolution in class methods and closures
- Recursive function support via closure capture

## [0.4.0] - 2024-10-28

### Added
- Phase 3: Classes and Inheritance with LSP support
- Phase 2: Resolver and LSP Integration
- Phase 1: Whitespace & Punctuation Diet grammar changes
- Classes: `class Name`, methods with `fn`, `this` reference
- Inheritance: `class Derived < Base`, `super.method()` calls
- Resolver: Static scope analysis, LSP diagnostics
- LSP: OOP completions, hover, semantic tokens
- String + number concatenation
- `fn name(params) =>` with indented block body
- Built-in functions in camelCase

### Changed
- **Breaking**: Indentation-based blocks (Python-style), removed `;`, `{`, `}`, `:`
- **Breaking**: Removed Cranelift codegen (deferred)
- Built-in function names: camelCase (`readFile`, `writeFile`, `listDir`, `len`, etc.)

## [0.3.0] - 2024-10-27

### Added
- Phase 1: Ownership IR and Borrow Checker implementation
- `src/ownership.rs` - BorrowChecker, CheckedEnvironment, MoveError, BorrowError
- `CheckedInterpret` in parser.rs with full ExpressionVisitor implementation
- `-b/--check-borrows` CLI flag for borrow checking mode
- Borrow checker integration tests (use-after-move, double borrow, reassignment)
- Version tracking in ROADMAP.md and version bump rule in .omp/RULES.md
- Git tag v0.2.0
- Custom OMP tools: bump-version, milestone-commit, update-changelog, update-syntax, test-suite, generate-changelog

### Fixed
- Parser compilation errors resolved
- Borrow checker move tracking for reassignments

## [0.1.0] - 2024-10-23

### Added
- Initial lexer implementation with keyword and literal tokenization
- Parser capable of parsing simple expressions
- Basic interpreter with visitor pattern
- CLI application with file and interactive modes
- Syntax error reporting

### Changed
- Refactored lib.rs structure
- Updated README with project status

---

## Milestone Rules

### Auto-commit after milestone
When a major milestone is completed (Phase, Epic, or significant feature):
1. Stage all changes: `git add -A`
2. Create conventional commit: `git commit -m "<type>: <description>"`
3. Update CHANGELOG.md from git history
4. Push to remote: `git push`

### Commit message format
Follow [Conventional Commits](https://www.conventionalcommits.org/):
- `feat:` - New feature
- `fix:` - Bug fix
- `docs:` - Documentation only
- `refactor:` - Code restructuring
- `test:` - Adding tests
- `chore:` - Maintenance

### Changelog generation
```bash
# Generate from git log (run after milestone commit)
git log --pretty=format:"- %s (%h)" --reverse > CHANGELOG_TEMPLATE.md
# Then curate and format into CHANGELOG.md
```

### OMP Custom Tools (in `.omp/tools/`)
- `bump-version` - Bump version in Cargo.toml and create git tag
- `milestone-commit` - Create milestone commit, bump version, push
- `update-changelog` - Update CHANGELOG.md from recent commits
- `update-syntax` - Update note/syntax.md from CLI
- `test-suite` - Run full test suite (fmt, clippy, test)
- `generate-changelog` - Generate full CHANGELOG.md from git history