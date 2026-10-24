# pseudo-lang Standard Library Roadmap

## Vision
Build a minimal, safe standard library for pseudo-lang with sandboxed I/O operations.

---

## Phase 1: Filesystem ✅ COMPLETE
**Goal**: Safe file read/write/list operations restricted to a sandbox directory.

### Safety Requirements
- **Path containment**: All operations restricted to `./sandbox/` directory
- **No path traversal**: Reject `..`, absolute paths, symlinks outside sandbox
- **Size limits**: Max file size 1MB, max total sandbox 10MB
- **No destructive ops**: No `rm`, `mv`, `chmod`, `chown` in Phase 1
- **Atomic writes**: Write to temp + rename for durability

### API (Built-in Functions)
```
readFile(path: string)       -> string | error
writeFile(path: string, content: string|number|bool) -> nil | error
listDir(path?: string)       -> string (newline-separated) | error
exists(path: string)         -> bool
isFile(path: string)         -> bool
isDir(path: string)          -> bool
mkdir(path: string)          -> nil | error
```

### Implementation Status
- [x] Sandbox config (default `./sandbox`)
- [x] Path resolver module (`src/fs/sandbox.rs`)
- [x] Built-in function registry via `visit_call`
- [x] `readFile` implementation
- [x] `writeFile` implementation
- [x] `listDir` implementation
- [x] `exists` / `isFile` / `isDir`
- [x] `mkdir` implementation
- [x] Integration tests in `scripts/fs_test.psudo`
- [x] 6 unit tests passing

---

## Phase 2: Process & Time
**Goal**: Basic process control and time functions.

### API Design
```
sleep(ms: number)            -> nil        // max 5000ms
now()                        -> number     // ms since epoch
exec(cmd: string, args?: [string]) -> string | error  // allowlist only
env(key: string)             -> string | nil
exit(code?: number)          -> never
```

### Implementation Plan
1. Add time module with `std::time`
2. Implement sleep with timeout enforcement
3. Process execution with command allowlist
4. Environment variable access (read-only)
5. Exit function for scripts

---

## Phase 3: Network (Future)
**Goal**: HTTP client with safety restrictions.

### API Design
```
httpGet(url: string)         -> {status: number, body: string, headers: map} | error
httpPost(url: string, body: string, headers?: map) -> same | error
httpRequest(method: string, url: string, opts?: map) -> same | error
```

### Safety Requirements
- Domain allowlist (configurable)
- Request size limits (1MB)
- Response size limits (10MB)
- Timeout (30s default)
- No automatic redirects
- TLS verification required

---

## Phase 4: Data Formats
**Goal**: Parse/serialize common data formats.

### API Design
```
jsonParse(str: string)       -> value | error
jsonStringify(value)         -> string | error
csvParse(str: string)        -> [[string]] | error
csvStringify(rows: [[string]]) -> string
tomlParse(str: string)       -> value | error
```

### Implementation Plan
1. Add `serde_json`, `csv`, `toml` dependencies
2. Map pseudo-lang values to serde Value
3. Handle type conversions (number, string, bool, nil, array, object)

---

## Phase 5: Crypto/Encoding
**Goal**: Common encoding and cryptographic operations.

### API Design
```
base64Encode(str: string)    -> string
base64Decode(str: string)    -> string | error
hexEncode(str: string)       -> string
hexDecode(str: string)       -> string | error
sha256(str: string)          -> string (hex)
uuid()                       -> string
randomBytes(n: number)       -> string (hex)
```

---

## Phase 6: Collections & Utilities
**Goal**: Higher-level data structures and utilities.

### API Design
```
range(start: number, end?: number, step?: number) -> [number]
len(value: string|[any])   -> number
keys(obj: map)             -> [string]
values(obj: map)           -> [any]
has(obj: map, key: string) -> bool
clone(value)               -> any
typeOf(value)              -> string  // "number"|"string"|"bool"|"nil"|"array"|"object"
```

---

## Complete Standard Library Reference

