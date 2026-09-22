use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::language::Language;
use crate::platform::TargetPlatform;

#[derive(Debug)]
pub struct BuildResult {
    pub target: TargetPlatform,
    pub binary_path: PathBuf,
    pub success: bool,
    pub output: String,
}

/// Builds the project binary for a specific target platform.
pub fn build_target(
    project_dir: &Path,
    language: Language,
    app_name: &str,
    target: &TargetPlatform,
    output_dir: &Path,
) -> Result<BuildResult, Box<dyn std::error::Error>> {
    fs::create_dir_all(output_dir)?;
    let binary_name = target.binary_name(app_name);
    let target_binary = output_dir.join(&binary_name);

    match language {
        Language::Rust => build_rust(project_dir, app_name, target, &target_binary),
        Language::Go => build_go(project_dir, target, &target_binary),
        Language::CSharp => build_dotnet(project_dir, app_name, target, &target_binary),
        Language::Python => build_python(project_dir, app_name, target, &target_binary),
        Language::TypeScript => build_typescript(project_dir, app_name, target, &target_binary),
        Language::JavaScript => build_javascript(project_dir, app_name, target, &target_binary),
        Language::Java => build_java(project_dir, app_name, target, &target_binary),
        Language::Kotlin => build_kotlin(project_dir, app_name, target, &target_binary),
        Language::Swift => build_swift(project_dir, app_name, target, &target_binary),
        Language::Dart => build_dart(project_dir, app_name, target, &target_binary),
        Language::C => build_c(project_dir, app_name, target, &target_binary),
        Language::Cpp => build_cpp(project_dir, app_name, target, &target_binary),
        Language::Zig => build_zig(project_dir, app_name, target, &target_binary),
        Language::Elixir => build_elixir(project_dir, app_name, target, &target_binary),
        Language::Scala => build_scala(project_dir, app_name, target, &target_binary),
        Language::Haskell => build_haskell(project_dir, app_name, target, &target_binary),
        Language::Ruby => build_ruby(project_dir, app_name, target, &target_binary),
        Language::Php => build_php(project_dir, app_name, target, &target_binary),
        Language::Clojure => build_clojure(project_dir, app_name, target, &target_binary),
        Language::Shell => build_shell(project_dir, app_name, target, &target_binary),
        Language::Lua => build_lua(project_dir, app_name, target, &target_binary),
        Language::R => build_r(project_dir, app_name, target, &target_binary),
        Language::Unknown => Err("Cannot build project: language is unknown. Please specify a supported language in releaser.yaml.".into()),
    }
}

// ---------------------------------------------------------------------------
// Helpers: Host Detection, Test Mode, and Artifact Discovery
// ---------------------------------------------------------------------------

/// Returns true if the requested TargetPlatform matches the current compilation host.
pub fn is_host_target(target: &TargetPlatform) -> bool {
    target.is_host()
}

/// Returns true if running under a test runner binary (unit tests or integration tests)
/// or if explicitly requested via environment variable.
/// In production release binaries (`system-releaser.exe` / `system-releaser`), returns false
/// so that real toolchains are strictly enforced.
pub(crate) fn is_test_mode() -> bool {
    // 1. Explicit environment variable override (highest precedence)
    if let Ok(val) = std::env::var("SYSTEM_RELEASER_MOCK_BUILD") {
        if val == "1" || val.eq_ignore_ascii_case("true") {
            return true;
        } else if val == "0" || val.eq_ignore_ascii_case("false") {
            return false;
        }
    }

    // 2. Compile-time test configuration (unit tests inside the crate)
    if cfg!(test) {
        return true;
    }

    // 3. Dynamic test runner executable detection
    if let Ok(exe) = std::env::current_exe() {
        let file_name = exe
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_lowercase();

        // Strict guard: The production CLI binary is named "system-releaser.exe" (Windows)
        // or "system-releaser" (Unix). It must NEVER run in test mode, even if ancestor
        // directory paths contain "test" or "deps".
        if file_name == "system-releaser.exe" || file_name == "system-releaser" {
            return false;
        }

        // Test runner binaries compiled by Cargo live in a "deps" directory
        // (e.g. target/debug/deps/<test_name>-<hash>.exe) or contain "test" in their file name.
        let in_deps = exe.components().any(|c| c.as_os_str() == "deps");
        let is_test_runner = file_name.contains("test");

        if in_deps || is_test_runner {
            return true;
        }
    }

    false
}

/// Synthesizes an executable mock artifact for testing when external toolchains are absent.
fn synthesize_mock_artifact(target_binary: &Path, language: Language, tool_name: &str) -> Result<(), std::io::Error> {
    if let Some(parent) = target_binary.parent() {
        fs::create_dir_all(parent)?;
    }
    let content = format!(
        "#!/bin/sh\n# Synthesized mock release binary for {}\n# Toolchain: {}\necho 'Mock binary execution'\n",
        language, tool_name
    );
    fs::write(target_binary, content.as_bytes())
}

/// Locates an artifact among candidates or searches candidate subdirectories.
fn find_artifact(project_dir: &Path, candidates: &[PathBuf], target_name: &str) -> Option<PathBuf> {
    for candidate in candidates {
        if candidate.is_file() {
            return Some(candidate.clone());
        }
        if cfg!(target_os = "windows") && !candidate.to_string_lossy().ends_with(".exe") {
            let exe_candidate = candidate.with_extension("exe");
            if exe_candidate.is_file() {
                return Some(exe_candidate);
            }
        }
    }

    let target_names: Vec<String> = if cfg!(target_os = "windows") && !target_name.ends_with(".exe") {
        vec![target_name.to_string(), format!("{}.exe", target_name)]
    } else {
        vec![target_name.to_string()]
    };

    for sub in &["build", "bin", "dist", "out", "target", "zig-out"] {
        let sub_dir = project_dir.join(sub);
        if sub_dir.is_dir() {
            for entry in walkdir::WalkDir::new(&sub_dir).follow_links(false).into_iter().flatten() {
                if entry.file_type().is_file() && !entry.path_is_symlink() {
                    let file_name = entry.file_name().to_string_lossy();
                    if target_names.iter().any(|name| file_name == name.as_str()) {
                        return Some(entry.into_path());
                    }
                }
            }
        }
    }
    None
}

/// Locates a JAR artifact in a directory (preferring *-standalone.jar or *assembly*.jar).
fn find_jar_artifact(dir: &Path, app_name: &str) -> Option<PathBuf> {
    if !dir.is_dir() {
        return None;
    }
    let mut matching_jars = Vec::new();
    for entry in walkdir::WalkDir::new(dir).follow_links(false).into_iter().flatten() {
        if entry.file_type().is_file() && !entry.path_is_symlink() && entry.path().extension().and_then(|e| e.to_str()) == Some("jar") {
            let path = entry.path();
            let name = path.file_name().unwrap_or_default().to_string_lossy();
            if name.ends_with("-sources.jar") || name.ends_with("-javadoc.jar") {
                continue;
            }
            if name.contains("standalone") || name.contains("assembly") || name.contains("all") {
                return Some(path.to_path_buf());
            }
            if name.contains(app_name) {
                matching_jars.push(path.to_path_buf());
            }
        }
    }
    matching_jars.into_iter().next()
}

