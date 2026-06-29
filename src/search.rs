use crate::meta::{Icon, Meta};

/// 필드 가중치. name > alias > tag.
const W_NAME: i32 = 100;
const W_ALIAS: i32 = 0;
const W_TAG: i32 = -60;

/// 단일 haystack 매칭 점수. 0 이면 매칭 없음.
fn match_score(haystack: &str, query: &str) -> i32 {
    if haystack == query {
        return 1000;
    }
    if haystack.starts_with(query) {
        // 짧을수록 가점 (account 가 account-box 보다 우선).
        return 800 - (haystack.len() as i32 - query.len() as i32).min(700);
    }
    if haystack.contains(query) {
        return 500;
    }
    if let Some(gaps) = subsequence_gaps(haystack, query) {
        return (250 - gaps as i32).max(50);
    }
    0
}

/// query 가 haystack 의 부분수열이면 매칭 간 총 간격(gap)을 반환. 아니면 None.
fn subsequence_gaps(haystack: &str, query: &str) -> Option<usize> {
    let mut gaps = 0usize;
    let mut last = None::<usize>;
    let mut q = query.chars().peekable();
    let mut next = q.next()?;
    for (i, c) in haystack.chars().enumerate() {
        if c == next {
            if let Some(l) = last {
                gaps += i - l - 1;
            }
            last = Some(i);
            match q.next() {
                Some(n) => next = n,
                None => return Some(gaps),
            }
        }
    }
    None
}

/// 아이콘 1개의 종합 점수 (모든 haystack 중 최고).
fn icon_score(icon: &Icon, query: &str) -> i32 {
    let mut best = match_score(&icon.name, query) + W_NAME;
    // name 매칭이 0 이면 W_NAME 만 남으므로 보정.
    if match_score(&icon.name, query) == 0 {
        best = 0;
    }
    for a in &icon.aliases {
        let s = match_score(&a.to_lowercase(), query);
        if s > 0 {
            best = best.max(s + W_ALIAS);
        }
    }
    for t in &icon.tags {
        let s = match_score(&t.to_lowercase(), query);
        if s > 0 {
            best = best.max(s + W_TAG);
        }
    }
    best
}

/// 검색 실행. 점수 내림차순, 동점 시 이름 오름차순, limit 개 컷.
pub fn search<'a>(meta: &'a Meta, raw_query: &str, limit: usize) -> Vec<&'a Icon> {
    let query = raw_query.trim().to_lowercase();

    if query.is_empty() {
        let mut all: Vec<&Icon> = meta.icons.iter().collect();
        all.sort_by(|a, b| a.name.cmp(&b.name));
        all.truncate(limit);
        return all;
    }

    let mut scored: Vec<(i32, &Icon)> = meta
        .icons
        .iter()
        .filter_map(|i| {
            let s = icon_score(i, &query);
            (s > 0).then_some((s, i))
        })
        .collect();

    scored.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.name.cmp(&b.1.name)));
    scored.truncate(limit);
    scored.into_iter().map(|(_, i)| i).collect()
}
