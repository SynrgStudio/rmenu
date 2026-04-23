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

pub fn load_directory_descriptor(module_dir: &Path) -> Result<ModuleDescriptor, ManifestParseError> {
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
            current_section = trimmed.trim_start_matches('[').trim_end_matches(']').to_string();
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

    let name = string_value(&root_values, "name").ok_or(ManifestParseError::MissingRequiredField("name"))?;
    let version = string_value(&root_values, "version").ok_or(ManifestParseError::MissingRequiredField("version"))?;
    let api_version_str = string_value(&root_values, "api_version")
        .ok_or(ManifestParseError::MissingRequiredField("api_version"))?;
    let api_version = api_version_str
        .parse::<u32>()
        .map_err(|_| ManifestParseError::InvalidApiVersion(api_version_str.clone()))?;

    let kind = string_value(&root_values, "kind").ok_or(ManifestParseError::MissingRequiredField("kind"))?;
    let entry = string_value(&root_values, "entry").ok_or(ManifestParseError::MissingEntry)?;
    let capabilities = parse_array(&root_values, "capabilities");

    let entry_path = module_dir.join(entry);
    let entry_code = fs::read_to_string(&entry_path).map_err(|err| ManifestParseError::Io(err.to_string()))?;

    let config_json = string_value(&config_values, "file")
        .map(|file_name| module_dir.join(file_name))
        .filter(|path| path.exists())
        .map(|path| fs::read_to_string(path).map_err(|err| ManifestParseError::Io(err.to_string())))
        .transpose()?;

    let readme_path = module_dir.join("README.md");
    let readme = if readme_path.exists() {
        Some(fs::read_to_string(readme_path).map_err(|err| ManifestParseError::Io(err.to_string()))?)
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
    values.get(key).and_then(|raw| raw.trim().parse::<i32>().ok())
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
