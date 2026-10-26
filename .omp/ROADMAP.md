# psudo Roadmap: Interpreted → Compiled

## Vision

Build a dual-mode language: **interpreted first** (rapid iteration, memory safety validation), **compiled second** (safe native artifacts). The interpreter acts as a "borrow checker simulator" — ironing out ownership/memory issues before code reaches the compiler backend.

---

## Phase 0: Current State (Interpreter Complete)

**Status:** ✅ Done
- Tree-walking interpreter with visitor pattern
- Sandbox filesystem with project config
- LSP server for IDE support
- Lean 4 formal verification (24/30 theorems)
- REPL, scripts, tests all passing

---

## Phase 1: IR & Memory Model Design (Interpreted Phase)

**Goal:** Design intermediate representation that mirrors Rust's ownership model, validate in interpreter.

### 1.1 Ownership IR
- [ ] Design `Value` enum with ownership tags: `Owned`, `Borrowed(&mut)`, `Borrowed(&)`, `Static`
- [ ] Add lifetime annotations to IR nodes
- [ ] Implement borrow checking in interpreter (panic on violation, not UB)
- [ ] Track move semantics explicitly in AST

### 1.2 Memory Safety in Interpreter
- [ ] Stack frames with explicit drop order
- [ ] Reference counting for shared ownership (Rc/Arc equivalent)
- [ ] Interior mutability detection (RefCell equivalent)
- [ ] Panic on use-after-move, double-free, dangling reference

### 1.3 Borrow Checker Simulation
- [ ] Implement NLL (non-lexical lifetimes) in interpreter
- [ ] Track loan regions per variable
- [ ] Report precise error: "value borrowed here, used here after move"
- [ ] Match Rust error messages as closely as possible

### 1.4 Test Suite for Memory Model
- [ ] Port Rust borrow checker test suite (from rust-lang/rust)
- [ ] Add psudo-specific tests
- [ ] CI: run interpreter with memory checks on all examples

---

## Phase 2: Compiler Backend - Single Platform (x86_64 Linux)

**Goal:** Emit safe native code for one platform first.

### 2.1 Codegen Target Selection
- [ ] Choose: LLVM IR → `inkwell` OR Cranelift OR direct x86_64 assembly
- **Recommendation:** Cranelift (faster compile, Rust-native, good for JIT/AOT)

### 2.2 Lowering Pipeline
```
AST → HIR (with lifetimes) → MIR (borrow-checked) → Cranelift IR → object file
```

### 2.3 Memory Model in Codegen
- [ ] Stack slots for owned values
- [ ] Pointer arithmetic for references
- [ ] Drop glue insertion at scope end
- [ ] No runtime borrow checking (verified in Phase 1)

### 2.4 Runtime
- [ ] Minimal runtime: allocator, panic handler, abort on OOM
- [ ] No GC — all memory managed via ownership
- [ ] `jemalloc` or `mimalloc` for performance

### 2.5 Platform Support: x86_64 Linux
- [ ] ELF executable emission
- [ ] System calls via `libc` or raw syscalls
- [ ] Dynamic linking for libc only
- [ ] Static linking option

---

## Phase 3: Cross-Platform Support

**Goal:** Extend compiler to multiple targets.

### 3.1 Target Triples
| Tier | Target | Status |
|------|--------|--------|
| 1 | x86_64-unknown-linux-gnu | Phase 2 |
| 1 | x86_64-pc-windows-msvc | Phase 3.1 |
| 1 | aarch64-apple-darwin | Phase 3.2 |
| 2 | x86_64-unknown-linux-musl | Phase 3.3 |
| 2 | aarch64-unknown-linux-gnu | Phase 3.4 |
| 3 | wasm32-wasip1 | Phase 3.5 |

### 3.2 Platform Abstraction
- [ ] Abstract syscall interface
- [ ] Platform-specific stdlib modules
- [ ] Conditional compilation for target features

### 3.3 Cross-Compilation
- [ ] `psudo build --target <triple>`
- [ ] Docker-based CI for each target
- [ ] Pre-built runtime artifacts per platform

---

## Phase 4: Language Evolution

### 4.1 Module System
- [ ] `mod` / `use` / `pub`
- [ ] File-based modules (like Rust)
- [ ] Package management (`psudo new`, `psudo add`)

### 4.2 Generics & Traits
- [ ] Monomorphization in compiler
- [ ] Trait objects (vtable)
- [ ] Const generics (array sizes)

### 4.3 Async/Await
- [ ] State machine lowering
- [ ] Executor integration (optional)

### 4.4 Standard Library Expansion
| Module | Phase |
|--------|-------|
| `std::io` | 3 |
| `std::fs` | 3 |
| `std::net` | 4 |
| `std::thread` | 4 |
| `std::sync` | 4 |
| `std::collections` | 3 |
| `std::fmt` | 3 |

---

## Migration Strategy

### Interpreted → Compiled Workflow

```bash
# 1. Develop in interpreter (fast feedback, memory checks)
psudo -f src/main.psudo

# 2. Run with strict borrow checking
psudo --check-borrows -f src/main.psudo

# 3. Compile for current platform
psudo build --release

# 4. Cross-compile
psudo build --target x86_64-pc-windows-msvc
```

### Compatibility Guarantees
- **Interpreted = Spec**: If it runs in interpreter, it compiles
- **No interpreter-only features**: All features must have codegen path
- **Same semantics**: No "works in interpreter, UB in compiled"

---

## Milestones

| Milestone | Target | Deliverable |
|-----------|--------|-------------|
| M1 | IR + borrow checker in interpreter | `--check-borrows` flag |
| M2 | Cranelift backend, x86_64 Linux | `psudo build` → native binary |
| M3 | Windows + macOS | Cross-compile from Linux |
| M4 | Module system + stdlib | Usable for real projects |
| M5 | WASM target | Web deployment |

---

## Risk Mitigation

| Risk | Mitigation |
|------|------------|
| Cranelift learning curve | Start with simple functions, iterate |
| Borrow checker complexity | Port Rust's test suite incrementally |
| Runtime size | Optional stdlib, minimal default |
| Cross-compile complexity | Docker CI per target from day 1 |
| Lean 4 verification gap | Verify MIR lowering, not just interpreter |

---

## Decision Log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-10-26 | Cranelift over LLVM | Faster compile, Rust ecosystem, JIT capable |
| 2026-10-26 | Single platform first | Reduce scope, validate pipeline |
| 2026-10-26 | Interpreter as borrow checker | Catch memory bugs before codegen |

---

## Next Actions

1. **Immediate**: Design ownership IR in `src/ir/` module
2. **Week 1**: Implement borrow checking in interpreter (`--check-borrows`)
3. **Week 2**: Add Cranelift dependency, emit simple function
4. **Week 3**: Stack frame management, drop glue
5. **Week 4**: Full x86_64 Linux executable

---

*Roadmap is living — update as decisions change.*