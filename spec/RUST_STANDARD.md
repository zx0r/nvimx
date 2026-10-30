# Principal+ Rust Engineering Standard: Configuration, Micro-Binary Optimization, Toolchain, Distribution & Lifecycle

**Level:** Principal Systems Architect & Lead Platform Engineer  
**Scope:** Modern Rust (1.85 – 1.96+, 2024 Edition Ready)  
**Standard:** 100% Deterministic, Zero-Supply-Chain-Risk, Data-Oriented, Micro-Footprint Architecture  

---

## 1. Executive Architecture Summary

This specification establishes the **canonical industrial benchmark** for designing, configuring, optimizing, testing, and distributing enterprise-grade Rust systems. It synthesizes state-of-the-art practices across six critical architectural domains:

1. **Manifest & Workspace Governance:** RFC 3389 `[workspace.lints]`, MSRV-aware `resolver = "3"`, workspace dependency inheritance, and compilation profile hierarchy.
2. **Extreme Micro-Binary Engineering:** Aggressive size minimization down to $< 300\text{ KB}$ for CLI tools and $< 30\text{ KB}$ for micro-runtimes through LLVM Dead-Code Elimination (DCE), fat LTO, panic machinery stripping, and formatting bloat avoidance.
3. **Repository Source-Boundary Scoping:** Strict isolation of Rust compilation units in mixed/polyglot repositories containing datasets, scripts, notebooks, specs, and non-Rust assets via `package.include`, `.cargoignore`, and narrow `cargo::rerun-if-changed` scoping.
4. **Complete Production Configuration Suite:** Production-ready templates for `.cargo/config.toml`, `rust-toolchain.toml`, `clippy.toml`, `rustfmt.toml`, `deny.toml`, `.config/nextest.toml`, and `.cargo-llvm-cov.toml`.
5. **Universal Multi-Platform Distribution (`cargo-dist`):** Exhaustive schema specification for `dist-workspace.toml` / `dist.toml`, multi-architecture matrix packaging, shell/powershell/homebrew/npm/msi installers, cryptographic SLSA Level 3 GitHub Attestations, and in-place binary auto-updating.
6. **Enterprise Quality Gates & Lifecycle:** Inner loop acceleration (`bacon`, `mold`, `sccache`), supply chain audit ledgers (`cargo-deny`, `cargo-vet`, `cargo-machete`), mutation testing (`cargo-mutants`), Undefined Behavior detection (`miri` with Tree Borrows), and pinned, least-privilege CI/CD workflows.

---

## 2. Manifest & Workspace Governance (`Cargo.toml`)

### 2.1. Resolver 3 (`resolver = "3"`) & MSRV Awareness
Introduced in Rust 1.84/1.85 (stabilized in 1.96.0) as the default for `edition = "2024"`:
* **Feature Decoupling:** Build dependencies (`build-dependencies`) and development test dependencies (`dev-dependencies`) cannot artificially enable features in target production binaries.
* **MSRV-Aware Dependency Resolution:** Cargo evaluates candidate dependencies against the declared `workspace.package.rust-version`. If a third-party dependency releases a new minor version bumping its MSRV beyond your pinned toolchain, `cargo update` automatically falls back to the latest MSRV-compatible release, preventing CI breakage.

### 2.2. Master Workspace `Cargo.toml` Reference

