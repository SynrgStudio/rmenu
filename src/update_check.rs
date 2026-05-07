use std::cmp::Ordering;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::settings::rmenu_data_dirs;

pub const DEFAULT_LATEST_RELEASE_URL: &str =
    "https://api.github.com/repos/SynrgStudio/rmenu/releases/latest";
const CHECKSUMS_ASSET_NAME: &str = "SHA256SUMS.txt";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpdateCheckError {
    Io(String),
    Json(String),
    InvalidRelease(String),
    Fetch(String),
}

impl UpdateCheckError {
    pub fn message(&self) -> String {
        match self {
            Self::Io(message)
            | Self::Json(message)
            | Self::InvalidRelease(message)
            | Self::Fetch(message) => message.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdatesCache {
    pub last_checked: String,
    pub latest_version: String,
    pub release_url: String,
    pub installer_asset_url: Option<String>,
    pub checksums_asset_url: Option<String>,
    pub portable_zip_asset_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LatestReleaseMetadata {
    pub tag_name: String,
    pub version: String,
    pub release_url: String,
    pub installer_asset_url: Option<String>,
    pub checksums_asset_url: Option<String>,
    pub portable_zip_asset_url: Option<String>,
    pub checked_at: String,
}

impl LatestReleaseMetadata {
    pub fn into_cache(self) -> UpdatesCache {
        UpdatesCache {
            last_checked: self.checked_at,
            latest_version: self.version,
            release_url: self.release_url,
            installer_asset_url: self.installer_asset_url,
            checksums_asset_url: self.checksums_asset_url,
            portable_zip_asset_url: self.portable_zip_asset_url,
        }
    }
}

#[derive(Debug, Deserialize)]
struct GithubReleaseResponse {
    tag_name: String,
    html_url: String,
    #[serde(default)]
    assets: Vec<GithubReleaseAsset>,
}

#[derive(Debug, Deserialize)]
struct GithubReleaseAsset {
    name: String,
    browser_download_url: String,
}

pub fn updates_cache_path(cli_data_dir: Option<&str>) -> PathBuf {
    rmenu_data_dirs(cli_data_dir).state_dir.join("updates.json")
}

pub fn read_updates_cache(cli_data_dir: Option<&str>) -> Result<UpdatesCache, UpdateCheckError> {
    let path = updates_cache_path(cli_data_dir);
    let content = fs::read_to_string(&path).map_err(io_error)?;
    serde_json::from_str(&content).map_err(|error| UpdateCheckError::Json(error.to_string()))
}

pub fn write_updates_cache(
    cli_data_dir: Option<&str>,
    cache: &UpdatesCache,
) -> Result<PathBuf, UpdateCheckError> {
    let path = updates_cache_path(cli_data_dir);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(io_error)?;
    }
    let content = serde_json::to_string_pretty(cache)
        .map_err(|error| UpdateCheckError::Json(error.to_string()))?;
    fs::write(&path, format!("{content}\n")).map_err(io_error)?;
    Ok(path)
}

pub fn fetch_latest_release(
    url: &str,
    cli_data_dir: Option<&str>,
) -> Result<LatestReleaseMetadata, UpdateCheckError> {
    let content = fetch_text(url, cli_data_dir)?;
    let metadata = parse_latest_release_json(&content)?;
    write_updates_cache(cli_data_dir, &metadata.clone().into_cache())?;
    Ok(metadata)
}

pub fn parse_latest_release_json(content: &str) -> Result<LatestReleaseMetadata, UpdateCheckError> {
    let release: GithubReleaseResponse =
        serde_json::from_str(content).map_err(|error| UpdateCheckError::Json(error.to_string()))?;
    let version = normalize_tag_version(&release.tag_name).ok_or_else(|| {
        UpdateCheckError::InvalidRelease(format!("invalid release tag: {}", release.tag_name))
    })?;
    if release.html_url.trim().is_empty() {
        return Err(UpdateCheckError::InvalidRelease(
            "release html_url is empty".to_string(),
        ));
    }

    Ok(LatestReleaseMetadata {
        tag_name: release.tag_name,
        installer_asset_url: find_installer_asset(&release.assets, &version),
        checksums_asset_url: find_asset_exact(&release.assets, CHECKSUMS_ASSET_NAME),
        portable_zip_asset_url: find_portable_zip_asset(&release.assets, &version),
        release_url: release.html_url,
        version,
        checked_at: current_timestamp_string(),
    })
}

pub fn is_newer_version(latest: &str, current: &str) -> bool {
    compare_versions(latest, current) == Ordering::Greater
}

fn normalize_tag_version(tag: &str) -> Option<String> {
    let trimmed = tag.trim().trim_start_matches('v').trim_start_matches('V');
    if parse_numeric_version(trimmed).is_some() {
        Some(trimmed.to_string())
    } else {
        None
    }
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
    let trimmed = value.trim().trim_start_matches('v').trim_start_matches('V');
    if trimmed.is_empty() {
        return None;
    }
    let version = trimmed.split(['-', '+']).next()?;
    let mut parts = Vec::new();
    for part in version.split('.') {
        parts.push(part.parse::<u64>().ok()?);
    }
    Some(parts)
}

fn find_asset_exact(assets: &[GithubReleaseAsset], name: &str) -> Option<String> {
    assets
        .iter()
        .find(|asset| asset.name.eq_ignore_ascii_case(name))
        .map(|asset| asset.browser_download_url.clone())
}

fn find_installer_asset(assets: &[GithubReleaseAsset], version: &str) -> Option<String> {
    let expected = format!("rmenu-setup-v{version}.exe");
    find_asset_exact(assets, &expected).or_else(|| {
        assets
            .iter()
            .find(|asset| {
                let name = asset.name.to_ascii_lowercase();
                name.starts_with("rmenu-setup-v") && name.ends_with(".exe")
            })
            .map(|asset| asset.browser_download_url.clone())
    })
}

fn find_portable_zip_asset(assets: &[GithubReleaseAsset], version: &str) -> Option<String> {
    let expected = format!("rmenu-v{version}-windows-x64.zip");
    find_asset_exact(assets, &expected).or_else(|| {
        assets
            .iter()
            .find(|asset| {
                let name = asset.name.to_ascii_lowercase();
                name.starts_with("rmenu-v") && name.ends_with("-windows-x64.zip")
            })
            .map(|asset| asset.browser_download_url.clone())
    })
}

fn fetch_text(url: &str, cli_data_dir: Option<&str>) -> Result<String, UpdateCheckError> {
    if url.trim().is_empty() {
        return Err(UpdateCheckError::Fetch("empty URL".to_string()));
    }
    if let Some(path) = url.strip_prefix("file://") {
        return fs::read_to_string(path).map_err(io_error);
    }
    fetch_http_text(url, cli_data_dir)
}

#[cfg(windows)]
fn fetch_http_text(url: &str, cli_data_dir: Option<&str>) -> Result<String, UpdateCheckError> {
    use windows::core::PCWSTR;
    use windows::Win32::System::Com::Urlmon::URLDownloadToFileW;

    if !url.starts_with("https://") && !url.starts_with("http://") {
        return Err(UpdateCheckError::Fetch(format!(
            "unsupported URL scheme: {url}"
        )));
    }

    let downloads_dir = rmenu_data_dirs(cli_data_dir)
        .state_dir
        .join("updates")
        .join("downloads");
    fs::create_dir_all(&downloads_dir).map_err(io_error)?;
    let path = downloads_dir.join("latest-release.tmp");
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
    .map_err(|error| UpdateCheckError::Fetch(format!("download failed: {error}")))?;
    let content = fs::read_to_string(&path).map_err(io_error)?;
    let _ = fs::remove_file(path);
    Ok(content)
}

#[cfg(not(windows))]
fn fetch_http_text(_url: &str, _cli_data_dir: Option<&str>) -> Result<String, UpdateCheckError> {
    Err(UpdateCheckError::Fetch(
        "HTTP update checks are only implemented on Windows".to_string(),
    ))
}

#[cfg(windows)]
fn wide_null(value: &str) -> Vec<u16> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    OsStr::new(value).encode_wide().chain(Some(0)).collect()
}

fn current_timestamp_string() -> String {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => format!("unix:{}", duration.as_secs()),
        Err(_) => "unix:0".to_string(),
    }
}

fn io_error(error: std::io::Error) -> UpdateCheckError {
    UpdateCheckError::Io(error.to_string())
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::{
        fetch_latest_release, is_newer_version, parse_latest_release_json, read_updates_cache,
        updates_cache_path, write_updates_cache, UpdatesCache,
    };

    const RELEASE_JSON: &str = r#"
{
  "tag_name": "v0.3.1",
  "html_url": "https://github.com/SynrgStudio/rmenu/releases/tag/v0.3.1",
  "assets": [
    {
      "name": "rmenu-v0.3.1-windows-x64.zip",
      "browser_download_url": "https://github.com/SynrgStudio/rmenu/releases/download/v0.3.1/rmenu-v0.3.1-windows-x64.zip"
    },
    {
      "name": "rmenu-setup-v0.3.1.exe",
      "browser_download_url": "https://github.com/SynrgStudio/rmenu/releases/download/v0.3.1/rmenu-setup-v0.3.1.exe"
    },
    {
      "name": "SHA256SUMS.txt",
      "browser_download_url": "https://github.com/SynrgStudio/rmenu/releases/download/v0.3.1/SHA256SUMS.txt"
    }
  ]
}
"#;

    #[test]
    fn parses_latest_release_assets() {
        let metadata = parse_latest_release_json(RELEASE_JSON).expect("release metadata");

        assert_eq!(metadata.tag_name, "v0.3.1");
        assert_eq!(metadata.version, "0.3.1");
        assert_eq!(
            metadata.release_url,
            "https://github.com/SynrgStudio/rmenu/releases/tag/v0.3.1"
        );
        assert_eq!(
            metadata.installer_asset_url.as_deref(),
            Some("https://github.com/SynrgStudio/rmenu/releases/download/v0.3.1/rmenu-setup-v0.3.1.exe")
        );
        assert_eq!(
            metadata.checksums_asset_url.as_deref(),
            Some("https://github.com/SynrgStudio/rmenu/releases/download/v0.3.1/SHA256SUMS.txt")
        );
    }

    #[test]
    fn compares_versions_numerically() {
        assert!(is_newer_version("0.3.10", "0.3.2"));
        assert!(is_newer_version("v0.4.0", "0.3.99"));
        assert!(!is_newer_version("0.3.0", "0.3.0"));
        assert!(!is_newer_version("0.2.9", "0.3.0"));
    }

    #[test]
    fn updates_cache_roundtrips_under_data_root() {
        let root =
            std::env::temp_dir().join(format!("rmenu-updates-cache-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let root_string = root.to_string_lossy().to_string();
        let cache = UpdatesCache {
            last_checked: "unix:1".to_string(),
            latest_version: "0.3.1".to_string(),
            release_url: "https://github.com/SynrgStudio/rmenu/releases/tag/v0.3.1".to_string(),
            installer_asset_url: Some("https://example.test/installer.exe".to_string()),
            checksums_asset_url: Some("https://example.test/SHA256SUMS.txt".to_string()),
            portable_zip_asset_url: Some("https://example.test/rmenu.zip".to_string()),
        };

        let path = write_updates_cache(Some(&root_string), &cache).expect("write cache");
        assert_eq!(path, root.join("state").join("updates.json"));
        assert_eq!(updates_cache_path(Some(&root_string)), path);
        let loaded = read_updates_cache(Some(&root_string)).expect("read cache");
        assert_eq!(loaded, cache);

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn fetch_latest_release_supports_file_url_for_tests() {
        let root =
            std::env::temp_dir().join(format!("rmenu-updates-fetch-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).expect("create temp root");
        let release_path = root.join("latest.json");
        fs::write(&release_path, RELEASE_JSON).expect("write fixture");
        let root_string = root.to_string_lossy().to_string();
        let url = format!("file://{}", release_path.to_string_lossy());

        let metadata = fetch_latest_release(&url, Some(&root_string)).expect("fetch fixture");
        assert_eq!(metadata.version, "0.3.1");
        let cached = read_updates_cache(Some(&root_string)).expect("read written cache");
        assert_eq!(cached.latest_version, "0.3.1");

        let _ = fs::remove_dir_all(root);
    }
}
