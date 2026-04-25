use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use super::types::{ModuleDescriptor, ModuleSourceType};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ManifestParseError {
    Io(String),
    MissingRequiredField(&'static str),
    InvalidApiVersion(String),
    MissingEntry,
}

pub fn load_directory_descriptor(
    module_dir: &Path,
) -> Result<ModuleDescriptor, ManifestParseError> {
    let manifest_path = module_dir.join("module.toml");
    let manifest_content = fs::read_to_string(&manifest_path)
        .map_err(|err| ManifestParseError::Io(err.to_string()))?;

    let mut root_values: BTreeMap<String, String> = BTreeMap::new();
    let mut config_values: BTreeMap<String, String> = BTreeMap::new();
    let mut current_section = String::new();

    for line in manifest_content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            current_section = trimmed
                .trim_start_matches('[')
                .trim_end_matches(']')
                .to_string();
            continue;
        }

        let Some((raw_key, raw_value)) = trimmed.split_once('=') else {
            continue;
        };

        let key = raw_key.trim().to_string();
        let value = raw_value.trim().to_string();

        if current_section == "config" {
            config_values.insert(key, value);
        } else {
            root_values.insert(key, value);
        }
    }

    let name = string_value(&root_values, "name")
        .ok_or(ManifestParseError::MissingRequiredField("name"))?;
    let version = string_value(&root_values, "version")
        .ok_or(ManifestParseError::MissingRequiredField("version"))?;
    let api_version_str = string_value(&root_values, "api_version")
        .ok_or(ManifestParseError::MissingRequiredField("api_version"))?;
    let api_version = api_version_str
        .parse::<u32>()
        .map_err(|_| ManifestParseError::InvalidApiVersion(api_version_str.clone()))?;

    let kind = string_value(&root_values, "kind")
        .ok_or(ManifestParseError::MissingRequiredField("kind"))?;
    let entry = string_value(&root_values, "entry").ok_or(ManifestParseError::MissingEntry)?;
    let capabilities = parse_array(&root_values, "capabilities");

    let entry_path = module_dir.join(entry);
    let entry_code =
        fs::read_to_string(&entry_path).map_err(|err| ManifestParseError::Io(err.to_string()))?;

    let config_json = string_value(&config_values, "file")
        .map(|file_name| module_dir.join(file_name))
        .filter(|path| path.exists())
        .map(|path| fs::read_to_string(path).map_err(|err| ManifestParseError::Io(err.to_string())))
        .transpose()?;

    let readme_path = module_dir.join("README.md");
    let readme = if readme_path.exists() {
        Some(
            fs::read_to_string(readme_path)
                .map_err(|err| ManifestParseError::Io(err.to_string()))?,
        )
    } else {
        None
    };

    Ok(ModuleDescriptor {
        source_type: ModuleSourceType::Directory,
        source_path: module_dir.to_string_lossy().to_string(),
        name,
        version,
        api_version,
        kind,
        capabilities,
        enabled: bool_value(&root_values, "enabled").unwrap_or(true),
        priority: int_value(&root_values, "priority").unwrap_or(0),
        description: string_value(&root_values, "description"),
        author: string_value(&root_values, "author"),
        homepage: string_value(&root_values, "homepage"),
        entry_code,
        config_json,
        readme,
    })
}

fn string_value(values: &BTreeMap<String, String>, key: &str) -> Option<String> {
    values
        .get(key)
        .map(|raw| raw.trim().trim_matches('"').to_string())
        .filter(|value| !value.is_empty())
}

fn bool_value(values: &BTreeMap<String, String>, key: &str) -> Option<bool> {
    values.get(key).and_then(|raw| match raw.trim() {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    })
}

fn int_value(values: &BTreeMap<String, String>, key: &str) -> Option<i32> {
    values
        .get(key)
        .and_then(|raw| raw.trim().parse::<i32>().ok())
}

