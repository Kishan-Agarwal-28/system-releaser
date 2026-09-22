#[allow(clippy::too_many_arguments)]
pub fn generate_homebrew_formula(
    app_name: &str,
    version: &str,
    description: &str,
    homepage: &str,
    repo: &str,
    mac_arm64_hash: Option<&str>,
    mac_x64_hash: Option<&str>,
    linux_x64_hash: Option<&str>,
    linux_arm64_hash: Option<&str>,
) -> String {
    let class_name = to_camel_case(app_name);
    let clean_repo = repo
        .trim_start_matches("https://github.com/")
        .trim_start_matches("http://github.com/")
        .trim_end_matches(".git");
    let clean_v = version.trim_start_matches('v');

    let mac_arm_h = mac_arm64_hash.unwrap_or("TODO_SHA256_MAC_ARM64");
    let mac_x64_h = mac_x64_hash.unwrap_or("TODO_SHA256_MAC_AMD64");
    let lin_x64_h = linux_x64_hash.unwrap_or("TODO_SHA256_LINUX_AMD64");
    let lin_arm_h = linux_arm64_hash.unwrap_or("TODO_SHA256_LINUX_ARM64");

    let bin_path = "#{bin}";

    let clean_desc = escape_ruby_str(description);
    let clean_homepage = escape_ruby_str(homepage);

    format!(r#"class {class_name} < Formula
  desc "{clean_desc}"
  homepage "{clean_homepage}"
  version "{clean_v}"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/{clean_repo}/releases/download/v{clean_v}/{app_name}_{clean_v}_darwin_arm64.tar.gz"
      sha256 "{mac_arm_h}"
    else
      url "https://github.com/{clean_repo}/releases/download/v{clean_v}/{app_name}_{clean_v}_darwin_amd64.tar.gz"
      sha256 "{mac_x64_h}"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/{clean_repo}/releases/download/v{clean_v}/{app_name}_{clean_v}_linux_arm64.tar.gz"
      sha256 "{lin_arm_h}"
    else
      url "https://github.com/{clean_repo}/releases/download/v{clean_v}/{app_name}_{clean_v}_linux_amd64.tar.gz"
      sha256 "{lin_x64_h}"
    end
  end

  def install
    bin.install "{app_name}"
  end

  test do
    system "{bin_path}/{app_name}", "--version"
  end
end
"#)
}

fn to_camel_case(s: &str) -> String {
    s.split(['-', '_'])
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect()
}

fn escape_ruby_str(s: &str) -> String {
    s.lines()
        .next()
        .unwrap_or("")
        .trim()
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('#', "\\#")
        .replace('$', "\\$")
        .replace('`', "\\`")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_homebrew_formula_generation() {
        let formula = generate_homebrew_formula(
            "my-tool",
            "1.2.0",
            "Super Fast CLI",
            "https://mytool.dev",
            "https://github.com/org/my-tool",
            Some("sha_arm"),
            Some("sha_intel"),
            Some("sha_lin"),
            Some("sha_lin_arm"),
        );

        assert!(formula.contains("class MyTool < Formula"));
        assert!(formula.contains("desc \"Super Fast CLI\""));
        assert!(formula.contains("sha256 \"sha_arm\""));
        assert!(formula.contains("bin.install \"my-tool\""));
    }

    #[test]
    fn test_homebrew_formula_escaping() {
        let formula = generate_homebrew_formula(
            "bad-tool",
            "1.0.0",
            "A tool with \"quotes\" & <special> chars $VAR `backtick` #{1+1}",
            "https://bad.tool",
            "https://github.com/org/bad-tool",
            None,
            None,
            None,
            None,
        );

        // $VAR escaped
        assert!(formula.contains(r"\$VAR"));
        // #{1+1} escaped
        assert!(formula.contains(r"\#{1+1}"));
        // backtick escaped
        assert!(formula.contains(r"\`backtick\`"));
        // quotes escaped
        assert!(formula.contains(r#"\"quotes\""#));
    }
}