```toml
[workspace]
members = [
    "crates/*",
    "services/*",
    "tools/*",
]
default-members = ["services/gateway", "crates/core"]
exclude = [
    "fuzz",
    "benches/microbenchmarks",
]
resolver = "3"

[workspace.package]
version = "0.1.0"
edition = "2024"
rust-version = "1.96.0"
authors = ["Principal Systems Architecture Group <systems@enterprise.internal>"]
license = "Apache-2.0 OR MIT"
readme = "README.md"
repository = "https://github.com/enterprise-systems/nexus-core"
homepage = "https://systems.internal/nexus"
documentation = "https://docs.rs/nexus-core"
keywords = ["systems", "low-latency", "zero-copy", "zero-alloc", "data-oriented"]
categories = ["command-line-utilities", "asynchronous", "network-programming"]
publish = false

# ------------------------------------------------------------------------------
# Source Ingestion & Repository Isolation (Guards Against Stray Non-Rust Files)
# ------------------------------------------------------------------------------
# In repositories containing notebooks, datasets, specs, and scripts:
# Explicitly whitelist exactly what enters crate archives and build inputs.
include = [
    "src/**/*",
    "tests/**/*",
    "benches/**/*",
    "Cargo.toml",
    "README.md",
    "LICENSE*",
]

[workspace.dependencies]
# Internal workspace crates
nexus-core = { version = "0.1.0", path = "crates/core", default-features = false }
nexus-net = { version = "0.1.0", path = "crates/net" }

# High-leverage std/alloc-only primitives
bytes = { version = "1.9", default-features = false }
smallvec = { version = "1.13", default-features = false, features = ["const_generics", "union"] }
ahash = { version = "0.8", default-features = false }
parking_lot = { version = "0.12", default-features = false }
thiserror = { version = "2.0", default-features = false }

# Serialization & Deserialization
serde = { version = "1.0", default-features = false, features = ["alloc", "derive"] }
rkyv = { version = "0.8", default-features = false, features = ["alloc", "size_64"] }

# ------------------------------------------------------------------------------
# Compilation Profiles (Industrial Tuning Matrix)
# ------------------------------------------------------------------------------

# 1. Dev Profile: Lightning fast incremental compilation, pre-optimized deps
[profile.dev]
opt-level = 0
debug = 2                      # Full DWARF/PDB
split-debuginfo = "unpacked"   # macOS: bypasses dsymutil; Linux: split DWARF
incremental = true
codegen-units = 256
overflow-checks = true
panic = "unwind"
rpath = false

# Pre-optimize dependencies for 10x faster local test and debugging loops
[profile.dev.package."*"]
opt-level = 3

# 2. Production Release: Maximum throughput, fat LTO, symbol stripping
[profile.release]
opt-level = 3
lto = "fat"                    # Monolithic inter-procedural optimization across all crates
codegen-units = 1              # Single codegen unit allows LLVM global dead-code elimination
panic = "abort"                # Eliminates personality routines & unwind tables (-15% binary size)
strip = "symbols"              # Strips both symbol tables & debuginfo; use "debuginfo" if preserving backtrace symbols
split-debuginfo = "off"
incremental = false
overflow-checks = false        # Max throughput; keep true for crypto/financial logic
debug = false
debug-assertions = false
rpath = false

# 2b. Distribution Profile (cargo-dist)
[profile.dist]
inherits = "release"
lto = "fat"

# 3. Micro-Release: Size-critical CLI utilities, embedded, WASM, and sidecars
[profile.release-small]
inherits = "release"
opt-level = "z"                # Aggressive code size minimization over execution speed
lto = "fat"
codegen-units = 1
panic = "abort"
strip = true

# 4. Continuous Profiling: Production speed with line-tables for perf / eBPF / flamegraph
[profile.profiling]
inherits = "release"
debug = "line-tables-only"
strip = false
panic = "unwind"

# 5. CI / Nextest Profile: Fast compilation with runtime assertions enabled
[profile.test]
inherits = "dev"
opt-level = 1
debug = 1
overflow-checks = true
split-debuginfo = "unpacked"

# 6. Microbenchmarking: Bit-identical to release with profiling symbols
[profile.bench]
inherits = "release"
debug = true
opt-level = 3
lto = "thin"

# ------------------------------------------------------------------------------
# Unified Static Lint Governance (RFC 3389 / Rust 1.74 - 1.96+)
# ------------------------------------------------------------------------------
[workspace.lints.rust]
unsafe_code = "deny"
unreachable_pub = "warn"
unused_qualifications = "warn"
unused_lifetimes = "warn"
trivial_casts = "warn"
trivial_numeric_casts = "warn"
rust_2018_idioms = { level = "warn", priority = -1 }
missing_docs = "warn"
missing_debug_implementations = "warn"
missing_copy_implementations = "warn"

[workspace.lints.clippy]
# Lint groups MUST have priority = -1
all = { level = "warn", priority = -1 }
pedantic = { level = "warn", priority = -1 }
nursery = { level = "warn", priority = -1 }
cargo = { level = "warn", priority = -1 }

# Specific enforcement (default priority = 0 overrides groups)
unwrap_used = "deny"
expect_used = "warn"
panic = "deny"
todo = "warn"
unimplemented = "warn"
unreachable = "warn"
indexing_slicing = "warn"
string_slice = "deny"
dbg_macro = "deny"
print_stdout = "warn"
print_stderr = "warn"
rc_mutex = "deny"
rc_buffer = "deny"
clone_on_ref_ptr = "warn"
await_holding_lock = "deny"
large_futures = "warn"
as_conversions = "warn"
cast_possible_truncation = "warn"
cast_possible_wrap = "warn"
cast_precision_loss = "warn"
cast_sign_loss = "warn"
fn_to_numeric_cast_any = "deny"
lossy_float_literal = "deny"
float_cmp = "deny"
float_cmp_const = "deny"
wildcard_imports = "deny"
multiple_inherent_impl = "warn"
str_to_string = "warn"
create_dir = "deny"
exit = "deny"
mem_forget = "deny"

# Intentional ergonomic allowances
module_name_repetitions = "allow"
doc_markdown = "allow"
missing_errors_doc = "allow"
missing_panics_doc = "allow"
must_use_candidate = "allow"
similar_names = "allow"
too_many_lines = "allow"
redundant_pub_crate = "allow"

[workspace.lints.rustdoc]
all = { level = "warn", priority = -1 }
broken_intra_doc_links = "deny"
private_intra_doc_links = "warn"
missing_crate_level_docs = "warn"
invalid_codeblock_attributes = "deny"
invalid_rust_codeblocks = "deny"
bare_urls = "warn"
```

---

## 3. Micro-Binary Footprint Engineering ($< 300\text{ KB}$ Standard)

