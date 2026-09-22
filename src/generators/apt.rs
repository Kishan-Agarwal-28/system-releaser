pub struct AptArtifacts {
    pub control: String,
    pub install_script: String,
}

#[allow(clippy::too_many_arguments)]
pub fn generate_apt_artifacts(
    app_name: &str,
    version: &str,
    description: &str,
    maintainer: &str,
    homepage: &str,
    repo: &str,
) -> AptArtifacts {
    let clean_repo = repo
        .trim_start_matches("https://github.com/")
        .trim_start_matches("http://github.com/")
        .trim_end_matches(".git");
    let clean_v = version.trim_start_matches('v');

    let control = format!(
        r#"Package: {app_name}
Version: {clean_v}
Section: utils
Priority: optional
Architecture: amd64 arm64
Maintainer: {maintainer}
Homepage: {homepage}
Description: {description}
"#
    );

    let install_script = format!(
        r#"#!/bin/sh
set -e

APP_NAME="{app_name}"
VERSION="{clean_v}"
REPO="{clean_repo}"

ARCH="$(dpkg --print-architecture 2>/dev/null || uname -m)"
case "$ARCH" in
    amd64|x86_64)
        DEB_ARCH="amd64"
        ;;
    arm64|aarch64)
        DEB_ARCH="arm64"
        ;;
    *)
        echo "Unsupported architecture: $ARCH" >&2
        exit 1
        ;;
esac

TAR_FILE="${{APP_NAME}}_${{VERSION}}_linux_${{DEB_ARCH}}.tar.gz"
DOWNLOAD_URL="https://github.com/${{REPO}}/releases/download/v${{VERSION}}/${{TAR_FILE}}"
CHECKSUMS_URL="https://github.com/${{REPO}}/releases/download/v${{VERSION}}/checksums.txt"

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

echo "Downloading $DOWNLOAD_URL..."
curl -fsSL "$DOWNLOAD_URL" -o "$TMP_DIR/$TAR_FILE"
curl -fsSL "$CHECKSUMS_URL" -o "$TMP_DIR/checksums.txt"

if command -v sha256sum >/dev/null 2>&1; then
    EXPECTED="$(awk -v a="$TAR_FILE" '($2 == a || $2 == "*"a) {{print $1; exit}}' "$TMP_DIR/checksums.txt")"
    if [ -n "$EXPECTED" ]; then
        ACTUAL="$(sha256sum "$TMP_DIR/$TAR_FILE" | awk '{{print $1}}')"
        if [ "$ACTUAL" != "$EXPECTED" ]; then
            echo "Checksum verification failed! Expected: $EXPECTED, got: $ACTUAL" >&2
            exit 1
        fi
        echo "Checksum verified successfully."
    fi
fi

mkdir -p "$TMP_DIR/deb/DEBIAN" "$TMP_DIR/deb/usr/bin"
tar -xzf "$TMP_DIR/$TAR_FILE" -C "$TMP_DIR/deb/usr/bin"
chmod 755 "$TMP_DIR/deb/usr/bin/$APP_NAME"

cat << 'EOF' > "$TMP_DIR/deb/DEBIAN/control"
{control}
EOF

if command -v dpkg-deb >/dev/null 2>&1; then
    dpkg-deb --build "$TMP_DIR/deb" "$TMP_DIR/${{APP_NAME}}.deb"
    sudo dpkg -i "$TMP_DIR/${{APP_NAME}}.deb" || sudo apt-get install -f -y
else
    sudo cp "$TMP_DIR/deb/usr/bin/$APP_NAME" /usr/local/bin/
fi

echo "$APP_NAME v$VERSION installed successfully!"
"#
    );

    AptArtifacts {
        control,
        install_script,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apt_artifacts_generation() {
        let apt = generate_apt_artifacts(
            "my-tool",
            "1.2.0",
            "A fast command line utility",
            "Alice <alice@example.com>",
            "https://mytool.dev",
            "https://github.com/myorg/my-tool",
        );

        assert!(apt.control.contains("Package: my-tool"));
        assert!(apt.control.contains("Version: 1.2.0"));
        assert!(apt.control.contains("Maintainer: Alice <alice@example.com>"));
        assert!(apt.install_script.contains("APP_NAME=\"my-tool\""));
        assert!(apt.install_script.contains("dpkg -i"));
    }
}
