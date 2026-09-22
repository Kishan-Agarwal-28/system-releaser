use std::fs;
use tempfile::tempdir;
use system_releaser::{
    init_project, run_preflight_checks, InitOptions, ProjectConfig,
};

#[test]
fn test_e2e_init_and_check() {
    let dir = tempdir().unwrap();

    // 1. Create a mock Rust project
    let cargo_toml = r#"
[package]
name = "my-awesome-tool"
version = "0.5.0"
description = "A fast cross-platform CLI tool"
license = "MIT"
repository = "https://github.com/myorg/my-awesome-tool"
"#;
    fs::write(dir.path().join("Cargo.toml"), cargo_toml).unwrap();
    fs::create_dir_all(dir.path().join("src")).unwrap();
    fs::write(dir.path().join("src/main.rs"), "fn main() {}\n").unwrap();

    // 2. Run init_project
    let res = init_project(InitOptions {
        target_dir: dir.path().to_path_buf(),
        force: false,
        full: false,
        name: None,
    })
    .expect("init_project should succeed");

    assert_eq!(res.project_name, "my-awesome-tool");
    assert!(res.config_path.exists());

    // 3. Load and verify generated releaser.yaml
    let (cfg, _) = ProjectConfig::load_from_dir(dir.path())
        .unwrap()
        .expect("config must be found");

    assert_eq!(cfg.name, "my-awesome-tool");
    assert_eq!(cfg.version, "auto");
    assert_eq!(cfg.description.as_deref(), Some("A fast cross-platform CLI tool"));
    assert_eq!(cfg.license.as_deref(), Some("MIT"));
    assert_eq!(cfg.repository.as_deref(), Some("https://github.com/myorg/my-awesome-tool"));
    assert!(cfg.package_managers.enabled.len() >= 6);

    // 4. Run preflight checks
    let report = run_preflight_checks(dir.path());
    // Should pass configuration check and language detection
    assert!(report.items.iter().any(|i| i.name == "Configuration File" && i.status == system_releaser::CheckStatus::Pass));
    assert!(report.items.iter().any(|i| i.name == "Language Detection" && i.status == system_releaser::CheckStatus::Pass));
}

#[test]
fn test_init_full_template() {
    let dir = tempdir().unwrap();
    let res = init_project(InitOptions {
        target_dir: dir.path().to_path_buf(),
        force: false,
        full: true,
        name: Some("full-suite".to_string()),
    })
    .unwrap();

    let (cfg, _) = ProjectConfig::load_from_dir(dir.path()).unwrap().unwrap();
    assert_eq!(res.package_managers_count, 14);
    assert_eq!(cfg.package_managers.enabled.len(), 14);
}
