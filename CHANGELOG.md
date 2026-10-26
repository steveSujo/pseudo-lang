# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed
- **Breaking**: Cranelift codegen deferred due to API incompatibility with cranelift 0.109
  - `CodegenContext::new`, `compile_program`, `emit_object` now `todo!()` 
  - Codegen tests marked `#[ignore]`
  - TargetSpec architecture matching fixed for target_lexicon 0.12
  - Migration needed for cranelift-native, cranelift-object, target-lexicon APIs

### Added
- Phase 2: Cranelift x86_64 Linux codegen backend (planned)

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