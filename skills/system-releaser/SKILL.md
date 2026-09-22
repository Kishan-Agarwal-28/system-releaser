---
name: system-releaser
description: Automates binary release engineering, cross-compilation, packaging, manifest generation (14 package managers), and GitHub Releases publishing across 23 languages.
---

# system-releaser Agent Skill

This skill guides AI coding assistants (Claude Code, Cursor, Windsurf, GitHub Copilot, Antigravity) in automating software releases using `system-releaser`.

## 1. When to Use This Skill

Activate this skill when:
- The user wants to release or publish a software project (CLI, daemon, library).
- Setting up multi-platform binary distribution (Linux, macOS, Windows).
- Generating package manager manifests (Homebrew, WinGet, APT, Pacman, Scoop, Chocolatey, Nix, etc.).
- Creating universal installation scripts (`install.sh` and `install.ps1`).
- Configuring automated CI/CD release workflows on GitHub Actions.
- Debugging preflight validation errors or failed releases.

## 2. CLI Command Workflows

Always follow this sequence when preparing or publishing a release:

### Phase 1: Project Discovery
```bash
releaser detect
```
- Scans root manifest files (`Cargo.toml`, `go.mod`, `package.json`, `pyproject.toml`, etc.).
- Identifies the primary programming language and default compiler toolchain.
- Reports confidence level and detected project name/version.

### Phase 2: Configuration Initialization
```bash
# Generate baseline releaser.yaml
releaser init

# Include all 14 package managers in the template
releaser init --full

# Force overwrite existing configuration
releaser init --force
```

### Phase 3: Preflight Validation
```bash
releaser check
```
Runs 6 critical environment audits:
1. Syntax and schema validation of `releaser.yaml`.
2. Required compiler toolchain presence in PATH.
3. Cleanliness of the git working tree.
4. Resolution of the 6 target cross-compilation platforms.
5. Presence and validity of `GITHUB_TOKEN`.
6. Package manager generator prerequisites.

### Phase 4: Test Suite Execution
```bash
releaser test
```
Executes the native test runner for the detected project language (e.g. `cargo test`, `go test ./...`, `npm test`, `pytest`).

### Phase 5: Build, Package, and Publish
```bash
# Dry run (builds, packages, and hashes without pushing git tags or GitHub releases)
releaser release --dry-run

# Patch bump (e.g. 1.0.0 -> 1.0.1)
releaser release --patch

# Minor bump (e.g. 1.0.0 -> 1.1.0)
releaser release --minor

# Major bump (e.g. 1.0.0 -> 2.0.0)
releaser release --major

# Explicit semver release
releaser release --set-version 2.1.0
```

### Phase 6: CI/CD Generation
```bash
releaser init-ci
```
Generates `.github/workflows/release.yml` for automated releases on git tag pushes.

## 3. Configuration Contract (`releaser.yaml`)

```yaml
name: my-app                   # Binary application name
version: auto                 # Resolves from git tag or project manifest
description: CLI utility      # Project summary
homepage: https://example.com # Project website
repository: user/my-app       # GitHub owner/repo
license: MIT OR Apache-2.0    # SPDX license identifier

# 6 standard cross-compilation targets
platforms:
  - linux/amd64               # x86_64-unknown-linux-musl (.tar.gz)
  - linux/arm64               # aarch64-unknown-linux-musl (.tar.gz)
  - darwin/arm64              # aarch64-apple-darwin (.tar.gz)
  - darwin/amd64              # x86_64-apple-darwin (.tar.gz)
  - windows/amd64             # x86_64-pc-windows-msvc (.zip)
  - windows/arm64             # aarch64-pc-windows-msvc (.zip)

# Enabled package manager manifest generators
package_managers:
  - homebrew                  # Formula/my-app.rb
  - winget                    # manifests/u/my-app.yaml
  - pacman                    # PKGBUILD
  - scoop                     # bucket/my-app.json
  - choco                     # my-app.nuspec + chocolateyInstall.ps1
  - nix                       # flake.nix
  - apt                       # debian/control
  - rpm                       # my-app.spec
  - apk                       # APKBUILD
  - snap                      # snapcraft.yaml
  - flatpak                   # my-app.yaml
  - macports                  # Portfile
  - appimage                  # AppRun + AppImage.yml
  - asdf_mise                 # bin/list-all, bin/download, bin/install

install_scripts:
  enabled: true
  sh_name: install.sh
  ps1_name: install.ps1
  install_dir_unix: /usr/local/bin
  install_dir_win: $HOME/.local/bin

output_dir: dist
```

## 4. Security Invariants for AI Agents

When modifying release configurations or generating custom build scripts, maintain these security invariants:
1. **Never pass tokens in CLI arguments**: Pass `GITHUB_TOKEN` via environment variable or global config. `system-releaser` injects it via temporary curl config files (`-K`) with 0600 permissions.
2. **Path Traversal Protection**: Ensure `output_dir` is a relative subfolder without `..` parent references.
3. **Atomic Operations**: All archives stage in `.tmp` files before being atomically renamed upon SHA-256 verification.
