use std::path::Path;
use std::process::Command;

use crate::language::Language;

#[derive(Debug)]
pub struct TestRunResult {
    pub success: bool,
    pub command: String,
    pub output: String,
}

/// Automatically runs the project's native test suite according to the detected language.
pub fn run_project_tests(project_dir: &Path, language: Language) -> Result<TestRunResult, std::io::Error> {
    let (program, args) = match language {
        Language::Rust => ("cargo", vec!["test"]),
        Language::Go => ("go", vec!["test", "./..."]),
        Language::TypeScript | Language::JavaScript => {
            if project_dir.join("pnpm-lock.yaml").exists() {
                ("pnpm", vec!["test"])
            } else if project_dir.join("yarn.lock").exists() {
                ("yarn", vec!["test"])
            } else if project_dir.join("bun.lockb").exists() {
                ("bun", vec!["test"])
            } else {
                ("npm", vec!["test"])
            }
        }
        Language::Python => {
            if project_dir.join("poetry.lock").exists() {
                ("poetry", vec!["run", "pytest"])
            } else {
                ("pytest", vec![])
            }
        }
        Language::CSharp => ("dotnet", vec!["test"]),
        Language::Java | Language::Kotlin => {
            if project_dir.join("gradlew").exists() || project_dir.join("gradlew.bat").exists() {
                ("gradle", vec!["test"])
            } else {
                ("mvn", vec!["test"])
            }
        }
        _ => {
            return Ok(TestRunResult {
                success: true,
                command: "none".to_string(),
                output: format!("No standard test runner defined for language {}", language),
            });
        }
    };

    let full_cmd = format!("{} {}", program, args.join(" "));

    let output = Command::new(program)
        .args(&args)
        .current_dir(project_dir)
        .output();

    match output {
        Ok(out) => {
            let combined = format!(
                "{}\n{}",
                String::from_utf8_lossy(&out.stdout),
                String::from_utf8_lossy(&out.stderr)
            );
            Ok(TestRunResult {
                success: out.status.success(),
                command: full_cmd,
                output: combined.trim().to_string(),
            })
        }
        Err(err) => Err(std::io::Error::new(
            err.kind(),
            format!("Failed to execute '{}': {}", full_cmd, err),
        )),
    }
}
