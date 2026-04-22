use crate::app_state::{LauncherItem, LauncherSource};
use crate::settings::LauncherConfig;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

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
        LauncherSource::StartMenu => "start",
        LauncherSource::Path => "path",
        LauncherSource::History => "history",
        LauncherSource::Direct => "direct",
    }
}

fn source_from_cache(value: &str) -> Option<LauncherSource> {
    match value {
        "start" => Some(LauncherSource::StartMenu),
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

fn launcher_label_from_target(target: &str) -> String {
    let path = Path::new(target);
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

fn read_index_cache(config: &LauncherConfig, blacklist: &HashSet<String>) -> Option<Vec<LauncherItem>> {
    let path = index_cache_file_path()?;
    let raw = fs::read_to_string(path).ok()?;

    let mut items = Vec::new();
    for line in raw.lines() {
        let mut parts = line.splitn(3, '\t');
        let source_raw = parts.next()?;
        let label = parts.next()?.to_string();
        let target = parts.next()?.to_string();

        let source = source_from_cache(source_raw)?;

        if !config.enable_start_menu && matches!(source, LauncherSource::StartMenu) {
            continue;
        }
        if !config.enable_path && matches!(source, LauncherSource::Path) {
            continue;
        }
        if matches!(source, LauncherSource::Path) && is_blacklisted_command_name(&target, blacklist) {
            continue;
        }

        items.push(LauncherItem::new(label, target, source));
    }

    Some(items)
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

    let mut lines: Vec<String> = Vec::new();
    for item in items {
        if !matches!(item.source, LauncherSource::StartMenu | LauncherSource::Path) {
            continue;
        }

        let label = item.label.replace('\t', " ").replace('\n', " ");
        let target = item.target.replace('\t', " ").replace('\n', " ");
        lines.push(format!("{}\t{}\t{}", source_to_cache(item.source), label, target));
    }

    if let Err(e) = fs::write(&path, lines.join("\n")) {
        if !silent_mode {
            eprintln!("Error writing index cache '{}': {}", path.display(), e);
        }
    }
}

fn collect_start_menu_items(items: &mut Vec<LauncherItem>, seen_targets: &mut HashSet<String>) {
    let mut roots: Vec<PathBuf> = Vec::new();

    if let Ok(appdata) = std::env::var("APPDATA") {
        roots.push(Path::new(&appdata).join("Microsoft\\Windows\\Start Menu\\Programs"));
    }
    if let Ok(program_data) = std::env::var("ProgramData") {
        roots.push(Path::new(&program_data).join("Microsoft\\Windows\\Start Menu\\Programs"));
    }

    for root in roots {
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

pub fn load_launcher_items(config: &LauncherConfig, silent_mode: bool) -> Vec<LauncherItem> {
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

    let indexed_items = read_index_cache(config, &blacklist).unwrap_or_else(|| {
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
    use super::{build_blacklist_set, is_blacklisted_command_name};
    use crate::settings::RmenuConfig;

    #[test]
    fn blacklist_detects_plain_and_path_variants() {
        let cfg = RmenuConfig::default();
        let blacklist = build_blacklist_set(&cfg.launcher);

        assert!(is_blacklisted_command_name("powercfg", &blacklist));
        assert!(is_blacklisted_command_name("C:/Windows/System32/powercfg.exe", &blacklist));
        assert!(!is_blacklisted_command_name("powershell", &blacklist));
    }
}
