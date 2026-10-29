# Architectural Specification: `nvimx` Reference Engine
**System:** Isolated Neovim Profile Manager & Zero-Overhead Dispatcher  
**Standard:** 100% Zero-Dependency (`std`-only), Data-Oriented Design, Linear Memory Ownership, Zero-Copy POSIX Image Replacement  
**Status:** Approved for Implementation (Synthesized Production Canon)  

---

## 1. System Overview & Performance Budgets

`nvimx` is a zero-dependency, zero-overhead Neovim profile manager and environment dispatcher. It replaces heavyweight runtime dependencies and interpretation layers with a pure `std`-only Rust implementation featuring direct POSIX process replacement (`execvp`), 0-cost argument disambiguation, and deterministic XDG layout resolution.

### Target Performance Budgets

* **Cold Startup Latency (Fast-Path):** $< 0.1$ ms (0 filesystem syscalls for `-v`, `--version`, `help`).
* **Launch Overhead (Hot-Path):** $\le 1.0$ ms (well below the $\le 15.0$ ms E2E test threshold).
* **Binary Footprint:** $\sim 300 - 350$ KB stripped release binary.
* **External Dependencies:** `0` crates (`[dependencies]` section in `Cargo.toml` is empty).
* **Code Volume:** $\le 950$ LOC across the entire codebase.

---

## 2. Flat File Tree & Strict LOC Budget

```
nvimx/
├── Cargo.toml                 #  45 LOC | Zero dependencies, strict clippy lints, fat LTO
├── Justfile                   #  25 LOC | Standard automation: build, test, lint, e2e
├── SPEC.md                    # Upstream system specification contract
├── spec/
│   └── ARCHITECTURE.md        # Architectural specification (this document)
├── tests/
│   └── e2e/
│       └── validate.sh        # Hard compatibility suite (6/6 tests invariant)
└── src/
    ├── main.rs                # 120 LOC | Central control flow, dispatch loop & main()
    ├── types.rs               # 120 LOC | Domain models, const BUILTIN_PRESETS, ProfileId newtype
    ├── error.rs               #  60 LOC | Single flat Error enum, Result<T>, Display, From impls
    ├── cli.rs                 # 180 LOC | Zero-copy args_os() parser & fast-path bypass
    ├── config.rs              # 120 LOC | Line-based TOML parser, [presets], tilde expansion
    ├── layout.rs              # 170 LOC | Unified EnvLayout resolver (NVIM_APPNAME & XDG mappings)
    ├── net.rs                 # 100 LOC | External git/gh/curl execution & release JSON parser
    └── term.rs                # 140 LOC | TTY detection, NO_COLOR-compliant ANSI, fzf TUI pipe
```

### LOC Budget Table

| File | Hard LOC Limit | Responsibility | Key Invariants Enforced |
|---|---|---|---|
| `Cargo.toml` | **45** | Manifest & strict lints | 0 dependencies, `strip = "symbols"`, pedantic clippy |
| `Justfile` | **25** | Developer automation | Single-command build, test, lint, and e2e validation |
| `src/main.rs` | **120** | Orchestration & process replacement | Complete lifecycle on one screen; `execvp` never returns |
| `src/types.rs` | **120** | Domain models, `const PRESETS`, `ProfileId` | "Parse, Don't Validate" newtype; embedded starter catalog |
| `src/error.rs` | **60** | Unified error enum & exit codes | No external macros; `std::fmt::Display` and `From<io::Error>` |
| `src/cli.rs` | **180** | Zero-copy `args_os()` parser | Fast-path bypass (`-v`, `help`) with 0 syscalls / 0 FS reads |
| `src/config.rs`| **120** | Minimal TOML line parser | Resilient to corrupted files; parses `[presets]` & defaults |
| `src/layout.rs`| **170** | Unified `EnvLayout` & atomic file ops | Single resolver for all commands; `EXDEV` atomic fallback |
| `src/net.rs` | **100** | External `git`/`gh`/`curl` execution | 0 TLS stack; minimal GitHub releases JSON parser for `update` |
| `src/term.rs` | **140** | ANSI styling & interactive TUI | Suppress escape codes if `!is_terminal()` or `NO_COLOR` present |
| **TOTAL** | **$\le$ 950** | **Entire Application** | **Strip release binary: $\le$ 350 KB; Startup latency: $\le$ 1.0 ms** |

---

## 3. Core Data Types (Types-First Specification)

### 3.1. Error Model (`src/error.rs`)