Binary bloat in Rust stems from five primary culprits: **unwind landing pads**, **LLVM codegen partitions**, **symbol tables / DWARF**, **monomorphized `core::fmt` formatting engines**, and **bundled allocators**.

### 3.1. The 5 Levers of Binary Slimming

| Factor | Standard Build | Principal Micro-Optimization | Mechanism & Size Impact |
|---|---|---|---|
| **Optimization Level** | `opt-level = 3` | `opt-level = "z"` or `"s"` | Optimizes for byte density; restricts loop unrolling and function duplication ($-10\%$ to $-25\%$). |
| **Link-Time Optimization** | `lto = false` | `lto = "fat"` | Monolithic whole-program AST analysis; enables aggressive Cross-Crate Dead Code Elimination (DCE). |
| **Codegen Partitions** | `codegen-units = 16` | `codegen-units = 1` | Single compilation unit allows LLVM to prove functions are unused and purge them globally. |
| **Unwind Tables** | `panic = "unwind"` | `panic = "abort"` | Strips `.eh_frame`, exception personality routines, and landing pads ($-15\%$ to $-20\%$). |
| **Symbol Tables** | Unstripped | `strip = "symbols"` / `true` (or `"debuginfo"`) | Strips ELF/Mach-O `.symtab`, `.strtab`, and DWARF sections ($-60\%$ to $-80\%$). Use `"debuginfo"` if backtrace symbols must be preserved. |

### 3.2. Eliminating `core::fmt` & Panic String Bloat
The Rust formatting engine (`std::fmt::Display`, `format!`, `panic!("...")`) pulls in extensive string formatting tables, float-to-string algorithms (Ryū/Dragon4), and formatting buffers.

1. **Static Error Slices vs. Allocated Strings:**
   Replace dynamic string errors with static slices or flat enums:
   ```rust
   // ANTI-PATTERN: Dynamically pulls in format machinery
   return Err(format!("Invalid token on line {line}"));

   // PRINCIPAL STANDARD: Zero-allocation, static string representation
   pub enum Error {
       InvalidToken(u32),
       MissingHeader(&'static str),
   }
   ```
2. **Nightly Micro-Targeting (`build-std`):**
   When deploying to size-critical containers or micro-VMs, rebuild `core` and `std` without panic formatting:
   ```bash
   cargo +nightly build --release \
     -Z build-std=std,panic_abort \
     -Z build-std-features=panic_immediate_abort
   ```
   *Result:* Replaces panic formatting with a single CPU instruction (`ud2` on x86, `brk` on ARM), cutting the baseline runtime from $\sim 300\text{ KB}$ down to $< 30\text{ KB}$.

3. **Allocator Selection:**
   - Standard library defaults to system `malloc` on macOS/Windows and `glibc` on Linux.
   - Adding `jemalloc` or `mimalloc` improves multithreaded allocation throughput by $15\%$, but adds $300\text{ KB} - 500\text{ KB}$ of fixed static binary overhead. For ultra-lightweight CLI tools, retain the native `System` allocator.

4. **Linker Dead-Code Stripping:**
   Configured in `.cargo/config.toml` (see Section 5.1):
   - **Linux:** `-C link-arg=-Wl,--gc-sections`
   - **macOS:** `-C link-arg=-Wl,-dead_strip`
   - **Windows:** `-C link-arg=/OPT:REF /OPT:ICF`

5. **Diagnostic Binary Profilers:**
   - `cargo bloat --release --crates`: Identifies which dependencies contribute the most bytes to the `.text` segment.
   - `cargo bloat --release -n 30`: Lists the top 30 largest compiled functions/monomorphizations.

---

## 4. Repository Source Scoping & Non-Rust File Isolation

In mixed-content repositories containing Python scripts, Jupyter notebooks, datasets, documentation, and architecture specs, Cargo can suffer from **false compilation cache invalidation**, **bloated package tarballs**, and **unintended asset embedding**.

### 4.1. Crate Inclusion Boundary (`package.include`)
By default, `cargo package` and `cargo publish` package all files in the directory unless listed in `.gitignore`.  
**Principal Mandate:** Always explicitly declare `include` in `Cargo.toml`:
```toml
[package]
name = "nexus-core"
version = "0.1.0"
edition = "2024"

# Strict boundary: ONLY compile and package Rust sources and essential metadata.
# Prevents specs, datasets, .py, and .ipynb files from dirtying the package checksum.
include = [
    "src/**/*",
    "tests/**/*",
    "benches/**/*",
    "Cargo.toml",
    "LICENSE*",
    "README.md",
]
```

### 4.2. Build Script (`build.rs`) Hygiene
A poorly written `build.rs` can trigger re-compilation on every single `cargo check` if any file in the repository changes:

```rust
// ❌ ANTI-PATTERN: Re-compiles whenever ANY file in the workspace changes (e.g. editing a .md file)
println!("cargo::rerun-if-changed=.");

// ✅ PRINCIPAL STANDARD: Target specific files and deterministic directories
fn main() {
    println!("cargo::rerun-if-changed=src");
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-changed=Cargo.toml");
}
```

