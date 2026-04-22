use crate::app_state::{LauncherItem, LauncherSource};
use crate::settings::LauncherConfig;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::ffi::c_void;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use windows::core::PCWSTR;
use windows::Win32::Storage::FileSystem::{
    GetFileVersionInfoSizeW, GetFileVersionInfoW, VerQueryValueW,
};

const INDEX_CACHE_VERSION: u32 = 5;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CacheFormat {
    JsonV3,
    LegacyTsv,
}

#[derive(Debug, Clone)]
struct CacheLoad {
    items: Vec<LauncherItem>,
    format: CacheFormat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct IndexCacheFile {
    version: u32,
    generated_at_unix_ms: u64,
    env_signature: CacheEnvironmentSignature,
    items: Vec<CachedLauncherItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CachedLauncherItem {
    source: String,
    label: String,
    target: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
struct CacheEnvironmentSignature {
    path_hash: u64,
    start_menu_user_mtime_ms: u64,
    start_menu_common_mtime_ms: u64,
}

fn history_file_path() -> Option<PathBuf> {
    dirs::config_dir().map(|mut path| {
        path.push("rmenu");
        path.push("history.txt");
        path
    })
}

fn index_cache_file_path() -> Option<PathBuf> {
    dirs::config_dir().map(|mut path| {
        path.push("rmenu");
        path.push("index.json");
        path
    })
}

pub fn index_cache_size_bytes() -> Option<u64> {
    let path = index_cache_file_path()?;
    fs::metadata(path).ok().map(|meta| meta.len())
}

fn source_to_cache(source: LauncherSource) -> &'static str {
    match source {
        LauncherSource::StartMenu => "start_menu",
        LauncherSource::Path => "path",
        LauncherSource::History => "history",
        LauncherSource::Direct => "direct",
    }
}

fn source_from_cache(value: &str) -> Option<LauncherSource> {
    match value {
        "start" | "start_menu" => Some(LauncherSource::StartMenu),
        "path" => Some(LauncherSource::Path),
        "history" => Some(LauncherSource::History),
        "direct" => Some(LauncherSource::Direct),
        _ => None,
    }
}

pub fn persist_history_entry(target: &str, silent_mode: bool, max_items: usize) {
    let Some(path) = history_file_path() else {
        return;
    };

    if let Some(parent) = path.parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            if !silent_mode {
                eprintln!("Error creating history directory '{}': {}", parent.display(), e);
            }
            return;
        }
    }

    let mut values = read_history_targets();
    values.retain(|item| !item.eq_ignore_ascii_case(target));
    values.insert(0, target.to_string());
    if values.len() > max_items {
        values.truncate(max_items);
    }

    if let Err(e) = fs::write(&path, values.join("\n")) {
        if !silent_mode {
            eprintln!("Error writing history file '{}': {}", path.display(), e);
        }
    }
}

fn read_history_targets() -> Vec<String> {
    let Some(path) = history_file_path() else {
        return Vec::new();
    };

    let raw = match fs::read_to_string(path) {
        Ok(raw) => raw,
        Err(_) => return Vec::new(),
    };

    raw.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_string)
        .collect()
}

fn to_wstring(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(std::iter::once(0)).collect()
}

fn read_version_info_string(block: &[u8], sub_block: &str) -> Option<String> {
    let sub_block_w = to_wstring(sub_block);
    let mut value_ptr: *mut c_void = std::ptr::null_mut();
    let mut value_len: u32 = 0;

    let ok = unsafe {
        VerQueryValueW(
            block.as_ptr() as *const c_void,
            PCWSTR(sub_block_w.as_ptr()),
            &mut value_ptr,
            &mut value_len,
        )
        .as_bool()
    };

    if !ok || value_ptr.is_null() || value_len <= 1 {
        return None;
    }

    let value_slice = unsafe {
        std::slice::from_raw_parts(value_ptr as *const u16, value_len.saturating_sub(1) as usize)
    };

    let value = String::from_utf16(value_slice).ok()?;
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }

    Some(trimmed.to_string())
}

