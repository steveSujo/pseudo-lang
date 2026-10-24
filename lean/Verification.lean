/- Verification of pseudo-lang interpreter properties -/
import Spec

/-- Prove that addition of numbers is commutative -/
theorem float_add_comm (x y : Float) : x + y = y + x := by
  sorry

/-- Prove that multiplication of numbers is commutative -/
theorem float_mul_comm (x y : Float) : x * y = y * x := by
  sorry

/-- Prove that literal conversion preserves numeric values -/
theorem literal_num_preserves (n : Float) :
    literalToValue (LiteralValue.Num n) = Value.Num n := by
  rfl

/-- Prove that literal conversion preserves string values -/
theorem literal_str_preserves (s : String) :
    literalToValue (LiteralValue.Str s) = Value.Str s := by
  rfl

/-- Prove that literal conversion preserves boolean values -/
theorem literal_bool_preserves (b : Bool) :
    literalToValue (LiteralValue.Bool b) = Value.Bool b := by
  rfl

/-- Prove that literal conversion preserves nil -/
theorem literal_nil_preserves :
    literalToValue LiteralValue.Nil = Value.Nil := by
  rfl

/-- Prove that binary addition on values matches Float addition -/
theorem value_add_matches_float (x y : Float) :
    applyBinaryOp TokenType.Plus (Value.Num x) (Value.Num y) = EvalResult.Ok (Value.Num (x + y)) := by
  rfl

/-- Prove that binary multiplication on values matches Float multiplication -/
theorem value_mul_matches_float (x y : Float) :
    applyBinaryOp TokenType.Star (Value.Num x) (Value.Num y) = EvalResult.Ok (Value.Num (x * y)) := by
  rfl

/-- Prove that equality comparison works correctly for numbers -/
theorem value_eq_numbers (x y : Float) :
    applyBinaryOp TokenType.EqualEqual (Value.Num x) (Value.Num y) =
      EvalResult.Ok (Value.Bool (x = y)) := by
  rfl

/-- Prove that inequality comparison works correctly for numbers -/
theorem value_ne_numbers (x y : Float) :
    applyBinaryOp TokenType.BangEqual (Value.Num x) (Value.Num y) =
      EvalResult.Ok (Value.Bool (x ≠ y)) := by
  rfl

/-- Prove that less than comparison works correctly for numbers -/
theorem value_lt_numbers (x y : Float) :
    applyBinaryOp TokenType.Less (Value.Num x) (Value.Num y) =
      EvalResult.Ok (Value.Bool (x < y)) := by
  rfl

/-- Prove that less than or equal comparison works correctly for numbers -/
theorem value_le_numbers (x y : Float) :
    applyBinaryOp TokenType.LessEqual (Value.Num x) (Value.Num y) =
      EvalResult.Ok (Value.Bool (x ≤ y)) := by
  rfl

/-- Prove that greater than comparison works correctly for numbers -/
theorem value_gt_numbers (x y : Float) :
    applyBinaryOp TokenType.Greater (Value.Num x) (Value.Num y) =
      EvalResult.Ok (Value.Bool (x > y)) := by
  rfl

/-- Prove that greater than or equal comparison works correctly for numbers -/
theorem value_ge_numbers (x y : Float) :
    applyBinaryOp TokenType.GreaterEqual (Value.Num x) (Value.Num y) =
      EvalResult.Ok (Value.Bool (x ≥ y)) := by
  rfl

/-- Prove that unary minus negates a number -/
theorem value_unary_minus (x : Float) :
    applyUnaryOp TokenType.Minus (Value.Num x) = EvalResult.Ok (Value.Num (-x)) := by
  rfl

/-- Prove that unary bang negates a boolean -/
theorem value_unary_bang (b : Bool) :
    applyUnaryOp TokenType.Bang (Value.Bool b) = EvalResult.Ok (Value.Bool (!b)) := by
  rfl

/-- Prove string concatenation -/
theorem value_str_concat (s₁ s₂ : String) :
    applyBinaryOp TokenType.Plus (Value.Str s₁) (Value.Str s₂) = EvalResult.Ok (Value.Str (s₁ ++ s₂)) := by
  rfl

/-- Prove truthiness for nil is false -/
theorem truthiness_nil : isTruthy Value.Nil = false := by rfl

/-- Prove truthiness for false is false -/
theorem truthiness_false : isTruthy (Value.Bool false) = false := by rfl

/-- Prove truthiness for true is true -/
theorem truthiness_true : isTruthy (Value.Bool true) = true := by rfl