### 4.3. Embedding Hygiene (`include_bytes!` & `include_str!`)
* Never embed non-Rust files from external folders without pinning them in `build.rs`.
* Use the `.cargoignore` file to ensure packaging tools ignore development sidecars, artifacts, and local environments:
```gitignore
# .cargoignore
target/
.specs/
spec/
notebooks/
data/
*.ipynb
*.py
*.sh
```

---

## 5. Complete Production Configuration Suite

### 5.1. `.cargo/config.toml` (Linker Acceleration & Cross-Target Flags)

```toml
[build]
rustc-wrapper = "sccache"
rustflags = ["-Wrust_2024_compatibility"]

# ------------------------------------------------------------------------------
# High-Speed Linker Configuration (mold on Linux, lld on macOS/Windows)
# ------------------------------------------------------------------------------

# Linux x86_64: Mold Linker (4x-10x link speedup)
[target.x86_64-unknown-linux-gnu]
linker = "clang"
rustflags = [
    "-C", "link-arg=-fuse-ld=mold",
    "-C", "link-arg=-Wl,--threads=auto",
    "-C", "link-arg=-Wl,--gc-sections",
]

# Linux AArch64: Mold Linker
[target.aarch64-unknown-linux-gnu]
linker = "clang"
rustflags = [
    "-C", "link-arg=-fuse-ld=mold",
    "-C", "link-arg=-Wl,--threads=auto",
    "-C", "link-arg=-Wl,--gc-sections",
]

# Linux MUSL Static: LLD Linker
[target.x86_64-unknown-linux-musl]
linker = "clang"
rustflags = [
    "-C", "link-arg=-fuse-ld=lld",
    "-C", "target-feature=+crt-static",
    "-C", "link-arg=-Wl,--gc-sections",
]

# macOS Apple Silicon: Dead code stripping & header padding
[target.aarch64-apple-darwin]
rustflags = [
    "-C", "link-arg=-Wl,-dead_strip",
    "-C", "link-arg=-Wl,-headerpad_max_install_names",
]

# macOS Intel x86_64
[target.x86_64-apple-darwin]
rustflags = [
    "-C", "link-arg=-Wl,-dead_strip",
    "-C", "link-arg=-Wl,-headerpad_max_install_names",
]

# Windows MSVC: lld-link fastlink with unused reference stripping
[target.x86_64-pc-windows-msvc]
linker = "lld-link"
rustflags = [
    "-C", "link-arg=/DEBUG:FASTLINK",
    "-C", "link-arg=/OPT:REF",
    "-C", "link-arg=/OPT:ICF",
]

# ------------------------------------------------------------------------------
# Emulated Runners
# ------------------------------------------------------------------------------
[target.wasm32-wasip1]
runner = "wasmtime run --dir=."

[target.aarch64-unknown-linux-musl]
runner = "qemu-aarch64 -L /usr/aarch64-linux-gnu"

# ------------------------------------------------------------------------------
# Principal Productivity Aliases
# ------------------------------------------------------------------------------
[alias]
qa = "clippy --workspace --all-targets --all-features -- -D warnings"
fmt-check = "fmt --all -- --check"
deny-check = "deny check"
t = "nextest run --all-features"
t-ci = "nextest run --profile ci --all-features"
cov = "llvm-cov --workspace --all-features --lcov --output-path target/llvm-cov/lcov.info"
cov-html = "llvm-cov --workspace --all-features --html --open"
bloat-crates = "bloat --release --crates"
bloat-fns = "bloat --release -n 30"
udeps = "+nightly udeps --workspace --all-targets"
```

### 5.2. `rust-toolchain.toml` (Pinned Reproducibility)

```toml
[toolchain]
channel = "1.96.0"
profile = "minimal"
components = [
    "rustfmt",
    "clippy",
    "rust-analyzer",
    "llvm-tools",
    "rust-src",
]
targets = [
    "x86_64-unknown-linux-gnu",
    "x86_64-unknown-linux-musl",
    "aarch64-unknown-linux-gnu",
    "aarch64-unknown-linux-musl",
    "x86_64-apple-darwin",
    "aarch64-apple-darwin",
    "x86_64-pc-windows-msvc",
    "wasm32-wasip1",
]
```

### 5.3. `clippy.toml` (Metric Thresholds & Primitive Bans)

