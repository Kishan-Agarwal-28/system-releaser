pub mod archive;
pub mod builder;
pub mod check;
pub mod ci;
pub mod config;
pub mod detector;
pub mod generators;
pub mod init;
pub mod installers;
pub mod language;
pub mod metadata;
pub mod package_manager;
pub mod platform;
pub mod release;
pub mod test_runner;
pub mod upload;
pub mod version;

pub use archive::{compute_sha256, create_tar_gz, create_zip, generate_checksums_file};
pub use builder::{build_target, BuildResult};
pub use check::{run_preflight_checks, CheckItem, CheckReport, CheckStatus};
pub use ci::generate_github_action_workflow;
pub use config::{
    BuildConfig, BuildMode, GlobalConfig, InstallScriptsConfig, PackageManagersConfig,
    PlatformsConfig, ProjectConfig,
};
pub use detector::{detect_language, Confidence, DetectionResult, LanguageStat};
pub use generators::{generate_all_manifests, ManifestGenerationContext};
pub use init::{init_project, InitOptions, InitResult};
pub use installers::{generate_install_ps1, generate_install_sh};
pub use language::Language;
pub use metadata::ProjectMetadata;
pub use package_manager::{PackageManager, TargetOs};
pub use platform::{Arch, TargetPlatform, OS};
pub use release::{execute_release, ReleaseOptions, ReleaseSummary};
pub use test_runner::{run_project_tests, TestRunResult};
pub use upload::{upload_release_assets, UploadResult};
pub use version::{
    git_commit_and_tag, resolve_current_version, sync_version_to_manifests, VersionBump,
};