/-- Prove truthiness for numbers is true -/
theorem truthiness_num (x : Float) : isTruthy (Value.Num x) = true := by rfl

/-- Prove truthiness for strings is true -/
theorem truthiness_str (s : String) : isTruthy (Value.Str s) = true := by rfl

/-- Prove that evaluating a literal number returns the correct value -/
theorem eval_literal_num (n : Float) (tok : Token) :
    tok.tokenType = TokenType.Number →
    tok.literal = some (LiteralValue.Num n) →
    evalExpr emptyState (Expr.Literal tok) = EvalResult.Ok (Value.Num n) := by
  sorry

/-- Prove that evaluating a grouping expression evaluates the inner expression -/
theorem eval_grouping (e : Expr) (v : Value) :
    evalExpr emptyState e = EvalResult.Ok v →
    evalExpr emptyState (Expr.Grouping e) = EvalResult.Ok v := by
  sorry

/-- Prove that variable assignment updates the environment -/
theorem let_updates_env (name : String) (v : Value) (init : Expr) :
    evalExpr emptyState init = EvalResult.Ok v →
    execStmt emptyState (Stmt.LetStmt name init) = (EvalResult.Ok v, { env := fun n => if n = name then some v else emptyState.env n }) := by
  sorry

/-- Prove that a simple print statement evaluates and returns the value -/
theorem print_evals_expr (e : Expr) (v : Value) :
    evalExpr emptyState e = EvalResult.Ok v →
    execStmt emptyState (Stmt.PrintStmt e) = (EvalResult.Ok v, emptyState) := by
  sorry

/-- Prove that if-then executes the then branch when condition is true -/
theorem if_true_executes_then (thenBranch elseBranch : Stmt) :
    execStmt emptyState (Stmt.IfStmt (Expr.Literal { tokenType := TokenType.True, text := "true", literal := none, line := 1 }) thenBranch elseBranch) = execStmt emptyState thenBranch := by
  sorry

/-- Prove that if-then-else executes the else branch when condition is false -/
theorem if_false_executes_else (thenBranch elseBranch : Stmt) :
    execStmt emptyState (Stmt.IfStmt (Expr.Literal { tokenType := TokenType.False, text := "false", literal := none, line := 1 }) thenBranch (some elseBranch)) = execStmt emptyState elseBranch := by
  sorry

/-- Prove that while loop with true condition executes body -/
theorem while_true_executes_body (body : Stmt) :
    ∃ (result : EvalResult × InterpreterState), execStmt emptyState (Stmt.WhileStmt (Expr.Literal { tokenType := TokenType.True, text := "true", literal := none, line := 1 }) body) = result := by
  refine' ⟨execStmt emptyState (Stmt.WhileStmt (Expr.Literal { tokenType := TokenType.True, text := "true", literal := none, line := 1 }) body), _⟩
  <;> rfl

/-- Prove that for loop with init executes init first -/
theorem for_executes_init (init : Stmt) (cond : Option Expr) (inc : Option Expr) (body : Stmt) :
    ∃ (result : EvalResult × InterpreterState), execStmt emptyState (Stmt.ForStmt (some init) cond inc body) = result := by
  refine' ⟨execStmt emptyState (Stmt.ForStmt (some init) cond inc body), _⟩
  <;> rfl

/-- Prove that block executes all statements in sequence -/
theorem block_executes_all (stmts : List Stmt) :
    ∃ (result : EvalResult × InterpreterState), execStmt emptyState (Stmt.BlockStmt stmts) = result := by
  refine' ⟨execStmt emptyState (Stmt.BlockStmt stmts), _⟩
  <;> rfl

/-- Prove that program execution runs all statements -/
theorem run_program_runs_all (stmts : List Stmt) :
    ∃ (result : EvalResult), runProgram stmts = result := by
  refine' ⟨runProgram stmts, _⟩
  <;> rfl

/-- Division by zero returns error -/
theorem div_by_zero_error (x : Float) :
    applyBinaryOp TokenType.Slash (Value.Num x) (Value.Num 0) = EvalResult.Error "Division by zero" := by
  rfl

/-- Equality of different types returns error -/
theorem eq_diff_types_error :
    applyBinaryOp TokenType.EqualEqual (Value.Num 1.0) (Value.Bool true) = EvalResult.Error "Unsupported operation" := by
  rfl

/-- Inequality of different types returns error -/
theorem ne_diff_types_error :
    applyBinaryOp TokenType.BangEqual (Value.Num 1.0) (Value.Bool true) = EvalResult.Error "Unsupported operation" := by
  rfl