fn read_exe_file_description(path: &Path) -> Option<String> {
    let ext = path.extension().and_then(|value| value.to_str())?;
    if !ext.eq_ignore_ascii_case("exe") {
        return None;
    }

    let path_w = to_wstring(path.to_string_lossy().as_ref());
    let mut handle: u32 = 0;
    let size = unsafe { GetFileVersionInfoSizeW(PCWSTR(path_w.as_ptr()), Some(&mut handle)) };
    if size == 0 {
        return None;
    }

    let mut block = vec![0u8; size as usize];
    let loaded = unsafe {
        GetFileVersionInfoW(
            PCWSTR(path_w.as_ptr()),
            0,
            size,
            block.as_mut_ptr() as *mut c_void,
        )
        .as_bool()
    };
    if !loaded {
        return None;
    }

    let mut translation_ptr: *mut c_void = std::ptr::null_mut();
    let mut translation_len: u32 = 0;
    let translation_key = to_wstring("\\VarFileInfo\\Translation");

    let has_translation = unsafe {
        VerQueryValueW(
            block.as_ptr() as *const c_void,
            PCWSTR(translation_key.as_ptr()),
            &mut translation_ptr,
            &mut translation_len,
        )
        .as_bool()
    };

    if has_translation && !translation_ptr.is_null() && translation_len >= 4 {
        let pair_count = (translation_len as usize) / 4;
        let lang_pairs = unsafe {
            std::slice::from_raw_parts(translation_ptr as *const u16, pair_count * 2)
        };

        for pair in lang_pairs.chunks_exact(2) {
            let lang = pair[0];
            let code_page = pair[1];
            let sub_block = format!("\\StringFileInfo\\{:04X}{:04X}\\FileDescription", lang, code_page);
            if let Some(value) = read_version_info_string(&block, &sub_block) {
                return Some(value);
            }
        }
    }

    read_version_info_string(&block, "\\StringFileInfo\\040904B0\\FileDescription")
        .or_else(|| read_version_info_string(&block, "\\StringFileInfo\\040904E4\\FileDescription"))
}

fn windowsapps_alias_display_label(path: &Path) -> Option<&'static str> {
    let full_path_lc = path
        .to_string_lossy()
        .to_ascii_lowercase()
        .replace('/', "\\");
    if !full_path_lc.contains("\\windowsapps\\") {
        return None;
    }

    let stem = path.file_stem().and_then(|value| value.to_str())?.to_ascii_lowercase();
    match stem.as_str() {
        "mspaint" => Some("Paint"),
        "msedge" => Some("Microsoft Edge"),
        "wt" => Some("Windows Terminal"),
        _ => None,
    }
}

fn launcher_label_from_target(target: &str) -> String {
    let path = Path::new(target);

    if let Some(description) = read_exe_file_description(path) {
        return description;
    }

    if let Some(alias_label) = windowsapps_alias_display_label(path) {
        return alias_label.to_string();
    }

    path.file_stem()
        .and_then(|value| value.to_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(target)
        .to_string()
}

fn collect_files_recursive(root: &Path, out: &mut Vec<PathBuf>) {
    let entries = match fs::read_dir(root) {
        Ok(entries) => entries,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_files_recursive(&path, out);
        } else {
            out.push(path);
        }
    }
}

fn is_launcher_extension(path: &Path) -> bool {
    let ext = path
        .extension()
        .and_then(|value| value.to_str())
        .map(str::to_ascii_lowercase);

    matches!(
        ext.as_deref(),
        Some("exe") | Some("lnk") | Some("cmd") | Some("bat")
    )
}

