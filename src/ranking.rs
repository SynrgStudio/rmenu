use crate::app_state::{ensure_selection_visible, source_boost, AppState, LauncherItem, LauncherSource};
use crate::fuzzy::{compact_lower_alnum, fuzzy_score, fuzzy_score_precomputed_lower};

#[derive(Debug, Clone)]
pub struct RankedItem {
    pub item: LauncherItem,
    pub fuzzy_score: i64,
    pub source_boost: i64,
    pub total_score: i64,
}

const PARTIAL_TOPK_THRESHOLD: usize = 1200;
const PARTIAL_TOPK_LIMIT: usize = 400;
const STRONG_LABEL_MATCH_SCORE: i64 = 2300;

fn rank_compare_desc(a: &RankedItem, b: &RankedItem) -> std::cmp::Ordering {
    b.total_score
        .cmp(&a.total_score)
        .then_with(|| a.item.label.cmp(&b.item.label))
}

pub fn source_name(source: LauncherSource) -> &'static str {
    match source {
        LauncherSource::Direct => "direct",
        LauncherSource::History => "history",
        LauncherSource::StartMenu => "start_menu",
        LauncherSource::Path => "path",
    }
}

fn fuzzy_best_score(item: &LauncherItem, query: &str, query_norm: &str, query_compact: &str, case_sensitive: bool) -> i64 {
    if case_sensitive {
        let label_score = fuzzy_score(query, &item.label, true);
        if label_score >= STRONG_LABEL_MATCH_SCORE {
            return label_score;
        }
        let target_score = fuzzy_score(query, &item.target_name, true);
        return label_score.max(target_score);
    }

    let label_score = fuzzy_score_precomputed_lower(
        query_norm,
        query_compact,
        &item.label_lc,
        &item.label_compact,
    );
    if label_score >= STRONG_LABEL_MATCH_SCORE {
        return label_score;
    }

    let target_score = fuzzy_score_precomputed_lower(
        query_norm,
        query_compact,
        &item.target_name_lc,
        &item.target_name_compact,
    );
    label_score.max(target_score)
}

pub fn rank_items(app_state: &AppState, query: &str, case_sensitive: bool) -> Vec<RankedItem> {
    let query_norm = if case_sensitive {
        String::new()
    } else {
        query.to_lowercase()
    };
    let query_compact = if case_sensitive {
        String::new()
    } else {
        compact_lower_alnum(&query_norm)
    };

    let mut ranked: Vec<RankedItem> = app_state
        .all_items
        .iter()
        .filter_map(|item| {
            let fuzzy = fuzzy_best_score(item, query, &query_norm, &query_compact, case_sensitive);

            if fuzzy <= 0 {
                return None;
            }
            let boost = source_boost(app_state, item.source);
            let total = fuzzy + boost;
            Some(RankedItem {
                item: item.clone(),
                fuzzy_score: fuzzy,
                source_boost: boost,
                total_score: total,
            })
        })
        .collect();

    if ranked.len() > PARTIAL_TOPK_THRESHOLD {
        let keep = PARTIAL_TOPK_LIMIT.min(ranked.len());
        ranked.select_nth_unstable_by(keep - 1, rank_compare_desc);
        ranked.truncate(keep);
    }

    ranked.sort_unstable_by(rank_compare_desc);
    ranked
}

pub fn update_matching_items(app_state: &mut AppState, case_sensitive: bool, max_visible_items: usize) {
    if app_state.current_input.is_empty() {
        app_state.matching_items = app_state.all_items.clone();
        ensure_selection_visible(app_state, max_visible_items);
        return;
    }

    let ranked = rank_items(app_state, &app_state.current_input, case_sensitive);
    app_state.matching_items = ranked.into_iter().map(|entry| entry.item).collect();
    ensure_selection_visible(app_state, max_visible_items);
}

#[cfg(test)]
mod tests {
    use super::rank_items;
    use crate::app_state::{AppState, LauncherItem, LauncherSource};

    #[test]
    fn ranking_keeps_executable_name_matching_when_label_is_friendly() {
        let state = AppState {
            all_items: vec![LauncherItem::new(
                "Paint".to_string(),
                "C:/Users/test/AppData/Local/Microsoft/WindowsApps/mspaint.exe".to_string(),
                LauncherSource::Path,
            )],
            source_boost_path: 0,
            ..Default::default()
        };

        let ranked = rank_items(&state, "mspaint", false);
        assert_eq!(ranked.len(), 1);
        assert_eq!(ranked[0].item.label, "Paint");
    }
}
