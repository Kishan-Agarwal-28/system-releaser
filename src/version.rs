use std::fs;
use std::path::Path;
use std::process::Command;
use std::str::FromStr;
use semver::Version;

use crate::config::ProjectConfig;
use crate::language::Language;
use crate::metadata::ProjectMetadata;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VersionBump {
    Major,
    Minor,
    Patch,
    Set(Version),
}

impl VersionBump {
    /// Calculate the next version based on the current semantic version.
    pub fn next_version(&self, current: &Version) -> Version {
        match self {
            VersionBump::Major => Version::new(current.major + 1, 0, 0),
            VersionBump::Minor => Version::new(current.major, current.minor + 1, 0),
            VersionBump::Patch => Version::new(current.major, current.minor, current.patch + 1),
            VersionBump::Set(v) => v.clone(),
        }
    }
}

impl FromStr for VersionBump {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().trim() {
            "major" => Ok(VersionBump::Major),
            "minor" => Ok(VersionBump::Minor),
            "patch" => Ok(VersionBump::Patch),
            other => {
                let clean = other.trim_start_matches('v');
                Version::parse(clean)
                    .map(VersionBump::Set)
                    .map_err(|e| format!("Invalid version or bump specifier '{}': {}", other, e))
            }
        }
    }
}

/// Resolves the current version of the project by checking releaser.yaml, project manifests, or git tags.
pub fn resolve_current_version(project_dir: &Path, language: Language) -> Result<Version, Box<dyn std::error::Error>> {
    // 1. Check if releaser.yaml has an explicit version
    if let Ok(Some((cfg, _))) = ProjectConfig::load_from_dir(project_dir)
        && cfg.version != "auto"
    {
        let clean = cfg.version.trim_start_matches('v');
        if let Ok(v) = Version::parse(clean) {
            return Ok(v);
        }
    }

    // 2. Check project manifest via metadata extractor
    let meta = ProjectMetadata::extract(project_dir, language);
    if let Some(v_str) = meta.version {
        let clean = v_str.trim_start_matches('v');
        if let Ok(v) = Version::parse(clean) {
            return Ok(v);
        }
    }

    // 3. Fallback: query latest git tag
    let git_tag = Command::new("git")
        .args(["describe", "--tags", "--abbrev=0"])
        .current_dir(project_dir)
        .output();

    if let Ok(out) = git_tag
        && out.status.success()
    {
        let tag_str = String::from_utf8_lossy(&out.stdout).trim().to_string();
        let clean = tag_str.trim_start_matches('v');
        if let Ok(v) = Version::parse(clean) {
            return Ok(v);
        }
    }

    // Default baseline if no prior version found
    Ok(Version::new(0, 1, 0))
}

/// Synchronizes the new version across all project manifest files (Cargo.toml, package.json, pyproject.toml).
pub fn sync_version_to_manifests(
    project_dir: &Path,
    new_version: &Version,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut modified = Vec::new();
    let version_str = new_version.to_string();

    // 1. Cargo.toml
    let cargo_path = project_dir.join("Cargo.toml");
    if cargo_path.is_file() {
        let content = fs::read_to_string(&cargo_path)?;
        let mut new_lines = Vec::new();
        let mut in_package = false;
        let mut updated = false;

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed == "[package]" {
                in_package = true;
                new_lines.push(line.to_string());
                continue;
            } else if trimmed.starts_with('[') && trimmed.ends_with(']') {
                in_package = false;
            }

            // Match only a bare `version = "..."` line at the start of the [package] section.
            // Skip `version.workspace = true` and any other dotted keys.
            let is_version_key = trimmed.starts_with("version")
                && trimmed.contains('=')
                && !trimmed.starts_with("version."); // avoid version.workspace = true

            if in_package && !updated && is_version_key {
                new_lines.push(format!("version = \"{}\"", version_str));
                updated = true;
            } else {
                new_lines.push(line.to_string());
            }
        }

        if updated {
            fs::write(&cargo_path, new_lines.join("\n") + "\n")?;
            modified.push("Cargo.toml".to_string());
        }
    }

    // 2. package.json
    let pkg_path = project_dir.join("package.json");
    if pkg_path.is_file() {
        let content = fs::read_to_string(&pkg_path)?;
        if let Ok(mut json) = serde_json::from_str::<serde_json::Value>(&content)
            && let Some(obj) = json.as_object_mut()
        {
            obj.insert("version".to_string(), serde_json::Value::String(version_str.clone()));
            let formatted = serde_json::to_string_pretty(&json)?;
            fs::write(&pkg_path, formatted + "\n")?;
            modified.push("package.json".to_string());
        }
    }

    // 3. pyproject.toml
    let py_path = project_dir.join("pyproject.toml");
    if py_path.is_file() {
        let content = fs::read_to_string(&py_path)?;
        let mut new_lines = Vec::new();
        let mut in_target = false;
        let mut updated = false;

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed == "[project]" || trimmed == "[tool.poetry]" {
                in_target = true;
                new_lines.push(line.to_string());
                continue;
            } else if trimmed.starts_with('[') && trimmed.ends_with(']') {
                in_target = false;
            }

            let is_version_key = trimmed.starts_with("version")
                && trimmed.contains('=')
                && !trimmed.starts_with("version.");

            if in_target && !updated && is_version_key {
                new_lines.push(format!("version = \"{}\"", version_str));
                updated = true;
            } else {
                new_lines.push(line.to_string());
            }
        }

        if updated {
            fs::write(&py_path, new_lines.join("\n") + "\n")?;
            modified.push("pyproject.toml".to_string());
        }
    }

    Ok(modified)
}

