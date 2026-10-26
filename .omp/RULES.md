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
- No direct pushes to main without PR (when team grows)

### Testing
- All new features need tests
- Run full test suite before push
- Integration tests for CLI features
- LSP tests for language server changes

### Documentation
- Update README.md for user-facing changes
- Update AGENTS.md for architecture changes
- Note/ folder for reference docs (syntax, config, status)