fn build_blacklist_set(config: &LauncherConfig) -> HashSet<String> {
    config
        .blacklist_path_commands
        .iter()
        .map(|value| value.trim().to_ascii_lowercase())
        .filter(|value| !value.is_empty())
        .collect()
}

fn is_blacklisted_command_name(value: &str, blacklist: &HashSet<String>) -> bool {
    let normalized = Path::new(value)
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or(value)
        .trim()
        .to_ascii_lowercase();

    blacklist.contains(&normalized)
}

fn is_blacklisted_cli_tool(path: &Path, blacklist: &HashSet<String>) -> bool {
    let Some(stem) = path.file_stem().and_then(|value| value.to_str()) else {
        return false;
    };
    blacklist.contains(&stem.to_ascii_lowercase())
}

fn cached_item_to_launcher(
    source_raw: &str,
    label: String,
    target: String,
    config: &LauncherConfig,
    blacklist: &HashSet<String>,
) -> Option<LauncherItem> {
    let source = source_from_cache(source_raw)?;

    if !config.enable_start_menu && matches!(source, LauncherSource::StartMenu) {
        return None;
    }
    if !config.enable_path && matches!(source, LauncherSource::Path) {
        return None;
    }
    if matches!(source, LauncherSource::Path) && is_blacklisted_command_name(&target, blacklist) {
        return None;
    }

    Some(LauncherItem::new(label, target, source))
}

fn parse_index_cache_json(raw: &str) -> Option<IndexCacheFile> {
    serde_json::from_str::<IndexCacheFile>(raw).ok()
}

fn parse_legacy_index_cache(
    raw: &str,
    config: &LauncherConfig,
    blacklist: &HashSet<String>,
) -> Option<Vec<LauncherItem>> {
    let mut items = Vec::new();

    for line in raw.lines() {
        let mut parts = line.splitn(3, '\t');
        let Some(source_raw) = parts.next() else {
            continue;
        };
        let Some(label) = parts.next() else {
            continue;
        };
        let Some(target) = parts.next() else {
            continue;
        };

        if let Some(launcher_item) = cached_item_to_launcher(
            source_raw,
            label.to_string(),
            target.to_string(),
            config,
            blacklist,
        ) {
            items.push(launcher_item);
        }
    }

    if items.is_empty() {
        return None;
    }

    Some(items)
}

fn map_cached_json_items(
    parsed: IndexCacheFile,
    config: &LauncherConfig,
    blacklist: &HashSet<String>,
) -> Vec<LauncherItem> {
    let mut items = Vec::new();
    for item in parsed.items {
        if let Some(launcher_item) =
            cached_item_to_launcher(&item.source, item.label, item.target, config, blacklist)
        {
            items.push(launcher_item);
        }
    }
    items
}

