#[allow(clippy::too_many_arguments)]
pub fn generate_macports_portfile(
    app_name: &str,
    version: &str,
    description: &str,
    license: &str,
    maintainer: &str,
    homepage: &str,
    repo: &str,
    mac_x64_hash: Option<&str>,
    mac_arm64_hash: Option<&str>,
) -> String {
    let clean_repo = repo
        .trim_start_matches("https://github.com/")
        .trim_start_matches("http://github.com/")
        .trim_end_matches(".git");
    let clean_v = version.trim_start_matches('v');

    let x64_h = mac_x64_hash.unwrap_or("0000000000000000000000000000000000000000000000000000000000000000");
    let arm_h = mac_arm64_hash.unwrap_or("0000000000000000000000000000000000000000000000000000000000000000");

    let clean_desc = description.replace('{', "\\{").replace('}', "\\}");

    format!(
        r#"# -*- coding: utf-8; mode: tcl; tab-width: 4; indent-tabs-mode: nil; c-basic-offset: 4 -*-
PortSystem          1.0

name                {app_name}
version             {clean_v}
categories          devel
platforms           darwin
license             {license}
maintainers         {{{maintainer}}}
description         {{{clean_desc}}}
long_description    {{{clean_desc}}}

homepage            {homepage}
master_sites        https://github.com/{clean_repo}/releases/download/v${{version}}/

if {{${{build_arch}} eq "arm64"}} {{
    distfiles       ${{name}}_${{version}}_darwin_arm64.tar.gz
    checksums       sha256  {arm_h}
}} else {{
    distfiles       ${{name}}_${{version}}_darwin_amd64.tar.gz
    checksums       sha256  {x64_h}
}}

use_configure       no

build {{}}

destroot {{
    xinstall -m 755 ${{worksrcpath}}/${{name}} ${{destroot}}${{prefix}}/bin/
}}
"#
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_macports_portfile_generation() {
        let portfile = generate_macports_portfile(
            "cli-helper",
            "1.8.0",
            "Command line helper",
            "BSD-3-Clause",
            "alice",
            "https://cli-helper.io",
            "https://github.com/alice/cli-helper",
            Some("sha_portfile_x64"),
            Some("sha_portfile_arm64"),
        );

        assert!(portfile.contains("name                cli-helper"));
        assert!(portfile.contains("version             1.8.0"));
        assert!(portfile.contains("sha256  sha_portfile_arm64"));
        assert!(portfile.contains("xinstall -m 755 ${worksrcpath}/${name} ${destroot}${prefix}/bin/"));
    }
}
