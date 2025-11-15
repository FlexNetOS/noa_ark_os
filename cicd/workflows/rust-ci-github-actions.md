# Rust CI/CD with GitHub Actions

**Source**: WSL noa_ark_os `.github/workflows/ci.yml`  
**Integration**: Cycle 5 - NOA Ark OS Selective Drop  
**Date**: 2025-01-XX

---

## Overview

This document captures GitHub Actions patterns for Rust CI/CD, extracted from the WSL noa_ark_os repository's production workflow. These patterns demonstrate multi-workspace Rust builds with caching optimization.

---

## Original Workflow Structure

### Jobs

#### 1. **rust-check** - Rust Services Validation
```yaml
name: Rust check (services/ark-ai-os-workspace)
runs-on: ubuntu-latest
defaults:
  run:
    working-directory: services/ark-ai-os-workspace

steps:
  - name: Checkout
    uses: actions/checkout@v4

  - name: Install Rust (stable)
    uses: dtolnay/rust-toolchain@stable

  - name: Cache cargo
    uses: Swatinem/rust-cache@v2
    with:
      workspaces: |
        services/ark-ai-os-workspace

  - name: Cargo check (quiet)
    run: |
      cargo check --all-targets --quiet
```

**Key Features**:
- Workspace-specific working directory
- Rust stable toolchain via dtolnay action
- Swatinem/rust-cache for dependency caching
- `cargo check --all-targets --quiet` for validation

---

#### 2. **mono-metadata** - Metadata Generation
```yaml
name: Rust metadata (mono workspace)
runs-on: ubuntu-latest
defaults:
  run:
    working-directory: mono

steps:
  - name: Checkout
    uses: actions/checkout@v4

  - name: Install Rust (stable)
    uses: dtolnay/rust-toolchain@stable

  - name: Cargo metadata
    run: |
      cargo metadata --no-deps --format-version 1 > target/metadata.json
      test -s target/metadata.json
```

**Key Features**:
- Separate mono workspace handling
- Metadata generation for dependency analysis
- Validation that metadata.json is non-empty

---

#### 3. **node-env** - Node.js Environment Sanity Check
```yaml
name: Node environment sanity
runs-on: ubuntu-latest

steps:
  - name: Checkout
    uses: actions/checkout@v4

  - name: Setup Node.js
    uses: actions/setup-node@v4
    with:
      node-version: '18.x'
      cache: 'npm'

  - name: Print versions
    run: |
      node --version
      npm --version

  - name: List package manifests (apps subset)
    run: |
      find apps -maxdepth 3 -name package.json | head -n 20 | sed 's/^/ - /' || true
```

**Key Features**:
- Node.js 18.x with npm cache
- Version verification
- Package manifest discovery (apps/ directory)

---

## Adaptation for NoaArkOS

### Current State
NoaArkOS uses:
- Portable Cargo toolchain: `server/tools/cargo-portable/`
- Local Rust development (fixed rust-analyzer to use "check")
- Manual build processes

### Integration Opportunities

#### 1. Workspace Configuration
**Pattern**: Multiple workspace support
```yaml
workspaces: |
  core
  agents
  workflow
  sandbox
  cicd
```

**Benefit**: Cache each workspace separately for faster CI

---

#### 2. Rust Stable Toolchain
**Current**: Portable Cargo (manual management)  
**Enhancement**: Use `dtolnay/rust-toolchain@stable` for CI consistency

**Rationale**: 
- Automatic toolchain updates
- Consistent with GitHub-hosted runners
- Reduces manual toolchain management

---

#### 3. Cargo Check Strategy
**Pattern**: `cargo check --all-targets --quiet`

**Benefits**:
- Fast validation (no codegen)
- Checks all targets (bins, tests, benches, examples)
- Quiet mode reduces log noise

**Application**:
```bash
# Current: Manual check
cd D:\dev\workspaces\noa_ark_os
cargo check

# Enhanced: Comprehensive validation
cargo check --all-targets --workspace --quiet
```

---

#### 4. Quarantine Guard & Rotation (New)

**Objective**: Block builds that reference `archive/quarantine/` bundles and relocate aged bundles on a 90-day cadence.

**Implementation**:
- Guard execution: `cargo run -p quarantine_guard --bin quarantine_guard -- $(git diff --name-only origin/main...)`
- Scheduled rotation: `cargo run -p quarantine_guard --bin quarantine_rotate`
- Workflow excerpt:

```yaml
jobs:
  quarantine-guard:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - name: Quarantine guard
        run: |
          cargo run -p quarantine_guard --bin quarantine_guard -- $(git diff --name-only origin/main...)

  quarantine-rotation:
    if: github.event_name == 'schedule'
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - name: Rotate quarantine bundles
        run: cargo run -p quarantine_guard --bin quarantine_rotate
```