fn stable_fnv1a_64(raw: &str) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for b in raw.as_bytes() {
        hash ^= *b as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

fn start_menu_roots() -> [Option<PathBuf>; 2] {
    let user = std::env::var("APPDATA")
        .ok()
        .map(|appdata| Path::new(&appdata).join("Microsoft\\Windows\\Start Menu\\Programs"));
    let common = std::env::var("ProgramData")
        .ok()
        .map(|program_data| Path::new(&program_data).join("Microsoft\\Windows\\Start Menu\\Programs"));

    [user, common]
}

fn dir_mtime_unix_ms(path: Option<&Path>) -> u64 {
    let Some(path) = path else {
        return 0;
    };

    let modified = match fs::metadata(path).and_then(|meta| meta.modified()) {
        Ok(value) => value,
        Err(_) => return 0,
    };

    match modified.duration_since(UNIX_EPOCH) {
        Ok(duration) => duration.as_millis() as u64,
        Err(_) => 0,
    }
}

fn current_env_signature() -> CacheEnvironmentSignature {
    let path_hash = stable_fnv1a_64(&std::env::var("PATH").unwrap_or_default().to_ascii_lowercase());
    let [user_root, common_root] = start_menu_roots();

    CacheEnvironmentSignature {
        path_hash,
        start_menu_user_mtime_ms: dir_mtime_unix_ms(user_root.as_deref()),
        start_menu_common_mtime_ms: dir_mtime_unix_ms(common_root.as_deref()),
    }
}

fn read_index_cache(
    config: &LauncherConfig,
    blacklist: &HashSet<String>,
    force_reindex: bool,
) -> Option<CacheLoad> {
    let path = index_cache_file_path()?;
    let raw = fs::read_to_string(path).ok()?;

    if let Some(parsed) = parse_index_cache_json(&raw) {
        if parsed.version == INDEX_CACHE_VERSION {
            let current_signature = current_env_signature();
            if force_reindex || parsed.env_signature != current_signature {
                return None;
            }

            return Some(CacheLoad {
                items: map_cached_json_items(parsed, config, blacklist),
                format: CacheFormat::JsonV3,
            });
        }
    }

    if force_reindex {
        return None;
    }

    if let Some(items) = parse_legacy_index_cache(&raw, config, blacklist) {
        return Some(CacheLoad {
            items,
            format: CacheFormat::LegacyTsv,
        });
    }

    None
}

fn now_unix_ms() -> u64 {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => duration.as_millis() as u64,
        Err(_) => 0,
    }
}

fn write_index_cache(items: &[LauncherItem], silent_mode: bool) {
    let Some(path) = index_cache_file_path() else {
        return;
    };

    if let Some(parent) = path.parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            if !silent_mode {
                eprintln!("Error creating index cache directory '{}': {}", parent.display(), e);
            }
            return;
        }
    }

    let cache = IndexCacheFile {
        version: INDEX_CACHE_VERSION,
        generated_at_unix_ms: now_unix_ms(),
        env_signature: current_env_signature(),
        items: items
            .iter()
            .filter(|item| matches!(item.source, LauncherSource::StartMenu | LauncherSource::Path))
            .map(|item| CachedLauncherItem {
                source: source_to_cache(item.source).to_string(),
                label: item.label.clone(),
                target: item.target.clone(),
            })
            .collect(),
    };

    let serialized = match serde_json::to_string(&cache) {
        Ok(value) => value,
        Err(e) => {
            if !silent_mode {
                eprintln!("Error serializing index cache '{}': {}", path.display(), e);
            }
            return;
        }
    };

    if let Err(e) = fs::write(&path, serialized) {
        if !silent_mode {
            eprintln!("Error writing index cache '{}': {}", path.display(), e);
        }
    }
}

fn collect_start_menu_items(items: &mut Vec<LauncherItem>, seen_targets: &mut HashSet<String>) {
    let roots = start_menu_roots();

    for root in roots.into_iter().flatten() {
        if !root.exists() {
            continue;
        }

        let mut files = Vec::new();
        collect_files_recursive(&root, &mut files);

        for file in files {
            if !is_launcher_extension(&file) {
                continue;
            }

            let target = file.to_string_lossy().to_string();
            let dedupe = target.to_lowercase();
            if !seen_targets.insert(dedupe) {
                continue;
            }

            items.push(LauncherItem::new(
                launcher_label_from_target(&target),
                target,
                LauncherSource::StartMenu,
            ));
        }
    }
}

fn collect_path_items(
    items: &mut Vec<LauncherItem>,
    seen_targets: &mut HashSet<String>,
    blacklist: &HashSet<String>,
) {
    let path_env = std::env::var("PATH").unwrap_or_default();

    for dir in std::env::split_paths(&path_env) {
        let entries = match fs::read_dir(&dir) {
            Ok(entries) => entries,
            Err(_) => continue,
        };

        for entry in entries.flatten() {
            let file = entry.path();
            if !file.is_file() || !is_launcher_extension(&file) {
                continue;
            }
            if is_blacklisted_cli_tool(&file, blacklist) {
                continue;
            }

            let target = file.to_string_lossy().to_string();
            let dedupe = target.to_lowercase();
            if !seen_targets.insert(dedupe) {
                continue;
            }

            items.push(LauncherItem::new(
                launcher_label_from_target(&target),
                target,
                LauncherSource::Path,
            ));
        }
    }
}

