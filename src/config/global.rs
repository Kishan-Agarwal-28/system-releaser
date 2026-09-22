use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct GlobalConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maintainer: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub github_token: Option<String>,

    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub package_managers: HashMap<String, serde_yml::Value>,
}

impl GlobalConfig {
    /// Discovers default global configuration path:
    /// 1. $SYSTEM_RELEASER_CONFIG_DIR/config.yaml
    /// 2. dirs::config_dir()/system-releaser/config.yaml
    pub fn default_path() -> Option<PathBuf> {
        if let Ok(custom) = env::var("SYSTEM_RELEASER_CONFIG_DIR") {
            return Some(Path::new(&custom).join("config.yaml"));
        }

        dirs::config_dir().map(|p| p.join("system-releaser").join("config.yaml"))
    }

    /// Loads global configuration if present on disk, otherwise returns default empty config.
    pub fn load() -> Self {
        if let Some(path) = Self::default_path()
            && path.is_file()
            && let Ok(content) = fs::read_to_string(&path)
            && let Ok(cfg) = serde_yml::from_str::<GlobalConfig>(&content)
        {
            return cfg;
        }
        GlobalConfig::default()
    }

    /// Resolves GitHub token in order of precedence:
    /// 1. Environment variable (SYSTEM_RELEASER_GITHUB_TOKEN or GITHUB_TOKEN)
    /// 2. Global config file
    pub fn resolve_github_token(&self) -> Option<String> {
        env::var("SYSTEM_RELEASER_GITHUB_TOKEN")
            .ok()
            .or_else(|| env::var("GITHUB_TOKEN").ok())
            .or_else(|| self.github_token.clone())
    }

    /// Resolves Chocolatey API key in order of precedence:
    /// 1. Environment variable (CHOCO_API_KEY)
    /// 2. Global config package_managers.choco.api_key
    pub fn resolve_choco_key(&self) -> Option<String> {
        env::var("CHOCO_API_KEY")
            .ok()
            .or_else(|| {
                self.package_managers
                    .get("choco")
                    .and_then(|v| v.get("api_key"))
                    .and_then(|k| k.as_str())
                    .map(ToString::to_string)
            })
    }

    /// Resolves Homebrew default tap in order of precedence:
    /// 1. Environment variable (HOMEBREW_TAP)
    /// 2. Global config package_managers.homebrew.tap
    pub fn resolve_homebrew_tap(&self) -> Option<String> {
        env::var("HOMEBREW_TAP")
            .ok()
            .or_else(|| {
                self.package_managers
                    .get("homebrew")
                    .and_then(|v| v.get("tap"))
                    .and_then(|t| t.as_str())
                    .map(ToString::to_string)
            })
    }

    /// Saves global configuration to default path.
    pub fn save(&self) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let path = Self::default_path().ok_or("Could not determine user config directory")?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let yaml = serde_yml::to_string(self)?;
        fs::write(&path, yaml)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = fs::Permissions::from_mode(0o600);
            let _ = fs::set_permissions(&path, perms);
        }
        Ok(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_global_config() {
        let yaml = r#"
maintainer: "Kishan <kishan@example.com>"
github_token: "ghp_mock123"
package_managers:
  homebrew:
    tap: "myorg/homebrew-tap"
  choco:
    api_key: "choco-key-456"
"#;
        let cfg: GlobalConfig = serde_yml::from_str(yaml).unwrap();
        assert_eq!(cfg.maintainer.as_deref(), Some("Kishan <kishan@example.com>"));
        assert_eq!(cfg.github_token.as_deref(), Some("ghp_mock123"));
        assert_eq!(cfg.resolve_homebrew_tap().as_deref(), Some("myorg/homebrew-tap"));
        assert_eq!(cfg.resolve_choco_key().as_deref(), Some("choco-key-456"));
    }
}
