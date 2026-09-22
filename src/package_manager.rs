use std::fmt;
use std::str::FromStr;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TargetOs {
    Windows,
    MacOS,
    Linux,
    Universal,
}

impl fmt::Display for TargetOs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TargetOs::Windows => write!(f, "Windows"),
            TargetOs::MacOS => write!(f, "macOS"),
            TargetOs::Linux => write!(f, "Linux"),
            TargetOs::Universal => write!(f, "Universal"),
        }
    }
}

/// Comprehensive list of package managers supported across Windows, macOS, and Linux ecosystems.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PackageManager {
    // Windows
    Winget,
    Choco,
    Scoop,

    // macOS & Unix
    Homebrew,
    Macports,
    Nix,

    // Linux Distro Packages
    Apt,
    Pacman,
    Rpm,
    Apk,

    // Linux Universal / Sandboxed
    Snap,
    Flatpak,
    Appimage,

    // Universal / Dev Tools
    AsdfMise,
}

impl PackageManager {
    /// Human-readable display name of the package manager.
    pub fn display_name(&self) -> &'static str {
        match self {
            PackageManager::Winget => "WinGet (Windows Package Manager)",
            PackageManager::Choco => "Chocolatey",
            PackageManager::Scoop => "Scoop",
            PackageManager::Homebrew => "Homebrew",
            PackageManager::Macports => "MacPorts",
            PackageManager::Nix => "Nix (Flake / Nixpkgs)",
            PackageManager::Apt => "APT (Debian / Ubuntu .deb)",
            PackageManager::Pacman => "Pacman (Arch Linux / AUR)",
            PackageManager::Rpm => "RPM (Fedora / RHEL / openSUSE)",
            PackageManager::Apk => "APK (Alpine Linux)",
            PackageManager::Snap => "Snapcraft (Canonical Snap)",
            PackageManager::Flatpak => "Flatpak (Flathub)",
            PackageManager::Appimage => "AppImage",
            PackageManager::AsdfMise => "asdf / mise version manager",
        }
    }

    /// Primary operating system targeted by this package manager.
    pub fn target_os(&self) -> TargetOs {
        match self {
            PackageManager::Winget | PackageManager::Choco | PackageManager::Scoop => {
                TargetOs::Windows
            }
            PackageManager::Homebrew | PackageManager::Macports => TargetOs::MacOS,
            PackageManager::Nix => TargetOs::Universal,
            PackageManager::Apt
            | PackageManager::Pacman
            | PackageManager::Rpm
            | PackageManager::Apk
            | PackageManager::Snap
            | PackageManager::Flatpak
            | PackageManager::Appimage => TargetOs::Linux,
            PackageManager::AsdfMise => TargetOs::Universal,
        }
    }

    /// Default manifest or artifact file generated for this package manager.
    pub fn default_artifact_name(&self, app_name: &str) -> String {
        match self {
            PackageManager::Winget => format!("{}.yaml", app_name),
            PackageManager::Choco => format!("{}.nuspec", app_name),
            PackageManager::Scoop => format!("{}.json", app_name),
            PackageManager::Homebrew => format!("{}.rb", app_name),
            PackageManager::Macports => "Portfile".to_string(),
            PackageManager::Nix => "flake.nix".to_string(),
            PackageManager::Apt => format!("{}.deb", app_name),
            PackageManager::Pacman => "PKGBUILD".to_string(),
            PackageManager::Rpm => format!("{}.rpm", app_name),
            PackageManager::Apk => "APKBUILD".to_string(),
            PackageManager::Snap => "snapcraft.yaml".to_string(),
            PackageManager::Flatpak => format!("{}.yaml", app_name),
            PackageManager::Appimage => format!("{}.AppImage", app_name),
            PackageManager::AsdfMise => format!("{}-plugin", app_name),
        }
    }

    /// List of all standard package managers.
    pub fn all() -> &'static [PackageManager] {
        &[
            PackageManager::Winget,
            PackageManager::Choco,
            PackageManager::Scoop,
            PackageManager::Homebrew,
            PackageManager::Macports,
            PackageManager::Nix,
            PackageManager::Apt,
            PackageManager::Pacman,
            PackageManager::Rpm,
            PackageManager::Apk,
            PackageManager::Snap,
            PackageManager::Flatpak,
            PackageManager::Appimage,
            PackageManager::AsdfMise,
        ]
    }

    /// Default recommended set of package managers for initial config.
    pub fn default_selection() -> &'static [PackageManager] {
        &[
            PackageManager::Winget,
            PackageManager::Choco,
            PackageManager::Scoop,
            PackageManager::Homebrew,
            PackageManager::Apt,
            PackageManager::Pacman,
            PackageManager::Rpm,
            PackageManager::Snap,
        ]
    }
}

