use std::fs;
use std::path::{Path, PathBuf};

/// Generates the streamlined, 10-line GitHub Action workflow into `.github/workflows/release.yml`.
pub fn generate_github_action_workflow(project_dir: &Path) -> Result<PathBuf, std::io::Error> {
    let workflow_dir = project_dir.join(".github").join("workflows");
    fs::create_dir_all(&workflow_dir)?;
    let workflow_file = workflow_dir.join("release.yml");

    let yaml = r#"name: Release

on:
  push:
    tags:
      - 'v*'

permissions:
  contents: write
  pull-requests: write

jobs:
  release:
    name: Build & Publish Release
    runs-on: ubuntu-latest
    steps:
      - name: Checkout repository
        uses: actions/checkout@v4
        with:
          fetch-depth: 0

      - name: Install containerless cross-compilation toolchains
        run: |
          pip install --quiet ziglang cargo-zigbuild

      - name: Run system-releaser
        uses: Kishan-Agarwal-28/system-releaser-action@v0.0.1
        with:
          github_token: ${{ secrets.GITHUB_TOKEN }}
"#;

    fs::write(&workflow_file, yaml)?;
    Ok(workflow_file)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_generate_github_action_workflow() {
        let dir = tempdir().unwrap();
        let path = generate_github_action_workflow(dir.path()).unwrap();
        assert!(path.is_file());

        let content = fs::read_to_string(path).unwrap();
        assert!(content.contains("Kishan-Agarwal-28/system-releaser-action@v0.0.1"));
        assert!(content.contains("tags:\n      - 'v*'"));
    }

    #[test]
    fn test_action_yml_structure() {
        let action_path = Path::new("github-action").join("action.yml");
        if action_path.is_file() {
            let content = fs::read_to_string(&action_path).unwrap();
            let parsed: serde_yml::Value = serde_yml::from_str(&content).expect("action.yml must be valid YAML");
            assert!(parsed.get("name").is_some());
            assert!(parsed.get("inputs").is_some());
            assert!(parsed.get("runs").is_some());

            let inputs = parsed.get("inputs").unwrap().as_mapping().unwrap();
            assert!(inputs.contains_key("github_token") || inputs.contains_key("github-token"));
        }
    }
}
