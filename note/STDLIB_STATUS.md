# psudo stdlib implementation status

## Sandbox filesystem (✅ complete)

| Function | Status | Notes |
|----------|--------|-------|
| `read_file(path)` | ✅ | Returns string content |
| `write_file(path, content)` | ✅ | Atomic write via temp file |
| `list_dir(path)` | ✅ | Returns sorted entry names |
| `exists(path)` | ✅ | Returns bool |
| `is_file(path)` | ✅ | Returns bool |
| `is_dir(path)` | ✅ | Returns bool |
| `mkdir(path)` | ✅ | Creates directory, error if exists |

**Security features:**
- Path traversal blocked (`..` rejected)
- Absolute paths rejected
- Configurable size limits (file + sandbox total)
- Project-level config via `psudo.toml`
- Deny list for system directories
- Sandbox auto-created on first use

## Config system (✅ complete)

| Feature | Status |
|---------|--------|
| Auto-detect `psudo.toml` walking up from CWD | ✅ |
| Sandbox root, size limits configurable | ✅ |
| External read allow/deny lists | ✅ |
| Sensible defaults (no config needed) | ✅ |

## Built-in functions (✅ complete)

| Function | Status |
|----------|--------|
| `print` | ✅ |
| `read_file` | ✅ |
| `write_file` | ✅ |
| `list_dir` | ✅ |
| `exists` | ✅ |
| `isFile` | ✅ |
| `isDir` | ✅ |
| `mkdir` | ✅ |

## Language features (✅ complete)

| Feature | Status |
|---------|--------|
| Variables (`let`, assignment) | ✅ |
| Control flow (`if/else`, `while`, `for`) | ✅ |
| Blocks `{ ... }` | ✅ |
| Numbers, strings, booleans, nil | ✅ |
| Arithmetic, comparison, logical ops | ✅ |
| String concatenation | ✅ |

## REPL (✅ complete)

| Feature | Status |
|---------|--------|
| Interactive mode | ✅ |
| Multi-line input | ❌ (single-line only) |
| History | ❌ |
| Tab completion | ❌ |

## LSP server (✅ complete)

| Feature | Status |
|---------|--------|
| Initialize/shutdown | ✅ |
| Text document sync (didOpen/didChange) | ✅ |
| Hover (variables, keywords, operators) | ✅ |
| Completion (keywords, built-ins, variables) | ✅ |
| Diagnostics (syntax, undefined vars) | ✅ |
| Semantic tokens | ✅ |

## Formal verification (Lean 4, ⚠️ partial)

| Theorem group | Status |
|---------------|--------|
| Literal preservation | ✅ |
| Binary/unary ops | ✅ |
| Comparisons | ✅ |
| Truthiness | ✅ |
| Division by zero safety | ✅ |
| Type preservation | ✅ |
| Full evaluation proofs | ❌ (6 `sorry`) |

## Pending / future stdlib

| Feature | Priority | Notes |
|---------|----------|-------|
| `read_line()` / stdin | Medium | For interactive input |
| `exec(cmd)` | Low | Security risk, needs design |
| `sleep(ms)` | Low | |
| `random()` | Low | |
| `time()` / `date()` | Low | |
| JSON parse/stringify | Medium | |
| HTTP client | Low | |
| Module system / `import` | High | Needed for larger programs |
| Functions (`fn`) | Partial | Parser has token, interpreter TODO |
| Arrays / lists | ✅ | Parser + interpreter |
| Objects / maps | ✅ | Parser + interpreter |

## Testing

| Suite | Tests | Status |
|-------|-------|--------|
| Integration | 7 | ✅ |
| LSP | 9 | ✅ |
| Sandbox | 6 | ✅ |
| **Total** | **22** | ✅ |