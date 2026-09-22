fn sanitize_rpm_text(s: &str) -> String {
    s.replace('%', "%%")
        .replace('`', "")
        .replace('$', "\\$")
}

#[allow(clippy::too_many_arguments)]
pub fn generate_rpm_spec(
    app_name: &str,
    version: &str,
    description: &str,
    license: &str,
    homepage: &str,
    repo: &str,
) -> String {
    let clean_repo = repo
        .trim_start_matches("https://github.com/")
        .trim_start_matches("http://github.com/")
        .trim_end_matches(".git");
    let clean_v = version.trim_start_matches('v');

    let clean_summary = sanitize_rpm_text(description.lines().next().unwrap_or("").trim());
    let clean_desc = sanitize_rpm_text(description);
    let date_str = chrono::Utc::now().format("%a %b %d %Y").to_string();

    format!(
        r#"Name:           {app_name}
Version:        {clean_v}
Release:        1%{{?dist}}
Summary:        {clean_summary}
License:        {license}
URL:            {homepage}

%ifarch x86_64
Source0:        https://github.com/{clean_repo}/releases/download/v%{{version}}/%{{name}}_%{{version}}_linux_amd64.tar.gz
%endif
%ifarch aarch64
Source0:        https://github.com/{clean_repo}/releases/download/v%{{version}}/%{{name}}_%{{version}}_linux_arm64.tar.gz
%endif

ExclusiveArch:  x86_64 aarch64

%description
{clean_desc}

%prep
%setup -q -c

%build
# Binary release package

%install
rm -rf %{{buildroot}}
mkdir -p %{{buildroot}}%{{_bindir}}
install -m 755 %{{name}} %{{buildroot}}%{{_bindir}}/%{{name}}

%files
%{{_bindir}}/%{{name}}

%changelog
* {date_str} system-releaser <noreply@github.com> - {clean_v}-1
- Automatic release of v{clean_v}
"#
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rpm_spec_generation() {
        let spec = generate_rpm_spec(
            "my-service",
            "2.1.0",
            "High performance service daemon",
            "Apache-2.0",
            "https://service.org",
            "https://github.com/org/my-service",
        );

        assert!(spec.contains("Name:           my-service"));
        assert!(spec.contains("Version:        2.1.0"));
        assert!(spec.contains("License:        Apache-2.0"));
        assert!(spec.contains("%build"));
        assert!(spec.contains("system-releaser <noreply@github.com> - 2.1.0-1"));
        assert!(spec.contains("install -m 755 %{name} %{buildroot}%{_bindir}/%{name}"));
    }

    #[test]
    fn test_rpm_spec_escaping() {
        let spec = generate_rpm_spec(
            "bad-app",
            "1.0.0",
            "A tool with \"quotes\" & <special> chars $VAR `backtick` %macro",
            "MIT",
            "https://bad.app",
            "https://github.com/org/bad-app",
        );

        // $VAR must be escaped as \$VAR
        assert!(spec.contains(r"\$VAR"));
        assert!(!spec.contains(" $VAR"));
        // Backticks must be stripped
        assert!(!spec.contains('`'));
        assert!(spec.contains("backtick"));
        // %macro must be escaped as %%macro
        assert!(spec.contains("%%macro"));
    }
}