fn parse_array(values: &BTreeMap<String, String>, key: &str) -> Vec<String> {
    let Some(raw) = values.get(key) else {
        return Vec::new();
    };

    let trimmed = raw.trim();
    if !trimmed.starts_with('[') || !trimmed.ends_with(']') {
        return Vec::new();
    }

    trimmed
        .trim_start_matches('[')
        .trim_end_matches(']')
        .split(',')
        .map(str::trim)
        .map(|value| value.trim_matches('"'))
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .collect()
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::{load_directory_descriptor, ManifestParseError};
    use crate::modules::types::ModuleSourceType;

    fn temp_module_dir(name: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time must be valid")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("rmenu-manifest-test-{name}-{nonce}"));
        fs::create_dir_all(&dir).expect("create temp module dir");
        dir
    }

    #[test]
    fn load_directory_descriptor_accepts_valid_module_toml() {
        let dir = temp_module_dir("valid");
        fs::write(
            dir.join("module.toml"),
            r#"
name = "dir-module"
version = "1.2.3"
api_version = "1"
kind = "external-js"
entry = "module.js"
capabilities = ["providers", "commands"]
enabled = false
priority = 42
description = "Directory module"
author = "Tester"
homepage = "https://example.test"

[config]
file = "config.json"
"#,
        )
        .expect("write manifest");
        fs::write(dir.join("module.js"), "export default () => ({})").expect("write entry");
        fs::write(dir.join("config.json"), "{\"ok\":true}").expect("write config");
        fs::write(dir.join("README.md"), "# Module").expect("write readme");

        let descriptor = load_directory_descriptor(&dir).expect("manifest should parse");

        assert_eq!(descriptor.source_type, ModuleSourceType::Directory);
        assert_eq!(descriptor.name, "dir-module");
        assert_eq!(descriptor.version, "1.2.3");
        assert_eq!(descriptor.api_version, 1);
        assert_eq!(descriptor.kind, "external-js");
        assert_eq!(descriptor.capabilities, vec!["providers", "commands"]);
        assert!(!descriptor.enabled);
        assert_eq!(descriptor.priority, 42);
        assert_eq!(descriptor.description.as_deref(), Some("Directory module"));
        assert_eq!(descriptor.author.as_deref(), Some("Tester"));
        assert_eq!(descriptor.homepage.as_deref(), Some("https://example.test"));
        assert_eq!(descriptor.entry_code, "export default () => ({})");
        assert_eq!(descriptor.config_json.as_deref(), Some("{\"ok\":true}"));
        assert_eq!(descriptor.readme.as_deref(), Some("# Module"));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn load_directory_descriptor_rejects_invalid_module_toml() {
        let missing_name = temp_module_dir("missing-name");
        fs::write(
            missing_name.join("module.toml"),
            r#"
version = "1.0.0"
api_version = "1"
kind = "external-js"
entry = "module.js"
"#,
        )
        .expect("write manifest");
        fs::write(missing_name.join("module.js"), "export default () => ({})")
            .expect("write entry");

        let err = load_directory_descriptor(&missing_name).expect_err("missing name must fail");
        assert_eq!(err, ManifestParseError::MissingRequiredField("name"));
        let _ = fs::remove_dir_all(missing_name);

        let invalid_api = temp_module_dir("invalid-api");
        fs::write(
            invalid_api.join("module.toml"),
            r#"
name = "bad-api"
version = "1.0.0"
api_version = "v1"
kind = "external-js"
entry = "module.js"
"#,
        )
        .expect("write manifest");
        fs::write(invalid_api.join("module.js"), "export default () => ({})").expect("write entry");

        let err = load_directory_descriptor(&invalid_api).expect_err("invalid api must fail");
        assert_eq!(err, ManifestParseError::InvalidApiVersion("v1".to_string()));
        let _ = fs::remove_dir_all(invalid_api);
    }

    #[test]
    fn load_directory_descriptor_rejects_missing_entry_file() {
        let dir = temp_module_dir("missing-entry-file");
        fs::write(
            dir.join("module.toml"),
            r#"
name = "missing-entry"
version = "1.0.0"
api_version = "1"
kind = "external-js"
entry = "module.js"
"#,
        )
        .expect("write manifest");

        let err = load_directory_descriptor(&dir).expect_err("missing entry file must fail");
        assert!(matches!(err, ManifestParseError::Io(_)));
        let _ = fs::remove_dir_all(dir);
    }
}
