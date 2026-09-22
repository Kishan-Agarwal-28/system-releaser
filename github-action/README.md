# system-releaser/release-action

Official GitHub Action for [system-releaser](https://github.com/system-releaser/system-releaser) — the universal, language-agnostic cross-platform build and package release engine.

## Features

- 🌐 **Language-Agnostic**: Detects and builds projects in 20+ languages (Rust, Go, C#, Python, TypeScript, Node.js, Java, Swift, C/C++, Zig, Dart, and more).
- 📦 **Universal Package Generation**: Automatically produces manifests and installers for 14 package managers: Homebrew, WinGet, Chocolatey, Scoop, Pacman/AUR, Snap, APT (.deb), RPM (.spec), Alpine APK, Nix flake, Flatpak, AppImage, asdf/mise, and MacPorts.
- 🚀 **10-Line CI Setup**: Drop into `.github/workflows/release.yml` with minimal boilerplate.

## Quickstart

Add the following workflow to `.github/workflows/release.yml` (or generate it automatically using `system-releaser init-ci`):

```yaml
name: Release

on:
  push:
    tags:
      - 'v*'

permissions:
  contents: write
  pull-requests: write

jobs:
  release:
    name: Build & Publish Release
    runs-on: ubuntu-latest
    steps:
      - name: Checkout repository
        uses: actions/checkout@v4
        with:
          fetch-depth: 0

      - name: Run system-releaser
        uses: system-releaser/release-action@v1
        with:
          github_token: ${{ secrets.GITHUB_TOKEN }}
```

## Inputs

| Input | Description | Required | Default |
|-------|-------------|----------|---------|
| `github_token` | GitHub token for creating releases and uploading artifacts | No | `${{ secrets.GITHUB_TOKEN }}` |
| `version_bump` | Version bump type (`patch`, `minor`, `major`, or semver string) | No | `""` |
| `skip_tests` | Skip pre-flight project tests before release (`true` / `false`) | No | `false` |
| `extra_args` | Additional CLI arguments passed to `system-releaser release` | No | `""` |

## License

MIT
