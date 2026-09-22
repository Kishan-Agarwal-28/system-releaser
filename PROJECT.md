# Project: system-releaser Comprehensive Audit

## Architecture
- **Language Detection & Metadata**: `src/detector.rs`, `src/metadata.rs`, `src/language.rs`
- **Configuration Engine**: `src/config/project.rs`, `src/config/global.rs`
- **Builder Subsystem**: `src/builder/mod.rs` (supporting 23 language variants), `src/platform.rs`
- **Archive & Hashing**: `src/archive.rs` (zip, tar.gz, sha256 checksums)
- **Installers & Generators**: `src/installers/sh.rs`, `src/installers/ps1.rs`, `src/generators/*.rs` (14 package managers)
- **Release & Upload Workflow**: `src/release.rs`, `src/upload.rs`, `src/version.rs`, `src/check.rs`, `src/ci.rs`
- **CLI Commands**: `src/main.rs` (`detect`, `init`, `init-ci`, `check`, `test`, `release`)
- **Test Infrastructure**: `tests/adversarial_stress_tests.rs`, `tests/cli_tests.rs`, `src/*` (96 total tests)

## Feature Inventory
| # | Feature / Area | Description | Milestone | Source |
|---|----------------|-------------|-----------|--------|
| 1 | Action Command Injection (SEC-01) | Command injection via `eval "$CMD"` in `github-action/action.yml` | M1 | survey_1 |
| 2 | Installer Script Injections (SEC-02) | String interpolation injection in `install.sh` & `install.ps1` | M1 | survey_1 |
| 3 | Manifest Generator Injections (SEC-03) | Code injection across 12 manifest generators (Ruby, Bash, XML, YAML, Tcl, Nix, RPM) | M1 | survey_1 |
| 4 | Path Traversal Vulnerabilities (SEC-04) | Arbitrary file overwrite via `output_dir`, `sh_name`, `ps1_name` | M1 | survey_1 |
| 5 | Credential Exposure & Perms (SEC-05) | Token exposure in `curl` CLI args and world-readable `config.yaml` | M1 | survey_1 |
| 6 | Insecure Remote Downloads (SEC-06) | Missing SHA-256 validation in `apt.rs`, `appimage.rs`, `asdf_mise.rs` | M1 | survey_1 |
| 7 | Flawed Checksum Parsing (SEC-07) | Unanchored regex grep and automatic `sudo` elevation in `install.sh` | M1 | survey_1 |
| 8 | URL Parameter Encoding (SEC-08) | Unencoded asset filenames in GitHub release asset uploads | M1 | survey_1 |
| 9 | Security Test Gap (SEC-09) | Zero existing security/injection test coverage across test suites | M1 | survey_1 |
| 10 | Error Architecture Deficits (FM-01) | Absence of `src/error.rs`, ad-hoc boxed errors, non-UTF8 panic risks | M2 | survey_2 |
| 11 | Workspace Root & Packages (FM-02, FM-03) | Root detection failure in monorepos; `[workspace.package]` ignored in Cargo | M2 | survey_2 |
| 12 | Polyglot Tie Resolution (FM-04) | Non-deterministic language detection on tied source counts via HashMap | M2 | survey_2 |
| 13 | Windows UNC Path Hazards (FM-05) | `canonicalize()` verbatim `\\?\` breaks Go, curl, PyInstaller | M2 | survey_2 |
| 14 | Cross-Platform Path Formats (FM-06, 07, 08) | Mixed slashes in PS1 PATH, spaces breaking pacman & homebrew | M2 | survey_2 |
| 15 | Filesystem Boundary Breaches (FM-09) | `output_dir` absolute path traversal in `release.rs` | M2 | survey_2 |
| 16 | Malformed Inputs & Config (FM-10, 11, 12, 13) | Missing `releaser.toml` support, empty `name`, swallowed YAML syntax errors | M2 | survey_2 |
| 17 | Toolchain Preflight & Masking (FM-14, FM-16) | Missing toolchains marked Warning not Fail; stale binary reuse | M2 | survey_2 |
| 18 | Host Fallback Contamination (FM-15) | Cross-compilation failure silently packages host binary into target archive | M2 | survey_2 |
| 19 | Test Runner Fallback (FM-17) | 17 of 23 languages return dummy `success: true` in test runner | M2 | survey_2 |
| 20 | Archive Recovery & Robustness (FM-18, 19, 20) | Non-atomic archives, silent skip of missing binaries, curl HTTP failure masking | M2 | survey_2 |
| 21 | Language Support Matrix (23 Languages) | Complete preflight, detection, toolchain, and artifact mapping for 23 languages | M3 | survey_3 |
| 22 | Package Manager Generators (14 PMs) | Manifest syntax, structural validity, parameters, and arch support for 14 PMs | M4 | survey_3 |
| 23 | Universal Installers (`sh.rs`, `ps1.rs`) | Installer scripts OS matrix, checksum verification, privilege escalation | M4 | survey_3 |
| 24 | CLI Commands Operational Correctness | Verification of `detect`, `init`, `init-ci`, `check`, `test`, `release` | M5 | survey_3 |
| 25 | Regression & Baseline Test Suite (96 Tests) | Execution and regression verification of 86 unit, 8 adversarial, 2 CLI tests | M5 | survey_3 |
| 26 | Master Audit Synthesis & Scorecards | Final report with severity catalog, reproductions, remediations, 4 scorecards | M6 | orchestrator |

## Milestones
| # | Name | Scope | Dependencies | Status |
|---|------|-------|-------------|--------|
| M1 | Security & Injection Audit (R1) | SEC-01 through SEC-09: Shell injection, manifest template injection, path traversal, credentials, TLS & checksums | Survey | DONE |
| M2 | Resilience & Edge Case Analysis (R2) | FM-01 through FM-20: Workspace configs, UNC paths, spaces, malformed inputs, toolchain fallbacks, archive recovery | Survey | DONE |
| M3 | Language Support Verification (R3) | Complete scorecard & verification for all 23 language variants in `src/language.rs` and `src/builder/mod.rs` | Survey | DONE |
| M4 | Manifest & Installer Validation (R4) | Complete scorecard & structural validation of 14 package managers, `install.sh`, and `install.ps1` | Survey | DONE |
| M5 | E2E Operational Correctness (R5) | Empirical verification of CLI commands (`detect`, `init`, `init-ci`, `check`, `test`, `release`) and 96 tests | Survey | DONE |
| M6 | Master Audit Synthesis & Forensic Verification | Master audit report compilation, 4 scorecards, forensic audit verification, Sentinel handoff | M1-M5 | DONE |

## Interface Contracts
### Security Audit (M1) -> Master Synthesis (M6)
- Input: SEC-01 through SEC-09 survey evidence
- Output: Categorized vulnerability entries with CVSS/severity, CWE, affected lines, concrete attack scenarios, reproduction steps, and remediation diffs.

### Resilience Audit (M2) -> Master Synthesis (M6)
- Input: FM-01 through FM-20 survey evidence
- Output: Failure mode catalog with severity, reproduction scenarios, cross-platform impact (Windows UNC vs Unix), and remediation diffs.

### Language Verification (M3) -> Master Synthesis (M6)
- Input: 23 language inventory from survey_3
- Output: (a) 23 Languages Scorecard with detection status, preflight status, build toolchain status, artifact location status, and notes.

### Manifest & Installer Validation (M4) -> Master Synthesis (M6)
- Input: 14 package manager and installer inventory from survey_3
- Output: (b) 14 Package Managers Scorecard and (c) Universal Installers Scorecard with syntax validity, arch matrix, and checksum rigor.

### E2E Operational Correctness (M5) -> Master Synthesis (M6)
- Input: CLI subcommand flows, 96-test baseline
- Output: (d) GitHub Release & CI integrations Scorecard, CLI test execution results, 96-test pass verification, regression scorecard.

## Code Layout
- `src/lib.rs`: Library root exporting all modules
- `src/main.rs`: CLI binary entrypoint (clap command routing)
- `src/detector.rs`: Language and manifest detection heuristics
- `src/language.rs`: 23 language enum variants
- `src/builder/mod.rs`: Target build execution and cross-compilation dispatch
- `src/package_manager.rs`: 14 package manager enum variants
- `src/generators/`: 14 package manager manifest generator implementations
- `src/installers/`: `sh.rs` and `ps1.rs` installer script generators
- `src/release.rs`: Core release orchestration pipeline
- `src/archive.rs`: Zip, tar.gz, and sha256 checksums
- `src/upload.rs`: GitHub Release asset upload via curl
- `src/check.rs`: Preflight validation checks
- `src/version.rs`: Semver parsing, bump arithmetic, manifest synchronization
- `src/ci.rs`: GitHub Actions workflow generation
- `tests/adversarial_stress_tests.rs`: Stress and multi-platform enum tests
- `tests/cli_tests.rs`: CLI integration tests
