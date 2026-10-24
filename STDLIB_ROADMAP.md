# pseudo-lang Standard Library Roadmap

## Vision
Build a minimal, safe standard library for pseudo-lang with sandboxed I/O operations.

## Phase 1: Filesystem (Current)
**Goal**: Safe file read/write/list operations restricted to a sandbox directory.

### Safety Requirements
- **Path containment**: All operations restricted to `./sandbox/` directory
- **No path traversal**: Reject `..`, absolute paths, symlinks outside sandbox
- **Size limits**: Max file size 1MB, max total sandbox 10MB
- **No destructive ops**: No `rm`, `mv`, `chmod`, `chown` in Phase 1
- **Atomic writes**: Write to temp + rename for durability

### API Design (Built-in Functions)
```
readFile(path: string) -> string | error
writeFile(path: string, content: string) -> nil | error
listDir(path: string) -> [string] | error
exists(path: string) -> bool
isFile(path: string) -> bool
isDir(path: string) -> bool
mkdir(path: string) -> nil | error
```

### Implementation Plan
1. Add sandbox directory config (env var or CLI flag)
2. Implement path resolution with containment check
3. Add built-in functions to interpreter
4. Register in global environment
5. Test with scripts

## Phase 2: Process & Time
- `sleep(ms: number)` - sandboxed sleep (max 5s)
- `now()` - current timestamp (ms since epoch)
- `exec(cmd: string, args: [string])` - restricted allowlist only

## Phase 3: Network (Future)
- HTTP client with domain allowlist
- WebSocket (maybe)

## Phase 4: Data Formats
- JSON parse/stringify
- CSV, TOML (maybe)

## Phase 5: Crypto/Encoding
- Base64, hex
- Hash (sha256)
- UUID generation

---

## Sandbox Architecture

```
project/
├── sandbox/           # <-- ONLY directory accessible to stdlib
│   ├── data/
│   ├── logs/
│   └── tmp/
├── scripts/           # User scripts (read-only from interpreter POV)
└── src/               # Interpreter source
```

### Path Resolution Rules
1. Normalize path (resolve `.`, remove duplicate `/`)
2. Check for `..` components → REJECT
3. Check for absolute path → REJECT
4. Prepend sandbox root
5. Verify result starts with sandbox root (canonical)
6. Verify not a symlink escaping sandbox

### Error Types
- `FSError.NotInSandbox` - path escapes sandbox
- `FSError.NotFound` - file/dir doesn't exist
- `FSError.PermissionDenied` - not allowed (e.g., write to dir)
- `FSError.TooLarge` - file/sandbox size limit exceeded
- `FSError.IOError` - underlying OS error

---

## Implementation Checklist

- [ ] Sandbox config (env var `PSUDO_SANDBOX` default `./sandbox`)
- [ ] Path resolver module (`src/fs/sandbox.rs`)
- [ ] Built-in function registry
- [ ] `readFile` implementation
- [ ] `writeFile` implementation
- [ ] `listDir` implementation
- [ ] `exists` / `isFile` / `isDir`
- [ ] `mkdir` implementation
- [ ] Integration tests in `scripts/fs_test.psudo`
- [ ] Documentation in README