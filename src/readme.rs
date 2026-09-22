use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::package_manager::PackageManager;
use crate::platform::{Arch, TargetPlatform, OS};

pub const INSTALL_MARKER_START: &str = "<!-- system-releaser:install:start -->";
pub const INSTALL_MARKER_END: &str = "<!-- system-releaser:install:end -->";

/// Generates a standardized, copy-pasteable installation section in GitHub Flavored Markdown.
#[allow(clippy::too_many_arguments)]
pub fn generate_installation_markdown(
    app_name: &str,
    version: &str,
    repository: Option<&str>,
    targets: &[TargetPlatform],
    hashes: &HashMap<String, String>,
    package_managers: &[PackageManager],
    install_scripts_enabled: bool,
    output_dir_name: &str,
) -> String {
    let mut out = String::new();
    let clean_version = version.trim_start_matches('v');
    let tag = format!("v{}", clean_version);
    let dist = if output_dir_name.trim().is_empty() {
        "dist"
    } else {
        output_dir_name.trim().trim_matches(['/', '\\'])
    };

    out.push_str(INSTALL_MARKER_START);
    out.push_str("\n## Installation\n\n");

    // 1. One-Liner Installers
    if let (true, Some(repo)) = (install_scripts_enabled, repository) {
        out.push_str("### 1. One-Line Installers (Recommended)\n\n");
        out.push_str("#### Linux & macOS (POSIX Shell)\n");
        out.push_str("```bash\n");
        out.push_str(&format!(
            "curl -fsSL https://raw.githubusercontent.com/{}/main/{}/install.sh | sh\n",
            repo, dist
        ));
        out.push_str("```\n\n");

        out.push_str("#### Windows (PowerShell)\n");
        out.push_str("```powershell\n");
        out.push_str(&format!(
            "irm https://raw.githubusercontent.com/{}/main/{}/install.ps1 | iex\n",
            repo, dist
        ));
        out.push_str("```\n\n");
        out.push_str("---\n\n");
    }

    // 2. Pre-Built Binary Direct Downloads
    if !targets.is_empty() {
        out.push_str(&format!("### 2. Pre-Built Binaries ({})\n\n", tag));
        out.push_str("| Platform | Architecture | Archive | Checksum |\n");
        out.push_str("|---|---|---|---|\n");

        for target in targets {
            let platform_name = match target.os {
                OS::Linux => "Linux",
                OS::Darwin => "macOS",
                OS::Windows => "Windows",
            };

            let arch_name = match (target.os, target.arch) {
                (OS::Darwin, Arch::Arm64) => "Apple Silicon (`arm64`)".to_string(),
                (OS::Darwin, Arch::Amd64) => "Intel (`x86_64`)".to_string(),
                (_, Arch::Amd64) => "x86_64 (`amd64`)".to_string(),
                (_, Arch::Arm64) => "ARM64 (`aarch64`)".to_string(),
            };

            let archive_filename = target.archive_name(app_name, clean_version);
            let key = format!("{}_{}", target.os, target.arch);
            let hash_display = hashes.get(&key).map(|h| {
                if h.len() >= 12 {
                    format!("`{}...`", &h[..10])
                } else {
                    format!("`{}`", h)
                }
            });

            if let Some(repo) = repository {
                let download_url = format!(
                    "https://github.com/{}/releases/download/{}/{}",
                    repo, tag, archive_filename
                );
                let checksums_url = format!(
                    "https://github.com/{}/releases/download/{}/checksums.txt",
                    repo, tag
                );

                out.push_str(&format!(
                    "| **{}** | {} | [{}]({}) | [SHA-256]({}) |\n",
                    platform_name, arch_name, archive_filename, download_url, checksums_url
                ));
            } else {
                let hash_str = hash_display.unwrap_or_else(|| "SHA-256".to_string());
                out.push_str(&format!(
                    "| **{}** | {} | `{}` | {} |\n",
                    platform_name, arch_name, archive_filename, hash_str
                ));
            }
        }

        if let Some(repo) = repository {
            out.push_str(&format!(
                "\n*Cryptographic SHA-256 hashes are listed in [`checksums.txt`](https://github.com/{}/releases/download/{}/checksums.txt).*\n\n",
                repo, tag
            ));
        } else {
            out.push_str("\n*Release archives include cryptographic SHA-256 verification.*\n\n");
        }
        out.push_str("---\n\n");
    }

    // 3. Package Managers
    let mut pm_commands = Vec::new();
    let repo_owner = repository
        .and_then(|r| r.split('/').next())
        .unwrap_or("user");

    for pm in package_managers {
        match pm {
            PackageManager::Winget => {
                pm_commands.push((
                    "WinGet (Windows)",
                    format!("winget install {}.{}", repo_owner, app_name),
                    "cmd",
                ));
            }
            PackageManager::Homebrew => {
                pm_commands.push((
                    "Homebrew (macOS & Linux)",
                    format!("brew install {}/tap/{}", repo_owner, app_name),
                    "bash",
                ));
            }
            PackageManager::Scoop => {
                if let Some(repo) = repository {
                    pm_commands.push((
                        "Scoop (Windows)",
                        format!(
                            "scoop install https://raw.githubusercontent.com/{}/main/{}/{}.json",
                            repo, dist, app_name
                        ),
                        "powershell",
                    ));
                } else {
                    pm_commands.push((
                        "Scoop (Windows)",
                        format!("scoop install {}/{}.json", dist, app_name),
                        "powershell",
                    ));
                }
            }
            PackageManager::Choco => {
                pm_commands.push((
                    "Chocolatey (Windows)",
                    format!("choco install {}", app_name),
                    "cmd",
                ));
            }
            PackageManager::Nix => {
                if let Some(repo) = repository {
                    pm_commands.push((
                        "Nix Flake",
                        format!("nix run github:{}/{}", repo, tag),
                        "bash",
                    ));
                }
            }
            _ => {}
        }
    }

    if !pm_commands.is_empty() {
        out.push_str("### 3. Package Managers\n\n");
        for (label, cmd, lang) in pm_commands {
            out.push_str(&format!("#### {}\n", label));
            out.push_str(&format!("```{}\n{}\n```\n\n", lang, cmd));
        }
    }

    out.push_str(INSTALL_MARKER_END);
    out
}

