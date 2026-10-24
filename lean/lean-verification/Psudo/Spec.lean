/-- Formal specification of the pseudo-lang (psudo) interpreter in Lean 4 -/

namespace Psudo

/-- Token types matching the Rust implementation -/
inductive TokenType
  | LeftParen | RightParen | LeftBrace | RightBrace | Comma | Dot | Minus | Plus | Semicolon | Slash | Star
  | Bang | BangEqual | Equal | EqualEqual | Greater | GreaterEqual | Less | LessEqual
  | Identifier | String | Number
  | And | Class | Else | False | Fun | For | If | Nil | Or | Print | Return | This | True | Let | While
  | Eof

/-- Literal values -/
inductive LiteralValue
  | Str : String → LiteralValue
  | Num : Float → LiteralValue
  | Bool : Bool → LiteralValue
  | Nil : LiteralValue

/-- Token with metadata -/
structure Token where
  tokenType : TokenType
  text : String
  literal : Option LiteralValue
  line : Nat

/-- Expression AST -/
inductive Expr where
  | Binary : Expr → Token → Expr → Expr
  | Unary : Token → Expr → Expr
  | Literal : Token → Expr
  | Grouping : Expr → Expr
  | Assign : String → Expr → Expr

/-- Statement AST -/
inductive Stmt where
  | ExprStmt : Expr → Stmt
  | PrintStmt : Expr → Stmt
  | LetStmt : String → Expr → Stmt
  | IfStmt : Expr → Stmt → Option Stmt → Stmt
  | WhileStmt : Expr → Stmt → Stmt
  | ForStmt : Option Stmt → Option Expr → Option Expr → Stmt → Stmt
  | BlockStmt : List Stmt → Stmt

/-- Runtime values -/
inductive Value where
  | Num : Float → Value
  | Bool : Bool → Value
  | Str : String → Value
  | Nil : Value

/-- Environment for variable storage -/
def Environment := String → Option Value

/-- Evaluation result -/
inductive EvalResult where
  | Ok : Value → EvalResult
  | Error : String → EvalResult

/-- Interpreter state -/
structure InterpreterState where
  env : Environment

def emptyState : InterpreterState := ⟨fun _ => none⟩

/-- Lookup variable in environment -/
def lookupVar (env : Environment) (name : String) : EvalResult :=
  match env name with
  | some v => EvalResult.Ok v
  | none => EvalResult.Error ("Undefined variable: " ++ name)

/-- Convert LiteralValue to Value -/
def literalToValue (lit : LiteralValue) : Value :=
  match lit with
  | LiteralValue.Str s => Value.Str s
  | LiteralValue.Num n => Value.Num n
  | LiteralValue.Bool b => Value.Bool b
  | LiteralValue.Nil => Value.Nil

/-- Convert Token to Value (for literals) -/
def tokenToValue (tok : Token) : EvalResult :=
  match tok.literal with
  | some lit => EvalResult.Ok (literalToValue lit)
  | none =>
    match tok.tokenType with
    | TokenType.True => EvalResult.Ok (Value.Bool true)
    | TokenType.False => EvalResult.Ok (Value.Bool false)
    | TokenType.Nil => EvalResult.Ok Value.Nil
    | TokenType.Identifier => lookupVar emptyState tok.text
    | _ => EvalResult.Error "Expected literal token"

/-- Binary operations on Values -/
def applyBinaryOp (op : TokenType) (a b : Value) : EvalResult :=
  match op, a, b with
  | TokenType.Plus, Value.Num x, Value.Num y => EvalResult.Ok (Value.Num (x + y))
  | TokenType.Plus, Value.Str x, Value.Str y => EvalResult.Ok (Value.Str (x ++ y))
  | TokenType.Minus, Value.Num x, Value.Num y => EvalResult.Ok (Value.Num (x - y))
  | TokenType.Slash, Value.Num x, Value.Num y =>
    if y = 0 then EvalResult.Error "Division by zero" else EvalResult.Ok (Value.Num (x / y))
  | TokenType.Star, Value.Num x, Value.Num y => EvalResult.Ok (Value.Num (x * y))
  | TokenType.EqualEqual, Value.Num x, Value.Num y => EvalResult.Ok (Value.Bool (x = y))
  | TokenType.EqualEqual, Value.Bool x, Value.Bool y => EvalResult.Ok (Value.Bool (x = y))
  | TokenType.EqualEqual, Value.Str x, Value.Str y => EvalResult.Ok (Value.Bool (x = y))
  | TokenType.BangEqual, Value.Num x, Value.Num y => EvalResult.Ok (Value.Bool (x ≠ y))
  | TokenType.BangEqual, Value.Bool x, Value.Bool y => EvalResult.Ok (Value.Bool (x ≠ y))
  | TokenType.BangEqual, Value.Str x, Value.Str y => EvalResult.Ok (Value.Bool (x ≠ y))
  | TokenType.Less, Value.Num x, Value.Num y => EvalResult.Ok (Value.Bool (x < y))
  | TokenType.LessEqual, Value.Num x, Value.Num y => EvalResult.Ok (Value.Bool (x ≤ y))
  | TokenType.Greater, Value.Num x, Value.Num y => EvalResult.Ok (Value.Bool (x > y))
  | TokenType.GreaterEqual, Value.Num x, Value.Num y => EvalResult.Ok (Value.Bool (x ≥ y))
  | _, _, _ => EvalResult.Error ("Unsupported operation: " ++ op.toString)

