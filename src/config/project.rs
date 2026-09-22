use std::collections::HashMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::path::{Path, PathBuf};
use std::fs;

use crate::package_manager::PackageManager;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BuildMode {
    #[default]
    Local,
    Ci,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BuildConfig {
    #[serde(default)]
    pub mode: BuildMode,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub pre_build: Vec<String>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub post_build: Vec<String>,
}

impl Default for BuildConfig {
    fn default() -> Self {
        Self {
            mode: BuildMode::Local,
            pre_build: Vec::new(),
            post_build: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum PlatformsConfig {
    #[default]
    All,
    Host,
    Specific(Vec<String>),
}

impl Serialize for PlatformsConfig {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            PlatformsConfig::All => serializer.serialize_str("all"),
            PlatformsConfig::Host => serializer.serialize_str("host"),
            PlatformsConfig::Specific(list) => list.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for PlatformsConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error;
        let value = serde_yml::Value::deserialize(deserializer)?;
        match value {
            serde_yml::Value::String(s) if s.to_lowercase() == "all" => Ok(PlatformsConfig::All),
            serde_yml::Value::String(s) if s.to_lowercase() == "host" => Ok(PlatformsConfig::Host),
            serde_yml::Value::Sequence(seq) => {
                let mut list = Vec::new();
                for item in seq {
                    if let serde_yml::Value::String(s) = item {
                        list.push(s);
                    } else {
                        return Err(D::Error::custom("Platforms list must contain strings"));
                    }
                }
                Ok(PlatformsConfig::Specific(list))
            }
            _ => Err(D::Error::custom("Platforms must be 'all', 'host', or a list of platform targets")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstallScriptsConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,

    #[serde(default = "default_sh_name")]
    pub sh_name: String,

    #[serde(default = "default_ps1_name")]
    pub ps1_name: String,

    #[serde(default = "default_unix_install_dir")]
    pub install_dir_unix: String,

    #[serde(default = "default_win_install_dir")]
    pub install_dir_win: String,
}

fn default_true() -> bool {
    true
}

fn default_sh_name() -> String {
    "install.sh".to_string()
}

fn default_ps1_name() -> String {
    "install.ps1".to_string()
}

fn default_unix_install_dir() -> String {
    "/usr/local/bin".to_string()
}

fn default_win_install_dir() -> String {
    "$HOME/.local/bin".to_string()
}

impl Default for InstallScriptsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            sh_name: default_sh_name(),
            ps1_name: default_ps1_name(),
            install_dir_unix: default_unix_install_dir(),
            install_dir_win: default_win_install_dir(),
        }
    }
}

/// Package managers configuration container that accepts either a flat list
/// or a detailed mapping with per-package-manager settings.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PackageManagersConfig {
    pub enabled: Vec<PackageManager>,
    pub options: HashMap<String, serde_yml::Value>,
}

impl Serialize for PackageManagersConfig {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if self.options.is_empty() {
            // Serialize as clean simple array of names
            self.enabled.serialize(serializer)
        } else {
            // Serialize as mapping — serde_yml::Mapping uses String keys
            let mut map = serde_yml::Mapping::new();
            for pm in &self.enabled {
                let key = pm.to_string();
                let val = self
                    .options
                    .get(&pm.to_string())
                    .cloned()
                    .unwrap_or(serde_yml::Value::Mapping(serde_yml::Mapping::new()));
                map.insert(key, val);
            }
            serde_yml::Value::Mapping(map).serialize(serializer)
        }
    }
}

impl<'de> Deserialize<'de> for PackageManagersConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error;
        let value = serde_yml::Value::deserialize(deserializer)?;
        match value {
            serde_yml::Value::Sequence(seq) => {
                let mut enabled = Vec::new();
                for item in seq {
                    if let serde_yml::Value::String(s) = item {
                        let pm = s.parse::<PackageManager>().map_err(D::Error::custom)?;
                        enabled.push(pm);
                    }
                }
                Ok(PackageManagersConfig {
                    enabled,
                    options: HashMap::new(),
                })
            }
            serde_yml::Value::Mapping(map) => {
                let mut enabled = Vec::new();
                let mut options = HashMap::new();
                // serde_yml::Mapping iterates as (String, Value) pairs
                for (name, v) in map {
                    let pm = name.parse::<PackageManager>().map_err(D::Error::custom)?;
                    enabled.push(pm);
                    options.insert(name, v);
                }
                Ok(PackageManagersConfig { enabled, options })
            }
            _ => Err(D::Error::custom("package_managers must be a list or mapping")),
        }
    }
}