/// Injects or replaces the installation guide inside existing README markdown content.
pub fn inject_installation_guide(original_content: &str, guide: &str, app_name: &str) -> String {
    let trimmed_original = original_content.trim();

    // Case 1: Empty file
    if trimmed_original.is_empty() {
        return format!("# {}\n\n{}\n", app_name, guide);
    }

    // Case 2: Markers already present
    let marker_span = original_content.find(INSTALL_MARKER_START).and_then(|start_idx| {
        original_content[start_idx..].find(INSTALL_MARKER_END).map(|end_offset| {
            let end_idx = start_idx + end_offset + INSTALL_MARKER_END.len();
            (start_idx, end_idx)
        })
    });
    if let Some((start_idx, end_idx)) = marker_span {
        let before = &original_content[..start_idx];
        let after = &original_content[end_idx..];
        return format!("{}\n\n{}\n\n{}", before.trim_end(), guide, after.trim_start());
    }

    // Case 3: Existing `## Installation` section (without markers)
    if let Some(install_idx) = original_content.find("## Installation") {
        let after_heading = &original_content[install_idx + "## Installation".len()..];
        // Look for the next section heading (## or #)
        let next_section_offset = after_heading
            .find("\n## ")
            .or_else(|| after_heading.find("\n# "));

        let before = &original_content[..install_idx];
        return match next_section_offset {
            Some(offset) => {
                let end_idx = install_idx + "## Installation".len() + offset;
                let after = &original_content[end_idx..];
                format!("{}\n\n{}\n\n{}", before.trim_end(), guide, after.trim_start())
            }
            None => {
                format!("{}\n\n{}\n", before.trim_end(), guide)
            }
        };
    }

    // Case 4: Prepend after main heading / badges / intro paragraph
    let lines: Vec<&str> = original_content.lines().collect();
    let mut insert_line_idx = 0;
    let mut found_title = false;

    for (idx, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("# ") && !found_title {
            found_title = true;
            insert_line_idx = idx + 1;
            continue;
        }

        if found_title {
            // Check if we hit the next major section (## )
            if trimmed.starts_with("## ") {
                insert_line_idx = idx;
                break;
            }

            // Keep scanning past badges, images, empty lines, and short introductory text
            if trimmed.starts_with("[![")
                || trimmed.starts_with("<p")
                || trimmed.starts_with("<img")
                || trimmed.starts_with("<a ")
                || trimmed.starts_with(">")
                || trimmed.is_empty()
            {
                insert_line_idx = idx + 1;
            } else if trimmed == "---" {
                // If a separator divider is right under the title/badges, insert right after it
                insert_line_idx = idx + 1;
                break;
            } else {
                // Intro text paragraph line — continue scanning until blank line or next section
                insert_line_idx = idx + 1;
            }
        }
    }

    if found_title && insert_line_idx <= lines.len() {
        let before = lines[..insert_line_idx].join("\n");
        let after = lines[insert_line_idx..].join("\n");
        let clean_before = before.trim_end();
        let clean_after = after.trim_start();

        if clean_after.is_empty() {
            format!("{}\n\n{}\n", clean_before, guide)
        } else {
            format!("{}\n\n{}\n\n---\n\n{}", clean_before, guide, clean_after)
        }
    } else {
        // Fallback: prepend at the very beginning of the document
        format!("{}\n\n---\n\n{}\n", guide, original_content.trim_start())
    }
}

