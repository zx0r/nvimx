# nvimx User Guide

This guide provides technical details on the architecture, configuration, and operation of **nvimx**.

---

## 1. Core Architecture

`nvimx` operates by manipulating environment variables to isolate Neovim sessions. It adheres strictly to the XDG Base Directory Specification.

### Environment Isolation
When a profile is selected, `nvimx` overrides the following environment variables before launching the `nvim` binary:

| Variable | Target Directory | Purpose |
| :--- | :--- | :--- |
| `XDG_CONFIG_HOME` | `~/.config/nvim/profiles/<name>` | Configuration files (`init.lua`, etc) |
| `XDG_DATA_HOME` | `~/.local/share/nvim/profiles/<name>` | Plugins, undo files, and shada |
| `XDG_STATE_HOME` | `~/.local/state/nvim/profiles/<name>` | Session state and logs |
| `XDG_CACHE_HOME` | `~/.cache/nvim/profiles/<name>` | Bytecode and plugin cache |

---

## 2. Profile Management

### Installation Methods
1.  **Direct Git:** `nvimx install user/repo` clones a configuration directly into a profile named after the repository
2.  **Registry-based:** `nvimx install <alias>` resolves the alias via configured JSON registries

### Lifecycle Commands
- `list`: Displays all installed profiles and the currently active default
- `clean <name>`: Removes all XDG directories associated with a specific profile
- `doctor`: Audits system dependencies (nvim, git, fzf) and verifies filesystem permissions

---

## 3. Sandbox Mode

The `sandbox` command executes a profile in a volatile environment. This is intended for testing configurations or debugging without modifying persistent data

```bash
nvimx sandbox <profile>
```

**Technical Implementation:**
- **Volatile Storage:** All XDG paths are redirected to `~/.cache/nvimx/sandbox/`
- **Pre-execution:** The existing sandbox directory for the profile is wiped
- **Context:** The `NVIMX_SANDBOX` variable is exported, containing the profile name
- **Visual Feedback:** The terminal title is updated to reflect the sandbox state

---

## 4. Registry System

`nvimx` supports decentralized profile discovery through JSON registries

### Configuration
Registries are defined in `~/.config/nvimx/config.toml`
- **Caching:** Remote registry metadata is cached locally to ensure high performance
- **Validation:** All registry responses are validated against an internal JSON schema before being committed to the local cache

### Commands
- `nvimx registry list`: Show all configured sources
- `nvimx registry validate`: Check the integrity of the local metadata

---

## 5. Shell Integration

For optimal performance, `nvimx` provides dynamic shell completions and a smart wrapper

### Setup
```bash
nvimx setup shell --override-nvim
```
This command adds an alias or function to your shell configuration that allows `nvimx` to handle standard Neovim flags while maintaining profile isolation

### Supported Shells
- **Fish:** Native completions in `~/.config/fish/completions/`
- **Zsh:** Generated functions via `compdef`
- **Bash:** Standard completion scripts

---

## 6. Troubleshooting

Use the `doctor` command as the first step for any issues. It provides a detailed report on:
- Binary availability and versions
- XDG path accessibility
- Registry connectivity and cache state
