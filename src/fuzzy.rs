fn normalize_for_match(value: &str, case_sensitive: bool) -> String {
    if case_sensitive {
        value.to_string()
    } else {
        value.to_lowercase()
    }
}

fn compact_alnum(value: &str, case_sensitive: bool) -> String {
    if case_sensitive {
        value
            .chars()
            .filter(|ch| ch.is_ascii_alphanumeric())
            .collect()
    } else {
        value
            .chars()
            .filter(|ch| ch.is_ascii_alphanumeric())
            .flat_map(char::to_lowercase)
            .collect()
    }
}

pub fn compact_lower_alnum(value: &str) -> String {
    compact_alnum(value, false)
}

fn length_penalty(candidate_len: usize, query_len: usize, cap: i64) -> i64 {
    ((candidate_len.saturating_sub(query_len)) as i64).min(cap)
}

fn token_prefix_score(candidate: &str, query: &str) -> Option<i64> {
    let mut token_index: i64 = 0;

    for token in candidate
        .split(|ch: char| !ch.is_ascii_alphanumeric())
        .filter(|token| !token.is_empty())
    {
        if token.starts_with(query) {
            let index_penalty = token_index * 18;
            let token_len_penalty = length_penalty(token.len(), query.len(), 70);
            return Some(2300 - index_penalty - token_len_penalty);
        }
        token_index += 1;
    }

    None
}

fn is_subsequence(query: &str, candidate: &str) -> bool {
    let mut candidate_chars = candidate.chars();

    for q in query.chars() {
        loop {
            let next = candidate_chars.next();
            match next {
                Some(c) if c == q => break,
                Some(_) => continue,
                None => return false,
            }
        }
    }

    true
}

fn fuzzy_score_core(
    query_norm: &str,
    query_compact: &str,
    candidate_norm: &str,
    candidate_compact: &str,
) -> i64 {
    if query_norm.is_empty() || candidate_norm.is_empty() {
        return 0;
    }

    if query_norm == candidate_norm {
        return 3200;
    }

    if !query_compact.is_empty() && query_compact == candidate_compact {
        return 3000;
    }

    if candidate_norm.starts_with(query_norm) {
        return 2600 - length_penalty(candidate_norm.len(), query_norm.len(), 200);
    }

    if !query_compact.is_empty() && candidate_compact.starts_with(query_compact) {
        return 2500 - length_penalty(candidate_compact.len(), query_compact.len(), 180);
    }

    if let Some(score) = token_prefix_score(candidate_norm, query_norm) {
        return score;
    }

    if candidate_norm.contains(query_norm) {
        let start_index = candidate_norm.find(query_norm).unwrap_or(usize::MAX);
        let position_penalty = (start_index as i64).min(160);
        let len_penalty = length_penalty(candidate_norm.len(), query_norm.len(), 120);
        return 1400 - position_penalty - len_penalty;
    }

    if !query_compact.is_empty() && candidate_compact.contains(query_compact) {
        let start_index = candidate_compact.find(query_compact).unwrap_or(usize::MAX);
        let position_penalty = (start_index as i64).min(140);
        let len_penalty = length_penalty(candidate_compact.len(), query_compact.len(), 110);
        return 1250 - position_penalty - len_penalty;
    }

    if is_subsequence(query_norm, candidate_norm) {
        let len_penalty = length_penalty(candidate_norm.len(), query_norm.len(), 180);
        return 700 - len_penalty;
    }

    if !query_compact.is_empty() && is_subsequence(query_compact, candidate_compact) {
        let len_penalty = length_penalty(candidate_compact.len(), query_compact.len(), 170);
        return 650 - len_penalty;
    }

    0
}

pub fn fuzzy_score_precomputed_lower(
    query_norm: &str,
    query_compact: &str,
    candidate_norm: &str,
    candidate_compact: &str,
) -> i64 {
    fuzzy_score_core(query_norm, query_compact, candidate_norm, candidate_compact)
}

pub fn fuzzy_score(query: &str, candidate: &str, case_sensitive: bool) -> i64 {
    let query_norm = normalize_for_match(query, case_sensitive);
    let candidate_norm = normalize_for_match(candidate, case_sensitive);

    let query_compact = compact_alnum(&query_norm, true);
    let candidate_compact = compact_alnum(&candidate_norm, true);

    fuzzy_score_core(
        &query_norm,
        &query_compact,
        &candidate_norm,
        &candidate_compact,
    )
}

#[cfg(test)]
mod tests {
    use super::fuzzy_score;

    #[test]
    fn exact_scores_higher_than_contains() {
        let exact = fuzzy_score("notepad", "notepad", false);
        let contains = fuzzy_score("pad", "notepad", false);
        assert!(exact > contains);
    }

    #[test]
    fn compact_matching_handles_separators() {
        let score = fuzzy_score("7zip", "7-zip file manager", false);
        assert!(score >= 2400);
    }

    #[test]
    fn subsequence_match_is_supported() {
        assert!(fuzzy_score("npd", "notepad", false) > 0);
        assert_eq!(fuzzy_score("zzz", "notepad", false), 0);
    }

    #[test]
    fn token_prefix_beats_generic_contains() {
        let token_prefix = fuzzy_score("power", "windows powershell", false);
        let contains = fuzzy_score("power", "x_superpower_tool", false);
        assert!(token_prefix > contains);
    }
}
