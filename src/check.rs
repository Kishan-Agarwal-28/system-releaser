use std::path::Path;
use std::process::Command;

use crate::config::{GlobalConfig, ProjectConfig};
use crate::detector::detect_language;

#[derive(Debug, Clone)]
pub struct CheckItem {
    pub name: String,
    pub status: CheckStatus,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckStatus {
    Pass,
    Warning,
    Fail,
}

#[derive(Debug, Clone)]
pub struct CheckReport {
    pub items: Vec<CheckItem>,
}

impl CheckReport {
    pub fn is_clean(&self) -> bool {
        !self.items.iter().any(|i| i.status == CheckStatus::Fail)
    }
}

pub fn run_preflight_checks(project_dir: &Path) -> CheckReport {
    let mut items = Vec::new();

    // 1. Check project configuration
    let config_res = ProjectConfig::load_from_dir(project_dir);
    let project_config = match config_res {
        Ok(Some((cfg, path))) => {
            items.push(CheckItem {
                name: "Configuration File".to_string(),
                status: CheckStatus::Pass,
                message: format!("Loaded valid config from '{}'", path.file_name().unwrap().to_string_lossy()),
            });
            Some(cfg)
        }
        Ok(None) => {
            items.push(CheckItem {
                name: "Configuration File".to_string(),
                status: CheckStatus::Fail,
                message: "No releaser.yaml found. Run 'system-releaser init' first.".to_string(),
            });
            None
        }
        Err(err) => {
            items.push(CheckItem {
                name: "Configuration File".to_string(),
                status: CheckStatus::Fail,
                message: format!("Invalid configuration syntax: {}", err),
            });
            None
        }
    };

    // 2. Check language & toolchain
    if let Ok(detection) = detect_language(project_dir) {
        let lang = detection.primary_language;
        items.push(CheckItem {
            name: "Language Detection".to_string(),
            status: CheckStatus::Pass,
            message: format!("Identified {} (Confidence: {})", lang, detection.confidence),
        });

        // Check if compiler / toolchain exists in PATH
        if let Some(tool) = lang.default_tool() {
            let candidates: Vec<&str> = tool.split('/').map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
            let check_cmd = if cfg!(target_os = "windows") { "where" } else { "which" };
            let found_tool = candidates.iter().find(|cmd| {
                let primary = cmd.split_whitespace().next().unwrap_or(cmd);
                Command::new(check_cmd).arg(primary).output().map(|o| o.status.success()).unwrap_or(false)
            });

            if let Some(tool_name) = found_tool {
                items.push(CheckItem {
                    name: "Build Toolchain".to_string(),
                    status: CheckStatus::Pass,
                    message: format!("Found toolchain executable '{}' in PATH", tool_name),
                });
            } else {
                let first = candidates.first().unwrap_or(&"compiler");
                items.push(CheckItem {
                    name: "Build Toolchain".to_string(),
                    status: CheckStatus::Warning,
                    message: format!("Toolchain for {} (e.g. '{}') was not found in PATH", lang, first),
                });
            }
        }

        // Check cross-compilation readiness if multi-target builds are configured
        let has_foreign_targets = if let Some(ref cfg) = project_config {
            let host = crate::platform::TargetPlatform::host();
            match cfg.platforms {
                crate::config::PlatformsConfig::All => true,
                crate::config::PlatformsConfig::Host => false,
                crate::config::PlatformsConfig::Specific(ref list) => list.iter().any(|s| {
                    s.parse::<crate::platform::TargetPlatform>()
                        .map(|t| t != host)
                        .unwrap_or(true)
                }),
            }
        } else {
            false
        };

        if has_foreign_targets && lang == crate::language::Language::Rust {
            if crate::builder::is_cargo_zigbuild_available() {
                items.push(CheckItem {
                    name: "Cross-Compilation Toolchain".to_string(),
                    status: CheckStatus::Pass,
                    message: "Found cargo-zigbuild / zig for containerless multi-platform builds".to_string(),
                });
            } else if crate::builder::is_cross_available() {
                items.push(CheckItem {
                    name: "Cross-Compilation Toolchain".to_string(),
                    status: CheckStatus::Pass,
                    message: "Found 'cross' CLI for multi-platform builds".to_string(),
                });
            } else {
                items.push(CheckItem {
                    name: "Cross-Compilation Toolchain".to_string(),
                    status: CheckStatus::Warning,
                    message: "No cross-compilation toolchain detected (cargo-zigbuild/zig/cross). Multi-platform targets may be skipped during local releases. Install via 'pip install ziglang cargo-zigbuild' or 'cargo install cargo-zigbuild'.".to_string(),
                });
            }
        }
    }

    // 3. Check Git repository status
    let git_check = Command::new("git")
        .args(["rev-parse", "--is-inside-work-tree"])
        .current_dir(project_dir)
        .output();

    match git_check {
        Ok(out) if out.status.success() => {
            items.push(CheckItem {
                name: "Git Repository".to_string(),
                status: CheckStatus::Pass,
                message: "Project is a git repository".to_string(),
            });

            // Check for uncommitted changes
            let status_check = Command::new("git")
                .args(["status", "--porcelain"])
                .current_dir(project_dir)
                .output();

            if let Ok(s_out) = status_check {
                let status_str = String::from_utf8_lossy(&s_out.stdout);
                if status_str.trim().is_empty() {
                    items.push(CheckItem {
                        name: "Git Working Tree".to_string(),
                        status: CheckStatus::Pass,
                        message: "Working directory is clean (ready for release)".to_string(),
                    });
                } else {
                    items.push(CheckItem {
                        name: "Git Working Tree".to_string(),
                        status: CheckStatus::Warning,
                        message: "Working directory has uncommitted changes".to_string(),
                    });
                }
            }
        }
        _ => {
            items.push(CheckItem {
                name: "Git Repository".to_string(),
                status: CheckStatus::Warning,
                message: "Directory is not a Git repository. Version tagging and GitHub Releases require Git.".to_string(),
            });
        }
    }

    // 4. Check credentials in global config or environment
    let global_cfg = GlobalConfig::load();
    let has_github_token = global_cfg.resolve_github_token().is_some();

    if let Some(ref cfg) = project_config {
        let is_local = cfg.build.mode == crate::config::BuildMode::Local;

        // Does the project require GitHub token for publishing?
        let requires_github = cfg.package_managers.enabled.contains(&crate::package_manager::PackageManager::Winget)
            || cfg.repository.is_some();

        if has_github_token {
            items.push(CheckItem {
                name: "GitHub Token".to_string(),
                status: CheckStatus::Pass,
                message: "Found GitHub token (via env or global config)".to_string(),
            });
        } else if is_local && requires_github {
            items.push(CheckItem {
                name: "GitHub Token".to_string(),
                status: CheckStatus::Fail,
                message: "Missing GITHUB_TOKEN. Required for publishing releases / package manifests (WinGet, GitHub Releases). Set GITHUB_TOKEN or run in CI mode.".to_string(),
            });
        } else if is_local {
            items.push(CheckItem {
                name: "GitHub Token".to_string(),
                status: CheckStatus::Warning,
                message: "No GitHub token found. Set GITHUB_TOKEN or run in CI mode for automated releases.".to_string(),
            });
        }

        // Chocolatey credentials check
        if cfg.package_managers.enabled.contains(&crate::package_manager::PackageManager::Choco) {
            if global_cfg.resolve_choco_key().is_some() {
                items.push(CheckItem {
                    name: "Chocolatey API Key".to_string(),
                    status: CheckStatus::Pass,
                    message: "Found Chocolatey API key (via CHOCO_API_KEY or global config)".to_string(),
                });
            } else if is_local {
                items.push(CheckItem {
                    name: "Chocolatey API Key".to_string(),
                    status: CheckStatus::Fail,
                    message: "Missing Chocolatey API key. Set CHOCO_API_KEY or configure global config to publish to Chocolatey.".to_string(),
                });
            }
        }

        // Homebrew tap check
        if cfg.package_managers.enabled.contains(&crate::package_manager::PackageManager::Homebrew) {
            let has_tap = global_cfg.resolve_homebrew_tap().is_some()
                || cfg.package_managers.options.get("homebrew").and_then(|v| v.get("tap")).is_some();
            if has_tap {
                items.push(CheckItem {
                    name: "Homebrew Tap".to_string(),
                    status: CheckStatus::Pass,
                    message: "Found Homebrew tap configuration".to_string(),
                });
            } else {
                items.push(CheckItem {
                    name: "Homebrew Tap".to_string(),
                    status: CheckStatus::Warning,
                    message: "No custom Homebrew tap specified. Formula will be generated locally in dist/.".to_string(),
                });
            }
        }
    } else if has_github_token {
        items.push(CheckItem {
            name: "GitHub Token".to_string(),
            status: CheckStatus::Pass,
            message: "Found GitHub token (via env or global config)".to_string(),
        });
    }

    CheckReport { items }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs;

    #[test]
    fn test_check_fails_when_no_config() {
        let dir = tempdir().unwrap();
        let report = run_preflight_checks(dir.path());
        assert!(!report.is_clean());
        assert!(report.items.iter().any(|i| i.name == "Configuration File" && i.status == CheckStatus::Fail));
    }

    #[test]
    fn test_check_fails_when_winget_requires_token() {
        let dir = tempdir().unwrap();
        let releaser_yaml = r#"
name: test-app
version: 1.0.0
repository: https://github.com/myorg/test-app
package_managers:
  - winget
"#;
        fs::write(dir.path().join("releaser.yaml"), releaser_yaml).unwrap();

        // Run checks ensuring GITHUB_TOKEN is unset for this test scope
        let old_token = std::env::var("GITHUB_TOKEN").ok();
        unsafe { std::env::remove_var("GITHUB_TOKEN"); }
        unsafe { std::env::remove_var("SYSTEM_RELEASER_GITHUB_TOKEN"); }

        let report = run_preflight_checks(dir.path());
        assert!(report.items.iter().any(|i| i.name == "GitHub Token" && i.status == CheckStatus::Fail));

        if let Some(t) = old_token {
            unsafe { std::env::set_var("GITHUB_TOKEN", t); }
        }
    }

    #[test]
    fn test_check_fails_when_choco_requires_key() {
        let dir = tempdir().unwrap();
        let releaser_yaml = r#"
name: test-app
version: 1.0.0
package_managers:
  - choco
"#;
        fs::write(dir.path().join("releaser.yaml"), releaser_yaml).unwrap();

        let old_key = std::env::var("CHOCO_API_KEY").ok();
        unsafe { std::env::remove_var("CHOCO_API_KEY"); }

        let report = run_preflight_checks(dir.path());
        assert!(report.items.iter().any(|i| i.name == "Chocolatey API Key" && i.status == CheckStatus::Fail));

        if let Some(k) = old_key {
            unsafe { std::env::set_var("CHOCO_API_KEY", k); }
        }
    }
}
