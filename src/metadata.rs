use std::fs;
use std::path::Path;
use serde_json::Value as JsonValue;

use crate::language::Language;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProjectMetadata {
    pub name: Option<String>,
    pub version: Option<String>,
    pub description: Option<String>,
    pub authors: Vec<String>,
    pub license: Option<String>,
    pub repository: Option<String>,
    pub homepage: Option<String>,
}

impl ProjectMetadata {
    /// Extracts metadata by scanning standard manifests in `root_path` according to `language`.
    pub fn extract(root_path: &Path, language: Language) -> Self {
        match language {
            Language::Rust => Self::extract_cargo(root_path),
            Language::TypeScript | Language::JavaScript => Self::extract_package_json(root_path),
            Language::Python => Self::extract_pyproject(root_path),
            Language::Go => Self::extract_go_mod(root_path),
            _ => {
                // Fallback: try Cargo.toml, then package.json, then pyproject.toml
                let cargo = Self::extract_cargo(root_path);
                if cargo.name.is_some() {
                    return cargo;
                }
                let pkg = Self::extract_package_json(root_path);
                if pkg.name.is_some() {
                    return pkg;
                }
                Self::extract_pyproject(root_path)
            }
        }
    }

    /// Extract from Cargo.toml
    pub fn extract_cargo(root_path: &Path) -> Self {
        let cargo_path = root_path.join("Cargo.toml");
        let Ok(content) = fs::read_to_string(cargo_path) else {
            return Self::default();
        };

        let mut meta = Self::default();
        let mut in_package = false;

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with('[') && trimmed.ends_with(']') {
                in_package = trimmed == "[package]";
                continue;
            }

            if in_package
                && let Some((k, v)) = parse_toml_key_value(trimmed)
            {
                match k {
                    "name" => meta.name = Some(v),
                    "version" => meta.version = Some(v),
                    "description" => meta.description = Some(v),
                    "license" => meta.license = Some(v),
                    "repository" => meta.repository = Some(v),
                    "homepage" => meta.homepage = Some(v),
                    "authors" => {
                        // Basic array extraction e.g. ["Name <email>"]
                        meta.authors = parse_string_array(&v);
                    }
                    _ => {}
                }
            }
        }

        meta
    }

    /// Extract from package.json
    pub fn extract_package_json(root_path: &Path) -> Self {
        let pkg_path = root_path.join("package.json");
        let Ok(content) = fs::read_to_string(pkg_path) else {
            return Self::default();
        };

        let Ok(json): Result<JsonValue, _> = serde_json::from_str(&content) else {
            return Self::default();
        };

        let mut authors = Vec::new();
        if let Some(author) = json.get("author").and_then(|a| a.as_str()) {
            authors.push(author.to_string());
        }

        let repository = json.get("repository").and_then(|repo| {
            if let Some(s) = repo.as_str() {
                Some(s.to_string())
            } else {
                repo.get("url").and_then(|u| u.as_str()).map(|url| {
                    url.strip_prefix("git+")
                        .unwrap_or(url)
                        .strip_suffix(".git")
                        .unwrap_or(url)
                        .to_string()
                })
            }
        });

        Self {
            name: json.get("name").and_then(|v| v.as_str()).map(ToString::to_string),
            version: json.get("version").and_then(|v| v.as_str()).map(ToString::to_string),
            description: json.get("description").and_then(|v| v.as_str()).map(ToString::to_string),
            authors,
            license: json.get("license").and_then(|v| v.as_str()).map(ToString::to_string),
            repository,
            homepage: json.get("homepage").and_then(|v| v.as_str()).map(ToString::to_string),
        }
    }

    /// Extract from pyproject.toml
    pub fn extract_pyproject(root_path: &Path) -> Self {
        let pyproject_path = root_path.join("pyproject.toml");
        let Ok(content) = fs::read_to_string(pyproject_path) else {
            return Self::default();
        };

        let mut meta = Self::default();
        let mut current_section = String::new();

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with('[') && trimmed.ends_with(']') {
                current_section = trimmed.to_string();
                continue;
            }

            if (current_section == "[project]" || current_section == "[tool.poetry]")
                && let Some((k, v)) = parse_toml_key_value(trimmed)
            {
                match k {
                    "name" => meta.name = Some(v),
                    "version" => meta.version = Some(v),
                    "description" => meta.description = Some(v),
                    "license" => meta.license = Some(v),
                    _ => {}
                }
            }
        }

        meta
    }

    /// Extract from go.mod
    pub fn extract_go_mod(root_path: &Path) -> Self {
        let go_mod_path = root_path.join("go.mod");
        let Ok(content) = fs::read_to_string(go_mod_path) else {
            return Self::default();
        };

        let mut meta = Self::default();
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("module ") {
                let module_path = trimmed.trim_start_matches("module ").trim();
                // If module is github.com/user/repo, derive name and repo
                if let Some(name) = module_path.split('/').next_back() {
                    meta.name = Some(name.to_string());
                }
                if module_path.starts_with("github.com/") {
                    meta.repository = Some(format!("https://{}", module_path));
                }
                break;
            }
        }

        meta
    }
}

