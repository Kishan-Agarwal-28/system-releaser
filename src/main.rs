use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::process;
use system_releaser::{
    detect_language, execute_release, generate_github_action_workflow, init_project,
    run_preflight_checks, run_project_tests, CheckStatus, InitOptions, Language, ReleaseOptions,
    VersionBump,
};

#[derive(Parser, Debug)]
#[command(
    name = "system-releaser",
    about = "Automated release manager - language detection & universal package distribution",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Path to target project directory (used when no subcommand is specified)
    #[arg(default_value = ".")]
    path: PathBuf,

    /// Output only the detected language name (for scripts/pipelines)
    #[arg(short = 's', long = "short", conflicts_with = "json")]
    short: bool,

    /// Output full detection analysis in JSON format
    #[arg(short, long)]
    json: bool,

    /// Show detailed verbose analysis including file breakdown and evidence
    #[arg(short, long, conflicts_with = "short")]
    verbose: bool,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Detect project language and build system
    Detect {
        /// Target project directory
        #[arg(default_value = ".")]
        path: PathBuf,

        /// Output only language name
        #[arg(short = 's', long = "short")]
        short: bool,

        /// Output in JSON
        #[arg(short, long)]
        json: bool,

        /// Verbose breakdown
        #[arg(short, long)]
        verbose: bool,
    },

    /// Initialize a releaser.yaml configuration for the project
    Init {
        /// Target project directory
        #[arg(default_value = ".")]
        path: PathBuf,

        /// Overwrite existing configuration if present
        #[arg(short, long)]
        force: bool,

        /// Include full template with all supported package managers
        #[arg(long)]
        full: bool,

        /// Explicit project name override
        #[arg(short, long)]
        name: Option<String>,
    },

    /// Generate a 10-line GitHub Actions release workflow (.github/workflows/release.yml)
    InitCi {
        /// Target project directory
        #[arg(default_value = ".")]
        path: PathBuf,
    },

    /// Run preflight checks on project config, git status, and toolchains
    Check {
        /// Target project directory
        #[arg(default_value = ".")]
        path: PathBuf,
    },

    /// Run the project's native test suite
    Test {
        /// Target project directory
        #[arg(default_value = ".")]
        path: PathBuf,
    },

    /// Build cross-platform binaries, archives, installers, and package manager manifests
    Release {
        /// Target project directory
        #[arg(default_value = ".")]
        path: PathBuf,

        /// Bump patch version (e.g. 1.0.0 -> 1.0.1)
        #[arg(long, conflicts_with_all = ["minor", "major", "set_version"])]
        patch: bool,

        /// Bump minor version (e.g. 1.0.0 -> 1.1.0)
        #[arg(long, conflicts_with_all = ["patch", "major", "set_version"])]
        minor: bool,

        /// Bump major version (e.g. 1.0.0 -> 2.0.0)
        #[arg(long, conflicts_with_all = ["patch", "minor", "set_version"])]
        major: bool,

        /// Explicit version to release (e.g. 1.5.0)
        #[arg(long, value_name = "SEMVER", conflicts_with_all = ["patch", "minor", "major"])]
        set_version: Option<String>,

        /// Only build and package for the current host architecture (e.g. windows/amd64)
        #[arg(long, alias = "host")]
        host_only: bool,

        /// Target specific platform(s) to compile (e.g. linux/amd64, windows/amd64). Repeatable.
        #[arg(long = "platform", value_name = "PLATFORM")]
        platforms: Vec<String>,

        /// Skip running tests prior to release
        #[arg(long)]
        skip_tests: bool,

        /// Dry run: perform build and packaging without creating git commits or tags
        #[arg(long)]
        dry_run: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Detect {
            path,
            short,
            json,
            verbose,
        }) => {
            handle_detect(path, short, json, verbose);
        }
        Some(Commands::Init {
            path,
            force,
            full,
            name,
        }) => {
            let target_dir = resolve_path(&path);
            println!("Initializing releaser configuration in '{}'...", target_dir.display());

            let options = InitOptions {
                target_dir: target_dir.clone(),
                force,
                full,
                name,
            };

            match init_project(options) {
                Ok(res) => {
                    println!("\nSuccessfully initialized releaser configuration!");
                    println!("  Project Name:     {}", res.project_name);
                    println!("  Detected Language:{}", res.language);
                    println!("  Package Managers: {} enabled", res.package_managers_count);
                    println!("  Config File:      {}", res.config_path.display());
                    println!("\nNext steps:");
                    println!("  1. Review and customize '{}'", res.config_path.file_name().unwrap().to_string_lossy());
                    println!("  2. Run 'system-releaser check' to verify prerequisites");
                    println!("  3. Run 'system-releaser release' to compile and distribute");
                }
                Err(err) => {
                    eprintln!("Error initializing project: {}", err);
                    process::exit(1);
                }
            }
        }
        Some(Commands::InitCi { path }) => {
            let target_dir = resolve_path(&path);
            match generate_github_action_workflow(&target_dir) {
                Ok(wf) => {
                    println!("Successfully created GitHub Actions workflow at '{}'!", wf.display());
                }
                Err(err) => {
                    eprintln!("Error creating GitHub Actions workflow: {}", err);
                    process::exit(1);
                }
            }
        }
        Some(Commands::Check { path }) => {
            let target_dir = resolve_path(&path);
            println!("Running preflight release checks for '{}'...\n", target_dir.display());

            let report = run_preflight_checks(&target_dir);
            for item in &report.items {
                let badge = match item.status {
                    CheckStatus::Pass => "[PASS]",
                    CheckStatus::Warning => "[WARN]",
                    CheckStatus::Fail => "[FAIL]",
                };
                println!("{:<8} {:<24} : {}", badge, item.name, item.message);
            }

            println!();
            if report.is_clean() {
                println!("All checks passed! The project is ready for release.");
            } else {
                eprintln!("Some checks failed. Resolve the issues above before releasing.");
                process::exit(1);
            }
        }
        Some(Commands::Test { path }) => {
            let target_dir = resolve_path(&path);
            let detection = detect_language(&target_dir).unwrap_or_else(|e| {
                eprintln!("Error detecting project: {}", e);
                process::exit(1);
            });

            println!(
                "Running {} tests for '{}'...\n",
                detection.primary_language,
                target_dir.display()
            );

            match run_project_tests(&target_dir, detection.primary_language) {
                Ok(res) => {
                    if !res.output.is_empty() {
                        println!("{}", res.output);
                    }
                    if res.success {
                        println!("\nTests passed successfully!");
                    } else {
                        eprintln!("\nTests failed (command: '{}')", res.command);
                        process::exit(1);
                    }
                }
                Err(err) => {
                    eprintln!("Error executing tests: {}", err);
                    process::exit(1);
                }
            }
        }
        Some(Commands::Release {
            path,
            patch,
            minor,
            major,
            set_version,
            host_only,
            platforms,
            skip_tests,
            dry_run,
        }) => {
            let target_dir = resolve_path(&path);
            let bump = if patch {
                Some(VersionBump::Patch)
            } else if minor {
                Some(VersionBump::Minor)
            } else if major {
                Some(VersionBump::Major)
            } else if let Some(v_str) = set_version {
                match v_str.parse::<VersionBump>() {
                    Ok(b) => Some(b),
                    Err(e) => {
                        eprintln!("Error parsing version: {}", e);
                        process::exit(1);
                    }
                }
            } else {
                None
            };

            let target_platforms = if platforms.is_empty() {
                None
            } else {
                let mut list = Vec::new();
                for p in &platforms {
                    match p.parse::<system_releaser::TargetPlatform>() {
                        Ok(tp) => list.push(tp),
                        Err(e) => {
                            eprintln!("Error parsing platform target '{}': {}", p, e);
                            process::exit(1);
                        }
                    }
                }
                Some(list)
            };

            let options = ReleaseOptions {
                project_dir: target_dir,
                bump,
                host_only,
                target_platforms,
                skip_tests,
                dry_run,
            };

            match execute_release(options) {
                Ok(summary) => {
                    println!("\n=======================================================");
                    println!("Release v{} successfully assembled!", summary.version);
                    println!("=======================================================");
                    println!("\nRelease Archives:");
                    for arc in &summary.archives {
                        println!("  - {}", arc.file_name().unwrap().to_string_lossy());
                    }
                    println!("\nIntegrity Checksums:");
                    println!("  - {}", summary.checksums_file.file_name().unwrap().to_string_lossy());
                    println!("\nPackage Manager Manifests & Installers:");
                    for man in &summary.manifests {
                        println!("  - {}", man.file_name().unwrap().to_string_lossy());
                    }
                    println!("\nAll assets are ready in the output directory!");
                }
                Err(err) => {
                    eprintln!("Release failed: {}", err);
                    process::exit(1);
                }
            }
        }
        None => {
            // Default action when run without subcommand: detection
            handle_detect(cli.path, cli.short, cli.json, cli.verbose);
        }
    }
}

