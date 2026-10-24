# RULES.md - Lean 4 and Folder Management Rules

## Lean 4 Project Structure Rules

### Directory Structure
- **DO** keep Lean files flat in `lean/` directory
- **DO NOT** create nested `lean-verification/` subdirectory
- All `.lean` files go directly in `lean/`

### Module Organization
- `lean/Spec.lean` - Module `Spec` containing all type definitions and interpreter logic
- `lean/Verification.lean` - Module `Verification` containing all proof theorems
- `lean/Psudo.lean` - Main module importing `Spec` and `Verification`
- All modules use flat namespace (no `Psudo.Spec` prefix)

### lakefile.toml Configuration
```toml
name = "psudo_verification"
version = "0.1.0"
defaultTargets = ["Psudo"]

[[lean_lib]]
name = "Psudo"
srcDir = "."
```

### Build Commands
```bash
cd lean
lake build          # Build and verify all proofs
lake clean          # Clean build artifacts
```

## Folder Management Rules

### Prohibited Patterns
- **NEVER** create `lean/lean-verification/` nested directory
- **NEVER** create `test_lake/`, `TestLake/` or similar test directories in lean folder
- **NEVER** leave `TestLake.lean` or other test files in lean folder
- **NEVER** leave `.lake/` build cache committed to git

### Required Cleanup
- After any Lean work, run `lake clean` and remove `.lake/` directory
- Remove any auto-generated test directories (`TestLake/`, `test_lake/`)
- Remove any `TestLake.lean` files
- Keep only: `Spec.lean`, `Verification.lean`, `Psudo.lean`, `lakefile.toml`, `lean-toolchain`, `README.md`

### Git Ignore Rules
Add to `.gitignore`:
```
/lean/.lake/
/lean/lake-manifest.json
/lean/TestLake*/
/lean/test_lake*/
/lean/*.lean.bak
```

## Lean 4 Code Conventions

### Module Declarations
- `Spec.lean`: Start with `module Spec` and end with `end Spec`
- `Verification.lean`: Start with `module Verification` and end with `end Verification`
- `Psudo.lean`: Simple imports only (`import Spec`, `import Verification`)

### Imports
- Use unqualified imports within the same flat namespace
- `import Spec` not `import Psudo.Spec`
- `import Verification` not `import Psudo.Verification`

### Function Definitions
- Use `partial def` for potentially non-terminating recursive functions
- Prefer `def` for total functions
- Use `structure` for records, `inductive` for sum types

### Proofs
- Prove basic lemmas first (literals, binary ops, truthiness)
- Use `rfl` for definitional equality
- Use `simp` with appropriate lemmas
- Mark incomplete proofs with `sorry` temporarily (remove before commit)