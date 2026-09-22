pub struct AppImageArtifacts {
    pub desktop_file: String,
    pub apprun_script: String,
    pub recipe_yaml: String,
}

#[allow(clippy::too_many_arguments)]
pub fn generate_appimage_artifacts(
    app_name: &str,
    version: &str,
    description: &str,
    repo: &str,
) -> AppImageArtifacts {
    let clean_repo = repo
        .trim_start_matches("https://github.com/")
        .trim_start_matches("http://github.com/")
        .trim_end_matches(".git");
    let clean_v = version.trim_start_matches('v');

    let desktop_file = format!(
        r#"[Desktop Entry]
Type=Application
Name={app_name}
Exec={app_name}
Icon={app_name}
Comment={description}
Categories=Utility;
Terminal=true
"#
    );

    let apprun_script = format!(
        r#"#!/bin/sh
HERE="$(dirname "$(readlink -f "${{0}}")")"
exec "${{HERE}}/usr/bin/{app_name}" "$@"
"#
    );

    let recipe_yaml = format!(
        r#"app: {app_name}
version: {clean_v}

ingredients:
  dist: focal
  sources:
    - deb http://archive.ubuntu.com/ubuntu/ focal main universe

script:
  - ARCH="${{ARCH:-amd64}}"
  - case "$ARCH" in x86_64) ARCH="amd64" ;; aarch64) ARCH="arm64" ;; esac
  - wget -qO- "https://github.com/{clean_repo}/releases/download/v{clean_v}/{app_name}_{clean_v}_linux_${{ARCH}}.tar.gz" | tar -xz -C "${{APPDIR}}/usr/bin/"
  - chmod +x "${{APPDIR}}/usr/bin/{app_name}"
"#
    );

    AppImageArtifacts {
        desktop_file,
        apprun_script,
        recipe_yaml,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_appimage_artifacts_generation() {
        let art = generate_appimage_artifacts(
            "my-editor",
            "3.0.1",
            "Terminal Editor",
            "https://github.com/myorg/my-editor",
        );

        assert!(art.desktop_file.contains("Name=my-editor"));
        assert!(art.desktop_file.contains("Exec=my-editor"));
        assert!(art.apprun_script.contains("/usr/bin/my-editor"));
        assert!(art.recipe_yaml.contains("app: my-editor"));
        assert!(art.recipe_yaml.contains("version: 3.0.1"));
    }
}