```toml
msrv = "1.96.0"
cognitive-complexity-threshold = 25
cyclomatic-complexity-threshold = 20
too-many-arguments-threshold = 7
too-many-lines-threshold = 100
type-complexity-threshold = 250
excessive-nesting-threshold = 4
array-size-threshold = 512
warn-on-all-wildcard-imports = true
allow-unwrap-in-tests = true

doc-valid-idents = [
    "GitHub", "GitLab", "OpenTelemetry", "PostgreSQL", "MySQL", "SQLite",
    "Kubernetes", "OAuth", "GraphQL", "WebAssembly", "JSON", "HTTP", "HTTPS",
    "TCP", "UDP", "TLS", "IPC", "API", "REST", "SIMD", "UUID", "MSRV", "RAII", "POSIX"
]

# Ban non-deterministic or slow types
disallowed-types = [
    { path = "std::collections::HashMap", reason = "Use `ahash::AHashMap` for high throughput or `BTreeMap` for deterministic key ordering." },
    { path = "std::collections::HashSet", reason = "Use `ahash::AHashSet` or `BTreeSet`." },
    { path = "std::sync::Mutex", reason = "Standard library Mutex lacks timeouts and fair queuing; use `parking_lot::Mutex` or `tokio::sync::Mutex`." },
    { path = "std::sync::RwLock", reason = "Standard library RwLock suffers from writer starvation; use `parking_lot::RwLock`." },
]

# Ban dangerous methods
disallowed-methods = [
    { path = "std::env::set_var", reason = "Mutating environment variables in multithreaded runtimes triggers POSIX undefined behavior." },
    { path = "std::env::remove_var", reason = "Mutating environment variables in multithreaded runtimes is thread-unsafe." },
    { path = "std::mem::forget", reason = "Use `std::mem::ManuallyDrop` to guard resource lifecycles explicitly." },
    { path = "std::time::Instant::now", reason = "Direct clock reads break deterministic unit testing; inject a time provider or use `tokio::time::Instant`." },
]
```

### 5.4. `rustfmt.toml` (Canonical Layout Engine)

```toml
edition = "2024"
max_width = 100
hard_tabs = false
tab_spaces = 4
newline_style = "Unix"

use_field_init_shorthand = true
use_try_shorthand = true
reorder_imports = true
reorder_modules = true
```

> **Note on Nightly Formatting:** Advanced AST formatting options (`imports_granularity`, `group_imports`, `wrap_comments`) require `unstable_features = true` and must be executed exclusively via `cargo +nightly fmt`. The baseline configuration above is 100% stable-compatible on Rust 1.96.0+.

### 5.5. `deny.toml` (`cargo-deny` Supply Chain Policy)

```toml
[graph]
targets = [
    "x86_64-unknown-linux-gnu",
    "x86_64-unknown-linux-musl",
    "aarch64-unknown-linux-gnu",
    "aarch64-unknown-linux-musl",
    "aarch64-apple-darwin",
    "x86_64-apple-darwin",
    "x86_64-pc-windows-msvc",
]
all-features = true
exclude-dev = false

[advisories]
db-path = "~/.cargo/advisory-db"
db-urls = ["https://github.com/rustsec/advisory-db"]
unmaintained = "workspace"
unsound = "workspace"
yanked = "deny"
ignore = []

[bans]
multiple-versions = "deny"
wildcards = "deny"
highlight = "all"
allow-workspace = true

deny = [
    { crate = "openssl", use-instead = "rustls" },
    { crate = "lazy_static", use-instead = "std::sync::LazyLock" },
    { crate = "once_cell", use-instead = "std::sync::LazyLock / std::sync::OnceLock" },
    { crate = "log", use-instead = "tracing" },
    { crate = "env_logger", use-instead = "tracing-subscriber" },
    { crate = "failure", use-instead = "thiserror / anyhow" },
    { crate = "chrono", use-instead = "time (Chrono has past RUSTSEC RCE advisories)" },
]
skip = []
skip-tree = []

[licenses]
confidence-threshold = 0.93
include-build = true
allow = [
    "MIT",
    "Apache-2.0",
    "Apache-2.0 WITH LLVM-exception",
    "BSD-2-Clause",
    "BSD-3-Clause",
    "ISC",
    "CC0-1.0",
    "Zlib",
    "Unicode-3.0",
    "Unicode-DFS-2016",
]
exceptions = [
    { allow = ["Zlib"], crate = "adler32" },
    { allow = ["BSD-3-Clause"], crate = "ring" },
]

[[licenses.clarify]]
crate = "ring"
expression = "ISC AND MIT AND OpenSSL"
license-files = [{ path = "LICENSE", hash = 0xbd0eed23 }]

[sources]
unknown-registry = "deny"
unknown-git = "deny"
allow-registry = [
    "https://github.com/rust-lang/crates.io-index",
    "sparse+https://index.crates.io/",
]
allow-git = []
```

### 5.6. `.config/nextest.toml` (`cargo-nextest` Execution Engine)

```toml
[profile.default]
test-threads = "num-cpus"
retries = 0
failure-output = "immediate"
success-output = "never"
status-level = "pass"
slow-timeout = { period = "60s", terminate-after = 2 }

[profile.default.leak-sanitizer]
threads-per-test = 1

[profile.ci]
retries = 2
fail-fast = false
failure-output = "immediate-final"
success-output = "never"
status-level = "fail"
slow-timeout = { period = "30s", terminate-after = 3, grace-period = "10s" }

[profile.ci.junit]
path = "target/nextest/ci/junit.xml"
report-name = "nexus-unit-tests"
store-success-output = false

[[profile.default.overrides]]
filter = 'test(/^integration::/) | test(/^e2e::/)'
slow-timeout = { period = "180s", terminate-after = 2 }
threads-per-test = 2

[[profile.ci.overrides]]
filter = 'test(/^integration::/) | test(/^e2e::/)'
slow-timeout = { period = "240s", terminate-after = 2 }
retries = 3
```

