pub struct AsdfMiseArtifacts {
    pub list_all_sh: String,
    pub download_sh: String,
    pub install_sh: String,
}

#[allow(clippy::too_many_arguments)]
pub fn generate_asdf_mise_plugin(
    app_name: &str,
    repo: &str,
) -> AsdfMiseArtifacts {
    let clean_repo = repo
        .trim_start_matches("https://github.com/")
        .trim_start_matches("http://github.com/")
        .trim_end_matches(".git");

    let list_all_sh = format!(
        r#"#!/usr/bin/env bash
set -euo pipefail

REPO="{clean_repo}"
git ls-remote --tags --refs "https://github.com/${{REPO}}" |
    awk -F/ '{{print $NF}}' |
    sed -e 's/^v//' |
    sort -V
"#
    );

    let download_sh = format!(
        r#"#!/usr/bin/env bash
set -euo pipefail

APP_NAME="{app_name}"
REPO="{clean_repo}"

VERSION="${{ASDF_INSTALL_VERSION:-${{1}}}}"
DOWNLOAD_PATH="${{ASDF_DOWNLOAD_PATH:-${{2}}}}"

OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"

case "$ARCH" in
    x86_64|amd64) ARCH="amd64" ;;
    aarch64|arm64) ARCH="arm64" ;;
    *) echo "Unsupported architecture $ARCH" >&2; exit 1 ;;
esac

ARCHIVE_NAME="${{APP_NAME}}_${{VERSION}}_${{OS}}_${{ARCH}}.tar.gz"
URL="https://github.com/${{REPO}}/releases/download/v${{VERSION}}/${{ARCHIVE_NAME}}"

mkdir -p "$DOWNLOAD_PATH"
echo "Downloading $URL to $DOWNLOAD_PATH..."
curl -fsSL "$URL" -o "${{DOWNLOAD_PATH}}/${{ARCHIVE_NAME}}"

CHECKSUMS_URL="https://github.com/${{REPO}}/releases/download/v${{VERSION}}/checksums.txt"
curl -fsSL "$CHECKSUMS_URL" -o "${{DOWNLOAD_PATH}}/checksums.txt" 2>/dev/null || true
if [ -f "${{DOWNLOAD_PATH}}/checksums.txt" ] && command -v sha256sum >/dev/null 2>&1; then
    EXPECTED="$(awk -v a="${{ARCHIVE_NAME}}" '($2 == a || $2 == "*"a) {{print $1; exit}}' "${{DOWNLOAD_PATH}}/checksums.txt")"
    if [ -n "$EXPECTED" ]; then
        ACTUAL="$(sha256sum "${{DOWNLOAD_PATH}}/${{ARCHIVE_NAME}}" | awk '{{print $1}}')"
        if [ "$ACTUAL" != "$EXPECTED" ]; then
            echo "Checksum verification failed! Expected: $EXPECTED, got: $ACTUAL" >&2
            exit 1
        fi
        echo "Checksum verified."
    fi
fi

tar -xzf "${{DOWNLOAD_PATH}}/${{ARCHIVE_NAME}}" -C "$DOWNLOAD_PATH"
"#
    );

    let install_sh = format!(
        r#"#!/usr/bin/env bash
set -euo pipefail

APP_NAME="{app_name}"
INSTALL_TYPE="${{ASDF_INSTALL_TYPE:-version}}"
VERSION="${{ASDF_INSTALL_VERSION:-${{1}}}}"
INSTALL_PATH="${{ASDF_INSTALL_PATH:-${{2}}}}"
DOWNLOAD_PATH="${{ASDF_DOWNLOAD_PATH:-${{INSTALL_PATH}}}}"

mkdir -p "${{INSTALL_PATH}}/bin"
cp "${{DOWNLOAD_PATH}}/${{APP_NAME}}" "${{INSTALL_PATH}}/bin/${{APP_NAME}}"
chmod +x "${{INSTALL_PATH}}/bin/${{APP_NAME}}"
echo "${{APP_NAME}} ${{VERSION}} installed to ${{INSTALL_PATH}}/bin/${{APP_NAME}}"
"#
    );

    AsdfMiseArtifacts {
        list_all_sh,
        download_sh,
        install_sh,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asdf_mise_plugin_generation() {
        let plugin = generate_asdf_mise_plugin("ripgrep", "https://github.com/BurntSushi/ripgrep");
        assert!(plugin.list_all_sh.contains("REPO=\"BurntSushi/ripgrep\""));
        assert!(plugin.download_sh.contains("APP_NAME=\"ripgrep\""));
        assert!(plugin.download_sh.contains("tar -xzf"));
        assert!(plugin.install_sh.contains("chmod +x \"${INSTALL_PATH}/bin/${APP_NAME}\""));
    }
}
