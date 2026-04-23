use std::fs;
use std::path::Path;

use super::manifest::{load_directory_descriptor, ManifestParseError};
use super::rmod::{parse_rmod, RmodParseError};
use super::types::ModuleDescriptor;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModuleLoadError {
    Io(String),
    Manifest(String),
    Rmod(String),
}

pub fn discover_module_descriptors(modules_dir: &Path) -> Result<Vec<ModuleDescriptor>, ModuleLoadError> {
    if !modules_dir.exists() {
        return Ok(Vec::new());
    }

    let mut descriptors = Vec::new();

    let entries = fs::read_dir(modules_dir).map_err(|err| ModuleLoadError::Io(err.to_string()))?;
    for entry in entries {
        let entry = entry.map_err(|err| ModuleLoadError::Io(err.to_string()))?;
        let path = entry.path();

        if path.is_dir() {
            let manifest_path = path.join("module.toml");
            if manifest_path.exists() {
                let descriptor = load_directory_descriptor(&path).map_err(map_manifest_err)?;
                descriptors.push(descriptor);
            }
            continue;
        }

        let is_rmod = path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.eq_ignore_ascii_case("rmod"))
            .unwrap_or(false);

        if is_rmod {
            let raw = fs::read_to_string(&path).map_err(|err| ModuleLoadError::Io(err.to_string()))?;
            let descriptor = parse_rmod(&raw, path.to_string_lossy().to_string()).map_err(map_rmod_err)?;
            descriptors.push(descriptor);
        }
    }

    descriptors.sort_by(|a, b| a.priority.cmp(&b.priority).then_with(|| a.name.cmp(&b.name)));
    Ok(descriptors)
}

fn map_manifest_err(err: ManifestParseError) -> ModuleLoadError {
    ModuleLoadError::Manifest(format!("{err:?}"))
}

fn map_rmod_err(err: RmodParseError) -> ModuleLoadError {
    ModuleLoadError::Rmod(format!("{}: {}", err.code(), err.message()))
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use super::discover_module_descriptors;

    fn test_dir(name: &str) -> PathBuf {
        let mut dir = std::env::temp_dir();
        dir.push(format!("rmenu-loader-test-{}-{}", name, std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("create test dir");
        dir
    }

    #[test]
    fn discover_mixed_directory_and_rmod() {
        let dir = test_dir("mixed");

        let rmod = dir.join("alpha.rmod");
        fs::write(
            &rmod,
            r#"#!rmod/v1
name: alpha
version: 0.1.0
api_version: 1
kind: script
capabilities: providers

---module.js---
export default function createModule() {}
"#,
        )
        .expect("write rmod");

        let beta_dir = dir.join("beta");
        fs::create_dir_all(&beta_dir).expect("create beta dir");
        fs::write(
            beta_dir.join("module.toml"),
            r#"name = "beta"
version = "0.1.0"
api_version = 1
kind = "script"
entry = "index.js"
capabilities = ["commands"]
"#,
        )
        .expect("write manifest");
        fs::write(
            beta_dir.join("index.js"),
            "export default function createModule() {}",
        )
        .expect("write index.js");

        let descriptors = discover_module_descriptors(&dir).expect("discover descriptors");
        let names = descriptors.iter().map(|d| d.name.as_str()).collect::<Vec<_>>();
        assert!(names.contains(&"alpha"));
        assert!(names.contains(&"beta"));

        let _ = fs::remove_dir_all(&dir);
    }
}
