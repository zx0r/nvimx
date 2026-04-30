# 📋 cargo-dist Bug Fix & PR Implementation Plan

## 1. Context & Objectives
During the deployment of the `nvimx` project, two critical issues were identified in the `cargo-dist` (v0.31.0) release flow. This plan outlines the technical rationale and implementation steps to fix these issues in the core `cargo-dist` engine to benefit the wider Rust ecosystem.

### Identified Bugs:
1. **[Bug A] Missing `dist` Profile**: CI fails with `error: profile 'dist' is not defined` because `dist init` may skip creating the profile if a `[profile]` table already exists but lacks a `dist` entry.
2. **[Bug B] Homebrew Checksum Mismatch**: In multi-stage GitHub Actions, the global artifact job generates a Homebrew formula with incorrect or placeholder SHA-256 hashes because it doesn't correctly propagate hashes from local build jobs.

---

## 2. Technical Analysis & Hypotheses

### Bug A: Profile Initialization
*   **Location**: `cargo-dist/src/init.rs` -> `fn init_dist_profile`.
*   **Hypothesis**: The current logic checks for the existence of the `PROFILE_DIST` key but might fail to write or correctly identify "None" states when the `[profile]` table is partially populated.
*   **Fix**: Update the logic to be "idempotent-enforcing" — ensuring that if `dist` isn't there, it's added with `inherits = "release"` and `lto = "thin"` regardless of other table contents.

### Bug B: Homebrew Checksums
*   **Location**: `cargo-dist/src/backend/installer/homebrew.rs` and `cargo-dist/templates/ci/github/release.yml.j2`.
*   **Hypothesis**: When `dist build --artifacts=global` runs, it creates a new manifest. If it doesn't have access to the *content* of the local artifacts (only their metadata), it might use stale or missing checksums.
*   **Fix**: 
    1.  Ensure `release.yml.j2` uses `merge-multiple: true` for `download-artifact@v7` (modern standard).
    2.  Modify the Homebrew generation logic to either verify hashes against local files if present in `target/distrib` or ensure the manifest is correctly "hydrated" with real hashes from the build-local stage.

---

## 3. Implementation Steps (Execution)

### Phase 1: Fixing `init.rs` (Cargo.toml logic)
*   **Action**: Modify `src/init.rs` to guarantee profile creation.
*   **Verification**: Run `cargo run -- dist init` on a test repo.

### Phase 2: Fixing Homebrew logic
*   **Action**: 
    *   Update `templates/ci/github/release.yml.j2` to ensure artifacts are correctly fetched.
    *   Modify `src/backend/installer/homebrew.rs` to improve hash retrieval accuracy.
*   **Verification**: Run `cargo dist plan` and check generated YAML.

---

## 4. Pull Request Strategy

### PR Title:
`fix: robust [profile.dist] initialization and accurate Homebrew checksum propagation`

### Key Highlights for Reviewers:
*   **Reliability**: Fixes a "silent failure" where `dist init` claims success but leaves the project unbuildable in CI.
*   **CI Stability**: Solves the common `checksum mismatch` error for Homebrew users by ensuring hashes are calculated from finalized artifacts.
*   **Modernization**: Updates GitHub Actions templates to use modern `upload-artifact` / `download-artifact` patterns.

---

## 5. Notes for Future Sessions
If a new agent takes over this task:
1.  Verify the changes in `~/x/dev/cargo-dist` using `cargo check`.
2.  Ensure `dist-workspace.toml` in the target project doesn't override these fixes with `allow-dirty`.
3.  The goal is to upstream these changes so that `allow-dirty = ["ci"]` is no longer a mandatory workaround for Homebrew fixes.
