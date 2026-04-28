# NvimX <img src="https://miro.medium.com/v2/format:webp/1*wL9FvRCwlO8X0ysJ8348kw.png" width="66" height="66" align="right">

[![Latest Release](https://img.shields.io/github/v/release/zx0r/nvimx?color=4be1ff&label=RELEASE&style=flat-square&logo=github&labelColor=1a1a1a)](https://github.com/zx0r/nvimx/releases) [![License: MIT](https://img.shields.io/badge/LICENSE-MIT-9fef00?style=flat-square&logo=opensourceinitiative&logoColor=white&labelColor=1a1a1a)](LICENSE) [![Rust](https://img.shields.io/badge/CORE-RUST-orange?style=flat-square&logo=rust&logoColor=white&labelColor=1a1a1a)](https://www.rust-lang.org/) [![Neovim](https://img.shields.io/badge/NVIM-0.10+-57A143?style=flat-square&logo=neovim&logoColor=white&labelColor=1a1a1a)](https://neovim.io/) [![XDG Compliance](https://img.shields.io/badge/XDG-COMPLIANT-00FF00?style=flat-square&logo=linux&logoColor=white&labelColor=1a1a1a)](https://specifications.freedesktop.org/basedir-spec/basedir-spec-latest.html) [![Hacker News](https://img.shields.io/badge/HACKER_NEWS-FF6600?style=flat-square&logo=y-combinator&logoColor=white&labelColor=1a1a1a)](https://news.ycombinator.com/) [![r/unixporn](https://img.shields.io/badge/R%2FUNIXPORN-RICE-black?style=flat-square&logo=reddit&logoColor=FF00FF&labelColor=1a1a1a&color=000)](https://reddit.com/r/unixporn)
### Neovim profile manager built with Rust

#### Isolates runtime environments using XDG overrides. Each profile gets its own config, data, state, and cache. No cross-contamination.

---

### Overview

Provides a complete lifecycle for Neovim environment management:

- **Profile Isolation**: Strict separation of `config`, `data`, `state`, and `cache` directories using XDG standard overrides.
- **Automated Installation**: Resolve and install profiles from remote repositories or JSON registries.
- **Isolated Execution**: `sandbox` mode for running configurations in volatile environments without persisting state.
- **Environment Diagnostics**: `doctor` command for auditing system dependencies and configuration integrity.
- **Registry Management**: Tools to inspect, validate, and manage multiple profile sources.
- **Shell Integration**: Dynamic TAB-completion with real-time discovery of installed profiles.

Profiles are treated as disposable environments. No global mutation.

---

### Design
- XDG isolation (`CONFIG`, `DATA`, `STATE`, `CACHE`)
- process-level separation
- no shared state between profiles
- deterministic execution
- no implicit writes outside assigned scope

---

### Features
- profile install from git or registry
- sandbox execution (ephemeral environments)
- environment diagnostics (`doctor`)
- registry inspection
- shell completion

---

### Quick Start

Get production&ndash;ready in seconds:

```bash
# 1. Install via Homebrew
brew install zx0r/tap/nvimx

# 2. Run the automated setup
nvimx setup

# 3. Enable shell integration (optional but recommended)
nvimx setup shell --override-nvim
```

---

### Usage

```text
Usage:
  nvimx [profile] [args...]  # Launch Neovim
  nvimx <command>            # Manage environment
```

#### Core Commands

```bash
nvimx — Neovim profile manager

A tool for managing, installing, and running multiple Neovim configurations.

Usage:
  nvimx <profile> [args...]
  nvimx <command>

Commands:
  list         List available profiles
  install      Install a profile from a repository or registry
  clean        Remove profile data (config, cache, state)
  doctor       Diagnose environment and dependencies
  setup        Initialize environment and onboarding
  sandbox      Run a profile in an isolated environment
  registry     Inspect and validate configured registries
  update       Update nvimx to the latest version
  completions  Generate shell completion scripts

Arguments:
  [PROFILE]    Profile to run (defaults to configured profile)
  [ARGS]...    Arguments passed to Neovim

Options:
  -h, --help     Show help
  -V, --version  Show version
```

---

### Sandbox Mode

The `sandbox` command executes a profile in a temporary environment, ensuring no persistent changes are made to the profile's data:

```bash
nvimx sandbox lazyvim
```
- **XDG Overrides**: Forcibly separates `XDG_CONFIG_HOME`, `DATA`, `STATE`, and `CACHE`.
- **Volatile State**: Wipes previous sandbox state before execution.
- **Environment Context**: Exports `NVIMX_SANDBOX` for use in configuration logic.

---

### Configuration

`nvimx` features &ldquo;Strong Defaults&rdquo; and works zero&ndash;config out of the box. Customize it via `~/.config/nvimx/config.toml`:

```toml
# default profile if none specified
# → Used when no profile argument is provided (skips fzf)
default_profile = "lazyvim"

# registry source
# → Remote JSON registry for profile installation
registry_url = "https://raw.githubusercontent.com/zx0r/nvimx-registry/main/index.json"

# enable fzf picker
# → Enables interactive selection when no default_profile
use_fzf = true

# logging
# → Controls verbosity level of CLI output
log_level = "info"   # debug | info | warn | error

# output format
# → "json" for scripting, "text" for human-readable CLI
output = "text"      # text | json

# auto update
# → Automatically update nvimx binary (not recommended by default)
auto_update = false

# fallback behavior
# → If profile not found → fallback to plain nvim instead of error
fallback_to_plain_nvim = true

# custom profiles dir (advanced)
# → Override default XDG config path for profiles
profiles_dir = "~/.config/nvim"

strict_mode = false
```


### License

Distributed under the MIT License. See `LICENSE` for more information

---

- **Built with ❤️ for the Neovim Community**
- [docs](docs/GUIDE.md) | [report](https://github.com/zx0r/nvimx/issues)
