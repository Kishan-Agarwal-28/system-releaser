use std::fmt;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Language {
    Rust,
    TypeScript,
    JavaScript,
    Python,
    Go,
    Java,
    Kotlin,
    C,
    Cpp,
    CSharp,
    Ruby,
    Php,
    Swift,
    Dart,
    Zig,
    Scala,
    Elixir,
    Haskell,
    Shell,
    Lua,
    R,
    Clojure,
    Unknown,
}

impl Language {
    /// Returns the human-readable display name of the programming language.
    pub fn name(&self) -> &'static str {
        match self {
            Language::Rust => "Rust",
            Language::TypeScript => "TypeScript",
            Language::JavaScript => "JavaScript",
            Language::Python => "Python",
            Language::Go => "Go",
            Language::Java => "Java",
            Language::Kotlin => "Kotlin",
            Language::C => "C",
            Language::Cpp => "C++",
            Language::CSharp => "C#",
            Language::Ruby => "Ruby",
            Language::Php => "PHP",
            Language::Swift => "Swift",
            Language::Dart => "Dart",
            Language::Zig => "Zig",
            Language::Scala => "Scala",
            Language::Elixir => "Elixir",
            Language::Haskell => "Haskell",
            Language::Shell => "Shell",
            Language::Lua => "Lua",
            Language::R => "R",
            Language::Clojure => "Clojure",
            Language::Unknown => "Unknown",
        }
    }

    /// Primary build tool or package manager commonly associated with the language.
    pub fn default_tool(&self) -> Option<&'static str> {
        match self {
            Language::Rust => Some("cargo"),
            Language::TypeScript => Some("npm / pnpm / yarn / bun / tsc"),
            Language::JavaScript => Some("node / npm / pnpm / yarn / bun"),
            Language::Python => Some("pyinstaller / python / pip / poetry / uv"),
            Language::Go => Some("go"),
            Language::Java => Some("mvn / gradle"),
            Language::Kotlin => Some("gradle"),
            Language::C => Some("cmake / make / ninja"),
            Language::Cpp => Some("cmake / make / ninja"),
            Language::CSharp => Some("dotnet"),
            Language::Ruby => Some("bundle / gem"),
            Language::Php => Some("composer"),
            Language::Swift => Some("swift"),
            Language::Dart => Some("dart / flutter"),
            Language::Zig => Some("zig"),
            Language::Scala => Some("sbt"),
            Language::Elixir => Some("mix"),
            Language::Haskell => Some("cabal / stack"),
            Language::Shell => Some("sh / bash"),
            Language::Lua => Some("luarocks"),
            Language::R => Some("R / renv"),
            Language::Clojure => Some("lein / clj"),
            Language::Unknown => None,
        }
    }

    /// Determine language from a file extension.
    pub fn from_extension(ext: &str) -> Option<Language> {
        let ext_lower = ext.to_lowercase();
        match ext_lower.as_str() {
            "rs" => Some(Language::Rust),
            "ts" | "tsx" | "mts" | "cts" => Some(Language::TypeScript),
            "js" | "jsx" | "mjs" | "cjs" => Some(Language::JavaScript),
            "py" | "pyi" | "pyw" => Some(Language::Python),
            "go" => Some(Language::Go),
            "java" => Some(Language::Java),
            "kt" | "kts" => Some(Language::Kotlin),
            "c" | "h" => Some(Language::C),
            "cpp" | "cxx" | "cc" | "hpp" | "hxx" | "hh" => Some(Language::Cpp),
            "cs" => Some(Language::CSharp),
            "rb" | "rake" | "gemspec" => Some(Language::Ruby),
            "php" | "phtml" => Some(Language::Php),
            "swift" => Some(Language::Swift),
            "dart" => Some(Language::Dart),
            "zig" => Some(Language::Zig),
            "scala" | "sc" => Some(Language::Scala),
            "ex" | "exs" => Some(Language::Elixir),
            "hs" | "lhs" => Some(Language::Haskell),
            "sh" | "bash" | "zsh" => Some(Language::Shell),
            "lua" => Some(Language::Lua),
            "r" | "rmd" => Some(Language::R),
            "clj" | "cljs" | "cljc" | "edn" => Some(Language::Clojure),
            _ => None,
        }
    }
}

impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}
