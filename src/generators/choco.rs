pub struct ChocoArtifacts {
    pub nuspec: String,
    pub install_ps1: String,
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[allow(clippy::too_many_arguments)]
pub fn generate_choco_artifacts(
    app_name: &str,
    version: &str,
    description: &str,
    author: &str,
    repo: &str,
    homepage: &str,
    x64_hash: Option<&str>,
    arm64_hash: Option<&str>,
) -> ChocoArtifacts {
    let clean_repo = repo
        .trim_start_matches("https://github.com/")
        .trim_start_matches("http://github.com/")
        .trim_end_matches(".git");
    let clean_v = version.trim_start_matches('v');
    let x64_h = x64_hash.unwrap_or("TODO_SHA256_WINDOWS_AMD64");
    let arm64_h = arm64_hash.unwrap_or("");

    let clean_app_xml = escape_xml(app_name);
    let clean_desc_xml = escape_xml(description);
    let clean_author_xml = escape_xml(author);
    let clean_homepage_xml = escape_xml(homepage);
    let clean_app_ps1 = app_name.replace('\'', "''");

    let nuspec = format!(r#"<?xml version="1.0" encoding="utf-8"?>
<package xmlns="http://schemas.microsoft.com/packaging/2015/06/nuspec.xsd">
  <metadata>
    <id>{clean_app_xml}</id>
    <version>{clean_v}</version>
    <title>{clean_app_xml}</title>
    <authors>{clean_author_xml}</authors>
    <projectUrl>{clean_homepage_xml}</projectUrl>
    <licenseUrl>https://github.com/{clean_repo}/blob/main/LICENSE</licenseUrl>
    <requireLicenseAcceptance>false</requireLicenseAcceptance>
    <description>{clean_desc_xml}</description>
    <tags>{clean_app_xml} cli cross-platform</tags>
  </metadata>
  <files>
    <file src="tools\**" target="tools" />
  </files>
</package>
"#);

    let install_ps1 = format!(r#"$ErrorActionPreference = 'Stop'
$toolsDir = "$(Split-Path -parent $MyInvocation.MyCommand.Definition)"
$url64 = "https://github.com/{clean_repo}/releases/download/v{clean_v}/{app_name}_{clean_v}_windows_amd64.zip"
$urlArm64 = "https://github.com/{clean_repo}/releases/download/v{clean_v}/{app_name}_{clean_v}_windows_arm64.zip"

$isArm64 = [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture -eq [System.Runtime.InteropServices.Architecture]::Arm64
$url = if ($isArm64 -and '{arm64_h}' -ne '') {{ $urlArm64 }} else {{ $url64 }}
$checksum = if ($isArm64 -and '{arm64_h}' -ne '') {{ '{arm64_h}' }} else {{ '{x64_h}' }}

$packageArgs = @{{
  packageName   = '{clean_app_ps1}'
  unzipLocation = $toolsDir
  fileType      = 'exe'
  url           = $url
  url64         = $url64
  checksum      = $checksum
  checksum64    = '{x64_h}'
  checksumType  = 'sha256'
  checksumType64= 'sha256'
}}

Install-ChocolateyZipPackage @packageArgs
"#);

    ChocoArtifacts {
        nuspec,
        install_ps1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_choco_artifacts_generation() {
        let artifacts = generate_choco_artifacts(
            "my-tool",
            "1.0.0",
            "Fast CLI",
            "Alice",
            "https://github.com/alice/my-tool",
            "https://mytool.dev",
            Some("sha64"),
            Some("sha_arm64"),
        );

        assert!(artifacts.nuspec.contains("<id>my-tool</id>"));
        assert!(artifacts.install_ps1.contains("checksum64    = 'sha64'"));
    }
}