### 5.7. `.cargo-llvm-cov.toml` (Source-Based Coverage)

```toml
[coverage]
min-total = 80.0
min-per-crate = 70.0

[report]
formats = ["html", "lcov", "text"]
html-dir = "target/llvm-cov/html"
lcov-file = "target/llvm-cov/lcov.info"

[ignore]
patterns = [
    "tests/*",
    "benches/*",
    "examples/*",
    "src/bin/*",
]

[workspace]
include-all-members = true

[test]
all-features = true
```

---

## 6. Multi-Platform Distribution Specification (`cargo-dist`)

`cargo-dist` (by axodotdev, versions `0.28.x` – `0.29.x+`) handles build orchestration, packaging, installer generation, checksumming, SLSA Level 3 attestations, and GitHub Releases.

```
                    ┌────────────────────────────────────────────────────────┐
                    │                      git push v1.0.0                   │
                    └───────────────────────────┬────────────────────────────┘
                                                │
                                                ▼
                                   [ Stage 1: dist plan ]
                               (Computes Target Build Matrix)
                                                │
                     ┌──────────────────────────┼──────────────────────────┐
                     ▼                          ▼                          ▼
          [ Stage 2: Linux Build ]    [ Stage 2: macOS Build ]   [ Stage 2: Win Build ]
          (x86_64 / aarch64 musl)      (Apple Silicon / Intel)      (x86_64 / arm64)
                     │                          │                          │
                     └──────────────────────────┼──────────────────────────┘
                                                │
                                                ▼
                                [ Stage 3: Global Artifacts ]
                             (Installers: sh, ps1, brew, npm, msi)
                                                │
                                                ▼
                                  [ Stage 4: GitHub Release ]
                               (Upload Assets, Tarballs, Checksums)
                                                │
                                                ▼
                                  [ Stage 5: Sign & Publish ]
                             - SLSA Level 3 Artifact Attestations
                             - Homebrew Formula Commit (Tap Repo)
                             - npm / Registry Publication
```

### 6.1. Master `dist-workspace.toml` Reference

```toml
[workspace]
members = ["cargo:."]

[dist]
# Pinned version executing in CI runners
cargo-dist-version = "0.33.1"

# Target CI engine
ci = ["github"]

# CI execution behavior on Pull Requests:
# - "plan": Dry-runs build matrix calculation without actual compilation
# - "upload": Builds artifacts and attaches them to PR run summary
# - "skip": Disables PR execution
pr-run-mode = "plan"

precise-cli-versions = true

# Target Platform Triples
targets = [
  "x86_64-unknown-linux-gnu",
  "x86_64-unknown-linux-musl",
  "aarch64-unknown-linux-gnu",
  "aarch64-unknown-linux-musl",
  "armv7-unknown-linux-gnueabihf",
  "x86_64-apple-darwin",
  "aarch64-apple-darwin",
  "x86_64-pc-windows-msvc",
  "aarch64-pc-windows-msvc"
]

# Installers Generated in Global Phase
installers = [
  "shell",
  "powershell",
  "homebrew",
  "npm",
  "msi"
]

# Destination path where shell/ps1 scripts place binaries
install-path = ["$CARGO_HOME/bin", "~/.local/bin"]

# Companion in-place auto-updater (<bin>-update binary)
install-updater = true

# Scope for npm wrapper packages (e.g. `npx @myorg/cli`)
npm-scope = "@myorg"

# Archive Compression
windows-archive = ".zip"
unix-archive = ".tar.gz"

# Homebrew Tap Repository & Publish Jobs
tap = "myorg/homebrew-tap"
formula = "my-cli"
publish-jobs = ["homebrew"]

# Supply Chain Provenance (SLSA Level 3 via GitHub Attestations)
github-attestations = true
github-attestations-phase = "announce"

# Embed dependency graph into compiled binaries for runtime vulnerability scanners
cargo-auditable = true

# GitHub Actions Runner Machine Mapping
[dist.github-custom-runners]
global = "ubuntu-latest"
aarch64-apple-darwin = "macos-latest"
x86_64-apple-darwin = "macos-latest"
x86_64-unknown-linux-gnu = "ubuntu-latest"
x86_64-unknown-linux-musl = "ubuntu-latest"
aarch64-unknown-linux-gnu = "ubuntu-latest"
aarch64-unknown-linux-musl = "ubuntu-latest"
x86_64-pc-windows-msvc = "windows-latest"
aarch64-pc-windows-msvc = "windows-latest"

# Extra Custom Assets
[[dist.extra-artifacts]]
artifacts = ["schemas/config-schema.json"]
build = ["cargo", "run", "--bin", "generate-schema", "--", "schemas/config-schema.json"]
```