```rust
// src/error.rs

use std::fmt;
use std::io;
use std::process::ExitCode;

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Parse { line: usize, reason: &'static str },
    InvalidProfileName(String),
    ReservedNameUsed(String),
    ProfileNotFound(String),
    ProfileAlreadyExists(String),
    MissingDependency(&'static str),
    CommandFailed { cmd: String, status: i32 },
    VersionParseError(String),
    SelfUpdateFailed(String),
    OperationCancelled,
    Generic(String),
}

pub type Result<T> = std::result::Result<T, Error>;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(err) => write!(f, "I/O error: {err}"),
            Self::Parse { line, reason } => write!(f, "config parse error on line {line}: {reason}"),
            Self::InvalidProfileName(name) => write!(f, "invalid profile name: '{name}'"),
            Self::ReservedNameUsed(name) => write!(f, "'{name}' is a reserved command and cannot be used as a profile name"),
            Self::ProfileNotFound(name) => write!(f, "profile '{name}' not found"),
            Self::ProfileAlreadyExists(name) => write!(f, "profile '{name}' already exists"),
            Self::MissingDependency(tool) => write!(f, "missing essential dependency: {tool}"),
            Self::CommandFailed { cmd, status } => write!(f, "command '{cmd}' failed with status {status}"),
            Self::VersionParseError(ver) => write!(f, "failed to parse version string: '{ver}'"),
            Self::SelfUpdateFailed(msg) => write!(f, "self-update failed: {msg}"),
            Self::OperationCancelled => write!(f, "operation cancelled by user"),
            Self::Generic(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(err) => Some(err),
            _ => None,
        }
    }
}

impl From<io::Error> for Error {
    #[inline]
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}

impl Error {
    #[inline]
    pub fn exit_code(&self) -> ExitCode {
        match self {
            Self::OperationCancelled => ExitCode::SUCCESS,
            _ => ExitCode::FAILURE,
        }
    }
}
```

---

### 3.2. Domain Models & Embedded Catalog (`src/types.rs`)

```rust
// src/types.rs

use std::ffi::OsString;
use std::path::PathBuf;
use crate::error::{Error, Result};

/// Built-in canonical Neovim starters. Embedded in .rodata: 0 syscalls, 0 ms latency.
#[derive(Debug, Clone, Copy)]
pub struct BuiltinPreset {
    pub name: &'static str,
    pub repo: &'static str,
    pub description: &'static str,
}

pub const BUILTIN_PRESETS: &[BuiltinPreset] = &[
    BuiltinPreset {
        name: "lazyvim",
        repo: "https://github.com/LazyVim/starter",
        description: "Full-featured distribution based on lazy.nvim",
    },
    BuiltinPreset {
        name: "astronvim",
        repo: "https://github.com/AstroNvim/template",
        description: "Aesthetic, extensible distribution template",
    },
    BuiltinPreset {
        name: "nvchad",
        repo: "https://github.com/NvChad/starter",
        description: "Blazing fast Neovim config with custom UI",
    },
    BuiltinPreset {
        name: "kickstart",
        repo: "https://github.com/nvim-lua/kickstart.nvim",
        description: "Minimal, single-file starting point",
    },
];

/// List of 11 system-reserved identifiers that cannot be used as profile names.
pub const RESERVED_COMMANDS: [&str; 11] = [
    "list", "install", "clean", "doctor", "setup",
    "sandbox", "update", "completions", "help", "repro", "debug",
];

/// A guaranteed valid profile identifier (Parse, Don't Validate).
/// Invariants:
/// 1. Length 1..=64 bytes.
/// 2. Strictly ASCII `[a-zA-Z0-9_-]`.
/// 3. Cannot match any of RESERVED_COMMANDS.
/// 4. 0 path-traversal risk (no `.`, `/`, `\`, null bytes, control chars).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProfileId(String);

impl ProfileId {
    pub fn parse(raw: &str) -> Result<Self> {
        let bytes = raw.as_bytes();
        if bytes.is_empty() || bytes.len() > 64 {
            return Err(Error::InvalidProfileName(raw.to_string()));
        }
        for &b in bytes {
            let valid = b.is_ascii_alphanumeric() || b == b'_' || b == b'-';
            if !valid {
                return Err(Error::InvalidProfileName(raw.to_string()));
            }
        }
        for &reserved in &RESERVED_COMMANDS {
            if raw == reserved {
                return Err(Error::ReservedNameUsed(raw.to_string()));
            }
        }
        Ok(Self(raw.to_string()))
    }

    #[inline(always)]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Parsed operational command from CLI args.
#[derive(Debug)]
pub enum Command {
    /// Fast-path commands (0 syscalls, instant exit)
    Version,
    Help { command: Option<String> },

    /// Hot-path Neovim invocation
    Launch(LaunchPayload),

    /// Service subcommands
    List { plain: bool },
    Install { repo_or_preset: String, name: String },
    Clean { profile: String, auto_confirm: bool },
    Doctor { json: bool },
    Setup(SetupOptions),
    Sandbox { target: String },
    Update,
    Completions { shell: ShellKind },

    /// Diagnostic reserved identifiers (always exit code 1)
    ReservedAbort { name: &'static str },
}

/// Payload required for Neovim process replacement.
#[derive(Debug)]
pub struct LaunchPayload {
    pub profile: Option<ProfileId>,
    pub nvim_args: Vec<OsString>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShellKind {
    Bash,
    Zsh,
    Fish,
    Elvish,
    PowerShell,
}

#[derive(Debug, Default)]
pub struct SetupOptions {
    pub force: bool,
    pub shell: Option<ShellSetupOptions>,
}

#[derive(Debug, Default)]
pub struct ShellSetupOptions {
    pub override_nvim: bool,
    pub force_env: bool,
    pub print_env: bool,
    pub print_script: bool,
    pub dry_run: bool,
    pub remove: bool,
}

/// Minimal in-memory configuration mapped from `config.toml`.
#[derive(Debug, Clone)]
pub struct Config {
    pub default_profile: Option<ProfileId>,
    pub use_fzf: bool,
    pub log_level: LogLevel,
    pub output_format: OutputFormat,
    pub auto_update: bool,
    pub auto_install_missing: bool,
    pub strict_mode: bool,
    pub fallback_to_plain_nvim: bool,
    pub profiles_dir: Option<PathBuf>,
    pub user_presets: Vec<(String, String)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Text,
    Json,
}

/// Diagnostic health status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthStatus {
    Ok,
    Warn,
    Error,
}

impl HealthStatus {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Warn => "warn",
            Self::Error => "error",
        }
    }
}

/// Parsed SemVer representation: (major, minor, patch).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SemVer(pub u32, pub u32, pub u32);
```