pub fn load_launcher_items(config: &LauncherConfig, silent_mode: bool, force_reindex: bool) -> Vec<LauncherItem> {
    let mut items: Vec<LauncherItem> = Vec::new();
    let mut seen_targets: HashSet<String> = HashSet::new();
    let blacklist = build_blacklist_set(config);

    if config.enable_history {
        for target in read_history_targets().into_iter().take(config.history_max_items) {
            if is_blacklisted_command_name(&target, &blacklist) {
                continue;
            }

            let dedupe = target.to_lowercase();
            if !seen_targets.insert(dedupe) {
                continue;
            }

            items.push(LauncherItem::new(
                launcher_label_from_target(&target),
                target,
                LauncherSource::History,
            ));
        }
    }

    let indexed_items = read_index_cache(config, &blacklist, force_reindex)
        .map(|cache| {
            if cache.format == CacheFormat::LegacyTsv {
                write_index_cache(&cache.items, silent_mode);
            }
            cache.items
        })
        .unwrap_or_else(|| {
            let mut built: Vec<LauncherItem> = Vec::new();
            let mut built_seen: HashSet<String> = HashSet::new();

            if config.enable_start_menu {
                collect_start_menu_items(&mut built, &mut built_seen);
            }
            if config.enable_path {
                collect_path_items(&mut built, &mut built_seen, &blacklist);
            }

            write_index_cache(&built, silent_mode);
            built
        });

    for item in indexed_items {
        let dedupe = item.target.to_lowercase();
        if !seen_targets.insert(dedupe) {
            continue;
        }
        items.push(item);
    }

    items
}

#[cfg(test)]
mod tests {
    use super::{
        build_blacklist_set, is_blacklisted_command_name, parse_legacy_index_cache, source_from_cache,
        stable_fnv1a_64, windowsapps_alias_display_label,
    };
    use crate::settings::RmenuConfig;
    use std::path::Path;

    #[test]
    fn blacklist_detects_plain_and_path_variants() {
        let cfg = RmenuConfig::default();
        let blacklist = build_blacklist_set(&cfg.launcher);

        assert!(is_blacklisted_command_name("powercfg", &blacklist));
        assert!(is_blacklisted_command_name("C:/Windows/System32/powercfg.exe", &blacklist));
        assert!(!is_blacklisted_command_name("powershell", &blacklist));
    }

    #[test]
    fn source_from_cache_supports_legacy_and_json_names() {
        assert!(source_from_cache("start").is_some());
        assert!(source_from_cache("start_menu").is_some());
    }

    #[test]
    fn legacy_index_cache_is_parsed_for_migration() {
        let cfg = RmenuConfig::default();
        let blacklist = build_blacklist_set(&cfg.launcher);
        let raw = "start\tCode\tC:/Code.exe\npath\tpowercfg\tC:/Windows/System32/powercfg.exe\n";

        let parsed = parse_legacy_index_cache(raw, &cfg.launcher, &blacklist).expect("expected parsed items");
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].label, "Code");
    }

    #[test]
    fn stable_path_hash_is_deterministic() {
        let a = stable_fnv1a_64("c:/tools;c:/windows/system32");
        let b = stable_fnv1a_64("c:/tools;c:/windows/system32");
        let c = stable_fnv1a_64("c:/tools;c:/windows/system32;d:/extra");

        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn windowsapps_alias_can_use_friendly_label() {
        let path = Path::new("C:/Users/test/AppData/Local/Microsoft/WindowsApps/mspaint.exe");
        assert_eq!(windowsapps_alias_display_label(path), Some("Paint"));
    }
}
