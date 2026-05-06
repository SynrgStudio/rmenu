#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::fs;
use std::io::{self, Read};
#[cfg(windows)]
use std::os::windows::ffi::OsStrExt;
use std::path::{Component, Path, PathBuf};
#[cfg(windows)]
use windows::core::PCWSTR;
#[cfg(windows)]
use windows::Win32::System::Com::Urlmon::URLDownloadToFileW;

use crate::modules::manifest::load_directory_descriptor;
use crate::modules::rmod::parse_rmod;
use crate::settings::rmenu_data_dirs;

pub const RMODS_REGISTRY_SCHEMA_VERSION: u32 = 1;
pub const DEFAULT_RMODS_REGISTRY_URL: &str =
    "https://raw.githubusercontent.com/SynrgStudio/rmods/main/registry.json";
pub const RMODS_INSTALLED_STATE_FILE: &str = "rmods-installed.json";
pub const RMODS_REGISTRY_CACHE_FILE: &str = "rmods-registry-cache.json";
pub const RMODS_DOWNLOADS_DIR: &str = "downloads";
pub const RMODS_PACKAGE_KIND_RMOD: &str = "rmod";
pub const RMODS_PACKAGE_KIND_RPACK: &str = "rpack";
pub const RMODS_MAX_PACKAGE_SIZE_BYTES: u64 = 10 * 1024 * 1024;
pub const RMODS_MAX_RPACK_FILES: usize = 256;
pub const RMODS_MAX_RPACK_TOTAL_SIZE_BYTES: u64 = 50 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct RmodsRegistry {
    pub schema: u32,
    pub generated_at: String,
    pub modules: Vec<RmodsRegistryItem>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct RmodsRegistryItem {
    pub id: String,
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub description: String,
    pub kind: String,
    #[serde(default)]
    pub download_url: String,
    #[serde(default)]
    pub base_url: String,
    pub sha256: String,
    pub size: u64,
    #[serde(default)]
    pub files: Vec<RmodsRegistryFile>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub requires_rmenu: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct RmodsRegistryFile {
    pub path: String,
    pub sha256: String,
    pub size: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Default)]
pub struct RmodsInstalledState {
    #[serde(default)]
    pub modules: BTreeMap<String, RmodsInstalledModule>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct RmodsInstalledModule {
    pub version: String,
    pub sha256: String,
    pub path: PathBuf,
    #[serde(default = "default_installed_kind")]
    pub kind: String,
    pub source_registry: String,
    pub installed_at: String,
}

fn default_installed_kind() -> String {
    RMODS_PACKAGE_KIND_RMOD.to_string()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RmodsLocalModule {
    pub id: String,
    pub version: String,
    pub sha256: String,
    pub path: PathBuf,
    pub kind: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RmodsInstallStatus {
    NotInstalled,
    Installed,
    UpdateAvailable,
    LocalNewer,
    ChecksumMismatch,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RmodsRegistryError {
    Json(String),
    UnsupportedSchema(u32),
    EmptyGeneratedAt,
    EmptyModuleList,
    DuplicateId(String),
    InvalidId(String),
    EmptyField { id: String, field: &'static str },
    UnsupportedKind { id: String, kind: String },
    InvalidDownloadUrl { id: String, url: String },
    InvalidSha256 { id: String, sha256: String },
    InvalidSize { id: String, size: u64 },
    InvalidTag { id: String, tag: String },
    Io(String),
    Fetch(String),
    RmodParse { path: PathBuf, message: String },
}

impl RmodsRegistryError {
    pub fn message(&self) -> String {
        match self {
            Self::Json(error) => format!("failed to parse rMods registry JSON: {error}"),
            Self::UnsupportedSchema(schema) => {
                format!("unsupported rMods registry schema: {schema}")
            }
            Self::EmptyGeneratedAt => "rMods registry generated_at is empty".to_string(),
            Self::EmptyModuleList => "rMods registry contains no modules".to_string(),
            Self::DuplicateId(id) => format!("duplicate rMods module id: {id}"),
            Self::InvalidId(id) => format!("invalid rMods module id: {id}"),
            Self::EmptyField { id, field } => {
                format!("rMods module {id} has empty required field: {field}")
            }
            Self::UnsupportedKind { id, kind } => {
                format!("rMods module {id} has unsupported package kind: {kind}")
            }
            Self::InvalidDownloadUrl { id, url } => {
                format!("rMods module {id} has invalid download_url: {url}")
            }
            Self::InvalidSha256 { id, sha256 } => {
                format!("rMods module {id} has invalid sha256: {sha256}")
            }
            Self::InvalidSize { id, size } => {
                format!("rMods module {id} has invalid size: {size}")
            }
            Self::InvalidTag { id, tag } => {
                format!("rMods module {id} has invalid tag: {tag}")
            }
            Self::Io(error) => format!("rMods registry I/O error: {error}"),
            Self::Fetch(error) => format!("rMods registry fetch failed: {error}"),
            Self::RmodParse { path, message } => {
                format!(
                    "failed to parse installed rmod {}: {message}",
                    path.display()
                )
            }
        }
    }
}

pub fn parse_registry_json(content: &str) -> Result<RmodsRegistry, RmodsRegistryError> {
    let registry = serde_json::from_str::<RmodsRegistry>(content)
        .map_err(|error| RmodsRegistryError::Json(error.to_string()))?;
    validate_registry(registry)
}

pub fn read_installed_state(
    cli_data_dir: Option<&str>,
) -> Result<RmodsInstalledState, RmodsRegistryError> {
    let path = rmods_installed_state_path(cli_data_dir);
    if !path.exists() {
        return Ok(RmodsInstalledState::default());
    }
    let content = fs::read_to_string(path).map_err(io_error)?;
    serde_json::from_str::<RmodsInstalledState>(&content)
        .map_err(|error| RmodsRegistryError::Json(error.to_string()))
}

pub fn write_installed_state(
    cli_data_dir: Option<&str>,
    state: &RmodsInstalledState,
) -> Result<PathBuf, RmodsRegistryError> {
    let path = rmods_installed_state_path(cli_data_dir);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(io_error)?;
    }
    let content = serde_json::to_string_pretty(state)
        .map_err(|error| RmodsRegistryError::Json(error.to_string()))?;
    fs::write(&path, format!("{content}\n")).map_err(io_error)?;
    Ok(path)
}

pub fn scan_installed_rmods(
    cli_data_dir: Option<&str>,
) -> Result<BTreeMap<String, RmodsLocalModule>, RmodsRegistryError> {
    let modules_dir = rmenu_data_dirs(cli_data_dir).modules_dir;
    let mut modules = BTreeMap::new();
    if !modules_dir.exists() {
        return Ok(modules);
    }

    let installed_state = read_installed_state(cli_data_dir).unwrap_or_default();

    for entry in fs::read_dir(modules_dir).map_err(io_error)? {
        let path = entry.map_err(io_error)?.path();
        if path.is_dir() {
            if !path.join("module.toml").exists() {
                continue;
            }
            let descriptor = load_directory_descriptor(&path).map_err(|error| {
                RmodsRegistryError::RmodParse {
                    path: path.clone(),
                    message: format!("{error:?}"),
                }
            })?;
            let id_lc = descriptor.name.to_ascii_lowercase();
            let sha256 = installed_state
                .modules
                .get(&descriptor.name)
                .filter(|installed| installed.kind == RMODS_PACKAGE_KIND_RPACK)
                .filter(|installed| installed.version == descriptor.version)
                .filter(|installed| installed.path == path)
                .map(|installed| installed.sha256.clone())
                .unwrap_or_else(|| sha256_directory(&path).unwrap_or_default());
            modules.insert(
                id_lc,
                RmodsLocalModule {
                    id: descriptor.name,
                    version: descriptor.version,
                    sha256,
                    path,
                    kind: RMODS_PACKAGE_KIND_RPACK.to_string(),
                },
            );
            continue;
        }
        if path.extension().and_then(|value| value.to_str()) != Some("rmod") {
            continue;
        }
        let content = fs::read_to_string(&path).map_err(io_error)?;
        let descriptor =
            parse_rmod(&content, path.to_string_lossy().to_string()).map_err(|error| {
                RmodsRegistryError::RmodParse {
                    path: path.clone(),
                    message: error.message(),
                }
            })?;
        let sha256 = sha256_file(&path)?;
        modules.insert(
            descriptor.name.to_ascii_lowercase(),
            RmodsLocalModule {
                id: descriptor.name,
                version: descriptor.version,
                sha256,
                path,
                kind: RMODS_PACKAGE_KIND_RMOD.to_string(),
            },
        );
    }

    Ok(modules)
}

pub fn install_status_for(
    registry_item: &RmodsRegistryItem,
    local: Option<&RmodsLocalModule>,
) -> RmodsInstallStatus {
    let Some(local) = local else {
        return RmodsInstallStatus::NotInstalled;
    };

    if local.kind != registry_item.kind {
        return RmodsInstallStatus::ChecksumMismatch;
    }

    match compare_versions(&local.version, &registry_item.version) {
        Ordering::Less => RmodsInstallStatus::UpdateAvailable,
        Ordering::Equal if local.sha256 == registry_item.sha256 => RmodsInstallStatus::Installed,
        Ordering::Equal => RmodsInstallStatus::ChecksumMismatch,
        Ordering::Greater => RmodsInstallStatus::LocalNewer,
    }
}

pub fn sha256_file(path: &PathBuf) -> Result<String, RmodsRegistryError> {
    let mut file = fs::File::open(path).map_err(io_error)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];
    loop {
        let read = file.read(&mut buffer).map_err(io_error)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

pub fn sha256_directory(path: &Path) -> Result<String, RmodsRegistryError> {
    let mut files = Vec::new();
    collect_directory_files(path, path, &mut files)?;
    files.sort_by(|left, right| left.0.cmp(&right.0));
    let mut hasher = Sha256::new();
    for (relative_path, file_path) in files {
        let file_sha = sha256_file(&file_path)?;
        let size = fs::metadata(&file_path).map_err(io_error)?.len();
        hasher.update(relative_path.as_bytes());
        hasher.update(b"\0");
        hasher.update(file_sha.as_bytes());
        hasher.update(b"\0");
        hasher.update(size.to_string().as_bytes());
        hasher.update(b"\n");
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn collect_directory_files(
    root: &Path,
    current: &Path,
    files: &mut Vec<(String, PathBuf)>,
) -> Result<(), RmodsRegistryError> {
    for entry in fs::read_dir(current).map_err(io_error)? {
        let path = entry.map_err(io_error)?.path();
        if path.is_dir() {
            collect_directory_files(root, &path, files)?;
            continue;
        }
        if !path.is_file() {
            continue;
        }
        let relative = path
            .strip_prefix(root)
            .map_err(|error| RmodsRegistryError::Io(error.to_string()))?
            .to_string_lossy()
            .replace('\\', "/");
        files.push((relative, path));
    }
    Ok(())
}

fn compare_versions(left: &str, right: &str) -> Ordering {
    let left_parts = parse_numeric_version(left);
    let right_parts = parse_numeric_version(right);
    match (left_parts, right_parts) {
        (Some(left_parts), Some(right_parts)) => left_parts.cmp(&right_parts),
        _ => left.cmp(right),
    }
}

fn parse_numeric_version(value: &str) -> Option<Vec<u64>> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }
    let mut parts = Vec::new();
    for part in trimmed.split('.') {
        parts.push(part.parse::<u64>().ok()?);
    }
    Some(parts)
}

pub fn read_registry_cache(
    cli_data_dir: Option<&str>,
) -> Result<RmodsRegistry, RmodsRegistryError> {
    let path = rmods_registry_cache_path(cli_data_dir);
    let content = fs::read_to_string(&path).map_err(io_error)?;
    parse_registry_json(&content)
}

pub fn write_registry_cache(
    cli_data_dir: Option<&str>,
    registry: &RmodsRegistry,
) -> Result<PathBuf, RmodsRegistryError> {
    let path = rmods_registry_cache_path(cli_data_dir);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(io_error)?;
    }
    let content = serde_json::to_string_pretty(registry)
        .map_err(|error| RmodsRegistryError::Json(error.to_string()))?;
    fs::write(&path, format!("{content}\n")).map_err(io_error)?;
    Ok(path)
}

pub fn fetch_registry(
    url: &str,
    cli_data_dir: Option<&str>,
) -> Result<RmodsRegistry, RmodsRegistryError> {
    let content = fetch_text(url, cli_data_dir)?;
    let registry = parse_registry_json(&content)?;
    write_registry_cache(cli_data_dir, &registry)?;
    Ok(registry)
}

pub fn fetch_default_registry(
    cli_data_dir: Option<&str>,
) -> Result<RmodsRegistry, RmodsRegistryError> {
    fetch_registry(DEFAULT_RMODS_REGISTRY_URL, cli_data_dir)
}

pub fn uninstall_rmod(id: &str, cli_data_dir: Option<&str>) -> Result<bool, RmodsRegistryError> {
    if !is_safe_module_id(id) {
        return Err(RmodsRegistryError::InvalidId(id.to_string()));
    }

    let dirs = rmenu_data_dirs(cli_data_dir);
    let rmod_path = dirs.modules_dir.join(format!("{id}.rmod"));
    let rpack_path = dirs.modules_dir.join(id);
    let mut removed = false;
    if rmod_path.exists() {
        fs::remove_file(&rmod_path).map_err(io_error)?;
        removed = true;
    }
    if rpack_path.exists() {
        fs::remove_dir_all(&rpack_path).map_err(io_error)?;
        removed = true;
    }

    let mut state = read_installed_state(cli_data_dir)?;
    removed |= state.modules.remove(id).is_some();
    write_installed_state(cli_data_dir, &state)?;
    Ok(removed)
}

pub fn download_verify_and_install_rmod(
    item: &RmodsRegistryItem,
    cli_data_dir: Option<&str>,
    source_registry: &str,
) -> Result<RmodsInstalledModule, RmodsRegistryError> {
    validate_item(item)?;
    if item.kind == RMODS_PACKAGE_KIND_RPACK {
        return download_verify_and_install_rpack(item, cli_data_dir, source_registry);
    }

    let downloaded = download_rmod_to_temp(item, cli_data_dir)?;
    let actual_size = fs::metadata(&downloaded).map_err(io_error)?.len();
    if actual_size != item.size {
        let _ = fs::remove_file(&downloaded);
        return Err(RmodsRegistryError::InvalidSize {
            id: item.id.clone(),
            size: actual_size,
        });
    }
    let actual_sha = sha256_file(&downloaded)?;
    if actual_sha != item.sha256 {
        let _ = fs::remove_file(&downloaded);
        return Err(RmodsRegistryError::InvalidSha256 {
            id: item.id.clone(),
            sha256: actual_sha,
        });
    }
    let content = fs::read_to_string(&downloaded).map_err(io_error)?;
    let descriptor =
        parse_rmod(&content, downloaded.to_string_lossy().to_string()).map_err(|error| {
            RmodsRegistryError::RmodParse {
                path: downloaded.clone(),
                message: error.message(),
            }
        })?;
    if descriptor.name != item.id {
        let _ = fs::remove_file(&downloaded);
        return Err(RmodsRegistryError::InvalidId(descriptor.name));
    }
    if descriptor.version != item.version {
        let _ = fs::remove_file(&downloaded);
        return Err(RmodsRegistryError::EmptyField {
            id: item.id.clone(),
            field: "version mismatch",
        });
    }

    install_verified_rmod(item, &downloaded, cli_data_dir, source_registry)
}

fn download_rmod_to_temp(
    item: &RmodsRegistryItem,
    cli_data_dir: Option<&str>,
) -> Result<PathBuf, RmodsRegistryError> {
    let downloads_dir = rmods_downloads_dir(cli_data_dir);
    fs::create_dir_all(&downloads_dir).map_err(io_error)?;
    let path = downloads_dir.join(format!("{}.rmod.tmp", item.id));

    copy_or_download_url_to_file(&item.download_url, &path)?;
    Ok(path)
}

fn install_verified_rmod(
    item: &RmodsRegistryItem,
    downloaded: &PathBuf,
    cli_data_dir: Option<&str>,
    source_registry: &str,
) -> Result<RmodsInstalledModule, RmodsRegistryError> {
    let dirs = rmenu_data_dirs(cli_data_dir);
    fs::create_dir_all(&dirs.modules_dir).map_err(io_error)?;
    let final_path = dirs.modules_dir.join(format!("{}.rmod", item.id));
    let staging_path = dirs
        .modules_dir
        .join(format!("{}.rmod.installing", item.id));
    let backup_path = dirs.modules_dir.join(format!("{}.rmod.bak", item.id));

    let _ = fs::remove_file(&staging_path);
    fs::copy(downloaded, &staging_path).map_err(io_error)?;

    if final_path.exists() {
        let _ = fs::remove_file(&backup_path);
        fs::rename(&final_path, &backup_path).map_err(io_error)?;
    }

    if let Err(error) = fs::rename(&staging_path, &final_path) {
        if backup_path.exists() {
            let _ = fs::rename(&backup_path, &final_path);
        }
        return Err(io_error(error));
    }

    let _ = fs::remove_file(&backup_path);
    let _ = fs::remove_file(downloaded);

    let installed = RmodsInstalledModule {
        version: item.version.clone(),
        sha256: item.sha256.clone(),
        path: final_path,
        kind: RMODS_PACKAGE_KIND_RMOD.to_string(),
        source_registry: source_registry.to_string(),
        installed_at: current_timestamp_string(),
    };
    let mut state = read_installed_state(cli_data_dir)?;
    state.modules.insert(item.id.clone(), installed.clone());
    write_installed_state(cli_data_dir, &state)?;
    Ok(installed)
}

fn download_verify_and_install_rpack(
    item: &RmodsRegistryItem,
    cli_data_dir: Option<&str>,
    source_registry: &str,
) -> Result<RmodsInstalledModule, RmodsRegistryError> {
    let staging_path =
        rmods_downloads_dir(cli_data_dir).join(format!("{}.rpack.installing", item.id));
    let _ = fs::remove_dir_all(&staging_path);
    fs::create_dir_all(&staging_path).map_err(io_error)?;

    let mut total_size = 0u64;
    for file in &item.files {
        let relative_path = safe_relative_path(&file.path)?;
        total_size = total_size.saturating_add(file.size);
        if total_size > RMODS_MAX_RPACK_TOTAL_SIZE_BYTES {
            let _ = fs::remove_dir_all(&staging_path);
            return Err(RmodsRegistryError::InvalidSize {
                id: item.id.clone(),
                size: total_size,
            });
        }
        let target_path = staging_path.join(&relative_path);
        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent).map_err(io_error)?;
        }
        let url = rpack_file_url(item, &file.path);
        copy_or_download_url_to_file(&url, &target_path)?;
        let actual_size = fs::metadata(&target_path).map_err(io_error)?.len();
        if actual_size != file.size {
            let _ = fs::remove_dir_all(&staging_path);
            return Err(RmodsRegistryError::InvalidSize {
                id: item.id.clone(),
                size: actual_size,
            });
        }
        let actual_sha = sha256_file(&target_path)?;
        if actual_sha != file.sha256 {
            let _ = fs::remove_dir_all(&staging_path);
            return Err(RmodsRegistryError::InvalidSha256 {
                id: item.id.clone(),
                sha256: actual_sha,
            });
        }
    }

    let descriptor = load_directory_descriptor(&staging_path).map_err(|error| {
        let _ = fs::remove_dir_all(&staging_path);
        RmodsRegistryError::RmodParse {
            path: staging_path.clone(),
            message: format!("{error:?}"),
        }
    })?;
    if descriptor.name != item.id {
        let _ = fs::remove_dir_all(&staging_path);
        return Err(RmodsRegistryError::InvalidId(descriptor.name));
    }
    if descriptor.version != item.version {
        let _ = fs::remove_dir_all(&staging_path);
        return Err(RmodsRegistryError::EmptyField {
            id: item.id.clone(),
            field: "version mismatch",
        });
    }
    let actual_sha = sha256_directory(&staging_path)?;
    if actual_sha != item.sha256 {
        let _ = fs::remove_dir_all(&staging_path);
        return Err(RmodsRegistryError::InvalidSha256 {
            id: item.id.clone(),
            sha256: actual_sha,
        });
    }

    install_verified_rpack(item, &staging_path, cli_data_dir, source_registry)
}

fn install_verified_rpack(
    item: &RmodsRegistryItem,
    staging_path: &Path,
    cli_data_dir: Option<&str>,
    source_registry: &str,
) -> Result<RmodsInstalledModule, RmodsRegistryError> {
    let dirs = rmenu_data_dirs(cli_data_dir);
    fs::create_dir_all(&dirs.modules_dir).map_err(io_error)?;
    let final_path = dirs.modules_dir.join(&item.id);
    let backup_path = dirs.modules_dir.join(format!("{}.rpack.bak", item.id));
    let conflicting_rmod = dirs.modules_dir.join(format!("{}.rmod", item.id));

    let _ = fs::remove_dir_all(&backup_path);
    if final_path.exists() {
        fs::rename(&final_path, &backup_path).map_err(io_error)?;
    }
    if conflicting_rmod.exists() {
        fs::remove_file(&conflicting_rmod).map_err(io_error)?;
    }

    if let Err(error) = fs::rename(staging_path, &final_path) {
        if backup_path.exists() {
            let _ = fs::rename(&backup_path, &final_path);
        }
        return Err(io_error(error));
    }

    let _ = fs::remove_dir_all(&backup_path);

    let installed = RmodsInstalledModule {
        version: item.version.clone(),
        sha256: item.sha256.clone(),
        path: final_path,
        kind: RMODS_PACKAGE_KIND_RPACK.to_string(),
        source_registry: source_registry.to_string(),
        installed_at: current_timestamp_string(),
    };
    let mut state = read_installed_state(cli_data_dir)?;
    state.modules.insert(item.id.clone(), installed.clone());
    write_installed_state(cli_data_dir, &state)?;
    Ok(installed)
}

fn rpack_file_url(item: &RmodsRegistryItem, path: &str) -> String {
    format!(
        "{}/{}",
        item.base_url.trim_end_matches('/'),
        path.replace('\\', "/")
    )
}

fn safe_relative_path(path: &str) -> Result<PathBuf, RmodsRegistryError> {
    let normalized = path.replace('\\', "/");
    if normalized.trim().is_empty() || normalized.starts_with('/') || normalized.contains('\0') {
        return Err(RmodsRegistryError::InvalidDownloadUrl {
            id: path.to_string(),
            url: path.to_string(),
        });
    }
    let candidate = Path::new(&normalized);
    if candidate.is_absolute() {
        return Err(RmodsRegistryError::InvalidDownloadUrl {
            id: path.to_string(),
            url: path.to_string(),
        });
    }
    let mut output = PathBuf::new();
    for component in candidate.components() {
        match component {
            Component::Normal(value) => output.push(value),
            _ => {
                return Err(RmodsRegistryError::InvalidDownloadUrl {
                    id: path.to_string(),
                    url: path.to_string(),
                });
            }
        }
    }
    Ok(output)
}

fn copy_or_download_url_to_file(url: &str, path: &PathBuf) -> Result<(), RmodsRegistryError> {
    if let Some(file_path) = url.strip_prefix("file://") {
        fs::copy(file_path, path).map_err(io_error)?;
        return Ok(());
    }
    download_url_to_file(url, path)
}

fn fetch_text(url: &str, cli_data_dir: Option<&str>) -> Result<String, RmodsRegistryError> {
    if url.trim().is_empty() {
        return Err(RmodsRegistryError::Fetch("empty URL".to_string()));
    }

    if let Some(path) = url.strip_prefix("file://") {
        return fs::read_to_string(path).map_err(io_error);
    }

    fetch_http_text(url, cli_data_dir)
}

#[cfg(windows)]
fn fetch_http_text(url: &str, cli_data_dir: Option<&str>) -> Result<String, RmodsRegistryError> {
    let downloads_dir = rmods_downloads_dir(cli_data_dir);
    fs::create_dir_all(&downloads_dir).map_err(io_error)?;
    let path = downloads_dir.join("registry-fetch.tmp");
    download_url_to_file(url, &path)?;
    let content = fs::read_to_string(&path).map_err(io_error)?;
    let _ = fs::remove_file(path);
    Ok(content)
}

#[cfg(windows)]
fn download_url_to_file(url: &str, path: &PathBuf) -> Result<(), RmodsRegistryError> {
    if !url.starts_with("https://") && !url.starts_with("http://") {
        return Err(RmodsRegistryError::Fetch(format!(
            "unsupported URL scheme: {url}"
        )));
    }
    let wide_url = wide_null(url);
    let wide_path = wide_null(&path.to_string_lossy());
    unsafe {
        URLDownloadToFileW(
            None,
            PCWSTR(wide_url.as_ptr()),
            PCWSTR(wide_path.as_ptr()),
            0,
            None,
        )
    }
    .map_err(|error| RmodsRegistryError::Fetch(format!("download failed: {error}")))
}

#[cfg(not(windows))]
fn download_url_to_file(_url: &str, _path: &PathBuf) -> Result<(), RmodsRegistryError> {
    Err(RmodsRegistryError::Fetch(
        "HTTP download is only implemented on Windows".to_string(),
    ))
}

#[cfg(not(windows))]
fn fetch_http_text(_url: &str, _cli_data_dir: Option<&str>) -> Result<String, RmodsRegistryError> {
    Err(RmodsRegistryError::Fetch(
        "HTTP registry fetch is only implemented on Windows".to_string(),
    ))
}

#[cfg(windows)]
fn wide_null(value: &str) -> Vec<u16> {
    std::ffi::OsStr::new(value)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

fn io_error(error: io::Error) -> RmodsRegistryError {
    RmodsRegistryError::Io(error.to_string())
}

fn current_timestamp_string() -> String {
    let seconds = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0);
    format!("unix:{seconds}")
}

pub fn validate_registry(mut registry: RmodsRegistry) -> Result<RmodsRegistry, RmodsRegistryError> {
    if registry.schema != RMODS_REGISTRY_SCHEMA_VERSION {
        return Err(RmodsRegistryError::UnsupportedSchema(registry.schema));
    }
    if registry.generated_at.trim().is_empty() {
        return Err(RmodsRegistryError::EmptyGeneratedAt);
    }
    if registry.modules.is_empty() {
        return Err(RmodsRegistryError::EmptyModuleList);
    }

    registry.modules.sort_by(|left, right| {
        left.id
            .to_ascii_lowercase()
            .cmp(&right.id.to_ascii_lowercase())
    });

    let mut seen = std::collections::BTreeSet::new();
    for item in &registry.modules {
        validate_item(item)?;
        let id_lc = item.id.to_ascii_lowercase();
        if !seen.insert(id_lc) {
            return Err(RmodsRegistryError::DuplicateId(item.id.clone()));
        }
    }

    Ok(registry)
}

fn validate_item(item: &RmodsRegistryItem) -> Result<(), RmodsRegistryError> {
    if !is_safe_module_id(&item.id) {
        return Err(RmodsRegistryError::InvalidId(item.id.clone()));
    }
    validate_non_empty(&item.id, "name", &item.name)?;
    validate_non_empty(&item.id, "version", &item.version)?;
    validate_non_empty(&item.id, "kind", &item.kind)?;
    validate_non_empty(&item.id, "sha256", &item.sha256)?;

    match item.kind.as_str() {
        RMODS_PACKAGE_KIND_RMOD => {
            validate_non_empty(&item.id, "download_url", &item.download_url)?;
            if !item.base_url.trim().is_empty() || !item.files.is_empty() {
                return Err(RmodsRegistryError::UnsupportedKind {
                    id: item.id.clone(),
                    kind: "rmod with rpack fields".to_string(),
                });
            }
            if !is_valid_rmod_download_url(&item.download_url) {
                return Err(RmodsRegistryError::InvalidDownloadUrl {
                    id: item.id.clone(),
                    url: item.download_url.clone(),
                });
            }
        }
        RMODS_PACKAGE_KIND_RPACK => {
            validate_non_empty(&item.id, "base_url", &item.base_url)?;
            if !item.download_url.trim().is_empty() {
                return Err(RmodsRegistryError::UnsupportedKind {
                    id: item.id.clone(),
                    kind: "rpack with download_url".to_string(),
                });
            }
            validate_rpack_files(item)?;
        }
        _ => {
            return Err(RmodsRegistryError::UnsupportedKind {
                id: item.id.clone(),
                kind: item.kind.clone(),
            });
        }
    }
    if !is_valid_sha256(&item.sha256) {
        return Err(RmodsRegistryError::InvalidSha256 {
            id: item.id.clone(),
            sha256: item.sha256.clone(),
        });
    }
    let max_size = if item.kind == RMODS_PACKAGE_KIND_RPACK {
        RMODS_MAX_RPACK_TOTAL_SIZE_BYTES
    } else {
        RMODS_MAX_PACKAGE_SIZE_BYTES
    };
    if item.size == 0 || item.size > max_size {
        return Err(RmodsRegistryError::InvalidSize {
            id: item.id.clone(),
            size: item.size,
        });
    }
    for tag in &item.tags {
        if tag.trim().is_empty() || tag.len() > 64 {
            return Err(RmodsRegistryError::InvalidTag {
                id: item.id.clone(),
                tag: tag.clone(),
            });
        }
    }
    Ok(())
}

fn validate_rpack_files(item: &RmodsRegistryItem) -> Result<(), RmodsRegistryError> {
    if item.files.is_empty() || item.files.len() > RMODS_MAX_RPACK_FILES {
        return Err(RmodsRegistryError::InvalidSize {
            id: item.id.clone(),
            size: item.files.len() as u64,
        });
    }
    if !is_valid_base_url(&item.base_url) {
        return Err(RmodsRegistryError::InvalidDownloadUrl {
            id: item.id.clone(),
            url: item.base_url.clone(),
        });
    }
    let mut total_size = 0u64;
    let mut seen = std::collections::BTreeSet::new();
    let mut has_manifest = false;
    for file in &item.files {
        if safe_relative_path(&file.path).is_err() || !seen.insert(file.path.to_ascii_lowercase()) {
            return Err(RmodsRegistryError::InvalidDownloadUrl {
                id: item.id.clone(),
                url: file.path.clone(),
            });
        }
        if file.path.replace('\\', "/") == "module.toml" {
            has_manifest = true;
        }
        if !is_valid_sha256(&file.sha256) {
            return Err(RmodsRegistryError::InvalidSha256 {
                id: item.id.clone(),
                sha256: file.sha256.clone(),
            });
        }
        if file.size == 0 || file.size > RMODS_MAX_PACKAGE_SIZE_BYTES {
            return Err(RmodsRegistryError::InvalidSize {
                id: item.id.clone(),
                size: file.size,
            });
        }
        total_size = total_size.saturating_add(file.size);
    }
    if !has_manifest || total_size == 0 || total_size > RMODS_MAX_RPACK_TOTAL_SIZE_BYTES {
        return Err(RmodsRegistryError::InvalidSize {
            id: item.id.clone(),
            size: total_size,
        });
    }
    if item.size != total_size {
        return Err(RmodsRegistryError::InvalidSize {
            id: item.id.clone(),
            size: item.size,
        });
    }
    Ok(())
}

fn validate_non_empty(
    id: &str,
    field: &'static str,
    value: &str,
) -> Result<(), RmodsRegistryError> {
    if value.trim().is_empty() {
        Err(RmodsRegistryError::EmptyField {
            id: id.to_string(),
            field,
        })
    } else {
        Ok(())
    }
}

pub fn is_safe_module_id(id: &str) -> bool {
    let len = id.len();
    len > 0
        && len <= 96
        && id
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.'))
        && !id.starts_with('.')
        && !id.ends_with('.')
        && !id.contains("..")
}

pub fn is_valid_sha256(value: &str) -> bool {
    value.len() == 64 && value.chars().all(|ch| ch.is_ascii_hexdigit())
}

fn is_valid_rmod_download_url(value: &str) -> bool {
    let trimmed = value.trim();
    is_valid_base_url(trimmed) && trimmed.ends_with(".rmod")
}

fn is_valid_base_url(value: &str) -> bool {
    let trimmed = value.trim();
    trimmed.starts_with("https://")
        || trimmed.starts_with("http://")
        || trimmed.starts_with("file://")
}

pub fn rmods_installed_state_path(cli_data_dir: Option<&str>) -> PathBuf {
    rmenu_data_dirs(cli_data_dir)
        .state_dir
        .join(RMODS_INSTALLED_STATE_FILE)
}

pub fn rmods_registry_cache_path(cli_data_dir: Option<&str>) -> PathBuf {
    rmenu_data_dirs(cli_data_dir)
        .state_dir
        .join(RMODS_REGISTRY_CACHE_FILE)
}

pub fn rmods_downloads_dir(cli_data_dir: Option<&str>) -> PathBuf {
    rmenu_data_dirs(cli_data_dir)
        .state_dir
        .join(RMODS_DOWNLOADS_DIR)
}

#[cfg(test)]
mod tests {
    use super::*;

    const CALCULATOR_REGISTRY: &str = r#"{
  "schema": 1,
  "generated_at": "2026-05-06T05:28:40Z",
  "modules": [
    {
      "id": "calculator",
      "name": "calculator",
      "version": "0.1.0",
      "description": "Shows simple arithmetic results in the input bar.",
      "kind": "rmod",
      "download_url": "https://raw.githubusercontent.com/SynrgStudio/rmods/main/modules/calculator.rmod",
      "sha256": "de00cc81828884f32688d344099e8fb2553887d7d30fc652d0b3b1e0f5c7f227",
      "size": 3021,
      "tags": []
    }
  ]
}"#;

    #[test]
    fn rmods_parse_valid_calculator_registry() {
        let registry = parse_registry_json(CALCULATOR_REGISTRY).expect("valid registry");
        assert_eq!(registry.schema, 1);
        assert_eq!(registry.modules.len(), 1);
        let calculator = &registry.modules[0];
        assert_eq!(calculator.id, "calculator");
        assert_eq!(calculator.kind, "rmod");
        assert_eq!(calculator.size, 3021);
    }

    #[test]
    fn rmods_rejects_unsupported_schema() {
        let json = CALCULATOR_REGISTRY.replace("\"schema\": 1", "\"schema\": 2");
        let error = parse_registry_json(&json).expect_err("schema should fail");
        assert!(matches!(error, RmodsRegistryError::UnsupportedSchema(2)));
    }

    #[test]
    fn rmods_rejects_unsafe_ids() {
        let json = CALCULATOR_REGISTRY.replace("\"id\": \"calculator\"", "\"id\": \"../bad\"");
        let error = parse_registry_json(&json).expect_err("id should fail");
        assert!(matches!(error, RmodsRegistryError::InvalidId(id) if id == "../bad"));
    }

    #[test]
    fn rmods_rejects_duplicate_ids_case_insensitive() {
        let registry = RmodsRegistry {
            schema: 1,
            generated_at: "2026-05-06T00:00:00Z".to_string(),
            modules: vec![sample_item("calculator"), sample_item("Calculator")],
        };
        let error = validate_registry(registry).expect_err("duplicate should fail");
        assert!(matches!(error, RmodsRegistryError::DuplicateId(id) if id == "Calculator"));
    }

    #[test]
    fn rmods_rejects_invalid_kind() {
        let json = CALCULATOR_REGISTRY.replace("\"kind\": \"rmod\"", "\"kind\": \"zip\"");
        let error = parse_registry_json(&json).expect_err("kind should fail");
        assert!(matches!(error, RmodsRegistryError::UnsupportedKind { kind, .. } if kind == "zip"));
    }

    #[test]
    fn rmods_rejects_invalid_sha256() {
        let json = CALCULATOR_REGISTRY.replace(
            "de00cc81828884f32688d344099e8fb2553887d7d30fc652d0b3b1e0f5c7f227",
            "not-a-sha",
        );
        let error = parse_registry_json(&json).expect_err("sha should fail");
        assert!(matches!(error, RmodsRegistryError::InvalidSha256 { .. }));
    }

    #[test]
    fn rmods_rejects_zero_size() {
        let json = CALCULATOR_REGISTRY.replace("\"size\": 3021", "\"size\": 0");
        let error = parse_registry_json(&json).expect_err("size should fail");
        assert!(matches!(
            error,
            RmodsRegistryError::InvalidSize { size: 0, .. }
        ));
    }

    #[test]
    fn rmods_state_paths_use_data_root() {
        assert_eq!(
            rmods_installed_state_path(Some("C:\\rMenuData")),
            PathBuf::from("C:\\rMenuData\\state\\rmods-installed.json")
        );
        assert_eq!(
            rmods_registry_cache_path(Some("C:\\rMenuData")),
            PathBuf::from("C:\\rMenuData\\state\\rmods-registry-cache.json")
        );
        assert_eq!(
            rmods_downloads_dir(Some("C:\\rMenuData")),
            PathBuf::from("C:\\rMenuData\\state\\downloads")
        );
    }

    #[test]
    fn rmods_cache_roundtrip_uses_state_path() {
        let root = std::env::temp_dir().join(format!("rmods-cache-test-{}", std::process::id()));
        let root_string = root.to_string_lossy().to_string();
        let registry = parse_registry_json(CALCULATOR_REGISTRY).expect("valid registry");
        let path = write_registry_cache(Some(&root_string), &registry).expect("write cache");
        assert_eq!(path, root.join("state").join("rmods-registry-cache.json"));
        let cached = read_registry_cache(Some(&root_string)).expect("read cache");
        assert_eq!(cached.modules[0].id, "calculator");
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn rmods_fetch_file_url_writes_cache() {
        let root = std::env::temp_dir().join(format!("rmods-fetch-test-{}", std::process::id()));
        let root_string = root.to_string_lossy().to_string();
        let source = root.join("source-registry.json");
        std::fs::create_dir_all(&root).expect("create temp root");
        std::fs::write(&source, CALCULATOR_REGISTRY).expect("write source");
        let url = format!("file://{}", source.display());
        let registry = fetch_registry(&url, Some(&root_string)).expect("fetch file registry");
        assert_eq!(registry.modules[0].id, "calculator");
        assert!(root
            .join("state")
            .join("rmods-registry-cache.json")
            .exists());
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn rmods_installed_state_roundtrip() {
        let root =
            std::env::temp_dir().join(format!("rmods-installed-state-test-{}", std::process::id()));
        let root_string = root.to_string_lossy().to_string();
        let mut state = RmodsInstalledState::default();
        state.modules.insert(
            "calculator".to_string(),
            RmodsInstalledModule {
                version: "0.1.0".to_string(),
                sha256: "de00cc81828884f32688d344099e8fb2553887d7d30fc652d0b3b1e0f5c7f227"
                    .to_string(),
                path: PathBuf::from("C:\\rMenuData\\modules\\calculator.rmod"),
                kind: RMODS_PACKAGE_KIND_RMOD.to_string(),
                source_registry: DEFAULT_RMODS_REGISTRY_URL.to_string(),
                installed_at: "2026-05-06T00:00:00Z".to_string(),
            },
        );
        write_installed_state(Some(&root_string), &state).expect("write state");
        let loaded = read_installed_state(Some(&root_string)).expect("read state");
        assert_eq!(loaded.modules["calculator"].version, "0.1.0");
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn rmods_scan_installed_rmods_recovers_metadata() {
        let root =
            std::env::temp_dir().join(format!("rmods-installed-scan-test-{}", std::process::id()));
        let modules_dir = root.join("modules");
        std::fs::create_dir_all(&modules_dir).expect("create modules dir");
        let rmod_path = modules_dir.join("calculator.rmod");
        std::fs::write(&rmod_path, sample_rmod("calculator", "0.1.0")).expect("write sample rmod");
        let root_string = root.to_string_lossy().to_string();
        let modules = scan_installed_rmods(Some(&root_string)).expect("scan installed rmods");
        let calculator = modules.get("calculator").expect("calculator module");
        assert_eq!(calculator.version, "0.1.0");
        assert!(is_valid_sha256(&calculator.sha256));
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn rmods_install_status_detects_update_installed_local_newer_and_mismatch() {
        let item = sample_item("calculator");
        assert_eq!(
            install_status_for(&item, None),
            RmodsInstallStatus::NotInstalled
        );
        let mut local = RmodsLocalModule {
            id: "calculator".to_string(),
            version: "0.1.0".to_string(),
            sha256: item.sha256.clone(),
            path: PathBuf::from("calculator.rmod"),
            kind: RMODS_PACKAGE_KIND_RMOD.to_string(),
        };
        assert_eq!(
            install_status_for(&item, Some(&local)),
            RmodsInstallStatus::Installed
        );
        local.version = "0.0.9".to_string();
        local.sha256 = "different".to_string();
        assert_eq!(
            install_status_for(&item, Some(&local)),
            RmodsInstallStatus::UpdateAvailable
        );
        local.version = "0.2.0".to_string();
        assert_eq!(
            install_status_for(&item, Some(&local)),
            RmodsInstallStatus::LocalNewer
        );
        local.version = "0.1.0".to_string();
        assert_eq!(
            install_status_for(&item, Some(&local)),
            RmodsInstallStatus::ChecksumMismatch
        );
    }

    #[test]
    fn rmods_download_verify_and_install_file_url() {
        let root = std::env::temp_dir().join(format!("rmods-install-test-{}", std::process::id()));
        let source_dir = root.join("source");
        std::fs::create_dir_all(&source_dir).expect("create source dir");
        let source = source_dir.join("calculator.rmod");
        std::fs::write(&source, sample_rmod("calculator", "0.1.0")).expect("write source rmod");
        let sha256 = sha256_file(&source).expect("hash source");
        let item = RmodsRegistryItem {
            id: "calculator".to_string(),
            name: "calculator".to_string(),
            version: "0.1.0".to_string(),
            description: String::new(),
            kind: "rmod".to_string(),
            download_url: format!("file://{}", source.display()),
            base_url: String::new(),
            sha256,
            size: source.metadata().expect("metadata").len(),
            files: Vec::new(),
            tags: Vec::new(),
            requires_rmenu: None,
        };
        let root_string = root.to_string_lossy().to_string();
        let installed =
            download_verify_and_install_rmod(&item, Some(&root_string), DEFAULT_RMODS_REGISTRY_URL)
                .expect("install rmod");
        assert!(installed.path.exists());
        assert_eq!(installed.version, "0.1.0");
        let state = read_installed_state(Some(&root_string)).expect("read installed state");
        assert!(state.modules.contains_key("calculator"));
        let scanned = scan_installed_rmods(Some(&root_string)).expect("scan installed");
        assert!(scanned.contains_key("calculator"));
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn rmods_download_verify_and_install_rpack_file_url() {
        let root =
            std::env::temp_dir().join(format!("rmods-rpack-install-test-{}", std::process::id()));
        let source_dir = root.join("source").join("shortcuts");
        write_sample_rpack(&source_dir, "shortcuts", "0.3.0");
        let files = rpack_files_for(&source_dir);
        let size = files.iter().map(|file| file.size).sum();
        let item = RmodsRegistryItem {
            id: "shortcuts".to_string(),
            name: "shortcuts".to_string(),
            version: "0.3.0".to_string(),
            description: String::new(),
            kind: RMODS_PACKAGE_KIND_RPACK.to_string(),
            download_url: String::new(),
            base_url: format!("file://{}", source_dir.display()),
            sha256: sha256_directory(&source_dir).expect("hash rpack"),
            size,
            files,
            tags: Vec::new(),
            requires_rmenu: None,
        };
        validate_item(&item).expect("valid rpack item");
        let root_string = root.to_string_lossy().to_string();
        let installed =
            download_verify_and_install_rmod(&item, Some(&root_string), DEFAULT_RMODS_REGISTRY_URL)
                .expect("install rpack");
        assert!(installed.path.join("module.toml").exists());
        assert!(installed.path.join("module.js").exists());
        assert_eq!(installed.kind, RMODS_PACKAGE_KIND_RPACK);
        std::fs::write(installed.path.join("shortcuts.user.json"), "{}\n")
            .expect("write user state");
        let scanned = scan_installed_rmods(Some(&root_string)).expect("scan installed");
        assert_eq!(scanned["shortcuts"].kind, RMODS_PACKAGE_KIND_RPACK);
        assert_eq!(scanned["shortcuts"].sha256, item.sha256);
        let state = read_installed_state(Some(&root_string)).expect("read state");
        assert_eq!(state.modules["shortcuts"].kind, RMODS_PACKAGE_KIND_RPACK);
        assert!(uninstall_rmod("shortcuts", Some(&root_string)).expect("uninstall rpack"));
        assert!(!installed.path.exists());
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn rmods_rejects_unsafe_rpack_file_path() {
        let mut item = sample_item("bad-rpack");
        item.kind = RMODS_PACKAGE_KIND_RPACK.to_string();
        item.download_url = String::new();
        item.base_url = "https://example.test/rpacks/bad-rpack".to_string();
        item.files = vec![RmodsRegistryFile {
            path: "../module.toml".to_string(),
            sha256: "de00cc81828884f32688d344099e8fb2553887d7d30fc652d0b3b1e0f5c7f227".to_string(),
            size: 1,
        }];
        let error = validate_item(&item).expect_err("unsafe path should fail");
        assert!(matches!(
            error,
            RmodsRegistryError::InvalidDownloadUrl { .. }
        ));
    }

    #[test]
    fn rmods_uninstall_removes_file_and_state() {
        let root =
            std::env::temp_dir().join(format!("rmods-uninstall-test-{}", std::process::id()));
        let modules_dir = root.join("modules");
        std::fs::create_dir_all(&modules_dir).expect("create modules dir");
        let module_path = modules_dir.join("calculator.rmod");
        std::fs::write(&module_path, sample_rmod("calculator", "0.1.0")).expect("write module");
        let root_string = root.to_string_lossy().to_string();
        let mut state = RmodsInstalledState::default();
        state.modules.insert(
            "calculator".to_string(),
            RmodsInstalledModule {
                version: "0.1.0".to_string(),
                sha256: sha256_file(&module_path).expect("hash"),
                path: module_path.clone(),
                kind: RMODS_PACKAGE_KIND_RMOD.to_string(),
                source_registry: DEFAULT_RMODS_REGISTRY_URL.to_string(),
                installed_at: "unix:0".to_string(),
            },
        );
        write_installed_state(Some(&root_string), &state).expect("write state");
        assert!(uninstall_rmod("calculator", Some(&root_string)).expect("uninstall"));
        assert!(!module_path.exists());
        let state = read_installed_state(Some(&root_string)).expect("read state");
        assert!(!state.modules.contains_key("calculator"));
        let _ = std::fs::remove_dir_all(root);
    }

    fn write_sample_rpack(dir: &Path, name: &str, version: &str) {
        std::fs::create_dir_all(dir).expect("create rpack dir");
        std::fs::write(
            dir.join("module.toml"),
            format!(
                "name = \"{name}\"\nversion = \"{version}\"\napi_version = \"1\"\nkind = \"script\"\nentry = \"module.js\"\ncapabilities = [\"input-accessory\"]\n"
            ),
        )
        .expect("write module.toml");
        std::fs::write(
            dir.join("module.js"),
            "export default function createModule() { return {}; }\n",
        )
        .expect("write module.js");
        std::fs::write(dir.join("config.json"), "{\"shortcuts\":[]}\n").expect("write config");
        std::fs::write(dir.join("README.md"), "# shortcuts\n").expect("write readme");
    }

    fn rpack_files_for(dir: &Path) -> Vec<RmodsRegistryFile> {
        let mut files = Vec::new();
        collect_directory_files(dir, dir, &mut files).expect("collect files");
        files.sort_by(|left, right| left.0.cmp(&right.0));
        files
            .into_iter()
            .map(|(path, file_path)| RmodsRegistryFile {
                path,
                sha256: sha256_file(&file_path).expect("hash file"),
                size: file_path.metadata().expect("metadata").len(),
            })
            .collect()
    }

    fn sample_rmod(name: &str, version: &str) -> String {
        format!(
            "#!rmod/v1\nname: {name}\nversion: {version}\napi_version: 1\nkind: script\ncapabilities: providers\n\n---module.js---\nexport default function createModule() {{ return {{ provideItems() {{ return []; }} }}; }}\n"
        )
    }

    fn sample_item(id: &str) -> RmodsRegistryItem {
        RmodsRegistryItem {
            id: id.to_string(),
            name: id.to_string(),
            version: "0.1.0".to_string(),
            description: String::new(),
            kind: "rmod".to_string(),
            download_url: format!("https://example.test/{id}.rmod"),
            base_url: String::new(),
            sha256: "de00cc81828884f32688d344099e8fb2553887d7d30fc652d0b3b1e0f5c7f227".to_string(),
            size: 1,
            files: Vec::new(),
            tags: Vec::new(),
            requires_rmenu: None,
        }
    }
}
