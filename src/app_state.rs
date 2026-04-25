#[derive(Debug, Default, Clone, Copy)]
pub enum LauncherSource {
    #[default]
    Direct,
    History,
    StartMenu,
    Path,
}

#[derive(Debug, Default, Clone)]
pub struct LauncherItem {
    pub label: String,
    pub label_lc: String,
    pub label_compact: String,
    pub target_name: String,
    pub target_name_lc: String,
    pub target_name_compact: String,
    pub target: String,
    pub source: LauncherSource,
    pub trailing_hint: Option<String>,
    pub quick_select_key: Option<String>,
    pub trailing_badge: Option<String>,
}

impl LauncherItem {
    pub fn new(label: String, target: String, source: LauncherSource) -> Self {
        let label_lc = label.to_lowercase();
        let label_compact = label_lc
            .chars()
            .filter(|ch| ch.is_ascii_alphanumeric())
            .collect::<String>();

        let target_name = std::path::Path::new(&target)
            .file_stem()
            .and_then(|value| value.to_str())
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or(&target)
            .to_string();
        let target_name_lc = target_name.to_lowercase();
        let target_name_compact = target_name_lc
            .chars()
            .filter(|ch| ch.is_ascii_alphanumeric())
            .collect::<String>();

        Self {
            label,
            label_lc,
            label_compact,
            target_name,
            target_name_lc,
            target_name_compact,
            target,
            source,
            trailing_hint: None,
            quick_select_key: None,
            trailing_badge: None,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct AppState {
    pub current_input: String,
    pub selected_index: usize,
    pub scroll_offset: usize,
    pub matching_items: Vec<LauncherItem>,
    pub all_items: Vec<LauncherItem>,
    pub prompt: Option<String>,
    pub launcher_mode: bool,
    pub silent_mode: bool,
    pub history_max_items: usize,
    pub source_boost_history: i64,
    pub source_boost_start_menu: i64,
    pub source_boost_path: i64,
}

pub fn ensure_selection_visible(app_state: &mut AppState, max_visible_items: usize) {
    if app_state.matching_items.is_empty() {
        app_state.selected_index = 0;
        app_state.scroll_offset = 0;
        return;
    }

    if app_state.selected_index >= app_state.matching_items.len() {
        app_state.selected_index = app_state.matching_items.len() - 1;
    }

    if max_visible_items == 0 {
        app_state.scroll_offset = 0;
        return;
    }

    if app_state.selected_index < app_state.scroll_offset {
        app_state.scroll_offset = app_state.selected_index;
    }

    let window_end = app_state.scroll_offset + max_visible_items;
    if app_state.selected_index >= window_end {
        app_state.scroll_offset = app_state.selected_index + 1 - max_visible_items;
    }
}

pub fn source_boost(app_state: &AppState, source: LauncherSource) -> i64 {
    match source {
        LauncherSource::History => app_state.source_boost_history,
        LauncherSource::StartMenu => app_state.source_boost_start_menu,
        LauncherSource::Path => app_state.source_boost_path,
        LauncherSource::Direct => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::{ensure_selection_visible, source_boost, AppState, LauncherItem, LauncherSource};
    use crate::fuzzy::fuzzy_score;

    #[test]
    fn selection_visibility_tracks_scroll_window() {
        let mut state = AppState {
            matching_items: (0..20)
                .map(|i| {
                    LauncherItem::new(
                        format!("item-{i}"),
                        format!("target-{i}"),
                        LauncherSource::Direct,
                    )
                })
                .collect(),
            selected_index: 15,
            scroll_offset: 0,
            ..Default::default()
        };

        ensure_selection_visible(&mut state, 10);
        assert_eq!(state.scroll_offset, 6);

        state.selected_index = 2;
        ensure_selection_visible(&mut state, 10);
        assert_eq!(state.scroll_offset, 2);
    }

    #[test]
    fn source_boost_can_prioritize_start_menu_over_path_noise() {
        let state = AppState {
            source_boost_start_menu: 480,
            source_boost_path: 0,
            ..Default::default()
        };

        let pow_shell_score = fuzzy_score("pow", "windows powershell", false)
            + source_boost(&state, LauncherSource::StartMenu);
        let powercfg_score =
            fuzzy_score("pow", "powercfg", false) + source_boost(&state, LauncherSource::Path);

        assert!(pow_shell_score > powercfg_score);
    }
}
