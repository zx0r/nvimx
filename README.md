# n v i m x

[![Latest Release](https://img.shields.io/github/v/release/zx0r/nvimx?color=4be1ff&label=RELEASE&style=flat-square&logo=github&labelColor=1a1a1a)](https://github.com/zx0r/nvimx/releases) [![License: MIT](https://img.shields.io/badge/LICENSE-MIT-9fef00?style=flat-square&logo=opensourceinitiative&logoColor=white&labelColor=1a1a1a)](LICENSE) [![Rust](https://img.shields.io/badge/CORE-RUST-orange?style=flat-square&logo=rust&logoColor=white&labelColor=1a1a1a)](https://www.rust-lang.org/) [![Neovim](https://img.shields.io/badge/NVIM-0.10+-57A143?style=flat-square&logo=neovim&logoColor=white&labelColor=1a1a1a)](https://neovim.io/) [![XDG Compliance](https://img.shields.io/badge/XDG-COMPLIANT-00FF00?style=flat-square&logo=linux&logoColor=white&labelColor=1a1a1a)](https://specifications.freedesktop.org/basedir-spec/basedir-spec-latest.html) [![Hacker News](https://img.shields.io/badge/HACKER_NEWS-FF6600?style=flat-square&logo=y-combinator&logoColor=white&labelColor=1a1a1a)](https://news.ycombinator.com/) [![r/unixporn](https://img.shields.io/badge/R%2FUNIXPORN-black?style=flat-square&logo=reddit&logoColor=FF00FF&labelColor=1a1a1a&color=000)](https://reddit.com/r/unixporn)

#### Zero-overhead profile manager for Neovim

> Install, manage and run Neovim configurations as fully isolated environments: reproducible, conflict-free, and without touching your existing setup.nvimx treats each config as a self-contained workspace: no shared state, no global mutations, and no behavioral differences from native nvim.

---

### Quick Start

#### 1. Installation

**macOS / Linux (Homebrew)**

```bash
brew install zx0r/tap/nvimx
```


**Rust Toolchain (Cargo)**

```bash
cargo install nvimx
```

#### 2. Initialization

```bash
nvimx setup
```

#### 3. Shell Integration (Recommended)

```bash
nvimx setup shell
```

---

### 0verview

> nvimx is a CLI tool that manages Neovim profiles as fully isolated environments.

**It works as a transparent layer over Neovim:**

* no wrappers, patches, or feature limitations
* everything runs as if you launched nvim directly
* configs, plugins, LSPs, DAPs, and CLI flags behave exactly as usual

Each profile is completely independent, enabling reproducible and conflict-free setups

### Features

* profile install from Git or registry
* sandbox execution (ephemeral environments)
* environment diagnostics (`doctor`) 
* registry inspection
* shell completion

```bash
# Install from Git (available)
nvimx install https://github.com/LazyVim/starter lazyvim

# Install from registry (planned)
nvimx install lazyvim

# Set default profile (planned)
nvimx use lazyvim
```

---

### Usage

```bash
# Launch Neovim with a profile
nvimx [PROFILE] [FILES...][-- ARGS...] 

# Manage profiles & environment
nvimx <COMMAND> [ARGS...] 

Arguments:
  [PROFILE]                      # Profile to run (defaults to configured profile)
  [FILES...]                     # Files to open
  [ARGS...]                      # Arguments passed to Neovim (after `--`)

EXAMPLES:
   nvimx                         # Launch with default profile
   nvimx list                    # List available profiles  
   nvimx lazyvim main.py         # Open main.py using the 'python' profile
   nvimx -- -c 'echo "hello"'    # Pass arguments directly to nvim
```

#### Core Commands

```bash
Commands:
  list         List all locally installed profiles
  install      Install a new profile from a git repository or registry
  clean        Safely remove data, state, and cache for a profile
  doctor       Check system dependencies and environment health
  setup        Initialize environment, config, and install initial profiles
  sandbox      Run a profile in an isolated environment
  registry     Inspect and manage configured profile registries
  update       Self-update the nvimx binary to the latest version
  completions  Generate auto-completion scripts for your shell
```

---

#### Configuration

* strong defaults. zero-config
* If no profile is specified, an interactive picker (fzf) is used automatically

**Optional configuration file:**

```bash
cp config.example.toml ~/.config/nvimx/config.toml
```

```toml
# Default profile (skips fzf when set)
default_profile = "lazyvim"

# Remote registry for profile installation
registry_url = "https://raw.githubusercontent.com/zx0r/nvimx-registry/main/index.json"
```

#### Sandbox Mode

* **Volatile State**: Wipes previous sandbox state before execution
* **Environment Context**: Exports `NVIMX_SANDBOX` for use in configuration logic
* **XDG Overrides**: Forcibly separates `XDG_CONFIG_HOME`, `DATA`, `STATE`, and `CACHE`


```shell
# The `sandbox` command executes a profile in a temporary environment
nvimx sandbox lazyvim
```

---

##### [ n v i m x ] 

<p align="center">
  <img src="./assets/img/rust.gif" width="480" height="480" alt="nvimx demo" />
</p>


```shell
nvimx lazyvim --headless -c "lua print(vim.fn.stdpath('config'))" +qa

[✓] zero-overhead
[✓] isolated environment
[✓] argv passthrough

you bring args → we don't touch them  
you choose a profile → we don't leak state

that's it.
```