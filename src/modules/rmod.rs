use std::collections::BTreeMap;

use super::types::{ModuleDescriptor, ModuleSourceType};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RmodParseError {
    InvalidMagic,
    HeaderMalformed(String),
    MissingRequiredHeader(&'static str),
    InvalidApiVersion(String),
    DuplicateBlock(String),
    MissingModuleJs,
    ConfigNotJson(String),
}

impl RmodParseError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::InvalidMagic => "RMOD_E_INVALID_MAGIC",
            Self::HeaderMalformed(_) => "RMOD_E_HEADER_MALFORMED",
            Self::MissingRequiredHeader(_) => "RMOD_E_MISSING_REQUIRED_HEADER",
            Self::InvalidApiVersion(_) => "RMOD_E_INVALID_API_VERSION",
            Self::DuplicateBlock(_) => "RMOD_E_DUPLICATE_BLOCK",
            Self::MissingModuleJs => "RMOD_E_MISSING_MODULE_JS",
            Self::ConfigNotJson(_) => "RMOD_E_CONFIG_NOT_JSON",
        }
    }

    pub fn message(&self) -> String {
        match self {
            Self::InvalidMagic => "invalid rmod magic, expected '#!rmod/v1'".to_string(),
            Self::HeaderMalformed(line) => format!("malformed header line: '{line}'"),
            Self::MissingRequiredHeader(field) => format!("missing required header: '{field}'"),
            Self::InvalidApiVersion(value) => format!("invalid api_version value: '{value}'"),
            Self::DuplicateBlock(name) => format!("duplicate block: '{name}'"),
            Self::MissingModuleJs => "missing required block: 'module.js'".to_string(),
            Self::ConfigNotJson(error) => format!("invalid config.json: {error}"),
        }
    }
}

pub fn parse_rmod(content: &str, source_path: String) -> Result<ModuleDescriptor, RmodParseError> {
    let normalized = content.strip_prefix('\u{feff}').unwrap_or(content);
    let mut lines = normalized.lines();

    let Some(first_line) = lines.next() else {
        return Err(RmodParseError::InvalidMagic);
    };

    if first_line.trim() != "#!rmod/v1" {
        return Err(RmodParseError::InvalidMagic);
    }

    let mut header = BTreeMap::new();
    let mut reached_blank = false;
    let mut remainder_lines: Vec<String> = Vec::new();

    for line in lines {
        if !reached_blank {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                reached_blank = true;
                continue;
            }

            let Some((key, value)) = trimmed.split_once(':') else {
                return Err(RmodParseError::HeaderMalformed(trimmed.to_string()));
            };

            header.insert(key.trim().to_string(), value.trim().to_string());
        } else {
            remainder_lines.push(line.to_string());
        }
    }

    let name = header
        .get("name")
        .cloned()
        .ok_or(RmodParseError::MissingRequiredHeader("name"))?;
    let version = header
        .get("version")
        .cloned()
        .ok_or(RmodParseError::MissingRequiredHeader("version"))?;
    let api_version_str = header
        .get("api_version")
        .cloned()
        .ok_or(RmodParseError::MissingRequiredHeader("api_version"))?;
    let kind = header
        .get("kind")
        .cloned()
        .ok_or(RmodParseError::MissingRequiredHeader("kind"))?;
    let capabilities_raw = header
        .get("capabilities")
        .cloned()
        .ok_or(RmodParseError::MissingRequiredHeader("capabilities"))?;

    let api_version = api_version_str
        .parse::<u32>()
        .map_err(|_| RmodParseError::InvalidApiVersion(api_version_str.clone()))?;

    let capabilities = capabilities_raw
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .collect::<Vec<_>>();

    let enabled = header
        .get("enabled")
        .map(|value| value.eq_ignore_ascii_case("true"))
        .unwrap_or(true);
    let priority = header
        .get("priority")
        .and_then(|value| value.parse::<i32>().ok())
        .unwrap_or(0);

    let mut blocks: BTreeMap<String, String> = BTreeMap::new();
    let mut current_block: Option<String> = None;
    let mut current_content = String::new();

    for line in remainder_lines {
        let trimmed = line.trim();
        let is_delimiter =
            trimmed.starts_with("---") && trimmed.ends_with("---") && trimmed.len() > 6;

        if is_delimiter {
            if let Some(block_name) = current_block.take() {
                blocks.insert(
                    block_name,
                    current_content.trim_end_matches('\n').to_string(),
                );
                current_content.clear();
            }

            let block_name = trimmed
                .trim_start_matches("---")
                .trim_end_matches("---")
                .trim()
                .to_string();
            if blocks.contains_key(&block_name) {
                return Err(RmodParseError::DuplicateBlock(block_name));
            }
            current_block = Some(block_name);
            continue;
        }

        if current_block.is_some() {
            current_content.push_str(&line);
            current_content.push('\n');
        }
    }

    if let Some(block_name) = current_block {
        blocks.insert(
            block_name,
            current_content.trim_end_matches('\n').to_string(),
        );
    }

    let entry_code = blocks
        .remove("module.js")
        .ok_or(RmodParseError::MissingModuleJs)?;

    let config_json = blocks.remove("config.json");
    if let Some(config) = &config_json {
        serde_json::from_str::<serde_json::Value>(config)
            .map_err(|err| RmodParseError::ConfigNotJson(err.to_string()))?;
    }

    let readme = blocks.remove("readme.md");

    Ok(ModuleDescriptor {
        source_type: ModuleSourceType::Rmod,
        source_path,
        name,
        version,
        api_version,
        kind,
        capabilities,
        enabled,
        priority,
        description: header.get("description").cloned(),
        author: header.get("author").cloned(),
        homepage: header.get("homepage").cloned(),
        entry_code,
        config_json,
        readme,
    })
}

