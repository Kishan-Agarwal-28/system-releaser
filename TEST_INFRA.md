# E2E Test Infra: system-releaser

## Test Philosophy
- Opaque-box, requirement-driven. No dependency on implementation design.
- Methodology: Category-Partition + BVA + Pairwise + Workload Testing.

## Feature Inventory
| # | Feature | Source (requirement) | Tier 1 | Tier 2 | Tier 3 |
|---|---------|---------------------|:------:|:------:|:------:|
| 1 | Language Builders (16 languages) | ORIGINAL_REQUEST §R1 | 16 | 16 | ✓ |
| 2 | Package Manager Generators (8 new) | ORIGINAL_REQUEST §R2 | 8 | 8 | ✓ |
| 3 | GitHub Releases Uploader | ORIGINAL_REQUEST §R3 | 5 | 5 | ✓ |
| 4 | install.ps1 SHA-256 Checksum | ORIGINAL_REQUEST §R4 | 5 | 5 | ✓ |
| 5 | Token Pre-flight Validation | ORIGINAL_REQUEST §R5 | 5 | 5 | ✓ |
| 6 | GitHub Action Scaffold | ORIGINAL_REQUEST §R6 | 5 | 5 | ✓ |

## Test Architecture
- Test runner: `cargo test`
- Location: unit test modules in `src/` and integration tests in `tests/`
- Directory layout:
  - `src/builder/mod.rs` tests
  - `src/generators/*` tests
  - `src/upload.rs` tests
  - `src/installers/ps1.rs` tests
  - `src/check.rs` tests
  - `tests/cli_tests.rs` / `tests/e2e_test.rs`

## Real-World Application Scenarios (Tier 4)
| # | Scenario | Features Exercised | Complexity |
|---|----------|--------------------|------------|
| 1 | Full multi-language project release simulation | Builders, Checksums, Manifests, Upload mock | High |
| 2 | Dry run release without credentials | Pre-flight, Dry-run upload skip, Manifest generation | Medium |
| 3 | Pre-flight validation CLI check failures and passes | WinGet token fail, Choco key fail, Homebrew tap warning | Medium |
| 4 | PowerShell installer checksum verification script execution | ps1 checksum parsing, Get-FileHash comparison | Medium |
| 5 | GitHub Action workflow execution with composite action | Action input parsing, CLI argument dispatch | Medium |

## Coverage Thresholds
- Tier 1: ≥5 per feature category
- Tier 2: ≥5 boundary & edge cases
- Tier 3: Pairwise combinations
- Tier 4: ≥5 realistic application scenarios
