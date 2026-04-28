#  n v i m x  <img src="https://miro.medium.com/v2/format:webp/1*wL9FvRCwlO8X0ysJ8348kw.png" width="66" height="66" align="right">

[![Latest Release](https://img.shields.io/github/v/release/zx0r/nvimx?color=4be1ff&label=RELEASE&style=flat-square&logo=github&labelColor=1a1a1a)](https://github.com/zx0r/nvimx/releases) [![License: MIT](https://img.shields.io/badge/LICENSE-MIT-9fef00?style=flat-square&logo=opensourceinitiative&logoColor=white&labelColor=1a1a1a)](LICENSE) [![Rust](https://img.shields.io/badge/CORE-RUST-orange?style=flat-square&logo=rust&logoColor=white&labelColor=1a1a1a)](https://www.rust-lang.org/) [![Neovim](https://img.shields.io/badge/NVIM-0.10+-57A143?style=flat-square&logo=neovim&logoColor=white&labelColor=1a1a1a)](https://neovim.io/) [![XDG Compliance](https://img.shields.io/badge/XDG-COMPLIANT-00FF00?style=flat-square&logo=linux&logoColor=white&labelColor=1a1a1a)](https://specifications.freedesktop.org/basedir-spec/basedir-spec-latest.html) [![Hacker News](https://img.shields.io/badge/HACKER_NEWS-FF6600?style=flat-square&logo=y-combinator&logoColor=white&labelColor=1a1a1a)](https://news.ycombinator.com/) [![r/unixporn](https://img.shields.io/badge/R%2FUNIXPORN-RICE-black?style=flat-square&logo=reddit&logoColor=FF00FF&labelColor=1a1a1a&color=000)](https://reddit.com/r/unixporn)

#### Zero-overhead profile manager and launcher for Neovim
```shell
# Isolates runtime environments using XDG overrides (config, data, state, cache) with zero restrictions and no cross-contamination
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

#### Quick Start
```shell

# Get production&ndash;ready in seconds:
#
# 1. Install via Homebrew
# $ brew install zx0r/tap/nvimx
# 
# 2. Run the automated setup
# $ nvimx setup
#
# 3. Enable shell integration
# $ nvimx setup shell
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
# Usage:
#  nvimx [profile] [args...]
#  nvimx [command]
#
# Commands:
#  list         List available profiles
#  install      Install a profile from a repository or registry
#  clean        Remove profile data (config, cache, state)
#  doctor       Diagnose environment and dependencies
#  setup        Initialize environment and onboarding
#  sandbox      Run a profile in an isolated environment
#  registry     Inspect and validate configured registries
#  update       Update nvimx to the latest version
#  completions  Generate shell completion scripts
```

---

#### Sandbox Mode

```shell
# The `sandbox` command executes a profile in a temporary environment, ensuring no persistent changes are made to the profile's data:
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
# nvimx features &ldquo;Strong Defaults&rdquo; and works zero&ndash;config out of the box. Customize it via `~/.config/nvimx/config.toml`:
#
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
# - **Built with ❤️ for the Neovim Community**
[docs](docs/GUIDE.md) | [report](https://github.com/zx0r/nvimx/issues) 
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

# nothing else.. happy hacking.. </>
```


##### [ n v i m x ] <img src="https://miro.medium.com/v2/format:webp/1*wL9FvRCwlO8X0ysJ8348kw.png" width="66" height="66" align="right">