---

### 3.3. Unified Environment Layout (`src/layout.rs`)

```rust
// src/layout.rs

use std::path::PathBuf;

/// Complete environment topology and computed Neovim variables.
#[derive(Clone, Debug)]
pub struct EnvLayout {
    pub config_root: PathBuf,
    pub data_root: PathBuf,
    pub state_root: PathBuf,
    pub cache_root: PathBuf,
    pub system_config: PathBuf,
    pub profiles_dir: PathBuf,
    /// Calculated XDG_CONFIG_HOME for custom profiles_dir (§2.2)
    pub xdg_config_home: Option<PathBuf>,
    /// Calculated NVIM_APPNAME ("nvim/<profile>")
    pub nvim_appname: Option<String>,
}
```

---

## 4. Module Boundary Signatures & Ownership Flow

```
                    CLI Input: std::env::args_os()
                                 │
                   cli::parse_invocation(args)
                                 │  [Zero-Copy Slices]
                                 ▼
                         cli::Invocation
                                 │
           ┌─────────────────────┴─────────────────────┐
           ▼                                           ▼
 [Fast-Path: -v, help]                      [Subcommand / Hot Path]
           │                                           │
       exit(0)                             config::load_config(&layout)
                                                       │
                                           layout::resolve(&cfg, profile)
                                                       │
                                   ┌───────────────────┴───────────────────┐
                                   ▼                                       ▼
                          term::select_profile_fzf()             layout::exec_nvim()
                                   │                                       │
                                   └───────────────────────────────────────┘
                                                       │
                                                  execvp("nvim")
```

### 4.1. CLI Parsing Boundary (`src/cli.rs`)

```rust
use std::ffi::OsString;
use crate::types::Command;
use crate::error::Result;

pub fn parse_invocation(args: Vec<OsString>) -> Result<Command>;
pub fn try_fast_path(first_arg: Option<&std::ffi::OsStr>) -> Option<Command>;
```

### 4.2. Layout & Process Replacement Boundary (`src/layout.rs`)

