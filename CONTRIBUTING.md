# Contributing to nvimx

Thank you for considering to contribute to **nvimx**! This project thrives on community involvement, and we appreciate your time and effort.

## Add an entry to the changelog

Keeping the `CHANGELOG.md` file up-to-date makes the release process much easier and helps get your changes into a new **nvimx** release faster. However, not every change requires a changelog entry.

**Please update the changelog if your contribution contains changes regarding any of the following:**
- The behavior of **nvimx** (new commands, flags, or logic).
- Profile management logic or registry handling.
- The build system, linting, or CI workflows.
- Performance improvements or bug fixes.

**A changelog entry is not necessary when:**
- Updating documentation or examples.
- Fixing typos or formatting.

> [!NOTE]
> For Pull Requests, a CI workflow verifies that a suitable changelog entry is added. If your changes do not need an entry, the workflow failure can be disregarded during review.

### Changelog entry format
The top of the `CHANGELOG.md` contains an `[Unreleased]` section. Please add your entry to the subsection that best describes your change (Features, Bug Fixes, Performance, etc.).

Entries must follow this format:
```markdown
- Short description of what has been changed, see #123 (@user)
```
Replace `#123` with your pull request number and `@user` with your GitHub username.

## Development

To get started with development, ensure you have a modern Rust toolchain installed (edition 2024 is used).

### Adding a new feature
Please consider opening a **Feature Request** issue first. This gives us a chance to discuss the specifics and technical direction before you invest significant time in building it.

### Style and Standards
- We follow standard Rust idioms and the project's own architecture (see `brain/` for deep technical context).
- Run `cargo fmt --all -- --check` before submitting.
- Run `cargo clippy --all-targets -- -D warnings` to ensure code quality.

## Regression tests

You are strongly encouraged to add regression tests. They ensure that your contribution works as expected and stays working in the future.

### Integration Tests
For functional changes, add a test to `tests/integration_test.rs`. Look at existing tests to see how to invoke the `nvimx` binary and assert on its output or side effects.

### Running Tests
To run tests locally, you can use:
```bash
cargo test
```

If you want to run tests against the debug binary specifically, ensure it's in your `PATH`:
```bash
cargo build
export PATH="$PATH:$(pwd)/target/debug"
cargo test
```

## Community and Communication
If you have questions, feel free to reach out via GitHub Issues or join our discussions. We aim to keep **nvimx** the fastest and most transparent profile manager for Neovim.