#[cfg(test)]
mod tests {
    use super::{parse_rmod, RmodParseError};

    #[test]
    fn parse_valid_rmod() {
        let input = r#"#!rmod/v1
name: test
version: 0.1.0
api_version: 1
kind: script
capabilities: keys,commands

---module.js---
export default function createModule() {}
---config.json---
{ "ok": true }
"#;

        let parsed = parse_rmod(input, "modules/test.rmod".to_string()).expect("rmod should parse");
        assert_eq!(parsed.name, "test");
        assert_eq!(parsed.api_version, 1);
        assert_eq!(parsed.capabilities.len(), 2);
    }

    #[test]
    fn parse_rejects_invalid_magic() {
        let input = "#!rmod/v0\nname: x\n";
        let err = parse_rmod(input, "x.rmod".to_string()).expect_err("should fail");
        assert!(matches!(err, RmodParseError::InvalidMagic));
        assert_eq!(err.code(), "RMOD_E_INVALID_MAGIC");
    }

    #[test]
    fn parse_rejects_missing_module_js() {
        let input = r#"#!rmod/v1
name: test
version: 0.1.0
api_version: 1
kind: script
capabilities: keys

---readme.md---
hello
"#;
        let err = parse_rmod(input, "x.rmod".to_string()).expect_err("should fail");
        assert!(matches!(err, RmodParseError::MissingModuleJs));
        assert_eq!(err.code(), "RMOD_E_MISSING_MODULE_JS");
    }

    #[test]
    fn parse_rejects_invalid_config_json() {
        let input = r#"#!rmod/v1
name: test
version: 0.1.0
api_version: 1
kind: script
capabilities: keys

---module.js---
export default function createModule() {}
---config.json---
{ invalid json }
"#;
        let err = parse_rmod(input, "x.rmod".to_string()).expect_err("should fail");
        assert!(matches!(err, RmodParseError::ConfigNotJson(_)));
        assert_eq!(err.code(), "RMOD_E_CONFIG_NOT_JSON");
    }
}