```rust
use std::path::{Path, PathBuf};
use std::ffi::OsString;
use crate::types::{Config, ProfileId};
use crate::error::Result;

impl EnvLayout {
    /// Computes unified XDG and NVIM_APPNAME layout strictly following SPEC.md sections 1.1.1 and 2.2.
    pub fn resolve(cfg: &Config, profile: Option<&ProfileId>) -> Self;

    /// Resolves the 4 isolated XDG paths for a profile (Config, Data, State, Cache).
    pub fn profile_storage_paths(&self, profile: &str) -> [PathBuf; 4];

    /// Replaces the current process with `nvim` via POSIX execvp(3). Never returns on success.
    pub fn exec_nvim(&self, args: &[OsString]) -> Result<()>;

    /// Atomically replaces target binary, safely handling cross-device links (EXDEV).
    pub fn atomic_replace_exe(current_exe: &Path, temp_download: &Path) -> Result<()>;

    /// Scans profiles_dir returning sorted valid profile identifiers.
    pub fn list_profiles(&self) -> Result<Vec<ProfileId>>;

    /// Checks if a profile configuration directory exists on disk.
    pub fn profile_exists(&self, profile: &ProfileId) -> bool;
}
```

### 4.3. Configuration Parser Boundary (`src/config.rs`)

```rust
use std::path::Path;
use crate::types::Config;

impl Config {
    /// Zero-allocation line parser for `config.toml`. Fills defaults on syntax errors.
    pub fn load_from_str(content: &str, home: &Path) -> Self;
    pub fn load(system_config_dir: &Path) -> Self;
    pub fn default_config() -> Self;
}
```

### 4.4. External Tool Bridge Boundary (`src/net.rs`)

```rust
use std::path::Path;
use crate::types::{Config, SemVer};
use crate::error::Result;

/// Resolves repository source through 3-tier hierarchy:
/// 1. Built-in presets (0 ms, offline) -> 2. User config presets -> 3. GitHub slug (owner/repo).
pub fn resolve_source_url<'a>(raw_source: &'a str, cfg: &'a Config) -> &'a str;

/// Clones git repository via `gh repo clone` with fallback to `git clone --depth 1`.
pub fn clone_repository(repo_url: &str, dest: &Path) -> Result<()>;

/// Queries GitHub releases API via system curl and returns latest version and download URL.
pub fn fetch_latest_release() -> Result<(SemVer, String)>;

/// Downloads a binary payload directly to a local file destination using system curl.
pub fn download_file(url: &str, dest: &Path) -> Result<()>;

/// Parses SemVer string (e.g. "v0.10.4" or "2.48.1") into SemVer(u32, u32, u32).
pub fn parse_semver(raw: &str) -> Result<SemVer>;
```

### 4.5. Terminal Controller Boundary (`src/term.rs`)

```rust
use crate::types::ProfileId;
use crate::error::Result;

pub struct Terminal {
    is_tty: bool,
    color_enabled: bool,
}

impl Terminal {
    pub fn new() -> Self;
    pub fn cyan_arrow(&self, msg: &str);
    pub fn success(&self, msg: &str);
    pub fn error(&self, msg: &str);
    pub fn hint(&self, msg: &str);
    pub fn dimmed(&self, msg: &str);
    pub fn prompt_confirm(&self, prompt_text: &str) -> Result<bool>;
    pub fn erase_lines(&self, count: usize);
    pub fn select_profile_fzf(&self, profiles: &[ProfileId]) -> Result<Option<ProfileId>>;
}
```

---

## 5. Centralized Control Flow (`src/main.rs`)

```rust
fn main() -> ExitCode {
    let args: Vec<OsString> = std::env::args_os().collect();

    // 1. Zero-Cost Fast Path: -v / --version / help (0 syscalls, 0 FS reads)
    if let Some(cmd) = cli::try_fast_path(args.get(1).map(|s| s.as_os_str())) {
        return execute_fast_path(cmd);
    }

    // 2. Load Configuration & System Topology
    let bootstrap_config_dir = layout::resolve_bootstrap_config_dir();
    let config = config::Config::load(&bootstrap_config_dir);
    let term = term::Terminal::new();

    // 3. Parse Full Invocation (Subcommand or Hot-Path Launch)
    let command = match cli::parse_invocation(args, &config) {
        Ok(cmd) => cmd,
        Err(err) => {
            term.error(&err.to_string());
            return err.exit_code();
        }
    };

    // 4. Central Dispatch
    match command {
        Command::Launch(payload) => handle_launch(payload, &config, &term),
        Command::List { plain } => handle_list(plain, &config, &term),
        Command::Install { repo_or_preset, name } => {
            handle_install(&repo_or_preset, &name, &config, &term)
        }
        Command::Clean { profile, auto_confirm } => handle_clean(&profile, auto_confirm, &config, &term),
        Command::Doctor { json } => handle_doctor(json, &config, &term),
        Command::Setup(opts) => handle_setup(opts, &config, &term),
        Command::Sandbox { target } => handle_sandbox(&target, &config, &term),
        Command::Update => handle_update(&term),
        Command::Completions { shell } => handle_completions(shell),
        Command::ReservedAbort { name } => {
            term.error(&format!("'{name}' is a reserved command and cannot be used as a profile name"));
            ExitCode::FAILURE
        }
        Command::Version | Command::Help { .. } => unreachable!(),
    }
}
```

