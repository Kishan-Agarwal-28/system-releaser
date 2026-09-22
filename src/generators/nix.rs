#[allow(clippy::too_many_arguments)]
pub fn generate_nix_flake(
    app_name: &str,
    version: &str,
    description: &str,
    repo: &str,
    x64_linux_hash: Option<&str>,
    arm64_linux_hash: Option<&str>,
    x64_darwin_hash: Option<&str>,
    arm64_darwin_hash: Option<&str>,
) -> String {
    let clean_repo = repo
        .trim_start_matches("https://github.com/")
        .trim_start_matches("http://github.com/")
        .trim_end_matches(".git");
    let clean_v = version.trim_start_matches('v');

    let x64_lin = x64_linux_hash.unwrap_or("0000000000000000000000000000000000000000000000000000000000000000");
    let arm_lin = arm64_linux_hash.unwrap_or("0000000000000000000000000000000000000000000000000000000000000000");
    let x64_mac = x64_darwin_hash.unwrap_or("0000000000000000000000000000000000000000000000000000000000000000");
    let arm_mac = arm64_darwin_hash.unwrap_or("0000000000000000000000000000000000000000000000000000000000000000");
    let clean_desc = description
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace("${", "\\${");

    format!(
        r#"{{
  description = "{clean_desc}";

  inputs = {{
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  }};

  outputs = {{ self, nixpkgs, flake-utils }}:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${{system}};
        sources = {{
          "x86_64-linux" = {{
            url = "https://github.com/{clean_repo}/releases/download/v{clean_v}/{app_name}_{clean_v}_linux_amd64.tar.gz";
            sha256 = "{x64_lin}";
          }};
          "aarch64-linux" = {{
            url = "https://github.com/{clean_repo}/releases/download/v{clean_v}/{app_name}_{clean_v}_linux_arm64.tar.gz";
            sha256 = "{arm_lin}";
          }};
          "x86_64-darwin" = {{
            url = "https://github.com/{clean_repo}/releases/download/v{clean_v}/{app_name}_{clean_v}_darwin_amd64.tar.gz";
            sha256 = "{x64_mac}";
          }};
          "aarch64-darwin" = {{
            url = "https://github.com/{clean_repo}/releases/download/v{clean_v}/{app_name}_{clean_v}_darwin_arm64.tar.gz";
            sha256 = "{arm_mac}";
          }};
        }};
        srcInfo = sources.${{system}} or (throw "Unsupported system: ${{system}}");
      in
      {{
        packages.default = pkgs.stdenv.mkDerivation {{
          pname = "{app_name}";
          version = "{clean_v}";
          src = pkgs.fetchurl {{
            url = srcInfo.url;
            sha256 = srcInfo.sha256;
          }};
          dontBuild = true;
          installPhase = ''
            mkdir -p $out/bin
            cp {app_name} $out/bin/
            chmod +x $out/bin/{app_name}
          '';
        }};
      }}
    );
}}
"#
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nix_flake_generation() {
        let flake = generate_nix_flake(
            "nix-tool",
            "1.0.0",
            "Nix Flake Tool",
            "https://github.com/nix/nix-tool",
            Some("sha_linux_amd64"),
            Some("sha_linux_arm64"),
            Some("sha_darwin_amd64"),
            Some("sha_darwin_arm64"),
        );

        assert!(flake.contains("pname = \"nix-tool\";"));
        assert!(flake.contains("version = \"1.0.0\";"));
        assert!(flake.contains("sha_linux_amd64"));
        assert!(flake.contains("cp nix-tool $out/bin/"));
    }
}