### 6.2. Package-Level Metadata (`[package.metadata.dist]`)
Declared in individual crate `Cargo.toml` files within workspaces:
```toml
[package.metadata.dist]
dist = true
binaries = ["my-cli", "my-daemon"]
all-features = false
default-features = true
features = ["tls-vendored"]
```

### 6.3. Signing, Notarization & Provenance
1. **GitHub Artifact Attestations (SLSA Level 3):**
   - Automatically signed using GitHub’s internal Sigstore root via OIDC (`id-token: write`).
   - End-users verify binary integrity directly via GitHub CLI:
     ```bash
     gh attestation verify ./my-cli-aarch64-apple-darwin.tar.xz --repo myorg/my-cli
     ```
2. **Windows Authenticode:**
   - Supports Azure Trusted Signing via `[dist.azure-windows-sign]` with zero local certificate management.
3. **macOS Codesigning & Notarization:**
   - Injected via workflow hooks using `rcodesign` or `apple-codesign` with `Developer ID Application` certificates and Apple `notarytool` API keys.

---

## 7. Industrial Ecosystem Lifecycle & Tooling Matrix

```
┌────────────────────────────────────────────────────────────────────────┐
│                        1. LOCAL INNER LOOP                             │
│   bacon (TUI daemon) ──► sccache (compiler cache) ──► mold/lld (fast)  │
└───────────────────────────────────┬────────────────────────────────────┘
                                    │ git commit
                                    ▼
┌────────────────────────────────────────────────────────────────────────┐
│                        2. STATIC GATES & AUDITS                        │
│   cargo-clippy ──► cargo-deny (licenses/bans) ──► cargo-machete (deps) │
└───────────────────────────────────┬────────────────────────────────────┘
                                    │ git push
                                    ▼
┌────────────────────────────────────────────────────────────────────────┐
│                   3. VERIFICATION & SOUNDNESS (CI)                     │
│   cargo-nextest (isolated) ──► cargo-llvm-cov ──► miri (Tree Borrows)   │
└───────────────────────────────────┬────────────────────────────────────┘
                                    │ git tag vX.Y.Z
                                    ▼
┌────────────────────────────────────────────────────────────────────────┐
│                   4. MULTI-PLATFORM DISTRIBUTION                       │
│   cargo-dist ──► SLSA Provenance ──► Homebrew / Shell / Powershell     │
└────────────────────────────────────────────────────────────────────────┘
```

### 7.1. Industrial Tooling Spectrum

| Tool | Category | Architectural Role | Enterprise Advantage |
|---|---|---|---|
| **`bacon`** | Inner Loop | Background check & test daemon | Sub-second TUI feedback; pauses during typing; zero screen flicker. |
| **`mold` / `lld`** | Linker | Multi-threaded binary linking | Cuts link phase from $20\text{s}$ down to $< 1.0\text{s}$ on large workspaces. |
| **`sccache`** | Build Cache | Shared compilation cache (S3/GCS/Redis) | Eliminates redundant compilation across feature branches and CI nodes. |
| **`cargo-nextest`** | Test Runner | Process-isolated parallel test execution | Retries flaky tests; JUnit XML reporting; prevents runner-killing segfaults. |
| **`cargo-llvm-cov`** | Code Coverage | Source-based LLVM instrumentation | Accurate branch/line metrics without DWARF distortion or ptrace crashes. |
| **`cargo-deny`** | Security Gate | Multi-pillar policy governance | Blocks duplicate dependencies, copyleft licenses, and RustSec CVEs. |
| **`cargo-machete`** | Hygiene | Ultra-fast unused dependency detector | AST/ripgrep scanner running in $15\text{ ms}$; replaces slow `cargo-udeps`. |
| **`cargo-semver-checks`**| API Governance | Semantic breaking change detection | Queries `rustdoc` JSON graph to prevent accidental API breakage. |
| **`cargo-mutants`** | Quality Gate | Mutation testing engine | Inverts operators to prove test suites actually catch defects. |
| **`miri`** | Memory Soundness| MIR Undefined Behavior interpreter | Validates pointer provenance, alignment, and aliasing rules via Tree Borrows. |
| **`divan`** | Performance | Benchmark suite with allocation profiling | Native allocation volume tracking; replaces heavy macro-bound Criterion. |
| **`cargo-dist`** | Distribution | Multi-platform packaging & release pipeline | Auto-generates shell/ps1/brew installers, SLSA provenance, and GitHub releases. |
| **`cargo-binstall`** | Distribution | Zero-compile binary installer | Downloads pre-compiled release artifacts in seconds without compiling. |

---

## 8. Hardened Enterprise CI/CD Pipeline (GitHub Actions)

Production-hardened workflow featuring **SHA-pinned actions**, **least-privilege permissions**, and **isolated gate flow**.