---

## 6. Build & Lint Configuration

### `Cargo.toml`
```toml
[package]
name = "nvimx"
version = "0.1.0"
edition = "2021"
rust-version = "1.75"

[dependencies]
# 100% Zero-Dependency (std-only target architecture)

[profile.release]
opt-level = 3
lto = "fat"
codegen-units = 1
panic = "abort"
strip = "symbols"

[lints.rust]
unsafe_code = "deny"

[lints.clippy]
all = { level = "warn", priority = -1 }
pedantic = { level = "warn", priority = -1 }
nursery = { level = "warn", priority = -1 }
unwrap_used = "warn"
expect_used = "warn"
undocumented_unsafe_blocks = "deny"
multiple_unsafe_ops_per_block = "deny"
needless_pass_by_value = "warn"
module_name_repetitions = "allow"
cast_possible_truncation = "allow"
cast_sign_loss = "allow"
cast_precision_loss = "allow"
```

### `Justfile`
```just
set shell := ["sh", "-cu"]

build:
    cargo build --release

test:
    cargo test --all-targets

lint:
    cargo clippy --all-targets -- -D warnings

e2e:
    tests/e2e/validate.sh
```

---

## 7. Step-by-Step Topological Build & Verification Plan

```
       Step 1             Step 2             Step 3             Step 4
   ┌────────────┐     ┌────────────┐     ┌────────────┐     ┌────────────┐
   │ types.rs   │ ──> │ layout.rs  │ ──> │ config.rs  │ ──> │ term.rs    │
   │ error.rs   │     │ (topology) │     │ (TOML)     │     │ (ANSI/TTY) │
   └────────────┘     └────────────┘     └────────────┘     └────────────┘
         │                                                        │
         ▼                                                        ▼
   ┌────────────┐     ┌────────────┐     ┌────────────┐     ┌────────────┐
   │ net.rs     │ ──> │ cli.rs     │ ──> │ main.rs    │ ──> │ E2E Suite  │
   │ (git/curl) │     │ (parser)   │     │ (dispatch) │     │ (validate) │
   └────────────┘     └────────────┘     └────────────┘     └────────────┘
       Step 5             Step 6             Step 7             Step 8
```

### Step 1: Scaffold, Lints, Flat Error Enum & Core Types
* **Files:** `Cargo.toml`, `Justfile`, `src/error.rs`, `src/types.rs`
* **Verification:** `cargo test --lib error types && cargo clippy --all-targets -- -D warnings`

### Step 2: Unified `EnvLayout` Resolver & Atomic FS
* **Files:** `src/layout.rs`
* **Verification:** Unit tests verifying XDG calculation, custom `profiles_dir` mapping, and atomic file replace.

### Step 3: Zero-Dependency TOML Config Parser
* **Files:** `src/config.rs`
* **Verification:** Unit tests for `load_from_str`, comments, `[presets]` table, missing keys, and invalid syntax defaults.

### Step 4: Terminal Controller & ANSI Engine
* **Files:** `src/term.rs`
* **Verification:** Unit tests verifying `is_terminal()` and `NO_COLOR` escape code suppression and line-erase sequences.

### Step 5: External Tool Bridge & Release JSON Parser
* **Files:** `src/net.rs`
* **Verification:** Unit tests for 3-tier `resolve_source_url` and `parse_semver`.

### Step 6: Zero-Copy CLI Argument Parser
* **Files:** `src/cli.rs`
* **Verification:** Unit tests covering fast-path bypass, subcommand flag parsing, and hot-path profile disambiguation.

### Step 7: Central Orchestration & Process Replacement
* **Files:** `src/main.rs`
* **Verification:** `cargo build --release` producing stripped binary $< 350$ KB.

### Step 8: E2E Compatibility Validation
* **Files:** `tests/e2e/validate.sh`
* **Verification:** `just e2e` passing all 6 tests in $\le 1.0$ ms cold overhead.