/-- Unary operations on Values -/
def applyUnaryOp (op : TokenType) (v : Value) : EvalResult :=
  match op, v with
  | TokenType.Minus, Value.Num x => EvalResult.Ok (Value.Num (-x))
  | TokenType.Bang, Value.Bool x => EvalResult.Ok (Value.Bool (!x))
  | _, _ => EvalResult.Error ("Unsupported unary operation: " ++ op.toString)

/-- Truthiness of a Value (Lox semantics) -/
def isTruthy (v : Value) : Bool :=
  match v with
  | Value.Nil => false
  | Value.Bool b => b
  | _ => true

/-- Evaluate an expression -/
partial def evalExpr (state : InterpreterState) (e : Expr) : EvalResult :=
  match e with
  | Expr.Literal tok => tokenToValue tok
  | Expr.Grouping e => evalExpr state e
  | Expr.Unary opTok right =>
    evalExpr state right |>.bind (applyUnaryOp opTok.tokenType)
  | Expr.Binary left opTok right =>
    evalExpr state left |>.bind (fun a =>
    evalExpr state right |>.bind (fun b =>
    applyBinaryOp opTok.tokenType a b))
  | Expr.Assign name value =>
    evalExpr state value |>.bind (fun v =>
      EvalResult.Ok v) -- Note: state update handled at statement level

/-- Execute a statement -/
partial def execStmt (state : InterpreterState) (s : Stmt) : EvalResult × InterpreterState :=
  match s with
  | Stmt.ExprStmt e =>
    (evalExpr state e, state)
  | Stmt.PrintStmt e =>
    (evalExpr state e, state)
  | Stmt.LetStmt name init =>
    match evalExpr state init with
    | EvalResult.Ok v =>
      (EvalResult.Ok v, ⟨fun n => if n = name then some v else state.env n⟩)
    | EvalResult.Error e =>
      (EvalResult.Error e, state)
  | Stmt.IfStmt cond thenBranch elseBranch =>
    match evalExpr state cond with
    | EvalResult.Ok condVal =>
      if isTruthy condVal then
        execStmt state thenBranch
      else
        match elseBranch with
        | some elseStmt => execStmt state elseStmt
        | none => (EvalResult.Ok Value.Nil, state)
    | EvalResult.Error e =>
      (EvalResult.Error e, state)
  | Stmt.WhileStmt cond body =>
    let rec whileLoop (s : InterpreterState) : EvalResult × InterpreterState :=
      match evalExpr s cond with
      | EvalResult.Ok condVal =>
        if isTruthy condVal then
          match execStmt s body with
          | (EvalResult.Error e, s') => (EvalResult.Error e, s')
          | (_, s') => whileLoop s'
      | EvalResult.Error e => (EvalResult.Error e, s)
    whileLoop state
  | Stmt.ForStmt init cond increment body =>
    let rec forLoop (s : InterpreterState) (initDone : Bool) : EvalResult × InterpreterState :=
      if initDone then
        match cond with
        | some condExpr =>
          match evalExpr s condExpr with
          | EvalResult.Ok condVal =>
            if isTruthy condVal then
              match execStmt s body with
              | (EvalResult.Error e, s') => (EvalResult.Error e, s')
              | (_, s') =>
                match increment with
                | some incExpr =>
                  match evalExpr s' incExpr with
                  | EvalResult.Ok _ => forLoop s' true
                  | EvalResult.Error e => (EvalResult.Error e, s')
                | none => forLoop s' true
            else (EvalResult.Ok Value.Nil, s')
          | EvalResult.Error e => (EvalResult.Error e, s)
        | none =>
          match execStmt s body with
          | (EvalResult.Error e, s') => (EvalResult.Error e, s')
          | (_, s') =>
            match increment with
            | some incExpr =>
              match evalExpr s' incExpr with
              | EvalResult.Ok _ => forLoop s' true
              | EvalResult.Error e => (EvalResult.Error e, s')
            | none => forLoop s' true
      else
        match init with
        | some initStmt =>
          match execStmt s initStmt with
          | (EvalResult.Error e, s') => (EvalResult.Error e, s')
          | (_, s') => forLoop s' true
        | none => forLoop s true
    forLoop state false
  | Stmt.BlockStmt stmts =>
    let rec execBlock (s : InterpreterState) (stmts : List Stmt) : EvalResult × InterpreterState :=
      match stmts with
      | [] => (EvalResult.Ok Value.Nil, s)
      | stmt :: rest =>
        match execStmt s stmt with
        | (EvalResult.Error e, s') => (EvalResult.Error e, s')
        | (_, s') => execBlock s' rest
    execBlock state stmts

/-- Run a program (list of statements) -/
def runProgram (stmts : List Stmt) : EvalResult :=
  let rec run (state : InterpreterState) (stmts : List Stmt) : EvalResult :=
    match stmts with
    | [] => EvalResult.Ok Value.Nil
    | stmt :: rest =>
      match execStmt state stmt with
      | (EvalResult.Error e, _) => EvalResult.Error e
      | (_, newState) => run newState rest
  run emptyState stmts

end Psudo