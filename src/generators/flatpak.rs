#[allow(clippy::too_many_arguments)]
pub fn generate_flatpak_manifest(
    app_name: &str,
    version: &str,
    description: &str,
    repo: &str,
    x64_hash: Option<&str>,
    arm64_hash: Option<&str>,
) -> String {
    let clean_repo = repo
        .trim_start_matches("https://github.com/")
        .trim_start_matches("http://github.com/")
        .trim_end_matches(".git");
    let clean_v = version.trim_start_matches('v');

    let x64_h = x64_hash.unwrap_or("0000000000000000000000000000000000000000000000000000000000000000");
    let arm64_h = arm64_hash.unwrap_or("0000000000000000000000000000000000000000000000000000000000000000");

    format!(
        r#"# {description}
app-id: org.system_releaser.{app_name}
runtime: org.freedesktop.Platform
runtime-version: '23.08'
sdk: org.freedesktop.Sdk
command: {app_name}

finish-args:
  - --share=network
  - --share=ipc
  - --filesystem=host

modules:
  - name: {app_name}
    buildsystem: simple
    build-commands:
      - install -Dm755 {app_name} /app/bin/{app_name}
    sources:
      - type: archive
        only-arches:
          - x86_64
        url: https://github.com/{clean_repo}/releases/download/v{clean_v}/{app_name}_{clean_v}_linux_amd64.tar.gz
        sha256: {x64_h}
      - type: archive
        only-arches:
          - aarch64
        url: https://github.com/{clean_repo}/releases/download/v{clean_v}/{app_name}_{clean_v}_linux_arm64.tar.gz
        sha256: {arm64_h}
"#
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flatpak_manifest_generation() {
        let manifest = generate_flatpak_manifest(
            "app-viewer",
            "1.4.2",
            "A modern viewer app",
            "https://github.com/org/app-viewer",
            Some("flatpak_sha_64"),
            Some("flatpak_sha_arm"),
        );

        assert!(manifest.contains("app-id: org.system_releaser.app-viewer"));
        assert!(manifest.contains("command: app-viewer"));
        assert!(manifest.contains("flatpak_sha_64"));
        assert!(manifest.contains("install -Dm755 app-viewer /app/bin/app-viewer"));
    }
}
