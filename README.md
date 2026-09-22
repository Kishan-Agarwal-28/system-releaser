# system-releaser

[![Rust](https://img.shields.io/badge/rust-edition%202024-orange.svg)](https://www.rust-lang.org/)
[![Tests](https://img.shields.io/badge/tests-100%2F100%20passing-brightgreen.svg)]()
[![Clippy](https://img.shields.io/badge/clippy-0%20warnings-brightgreen.svg)]()
[![Security](https://img.shields.io/badge/security-hardened%20(CWE--78%2C%2022%2C%2094%2C%20214)-blue.svg)]()
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

**system-releaser** is a security-hardened, zero-friction project detection, cross-platform build, and multi-package-manager release engine written in Rust.

With zero or minimal configuration, `system-releaser` automatically inspects your repository, detects your language toolchains, runs your test suites, bumps semantic versions, compiles cross-platform binaries, generates universal one-liner installer scripts (`curl | sh` and `irm | iex`), and generates production-ready manifests across **14 system package managers** and **23 programming languages**.

---

## Key Highlights

- **Polyglot Project Detection**: Automatically identifies language, project name, version, authors, license, and repository across 23 languages and monorepo workspace hierarchies.
- **14 System Package Managers**: WinGet, Chocolatey, Scoop, Homebrew, Pacman, Snap, APT (.deb), RPM (.spec), Alpine APK, Nix Flake, Flatpak, AppImage, asdf/mise, and MacPorts.
- **Universal One-Liner Installers**: Generates hardened POSIX `install.sh` (`curl -fsSL ... | sh`) and PowerShell `install.ps1` (`irm ... | iex`) with multi-architecture detection and SHA-256 cryptographic verification.
- **Automated Semver Bumping**: Bump releases with `--patch`, `--minor`, `--major`, or `--set-version <semver>`. Automatically synchronizes manifests (`Cargo.toml`, `package.json`, `pyproject.toml`), commits, and creates annotated Git tags.
- **Security-First Architecture**:
  - **No Command Injection**: Safe bash array expansion in GitHub Actions, regex sanitization, and metacharacter escaping across all template generators.
  - **Path Traversal Protection**: Rejects relative `..` sequences, Unix root paths, and absolute paths in `output_dir` and binary names.
  - **Credential Safety**: Bearer tokens are kept off command-line arguments using temporary curl config files with `0o600` permissions.
  - **Atomic File Operations**: Archives are generated via `.tmp` staging files and atomically renamed to prevent corruption.
- **Turnkey CI Integration**: Generates clean, minimal GitHub Actions release workflows in seconds (`system-releaser init-ci`).

---

## Installation & Quick Start

### Build from Source

```bash
# Clone repository
git clone https://github.com/your-org/system-releaser.git
cd system-releaser

# Compile release binary
cargo build --release

# Binary will be located at target/release/system-releaser
```

### Quick Workflow

```bash
# 1. Inspect repository and detect language / manifests
system-releaser detect

# 2. Scaffold releaser.yaml configuration
system-releaser init

# 3. Preflight check environment, git status, and toolchains
system-releaser check

# 4. Run native project test suite
system-releaser test

# 5. Build release artifacts, archives, checksums, and manifests
system-releaser release --minor
```

---

## Supported Languages (23 Languages)

`system-releaser` detects project manifests and invokes native cross-compilation toolchains:

| # | Language | Primary Manifests | Native Toolchain | Build Artifact |
|---|---|---|---|---|
| 1 | **Rust** | `Cargo.toml` | `cargo` / `cross` | Native executable (`.exe` / ELF / Mach-O) |
| 2 | **Go** | `go.mod` | `go build` with `GOOS`/`GOARCH` | Native static binary |
| 3 | **C# / .NET** | `*.csproj`, `*.sln` | `dotnet publish -r <RID>` | Self-contained single-file executable |
| 4 | **Python** | `pyproject.toml`, `setup.py` | `pyinstaller --onefile` | Standalone executable binary |
| 5 | **TypeScript** | `tsconfig.json` | `pkg` / `node` bundler | Standalone Node executable |
| 6 | **JavaScript** | `package.json` | `pkg` / `node` bundler | Standalone Node executable |
| 7 | **Java** | `pom.xml`, `build.gradle` | `mvn package` / `gradle shadowJar` | Standalone runnable fat JAR |
| 8 | **Kotlin** | `build.gradle.kts` | `gradle shadowJar` | Standalone runnable fat JAR |
| 9 | **Swift** | `Package.swift` | `swift build -c release` | Native executable binary |
| 10 | **Dart** | `pubspec.yaml` | `dart compile exe` | Self-contained native executable |
| 11 | **C** | `CMakeLists.txt`, `Makefile` | `cmake --build` / `make` | Native binary |
| 12 | **C++** | `CMakeLists.txt`, `Makefile` | `cmake --build` / `make` | Native binary |
| 13 | **Zig** | `build.zig` | `zig build -Doptimize=ReleaseSafe` | Native cross-compiled binary |
| 14 | **Scala** | `build.sbt` | `sbt assembly` | Standalone assembly JAR |
| 15 | **Elixir** | `mix.exs` | `mix release` | OTP standalone release bundle |
| 16 | **Haskell** | `stack.yaml`, `*.cabal` | `stack build` / `cabal build` | Native executable binary |
| 17 | **Ruby** | `Gemfile`, `*.gemspec` | Script bundler | Packaged executable |
| 18 | **PHP** | `composer.json` | `box` / standalone PHAR | Executable PHAR binary |
| 19 | **Clojure** | `project.clj` | `lein uberjar` | Executable uberjar |
| 20 | **Shell** | `*.sh` | Script packager | Executable shell binary |
| 21 | **Lua** | `*.lua` | Script packager | Executable script bundle |
| 22 | **R** | `*.R` | Script packager | Executable script bundle |
| 23 | **Unknown** | None | Explanatory error | Rejects unsupported roots cleanly |

---

## Supported Package Managers (14 Package Managers)

`system-releaser` generates validated distribution manifests populated with release URLs, descriptions, licenses, and SHA-256 hashes:

| # | Package Manager | Target Platforms | Manifest Format | Key Capabilities |
|---|---|---|---|---|
| 1 | **WinGet** | Windows (x64, arm64) | `dist/<app>.yaml` | Singleton v1.6.0 schema, portable nested installer |
| 2 | **Chocolatey** | Windows (x64, arm64) | `dist/<app>.nuspec` + `.ps1` | XML-escaped metadata, 64-bit and ARM64 release URLs |
| 3 | **Scoop** | Windows (x64, arm64) | `dist/<app>.json` | Autoupdate block, JSON-escaped metadata |
| 4 | **Homebrew** | macOS & Linux (x64, arm64) | `dist/<app>.rb` | Ruby Formula with escaped interpolation, formula DSL |
| 5 | **Pacman / AUR** | Linux (x86_64, aarch64) | `dist/PKGBUILD` | Escaped Bash strings, multi-arch tarball sources |
| 6 | **Snap** | Linux | `dist/snapcraft.yaml` | Core22 base, confinement (`classic`/`strict`) options |
| 7 | **APT (.deb)** | Linux (amd64, arm64) | `dist/control` + `.sh` | Debian control file, SHA-256 verified installer script |
| 8 | **RPM** | Linux (x86_64, aarch64) | `dist/<app>.spec` | `%ifarch` parameterized `Source0`, escaped `%` spec macros |
| 9 | **Alpine APK** | Linux (x86_64, aarch64) | `dist/APKBUILD` | Escaped metadata, abuild-ready checksum verification |
| 10 | **Nix Flake** | Linux & macOS (4 systems) | `dist/flake.nix` | Flake outputs for x86_64/aarch64 Linux and Darwin |
| 11 | **Flatpak** | Linux (x86_64, aarch64) | `dist/flatpak.yaml` | Freedesktop Platform runtime manifest |
| 12 | **AppImage** | Linux (amd64, arm64) | `dist/AppImage.yml` + `.desktop` | AppRun desktop recipe with architecture parameterization |
| 13 | **asdf / mise** | Linux & macOS (amd64, arm64) | `dist/asdf/` | Plugin scripts (`list-all`, `download`, `install`) with SHA-256 check |
| 14 | **MacPorts** | macOS (x64, arm64) | `dist/Portfile` | TCL-escaped Portfile, authentic sha256 checksums |

---

## Universal Installers

In addition to system package managers, `system-releaser` generates self-contained installation scripts:

### POSIX Shell (`install.sh`)
```bash
curl -fsSL https://github.com/user/my-cli/releases/latest/download/install.sh | sh
# Or with wget:
wget -qO- https://github.com/user/my-cli/releases/latest/download/install.sh | sh
```
- Automatically detects OS (`linux`, `darwin`, `freebsd`) and architecture (`amd64`, `arm64`).
- Downloads and cryptographically verifies release tarballs against `checksums.txt` using anchored `awk` parsing.
- Verifies user permissions before executing `sudo` and provides non-root fallback.
- Validates that `$INSTALL_DIR` is in `$PATH` using colon-delimited token matching.

### Windows PowerShell (`install.ps1`)
```powershell
irm https://github.com/user/my-cli/releases/latest/download/install.ps1 | iex
```
- Detects architecture (`amd64` or `arm64`), cleanly rejecting unsupported 32-bit environments.
- Downloads release `.zip` and verifies SHA-256 using `Get-FileHash`.
- Expands archive, installs binary to `$HOME/.local/bin`, and registers the directory in User `Path`.
- Cleans up temporary extraction directories upon completion.

---

## CLI Command Reference

### `system-releaser detect`
Inspects project files, monorepo ancestor trees, and returns detected language, manifest evidence, and confidence scores:
```bash
system-releaser detect
system-releaser detect --short
system-releaser detect --json
```

### `system-releaser init`
Generates a minimal or complete `releaser.yaml` config populated with project metadata:
```bash
# Generate minimal config
system-releaser init

# Generate complete configuration with all 14 package managers
system-releaser init --full

# Force overwrite existing configuration
system-releaser init --force
```

### `system-releaser check`
Preflight verification of project readiness:
```bash
system-releaser check
```
Verifies:
- Project configuration syntax (`releaser.yaml`).
- Git repository cleanliness and branch status.
- Toolchain availability (`cargo`, `go`, `dotnet`, `pyinstaller`, `mvn`, etc.).
- Authentication tokens (`GITHUB_TOKEN`, `CHOCO_API_KEY`, `WINGET_TOKEN`).

### `system-releaser test`
Executes the native test runner for the detected project:
```bash
system-releaser test
```

### `system-releaser release`
Builds release targets, creates atomic archives, generates checksums, installers, and package manager manifests:
```bash
# Release using current version
system-releaser release

# Release with automatic semantic version bumping
system-releaser release --patch
system-releaser release --minor
system-releaser release --major

# Release with explicit version override
system-releaser release --set-version 1.5.0

# Dry-run mode (compiles artifacts without committing or tagging)
system-releaser release --dry-run

# Skip test suite execution during release
system-releaser release --skip-tests

# Upload release assets directly to GitHub Releases
system-releaser release --upload
```

### `system-releaser init-ci`
Generates `.github/workflows/release.yml` for automated GitHub Actions releases:
```bash
system-releaser init-ci
```

---

## Configuration Schema (`releaser.yaml`)

```yaml
# Application / binary name
name: my-cli

# Project version: "auto" reads from project manifest or latest git tag
version: auto

description: "High performance cross-platform CLI tool"
homepage: https://my-cli.dev
repository: https://github.com/my-org/my-cli
license: MIT
icon_url: https://my-cli.dev/assets/logo.png

# Target compilation matrix: 'all' or list of targets
platforms:
  - linux/amd64
  - linux/arm64
  - darwin/amd64
  - darwin/arm64
  - windows/amd64
  - windows/arm64

build:
  mode: local                      # 'local' or 'ci'
  pre_build: []                    # Optional hook commands
  post_build: []

# Package managers to generate manifests for
package_managers:
  - winget
  - choco
  - scoop
  - homebrew
  - pacman
  - snap
  - apt
  - rpm
  - apk
  - nix
  - flatpak
  - appimage
  - asdf
  - macports

# Universal installer script settings
install_scripts:
  enabled: true
  sh_name: install.sh
  ps1_name: install.ps1
  install_dir_unix: /usr/local/bin
  install_dir_win: $HOME/.local/bin

# Output directory for release artifacts (must be a relative subpath)
output_dir: dist
```

### Global Configuration (`~/.config/system-releaser/config.yaml`)

You can set global tokens and defaults so you don't need to pass them per-project:
```yaml
github_token: ghp_xxxxxxxxxxxxxxxxxxxx
default_build_mode: local
default_platforms: all
```
*Note: On Unix systems, `system-releaser` automatically sets file permissions on `config.yaml` to `0600` to prevent unauthorized credential reading.*

---

## GitHub Actions CI Workflow

`system-releaser init-ci` scaffolds a lightweight GitHub Action workflow:

```yaml
name: Release

on:
  push:
    tags:
      - 'v*'

jobs:
  release:
    runs-on: ubuntu-latest
    permissions:
      contents: write
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0

      - uses: actions/setup-node@v4
      # Toolchain setup steps as needed

      - name: Run system-releaser
        uses: ./github-action
        with:
          github_token: ${{ secrets.GITHUB_TOKEN }}
          extra_args: "--skip-tests"
```

The composite action in `github-action/action.yml` avoids `eval` and safely forwards arguments via environment-variable-backed bash array expansion (`"${ARGS[@]}"`).

---

## Security Architecture

| Security Area | Implementation & Protections |
|---|---|
| **No Command Injection** | Avoids shell string concatenation; uses `std::process::Command` with direct argument vectors. Composite action passes arguments as safe bash arrays. |
| **Path Traversal Protection** | Restricts `output_dir` and artifact names; rejects `..` parent traversal sequences, root paths (`has_root()`), and absolute paths. |
| **Credential Protection** | `upload.rs` communicates with curl via temporary configuration files (`-K <file>`) set to `0o600` permissions, keeping bearer tokens out of `/proc` and `ps` process arguments. |
| **Atomic File Writes** | Archives (`.zip`, `.tar.gz`) are written to temporary `.tmp` files and atomically renamed only upon successful completion. |
| **Integrity Verification** | All installer scripts and secondary downloaders verify SHA-256 checksums against `checksums.txt` before executing extracted binaries. |

---

## Testing & Quality Assurance

```bash
# Run unit and integration tests
cargo test

# Run with linter validation (0 warnings required)
cargo clippy --all-targets -- -D warnings
```

The test suite contains **100 tests** covering:
- Language detection heuristics across 23 languages and monorepo ancestors.
- Preflight validation checks and token requirements.
- Semver bump arithmetic and manifest synchronization.
- Atomic archive generation and SHA-256 calculation.
- Syntax and escaping validation across all 14 package manager manifest generators.
- Adversarial stress tests for path traversal, metacharacter escaping, and shell injection.

---

## License

This project is licensed under the [MIT License](LICENSE).
