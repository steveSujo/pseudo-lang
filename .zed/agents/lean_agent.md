# Lean Agent - Formal Verification Loop

## Purpose
Automated agent for iterating through Lean 4 formal verification: building, fixing errors, proving theorems, and maintaining the specification.

## Environment
- **Working directory**: `lean/`
- **Toolchain**: Lean 4.35.0 via `elan`
- **Build tool**: `lake` (Lake 5.0.0)

## Loop Phases

### Phase 1: Build & Diagnose
```bash
cd lean
lake build 2>&1 | head -100
```
- Capture all errors
- Categorize: syntax, type mismatch, missing imports, proof failures
- Prioritize: syntax → type errors → proof obligations

### Phase 2: Fix Specification (Spec.lean)
```bash
# Edit lean/Spec.lean
# Common fixes:
# - Add missing `module Spec` / `end Spec`
# - Fix inductive type declarations
# - Fix `partial def` syntax for recursive functions
# - Ensure all types used in Verification.lean are defined
```

### Phase 3: Fix Proofs (Verification.lean)
```bash
# Edit lean/Verification.lean
# Common fixes:
# - Add missing `module Verification` / `end Verification`
# - Fix `open Spec` statement
# - Replace `sorry` with actual proofs
# - Use `rfl` for definitional equality
# - Use `simp [lemma_name]` for simplification
# - Use `aesop` for auto-solving
```

### Phase 4: Verify Main Module (Psudo.lean)
```bash
# Ensure lean/Psudo.lean contains only:
import Spec
import Verification
```

### Phase 5: Iterate Until Clean
```bash
lake build
# Repeat phases 2-4 until build succeeds
```

## Proof Strategy Priority

### 1. Basic Lemmas (RFL)
- Literal conversion preservation
- Binary operation correctness
- Unary operation correctness
- Truthiness properties

```lean
theorem literal_num_preserves (n : Float) :
    literalToValue (LiteralValue.Num n) = Value.Num n := by rfl
```

### 2. Structural Lemmas (SIMP + AESOP)
- Expression evaluation
- Statement execution
- Environment updates

```lean
theorem eval_literal_num (n : Float) (tok : Token) :
    tok.tokenType = TokenType.Number →
    tok.literal = some (LiteralValue.Num n) →
    evalExpr emptyState (Expr.Literal tok) = EvalResult.Ok (Value.Num n) := by
  intro h₁ h₂
  simp [evalExpr, tokenToValue, literalToValue, h₁, h₂]
  <;> aesop
```

### 3. Complex Properties (MANUAL)
- Division by zero safety
- Type preservation
- Progress/termination
- Environment consistency

## Error Resolution Patterns

### "unknown module prefix"
- Check `module Spec` / `end Spec` in Spec.lean
- Check `module Verification` / `end Verification` in Verification.lean
- Ensure Psudo.lean uses `import Spec` not `import Psudo.Spec`

### "failed to synthesize instance"
- Add missing typeclass instances
- Check imports of `Float`, `String`, `Option`, `List`

### "unsolved goals" / proof failures
- Start with `have` statements to break down
- Use `norm_num` for numeric computations
- Use `cases` for inductive types
- Use `induction` for recursive structures

### "partial def" errors
- Ensure recursive calls are structurally decreasing
- Add `termination_by` clause if needed
- Consider using `WellFounded` relations

## Automation Script

```bash
#!/bin/bash
# lean_verify_loop.sh
set -e

cd lean

for i in {1..50}; do
    echo "=== Iteration $i ==="
    if lake build 2>&1 | tee build.log; then
        echo "✓ Build successful!"
        exit 0
    fi
    
    # Check error type
    if grep -q "unknown module prefix" build.log; then
        echo "Fixing module declarations..."
        # Auto-fix module headers
    elif grep -q "unsolved goals" build.log; then
        echo "Proof obligations remaining..."
        # List sorry count
        grep -c "sorry" Verification.lean
    elif grep -q "type mismatch" build.log; then
        echo "Type errors detected..."
    fi
    
    sleep 2
done

echo "Max iterations reached"
exit 1
```

## Proof Completion Checklist

- [ ] Spec.lean compiles (module Spec ... end Spec)
- [ ] Verification.lean compiles (module Verification ... end Verification)
- [ ] Psudo.lean imports both successfully
- [ ] `lake build` succeeds with zero errors
- [ ] Zero `sorry` in Verification.lean
- [ ] All basic lemmas proven (literals, ops, truthiness)
- [ ] Key interpreter properties proven (eval, exec, env)
- [ ] Safety properties (div by zero, type preservation)

## File Watcher (Optional)
```bash
# Watch for changes and auto-rebuild
watchexec -w lean -e lean "cd lean && lake build"
```

## Integration with CI
```yaml
# .github/workflows/lean.yml
name: Lean Verification
on: [push, pull_request]
jobs:
  verify:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Setup Lean
        uses: leanprover/lean-action@v1
      - name: Build
        run: cd lean && lake build
```

## Notes
- Always run `lake clean` before major changes
- Keep `.lake/` out of git (add to .gitignore)
- Use `lake update` to update dependencies
- Target Lean 4.10.0+ but test with current 4.35.0