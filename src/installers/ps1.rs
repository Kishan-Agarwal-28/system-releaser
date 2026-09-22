/// Generates a PowerShell script (`install.ps1`) for Windows users running:
/// `irm https://.../install.ps1 | iex`
pub fn generate_install_ps1(
    app_name: &str,
    repo: Option<&str>,
    install_dir: &str,
) -> String {
    let repo_url = repo.unwrap_or("https://github.com/user/repo");
    let clean_repo = repo_url
        .trim_start_matches("https://github.com/")
        .trim_start_matches("http://github.com/")
        .trim_end_matches(".git");

    let clean_app = app_name
        .replace('`', "``")
        .replace('"', "`\"")
        .replace('$', "`$");
    let clean_repo_esc = clean_repo
        .replace('`', "``")
        .replace('"', "`\"")
        .replace('$', "`$");
    let clean_install_dir = install_dir
        .replace('`', "``")
        .replace('"', "`\"")
        .replace("$(", "`$(");

    format!(r#"$ErrorActionPreference = 'Stop'

$AppName = "{clean_app}"
$GithubRepo = "{clean_repo_esc}"
$InstallDirConfig = "{clean_install_dir}"

Write-Host "Installing $AppName for Windows..." -ForegroundColor Cyan

# 1. Detect Architecture
$Arch = if ([System.Environment]::Is64BitOperatingSystem) {{
    $osArch = [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture
    if ($osArch -eq [System.Runtime.InteropServices.Architecture]::Arm64) {{
        "arm64"
    }} elseif ($osArch -eq [System.Runtime.InteropServices.Architecture]::X64) {{
        "amd64"
    }} else {{
        throw "Unsupported operating system architecture: $osArch"
    }}
}} else {{
    throw "32-bit Windows is not supported."
}}

Write-Host "Detected architecture: $Arch" -ForegroundColor Gray

# 2. Get latest release from GitHub API
$ReleaseUrl = "https://api.github.com/repos/$GithubRepo/releases/latest"
try {{
    $Release = Invoke-RestMethod -Uri $ReleaseUrl -UseBasicParsing
    $Tag = $Release.tag_name
    $Version = $Tag.TrimStart('v')
}} catch {{
    throw "Failed to fetch latest release from $ReleaseUrl : $_"
}}

Write-Host "Latest release: $Tag" -ForegroundColor Gray

$ArchiveName = "${{AppName}}_${{Version}}_windows_${{Arch}}.zip"
$DownloadUrl = "https://github.com/$GithubRepo/releases/download/$Tag/$ArchiveName"

# 3. Download Archive
$TempDir = Join-Path ([System.IO.Path]::GetTempPath()) ([System.Guid]::NewGuid().ToString())
New-Item -ItemType Directory -Path $TempDir -Force | Out-Null
$ZipPath = Join-Path $TempDir $ArchiveName

try {{
    Write-Host "Downloading $DownloadUrl..." -ForegroundColor Cyan
    Invoke-WebRequest -Uri $DownloadUrl -OutFile $ZipPath -UseBasicParsing

    # 4. Verify SHA-256 Checksum
    $ChecksumsUrl = "https://github.com/$GithubRepo/releases/download/$Tag/checksums.txt"
    $ChecksumsPath = Join-Path $TempDir "checksums.txt"
    Write-Host "Verifying SHA-256 checksum..." -ForegroundColor Cyan
    try {{
        Invoke-WebRequest -Uri $ChecksumsUrl -OutFile $ChecksumsPath -UseBasicParsing
    }} catch {{
        throw "Failed to download checksums.txt from $ChecksumsUrl : $_"
    }}

    $ActualHash = (Get-FileHash -Path $ZipPath -Algorithm SHA256).Hash.ToLower()
    $ExpectedHash = $null

    foreach ($Line in (Get-Content $ChecksumsPath)) {{
        $Parts = $Line.Trim() -split '\s+', 2
        if ($Parts.Count -ge 2 -and $Parts[1].Trim() -eq $ArchiveName) {{
            $ExpectedHash = $Parts[0].Trim().ToLower()
            break
        }}
    }}

    if (-not $ExpectedHash) {{
        throw "Checksum for $ArchiveName not found in checksums.txt"
    }}

    if ($ActualHash -ne $ExpectedHash) {{
        throw "Checksum mismatch! Expected: $ExpectedHash  Got: $ActualHash - aborting for security."
    }}

    Write-Host "Checksum verified." -ForegroundColor Green

    # 5. Resolve Install Directory
    $TargetDir = if ($InstallDirConfig.StartsWith("$HOME")) {{
        $InstallDirConfig.Replace("$HOME", $HOME)
    }} else {{
        Join-Path $HOME ".local\bin"
    }}

    if (-not (Test-Path $TargetDir)) {{
        New-Item -ItemType Directory -Path $TargetDir -Force | Out-Null
    }}

    Write-Host "Extracting to $TargetDir..." -ForegroundColor Cyan
    Expand-Archive -Path $ZipPath -DestinationPath $TempDir -Force
    $ExeSource = Join-Path $TempDir "$AppName.exe"

    if (-not (Test-Path $ExeSource)) {{
        # Find exe recursively if inside subfolder
        $Found = Get-ChildItem -Path $TempDir -Filter "$AppName.exe" -Recurse | Select-Object -First 1
        if ($Found) {{
            $ExeSource = $Found.FullName
        }} else {{
            throw "Could not find $AppName.exe in archive."
        }}
    }}

    $TargetExe = Join-Path $TargetDir "$AppName.exe"
    Copy-Item -Path $ExeSource -Destination $TargetExe -Force

    # 6. Check / Add to User PATH
    $UserPath = [Environment]::GetEnvironmentVariable("Path", [EnvironmentVariableTarget]::User)
    if ($UserPath -notlike "*$TargetDir*") {{
        Write-Host "Adding $TargetDir to User PATH environment variable..." -ForegroundColor Yellow
        [Environment]::SetEnvironmentVariable("Path", "$UserPath;$TargetDir", [EnvironmentVariableTarget]::User)
        $Env:Path = "$Env:Path;$TargetDir"
    }}

    Write-Host "`n$AppName $Tag was installed successfully to $TargetExe!" -ForegroundColor Green
    Write-Host "You may need to restart your terminal for PATH changes to take effect." -ForegroundColor Gray
}} finally {{
    Remove-Item -Path $TempDir -Recurse -Force -ErrorAction SilentlyContinue
}}
"#
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_install_ps1() {
        let script = generate_install_ps1("my-cli", Some("https://github.com/acme/my-cli"), "$HOME/.local/bin");
        assert!(script.contains("$AppName = \"my-cli\""));
        assert!(script.contains("$GithubRepo = \"acme/my-cli\""));
        assert!(script.contains("Expand-Archive"));
        assert!(script.contains("[EnvironmentVariableTarget]::User"));
    }

    #[test]
    fn test_install_ps1_contains_checksum_verification() {
        let script = generate_install_ps1("my-cli", Some("https://github.com/acme/my-cli"), "$HOME/.local/bin");
        assert!(script.contains("checksums.txt"), "install.ps1 must download checksums.txt");
        assert!(script.contains("Get-FileHash"), "install.ps1 must compute hash using Get-FileHash");
        assert!(script.contains("SHA256"), "install.ps1 must specify SHA256 algorithm");
        assert!(script.contains("Checksum mismatch"), "install.ps1 must abort on checksum mismatch");
        assert!(script.contains(".Hash.ToLower()"), "install.ps1 must convert computed hash to lowercase");
        assert!(script.contains("throw \"Checksum mismatch!"), "install.ps1 must throw an exception on mismatch");
    }

    #[test]
    fn test_generate_install_ps1_is_pure_ascii() {
        let script = generate_install_ps1("my-cli", Some("https://github.com/acme/my-cli"), "$HOME/.local/bin");
        for (idx, ch) in script.chars().enumerate() {
            assert!(
                ch.is_ascii(),
                "Non-ASCII character '{}' (U+{:04X}) detected at char position {}",
                ch,
                ch as u32,
                idx
            );
        }
    }
}