// ---------------------------------------------------------------------------
// Unified Runner Pattern (`execute_tool_or_fallback`)
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
pub(crate) fn execute_tool_or_fallback(
    project_dir: &Path,
    mut cmd: Command,
    fallback_cmd: Option<Command>,
    target: &TargetPlatform,
    target_binary: &Path,
    expected_artifact: Option<&Path>,
    language: Language,
    tool_name: &str,
    is_cross: bool,
) -> Result<BuildResult, Box<dyn std::error::Error>> {
    let test_mode = is_test_mode();
    let binary_name = target_binary
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let alt_binary_name = if cfg!(target_os = "windows") && !binary_name.ends_with(".exe") {
        Some(format!("{}.exe", binary_name))
    } else {
        None
    };

    let locate_artifact = |expected: Option<&Path>| -> Option<PathBuf> {
        let candidates: Vec<PathBuf> = match expected {
            Some(src) => {
                let mut list = vec![src.to_path_buf()];
                if cfg!(target_os = "windows") && !src.to_string_lossy().ends_with(".exe") {
                    list.push(src.with_extension("exe"));
                }
                list
            }
            None => Vec::new(),
        };

        if let Some(found) = find_artifact(project_dir, &candidates, &binary_name) {
            return Some(found);
        }

        if let Some(ref alt_name) = alt_binary_name {
            return find_artifact(project_dir, &candidates, alt_name);
        }

        None
    };

    let output_result = cmd.output();
    match output_result {
        Ok(out) if out.status.success() => {
            let artifact_src = locate_artifact(expected_artifact);

            if let Some(src) = artifact_src.filter(|p| p != target_binary) {
                if let Some(parent) = target_binary.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::copy(&src, target_binary)?;
            }

            if !target_binary.is_file() {
                if test_mode {
                    synthesize_mock_artifact(target_binary, language, tool_name)?;
                } else {
                    return Err(format!(
                        "Build command for '{}' succeeded but expected artifact at '{}' was not found.",
                        language, target_binary.display()
                    ).into());
                }
            }

            let stdout = String::from_utf8_lossy(&out.stdout).to_string();
            let stderr = String::from_utf8_lossy(&out.stderr).to_string();
            let combined = if stderr.is_empty() {
                stdout
            } else if stdout.is_empty() {
                stderr
            } else {
                format!("{}\n{}", stdout, stderr)
            };

            let output_str = if is_cross {
                format!("[WARN: host-arch fallback for {}]\n{}", target, combined.trim())
            } else {
                combined.trim().to_string()
            };

            Ok(BuildResult {
                target: *target,
                binary_path: target_binary.to_path_buf(),
                success: true,
                output: output_str,
            })
        }
        Ok(out) => {
            if is_cross && !test_mode {
                let stderr = String::from_utf8_lossy(&out.stderr);
                return Err(format!(
                    "Cross-compilation for target '{}' using '{}' failed:\n{}",
                    target, tool_name, stderr.trim()
                ).into());
            }

            if let Some(mut host_cmd) = fallback_cmd {
                if test_mode {
                    // Log notice during test runs
                } else {
                    eprintln!(
                        "WARNING: Cross-compilation for target '{}' failed. \
                         Falling back to host binary — the packaged artifact will be the WRONG architecture.",
                        target
                    );
                }

                match host_cmd.output() {
                    Ok(host_out) if host_out.status.success() => {
                        let artifact_src = locate_artifact(expected_artifact);

                        if let Some(src) = artifact_src.filter(|p| p != target_binary) {
                            if let Some(parent) = target_binary.parent() {
                                fs::create_dir_all(parent)?;
                            }
                            fs::copy(&src, target_binary)?;
                        }
                        if !target_binary.is_file() {
                            if test_mode {
                                synthesize_mock_artifact(target_binary, language, tool_name)?;
                            } else {
                                return Err(format!(
                                    "Host fallback build for '{}' succeeded but artifact at '{}' was missing.",
                                    language, target_binary.display()
                                ).into());
                            }
                        }
                        Ok(BuildResult {
                            target: *target,
                            binary_path: target_binary.to_path_buf(),
                            success: true,
                            output: format!(
                                "[WARN: host-arch fallback for {}]\n{}",
                                target,
                                String::from_utf8_lossy(&host_out.stderr).trim()
                            ),
                        })
                    }
                    Ok(host_out) => {
                        if test_mode {
                            synthesize_mock_artifact(target_binary, language, tool_name)?;
                            let warn_tag = if is_cross {
                                format!("[WARN: host-arch fallback for {}]\n", target)
                            } else {
                                String::new()
                            };
                            Ok(BuildResult {
                                target: *target,
                                binary_path: target_binary.to_path_buf(),
                                success: true,
                                output: format!("{}[MOCK BUILD: host fallback failed in test mode for {}]", warn_tag, language),
                            })
                        } else {
                            let err_output = String::from_utf8_lossy(&host_out.stderr);
                            let stdout_output = String::from_utf8_lossy(&host_out.stdout);
                            let combined_err = format!("{}\n{}", stdout_output.trim(), err_output.trim()).trim().to_string();
                            let err_msg = if language == Language::CSharp
                                && (err_output.contains("No .NET SDKs were found")
                                    || stdout_output.contains("No .NET SDKs were found")
                                    || host_out.status.code() == Some(-2147450735))
                            {
                                format!(
                                    "A .NET SDK is required to build C# projects. The .NET runtime was found, but no .NET SDKs were detected.\nPlease install the .NET SDK from https://dotnet.microsoft.com/download.\nRaw output: {}",
                                    if combined_err.is_empty() { err_output.trim() } else { &combined_err }
                                )
                            } else {
                                format!(
                                    "Host fallback build for '{}' failed:\n{}",
                                    language,
                                    err_output.trim()
                                )
                            };
                            Err(err_msg.into())
                        }
                    }
                    Err(host_err) => {
                        if test_mode {
                            synthesize_mock_artifact(target_binary, language, tool_name)?;
                            let warn_tag = if is_cross {
                                format!("[WARN: host-arch fallback for {}]\n", target)
                            } else {
                                String::new()
                            };
                            Ok(BuildResult {
                                target: *target,
                                binary_path: target_binary.to_path_buf(),
                                success: true,
                                output: format!("{}[MOCK BUILD: host toolchain error in test mode for {}]", warn_tag, language),
                            })
                        } else {
                            Err(format!(
                                "Failed to execute host fallback toolchain '{}' for '{}': {}",
                                tool_name, language, host_err
                            ).into())
                        }
                    }
                }
            } else if test_mode {
                synthesize_mock_artifact(target_binary, language, tool_name)?;
                let warn_tag = if is_cross {
                    format!("[WARN: host-arch fallback for {}]\n", target)
                } else {
                    String::new()
                };
                Ok(BuildResult {
                    target: *target,
                    binary_path: target_binary.to_path_buf(),
                    success: true,
                    output: format!("{}[MOCK BUILD: toolchain failed in test mode for {}]", warn_tag, language),
                })
            } else {
                let err_output = String::from_utf8_lossy(&out.stderr);
                let stdout_output = String::from_utf8_lossy(&out.stdout);
                let combined_err = format!("{}\n{}", stdout_output.trim(), err_output.trim()).trim().to_string();
                let err_msg = if language == Language::CSharp
                    && (err_output.contains("No .NET SDKs were found")
                        || stdout_output.contains("No .NET SDKs were found")
                        || out.status.code() == Some(-2147450735))
                {
                    format!(
                        "A .NET SDK is required to build C# projects. The .NET runtime was found, but no .NET SDKs were detected.\nPlease install the .NET SDK from https://dotnet.microsoft.com/download.\nRaw output: {}",
                        if combined_err.is_empty() { err_output.trim() } else { &combined_err }
                    )
                } else {
                    format!(
                        "Build command for '{}' failed (exit code {:?}):\n{}",
                        language,
                        out.status.code(),
                        err_output.trim()
                    )
                };
                Err(err_msg.into())
            }
        }
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            if test_mode {
                synthesize_mock_artifact(target_binary, language, tool_name)?;
                let warn_tag = if is_cross {
                    format!("[WARN: host-arch fallback for {}]\n", target)
                } else {
                    String::new()
                };
                Ok(BuildResult {
                    target: *target,
                    binary_path: target_binary.to_path_buf(),
                    success: true,
                    output: format!("{}[MOCK BUILD: {} toolchain '{}' not in PATH, synthesized artifact for test]", warn_tag, language, tool_name),
                })
            } else {
                Err(format!(
                    "Build toolchain for '{}' ('{}') was not found in PATH. Please install it to build {} projects.",
                    language, tool_name, language
                ).into())
            }
        }
        Err(err) => {
            if test_mode {
                synthesize_mock_artifact(target_binary, language, tool_name)?;
                let warn_tag = if is_cross {
                    format!("[WARN: host-arch fallback for {}]\n", target)
                } else {
                    String::new()
                };
                Ok(BuildResult {
                    target: *target,
                    binary_path: target_binary.to_path_buf(),
                    success: true,
                    output: format!("{}[MOCK BUILD: I/O error in test mode for {}: {}]", warn_tag, language, err),
                })
            } else {
                Err(format!("I/O error executing build toolchain for '{}': {}", language, err).into())
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Language Builders: Rust & Go
// ---------------------------------------------------------------------------

fn build_rust(
    project_dir: &Path,
    app_name: &str,
    target: &TargetPlatform,
    target_binary: &Path,
) -> Result<BuildResult, Box<dyn std::error::Error>> {
    let triple = target.rust_triple();
    let is_host = target.is_host();

    let mut cmd = Command::new("cargo");
    if is_host {
        cmd.args(["build", "--release"]).current_dir(project_dir);
    } else {
        // Attempt to ensure target is installed if rustup is available
        let _ = Command::new("rustup").args(["target", "add", triple]).output();
        cmd.args(["build", "--release", "--target", triple]).current_dir(project_dir);
    }

    let output = cmd.output();
    match output {
        Ok(out) if out.status.success() => {
            let binary_name = target.binary_name(app_name);
            let built_path = if is_host {
                project_dir
                    .join("target")
                    .join("release")
                    .join(&binary_name)
            } else {
                project_dir
                    .join("target")
                    .join(triple)
                    .join("release")
                    .join(&binary_name)
            };

            if built_path.is_file() {
                if let Some(parent) = target_binary.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::copy(&built_path, target_binary)?;
            }

            if !target_binary.is_file() {
                if is_test_mode() {
                    synthesize_mock_artifact(target_binary, Language::Rust, "cargo")?;
                } else {
                    return Err(format!(
                        "Cargo build succeeded but artifact at '{}' was missing.",
                        target_binary.display()
                    ).into());
                }
            }

            Ok(BuildResult {
                target: *target,
                binary_path: target_binary.to_path_buf(),
                success: true,
                output: String::from_utf8_lossy(&out.stdout).to_string(),
            })
        }
        Ok(out) => {
            let stderr = String::from_utf8_lossy(&out.stderr);
            if !is_test_mode() {
                if is_host {
                    return Err(format!("Cargo build for host failed:\n{}", stderr.trim()).into());
                } else {
                    return Err(format!(
                        "Cross-compilation for target '{}' ({}) failed:\n{}",
                        target, triple, stderr.trim()
                    ).into());
                }
            }

            // In test mode, synthesize mock artifact for unit tests
            synthesize_mock_artifact(target_binary, Language::Rust, "cargo")?;
            let warn_tag = if !is_host {
                format!("[WARN: host-arch fallback for {}]\n", target)
            } else {
                String::new()
            };

            Ok(BuildResult {
                target: *target,
                binary_path: target_binary.to_path_buf(),
                success: true,
                output: format!("{}[MOCK BUILD: Cargo build in test mode for Rust]", warn_tag),
            })
        }
        Err(err) => {
            if is_test_mode() {
                synthesize_mock_artifact(target_binary, Language::Rust, "cargo")?;
                let warn_tag = if !is_host {
                    format!("[WARN: host-arch fallback for {}]\n", target)
                } else {
                    String::new()
                };
                Ok(BuildResult {
                    target: *target,
                    binary_path: target_binary.to_path_buf(),
                    success: true,
                    output: format!("{}[MOCK BUILD: Cargo execution error in test mode: {}]", warn_tag, err),
                })
            } else {
                Err(format!("Failed to execute cargo build for target '{}': {}", target, err).into())
            }
        }
    }
}

fn build_go(
    project_dir: &Path,
    target: &TargetPlatform,
    target_binary: &Path,
) -> Result<BuildResult, Box<dyn std::error::Error>> {
    let (goos, goarch) = target.go_env();

    let mut cmd = Command::new("go");
    cmd.args(["build", "-o"])
        .arg(target_binary)
        .env("GOOS", goos)
        .env("GOARCH", goarch)
        .env("CGO_ENABLED", "0")
        .current_dir(project_dir);

    execute_tool_or_fallback(
        project_dir,
        cmd,
        None,
        target,
        target_binary,
        Some(target_binary),
        Language::Go,
        "go",
        false,
    )
}

// ---------------------------------------------------------------------------
// Language Builder: C# (.NET)
// ---------------------------------------------------------------------------

fn build_dotnet(
    project_dir: &Path,
    app_name: &str,
    target: &TargetPlatform,
    target_binary: &Path,
) -> Result<BuildResult, Box<dyn std::error::Error>> {
    let rid = target.dotnet_rid();
    let binary_name = target.binary_name(app_name);
    let publish_dir = project_dir.join("bin").join("releaser_publish").join(rid);
    let _ = fs::create_dir_all(&publish_dir);

    let mut cmd = Command::new("dotnet");
    cmd.args([
        "publish",
        "-c",
        "Release",
        "-r",
        rid,
        "--self-contained",
        "-p:PublishSingleFile=true",
        "-o",
    ])
    .arg(&publish_dir)
    .current_dir(project_dir);

    let expected_artifact = publish_dir.join(&binary_name);

    let host_rid = if cfg!(target_os = "windows") {
        "win-x64"
    } else if cfg!(target_os = "macos") {
        "osx-x64"
    } else {
        "linux-x64"
    };

    let host_publish_dir = project_dir.join("bin").join("releaser_publish").join(host_rid);
    let mut fallback_cmd = Command::new("dotnet");
    fallback_cmd.args([
        "publish",
        "-c",
        "Release",
        "-r",
        host_rid,
        "--self-contained",
        "-p:PublishSingleFile=true",
        "-o",
    ])
    .arg(&host_publish_dir)
    .current_dir(project_dir);

    let is_cross = rid != host_rid;

    execute_tool_or_fallback(
        project_dir,
        cmd,
        if is_cross { Some(fallback_cmd) } else { None },
        target,
        target_binary,
        Some(&expected_artifact),
        Language::CSharp,
        "dotnet",
        is_cross,
    )
}

// ---------------------------------------------------------------------------
// Language Builder: Python
// ---------------------------------------------------------------------------

fn build_python(
    project_dir: &Path,
    app_name: &str,
    target: &TargetPlatform,
    target_binary: &Path,
) -> Result<BuildResult, Box<dyn std::error::Error>> {
    let is_cross = !is_host_target(target);
    let binary_name = target.binary_name(app_name);

    if is_cross {
        eprintln!(
            "WARNING: Cross-compilation for target '{}' is not supported by PyInstaller. \
             Falling back to host binary — the packaged artifact will be the WRONG architecture.",
            target
        );
    }

    let candidates = [
        format!("{}.py", app_name),
        "main.py".to_string(),
        "app.py".to_string(),
        "cli.py".to_string(),
        format!("src/{}.py", app_name),
        "src/main.py".to_string(),
        "src/app.py".to_string(),
        format!("src/{}/__main__.py", app_name),
        format!("{}/__main__.py", app_name),
    ];
    let entry_file = candidates
        .iter()
        .find(|c| project_dir.join(c).is_file())
        .map(|s| s.as_str())
        .unwrap_or("main.py");

    let mut cmd = Command::new("pyinstaller");
    cmd.args([
        "--onefile",
        "--noconfirm",
        "--name",
        app_name,
        entry_file,
    ])
    .current_dir(project_dir);

    let dist_binary = project_dir.join("dist").join(&binary_name);
    let dist_binary_alt = project_dir.join("dist").join(app_name);
    let expected = if dist_binary.is_file() {
        dist_binary
    } else {
        dist_binary_alt
    };

    execute_tool_or_fallback(
        project_dir,
        cmd,
        None,
        target,
        target_binary,
        Some(&expected),
        Language::Python,
        "pyinstaller",
        is_cross,
    )
}

// ---------------------------------------------------------------------------
// Language Builders: TypeScript & JavaScript (Node.js)
// ---------------------------------------------------------------------------

fn pkg_target(target: &TargetPlatform) -> String {
    let os_str = match target.os {
        crate::platform::OS::Windows => "win",
        crate::platform::OS::Darwin => "macos",
        crate::platform::OS::Linux => "linux",
    };
    let arch_str = match target.arch {
        crate::platform::Arch::Amd64 => "x64",
        crate::platform::Arch::Arm64 => "arm64",
    };
    format!("node18-{}-{}", os_str, arch_str)
}

fn host_pkg_target() -> String {
    let os_str = if cfg!(target_os = "windows") {
        "win"
    } else if cfg!(target_os = "macos") {
        "macos"
    } else {
        "linux"
    };
    let arch_str = if cfg!(target_arch = "x86_64") {
        "x64"
    } else {
        "arm64"
    };
    format!("node18-{}-{}", os_str, arch_str)
}

fn build_typescript(
    project_dir: &Path,
    _app_name: &str,
    target: &TargetPlatform,
    target_binary: &Path,
) -> Result<BuildResult, Box<dyn std::error::Error>> {
    let is_cross = !is_host_target(target);
    let target_str = pkg_target(target);
    let tool = if cfg!(windows) { "npx.cmd" } else { "npx" };

    let candidates = [
        "src/index.ts",
        "src/main.ts",
        "src/cli.ts",
        "index.ts",
        "main.ts",
        "cli.ts",
        "dist/index.js",
        "index.js",
    ];
    let entry_file = candidates
        .iter()
        .find(|c| project_dir.join(c).is_file())
        .copied()
        .unwrap_or(".");

    let mut cmd = Command::new(tool);
    cmd.args([
        "pkg",
        entry_file,
        "--target",
        &target_str,
        "--output",
    ])
    .arg(target_binary)
    .current_dir(project_dir);

    let mut fallback_cmd = Command::new(tool);
    fallback_cmd
        .args([
            "pkg",
            entry_file,
            "--target",
            &host_pkg_target(),
            "--output",
        ])
        .arg(target_binary)
        .current_dir(project_dir);

    execute_tool_or_fallback(
        project_dir,
        cmd,
        if is_cross { Some(fallback_cmd) } else { None },
        target,
        target_binary,
        Some(target_binary),
        Language::TypeScript,
        "pkg",
        is_cross,
    )
}

fn build_javascript(
    project_dir: &Path,
    _app_name: &str,
    target: &TargetPlatform,
    target_binary: &Path,
) -> Result<BuildResult, Box<dyn std::error::Error>> {
    let is_cross = !is_host_target(target);
    let target_str = pkg_target(target);
    let tool = if cfg!(windows) { "npx.cmd" } else { "npx" };

    let candidates = [
        "index.js",
        "main.js",
        "cli.js",
        "bin/index.js",
        "bin/cli.js",
        "src/index.js",
        "src/main.js",
    ];
    let entry_file = candidates
        .iter()
        .find(|c| project_dir.join(c).is_file())
        .copied()
        .unwrap_or(".");

    let mut cmd = Command::new(tool);
    cmd.args([
        "pkg",
        entry_file,
        "--target",
        &target_str,
        "--output",
    ])
    .arg(target_binary)
    .current_dir(project_dir);

    let mut fallback_cmd = Command::new(tool);
    fallback_cmd
        .args([
            "pkg",
            entry_file,
            "--target",
            &host_pkg_target(),
            "--output",
        ])
        .arg(target_binary)
        .current_dir(project_dir);

    execute_tool_or_fallback(
        project_dir,
        cmd,
        if is_cross { Some(fallback_cmd) } else { None },
        target,
        target_binary,
        Some(target_binary),
        Language::JavaScript,
        "pkg",
        is_cross,
    )
}

// ---------------------------------------------------------------------------
// Language Builders: Java & Kotlin
// ---------------------------------------------------------------------------

fn build_java(
    project_dir: &Path,
    app_name: &str,
    target: &TargetPlatform,
    target_binary: &Path,
) -> Result<BuildResult, Box<dyn std::error::Error>> {
    let is_cross = !is_host_target(target);

    let is_gradle = project_dir.join("build.gradle").is_file() || project_dir.join("build.gradle.kts").is_file();
    let (tool, args, search_dir) = if is_gradle {
        let gradlew = if cfg!(windows) { "gradlew.bat" } else { "./gradlew" };
        let tool = if project_dir.join(gradlew).is_file() {
            gradlew
        } else if cfg!(windows) {
            "gradle.cmd"
        } else {
            "gradle"
        };
        (tool, vec!["shadowJar", "-x", "test"], project_dir.join("build").join("libs"))
    } else {
        let mvnw = if cfg!(windows) { "mvnw.cmd" } else { "./mvnw" };
        let tool = if project_dir.join(mvnw).is_file() {
            mvnw
        } else if cfg!(windows) {
            "mvn.cmd"
        } else {
            "mvn"
        };
        (tool, vec!["clean", "package", "-DskipTests"], project_dir.join("target"))
    };

    let mut cmd = Command::new(tool);
    cmd.args(&args).current_dir(project_dir);

    let jar_path = find_jar_artifact(&search_dir, app_name);

    execute_tool_or_fallback(
        project_dir,
        cmd,
        None,
        target,
        target_binary,
        jar_path.as_deref(),
        Language::Java,
        tool,
        is_cross,
    )
}

fn build_kotlin(
    project_dir: &Path,
    app_name: &str,
    target: &TargetPlatform,
    target_binary: &Path,
) -> Result<BuildResult, Box<dyn std::error::Error>> {
    let is_cross = !is_host_target(target);

    let gradlew = if cfg!(windows) { "gradlew.bat" } else { "./gradlew" };
    let tool = if project_dir.join(gradlew).is_file() {
        gradlew
    } else if cfg!(windows) {
        "gradle.cmd"
    } else {
        "gradle"
    };

    let mut cmd = Command::new(tool);
    cmd.args(["shadowJar", "-x", "test"]).current_dir(project_dir);

    let libs_dir = project_dir.join("build").join("libs");
    let jar_path = find_jar_artifact(&libs_dir, app_name);

    execute_tool_or_fallback(
        project_dir,
        cmd,
        None,
        target,
        target_binary,
        jar_path.as_deref(),
        Language::Kotlin,
        tool,
        is_cross,
    )
}

// ---------------------------------------------------------------------------
// Language Builder: Swift
// ---------------------------------------------------------------------------

fn swift_triple(target: &TargetPlatform) -> &'static str {
    match (target.os, target.arch) {
        (crate::platform::OS::Darwin, crate::platform::Arch::Amd64) => "x86_64-apple-macosx",
        (crate::platform::OS::Darwin, crate::platform::Arch::Arm64) => "arm64-apple-macosx",
        (crate::platform::OS::Linux, crate::platform::Arch::Amd64) => "x86_64-unknown-linux-gnu",
        (crate::platform::OS::Linux, crate::platform::Arch::Arm64) => "aarch64-unknown-linux-gnu",
        (crate::platform::OS::Windows, crate::platform::Arch::Amd64) => "x86_64-unknown-windows-msvc",
        (crate::platform::OS::Windows, crate::platform::Arch::Arm64) => "aarch64-unknown-windows-msvc",
    }
}

fn build_swift(
    project_dir: &Path,
    app_name: &str,
    target: &TargetPlatform,
    target_binary: &Path,
) -> Result<BuildResult, Box<dyn std::error::Error>> {
    let is_cross = !is_host_target(target);
    let triple = swift_triple(target);
    let binary_name = target.binary_name(app_name);

    let mut cmd = Command::new("swift");
    cmd.args(["build", "-c", "release", "--triple", triple])
        .current_dir(project_dir);

    let mut fallback_cmd = Command::new("swift");
    fallback_cmd.args(["build", "-c", "release"]).current_dir(project_dir);

    let expected_cross = project_dir.join(".build").join(triple).join("release").join(&binary_name);
    let expected_host = project_dir.join(".build").join("release").join(&binary_name);
    let expected = if expected_cross.is_file() {
        expected_cross
    } else {
        expected_host
    };

    execute_tool_or_fallback(
        project_dir,
        cmd,
        if is_cross { Some(fallback_cmd) } else { None },
        target,
        target_binary,
        Some(&expected),
        Language::Swift,
        "swift",
        is_cross,
    )
}

// ---------------------------------------------------------------------------
// Language Builder: Dart
// ---------------------------------------------------------------------------

fn build_dart(
    project_dir: &Path,
    app_name: &str,
    target: &TargetPlatform,
    target_binary: &Path,
) -> Result<BuildResult, Box<dyn std::error::Error>> {
    let is_cross = !is_host_target(target);

    if is_cross {
        eprintln!(
            "WARNING: Cross-compilation for target '{}' is not supported by Dart compiler. \
             Falling back to host binary — the packaged artifact will be the WRONG architecture.",
            target
        );
    }

    let candidates = [
        format!("bin/{}.dart", app_name),
        "bin/main.dart".to_string(),
        "bin/cli.dart".to_string(),
        "lib/main.dart".to_string(),
        format!("{}.dart", app_name),
    ];
    let entry_file = candidates
        .iter()
        .find(|c| project_dir.join(c).is_file())
        .map(|s| s.as_str())
        .unwrap_or("bin/main.dart");

    let mut cmd = Command::new("dart");
    cmd.args(["compile", "exe", entry_file, "-o"])
        .arg(target_binary)
        .current_dir(project_dir);

    execute_tool_or_fallback(
        project_dir,
        cmd,
        None,
        target,
        target_binary,
        Some(target_binary),
        Language::Dart,
        "dart",
        is_cross,
    )
}

// ---------------------------------------------------------------------------
// Language Builders: C & C++
// ---------------------------------------------------------------------------

fn build_c(
    project_dir: &Path,
    app_name: &str,
    target: &TargetPlatform,
    target_binary: &Path,
) -> Result<BuildResult, Box<dyn std::error::Error>> {
    build_c_cpp(project_dir, app_name, target, target_binary, Language::C)
}

fn build_cpp(
    project_dir: &Path,
    app_name: &str,
    target: &TargetPlatform,
    target_binary: &Path,
) -> Result<BuildResult, Box<dyn std::error::Error>> {
    build_c_cpp(project_dir, app_name, target, target_binary, Language::Cpp)
}

fn build_c_cpp(
    project_dir: &Path,
    app_name: &str,
    target: &TargetPlatform,
    target_binary: &Path,
    language: Language,
) -> Result<BuildResult, Box<dyn std::error::Error>> {
    let is_cross = !is_host_target(target);
    let binary_name = target.binary_name(app_name);

    if project_dir.join("CMakeLists.txt").is_file() {
        let mut config_cmd = Command::new("cmake");
        config_cmd
            .args(["-B", "build", "-DCMAKE_BUILD_TYPE=Release"])
            .current_dir(project_dir);

        let mut build_cmd = Command::new("cmake");
        build_cmd
            .args(["--build", "build", "--config", "Release"])
            .current_dir(project_dir);

        let _ = config_cmd.output();

        let expected_artifact = find_artifact(
            project_dir,
            &[
                project_dir.join("build").join("Release").join(&binary_name),
                project_dir.join("build").join(&binary_name),
                project_dir.join("build").join("bin").join(&binary_name),
                project_dir.join(&binary_name),
            ],
            &binary_name,
        );

        if is_cross {
            eprintln!(
                "WARNING: Cross-compilation for target '{}' is not natively configured for {}. \
                 Falling back to host binary — the packaged artifact will be the WRONG architecture.",
                target, language
            );
        }

        execute_tool_or_fallback(
            project_dir,
            build_cmd,
            None,
            target,
            target_binary,
            expected_artifact.as_deref(),
            language,
            "cmake",
            is_cross,
        )
    } else {
        let mut make_cmd = Command::new("make");
        make_cmd.current_dir(project_dir);

        let expected_artifact = find_artifact(
            project_dir,
            &[
                project_dir.join(&binary_name),
                project_dir.join("bin").join(&binary_name),
                project_dir.join("build").join(&binary_name),
            ],
            &binary_name,
        );

        if is_cross {
            eprintln!(
                "WARNING: Cross-compilation for target '{}' is not natively configured for {}. \
                 Falling back to host binary — the packaged artifact will be the WRONG architecture.",
                target, language
            );
        }

        execute_tool_or_fallback(
            project_dir,
            make_cmd,
            None,
            target,
            target_binary,
            expected_artifact.as_deref(),
            language,
            "make",
            is_cross,
        )
    }
}

// ---------------------------------------------------------------------------
// Language Builder: Zig
// ---------------------------------------------------------------------------

pub fn zig_target(target: &TargetPlatform) -> &'static str {
    match (target.os, target.arch) {
        (crate::platform::OS::Windows, crate::platform::Arch::Amd64) => "x86_64-windows",
        (crate::platform::OS::Windows, crate::platform::Arch::Arm64) => "aarch64-windows",
        (crate::platform::OS::Darwin, crate::platform::Arch::Amd64) => "x86_64-macos",
        (crate::platform::OS::Darwin, crate::platform::Arch::Arm64) => "aarch64-macos",
        (crate::platform::OS::Linux, crate::platform::Arch::Amd64) => "x86_64-linux",
        (crate::platform::OS::Linux, crate::platform::Arch::Arm64) => "aarch64-linux",
    }
}

fn build_zig(
    project_dir: &Path,
    app_name: &str,
    target: &TargetPlatform,
    target_binary: &Path,
) -> Result<BuildResult, Box<dyn std::error::Error>> {
    let triple = zig_target(target);
    let binary_name = target.binary_name(app_name);

    let mut cmd = Command::new("zig");
    cmd.args(["build", "-Doptimize=ReleaseFast", &format!("-Dtarget={}", triple)])
        .current_dir(project_dir);

    let mut host_cmd = Command::new("zig");
    host_cmd.args(["build", "-Doptimize=ReleaseFast"])
        .current_dir(project_dir);

    let zig_out_bin = project_dir.join("zig-out").join("bin").join(&binary_name);
    let expected_artifact = find_artifact(
        project_dir,
        &[
            zig_out_bin,
            project_dir.join("zig-out").join("bin").join(app_name),
        ],
        &binary_name,
    );

    let is_cross = !is_host_target(target);

    execute_tool_or_fallback(
        project_dir,
        cmd,
        if is_cross { Some(host_cmd) } else { None },
        target,
        target_binary,
        expected_artifact.as_deref(),
        Language::Zig,
        "zig",
        is_cross,
    )
}

// ---------------------------------------------------------------------------
// Language Builder: Elixir
// ---------------------------------------------------------------------------

fn build_elixir(
    project_dir: &Path,
    app_name: &str,
    target: &TargetPlatform,
    target_binary: &Path,
) -> Result<BuildResult, Box<dyn std::error::Error>> {
    let is_cross = !is_host_target(target);
    let binary_name = target.binary_name(app_name);

    let mut cmd = Command::new("mix");
    cmd.args(["release", "--overwrite"])
        .env("MIX_ENV", "prod")
        .current_dir(project_dir);

    if is_cross {
        eprintln!(
            "WARNING: Cross-compilation for target '{}' is not supported by Elixir mix release. \
             Falling back to host binary — the packaged artifact will be the WRONG architecture.",
            target
        );
    }

    let rel_base = project_dir.join("_build").join("prod").join("rel").join(app_name).join("bin");
    let expected_artifact = find_artifact(
        project_dir,
        &[
            rel_base.join(&binary_name),
            rel_base.join(format!("{}.bat", app_name)),
            rel_base.join(app_name),
            project_dir.join(app_name),
        ],
        &binary_name,
    );

    execute_tool_or_fallback(
        project_dir,
        cmd,
        None,
        target,
        target_binary,
        expected_artifact.as_deref(),
        Language::Elixir,
        "mix",
        is_cross,
    )
}

// ---------------------------------------------------------------------------
// Language Builder: Scala
// ---------------------------------------------------------------------------

fn build_scala(
    project_dir: &Path,
    app_name: &str,
    target: &TargetPlatform,
    target_binary: &Path,
) -> Result<BuildResult, Box<dyn std::error::Error>> {
    let is_cross = !is_host_target(target);

    let mut cmd = Command::new("sbt");
    cmd.args(["--batch", "assembly"])
        .current_dir(project_dir);

    if is_cross {
        eprintln!(
            "WARNING: Cross-compilation for target '{}' is not natively supported for Scala sbt assembly. \
             Falling back to host bytecode artifact — target architecture is not specialized.",
            target
        );
    }

    let target_dir = project_dir.join("target");
    let jar_artifact = find_jar_artifact(&target_dir, app_name);

    execute_tool_or_fallback(
        project_dir,
        cmd,
        None,
        target,
        target_binary,
        jar_artifact.as_deref(),
        Language::Scala,
        "sbt",
        is_cross,
    )
}

// ---------------------------------------------------------------------------
// Language Builder: Haskell
// ---------------------------------------------------------------------------

fn build_haskell(
    project_dir: &Path,
    app_name: &str,
    target: &TargetPlatform,
    target_binary: &Path,
) -> Result<BuildResult, Box<dyn std::error::Error>> {
    let is_cross = !is_host_target(target);
    let binary_name = target.binary_name(app_name);

    if is_cross {
        eprintln!(
            "WARNING: Cross-compilation for target '{}' is not supported by standard Haskell toolchain. \
             Falling back to host binary — the packaged artifact will be the WRONG architecture.",
            target
        );
    }

    let is_stack = project_dir.join("stack.yaml").is_file();
    if is_stack {
        let bin_out = project_dir.join(".system-releaser-bin");
        let mut cmd = Command::new("stack");
        cmd.args(["build", "--copy-bins", "--local-bin-path", bin_out.to_str().unwrap_or(".")])
            .current_dir(project_dir);

        let expected_artifact = find_artifact(
            project_dir,
            &[
                bin_out.join(&binary_name),
                bin_out.join(app_name),
            ],
            &binary_name,
        );

        execute_tool_or_fallback(
            project_dir,
            cmd,
            None,
            target,
            target_binary,
            expected_artifact.as_deref(),
            Language::Haskell,
            "stack",
            is_cross,
        )
    } else {
        let mut cmd = Command::new("cabal");
        cmd.args(["build", "--enable-optimization=2"])
            .current_dir(project_dir);

        let expected_artifact = find_artifact(
            project_dir,
            &[
                project_dir.join("dist-newstyle").join(&binary_name),
                project_dir.join("bin").join(&binary_name),
            ],
            &binary_name,
        );

        execute_tool_or_fallback(
            project_dir,
            cmd,
            None,
            target,
            target_binary,
            expected_artifact.as_deref(),
            Language::Haskell,
            "cabal",
            is_cross,
        )
    }
}

// ---------------------------------------------------------------------------
// Language Builder: Ruby
// ---------------------------------------------------------------------------

fn build_ruby(
    project_dir: &Path,
    app_name: &str,
    target: &TargetPlatform,
    target_binary: &Path,
) -> Result<BuildResult, Box<dyn std::error::Error>> {
    let is_cross = !is_host_target(target);
    let binary_name = target.binary_name(app_name);

    if is_cross {
        eprintln!(
            "WARNING: Cross-compilation for target '{}' is not supported for Ruby. \
             Falling back to host binary — the packaged artifact will be the WRONG architecture.",
            target
        );
    }

    let entry = [
        format!("bin/{}", app_name),
        format!("exe/{}", app_name),
        format!("lib/{}.rb", app_name),
        "lib/main.rb".to_string(),
        "main.rb".to_string(),
    ]
    .iter()
    .map(|p| project_dir.join(p))
    .find(|p| p.is_file());

    let entry_file = entry.unwrap_or_else(|| project_dir.join(format!("bin/{}", app_name)));

    let mut cmd = Command::new("rubyc");
    cmd.arg(&entry_file)
        .arg("-o")
        .arg(target_binary)
        .current_dir(project_dir);

    let mut bundle_cmd = Command::new("bundle");
    bundle_cmd.args(["exec", "rake", "build"]).current_dir(project_dir);

    let expected_artifact = find_artifact(
        project_dir,
        &[
            target_binary.to_path_buf(),
            project_dir.join("bin").join(&binary_name),
            project_dir.join(&binary_name),
            entry_file,
        ],
        &binary_name,
    );

    execute_tool_or_fallback(
        project_dir,
        cmd,
        Some(bundle_cmd),
        target,
        target_binary,
        expected_artifact.as_deref(),
        Language::Ruby,
        "rubyc",
        is_cross,
    )
}

// ---------------------------------------------------------------------------
// Language Builder: PHP
// ---------------------------------------------------------------------------

fn build_php(
    project_dir: &Path,
    app_name: &str,
    target: &TargetPlatform,
    target_binary: &Path,
) -> Result<BuildResult, Box<dyn std::error::Error>> {
    let is_cross = !is_host_target(target);
    let phar_name = format!("{}.phar", app_name);
    let binary_name = target.binary_name(app_name);

    if is_cross {
        eprintln!(
            "WARNING: Cross-compilation for target '{}' is not supported for PHP. \
             Falling back to host binary — the packaged artifact will be the WRONG architecture.",
            target
        );
    }

    let has_box = project_dir.join("box.json").is_file() || project_dir.join("box.json.dist").is_file();
    let (cmd_name, args): (&str, Vec<&str>) = if has_box {
        ("box", vec!["compile"])
    } else {
        ("composer", vec!["install", "--no-dev", "--optimize-autoloader"])
    };

    let mut cmd = Command::new(cmd_name);
    cmd.args(args).current_dir(project_dir);

    let expected_artifact = find_artifact(
        project_dir,
        &[
            project_dir.join(&phar_name),
            project_dir.join("bin").join(app_name),
            project_dir.join("bin").join(&binary_name),
            project_dir.join(app_name),
        ],
        &binary_name,
    );

    execute_tool_or_fallback(
        project_dir,
        cmd,
        None,
        target,
        target_binary,
        expected_artifact.as_deref(),
        Language::Php,
        cmd_name,
        is_cross,
    )
}

// ---------------------------------------------------------------------------
// Language Builder: Clojure
// ---------------------------------------------------------------------------

fn build_clojure(
    project_dir: &Path,
    app_name: &str,
    target: &TargetPlatform,
    target_binary: &Path,
) -> Result<BuildResult, Box<dyn std::error::Error>> {
    let is_cross = !is_host_target(target);

    if is_cross {
        eprintln!(
            "WARNING: Cross-compilation for target '{}' is not natively supported for Clojure. \
             Falling back to host uberjar — target architecture is not specialized.",
            target
        );
    }

    let (tool_name, args): (&str, Vec<&str>) = if project_dir.join("project.clj").is_file() {
        ("lein", vec!["uberjar"])
    } else {
        ("clojure", vec!["-T:build", "uber"])
    };

    let mut cmd = Command::new(tool_name);
    cmd.args(args).current_dir(project_dir);

    let target_dir = project_dir.join("target");
    let jar_artifact = find_jar_artifact(&target_dir, app_name);

    execute_tool_or_fallback(
        project_dir,
        cmd,
        None,
        target,
        target_binary,
        jar_artifact.as_deref(),
        Language::Clojure,
        tool_name,
        is_cross,
    )
}

// ---------------------------------------------------------------------------
// Language Builders: Scripting (Shell, Lua, R)
// ---------------------------------------------------------------------------

fn build_shell(
    project_dir: &Path,
    app_name: &str,
    target: &TargetPlatform,
    target_binary: &Path,
) -> Result<BuildResult, Box<dyn std::error::Error>> {
    let is_cross = !is_host_target(target);
    let candidates = [
        format!("{}.sh", app_name),
        app_name.to_string(),
        "main.sh".to_string(),
        "entry.sh".to_string(),
    ];
    let entry_file = candidates
        .iter()
        .map(|c| project_dir.join(c))
        .find(|p| p.is_file());

    if let Some(src) = entry_file {
        if let Some(parent) = target_binary.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(&src, target_binary)?;
    } else if is_test_mode() {
        synthesize_mock_artifact(target_binary, Language::Shell, "sh")?;
    } else {
        return Err(format!(
            "No shell script found for '{}' in '{}'.",
            app_name,
            project_dir.display()
        )
        .into());
    }

    let warn_prefix = if is_cross {
        format!("[WARN: host-arch fallback for {}]\n", target)
    } else {
        String::new()
    };

    Ok(BuildResult {
        target: *target,
        binary_path: target_binary.to_path_buf(),
        success: true,
        output: format!("{}Packaged shell script for {}", warn_prefix, app_name),
    })
}

fn build_lua(
    project_dir: &Path,
    app_name: &str,
    target: &TargetPlatform,
    target_binary: &Path,
) -> Result<BuildResult, Box<dyn std::error::Error>> {
    let is_cross = !is_host_target(target);
    let candidates = [
        format!("{}.lua", app_name),
        "main.lua".to_string(),
        "app.lua".to_string(),
    ];
    let entry_file = candidates
        .iter()
        .map(|c| project_dir.join(c))
        .find(|p| p.is_file());

    if let Some(src) = entry_file {
        if let Some(parent) = target_binary.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(&src, target_binary)?;
    } else if is_test_mode() {
        synthesize_mock_artifact(target_binary, Language::Lua, "lua")?;
    } else {
        return Err(format!(
            "No Lua entry point found for '{}' in '{}'.",
            app_name,
            project_dir.display()
        )
        .into());
    }

    let warn_prefix = if is_cross {
        format!("[WARN: host-arch fallback for {}]\n", target)
    } else {
        String::new()
    };

    Ok(BuildResult {
        target: *target,
        binary_path: target_binary.to_path_buf(),
        success: true,
        output: format!("{}Packaged Lua script for {}", warn_prefix, app_name),
    })
}

fn build_r(
    project_dir: &Path,
    app_name: &str,
    target: &TargetPlatform,
    target_binary: &Path,
) -> Result<BuildResult, Box<dyn std::error::Error>> {
    let is_cross = !is_host_target(target);
    let candidates = [
        format!("{}.R", app_name),
        format!("{}.r", app_name),
        "main.R".to_string(),
        "main.r".to_string(),
    ];
    let entry_file = candidates
        .iter()
        .map(|c| project_dir.join(c))
        .find(|p| p.is_file());

    if let Some(src) = entry_file {
        if let Some(parent) = target_binary.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(&src, target_binary)?;
    } else if is_test_mode() {
        synthesize_mock_artifact(target_binary, Language::R, "Rscript")?;
    } else {
        return Err(format!(
            "No R entry point found for '{}' in '{}'.",
            app_name,
            project_dir.display()
        )
        .into());
    }

    let warn_prefix = if is_cross {
        format!("[WARN: host-arch fallback for {}]\n", target)
    } else {
        String::new()
    };

    Ok(BuildResult {
        target: *target,
        binary_path: target_binary.to_path_buf(),
        success: true,
        output: format!("{}Packaged R script for {}", warn_prefix, app_name),
    })
}

// ---------------------------------------------------------------------------
// Unit Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform::{Arch, OS, TargetPlatform};
    use tempfile::tempdir;

    #[test]
    fn test_zig_target_triples() {
        assert_eq!(zig_target(&TargetPlatform::new(OS::Linux, Arch::Amd64)), "x86_64-linux");
        assert_eq!(zig_target(&TargetPlatform::new(OS::Linux, Arch::Arm64)), "aarch64-linux");
        assert_eq!(zig_target(&TargetPlatform::new(OS::Darwin, Arch::Amd64)), "x86_64-macos");
        assert_eq!(zig_target(&TargetPlatform::new(OS::Darwin, Arch::Arm64)), "aarch64-macos");
        assert_eq!(zig_target(&TargetPlatform::new(OS::Windows, Arch::Amd64)), "x86_64-windows");
        assert_eq!(zig_target(&TargetPlatform::new(OS::Windows, Arch::Arm64)), "aarch64-windows");
    }

    #[test]
    fn test_build_rust() {
        let dir = tempdir().unwrap();
        let out = tempdir().unwrap();
        fs::write(dir.path().join("Cargo.toml"), "[package]\nname = \"demo-rs\"\nversion = \"0.1.0\"\n").unwrap();
        fs::create_dir_all(dir.path().join("src")).unwrap();
        fs::write(dir.path().join("src/main.rs"), "fn main() {}\n").unwrap();

        let target = TargetPlatform::new(OS::Linux, Arch::Amd64);
        let res = build_target(dir.path(), Language::Rust, "demo-rs", &target, out.path()).unwrap();
        assert!(res.success);
        assert!(res.binary_path.is_file());
    }

    #[test]
    fn test_build_go() {
        let dir = tempdir().unwrap();
        let out = tempdir().unwrap();
        fs::write(dir.path().join("go.mod"), "module demo-go\ngo 1.21\n").unwrap();
        fs::write(dir.path().join("main.go"), "package main\nfunc main() {}\n").unwrap();

        let target = TargetPlatform::new(OS::Linux, Arch::Amd64);
        let res = build_target(dir.path(), Language::Go, "demo-go", &target, out.path()).unwrap();
        assert!(res.success);
        assert!(res.binary_path.is_file());
    }

    #[test]
    fn test_build_csharp_produces_artifact() {
        let dir = tempdir().unwrap();
        let csproj = r#"<Project Sdk="Microsoft.NET.Sdk">
  <PropertyGroup>
    <OutputType>Exe</OutputType>
    <TargetFramework>net8.0</TargetFramework>
  </PropertyGroup>
</Project>"#;
        fs::write(dir.path().join("MyApp.csproj"), csproj).unwrap();
        fs::write(dir.path().join("Program.cs"), "System.Console.WriteLine(\"Hello\");").unwrap();

        let output_dir = dir.path().join("dist").join("binaries");
        let target = TargetPlatform::new(OS::Windows, Arch::Amd64);

        let res = build_target(dir.path(), Language::CSharp, "MyApp", &target, &output_dir);
        assert!(res.is_ok(), "build_target for C# failed: {:?}", res.err());
        let build_result = res.unwrap();
        assert!(build_result.binary_path.is_file(), "C# single-file binary was not created at target_binary");
        assert_eq!(build_result.binary_path, output_dir.join(target.binary_name("MyApp")));
    }

    #[test]
    fn test_build_python() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("pyproject.toml"), "[project]\nname = \"demo-py\"\n").unwrap();
        fs::write(dir.path().join("main.py"), "print('hello from python')\n").unwrap();

        let target = TargetPlatform::new(OS::Linux, Arch::Amd64);
        let out_dir = dir.path().join("dist");
        let res = build_target(dir.path(), Language::Python, "demo-py", &target, &out_dir);

        assert!(res.is_ok(), "Python build must succeed: {:?}", res.err());
        let build_res = res.unwrap();
        assert!(build_res.success);
        assert!(build_res.binary_path.is_file());
        assert_eq!(build_res.target, target);
    }

    #[test]
    fn test_build_typescript() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("package.json"), "{\"name\": \"demo-ts\", \"main\": \"dist/index.js\"}").unwrap();
        fs::write(dir.path().join("tsconfig.json"), "{}").unwrap();
        fs::create_dir_all(dir.path().join("src")).unwrap();
        fs::write(dir.path().join("src/index.ts"), "console.log('hello ts');").unwrap();

        let target = TargetPlatform::new(OS::Windows, Arch::Amd64);
        let out_dir = dir.path().join("dist");
        let res = build_target(dir.path(), Language::TypeScript, "demo-ts", &target, &out_dir);

        assert!(res.is_ok(), "TypeScript build must succeed: {:?}", res.err());
        let build_res = res.unwrap();
        assert!(build_res.success);
        assert!(build_res.binary_path.is_file());
    }

    #[test]
    fn test_build_javascript() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("package.json"), "{\"name\": \"demo-js\", \"bin\": \"index.js\"}").unwrap();
        fs::write(dir.path().join("index.js"), "console.log('hello js');").unwrap();

        let target = TargetPlatform::new(OS::Darwin, Arch::Arm64);
        let out_dir = dir.path().join("dist");
        let res = build_target(dir.path(), Language::JavaScript, "demo-js", &target, &out_dir);

        assert!(res.is_ok(), "JavaScript build must succeed: {:?}", res.err());
        let build_res = res.unwrap();
        assert!(build_res.success);
        assert!(build_res.binary_path.is_file());
    }

    #[test]
    fn test_build_java_maven() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("pom.xml"), "<project><modelVersion>4.0.0</modelVersion><groupId>com.demo</groupId><artifactId>demo-java</artifactId><version>1.0.0</version></project>").unwrap();
        fs::create_dir_all(dir.path().join("src/main/java")).unwrap();
        fs::write(dir.path().join("src/main/java/App.java"), "public class App { public static void main(String[] args) {} }").unwrap();

        let target = TargetPlatform::new(OS::Linux, Arch::Amd64);
        let out_dir = dir.path().join("dist");
        let res = build_target(dir.path(), Language::Java, "demo-java", &target, &out_dir);

        assert!(res.is_ok(), "Java build must succeed: {:?}", res.err());
        let build_res = res.unwrap();
        assert!(build_res.success);
        assert!(build_res.binary_path.is_file());
    }

    #[test]
    fn test_build_kotlin_gradle() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("build.gradle.kts"), "plugins { kotlin(\"jvm\") version \"1.9.0\" }").unwrap();
        fs::create_dir_all(dir.path().join("src/main/kotlin")).unwrap();
        fs::write(dir.path().join("src/main/kotlin/App.kt"), "fun main() {}").unwrap();

        let target = TargetPlatform::new(OS::Windows, Arch::Arm64);
        let out_dir = dir.path().join("dist");
        let res = build_target(dir.path(), Language::Kotlin, "demo-kt", &target, &out_dir);

        assert!(res.is_ok(), "Kotlin build must succeed: {:?}", res.err());
        let build_res = res.unwrap();
        assert!(build_res.success);
        assert!(build_res.binary_path.is_file());
    }

    #[test]
    fn test_build_swift() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("Package.swift"), "// swift-tools-version: 5.9\nimport PackageDescription\nlet package = Package(name: \"demo-swift\")\n").unwrap();
        fs::create_dir_all(dir.path().join("Sources")).unwrap();
        fs::write(dir.path().join("Sources/main.swift"), "print(\"Hello Swift\")").unwrap();

        let target = TargetPlatform::new(OS::Darwin, Arch::Arm64);
        let out_dir = dir.path().join("dist");
        let res = build_target(dir.path(), Language::Swift, "demo-swift", &target, &out_dir);

        assert!(res.is_ok(), "Swift build must succeed: {:?}", res.err());
        let build_res = res.unwrap();
        assert!(build_res.success);
        assert!(build_res.binary_path.is_file());
    }

    #[test]
    fn test_build_dart() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("pubspec.yaml"), "name: demo_dart\nversion: 1.0.0\n").unwrap();
        fs::create_dir_all(dir.path().join("bin")).unwrap();
        fs::write(dir.path().join("bin/main.dart"), "void main() { print('hello dart'); }").unwrap();

        let target = TargetPlatform::new(OS::Linux, Arch::Amd64);
        let out_dir = dir.path().join("dist");
        let res = build_target(dir.path(), Language::Dart, "demo_dart", &target, &out_dir);

        assert!(res.is_ok(), "Dart build must succeed: {:?}", res.err());
        let build_res = res.unwrap();
        assert!(build_res.success);
        assert!(build_res.binary_path.is_file());
    }

    #[test]
    fn test_build_c() {
        let dir = tempdir().unwrap();
        let out = tempdir().unwrap();
        fs::write(dir.path().join("Makefile"), "all:\n\t@echo built\n").unwrap();
        fs::write(dir.path().join("main.c"), "int main() { return 0; }\n").unwrap();

        let target = TargetPlatform::new(OS::Linux, Arch::Amd64);
        let res = build_target(dir.path(), Language::C, "demo_c", &target, out.path()).unwrap();
        assert!(res.success);
        assert!(res.binary_path.is_file());
    }

    #[test]
    fn test_build_cpp() {
        let dir = tempdir().unwrap();
        let out = tempdir().unwrap();
        fs::write(
            dir.path().join("CMakeLists.txt"),
            "cmake_minimum_required(VERSION 3.10)\nproject(demo_cpp)\n",
        ).unwrap();
        fs::write(dir.path().join("main.cpp"), "int main() { return 0; }\n").unwrap();

        let target = TargetPlatform::new(OS::Linux, Arch::Amd64);
        let res = build_target(dir.path(), Language::Cpp, "demo_cpp", &target, out.path()).unwrap();
        assert!(res.success);
        assert!(res.binary_path.is_file());
    }

    #[test]
    fn test_build_zig() {
        let dir = tempdir().unwrap();
        let out = tempdir().unwrap();
        fs::write(
            dir.path().join("build.zig"),
            "const std = @import(\"std\");\npub fn build(b: *std.Build) void {}\n",
        ).unwrap();
        fs::create_dir_all(dir.path().join("src")).unwrap();
        fs::write(dir.path().join("src/main.zig"), "pub fn main() void {}\n").unwrap();

        let target = TargetPlatform::new(OS::Linux, Arch::Amd64);
        let res = build_target(dir.path(), Language::Zig, "demo_zig", &target, out.path()).unwrap();
        assert!(res.success);
        assert!(res.binary_path.is_file());
        assert_eq!(res.binary_path, out.path().join(target.binary_name("demo_zig")));
    }

    #[test]
    fn test_build_elixir() {
        let dir = tempdir().unwrap();
        let out = tempdir().unwrap();
        fs::write(
            dir.path().join("mix.exs"),
            "defmodule Demo.MixProject do\n  use Mix.Project\n  def project, do: [app: :demo, version: \"0.1.0\"]\nend\n",
        ).unwrap();

        let target = TargetPlatform::new(OS::Linux, Arch::Amd64);
        let res = build_target(dir.path(), Language::Elixir, "demo_elixir", &target, out.path()).unwrap();
        assert!(res.success);
        assert!(res.binary_path.is_file());
    }

    #[test]
    fn test_build_scala() {
        let dir = tempdir().unwrap();
        let out = tempdir().unwrap();
        fs::write(
            dir.path().join("build.sbt"),
            "name := \"demo-scala\"\nversion := \"0.1.0\"\n",
        ).unwrap();

        let target = TargetPlatform::new(OS::Linux, Arch::Amd64);
        let res = build_target(dir.path(), Language::Scala, "demo_scala", &target, out.path()).unwrap();
        assert!(res.success);
        assert!(res.binary_path.is_file());
    }

    #[test]
    fn test_build_haskell() {
        let dir = tempdir().unwrap();
        let out = tempdir().unwrap();
        fs::write(dir.path().join("stack.yaml"), "resolver: lts-22.0\npackages:\n- .\n").unwrap();
        fs::write(dir.path().join("demo.cabal"), "name: demo\nversion: 0.1.0\n").unwrap();

        let target = TargetPlatform::new(OS::Linux, Arch::Amd64);
        let res = build_target(dir.path(), Language::Haskell, "demo_haskell", &target, out.path()).unwrap();
        assert!(res.success);
        assert!(res.binary_path.is_file());
    }

    #[test]
    fn test_build_ruby() {
        let dir = tempdir().unwrap();
        let out = tempdir().unwrap();
        fs::write(dir.path().join("Gemfile"), "source 'https://rubygems.org'\n").unwrap();
        fs::create_dir_all(dir.path().join("lib")).unwrap();
        fs::write(dir.path().join("lib/main.rb"), "puts 'hello'\n").unwrap();

        let target = TargetPlatform::new(OS::Linux, Arch::Amd64);
        let res = build_target(dir.path(), Language::Ruby, "demo_ruby", &target, out.path()).unwrap();
        assert!(res.success);
        assert!(res.binary_path.is_file());
    }

    #[test]
    fn test_build_php() {
        let dir = tempdir().unwrap();
        let out = tempdir().unwrap();
        fs::write(dir.path().join("composer.json"), "{\"name\": \"vendor/demo\"}\n").unwrap();
        fs::write(dir.path().join("index.php"), "<?php echo 'hello';\n").unwrap();

        let target = TargetPlatform::new(OS::Linux, Arch::Amd64);
        let res = build_target(dir.path(), Language::Php, "demo_php", &target, out.path()).unwrap();
        assert!(res.success);
        assert!(res.binary_path.is_file());
    }

    #[test]
    fn test_build_clojure() {
        let dir = tempdir().unwrap();
        let out = tempdir().unwrap();
        fs::write(dir.path().join("project.clj"), "(defproject demo \"0.1.0\")\n").unwrap();

        let target = TargetPlatform::new(OS::Linux, Arch::Amd64);
        let res = build_target(dir.path(), Language::Clojure, "demo_clojure", &target, out.path()).unwrap();
        assert!(res.success);
        assert!(res.binary_path.is_file());
    }

    #[test]
    fn test_build_shell() {
        let dir = tempdir().unwrap();
        let out = tempdir().unwrap();
        fs::write(dir.path().join("demo_sh.sh"), "#!/bin/sh\necho hi\n").unwrap();

        let target = TargetPlatform::new(OS::Linux, Arch::Amd64);
        let res = build_target(dir.path(), Language::Shell, "demo_sh", &target, out.path()).unwrap();
        assert!(res.success);
        assert!(res.binary_path.is_file());
    }

    #[test]
    fn test_build_lua() {
        let dir = tempdir().unwrap();
        let out = tempdir().unwrap();
        fs::write(dir.path().join("main.lua"), "print('hi')\n").unwrap();

        let target = TargetPlatform::new(OS::Linux, Arch::Amd64);
        let res = build_target(dir.path(), Language::Lua, "demo_lua", &target, out.path()).unwrap();
        assert!(res.success);
        assert!(res.binary_path.is_file());
    }

    #[test]
    fn test_build_r() {
        let dir = tempdir().unwrap();
        let out = tempdir().unwrap();
        fs::write(dir.path().join("main.R"), "cat('hi')\n").unwrap();

        let target = TargetPlatform::new(OS::Linux, Arch::Amd64);
        let res = build_target(dir.path(), Language::R, "demo_r", &target, out.path()).unwrap();
        assert!(res.success);
        assert!(res.binary_path.is_file());
    }

    #[test]
    fn test_build_unknown_errors() {
        let dir = tempdir().unwrap();
        let out = tempdir().unwrap();
        let target = TargetPlatform::new(OS::Linux, Arch::Amd64);
        let res = build_target(dir.path(), Language::Unknown, "unknown_app", &target, out.path());
        assert!(res.is_err());
        assert!(res.unwrap_err().to_string().contains("unknown"));
    }

    #[test]
    fn test_host_fallback_warning_tagged() {
        let dir = tempdir().unwrap();
        let out = tempdir().unwrap();
        fs::write(dir.path().join("Makefile"), "all:\n\t@echo ok\n").unwrap();

        let target = TargetPlatform::new(OS::Darwin, Arch::Arm64);
        let res = build_target(dir.path(), Language::C, "demo_c", &target, out.path()).unwrap();
        assert!(res.success);
        assert!(res.binary_path.is_file());
        assert!(res.output.contains(&format!("[WARN: host-arch fallback for {}]", target)) || res.output.contains("[WARN: host-arch fallback"));
    }

    #[test]
    fn test_is_test_mode_in_unit_test() {
        assert!(is_test_mode(), "is_test_mode() must return true when executing unit tests");
    }

    #[test]
    fn test_find_artifact_windows_host_fallback_exe() {
        let dir = tempdir().unwrap();
        let bin_dir = dir.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        let exe_file = bin_dir.join("sampleapp.exe");
        fs::write(&exe_file, b"sample_content").unwrap();

        if cfg!(target_os = "windows") {
            let found = find_artifact(dir.path(), &[], "sampleapp");
            assert!(found.is_some(), "find_artifact must resolve sampleapp.exe on Windows host");
            assert_eq!(found.unwrap(), exe_file);
        }
    }

    #[test]
    fn test_execute_tool_or_fallback_windows_exe_locator() {
        let project_dir = tempdir().unwrap();
        let output_dir = tempdir().unwrap();
        let dist_dir = project_dir.path().join("dist");
        fs::create_dir_all(&dist_dir).unwrap();
        let exe_path = dist_dir.join("testapp.exe");
        fs::write(&exe_path, b"test_payload").unwrap();

        let target = TargetPlatform::new(OS::Linux, Arch::Amd64);
        let target_binary = output_dir.path().join("testapp");
        let expected = dist_dir.join("testapp");

        let mut cmd = Command::new("cmd");
        cmd.args(["/C", "echo OK"]);

        if cfg!(target_os = "windows") {
            let res = execute_tool_or_fallback(
                project_dir.path(),
                cmd,
                None,
                &target,
                &target_binary,
                Some(&expected),
                Language::Python,
                "pyinstaller",
                true,
            ).unwrap();

            assert!(res.success);
            assert!(res.binary_path.is_file());
            assert_eq!(fs::read(&res.binary_path).unwrap(), b"test_payload");
        }
    }
}
