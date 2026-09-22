/// Generates a POSIX-compliant multi-architecture installation shell script
/// for `curl -fsSL ... | sh` and `wget -qO- ... | sh`.
pub fn generate_install_sh(
    app_name: &str,
    repo: Option<&str>,
    install_dir: &str,
) -> String {
    let repo_url = repo.unwrap_or("https://github.com/user/repo");
    let clean_repo = repo_url
        .trim_start_matches("https://github.com/")
        .trim_start_matches("http://github.com/");
    let clean_app = app_name
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('$', "\\$")
        .replace('`', "\\`");
    let clean_repo_esc = clean_repo
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('$', "\\$")
        .replace('`', "\\`");
    let clean_install_dir = install_dir
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('$', "\\$")
        .replace('`', "\\`");

    format!(r#"#!/bin/sh
set -e

APP_NAME="{clean_app}"
GITHUB_REPO="{clean_repo_esc}"
INSTALL_DIR="{clean_install_dir}"

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

info() {{
    printf "${{BLUE}}info:${{NC}} %s\n" "$1"
}}

success() {{
    printf "${{GREEN}}success:${{NC}} %s\n" "$1"
}}

error() {{
    printf "${{RED}}error:${{NC}} %s\n" "$1" >&2
    exit 1
}}

# 1. Detect OS
OS="$(uname -s)"
case "$OS" in
    Linux)
        TARGET_OS="linux"
        ;;
    Darwin)
        TARGET_OS="darwin"
        ;;
    *)
        error "Unsupported operating system: $OS"
        ;;
esac

# 2. Detect Architecture
ARCH="$(uname -m)"
case "$ARCH" in
    x86_64|amd64)
        TARGET_ARCH="amd64"
        ;;
    aarch64|arm64)
        TARGET_ARCH="arm64"
        ;;
    *)
        error "Unsupported architecture: $ARCH"
        ;;
esac

info "Installing ${{APP_NAME}} for ${{TARGET_OS}}/${{TARGET_ARCH}}..."

# 3. Determine latest version from GitHub API
RELEASE_URL="https://api.github.com/repos/${{GITHUB_REPO}}/releases/latest"
if command -v curl >/dev/null 2>&1; then
    TAG="$(curl -fsSL "${{RELEASE_URL}}" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')"
elif command -v wget >/dev/null 2>&1; then
    TAG="$(wget -qO- "${{RELEASE_URL}}" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')"
else
    error "Neither curl nor wget found. Please install either tool to proceed."
fi

if [ -z "$TAG" ]; then
    error "Failed to retrieve latest release tag from ${{RELEASE_URL}}"
fi

VERSION="$(echo "$TAG" | sed 's/^v//')"
info "Latest release: ${{TAG}}"

ARCHIVE_NAME="${{APP_NAME}}_${{VERSION}}_${{TARGET_OS}}_${{TARGET_ARCH}}.tar.gz"
DOWNLOAD_URL="https://github.com/${{GITHUB_REPO}}/releases/download/${{TAG}}/${{ARCHIVE_NAME}}"

TMP_DIR="$(mktemp -d 2>/dev/null || mktemp -d -t 'sysrel')"
cleanup() {{
    rm -rf "$TMP_DIR"
}}
trap cleanup EXIT

info "Downloading ${{DOWNLOAD_URL}}..."
if command -v curl >/dev/null 2>&1; then
    curl -fsSL "$DOWNLOAD_URL" -o "${{TMP_DIR}}/${{ARCHIVE_NAME}}"
else
    wget -qO "${{TMP_DIR}}/${{ARCHIVE_NAME}}" "$DOWNLOAD_URL"
fi

# Verify SHA-256 checksum
CHECKSUMS_URL="https://github.com/${{GITHUB_REPO}}/releases/download/${{TAG}}/checksums.txt"
info "Verifying SHA-256 checksum..."
if command -v curl >/dev/null 2>&1; then
    curl -fsSL "$CHECKSUMS_URL" -o "${{TMP_DIR}}/checksums.txt"
else
    wget -qO "${{TMP_DIR}}/checksums.txt" "$CHECKSUMS_URL"