/// Creates a Git commit and annotated tag for the release.
pub fn git_commit_and_tag(
    project_dir: &Path,
    version: &Version,
    files_to_commit: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
    let tag_name = format!("v{}", version);
    let commit_msg = format!("chore: release {}", tag_name);

    // 1. Git add modified files
    for file in files_to_commit {
        let status = Command::new("git")
            .args(["add", file])
            .current_dir(project_dir)
            .status()?;
        if !status.success() {
            return Err(format!("Failed to git add '{}'", file).into());
        }
    }

    // 2. Git commit
    let commit_status = Command::new("git")
        .args(["commit", "-m", &commit_msg])
        .current_dir(project_dir)
        .status()?;
    if !commit_status.success() {
        return Err("Failed to create git commit for release".into());
    }

    // 3. Git tag
    let tag_status = Command::new("git")
        .args(["tag", "-a", &tag_name, "-m", &format!("Release {}", tag_name)])
        .current_dir(project_dir)
        .status()?;
    if !tag_status.success() {
        return Err(format!("Failed to create git tag '{}'", tag_name).into());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_version_bump_arithmetic() {
        let v = Version::new(1, 2, 3);
        assert_eq!(VersionBump::Patch.next_version(&v), Version::new(1, 2, 4));
        assert_eq!(VersionBump::Minor.next_version(&v), Version::new(1, 3, 0));
        assert_eq!(VersionBump::Major.next_version(&v), Version::new(2, 0, 0));
        assert_eq!(
            VersionBump::Set(Version::new(3, 0, 0)).next_version(&v),
            Version::new(3, 0, 0)
        );
    }

    #[test]
    fn test_version_bump_parse() {
        assert_eq!("patch".parse::<VersionBump>().unwrap(), VersionBump::Patch);
        assert_eq!("minor".parse::<VersionBump>().unwrap(), VersionBump::Minor);
        assert_eq!("major".parse::<VersionBump>().unwrap(), VersionBump::Major);
        assert_eq!("v2.5.1".parse::<VersionBump>().unwrap(), VersionBump::Set(Version::new(2, 5, 1)));
    }

    #[test]
    fn test_sync_version_cargo() {
        let dir = tempdir().unwrap();
        let cargo_toml = "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n";
        fs::write(dir.path().join("Cargo.toml"), cargo_toml).unwrap();

        let new_v = Version::new(0, 2, 0);
        let modified = sync_version_to_manifests(dir.path(), &new_v).unwrap();
        assert!(modified.contains(&"Cargo.toml".to_string()));

        let updated = fs::read_to_string(dir.path().join("Cargo.toml")).unwrap();
        assert!(updated.contains("version = \"0.2.0\""));
    }

    #[test]
    fn test_sync_version_package_json() {
        let dir = tempdir().unwrap();
        let pkg_json = "{\"name\":\"demo\",\"version\":\"1.0.0\"}";
        fs::write(dir.path().join("package.json"), pkg_json).unwrap();

        let new_v = Version::new(1, 1, 0);
        let modified = sync_version_to_manifests(dir.path(), &new_v).unwrap();
        assert!(modified.contains(&"package.json".to_string()));

        let updated = fs::read_to_string(dir.path().join("package.json")).unwrap();
        assert!(updated.contains("\"version\": \"1.1.0\""));
    }

    #[test]
    fn test_sync_version_cargo_workspace_not_modified() {
        // A workspace member with `version.workspace = true` must NOT be overwritten.
        let dir = tempdir().unwrap();
        let cargo_toml = "[package]\nname = \"member\"\nversion.workspace = true\n";
        fs::write(dir.path().join("Cargo.toml"), cargo_toml).unwrap();

        let new_v = Version::new(2, 0, 0);
        let modified = sync_version_to_manifests(dir.path(), &new_v).unwrap();
        // Should not have modified this file because version.workspace = true is not a plain key
        assert!(!modified.contains(&"Cargo.toml".to_string()));

        let unchanged = fs::read_to_string(dir.path().join("Cargo.toml")).unwrap();
        assert!(unchanged.contains("version.workspace = true"), "workspace version must not be overwritten");
    }
}