/// Simple parser for key = "value" in TOML lines
fn parse_toml_key_value(line: &str) -> Option<(&str, String)> {
    let mut parts = line.splitn(2, '=');
    let key = parts.next()?.trim();
    let raw_val = parts.next()?.trim();

    let clean_val = raw_val
        .trim_matches('"')
        .trim_matches('\'')
        .to_string();

    Some((key, clean_val))
}

/// Simple parser for ["item1", "item2"] string arrays
fn parse_string_array(val: &str) -> Vec<String> {
    let trimmed = val.trim().trim_matches('[').trim_matches(']');
    trimmed
        .split(',')
        .map(|s| s.trim().trim_matches('"').trim_matches('\'').to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_extract_cargo_metadata() {
        let dir = tempdir().unwrap();
        let cargo_toml = r#"
[package]
name = "demo-cli"
version = "1.2.3"
description = "A fast demo CLI"
license = "MIT"
repository = "https://github.com/myorg/demo-cli"
homepage = "https://demo-cli.com"
authors = ["Alice <alice@example.com>"]
"#;
        fs::write(dir.path().join("Cargo.toml"), cargo_toml).unwrap();

        let meta = ProjectMetadata::extract(dir.path(), Language::Rust);
        assert_eq!(meta.name.as_deref(), Some("demo-cli"));
        assert_eq!(meta.version.as_deref(), Some("1.2.3"));
        assert_eq!(meta.description.as_deref(), Some("A fast demo CLI"));
        assert_eq!(meta.license.as_deref(), Some("MIT"));
        assert_eq!(meta.repository.as_deref(), Some("https://github.com/myorg/demo-cli"));
        assert_eq!(meta.homepage.as_deref(), Some("https://demo-cli.com"));
        assert_eq!(meta.authors, vec!["Alice <alice@example.com>"]);
    }

    #[test]
    fn test_extract_package_json_metadata() {
        let dir = tempdir().unwrap();
        let package_json = r#"{
  "name": "web-tool",
  "version": "2.0.0",
  "description": "Awesome Web Tool",
  "license": "Apache-2.0",
  "author": "Bob",
  "repository": {
    "type": "git",
    "url": "git+https://github.com/user/web-tool.git"
  }
}"#;
        fs::write(dir.path().join("package.json"), package_json).unwrap();

        let meta = ProjectMetadata::extract(dir.path(), Language::TypeScript);
        assert_eq!(meta.name.as_deref(), Some("web-tool"));
        assert_eq!(meta.version.as_deref(), Some("2.0.0"));
        assert_eq!(meta.license.as_deref(), Some("Apache-2.0"));
        assert_eq!(meta.repository.as_deref(), Some("https://github.com/user/web-tool"));
        assert_eq!(meta.authors, vec!["Bob"]);
    }

    #[test]
    fn test_extract_go_mod_metadata() {
        let dir = tempdir().unwrap();
        let go_mod = "module github.com/octocat/hello-world\n\ngo 1.21\n";
        fs::write(dir.path().join("go.mod"), go_mod).unwrap();

        let meta = ProjectMetadata::extract(dir.path(), Language::Go);
        assert_eq!(meta.name.as_deref(), Some("hello-world"));
        assert_eq!(meta.repository.as_deref(), Some("https://github.com/octocat/hello-world"));
    }
}