fn resolve_path(p: &std::path::Path) -> PathBuf {
    match p.canonicalize() {
        Ok(canonical) => {
            #[cfg(windows)]
            {
                let s = canonical.to_string_lossy();
                if let Some(stripped) = s.strip_prefix(r"\\?\UNC\") {
                    PathBuf::from(format!(r"\\{}", stripped))
                } else if let Some(stripped) = s.strip_prefix(r"\\?\") {
                    PathBuf::from(stripped)
                } else {
                    canonical
                }
            }
            #[cfg(not(windows))]
            canonical
        }
        Err(_) => p.to_path_buf(),
    }
}

fn handle_detect(path: PathBuf, short: bool, json: bool, verbose: bool) {
    let target_dir = resolve_path(&path);

    match detect_language(&target_dir) {
        Ok(result) => {
            if json {
                let json_output = serde_json::to_string_pretty(&result).unwrap_or_else(|e| {
                    eprintln!("Error formatting JSON: {}", e);
                    process::exit(1);
                });
                println!("{}", json_output);
                return;
            }

            if short {
                println!("{}", result.primary_language);
                if result.primary_language == Language::Unknown {
                    process::exit(1);
                }
                return;
            }

            let display_path = target_dir
                .to_string_lossy()
                .strip_prefix(r"\\?\")
                .map(ToString::to_string)
                .unwrap_or_else(|| target_dir.display().to_string());

            println!("Target: {}", display_path);
            println!("Detected Language: {}", result.primary_language);
            println!("Confidence: {}", result.confidence);

            if let Some(tool) = result.primary_language.default_tool() {
                println!("Build/Package Tool: {}", tool);
            }

            if !result.detected_manifests.is_empty() {
                println!("\nManifests found:");
                for manifest in &result.detected_manifests {
                    println!("  - {}", manifest);
                }
            }

            if verbose || !result.evidence.is_empty() {
                println!("\nEvidence:");
                for item in &result.evidence {
                    println!("  * {}", item);
                }
            }

            if verbose && !result.language_breakdown.is_empty() {
                println!(
                    "\nSource Files Breakdown (total: {} files):",
                    result.total_source_files
                );
                for stat in &result.language_breakdown {
                    println!(
                        "  - {:<12}: {:>4} files ({:>5.1}%)",
                        stat.language.name(),
                        stat.file_count,
                        stat.percentage
                    );
                }
            }

            if result.primary_language == Language::Unknown {
                process::exit(1);
            }
        }
        Err(err) => {
            eprintln!("Error detecting language: {}", err);
            process::exit(1);
        }
    }
}
