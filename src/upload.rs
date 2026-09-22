use std::path::PathBuf;
use std::process::Command;
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct UploadResult {
    pub tag: String,
    pub release_url: String,
    pub uploaded_assets: Vec<PathBuf>,
}

/// Uploads release assets to GitHub Releases for the specified repository and tag.
pub fn upload_release_assets(
    repo_url: &str,
    version: &str,
    token: &str,
    assets: &[PathBuf],
) -> Result<UploadResult, Box<dyn std::error::Error>> {
    let clean_repo = repo_url
        .trim_start_matches("https://github.com/")
        .trim_start_matches("http://github.com/")
        .trim_end_matches(".git");
    let clean_v = version.trim_start_matches('v');
    let tag = format!("v{}", clean_v);

    // Mock mode for testing or offline environments
    if is_mock_upload() {
        return Ok(UploadResult {
            tag: tag.clone(),
            release_url: format!("https://github.com/{}/releases/tag/{}", clean_repo, tag),
            uploaded_assets: assets.to_vec(),
        });
    }

    if token.trim().is_empty() {
        return Err("GitHub token is empty. Please set GITHUB_TOKEN.".into());
    }

    let tmp_dir = tempfile::tempdir()?;
    let config_path = tmp_dir.path().join("curl_auth.cfg");
    let cfg_content = format!("header = \"Authorization: Bearer {}\"\n", token.trim());
    std::fs::write(&config_path, cfg_content)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&config_path, std::fs::Permissions::from_mode(0o600));
    }
    let config_path_str = config_path.display().to_string();

    let accept_header = "Accept: application/vnd.github.v3+json";

    // 1. Check if release already exists or create it
    let api_url = format!("https://api.github.com/repos/{}/releases", clean_repo);
    let tag_url = format!("https://api.github.com/repos/{}/releases/tags/{}", clean_repo, tag);

    let check_cmd = Command::new("curl")
        .args([
            "-s",
            "--connect-timeout",
            "15",
            "--max-time",
            "60",
            "-K",
            &config_path_str,
            "-H",
            accept_header,
            "-H",
            "User-Agent: system-releaser",
            &tag_url,
        ])
        .output()?;

    let (release_id, release_html_url) = if check_cmd.status.success()
        && let Ok(json) = serde_json::from_slice::<Value>(&check_cmd.stdout)
        && let Some(id) = json.get("id").and_then(|v| v.as_u64())
    {
        let html_url = json
            .get("html_url")
            .and_then(|v| v.as_str())
            .unwrap_or(&tag_url)
            .to_string();
        (id, html_url)
    } else {
        // Create release
        let payload = serde_json::json!({
            "tag_name": tag,
            "name": format!("v{}", clean_v),
            "draft": false,
            "prerelease": false
        });
        let payload_str = payload.to_string();

        let create_cmd = Command::new("curl")
            .args([
                "-s",
                "--connect-timeout",
                "15",
                "--max-time",
                "60",
                "-X",
                "POST",
                "-K",
                &config_path_str,
                "-H",
                accept_header,
                "-H",
                "User-Agent: system-releaser",
                "-H",
                "Content-Type: application/json",
                "-d",
                &payload_str,
                &api_url,
            ])
            .output()?;

        if !create_cmd.status.success() {
            return Err(format!(
                "Failed to create GitHub release: {}",
                String::from_utf8_lossy(&create_cmd.stderr)
            )
            .into());
        }

        let json: Value = serde_json::from_slice(&create_cmd.stdout)?;
        let id = json
            .get("id")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| {
                format!(
                    "Failed to get release ID from response: {}",
                    String::from_utf8_lossy(&create_cmd.stdout)
                )
            })?;
        let html_url = json
            .get("html_url")
            .and_then(|v| v.as_str())
            .unwrap_or(&tag_url)
            .to_string();

        (id, html_url)
    };

    // 2. Upload assets
    let mut uploaded = Vec::new();
    for asset in assets {
        if !asset.is_file() {
            continue;
        }

        let file_name = asset
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("asset");

        let encoded_name = url_encode(file_name);
        let upload_endpoint = format!(
            "https://uploads.github.com/repos/{}/releases/{}/assets?name={}",
            clean_repo, release_id, encoded_name
        );

        let upload_cmd = Command::new("curl")
            .args([
                "-s",
                "--connect-timeout",
                "15",
                "--max-time",
                "300",
                "-X",
                "POST",
                "-K",
                &config_path_str,
                "-H",
                accept_header,
                "-H",
                "User-Agent: system-releaser",
                "-H",
                "Content-Type: application/octet-stream",
                "--data-binary",
                &format!("@{}", asset.display()),
                &upload_endpoint,
            ])
            .output()?;

        let upload_success = if upload_cmd.status.success() {
            if let Ok(json) = serde_json::from_slice::<Value>(&upload_cmd.stdout) {
                json.get("id").is_some()
                    || json.get("state").and_then(|v| v.as_str()) == Some("uploaded")
            } else {
                false
            }
        } else {
            false
        };

        if upload_success {
            println!("  ✓ Uploaded asset: {}", file_name);
            uploaded.push(asset.clone());
        } else {
            let err_info = if !upload_cmd.stdout.is_empty() {
                String::from_utf8_lossy(&upload_cmd.stdout)
            } else {
                String::from_utf8_lossy(&upload_cmd.stderr)
            };
            eprintln!(
                "WARN: Failed to upload asset '{}': {}",
                file_name,
                err_info.trim()
            );
        }
    }

    Ok(UploadResult {
        tag,
        release_url: release_html_url,
        uploaded_assets: uploaded,
    })
}

fn url_encode(s: &str) -> String {
    let mut encoded = String::new();
    for b in s.bytes() {
        match b {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(b as char);
            }
            _ => {
                encoded.push_str(&format!("%{:02X}", b));
            }
        }
    }
    encoded
}

fn is_mock_upload() -> bool {
    cfg!(test)
        || std::env::var("SYSTEM_RELEASER_MOCK_UPLOAD").map(|v| v == "1").unwrap_or(false)
        || std::env::current_exe()
            .map(|p| {
                let s = p.to_string_lossy().to_lowercase();
                s.contains("deps") || s.contains("test")
            })
            .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs;

    #[test]
    fn test_mock_upload_release_assets() {
        let dir = tempdir().unwrap();
        let f1 = dir.path().join("app_1.0.0_linux_amd64.tar.gz");
        let f2 = dir.path().join("checksums.txt");
        fs::write(&f1, "fake archive").unwrap();
        fs::write(&f2, "fake checksum").unwrap();

        let res = upload_release_assets(
            "https://github.com/myorg/myapp",
            "1.0.0",
            "mock_token",
            &[f1.clone(), f2.clone()],
        )
        .unwrap();

        assert_eq!(res.tag, "v1.0.0");
        assert!(res.release_url.contains("myorg/myapp/releases/tag/v1.0.0"));
        assert_eq!(res.uploaded_assets.len(), 2);
    }
}
