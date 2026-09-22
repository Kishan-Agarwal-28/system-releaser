pub fn generate_snapcraft_yaml(
    app_name: &str,
    version: &str,
    description: &str,
    grade: Option<&str>,
    confinement: Option<&str>,
) -> String {
    let clean_v = version.trim_start_matches('v');
    let g = grade.unwrap_or("stable");
    let c = confinement.unwrap_or("classic");

    format!(r#"name: {app_name}
base: core22
version: '{clean_v}'
summary: {description}
description: |
  {description}

grade: {g}
confinement: {c}

apps:
  {app_name}:
    command: bin/{app_name}

parts:
  {app_name}:
    plugin: dump
    source: dist/
    organize:
      {app_name}: bin/{app_name}
"#)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snapcraft_yaml_generation() {
        let yaml = generate_snapcraft_yaml(
            "my-tool",
            "1.0.0",
            "Snap CLI",
            Some("stable"),
            Some("classic"),
        );

        assert!(yaml.contains("name: my-tool"));
        assert!(yaml.contains("version: '1.0.0'"));
        assert!(yaml.contains("confinement: classic"));
    }
}
