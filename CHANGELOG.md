# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.7] - 2026-11-13

### Changed
- **Compiler Toolchain**: Upgraded hermetic compiler toolchain and MSRV boundary to Rust 1.99.0 across `rust-toolchain.toml`, `Cargo.toml`, `clippy.toml`, and CI workflows.
- **Terminal & I/O Architecture**: Refactored console I/O into `Terminal` struct utilizing locked streams (`StdoutLock`/`StderrLock`) and direct byte writes (`write_raw` via `include_bytes!`); eliminated raw `println!` / `eprintln!` calls to avoid broken pipe panics and eliminate mutex re-locking overhead.
- **Clippy Policy Hardening**: Enforced strict Clippy profile (`print_stdout = "deny"`, `print_stderr = "deny"`, `allow_attributes_without_reason = "deny"`), documented explicit reasons across all test modules, and configured `allow-print-in-tests = true`.

## [0.1.6] - 2026-11-05

### Changed
- **Unix SRP Architecture**: Removed `install` subcommand and all Git/network dependencies (`git`, `gh`, `curl`) adhering strictly to the Unix philosophy and Single Responsibility Principle.
- **Offline Ephemeral Sandbox**: `nvimx sandbox` now clones local profiles directly via standard filesystem operations with zero external processes or network access.
- **Dependency Hygiene**: Stripped network/preset fields (`auto_install_missing`, `[presets]`) from configuration parser and example config.
- **CI/CD Quality Gates**: Aligned Miri test suite with Tree Borrows and strict provenance model; cleaned up obsolete configuration files.

### Fixed
- Fixed Linux target linking configuration to use LLVM lld linker.
- Removed outdated badges from README.

## [0.1.5] - 2026-10-31

### Changed
- Updated GitHub Actions workflows and pinned action versions.

## [0.1.4] - 2026-10-30

### Added
- Native shell completions for Bash, Zsh, and Fish (`nvimx init <shell>`).
- End-to-end test suite for Neovim process execution and environment isolation.

### Changed
- Upgraded to Rust 2024 edition.
- Hardened CI quality gates and dependency security policies.

## [0.1.3] - 2026-10-28

### Changed
- Upgraded and pinned dependencies in `Cargo.toml`.
- Enhanced release profile configuration for optimized builds.

### Performance
- Eliminated redundant startup I/O, optimized CLI flag passthrough, and hardened profile path safety.

## [0.1.2] - 2026-10-28

### Changed
- Optimized release profile configuration with LTO and binary size reductions.
- Hardened CI release workflow.

## [0.1.1] - 2026-04-30

### Added
- Integrated **crates.io publication** into the release workflow.
- Implemented **Trusted Publishing (OIDC)** framework for secure, secretless authentication with crates.io.

### Changed
- Optimized the `announce` job to wait for all parallel publication tasks (Homebrew, crates.io).

## [0.1.0] - 2026-04-28

### Added
- Initial release of **nvimx** &mdash; a high&ndash;performance Neovim profile manager.
- Support for instant switching between multiple Neovim configurations.
- Automated installation and cleanup routines for configuration files.
- Full CI/CD integration via `cargo-dist` for cross&ndash;platform delivery.
- Automated Homebrew formula updates in `zx0r/homebrew-tap`.
- SLSA Level 3 provenance and CycloneDX SBOM generation for enhanced security.

[0.1.7]: https://github.com/zx0r/nvimx/compare/v0.1.6...v0.1.7
[0.1.6]: https://github.com/zx0r/nvimx/compare/v0.1.5...v0.1.6
[0.1.5]: https://github.com/zx0r/nvimx/compare/v0.1.4...v0.1.5
[0.1.4]: https://github.com/zx0r/nvimx/compare/v0.1.3...v0.1.4
[0.1.3]: https://github.com/zx0r/nvimx/compare/v0.1.2...v0.1.3
[0.1.2]: https://github.com/zx0r/nvimx/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/zx0r/nvimx/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/zx0r/nvimx/releases/tag/v0.1.0


