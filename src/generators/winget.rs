#[allow(clippy::too_many_arguments)]
pub fn generate_winget_manifest(
    publisher: &str,
    app_name: &str,
    version: &str,
    description: &str,
    license: &str,
    repo: &str,
    x64_hash: Option<&str>,
    arm64_hash: Option<&str>,
) -> String {
    let clean_repo = repo
        .trim_start_matches("https://github.com/")
        .trim_start_matches("http://github.com/")
        .trim_end_matches(".git");
    let clean_v = version.trim_start_matches('v');
    let x64_h = x64_hash.unwrap_or("TODO_SHA256_WINDOWS_AMD64");
    let arm64_h = arm64_hash.unwrap_or("TODO_SHA256_WINDOWS_ARM64");
    let clean_desc = description.replace('\n', " ").replace('"', "\\\"");

    format!(r#"# yaml-language-server: $schema=https://aka.ms/winget-manifest.singleton.1.6.0.schema.json
PackageIdentifier: {publisher}.{app_name}
PackageVersion: {clean_v}
PackageName: {app_name}
Publisher: {publisher}
License: {license}
ShortDescription: "{clean_desc}"
Installers:
  - Architecture: x64
    InstallerType: zip
    InstallerUrl: https://github.com/{clean_repo}/releases/download/v{clean_v}/{app_name}_{clean_v}_windows_amd64.zip
    InstallerSha256: {x64_h}
    NestedInstallerType: portable
    NestedInstallerFiles:
      - RelativeFilePath: {app_name}.exe
        PortableCommandAlias: {app_name}
  - Architecture: arm64
    InstallerType: zip
    InstallerUrl: https://github.com/{clean_repo}/releases/download/v{clean_v}/{app_name}_{clean_v}_windows_arm64.zip
    InstallerSha256: {arm64_h}
    NestedInstallerType: portable
    NestedInstallerFiles:
      - RelativeFilePath: {app_name}.exe
        PortableCommandAlias: {app_name}
ManifestType: singleton
ManifestVersion: 1.6.0
"#)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_winget_manifest_generation() {
        let manifest = generate_winget_manifest(
            "MyOrg",
            "my-tool",
            "1.0.0",
            "Sample CLI",
            "MIT",
            "https://github.com/myorg/my-tool",
            Some("hash_64"),
            Some("hash_arm"),
        );

        assert!(manifest.contains("PackageIdentifier: MyOrg.my-tool"));
        assert!(manifest.contains("InstallerSha256: hash_64"));
        assert!(manifest.contains("PortableCommandAlias: my-tool"));
    }
}
