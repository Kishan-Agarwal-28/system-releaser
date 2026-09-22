use std::fs;
use std::path::PathBuf;

use crate::config::{
    BuildConfig, BuildMode, InstallScriptsConfig, PackageManagersConfig, PlatformsConfig,
    ProjectConfig, ReadmeConfig,
};
use crate::detector::detect_language;
use crate::metadata::ProjectMetadata;
use crate::package_manager::PackageManager;

pub struct InitOptions {
    pub target_dir: PathBuf,
    pub force: bool,
    pub full: bool,
    pub name: Option<String>,
}

#[derive(Debug)]
pub struct InitResult {
    pub config_path: PathBuf,
    pub project_name: String,
    pub language: String,
    pub package_managers_count: usize,
}

pub fn init_project(options: InitOptions) -> Result<InitResult, Box<dyn std::error::Error>> {
    let dir = &options.target_dir;
    let config_file = dir.join("releaser.yaml");

    if config_file.exists() && !options.force {
        return Err(format!(
            "Configuration file '{}' already exists. Use --force to overwrite.",
            config_file.display()
        )
        .into());
    }

    // 1. Detect language
    let detection = detect_language(dir)?;
    let language = detection.primary_language;

    // 2. Extract metadata
    let metadata = ProjectMetadata::extract(dir, language);

    // 3. Resolve project name
    let project_name = options
        .name
        .or(metadata.name)
        .or_else(|| {
            dir.file_name()
                .and_then(|s| s.to_str())
                .map(ToString::to_string)
        })
        .unwrap_or_else(|| "my-cli".to_string());

    let package_managers = if options.full {
        PackageManager::all().to_vec()
    } else {
        PackageManager::default_selection().to_vec()
    };

    let count = package_managers.len();

    // 4. Build configuration object
    let config = ProjectConfig {
        name: project_name.clone(),
        version: "auto".to_string(),
        description: metadata.description.or(Some(format!("{} CLI tool", project_name))),
        homepage: metadata.homepage,
        repository: metadata.repository,
        license: metadata.license.or(Some("MIT".to_string())),
        icon_url: Some(format!(
            "https://raw.githubusercontent.com/user/{}/main/assets/icon.png",
            project_name
        )),
        platforms: PlatformsConfig::All,
        build: BuildConfig {
            mode: BuildMode::Local,
            pre_build: Vec::new(),
            post_build: Vec::new(),
        },
        package_managers: PackageManagersConfig {
            enabled: package_managers,
            options: std::collections::HashMap::new(),
        },
        install_scripts: InstallScriptsConfig::default(),
        readme: ReadmeConfig::default(),
        output_dir: "dist".to_string(),
    };

    let yaml = format!(
        "# yaml-language-server: $schema=https://kishan-agarwal-28.github.io/system-releaser/schema.json\n\n{}",
        config.to_yaml()?
    );
    fs::write(&config_file, yaml)?;

    Ok(InitResult {
        config_path: config_file,
        project_name,
        language: language.to_string(),
        package_managers_count: count,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_init_in_empty_dir() {
        let dir = tempdir().unwrap();
        let options = InitOptions {
            target_dir: dir.path().to_path_buf(),
            force: false,
            full: false,
            name: Some("test-tool".to_string()),
        };

        let result = init_project(options).unwrap();
        assert_eq!(result.project_name, "test-tool");
        assert!(result.config_path.exists());

        // Check if generated file can be loaded
        let content = fs::read_to_string(&result.config_path).unwrap();
        assert!(content.contains("# yaml-language-server: $schema=https://kishan-agarwal-28.github.io/system-releaser/schema.json"));

        let (cfg, _) = ProjectConfig::load_from_dir(dir.path()).unwrap().unwrap();
        assert_eq!(cfg.name, "test-tool");
        assert_eq!(cfg.version, "auto");
        assert_eq!(cfg.platforms, PlatformsConfig::All);
    }

    #[test]
    fn test_init_fails_without_force_when_exists() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("releaser.yaml"), "name: existing").unwrap();

        let options = InitOptions {
            target_dir: dir.path().to_path_buf(),
            force: false,
            full: false,
            name: None,
        };

        let err = init_project(options).unwrap_err();
        assert!(err.to_string().contains("already exists"));
    }
}