/// Updates the target project's README file with the generated installation section.
#[allow(clippy::too_many_arguments)]
pub fn update_readme_installation_section(
    project_dir: &Path,
    configured_file: &str,
    app_name: &str,
    version: &str,
    repository: Option<&str>,
    targets: &[TargetPlatform],
    hashes: &HashMap<String, String>,
    package_managers: &[PackageManager],
    install_scripts_enabled: bool,
    output_dir_name: &str,
) -> Result<Option<PathBuf>, Box<dyn std::error::Error>> {
    let target_file = if !configured_file.trim().is_empty() {
        project_dir.join(configured_file.trim())
    } else {
        // Search candidate files
        let candidates = ["README.md", "readme.md", "README", "readme"];
        candidates
            .iter()
            .map(|c| project_dir.join(c))
            .find(|p| p.is_file())
            .unwrap_or_else(|| project_dir.join("README.md"))
    };

    let original_content = if target_file.is_file() {
        fs::read_to_string(&target_file)?
    } else {
        String::new()
    };

    let guide = generate_installation_markdown(
        app_name,
        version,
        repository,
        targets,
        hashes,
        package_managers,
        install_scripts_enabled,
        output_dir_name,
    );

    let updated_content = inject_installation_guide(&original_content, &guide, app_name);

    if let Some(parent) = target_file.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&target_file, updated_content)?;

    Ok(Some(target_file))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_generate_installation_markdown_contains_expected_sections() {
        let targets = vec![
            TargetPlatform::new(OS::Linux, Arch::Amd64),
            TargetPlatform::new(OS::Windows, Arch::Amd64),
            TargetPlatform::new(OS::Darwin, Arch::Arm64),
        ];
        let mut hashes = HashMap::new();
        hashes.insert("linux_amd64".to_string(), "0123456789abcdef".to_string());
        hashes.insert("windows_amd64".to_string(), "fedcba9876543210".to_string());

        let pms = vec![PackageManager::Winget, PackageManager::Homebrew];

        let md = generate_installation_markdown(
            "cool-tool",
            "1.2.0",
            Some("my-org/cool-tool"),
            &targets,
            &hashes,
            &pms,
            true,
            "dist",
        );

        assert!(md.contains(INSTALL_MARKER_START));
        assert!(md.contains(INSTALL_MARKER_END));
        assert!(md.contains("curl -fsSL https://raw.githubusercontent.com/my-org/cool-tool/main/dist/install.sh | sh"));
        assert!(md.contains("irm https://raw.githubusercontent.com/my-org/cool-tool/main/dist/install.ps1 | iex"));
        assert!(md.contains("cool-tool_1.2.0_linux_amd64.tar.gz"));
        assert!(md.contains("cool-tool_1.2.0_windows_amd64.zip"));
        assert!(md.contains("cool-tool_1.2.0_darwin_arm64.tar.gz"));
        assert!(md.contains("winget install my-org.cool-tool"));
        assert!(md.contains("brew install my-org/tap/cool-tool"));
    }

    #[test]
    fn test_inject_into_empty_content() {
        let guide = "<!-- system-releaser:install:start -->\n## Installation\n<!-- system-releaser:install:end -->";
        let res = inject_installation_guide("", guide, "my-app");
        assert!(res.contains("# my-app"));
        assert!(res.contains("<!-- system-releaser:install:start -->"));
    }

    #[test]
    fn test_inject_replaces_existing_markers() {
        let existing = r#"# My Project

Intro here.

<!-- system-releaser:install:start -->
## Installation (old)
old content
<!-- system-releaser:install:end -->

## Features
- Fast
"#;
        let new_guide = "<!-- system-releaser:install:start -->\n## Installation (new)\n<!-- system-releaser:install:end -->";
        let res = inject_installation_guide(existing, new_guide, "My Project");
        assert!(!res.contains("old content"));
        assert!(res.contains("## Installation (new)"));
        assert!(res.contains("## Features"));
    }

    #[test]
    fn test_inject_replaces_unmarked_installation_heading() {
        let existing = r#"# My Project

Intro here.

## Installation
Manual compile steps.

## Features
- Great
"#;
        let guide = "<!-- system-releaser:install:start -->\n## Installation\ncurl | sh\n<!-- system-releaser:install:end -->";
        let res = inject_installation_guide(existing, guide, "My Project");
        assert!(!res.contains("Manual compile steps"));
        assert!(res.contains("curl | sh"));
        assert!(res.contains("## Features"));
    }

    #[test]
    fn test_update_readme_file_e2e() {
        let dir = tempdir().unwrap();
        let readme = dir.path().join("README.md");
        fs::write(&readme, "# Test Tool\n\nA small utility.\n\n## Usage\nRun test-tool.\n").unwrap();

        let targets = vec![TargetPlatform::new(OS::Linux, Arch::Amd64)];
        let hashes = HashMap::new();

        let res = update_readme_installation_section(
            dir.path(),
            "README.md",
            "test-tool",
            "0.1.0",
            Some("owner/test-tool"),
            &targets,
            &hashes,
            &[],
            true,
            "dist",
        ).unwrap();

        assert!(res.is_some());
        let updated = fs::read_to_string(readme).unwrap();
        assert!(updated.contains(INSTALL_MARKER_START));
        assert!(updated.contains("## Installation"));
        assert!(updated.contains("## Usage"));
    }
}