fi

EXPECTED="$(awk -v a="${{ARCHIVE_NAME}}" '($2 == a || $2 == "*"a) {{print $1; exit}}' "${{TMP_DIR}}/checksums.txt")"
if [ -z "$EXPECTED" ]; then
    EXPECTED="$(grep -E "[[:space:]]\*?${{ARCHIVE_NAME}}$" "${{TMP_DIR}}/checksums.txt" | awk '{{print $1}}' | head -n 1)"
fi

if command -v sha256sum >/dev/null 2>&1; then
    ACTUAL="$(sha256sum "${{TMP_DIR}}/${{ARCHIVE_NAME}}" | awk '{{print $1}}')"
elif command -v shasum >/dev/null 2>&1; then
    ACTUAL="$(shasum -a 256 "${{TMP_DIR}}/${{ARCHIVE_NAME}}" | awk '{{print $1}}')"
else
    error "Neither sha256sum nor shasum found — cannot verify download integrity"
fi

if [ -z "$EXPECTED" ]; then
    error "Checksum for ${{ARCHIVE_NAME}} not found in checksums.txt"
fi
if [ "$ACTUAL" != "$EXPECTED" ]; then
    error "Checksum mismatch! Expected: $EXPECTED  Got: $ACTUAL — aborting for security."
fi
success "Checksum verified."

info "Extracting binary..."
tar -xzf "${{TMP_DIR}}/${{ARCHIVE_NAME}}" -C "$TMP_DIR"

# Install target directory resolution
if [ ! -d "$INSTALL_DIR" ]; then
    if [ "$INSTALL_DIR" = "/usr/local/bin" ]; then
        INSTALL_DIR="$HOME/.local/bin"
    fi
    mkdir -p "$INSTALL_DIR"
fi

if [ -w "$INSTALL_DIR" ]; then
    cp "${{TMP_DIR}}/${{APP_NAME}}" "${{INSTALL_DIR}}/${{APP_NAME}}"
    chmod +x "${{INSTALL_DIR}}/${{APP_NAME}}"
else
    info "Elevated permissions required to install to ${{INSTALL_DIR}}"
    if command -v sudo >/dev/null 2>&1; then
        sudo cp "${{TMP_DIR}}/${{APP_NAME}}" "${{INSTALL_DIR}}/${{APP_NAME}}"
        sudo chmod +x "${{INSTALL_DIR}}/${{APP_NAME}}"
    else
        error "Cannot write to ${{INSTALL_DIR}} and sudo is unavailable. Set INSTALL_DIR to a writable directory."
    fi
fi

success "${{APP_NAME}} ${{TAG}} was installed successfully to ${{INSTALL_DIR}}/${{APP_NAME}}!"

# Check if INSTALL_DIR is in PATH
case ":$PATH:" in
    *":$INSTALL_DIR:"*) ;;
    *)
        printf "\n${{BLUE}}Note:${{NC}} Ensure '%s' is in your PATH.\n" "$INSTALL_DIR"
        printf "You can add it by running:\n  export PATH=\"%s:\$PATH\"\n" "$INSTALL_DIR"
        ;;
esac
"#
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_install_sh() {
        let script = generate_install_sh("my-cli", Some("https://github.com/acme/my-cli"), "/usr/local/bin");
        assert!(script.contains("APP_NAME=\"my-cli\""));
        assert!(script.contains("GITHUB_REPO=\"acme/my-cli\""));
        assert!(script.contains("INSTALL_DIR=\"/usr/local/bin\""));
        assert!(script.contains("tar -xzf"));
    }

    #[test]
    fn test_install_sh_contains_checksum_verification() {
        let script = generate_install_sh("my-cli", Some("https://github.com/acme/my-cli"), "/usr/local/bin");
        assert!(script.contains("checksums.txt"), "install.sh must download checksums.txt");
        assert!(script.contains("sha256sum") || script.contains("shasum"), "install.sh must verify checksums");
        assert!(script.contains("Checksum mismatch"), "install.sh must abort on checksum mismatch");
    }
}
