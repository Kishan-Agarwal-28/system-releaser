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

    let clean_desc = description.replace('%', "%%");

    format!(
        r#"Name:           {app_name}
Version:        {clean_v}
Release:        1%{{?dist}}
Summary:        {clean_desc}
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

%install
rm -rf %{{buildroot}}
mkdir -p %{{buildroot}}%{{_bindir}}
install -m 755 %{{name}} %{{buildroot}}%{{_bindir}}/%{{name}}

%files
%{{_bindir}}/%{{name}}

%changelog
* Release v{clean_v} system-releaser <noreply@github.com> - {clean_v}-1
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
        assert!(spec.contains("install -m 755 %{name} %{buildroot}%{_bindir}/%{name}"));
    }
}