### Core (Built-in)
| Function | Category | Description |
|----------|----------|-------------|
| `print(...)` | I/O | Print values to stdout |
| `readFile(path)` | FS | Read file from sandbox |
| `writeFile(path, content)` | FS | Write file to sandbox |
| `listDir(path?)` | FS | List directory contents |
| `exists(path)` | FS | Check if path exists |
| `isFile(path)` | FS | Check if path is file |
| `isDir(path)` | FS | Check if path is directory |
| `mkdir(path)` | FS | Create directory |

### Phase 2 (Planned)
| Function | Category | Description |
|----------|----------|-------------|
| `sleep(ms)` | Time | Sleep for milliseconds (max 5s) |
| `now()` | Time | Current timestamp (ms since epoch) |
| `exec(cmd, args?)` | Process | Execute allowlisted command |
| `env(key)` | Process | Get environment variable |
| `exit(code?)` | Process | Exit script with code |

### Phase 3 (Planned)
| Function | Category | Description |
|----------|----------|-------------|
| `httpGet(url)` | Network | HTTP GET request |
| `httpPost(url, body, headers?)` | Network | HTTP POST request |
| `httpRequest(method, url, opts?)` | Network | Generic HTTP request |

### Phase 4 (Planned)
| Function | Category | Description |
|----------|----------|-------------|
| `jsonParse(str)` | Data | Parse JSON string |
| `jsonStringify(value)` | Data | Serialize to JSON |
| `csvParse(str)` | Data | Parse CSV string |
| `csvStringify(rows)` | Data | Serialize to CSV |
| `tomlParse(str)` | Data | Parse TOML string |

### Phase 5 (Planned)
| Function | Category | Description |
|----------|----------|-------------|
| `base64Encode(str)` | Encoding | Base64 encode |
| `base64Decode(str)` | Encoding | Base64 decode |
| `hexEncode(str)` | Encoding | Hex encode |
| `hexDecode(str)` | Encoding | Hex decode |
| `sha256(str)` | Crypto | SHA-256 hash (hex) |
| `uuid()` | Crypto | Generate UUID v4 |
| `randomBytes(n)` | Crypto | Random bytes (hex) |

### Phase 6 (Planned)
| Function | Category | Description |
|----------|----------|-------------|
| `range(start, end?, step?)` | Collections | Generate number sequence |
| `len(value)` | Utils | Length of string/array |
| `keys(obj)` | Utils | Object keys |
| `values(obj)` | Utils | Object values |
| `has(obj, key)` | Utils | Check key exists |
| `clone(value)` | Utils | Deep clone |
| `typeOf(value)` | Utils | Get type name |

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

### Phase 1: Filesystem ✅
- [x] Sandbox config (default `./sandbox`)
- [x] Path resolver module (`src/fs/sandbox.rs`)
- [x] Built-in function registry
- [x] `readFile` implementation
- [x] `writeFile` implementation
- [x] `listDir` implementation
- [x] `exists` / `isFile` / `isDir`
- [x] `mkdir` implementation
- [x] Integration tests in `scripts/fs_test.psudo`
- [x] Documentation in README
- [x] Unit tests (6 passing)

### Phase 2: Process & Time
- [ ] Time module (`src/time/`)
- [ ] `sleep(ms)` with max 5s enforcement
- [ ] `now()` timestamp
- [ ] Process execution with allowlist
- [ ] `env(key)` read-only
- [ ] `exit(code?)`
- [ ] Tests in `scripts/process_test.psudo`

### Phase 3: Network
- [ ] HTTP client with `reqwest`
- [ ] Domain allowlist config
- [ ] Size limits & timeouts
- [ ] TLS verification
- [ ] Tests in `scripts/network_test.psudo`

### Phase 4: Data Formats
- [ ] JSON parse/stringify
- [ ] CSV parse/stringify
- [ ] TOML parse
- [ ] Tests in `scripts/data_test.psudo`

### Phase 5: Crypto/Encoding
- [ ] Base64, hex encoding
- [ ] SHA-256
- [ ] UUID v4
- [ ] Random bytes
- [ ] Tests in `scripts/crypto_test.psudo`

### Phase 6: Collections & Utilities
- [ ] Range function
- [ ] Len, keys, values, has
- [ ] Clone, typeOf
- [ ] Tests in `scripts/utils_test.psudo`