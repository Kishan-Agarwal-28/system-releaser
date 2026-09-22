use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use semver::Version;

use crate::archive::{compute_sha256, create_tar_gz, create_zip, generate_checksums_file};
use crate::builder::build_target;
use crate::check::run_preflight_checks;
use crate::config::ProjectConfig;
use crate::detector::detect_language;
use crate::generators::{generate_all_manifests, ManifestGenerationContext};
use crate::platform::TargetPlatform;
use crate::test_runner::run_project_tests;
use crate::version::{git_commit_and_tag, resolve_current_version, sync_version_to_manifests, VersionBump};

pub struct ReleaseOptions {
    pub project_dir: PathBuf,
    pub bump: Option<VersionBump>,
    pub skip_tests: bool,
    pub dry_run: bool,
}

#[derive(Debug)]
pub struct ReleaseSummary {
    pub version: Version,
    pub built_binaries: Vec<PathBuf>,
    pub archives: Vec<PathBuf>,
    pub checksums_file: PathBuf,
    pub manifests: Vec<PathBuf>,
}

pub fn execute_release(options: ReleaseOptions) -> Result<ReleaseSummary, Box<dyn std::error::Error>> {
    let dir = &options.project_dir;

    // 1. Load project config
    let (config, _) = ProjectConfig::load_from_dir(dir)?
        .ok_or("No releaser.yaml found. Please run 'system-releaser init' first.")?;

    // 2. Pre-flight checks — only block on hard failures (missing config, bad toolchain).
    // A dirty git tree is expected when the user hasn't committed yet; we warn but don't block.
    // In dry-run mode, missing remote publishing tokens do not block local build & packaging verification.
    let preflight = run_preflight_checks(dir);
    let hard_failures: Vec<_> = preflight.items.iter()
        .filter(|i| {
            if i.status != crate::check::CheckStatus::Fail {
                return false;
            }
            if options.dry_run && (i.name == "GitHub Token" || i.name == "Chocolatey API Key") {
                return false;
            }
            true
        })
        .collect();
    if !hard_failures.is_empty() {
        let msgs: Vec<_> = hard_failures.iter().map(|i| format!("  - {}: {}", i.name, i.message)).collect();
        return Err(format!(
            "Preflight checks failed. Run 'system-releaser check' for details.\n{}",
            msgs.join("\n")
        ).into());
    }
    // Warn about non-critical items (e.g. dirty working tree)
    for item in &preflight.items {
        if item.status == crate::check::CheckStatus::Warning {
            eprintln!("WARN [{}]: {}", item.name, item.message);
        }
    }

    // 3. Language detection
    let detection = detect_language(dir)?;
    let language = detection.primary_language;

    // 4. Run tests
    if !options.skip_tests {
        println!("Running project tests before release...");
        let test_res = run_project_tests(dir, language)?;
        if !test_res.success {
            return Err(format!("Tests failed. Release aborted. Output:\n{}", test_res.output).into());
        }
        println!("Tests passed successfully.");
    }

    // 5. Version resolution & bumping
    let current_version = resolve_current_version(dir, language)?;
    let target_version = if let Some(ref bump) = options.bump {
        let next_v = bump.next_version(&current_version);
        println!("Bumping version: {} -> {}", current_version, next_v);

        if !options.dry_run {
            let modified = sync_version_to_manifests(dir, &next_v)?;
            if !modified.is_empty() {
                println!("Updated manifests: {}", modified.join(", "));
                if let Err(e) = git_commit_and_tag(dir, &next_v, &modified) {
                    eprintln!("Warning: Git commit/tag failed: {}. Continuing release build...", e);
                }
            }
        }
        next_v
    } else {
        current_version
    };

    println!("Building release v{} for '{}'...", target_version, config.name);

    // 6. Resolve target platforms
    let targets = match &config.platforms {
        crate::config::PlatformsConfig::All => TargetPlatform::all_standard(),
        crate::config::PlatformsConfig::Specific(list) => {
            let mut resolved = Vec::new();
            for s in list {
                resolved.push(s.parse::<TargetPlatform>()?);
            }
            resolved
        }
    };

    if config.name.contains('/') || config.name.contains('\\') || config.name.contains("..") {
        return Err(format!("Project name '{}' contains invalid path characters.", config.name).into());
    }

    let clean_output_dir = config.output_dir.trim_end_matches(['/', '\\']);
    let raw_output = Path::new(clean_output_dir);
    if raw_output.is_absolute() || raw_output.has_root() {
        return Err(format!("'output_dir' in releaser.yaml must be a relative path, but found absolute path: '{}'", config.output_dir).into());
    }
    for component in raw_output.components() {
        if matches!(component, std::path::Component::ParentDir) {
            return Err(format!("'output_dir' in releaser.yaml contains '..' path traversal: '{}'", config.output_dir).into());
        }
    }
    let output_dir = dir.join(raw_output);
    let binaries_dir = output_dir.join("bin");
    fs::create_dir_all(&binaries_dir)?;

    let mut built_binaries = Vec::new();
    let mut archives = Vec::new();
    let mut hashes = HashMap::new();

    // Check for LICENSE file to include in archives
    let license_path = ["LICENSE", "LICENSE.md", "LICENSE.txt"]
        .iter()
        .map(|f| dir.join(f))
        .find(|p| p.is_file());

    // 7. Build binaries & package archives
    for target in &targets {
        println!("Compiling for target: {}...", target);
        let build_res = build_target(dir, language, &config.name, target, &binaries_dir)?;

        if build_res.binary_path.is_file() {
            built_binaries.push(build_res.binary_path.clone());

            let archive_filename = target.archive_name(&config.name, &target_version.to_string());
            let archive_path = output_dir.join(&archive_filename);

            let binary_entry_name = target.binary_name(&config.name);
            let mut files_in_archive = vec![(build_res.binary_path.as_path(), binary_entry_name.as_str())];

            if let Some(ref lic) = license_path {
                files_in_archive.push((lic.as_path(), "LICENSE"));
            }

            if target.os == crate::platform::OS::Windows {
                create_zip(&archive_path, &files_in_archive)?;
            } else {
                create_tar_gz(&archive_path, &files_in_archive)?;
            }

            let hash = compute_sha256(&archive_path)?;
            let key = format!("{}_{}", target.os, target.arch);
            hashes.insert(key, hash);

            archives.push(archive_path);
        }
    }

    // 8. Generate checksums.txt
    let checksums_file = generate_checksums_file(&output_dir, &archives)?;

    // 9. Generate universal installers and package manager manifests
    let ctx = ManifestGenerationContext {
        config: &config,
        version: &target_version.to_string(),
        hashes: &hashes,
    };
    let manifests = generate_all_manifests(&output_dir, &ctx)?;

    // 10. Upload assets to GitHub Releases (skipped if dry_run or missing token)
    if !options.dry_run {
        let global_cfg = crate::config::GlobalConfig::load();
        let token = global_cfg.resolve_github_token();
        if let Some(token_val) = token {
            if let Some(ref repo_str) = config.repository {
                println!("Uploading release assets to GitHub (repository: '{}')...", repo_str);
                let mut all_assets = archives.clone();
                all_assets.push(checksums_file.clone());
                all_assets.extend(manifests.iter().cloned());

                match crate::upload::upload_release_assets(
                    repo_str,
                    &target_version.to_string(),
                    &token_val,
                    &all_assets,
                ) {
                    Ok(upload_res) => {
                        println!(
                            "Successfully published release to GitHub: {}",
                            upload_res.release_url
                        );
                    }
                    Err(e) => {
                        eprintln!("WARN: GitHub release upload failed: {}. Release files remain available locally in '{}'.", e, output_dir.display());
                    }
                }
            } else {
                println!("Note: No 'repository' configured in releaser.yaml — skipping GitHub Release upload.");
            }
        } else {
            println!("Note: GITHUB_TOKEN not set — skipping GitHub Release upload.");
        }
    }

    Ok(ReleaseSummary {
        version: target_version,
        built_binaries,
        archives,
        checksums_file,
        manifests,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_execute_release_workflow() {
        let dir = tempdir().unwrap();

        // Write Cargo.toml
        let cargo_toml = "[package]\nname = \"rel-test\"\nversion = \"0.1.0\"\n";
        fs::write(dir.path().join("Cargo.toml"), cargo_toml).unwrap();
        fs::create_dir_all(dir.path().join("src")).unwrap();
        fs::write(dir.path().join("src/main.rs"), "fn main() {}\n").unwrap();

        // Write releaser.yaml
        let releaser_yaml = r#"
name: rel-test
version: 0.1.0
platforms:
  - windows/amd64
package_managers:
  - winget
  - choco
"#;
        fs::write(dir.path().join("releaser.yaml"), releaser_yaml).unwrap();

        let options = ReleaseOptions {
            project_dir: dir.path().to_path_buf(),
            bump: None,
            skip_tests: true,
            dry_run: true,
        };

        let summary = execute_release(options).unwrap();
        assert_eq!(summary.version, Version::new(0, 1, 0));
        assert!(summary.checksums_file.is_file());
        assert!(!summary.manifests.is_empty());
    }
}
