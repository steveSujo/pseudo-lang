# .omp RULES.md - Project Rules

## Mandatory Todo Usage

**ALWAYS use todos for any multi-step work.** No exceptions.

### When to create todos:
- Any task with 2+ steps
- Any task that modifies multiple files
- Any refactoring, feature addition, or bug fix
- Any work that spans multiple tool calls

### Todo format:
```markdown
todo(op="init", list=[
  {"phase": "Phase name", "items": ["item 1", "item 2"]}
])
```

### Todo lifecycle:
1. `init` at start of work (same turn as first action)
2. `start` when beginning a phase/item
3. `done` when completing a phase/item
4. `block`/`unblock` if stuck

### Enforcement:
- If you catch yourself working without todos for multi-step work: STOP, create todos, resume
- Never yield without todos complete or explicitly blocked
- Commit messages should reference todo completion

## Additional Rules

### Code Quality
- Run `cargo fmt`, `cargo clippy`, `cargo test` before commits
- No `unwrap()`/`expect()` in production paths
- All public APIs must have docs

### Git
- Conventional commits: `feat:`, `fix:`, `docs:`, `refactor:`, `chore:`
- One logical change per commit
### Milestone Commits
- **After completing a major milestone** (Phase, Epic, significant feature):
  1. Stage all changes: `git add -A`
  2. Create conventional commit: `git commit -m "<type>: <description>"`
  3. Update CHANGELOG.md from git history
  4. Push to remote: `git push`
- Milestone commits should summarize the completed work in the message body
- Reference related todos/issues in commit body

### Version Bumping
- **After each milestone commit**, bump version in `Cargo.toml`:
  - Phase completion: minor version (0.x.0 → 0.(x+1).0)
  - Patch fixes: patch version (0.x.y → 0.x.(y+1))
  - Major breaking changes: major version (x.0.0 → (x+1).0.0)
- Update version in `Cargo.toml` and `README.md` if present
- Tag the commit: `git tag v<version>`

### Testing
- All new features need tests
- Run full test suite before push
- Integration tests for CLI features
- LSP tests for language server changes

## Documentation
- Update README.md for user-facing changes
- Update AGENTS.md for architecture changes
- Note/ folder for reference docs (syntax, config, status)

## Syntax Flag Maintenance

**ALWAYS update syntax reference when language syntax changes.**

### Triggers (any of these require syntax update):
- New keyword, operator, or literal form
- New built-in function
- Change to control flow syntax
- Change to variable declaration syntax
- New type or type syntax
- Change to comment syntax
- New statement form

### Required updates:
1. `print_syntax()` function in `src/main.rs` — compressed LLM-optimized output
2. `note/syntax.md` — run `psudo -s > note/syntax.md` and commit
3. `note/psudo.toml.example` if config-related syntax added
4. `README.md` language reference section if user-facing

### Verification:
- Run `psudo -s` and verify output matches current language
- Run `cargo test` to ensure syntax tests pass