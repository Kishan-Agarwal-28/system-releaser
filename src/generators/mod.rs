pub mod apk;
pub mod appimage;
pub mod apt;
pub mod asdf_mise;
pub mod choco;
pub mod flatpak;
pub mod homebrew;
pub mod macports;
pub mod nix;
pub mod pacman;
pub mod scoop;
pub mod snap;
pub mod winget;
pub mod rpm;

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

pub use apk::generate_apkbuild;
pub use appimage::{generate_appimage_artifacts, AppImageArtifacts};
pub use apt::{generate_apt_artifacts, AptArtifacts};
pub use asdf_mise::{generate_asdf_mise_plugin, AsdfMiseArtifacts};
pub use choco::{generate_choco_artifacts, ChocoArtifacts};
pub use flatpak::generate_flatpak_manifest;
pub use homebrew::generate_homebrew_formula;
pub use macports::generate_macports_portfile;
pub use nix::generate_nix_flake;
pub use pacman::generate_pkgbuild;
pub use rpm::generate_rpm_spec;
pub use scoop::generate_scoop_manifest;
pub use snap::generate_snapcraft_yaml;
pub use winget::generate_winget_manifest;

use crate::config::ProjectConfig;
use crate::installers::{generate_install_ps1, generate_install_sh};
use crate::package_manager::PackageManager;

pub struct ManifestGenerationContext<'a> {
    pub config: &'a ProjectConfig,
    pub version: &'a str,
    pub hashes: &'a HashMap<String, String>, // mapping: "windows_amd64" -> sha256
}

