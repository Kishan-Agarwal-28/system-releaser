use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use serde::{Deserialize, Serialize};

use crate::language::Language;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Confidence {
    High,
    Medium,
    Low,
}

impl std::fmt::Display for Confidence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Confidence::High => write!(f, "High"),
            Confidence::Medium => write!(f, "Medium"),
            Confidence::Low => write!(f, "Low"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageStat {
    pub language: Language,
    pub file_count: usize,
    pub percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionResult {
    pub primary_language: Language,
    pub confidence: Confidence,
    pub detected_manifests: Vec<String>,
    pub evidence: Vec<String>,
    pub language_breakdown: Vec<LanguageStat>,
    pub total_source_files: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workspace_root: Option<String>,
}

const IGNORED_DIRS: &[&str] = &[
    ".git",
    "target",
    "node_modules",
    ".vscode",
    ".idea",
    "dist",
    "build",
    "out",
    ".next",
    ".nuxt",
    "vendor",
    "venv",
    ".venv",
    "__pycache__",
    "bin",
    "obj",
    ".gradle",
    ".cargo",
    ".turbo",
    ".cache",
];

/// Manifest configuration definition: (filename, associated language, description)
struct ManifestRule {
    file_name: &'static str,
    language: Language,
    description: &'static str,
}

const MANIFEST_RULES: &[ManifestRule] = &[
    // Rust
    ManifestRule {
        file_name: "Cargo.toml",
        language: Language::Rust,
        description: "Cargo manifest (Cargo.toml)",
    },
    // Go
    ManifestRule {
        file_name: "go.mod",
        language: Language::Go,
        description: "Go module file (go.mod)",
    },
    ManifestRule {
        file_name: "go.work",
        language: Language::Go,
        description: "Go workspace file (go.work)",
    },
    // Python
    ManifestRule {
        file_name: "pyproject.toml",
        language: Language::Python,
        description: "Python project specification (pyproject.toml)",
    },
    ManifestRule {
        file_name: "setup.py",
        language: Language::Python,
        description: "Python setup script (setup.py)",
    },
    ManifestRule {
        file_name: "Pipfile",
        language: Language::Python,
        description: "Pipenv manifest (Pipfile)",
    },
    ManifestRule {
        file_name: "poetry.lock",
        language: Language::Python,
        description: "Poetry lockfile (poetry.lock)",
    },
    ManifestRule {
        file_name: "requirements.txt",
        language: Language::Python,
        description: "Python requirements (requirements.txt)",
    },
    // TypeScript / JavaScript
    ManifestRule {
        file_name: "tsconfig.json",
        language: Language::TypeScript,
        description: "TypeScript configuration (tsconfig.json)",
    },
    ManifestRule {
        file_name: "deno.json",
        language: Language::TypeScript,
        description: "Deno configuration (deno.json)",
    },
    ManifestRule {
        file_name: "deno.jsonc",
        language: Language::TypeScript,
        description: "Deno configuration (deno.jsonc)",
    },
    ManifestRule {
        file_name: "package.json",
        language: Language::JavaScript, // May be upgraded to TypeScript if tsconfig or .ts files exist
        description: "Node.js package manifest (package.json)",
    },
    ManifestRule {
        file_name: "bun.lockb",
        language: Language::JavaScript,
        description: "Bun lockfile (bun.lockb)",
    },
    // Java / Kotlin / JVM
    ManifestRule {
        file_name: "pom.xml",
        language: Language::Java,
        description: "Maven POM (pom.xml)",
    },
    ManifestRule {
        file_name: "build.gradle.kts",
        language: Language::Kotlin,
        description: "Gradle Kotlin DSL (build.gradle.kts)",
    },
    ManifestRule {
        file_name: "build.gradle",
        language: Language::Java,
        description: "Gradle build script (build.gradle)",
    },
    // Ruby
    ManifestRule {
        file_name: "Gemfile",
        language: Language::Ruby,
        description: "Ruby Gemfile",
    },
    // PHP
    ManifestRule {
        file_name: "composer.json",
        language: Language::Php,
        description: "PHP Composer manifest (composer.json)",
    },
    // Swift
    ManifestRule {
        file_name: "Package.swift",
        language: Language::Swift,
        description: "Swift Package manifest (Package.swift)",
    },
    // Dart / Flutter
    ManifestRule {
        file_name: "pubspec.yaml",
        language: Language::Dart,
        description: "Dart / Flutter pubspec (pubspec.yaml)",
    },
    // Zig
    ManifestRule {
        file_name: "build.zig",
        language: Language::Zig,
        description: "Zig build file (build.zig)",
    },
    // Elixir
    ManifestRule {
        file_name: "mix.exs",
        language: Language::Elixir,
        description: "Elixir Mix file (mix.exs)",
    },
    // Scala
    ManifestRule {
        file_name: "build.sbt",
        language: Language::Scala,
        description: "SBT build file (build.sbt)",
    },
    // Haskell
    ManifestRule {
        file_name: "stack.yaml",
        language: Language::Haskell,
        description: "Haskell Stack configuration (stack.yaml)",
    },
    // Clojure
    ManifestRule {
        file_name: "project.clj",
        language: Language::Clojure,
        description: "Leiningen project file (project.clj)",
    },
    ManifestRule {
        file_name: "deps.edn",
        language: Language::Clojure,
        description: "Clojure CLI deps (deps.edn)",
    },
    // C / C++ build systems
    ManifestRule {
        file_name: "CMakeLists.txt",
        language: Language::Cpp,
        description: "CMake build file (CMakeLists.txt)",
    },
    ManifestRule {
        file_name: "meson.build",
        language: Language::C,
        description: "Meson build file (meson.build)",
    },
    ManifestRule {
        file_name: "Makefile",
        language: Language::C,
        description: "Standard Makefile",
    },
];

/// Check if any ancestor directory represents a monorepo workspace root
pub fn find_enclosing_workspace_root(start_dir: &Path) -> Option<(PathBuf, &'static str)> {
    let mut current = start_dir.parent();
    let mut depth = 0;
    while let Some(parent) = current {
        depth += 1;
        if depth > 6 {
            break;
        }

        // Check for Cargo workspace
        let cargo_toml = parent.join("Cargo.toml");
        if cargo_toml.is_file()
            && fs::read_to_string(&cargo_toml)
                .map(|c| c.contains("[workspace]"))
                .unwrap_or(false)
        {
            return Some((parent.to_path_buf(), "Cargo workspace ([workspace])"));
        }

        // Check for pnpm workspace
        if parent.join("pnpm-workspace.yaml").is_file() {
            return Some((parent.to_path_buf(), "pnpm workspace (pnpm-workspace.yaml)"));
        }

        // Check for npm/yarn/bun workspaces in package.json
        let pkg_json = parent.join("package.json");
        if pkg_json.is_file()
            && fs::read_to_string(&pkg_json)
                .map(|c| c.contains("\"workspaces\""))
                .unwrap_or(false)
        {
            return Some((parent.to_path_buf(), "npm/yarn/bun workspace (package.json workspaces)"));
        }

        // Check for lerna, turborepo, nx
        if parent.join("lerna.json").is_file() {
            return Some((parent.to_path_buf(), "Lerna workspace (lerna.json)"));
        }
        if parent.join("turbo.json").is_file() {
            return Some((parent.to_path_buf(), "Turborepo workspace (turbo.json)"));
        }
        if parent.join("nx.json").is_file() {
            return Some((parent.to_path_buf(), "Nx workspace (nx.json)"));
        }

        // Check for Go workspace
        if parent.join("go.work").is_file() {
            return Some((parent.to_path_buf(), "Go workspace (go.work)"));
        }

        // Check for Gradle multi-project
        let settings_gradle = parent.join("settings.gradle");
        let settings_gradle_kts = parent.join("settings.gradle.kts");
        if settings_gradle.is_file() || settings_gradle_kts.is_file() {
            let content = fs::read_to_string(&settings_gradle)
                .or_else(|_| fs::read_to_string(&settings_gradle_kts))
                .unwrap_or_default();
            if content.contains("include(") || content.contains("include ") || content.contains("includeBuild") {
                return Some((parent.to_path_buf(), "Gradle multi-project (settings.gradle)"));
            }
        }

        // Stop ascending at git root boundary
        if parent.join(".git").exists() {
            break;
        }

        current = parent.parent();
    }
    None
}

/// Detects the language of a project located at `root_path`.
pub fn detect_language(root_path: impl AsRef<Path>) -> Result<DetectionResult, std::io::Error> {
    let path = root_path.as_ref();
    if !path.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Path '{}' does not exist", path.display()),
        ));
    }

    let mut evidence = Vec::new();
    let mut detected_manifests = Vec::new();
    let mut manifest_languages: HashMap<Language, Vec<String>> = HashMap::new();

    // Check for enclosing workspace root (monorepo awareness)
    let workspace_info = find_enclosing_workspace_root(path);
    let workspace_root = if let Some((ref ws_path, desc)) = workspace_info {
        evidence.push(format!(
            "Detected enclosing workspace root at '{}' ({})",
            ws_path.display(),
            desc
        ));
        Some(ws_path.to_string_lossy().to_string())
    } else {
        None
    };

    // 1. Check exact manifest files in root directory
    for rule in MANIFEST_RULES {
        let manifest_path = path.join(rule.file_name);
        if manifest_path.is_file() {
            detected_manifests.push(rule.file_name.to_string());
            evidence.push(format!("Found {}", rule.description));
            manifest_languages
                .entry(rule.language)
                .or_default()
                .push(rule.file_name.to_string());
        }
    }

    // 1b. If no manifest found in current dir, check parent for workspace root manifest
    if detected_manifests.is_empty() && path.parent().is_some() {
        let parent = path.parent().unwrap();
        for rule in MANIFEST_RULES {
                let p_manifest = parent.join(rule.file_name);
                if p_manifest.is_file() {
                    detected_manifests.push(format!("../{}", rule.file_name));
                    evidence.push(format!("Found workspace {} in parent directory", rule.description));
                    manifest_languages
                        .entry(rule.language)
                        .or_default()
                        .push(rule.file_name.to_string());
                    break;
                }
            }
    }

    // 2. Check for wildcard extensions in root (e.g. .csproj, .sln, .gemspec, .cabal)
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let entry_path = entry.path();
            if entry_path.is_file()
                && let Some(ext) = entry_path.extension().and_then(|s| s.to_str())
            {
                match ext.to_lowercase().as_str() {
                    "csproj" | "fsproj" | "sln" => {
                        let fname = entry.file_name().to_string_lossy().to_string();
                        detected_manifests.push(fname.clone());
                        evidence.push(format!(".NET project file detected ({})", fname));
                        manifest_languages
                            .entry(Language::CSharp)
                            .or_default()
                            .push(fname);
                    }
                    "gemspec" => {
                        let fname = entry.file_name().to_string_lossy().to_string();
                        detected_manifests.push(fname.clone());
                        evidence.push(format!("Ruby gemspec detected ({})", fname));
                        manifest_languages
                            .entry(Language::Ruby)
                            .or_default()
                            .push(fname);
                    }
                    "cabal" => {
                        let fname = entry.file_name().to_string_lossy().to_string();
                        detected_manifests.push(fname.clone());
                        evidence.push(format!("Haskell Cabal file detected ({})", fname));
                        manifest_languages
                            .entry(Language::Haskell)
                            .or_default()
                            .push(fname);
                    }
                    _ => {}
                }
            }
        }
    }

    // 3. Scan source files by extension
    let (stats, total_source_files) = scan_source_files(path, 6);

    // 4. Refine JavaScript vs TypeScript disambiguation
    let ts_count = stats.get(&Language::TypeScript).copied().unwrap_or(0);
    let js_count = stats.get(&Language::JavaScript).copied().unwrap_or(0);

    // If package.json exists, check if typescript is installed or tsconfig is present
    let has_package_json = detected_manifests.iter().any(|m| m == "package.json");
    let has_tsconfig = detected_manifests.iter().any(|m| m == "tsconfig.json");

    let package_json_has_ts = if has_package_json {
        let pkg_path = path.join("package.json");
        fs::read_to_string(&pkg_path)
            .map(|content| content.contains("\"typescript\""))
            .unwrap_or(false)
    } else {
        false
    };

    if has_package_json && (has_tsconfig || package_json_has_ts || ts_count > js_count) {
        // Upgrade from JavaScript to TypeScript
        manifest_languages.remove(&Language::JavaScript);
        manifest_languages
            .entry(Language::TypeScript)
            .or_default()
            .push("package.json (TypeScript project)".to_string());
        if package_json_has_ts && !has_tsconfig {
            evidence.push("Found 'typescript' dependency in package.json".to_string());
        }
    }

    // 5. Refine C vs C++ if CMakeLists.txt or Makefile was found
    if manifest_languages.contains_key(&Language::Cpp) || manifest_languages.contains_key(&Language::C) {
        let c_count = stats.get(&Language::C).copied().unwrap_or(0);
        let cpp_count = stats.get(&Language::Cpp).copied().unwrap_or(0);
        if c_count > cpp_count && c_count > 0 {
            manifest_languages.remove(&Language::Cpp);
            manifest_languages.entry(Language::C).or_default();
        } else if cpp_count > 0 {
            manifest_languages.remove(&Language::C);
            manifest_languages.entry(Language::Cpp).or_default();
        }
    }

    // Build language breakdown sorted by count descending
    let mut breakdown: Vec<LanguageStat> = stats
        .iter()
        .map(|(&lang, &count)| {
            let percentage = if total_source_files > 0 {
                (count as f64 / total_source_files as f64) * 100.0
            } else {
                0.0
            };
            LanguageStat {
                language: lang,
                file_count: count,
                percentage,
            }
        })
        .collect();
    breakdown.sort_by_key(|b| std::cmp::Reverse(b.file_count));

    // Determine primary language and confidence
    let (primary_language, confidence) = if !manifest_languages.is_empty() {
        // If manifests are present, prefer the manifest language that also has source files,
        // or the first manifest language.
        let chosen_lang = if manifest_languages.len() == 1 {
            *manifest_languages.keys().next().unwrap()
        } else {
            // Multiple manifests detected (e.g. monorepo or dual project)
            // Choose the one that has the most source files, or fallback to first
            manifest_languages
                .keys()
                .max_by_key(|lang| stats.get(lang).unwrap_or(&0))
                .copied()
                .unwrap_or(Language::Unknown)
        };

        (chosen_lang, Confidence::High)
    } else if let Some(top) = breakdown.first() {
        // No manifest, rely on source file statistics
        evidence.push(format!(
            "No project manifest found; identified by file extension ({} {} files, {:.1}%)",
            top.file_count,
            top.language,
            top.percentage
        ));

        let conf = if top.percentage >= 50.0 && top.file_count >= 3 {
            Confidence::Medium
        } else {
            Confidence::Low
        };

        (top.language, conf)
    } else {
        evidence.push("No project manifests or recognizable source code files found".to_string());
        (Language::Unknown, Confidence::Low)
    };

    Ok(DetectionResult {
        primary_language,
        confidence,
        detected_manifests,
        evidence,
        language_breakdown: breakdown,
        total_source_files,
        workspace_root,
    })
}

