pub mod global;
pub mod project;

pub use global::GlobalConfig;
pub use project::{
    BuildConfig, BuildMode, InstallScriptsConfig, PackageManagersConfig, PlatformsConfig,
    ProjectConfig,
};