impl fmt::Display for PackageManager {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PackageManager::Winget => write!(f, "winget"),
            PackageManager::Choco => write!(f, "choco"),
            PackageManager::Scoop => write!(f, "scoop"),
            PackageManager::Homebrew => write!(f, "homebrew"),
            PackageManager::Macports => write!(f, "macports"),
            PackageManager::Nix => write!(f, "nix"),
            PackageManager::Apt => write!(f, "apt"),
            PackageManager::Pacman => write!(f, "pacman"),
            PackageManager::Rpm => write!(f, "rpm"),
            PackageManager::Apk => write!(f, "apk"),
            PackageManager::Snap => write!(f, "snap"),
            PackageManager::Flatpak => write!(f, "flatpak"),
            PackageManager::Appimage => write!(f, "appimage"),
            PackageManager::AsdfMise => write!(f, "asdf_mise"),
        }
    }
}

impl FromStr for PackageManager {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().trim() {
            "winget" => Ok(PackageManager::Winget),
            "choco" | "chocolatey" => Ok(PackageManager::Choco),
            "scoop" => Ok(PackageManager::Scoop),
            "homebrew" | "brew" => Ok(PackageManager::Homebrew),
            "macports" | "port" => Ok(PackageManager::Macports),
            "nix" | "nixpkgs" => Ok(PackageManager::Nix),
            "apt" | "deb" | "debian" | "ubuntu" => Ok(PackageManager::Apt),
            "pacman" | "aur" | "arch" => Ok(PackageManager::Pacman),
            "rpm" | "fedora" | "rhel" => Ok(PackageManager::Rpm),
            "apk" | "alpine" => Ok(PackageManager::Apk),
            "snap" | "snapcraft" => Ok(PackageManager::Snap),
            "flatpak" | "flathub" => Ok(PackageManager::Flatpak),
            "appimage" => Ok(PackageManager::Appimage),
            "asdf" | "mise" | "asdf_mise" => Ok(PackageManager::AsdfMise),
            other => Err(format!("Unknown package manager '{}'", other)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_package_manager_from_str() {
        assert_eq!("winget".parse::<PackageManager>().unwrap(), PackageManager::Winget);
        assert_eq!("choco".parse::<PackageManager>().unwrap(), PackageManager::Choco);
        assert_eq!("chocolatey".parse::<PackageManager>().unwrap(), PackageManager::Choco);
        assert_eq!("brew".parse::<PackageManager>().unwrap(), PackageManager::Homebrew);
        assert_eq!("homebrew".parse::<PackageManager>().unwrap(), PackageManager::Homebrew);
        assert_eq!("deb".parse::<PackageManager>().unwrap(), PackageManager::Apt);
        assert_eq!("aur".parse::<PackageManager>().unwrap(), PackageManager::Pacman);
        assert_eq!("pacman".parse::<PackageManager>().unwrap(), PackageManager::Pacman);
        assert_eq!("rpm".parse::<PackageManager>().unwrap(), PackageManager::Rpm);
        assert_eq!("snap".parse::<PackageManager>().unwrap(), PackageManager::Snap);
    }

    #[test]
    fn test_target_os() {
        assert_eq!(PackageManager::Winget.target_os(), TargetOs::Windows);
        assert_eq!(PackageManager::Homebrew.target_os(), TargetOs::MacOS);
        assert_eq!(PackageManager::Apt.target_os(), TargetOs::Linux);
        assert_eq!(PackageManager::Nix.target_os(), TargetOs::Universal);
    }

    #[test]
    fn test_default_artifact_names() {
        assert_eq!(PackageManager::Winget.default_artifact_name("mycli"), "mycli.yaml");
        assert_eq!(PackageManager::Homebrew.default_artifact_name("mycli"), "mycli.rb");
        assert_eq!(PackageManager::Apt.default_artifact_name("mycli"), "mycli.deb");
        assert_eq!(PackageManager::Pacman.default_artifact_name("mycli"), "PKGBUILD");
    }
}