**Notes**:
- Guard can be bypassed line-by-line with `QUARANTINE_GUARD_ALLOW` for documentation snippets only.
- Rotation workflow appends to `archive/YYYY/MM/ledger.json` and emits `.tar.zst` bundles under `archive/YYYY/MM/quarantine/` for audit.

---

#### 4. Metadata Generation
**Pattern**: `cargo metadata --no-deps --format-version 1`

**Use Cases**:
- Dependency analysis
- Workspace structure validation
- CI/CD pipeline metadata

**Application**:
```bash
cargo metadata --no-deps --format-version 1 > cicd/metadata.json
test -s cicd/metadata.json
```

---

## Rust Cache Optimization

### Swatinem/rust-cache Patterns

**Basic Configuration**:
```yaml
- name: Cache cargo
  uses: Swatinem/rust-cache@v2
```

**Multi-Workspace Configuration**:
```yaml
- name: Cache cargo
  uses: Swatinem/rust-cache@v2
  with:
    workspaces: |
      core
      agents
      workflow
      sandbox
      cicd
```

**Benefits**:
- Caches `~/.cargo/registry/` (crates.io packages)
- Caches `~/.cargo/git/` (git dependencies)
- Caches `target/` (build artifacts)
- Automatically invalidates on `Cargo.lock` changes

---

## GitHub Actions Workflow Template (NoaArkOS)

```yaml
name: CI

on:
  push:
    branches: [ master, main ]
  pull_request:

jobs:
  rust-check:
    name: Rust check (all workspaces)
    runs-on: ubuntu-latest
    steps:
      - name: Checkout
        uses: actions/checkout@v4

      - name: Install Rust (stable)
        uses: dtolnay/rust-toolchain@stable

      - name: Cache cargo
        uses: Swatinem/rust-cache@v2
        with:
          workspaces: |
            core
            agents
            workflow
            sandbox
            cicd

      - name: Cargo check (quiet)
        run: |
          cargo check --all-targets --workspace --quiet

  metadata:
    name: Workspace metadata
    runs-on: ubuntu-latest
    steps:
      - name: Checkout
        uses: actions/checkout@v4

      - name: Install Rust (stable)
        uses: dtolnay/rust-toolchain@stable

      - name: Generate metadata
        run: |
          cargo metadata --no-deps --format-version 1 > target/metadata.json
          test -s target/metadata.json

      - name: Upload metadata
        uses: actions/upload-artifact@v4
        with:
          name: workspace-metadata
          path: target/metadata.json
```

---

## Local CI Simulation

### Using Portable Cargo

```bash
# Set portable cargo environment
$env:CARGO_HOME = "D:\dev\workspaces\noa_ark_os\server\tools\cargo-portable"
$env:RUSTUP_HOME = "D:\dev\workspaces\noa_ark_os\server\tools\rustup-portable"
$env:PATH = "D:\dev\workspaces\noa_ark_os\server\tools\cargo-portable\bin;$env:PATH"

# Run CI-equivalent checks
cd D:\dev\workspaces\noa_ark_os

# 1. Comprehensive check
cargo check --all-targets --workspace --quiet

# 2. Generate metadata
cargo metadata --no-deps --format-version 1 > cicd\metadata.json

# 3. Verify metadata
Test-Path cicd\metadata.json
```

---

## Integration with CRC System

### Pre-Drop Validation
```yaml
pre_drop:
  - name: Cargo check
    run: cargo check --all-targets --quiet
    working_directory: $DROP_SOURCE
```

### Post-Integration Validation
```yaml
post_integration:
  - name: Workspace check
    run: cargo check --workspace --quiet
    
  - name: Generate metadata
    run: cargo metadata --no-deps > cicd/metadata.json
```

---

## References

### Actions Used
- **actions/checkout@v4**: Repository checkout
- **dtolnay/rust-toolchain@stable**: Rust toolchain installation
- **Swatinem/rust-cache@v2**: Cargo caching
- **actions/setup-node@v4**: Node.js setup (for mixed projects)
- **actions/upload-artifact@v4**: Artifact upload

### Documentation
- [GitHub Actions - Rust](https://docs.github.com/en/actions/automating-builds-and-tests/building-and-testing-rust)
- [Swatinem/rust-cache](https://github.com/Swatinem/rust-cache)
- [dtolnay/rust-toolchain](https://github.com/dtolnay/rust-toolchain)

### Related NoaArkOS Documentation
- Portable Cargo: `server/tools/README.md`
- CRC System: `crc/README.md`
- CI/CD Overview: `cicd/README.md` (existing)

---

## Next Steps

1. **Evaluate GitHub Actions**: Determine if GitHub hosting is planned
2. **Local CI Enhancement**: Adapt patterns for portable Cargo environment
3. **CRC Integration**: Add cargo check to drop-in validation
4. **Metadata Tracking**: Generate and track workspace metadata
5. **Cache Strategy**: Explore local caching for faster builds

---

*Document v1.0 - Cycle 5 Integration - Rust CI/CD Patterns*
