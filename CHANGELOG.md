# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Phase 1: Ownership IR and Borrow Checker implementation
- `src/ownership.rs` - BorrowChecker, CheckedEnvironment, MoveError, BorrowError
- `CheckedInterpret` in parser.rs with full ExpressionVisitor implementation
- `-b/--check-borrows` CLI flag for borrow checking mode
- Borrow checker integration tests (use-after-move, double borrow, reassignment)

## [0.2.0] - 2024-10-24

### Added
- Lean 4 formal verification with 24/30 theorems proven (6 marked `sorry`)
- Lean specification in `lean/Spec.lean`, proofs in `lean/Verification.lean`
- Sandboxed filesystem standard library (`read_file`, `write_file`, `list_dir`, etc.)
- Project configuration system (`psudo.toml`) with auto-detection
- LSP server with tower-lsp (initialize, did_change, hover, completion, diagnostics, semantic tokens)
- CLI improvements: `-t/--tokens`, `-a/--ast`, `-q/--quiet`, `-s/--syntax`, `--lsp`, `-V/--version`
- `note/syntax.md` and `note/psudo.toml.example` for syntax reference
- `.omp/RULES.md` and `.omp/ROADMAP.md` for project governance
- 6 minimal LLM-friendly language features

### Fixed
- Lean 4 verification build with core-only tactics
- Sandbox test initialization with `Once`
- Parser implementation with visitor pattern

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