/// Generates all configured package manager manifests and install scripts into `output_dir`.
pub fn generate_all_manifests(
    output_dir: &Path,
    ctx: &ManifestGenerationContext,
) -> Result<Vec<PathBuf>, std::io::Error> {
    fs::create_dir_all(output_dir)?;
    let mut generated_files = Vec::new();

    let name = &ctx.config.name;
    let version = ctx.version;
    let desc = ctx.config.description.as_deref().unwrap_or(name);
    let homepage = ctx.config.homepage.as_deref().unwrap_or("https://example.com");
    let repo = ctx.config.repository.as_deref().unwrap_or("https://github.com/user/repo");
    let license = ctx.config.license.as_deref().unwrap_or("MIT");

    let win_x64_h = ctx.hashes.get("windows_amd64").map(|s| s.as_str());
    let win_arm_h = ctx.hashes.get("windows_arm64").map(|s| s.as_str());
    let mac_arm_h = ctx.hashes.get("darwin_arm64").map(|s| s.as_str());
    let mac_x64_h = ctx.hashes.get("darwin_amd64").map(|s| s.as_str());
    let lin_x64_h = ctx.hashes.get("linux_amd64").map(|s| s.as_str());
    let lin_arm_h = ctx.hashes.get("linux_arm64").map(|s| s.as_str());

    // 1. Universal Installers (install.sh & install.ps1)
    if ctx.config.install_scripts.enabled {
        let sh_content = generate_install_sh(name, Some(repo), &ctx.config.install_scripts.install_dir_unix);
        let sh_path = output_dir.join(&ctx.config.install_scripts.sh_name);
        fs::write(&sh_path, sh_content)?;
        generated_files.push(sh_path);

        let ps1_content = generate_install_ps1(name, Some(repo), &ctx.config.install_scripts.install_dir_win);
        let ps1_path = output_dir.join(&ctx.config.install_scripts.ps1_name);
        fs::write(&ps1_path, ps1_content)?;
        generated_files.push(ps1_path);
    }

    // 2. Package Managers
    for pm in &ctx.config.package_managers.enabled {
        match pm {
            PackageManager::Homebrew => {
                let content = generate_homebrew_formula(
                    name, version, desc, homepage, repo,
                    mac_arm_h, mac_x64_h, lin_x64_h, lin_arm_h,
                );
                let path = output_dir.join(format!("{}.rb", name));
                fs::write(&path, content)?;
                generated_files.push(path);
            }
            PackageManager::Winget => {
                let publisher = ctx.config.package_managers.options
                    .get("winget")
                    .and_then(|v| v.get("publisher"))
                    .and_then(|p| p.as_str())
                    .unwrap_or(name);
                let content = generate_winget_manifest(
                    publisher, name, version, desc, license, repo,
                    win_x64_h, win_arm_h,
                );
                let path = output_dir.join(format!("{}.yaml", name));
                fs::write(&path, content)?;
                generated_files.push(path);
            }
            PackageManager::Choco => {
                let author = ctx.config.package_managers.options
                    .get("choco")
                    .and_then(|v| v.get("author"))
                    .and_then(|a| a.as_str())
                    .unwrap_or(name);
                let choco_art = generate_choco_artifacts(
                    name, version, desc, author, repo, homepage, win_x64_h, win_arm_h,
                );
                let nuspec_path = output_dir.join(format!("{}.nuspec", name));
                fs::write(&nuspec_path, choco_art.nuspec)?;
                generated_files.push(nuspec_path);

                let ps1_path = output_dir.join("chocolateyInstall.ps1");
                fs::write(&ps1_path, choco_art.install_ps1)?;
                generated_files.push(ps1_path);
            }
            PackageManager::Scoop => {
                let content = generate_scoop_manifest(
                    name, version, desc, homepage, license, repo,
                    win_x64_h, win_arm_h,
                );
                let path = output_dir.join(format!("{}.json", name));
                fs::write(&path, content)?;
                generated_files.push(path);
            }
            PackageManager::Pacman => {
                let content = generate_pkgbuild(
                    name, version, desc, homepage, license, repo,
                    lin_x64_h, lin_arm_h,
                );
                let path = output_dir.join("PKGBUILD");
                fs::write(&path, content)?;
                generated_files.push(path);
            }
            PackageManager::Snap => {
                let grade = ctx.config.package_managers.options
                    .get("snap")
                    .and_then(|v| v.get("grade"))
                    .and_then(|g| g.as_str());
                let confinement = ctx.config.package_managers.options
                    .get("snap")
                    .and_then(|v| v.get("confinement"))
                    .and_then(|c| c.as_str());
                let content = generate_snapcraft_yaml(name, version, desc, grade, confinement);
                let path = output_dir.join("snapcraft.yaml");
                fs::write(&path, content)?;
                generated_files.push(path);
            }
            PackageManager::Apt => {
                let maintainer = ctx.config.package_managers.options
                    .get("apt")
                    .and_then(|v| v.get("maintainer"))
                    .and_then(|m| m.as_str())
                    .unwrap_or(name);
                let apt_art = generate_apt_artifacts(name, version, desc, maintainer, homepage, repo);
                let control_path = output_dir.join(format!("{}.control", name));
                fs::write(&control_path, apt_art.control)?;
                generated_files.push(control_path);

                let script_path = output_dir.join(format!("{}_install_deb.sh", name));
                fs::write(&script_path, apt_art.install_script)?;
                generated_files.push(script_path);
            }
            PackageManager::Rpm => {
                let spec_content = generate_rpm_spec(name, version, desc, license, homepage, repo);
                let spec_path = output_dir.join(format!("{}.spec", name));
                fs::write(&spec_path, spec_content)?;
                generated_files.push(spec_path);
            }
            PackageManager::Apk => {
                let apk_content = generate_apkbuild(
                    name, version, desc, homepage, license, repo,
                    lin_x64_h, lin_arm_h,
                );
                let apk_path = output_dir.join("APKBUILD");
                fs::write(&apk_path, apk_content)?;
                generated_files.push(apk_path);
            }
            PackageManager::Nix => {
                let flake_content = generate_nix_flake(
                    name, version, desc, repo,
                    lin_x64_h, lin_arm_h, mac_x64_h, mac_arm_h,
                );
                let flake_path = output_dir.join("flake.nix");
                fs::write(&flake_path, flake_content)?;
                generated_files.push(flake_path);
            }
            PackageManager::Flatpak => {
                let flatpak_content = generate_flatpak_manifest(
                    name, version, desc, repo,
                    lin_x64_h, lin_arm_h,
                );
                let flatpak_path = output_dir.join(format!("org.system_releaser.{}.yaml", name));
                fs::write(&flatpak_path, flatpak_content)?;
                generated_files.push(flatpak_path);
            }
            PackageManager::Appimage => {
                let appimage_art = generate_appimage_artifacts(name, version, desc, repo);
                let desktop_path = output_dir.join(format!("{}.desktop", name));
                fs::write(&desktop_path, appimage_art.desktop_file)?;
                generated_files.push(desktop_path);

                let apprun_path = output_dir.join("AppRun");
                fs::write(&apprun_path, appimage_art.apprun_script)?;
                generated_files.push(apprun_path);

                let recipe_path = output_dir.join("AppImage.yml");
                fs::write(&recipe_path, appimage_art.recipe_yaml)?;
                generated_files.push(recipe_path);
            }
            PackageManager::AsdfMise => {
                let asdf_art = generate_asdf_mise_plugin(name, repo);
                let list_path = output_dir.join("asdf_list_all.sh");
                fs::write(&list_path, asdf_art.list_all_sh)?;
                generated_files.push(list_path);

                let download_path = output_dir.join("asdf_download.sh");
                fs::write(&download_path, asdf_art.download_sh)?;
                generated_files.push(download_path);

                let install_path = output_dir.join("asdf_install.sh");
                fs::write(&install_path, asdf_art.install_sh)?;
                generated_files.push(install_path);
            }
            PackageManager::Macports => {
                let maintainer = ctx.config.package_managers.options
                    .get("macports")
                    .and_then(|v| v.get("maintainer"))
                    .and_then(|m| m.as_str())
                    .unwrap_or(name);
                let portfile_content = generate_macports_portfile(
                    name, version, desc, license, maintainer, homepage, repo,
                    mac_x64_h, mac_arm_h,
                );
                let portfile_path = output_dir.join("Portfile");
                fs::write(&portfile_path, portfile_content)?;
                generated_files.push(portfile_path);
            }
        }
    }

    Ok(generated_files)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_generate_all_manifests_e2e() {
        let dir = tempdir().unwrap();
        let config = ProjectConfig {
            name: "testcli".to_string(),
            version: "1.0.0".to_string(),
            description: Some("Test tool".to_string()),
            homepage: Some("https://test.dev".to_string()),
            repository: Some("https://github.com/testorg/testcli".to_string()),
            license: Some("MIT".to_string()),
            icon_url: None,
            platforms: crate::config::PlatformsConfig::All,
            build: crate::config::BuildConfig::default(),
            package_managers: crate::config::PackageManagersConfig {
                enabled: PackageManager::all().to_vec(),
                options: HashMap::new(),
            },
            install_scripts: crate::config::InstallScriptsConfig::default(),
            output_dir: "dist".to_string(),
        };

        let mut hashes = HashMap::new();
        hashes.insert("windows_amd64".to_string(), "h_win".to_string());
        hashes.insert("darwin_arm64".to_string(), "h_mac".to_string());
        hashes.insert("linux_amd64".to_string(), "h_lin".to_string());

        let ctx = ManifestGenerationContext {
            config: &config,
            version: "1.0.0",
            hashes: &hashes,
        };

        let generated = generate_all_manifests(dir.path(), &ctx).unwrap();
        assert!(generated.iter().any(|p| p.ends_with("install.sh")));
        assert!(generated.iter().any(|p| p.ends_with("install.ps1")));
        assert!(generated.iter().any(|p| p.ends_with("testcli.rb")));
        assert!(generated.iter().any(|p| p.ends_with("testcli.yaml")));
        assert!(generated.iter().any(|p| p.ends_with("testcli.nuspec")));
        assert!(generated.iter().any(|p| p.ends_with("chocolateyInstall.ps1")));
        assert!(generated.iter().any(|p| p.ends_with("testcli.json")));
        assert!(generated.iter().any(|p| p.ends_with("PKGBUILD")));
        assert!(generated.iter().any(|p| p.ends_with("snapcraft.yaml")));
        assert!(generated.iter().any(|p| p.ends_with("testcli.control")));
        assert!(generated.iter().any(|p| p.ends_with("testcli.spec")));
        assert!(generated.iter().any(|p| p.ends_with("APKBUILD")));
        assert!(generated.iter().any(|p| p.ends_with("flake.nix")));
        assert!(generated.iter().any(|p| p.ends_with("org.system_releaser.testcli.yaml")));
        assert!(generated.iter().any(|p| p.ends_with("testcli.desktop")));
        assert!(generated.iter().any(|p| p.ends_with("AppRun")));
        assert!(generated.iter().any(|p| p.ends_with("asdf_install.sh")));
        assert!(generated.iter().any(|p| p.ends_with("Portfile")));
    }
}