/// Recursively scans directory up to `max_depth`, counting source files per language.
fn scan_source_files(path: &Path, max_depth: usize) -> (HashMap<Language, usize>, usize) {
    let mut stats: HashMap<Language, usize> = HashMap::new();
    let mut total_source_files = 0;

    let walker = WalkDir::new(path)
        .max_depth(max_depth)
        .follow_links(false)
        .into_iter();

    for entry in walker.filter_entry(|e| !is_ignored(e.path())) {
        let Ok(entry) = entry else { continue };

        if entry.file_type().is_file()
            && let Some(ext) = entry.path().extension().and_then(|s| s.to_str())
            && let Some(lang) = Language::from_extension(ext)
        {
            *stats.entry(lang).or_insert(0) += 1;
            total_source_files += 1;
        }
    }

    (stats, total_source_files)
}

/// Check whether a path segment is an ignored directory (e.g. .git, target, node_modules)
fn is_ignored(path: &Path) -> bool {
    if let Some(file_name) = path.file_name().and_then(|s| s.to_str())
        && IGNORED_DIRS.contains(&file_name)
    {
        return true;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use std::path::PathBuf;

    struct TempDirGuard {
        path: PathBuf,
    }

    impl TempDirGuard {
        fn new(name: &str) -> Self {
            let unique = format!(
                "test_detector_{}_{}_{}",
                name,
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            );
            let path = std::env::temp_dir().join(unique);
            fs::create_dir_all(&path).unwrap();
            Self { path }
        }

        fn create_file(&self, rel_path: &str, content: &str) {
            let full_path = self.path.join(rel_path);
            if let Some(parent) = full_path.parent() {
                fs::create_dir_all(parent).unwrap();
            }
            let mut f = File::create(full_path).unwrap();
            f.write_all(content.as_bytes()).unwrap();
        }
    }

    impl Drop for TempDirGuard {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    #[test]
    fn test_detect_rust_manifest() {
        let temp = TempDirGuard::new("rust");
        temp.create_file("Cargo.toml", "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n");
        temp.create_file("src/main.rs", "fn main() {}\n");

        let result = detect_language(&temp.path).unwrap();
        assert_eq!(result.primary_language, Language::Rust);
        assert_eq!(result.confidence, Confidence::High);
        assert!(result.detected_manifests.contains(&"Cargo.toml".to_string()));
    }

    #[test]
    fn test_detect_python_manifest() {
        let temp = TempDirGuard::new("python");
        temp.create_file("pyproject.toml", "[project]\nname = \"demo\"\n");
        temp.create_file("app.py", "print('hello')\n");

        let result = detect_language(&temp.path).unwrap();
        assert_eq!(result.primary_language, Language::Python);
        assert_eq!(result.confidence, Confidence::High);
    }

    #[test]
    fn test_detect_typescript_with_tsconfig() {
        let temp = TempDirGuard::new("ts");
        temp.create_file("package.json", "{\"name\": \"my-app\"}");
        temp.create_file("tsconfig.json", "{}");
        temp.create_file("src/index.ts", "console.log('hi');");

        let result = detect_language(&temp.path).unwrap();
        assert_eq!(result.primary_language, Language::TypeScript);
        assert_eq!(result.confidence, Confidence::High);
    }

    #[test]
    fn test_detect_javascript_plain() {
        let temp = TempDirGuard::new("js");
        temp.create_file("package.json", "{\"name\": \"my-app\"}");
        temp.create_file("index.js", "console.log('hi');");

        let result = detect_language(&temp.path).unwrap();
        assert_eq!(result.primary_language, Language::JavaScript);
        assert_eq!(result.confidence, Confidence::High);
    }

    #[test]
    fn test_detect_go_manifest() {
        let temp = TempDirGuard::new("go");
        temp.create_file("go.mod", "module example.com/demo\ngo 1.21\n");
        temp.create_file("main.go", "package main\nfunc main() {}\n");

        let result = detect_language(&temp.path).unwrap();
        assert_eq!(result.primary_language, Language::Go);
        assert_eq!(result.confidence, Confidence::High);
    }

    #[test]
    fn test_fallback_file_extensions_without_manifest() {
        let temp = TempDirGuard::new("ext");
        temp.create_file("lib.rs", "pub fn add(a: i32, b: i32) -> i32 { a + b }");
        temp.create_file("math.rs", "pub fn sub(a: i32, b: i32) -> i32 { a - b }");
        temp.create_file("test.rs", "#[test] fn t() {}");

        let result = detect_language(&temp.path).unwrap();
        assert_eq!(result.primary_language, Language::Rust);
        assert_eq!(result.confidence, Confidence::Medium);
        assert_eq!(result.total_source_files, 3);
    }

    #[test]
    fn test_empty_dir_unknown() {
        let temp = TempDirGuard::new("empty");

        let result = detect_language(&temp.path).unwrap();
        assert_eq!(result.primary_language, Language::Unknown);
        assert_eq!(result.confidence, Confidence::Low);
        assert_eq!(result.total_source_files, 0);
    }

    #[test]
    fn test_detect_cpp_cmake() {
        let temp = TempDirGuard::new("cpp");
        temp.create_file("CMakeLists.txt", "cmake_minimum_required(VERSION 3.10)");
        temp.create_file("main.cpp", "#include <iostream>\nint main() { return 0; }");

        let result = detect_language(&temp.path).unwrap();
        assert_eq!(result.primary_language, Language::Cpp);
        assert_eq!(result.confidence, Confidence::High);
    }

    #[test]
    fn test_detect_csharp_csproj() {
        let temp = TempDirGuard::new("csharp");
        temp.create_file("App.csproj", "<Project Sdk=\"Microsoft.NET.Sdk\"></Project>");
        temp.create_file("Program.cs", "class Program {}");

        let result = detect_language(&temp.path).unwrap();
        assert_eq!(result.primary_language, Language::CSharp);
        assert_eq!(result.confidence, Confidence::High);
    }

    #[test]
    fn test_detect_ruby_gemfile() {
        let temp = TempDirGuard::new("ruby");
        temp.create_file("Gemfile", "source 'https://rubygems.org'");
        temp.create_file("lib/app.rb", "puts 'hello'");

        let result = detect_language(&temp.path).unwrap();
        assert_eq!(result.primary_language, Language::Ruby);
        assert_eq!(result.confidence, Confidence::High);
    }

    #[test]
    fn test_detect_dart_pubspec() {
        let temp = TempDirGuard::new("dart");
        temp.create_file("pubspec.yaml", "name: my_app\n");
        temp.create_file("lib/main.dart", "void main() {}");

        let result = detect_language(&temp.path).unwrap();
        assert_eq!(result.primary_language, Language::Dart);
        assert_eq!(result.confidence, Confidence::High);
    }

    #[test]
    fn test_detect_monorepo_parent_manifest() {
        let temp = TempDirGuard::new("monorepo");
        temp.create_file("Cargo.toml", "[workspace]\nmembers = [\"subcrate\"]\n");
        let subcrate = temp.path.join("subcrate");
        fs::create_dir_all(&subcrate).unwrap();
        fs::write(subcrate.join("main.rs"), "fn main() {}").unwrap();

        let result = detect_language(&subcrate).unwrap();
        assert_eq!(result.primary_language, Language::Rust);
        assert!(result.detected_manifests.iter().any(|m| m.contains("Cargo.toml")));
    }

    #[test]
    fn test_detect_nested_package_with_workspace_root_awareness() {
        let temp = TempDirGuard::new("nested_workspace");
        temp.create_file("Cargo.toml", "[workspace]\nmembers = [\"packages/cli\"]\n");
        let pkg_dir = temp.path.join("packages").join("cli");
        fs::create_dir_all(&pkg_dir).unwrap();
        fs::write(pkg_dir.join("Cargo.toml"), "[package]\nname = \"cli\"\nversion = \"0.1.0\"\n").unwrap();
        fs::write(pkg_dir.join("main.rs"), "fn main() {}").unwrap();

        let result = detect_language(&pkg_dir).unwrap();
        assert_eq!(result.primary_language, Language::Rust);
        assert!(result.workspace_root.is_some());
        let ws_root = result.workspace_root.unwrap();
        assert_eq!(
            std::fs::canonicalize(&ws_root).unwrap(),
            std::fs::canonicalize(&temp.path).unwrap()
        );
        assert!(result.evidence.iter().any(|e| e.contains("Detected enclosing workspace root")));
    }
}

