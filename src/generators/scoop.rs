use serde_json::json;

#[allow(clippy::too_many_arguments)]
pub fn generate_scoop_manifest(
    app_name: &str,
    version: &str,
    description: &str,
    homepage: &str,
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

    let manifest = json!({
        "version": clean_v,
        "description": description,
        "homepage": homepage,
        "license": license,
        "architecture": {
            "64bit": {
                "url": format!("https://github.com/{}/releases/download/v{}/{}_{}_windows_amd64.zip", clean_repo, clean_v, app_name, clean_v),
                "hash": x64_h
            },
            "arm64": {
                "url": format!("https://github.com/{}/releases/download/v{}/{}_{}_windows_arm64.zip", clean_repo, clean_v, app_name, clean_v),
                "hash": arm64_h
            }
        },
        "bin": format!("{}.exe", app_name),
        "checkver": "github",
        "autoupdate": {
            "architecture": {
                "64bit": {
                    "url": format!("https://github.com/{}/releases/download/v$version/{}_$version_windows_amd64.zip", clean_repo, app_name)
                },
                "arm64": {
                    "url": format!("https://github.com/{}/releases/download/v$version/{}_$version_windows_arm64.zip", clean_repo, app_name)
                }
            }
        }
    });

    serde_json::to_string_pretty(&manifest).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scoop_manifest_generation() {
        let json_str = generate_scoop_manifest(
            "my-tool",
            "1.0.0",
            "Scoop test tool",
            "https://mytool.dev",
            "MIT",
            "https://github.com/myorg/my-tool",
            Some("h64"),
            Some("harm"),
        );

        assert!(json_str.contains("\"version\": \"1.0.0\""));
        assert!(json_str.contains("\"bin\": \"my-tool.exe\""));
        assert!(json_str.contains("h64"));
    }
}