```yaml
# .github/workflows/ci.yml
name: "CI Enterprise Quality Gates"

on:
  push:
    branches: ["main"]
  pull_request:
    branches: ["main"]

permissions:
  contents: "read"

env:
  CARGO_TERM_COLOR: "always"
  CARGO_INCREMENTAL: "0"

jobs:
  # ----------------------------------------------------------------------------
  # Gate 1: Fast Static Verification (< 60 seconds)
  # ----------------------------------------------------------------------------
  fast-checks:
    name: "Fast Quality Gates (Fmt, Clippy, Machete)"
    runs-on: "ubuntu-latest"
    steps:
      - name: "Checkout Source Code"
        uses: "actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1" # v7.0.1

      - name: "Install Pinned Rust Toolchain"
        run: "rustup show"

      - name: "Setup Rust Cache"
        uses: "Swatinem/rust-cache@d1527c8f67ef9af357290475d4af4b0fab8f12e0" # v2.9.3
        with:
          save-if: ${{ github.ref == 'refs/heads/main' }}

      - name: "Verify Rust Formatting"
        run: "cargo fmt --all -- --check"

      - name: "Install cargo-machete"
        uses: "taiki-e/install-action@86f7a0ffb25f40114606501a311ebcfaf882ea32" # v2.88.0
        with:
          tool: "cargo-machete@0.9.2"

      - name: "Detect Unused Dependencies"
        run: "cargo machete"

      - name: "Run Clippy (Pedantic/Nursery)"
        run: "cargo clippy --workspace --all-targets --all-features -- -D warnings"

  # ----------------------------------------------------------------------------
  # Gate 2: Supply Chain Security & License Policy
  # ----------------------------------------------------------------------------
  security-audit:
    name: "Supply Chain Audit (Cargo Deny)"
    runs-on: "ubuntu-latest"
    steps:
      - uses: "actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1" # v7.0.1
      - uses: "EmbarkStudios/cargo-deny-action@8371184b9282b7fd3378f16f800a444ce7288e5f" # v2.0.11
        with:
          command: "check"
          arguments: "--all-features"

  # ----------------------------------------------------------------------------
  # Gate 3: Test Execution & LLVM Source-Based Coverage
  # ----------------------------------------------------------------------------
  test-and-coverage:
    name: "Tests (Nextest) & Source Coverage"
    needs: ["fast-checks", "security-audit"]
    runs-on: "ubuntu-latest"
    permissions:
      id-token: "write"
    steps:
      - uses: "actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1" # v7.0.1
      - run: "rustup show"
      - uses: "Swatinem/rust-cache@d1527c8f67ef9af357290475d4af4b0fab8f12e0" # v2.9.3
        with:
          save-if: ${{ github.ref == 'refs/heads/main' }}

      - name: "Install Nextest & LLVM-Cov"
        uses: "taiki-e/install-action@51287a9df2c0509a247291a978f8cb2142279b9b" # v2
        with:
          tool: "cargo-nextest,cargo-llvm-cov"

      - name: "Execute Isolated Tests with Coverage"
        run: |
          cargo llvm-cov nextest \
            --profile ci \
            --workspace \
            --all-features \
            --fail-under-lines 80.0 \
            --lcov \
            --output-path target/llvm-cov/lcov.info

  # ----------------------------------------------------------------------------
  # Gate 4: Memory Model & Undefined Behavior (Miri)
  # ----------------------------------------------------------------------------
  miri-verification:
    name: "Memory Safety & Soundness (Miri)"
    needs: ["fast-checks"]
    runs-on: "ubuntu-latest"
    steps:
      - uses: "actions/checkout@11bd71901bbe5b1630ceea73d27597364c9af683"
      - name: "Install Nightly Miri"
        run: "rustup toolchain install nightly --component miri"
      - name: "Run Miri with Tree Borrows"
        env:
          MIRIFLAGS: "-Zmiri-tree-borrows -Zmiri-symbolic-alignment-check -Zmiri-strict-provenance"
        run: "cargo +nightly miri test --workspace --lib"
```

---

## 9. Conclusion & Operational Checklist

| Stage | Command / Verification Artifact | Success Invariant |
|---|---|---|
| **Compilation Boundary** | `package.include` in `Cargo.toml` | Only Rust files are hashed/packaged; non-Rust assets ignored. |
| **Code Formatting** | `cargo fmt --all -- --check` | 100% adherence to `rustfmt.toml` styling rules. |
| **Static Enforcement** | `cargo qa` (`cargo clippy ... -D warnings`) | 0 warnings across all targets under `pedantic`/`nursery`. |
| **Supply Chain** | `cargo deny check` | Zero advisories, approved licenses, and 0 duplicate crate versions. |
| **Test Concurrency** | `cargo t-ci` (`cargo nextest ...`) | All tests pass in dedicated isolated processes. |
| **Memory Soundness** | `cargo +nightly miri test --lib` | 0 provenance, alignment, or Tree Borrows aliasing violations. |
| **Binary Footprint** | `cargo bloat --release --crates` | Stripped production binary fits within strict size budget ($< 350\text{ KB}$). |
| **Release & Provenance**| `dist build` & `gh attestation verify` | Multi-arch packages, installers, and SLSA Level 3 attestations generated. |