/// Project-level configuration parsed from `releaser.yaml` or `.system-releaser.yaml`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectConfig {
    /// Binary / application name
    pub name: String,

    /// Project version or "auto" to automatically read from project manifest / git tag
    #[serde(default = "default_version_auto")]
    pub version: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub homepage: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_url: Option<String>,

    #[serde(default)]
    pub platforms: PlatformsConfig,

    #[serde(default)]
    pub build: BuildConfig,

    #[serde(default)]
    pub package_managers: PackageManagersConfig,

    #[serde(default)]
    pub install_scripts: InstallScriptsConfig,

    #[serde(default)]
    pub readme: ReadmeConfig,

    #[serde(default = "default_output_dir")]
    pub output_dir: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ReadmeConfig {
    #[serde(default = "default_true")]
    pub update: bool,

    #[serde(default = "default_readme_file")]
    pub file: String,
}

fn default_readme_file() -> String {
    "README.md".to_string()
}

impl Default for ReadmeConfig {
    fn default() -> Self {
        Self {
            update: true,
            file: default_readme_file(),
        }
    }
}

impl<'de> Deserialize<'de> for ReadmeConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = serde_yml::Value::deserialize(deserializer)?;
        match value {
            serde_yml::Value::Bool(b) => Ok(ReadmeConfig {
                update: b,
                file: default_readme_file(),
            }),
            serde_yml::Value::String(s) => Ok(ReadmeConfig {
                update: true,
                file: s,
            }),
            serde_yml::Value::Mapping(map) => {
                let mut cfg = ReadmeConfig::default();
                if let Some(serde_yml::Value::Bool(b)) = map.get("update") {
                    cfg.update = *b;
                }
                if let Some(serde_yml::Value::String(s)) = map.get("file") {
                    cfg.file = s.clone();
                }
                Ok(cfg)
            }
            _ => Ok(ReadmeConfig::default()),
        }
    }
}

fn default_version_auto() -> String {
    "auto".to_string()
}

fn default_output_dir() -> String {
    "dist".to_string()
}

impl ProjectConfig {
    /// Attempt to find and load `releaser.yaml` or `.system-releaser.yaml` in `dir`.
    pub fn load_from_dir(dir: &Path) -> Result<Option<(ProjectConfig, PathBuf)>, Box<dyn std::error::Error>> {
        let candidates = ["releaser.yaml", "releaser.yml", ".system-releaser.yaml", ".system-releaser.yml"];
        for candidate in candidates {
            let path = dir.join(candidate);
            if path.is_file() {
                let content = fs::read_to_string(&path)?;
                let config: ProjectConfig = serde_yml::from_str(&content)?;
                return Ok(Some((config, path)));
            }
        }
        Ok(None)
    }

    /// Serialize project configuration to clean YAML string.
    pub fn to_yaml(&self) -> Result<String, serde_yml::Error> {
        serde_yml::to_string(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_minimal_project_config() {
        let yaml = r#"
name: my-tool
platforms: all
package_managers:
  - winget
  - choco
  - homebrew
  - apt
"#;
        let cfg: ProjectConfig = serde_yml::from_str(yaml).unwrap();
        assert_eq!(cfg.name, "my-tool");
        assert_eq!(cfg.version, "auto");
        assert_eq!(cfg.platforms, PlatformsConfig::All);
        assert_eq!(cfg.package_managers.enabled.len(), 4);
        assert!(cfg.install_scripts.enabled);
        assert_eq!(cfg.output_dir, "dist");
    }

    #[test]
    fn test_parse_detailed_package_managers() {
        let yaml = r#"
name: advanced-tool
package_managers:
  homebrew:
    tap: myorg/homebrew-tap
  winget:
    publisher: MyCompany
"#;
        let cfg: ProjectConfig = serde_yml::from_str(yaml).unwrap();
        assert_eq!(cfg.name, "advanced-tool");
        assert_eq!(cfg.package_managers.enabled.len(), 2);
        assert!(cfg.package_managers.options.contains_key("homebrew"));
        assert!(cfg.package_managers.options.contains_key("winget"));
    }

    #[test]
    fn test_parse_platforms_host() {
        let yaml = r#"
name: host-tool
platforms: host
"#;
        let cfg: ProjectConfig = serde_yml::from_str(yaml).unwrap();
        assert_eq!(cfg.platforms, PlatformsConfig::Host);

        let out = cfg.to_yaml().unwrap();
        assert!(out.contains("platforms: host"));
    }
}
