# Warning-Free Compilation Guide

**Scope** → Best practices for maintaining zero-warning builds across all compilation targets, following patterns from production Rust crates like `serde` and `ripgrep`.

⸻

## 1. Own the Compilation Units

| Rule | Why | How |
|------|-----|-----|
| **1.1** Put "belongs-to-only-one-target" code in that target's file | If a helper is used only by your CLI, compiling the library for `cargo test` can't "see" it | `src/bin/cli.rs` (or `src/bin/cli/mod.rs`) owns its own helpers; they never touch `src/lib.rs` |
| **1.2** Split shared-but-optional code behind a feature | Multiple targets need it, but not every build. Features tell Cargo exactly when to compile it | `#[cfg(feature = "bench-helpers")] mod bench_helpers;` |
| **1.3** Promote helpers that must be reachable by all builds into the library API | Then the test/bench build of the library sees a call-site and the lint is happy | In `lib.rs`: `pub(crate) fn parse_header(...) { … }` |

⸻

## 2. Wire the Features in Cargo.toml

```toml
[features]
cli           = []          # code only the CLI needs
bench-helpers = []          # helper code for Criterion benches
default       = ["cli"]     # or keep [] if you want zero-feature core

[[bin]]
name             = "mytool"
path             = "src/bin/cli.rs"
required-features = ["cli"]       # <-- hard gate

[[bench]]
name             = "throughput"
harness          = false          # because Criterion supplies its own
required-features = ["bench-helpers"]
```

`required-features` makes Cargo skip the target when the feature is off, so CI builds like `cargo test --no-default-features` stay clean.

⸻

## 3. Gate the Code, Not the Lints

```rust
// src/lib.rs
cfg_if::cfg_if! {
    if #[cfg(feature = "bench-helpers")] {
        mod bench_helpers;
        pub use bench_helpers::ProfilingTimer;
    }
}

#[cfg(any(feature = "cli", test))]           // used by CLI *and* unit tests
pub(crate) fn parse_args() { … }
```

The compiler never sees an item that's unused in the current build, so `dead_code` never fires.

⸻

## 4. Compile with the Right Flavours

```bash
# Fast debug run of the CLI
cargo run --features cli --bin mytool -- <args>

# Strict tests with *only* the features they need
cargo test --no-default-features          # core crate
cargo test --features cli                 # integration tests that invoke the binary

# Benches
cargo bench --no-default-features --features bench-helpers
```

Because each invocation enables exactly the features its root target asked for, every build graph is self-contained and dead-code-free.

⸻

## 5. CI Guard-rails

1. **One job runs `cargo clippy --all-targets --all-features --deny warnings`**  
   That is the superset build; if it is clean, smaller builds are clean too.

2. **Another job runs `cargo test --no-default-features`**  
   Catches anything that default features accidentally hide.

⸻

## 6. "Isn't there a global tree-shaker?"

Not yet. The `dead_code` lint runs before LLVM's DCE and before Cargo can merge results from other targets. An RFC ("conditional-compilation checking" 3013) exists but is still experimental. Until it lands, precise feature fencing + `required-features` is the production-grade fix.

⸻

## Concrete Layout Cheat-sheet

```
mycrate/
├─ Cargo.toml
├─ src/
│  ├─ lib.rs            ← core API (no warnings)
│  └─ bin/
│     └─ cli.rs         ← owns all CLI-only helpers
├─ benches/
│   └─ throughput.rs    ← uses `cfg(feature="bench-helpers")`
└─ tests/
    └─ integration.rs   ← may opt-in to `features = ["cli"]`
```

Follow these six rules and you can leave `#![deny(dead_code)]` at the top of every crate and still enjoy warning-free `cargo test`, `cargo bench`, and `cargo run`.

⸻

## Project-Specific Applications

For this codebase:

- **CLI helpers** → Move to `src/bin/seams.rs` or `src/bin/seams/mod.rs`
- **Benchmark utilities** → Gate behind `bench-helpers` feature
- **Test utilities** → Use `#[cfg(test)]` or promote to `pub(crate)` if shared
- **Dialog detector APIs** → Used by benchmarks, promote to public API with clear documentation

**Never use `#[allow(dead_code)]`** — instead, architect the compilation units correctly so warnings never appear.