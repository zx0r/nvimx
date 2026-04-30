#  n v i m x  <img src="https://miro.medium.com/v2/format:webp/1*wL9FvRCwlO8X0ysJ8348kw.png" width="66" height="66" align="right">

[![Latest Release](https://img.shields.io/github/v/release/zx0r/nvimx?color=4be1ff&label=RELEASE&style=flat-square&logo=github&labelColor=1a1a1a)](https://github.com/zx0r/nvimx/releases) [![License: MIT](https://img.shields.io/badge/LICENSE-MIT-9fef00?style=flat-square&logo=opensourceinitiative&logoColor=white&labelColor=1a1a1a)](LICENSE) [![Rust](https://img.shields.io/badge/CORE-RUST-orange?style=flat-square&logo=rust&logoColor=white&labelColor=1a1a1a)](https://www.rust-lang.org/) [![Neovim](https://img.shields.io/badge/NVIM-0.10+-57A143?style=flat-square&logo=neovim&logoColor=white&labelColor=1a1a1a)](https://neovim.io/) [![XDG Compliance](https://img.shields.io/badge/XDG-COMPLIANT-00FF00?style=flat-square&logo=linux&logoColor=white&labelColor=1a1a1a)](https://specifications.freedesktop.org/basedir-spec/basedir-spec-latest.html) [![Hacker News](https://img.shields.io/badge/HACKER_NEWS-FF6600?style=flat-square&logo=y-combinator&logoColor=white&labelColor=1a1a1a)](https://news.ycombinator.com/) [![r/unixporn](https://img.shields.io/badge/R%2FUNIXPORN-RICE-black?style=flat-square&logo=reddit&logoColor=FF00FF&labelColor=1a1a1a&color=000)](https://reddit.com/r/unixporn)

#### Zero-overhead profile manager for Neovim

nvimx is a system CLI tool for managing multiple Neovim configurations (profiles).It imposes zero restrictions on native Neovim functionality.Profile management is implemented as a seamless overlay: configs, plugins, LSPs, DAPs, and CLI flags work exactly as if you called nvim directly.

---

#### Quick Start
```shell

# Get production&ndash;ready in seconds:
#
# 1. Install via Homebrew
brew install zx0r/tap/nvimx
# 
# 2. Run the automated setup
nvimx setup
#
# 3. Enable shell integration
nvimx setup shell
```

---

#### 0verview

```shell
# nvimx treats neovim configs as isolated environments
#
# xdg-scoped per profile:
# - config
# - data
# - state
# - cache
# 
# supports:
# - remote profiles
# - ephemeral execution
# - registry sources
# 
# no shared state
# no global mutation 
```

---

#### Design
```shell

# - XDG isolation (`CONFIG`, `DATA`, `STATE`, `CACHE`)
# - process-level separation
# - no shared state between profiles
# - deterministic execution
# - no implicit writes outside assigned scope
```

---

#### Features

```shell
# - profile install from git or registry
# - sandbox execution (ephemeral environments)
# - environment diagnostics (`doctor`)
# - registry inspection
# - shell completion
```

---

#### Usage

```shell
# ABOUT:
#  nvimx is a transparent profile manager and launcher for Neovim.
#  It imposes zero restrictions on native Neovim functionality.
#  Profile management is implemented as a seamless overlay: your plugins, configs,
#  LSPs, DAPs, and CLI flags work exactly as if you called `nvim` directly.
#
# USAGE:
#  nvimx [PROFILE] [FILE...] [-- NVIM_ARGS...]     # Launch Neovim with a profile
#  nvimx <COMMAND> [ARGS...]                       # Manage profiles & environment
#
# EXAMPLES:
#  nvimx                         # Launch with default profile
#  nvimx list                    # List available profiles  
#  nvimx lazyvim main.py         # Open main.py using the 'python' profile
#  nvimx -- -c 'echo "hello"'    # Pass arguments directly to nvim
```

##### Core Commands

```bash
Usage:
  nvimx [PROFILE] [FILES...][-- ARGS...]
  nvimx <COMMAND> [ARGS...]

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
  help         Show help information for nvimx or its subcommands

Arguments:
  [PROFILE]     Profile to run (defaults to configured profile)
  [FILES...]    Files to open
  [ARGS...]     Arguments passed to Neovim (after `--`)

```


---

#### Sandbox Mode

```shell
# The `sandbox` command executes a profile in a temporary environment
#
# $ nvimx sandbox lazyvim
#
# - **XDG Overrides**: Forcibly separates `XDG_CONFIG_HOME`, `DATA`, `STATE`, and `CACHE`.
# - **Volatile State**: Wipes previous sandbox state before execution.
# - **Environment Context**: Exports `NVIMX_SANDBOX` for use in configuration logic.
```

---

#### Configuration

```shell
# strong defaults. zero-config.
# optional config: ~/.config/nvimx/config.toml
#
# default profile if none specified
# → Used when no profile argument is provided (skips fzf)
# default_profile = "lazyvim"
#
# registry source
# → Remote JSON registry for profile installation
# registry_url = "https://raw.githubusercontent.com/zx0r/nvimx-registry/main/index.json"
```

---

#### License

```shell
# Distributed under the MIT License. See `LICENSE` for more information
```

---

```shell
# $ nvimx lazyvim --headless -c "lua print('you know gentoo → you know linux')" +qa

# [✓] zero-overhead exec
# [✓] argv passthrough
# [✓] isolated config
# [ ] anything else

# you bring args  
# we don't touch them  

# you choose a profile  
# we don't leak it  

# nothing else.. happy coding.. </>
```


##### [ n v i m x ] <img src="https://miro.medium.com/v2/format:webp/1*wL9FvRCwlO8X0ysJ8348kw.png" width="66" height="66" align="right">

