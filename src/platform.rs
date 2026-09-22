use std::fmt;
use std::str::FromStr;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OS {
    Windows,
    Darwin,
    Linux,
}

impl fmt::Display for OS {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OS::Windows => write!(f, "windows"),
            OS::Darwin => write!(f, "darwin"),
            OS::Linux => write!(f, "linux"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Arch {
    Amd64, // x86_64
    Arm64, // aarch64
}

impl fmt::Display for Arch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Arch::Amd64 => write!(f, "amd64"),
            Arch::Arm64 => write!(f, "arm64"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TargetPlatform {
    pub os: OS,
    pub arch: Arch,
}

impl TargetPlatform {
    pub fn new(os: OS, arch: Arch) -> Self {
        Self { os, arch }
    }

    /// Rust compiler target triple.
    pub fn rust_triple(&self) -> &'static str {
        match (self.os, self.arch) {
            (OS::Windows, Arch::Amd64) => "x86_64-pc-windows-msvc",
            (OS::Windows, Arch::Arm64) => "aarch64-pc-windows-msvc",
            (OS::Darwin, Arch::Amd64) => "x86_64-apple-darwin",
            (OS::Darwin, Arch::Arm64) => "aarch64-apple-darwin",
            (OS::Linux, Arch::Amd64) => "x86_64-unknown-linux-musl",
            (OS::Linux, Arch::Arm64) => "aarch64-unknown-linux-musl",
        }
    }

    /// Go GOOS and GOARCH.
    pub fn go_env(&self) -> (&'static str, &'static str) {
        let goos = match self.os {
            OS::Windows => "windows",
            OS::Darwin => "darwin",
            OS::Linux => "linux",
        };
        let goarch = match self.arch {
            Arch::Amd64 => "amd64",
            Arch::Arm64 => "arm64",
        };
        (goos, goarch)
    }

    /// .NET Runtime Identifier (RID).
    pub fn dotnet_rid(&self) -> &'static str {
        match (self.os, self.arch) {
            (OS::Windows, Arch::Amd64) => "win-x64",
            (OS::Windows, Arch::Arm64) => "win-arm64",
            (OS::Darwin, Arch::Amd64) => "osx-x64",
            (OS::Darwin, Arch::Arm64) => "osx-arm64",
            (OS::Linux, Arch::Amd64) => "linux-x64",
            (OS::Linux, Arch::Arm64) => "linux-arm64",
        }
    }

    /// Standard executable filename on this target.
    pub fn binary_name(&self, base_name: &str) -> String {
        if self.os == OS::Windows {
            format!("{}.exe", base_name)
        } else {
            base_name.to_string()
        }
    }

    /// Standard archive file extension (.zip for Windows, .tar.gz for macOS/Linux).
    pub fn archive_extension(&self) -> &'static str {
        if self.os == OS::Windows {
            "zip"
        } else {
            "tar.gz"
        }
    }

    /// Formatted standard release archive name: `{app}_{version}_{os}_{arch}.{ext}`
    pub fn archive_name(&self, app_name: &str, version: &str) -> String {
        let clean_v = version.trim_start_matches('v');
        format!("{}_{}_{}_{}.{}", app_name, clean_v, self.os, self.arch, self.archive_extension())
    }

    /// Default 6 standard release platforms (Windows, macOS, Linux x64 + ARM64).
    pub fn all_standard() -> Vec<TargetPlatform> {
        vec![
            TargetPlatform::new(OS::Linux, Arch::Amd64),
            TargetPlatform::new(OS::Linux, Arch::Arm64),
            TargetPlatform::new(OS::Darwin, Arch::Amd64),
            TargetPlatform::new(OS::Darwin, Arch::Arm64),
            TargetPlatform::new(OS::Windows, Arch::Amd64),
            TargetPlatform::new(OS::Windows, Arch::Arm64),
        ]
    }
}

impl fmt::Display for TargetPlatform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.os, self.arch)
    }
}

impl FromStr for TargetPlatform {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let lower = s.to_lowercase();
        let parts: Vec<&str> = lower.split('/').collect();
        if parts.len() != 2 {
            // Try hyphen separation e.g. linux-x64, windows-amd64
            let h_parts: Vec<&str> = lower.split('-').collect();
            if h_parts.len() == 2 {
                return parse_os_arch(h_parts[0], h_parts[1]);
            }
            return Err(format!("Invalid platform format '{}'. Expected 'os/arch' like 'linux/amd64'", s));
        }

        parse_os_arch(parts[0], parts[1])
    }
}

fn parse_os_arch(os_str: &str, arch_str: &str) -> Result<TargetPlatform, String> {
    let os = match os_str {
        "windows" | "win" => OS::Windows,
        "darwin" | "mac" | "macos" | "apple" => OS::Darwin,
        "linux" => OS::Linux,
        other => return Err(format!("Unsupported OS '{}'", other)),
    };

    let arch = match arch_str {
        "amd64" | "x86_64" | "x64" => Arch::Amd64,
        "arm64" | "aarch64" => Arch::Arm64,
        other => return Err(format!("Unsupported architecture '{}'", other)),
    };

    Ok(TargetPlatform::new(os, arch))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_target_platform_triples_and_names() {
        let win_x64 = TargetPlatform::new(OS::Windows, Arch::Amd64);
        assert_eq!(win_x64.rust_triple(), "x86_64-pc-windows-msvc");
        assert_eq!(win_x64.binary_name("mycli"), "mycli.exe");
        assert_eq!(win_x64.archive_name("mycli", "1.0.0"), "mycli_1.0.0_windows_amd64.zip");

        let mac_arm = TargetPlatform::new(OS::Darwin, Arch::Arm64);
        assert_eq!(mac_arm.rust_triple(), "aarch64-apple-darwin");
        assert_eq!(mac_arm.binary_name("mycli"), "mycli");
        assert_eq!(mac_arm.archive_name("mycli", "v1.0.0"), "mycli_1.0.0_darwin_arm64.tar.gz");

        let linux_x64 = TargetPlatform::new(OS::Linux, Arch::Amd64);
        assert_eq!(linux_x64.rust_triple(), "x86_64-unknown-linux-musl");
        assert_eq!(linux_x64.archive_name("mycli", "1.0.0"), "mycli_1.0.0_linux_amd64.tar.gz");
    }

    #[test]
    fn test_parse_platforms() {
        assert_eq!("linux/amd64".parse::<TargetPlatform>().unwrap(), TargetPlatform::new(OS::Linux, Arch::Amd64));
        assert_eq!("macos/arm64".parse::<TargetPlatform>().unwrap(), TargetPlatform::new(OS::Darwin, Arch::Arm64));
        assert_eq!("win-x64".parse::<TargetPlatform>().unwrap(), TargetPlatform::new(OS::Windows, Arch::Amd64));
    }
}
