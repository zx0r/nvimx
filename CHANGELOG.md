# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.1] 2026-04-30

### Added
- Integrated **crates.io publication** into the release workflow.
- Implemented **Trusted Publishing (OIDC)** framework for secure, secretless authentication with crates.io.

### Changed
- Optimized the `announce` job to wait for all parallel publication tasks (Homebrew, crates.io).

## [0.1.0] 2026-04-28

### Added
- Initial release of **nvimx** &mdash; a high&ndash;performance Neovim profile manager.
- Support for instant switching between multiple Neovim configurations.
- Automated installation and cleanup routines for configuration files.
- Full CI/CD integration via `cargo-dist` for cross&ndash;platform delivery.
- Automated Homebrew formula updates in `zx0r/homebrew-tap`.
- SLSA Level 3 provenance and CycloneDX SBOM generation for enhanced security.

