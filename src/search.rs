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

/// 최대 검색 단어 수.
const MAX_TERMS: usize = 5;

/// 파싱된 멀티-워드 쿼리.
///
/// - `or`: `+` 또는 첫 단어(묵시적 `+`). 전체 목록에서 OR(합집합)로 후보 수집.
/// - `and`: 공백 구분 단어. OR 후보를 모두 매칭하는 것으로 좁힘.
/// - `exclude`: `-` 단어. 매칭되면 마지막에 제거.
#[derive(Default)]
struct Query {
    or: Vec<String>,
    and: Vec<String>,
    exclude: Vec<String>,
}

impl Query {
    fn is_empty(&self) -> bool {
        self.or.is_empty() && self.and.is_empty() && self.exclude.is_empty()
    }
}

/// 멀티-워드 쿼리 파싱.
///
/// 연산자(`+`/`-`)는 공백 구분 토큰의 맨 앞에서만, 혹은 독립 토큰으로만 인식한다.
/// 따라서 단어 중간의 `-`(예: `account-box`)는 리터럴로 남는다.
/// 첫 단어는 연산자가 없으면 묵시적으로 `+`(OR). 최대 MAX_TERMS 단어까지.
fn parse_query(raw: &str) -> Query {
    let mut q = Query::default();
    let mut pending: Option<char> = None; // 독립 토큰으로 들어온 연산자
    let mut first = true;
    let mut count = 0usize;

    for tok in raw.to_lowercase().split_whitespace() {
        if tok == "+" || tok == "-" {
            pending = tok.chars().next();
            continue;
        }
        let (op, word) = match pending.take() {
            Some(p) => (Some(p), tok),
            None => {
                if let Some(rest) = tok.strip_prefix('+') {
                    (Some('+'), rest)
                } else if let Some(rest) = tok.strip_prefix('-') {
                    (Some('-'), rest)
                } else {
                    (None, tok)
                }
            }
        };
        if word.is_empty() {
            continue;
        }
        if count >= MAX_TERMS {
            break;
        }
        count += 1;
        match op {
            Some('+') => q.or.push(word.to_string()),
            Some('-') => q.exclude.push(word.to_string()),
            _ if first => q.or.push(word.to_string()), // 첫 단어 묵시적 +
            _ => q.and.push(word.to_string()),
        }
        first = false;
    }
    q
}

/// 검색 실행. 점수 내림차순, 동점 시 이름 오름차순, limit 개 컷.
///
/// 처리 순서: OR(합집합) → AND(교집합으로 좁힘) → exclude(제거).
/// 각 아이콘을 한 번씩만 평가하므로 OR 단계의 중복은 자연히 제거된다.
pub fn search<'a>(meta: &'a Meta, raw_query: &str, limit: usize) -> Vec<&'a Icon> {
    let q = parse_query(raw_query);

    if q.is_empty() {
        let mut all: Vec<&Icon> = meta.icons.iter().collect();
        all.sort_by(|a, b| a.name.cmp(&b.name));
        all.truncate(limit);
        return all;
    }

    let mut scored: Vec<(i32, &Icon)> = meta
        .icons
        .iter()
        .filter_map(|icon| {
            // OR: 후보 수집. or 가 비면(예: exclude/and 만) 전체를 후보로(점수 0).
            let or_score = if q.or.is_empty() {
                0
            } else {
                let mut best = 0;
                let mut hit = false;
                for t in &q.or {
                    let s = icon_score(icon, t);
                    if s > 0 {
                        hit = true;
                        best = best.max(s);
                    }
                }
                if !hit {
                    return None;
                }
                best
            };
            // AND: 모든 단어가 매칭되어야 함.
            let mut and_score = 0;
            for t in &q.and {
                let s = icon_score(icon, t);
                if s <= 0 {
                    return None;
                }
                and_score += s;
            }
            // exclude: 하나라도 매칭되면 제거.
            for t in &q.exclude {
                if icon_score(icon, t) > 0 {
                    return None;
                }
            }
            Some((or_score + and_score, icon))
        })
        .collect();

    scored.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.name.cmp(&b.1.name)));
    scored.truncate(limit);
    scored.into_iter().map(|(_, i)| i).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn icon(name: &str, aliases: &[&str], tags: &[&str]) -> Icon {
        Icon {
            name: name.into(),
            aliases: aliases.iter().map(|s| s.to_string()).collect(),
            tags: tags.iter().map(|s| s.to_string()).collect(),
            version: String::new(),
            path: String::new(),
        }
    }

    fn meta() -> Meta {
        Meta {
            version: String::new(),
            icons: vec![
                icon("account", &["user"], &[]),
                icon("account-box", &["user"], &[]),
                icon("account-cog", &[], &[]),
                icon("box", &[], &[]),
                icon("home", &[], &[]),
            ],
        }
    }

    fn names(hits: Vec<&Icon>) -> Vec<String> {
        hits.iter().map(|i| i.name.clone()).collect()
    }

    #[test]
    fn or_dedups() {
        // "user + account": account 가 user(alias)+account(name) 양쪽 매칭이지만 1번만.
        let m = meta();
        let hits = names(search(&m, "user + account", 50));
        assert_eq!(hits.iter().filter(|n| *n == "account").count(), 1);
    }

    #[test]
    fn and_narrows() {
        // "account box": account* 중 box 도 매칭하는 것만.
        let m = meta();
        let hits = names(search(&m, "account box", 50));
        assert_eq!(hits, vec!["account-box"]);
    }

    #[test]
    fn exclude_removes() {
        // "account -box": account* 에서 box 매칭 제거.
        let m = meta();
        let hits = names(search(&m, "account -box", 50));
        assert!(hits.contains(&"account".to_string()));
        assert!(hits.contains(&"account-cog".to_string()));
        assert!(!hits.contains(&"account-box".to_string()));
    }

    #[test]
    fn combined_or_and_exclude() {
        // "account + home -box": (account* OR home) 에서 box 제거.
        let m = meta();
        let hits = names(search(&m, "account + home -box", 50));
        assert!(hits.contains(&"home".to_string()));
        assert!(hits.contains(&"account".to_string()));
        assert!(!hits.contains(&"account-box".to_string()));
    }

    #[test]
    fn hyphen_in_word_is_literal() {
        // "account-box" 는 리터럴(제외 아님).
        let m = meta();
        let hits = names(search(&m, "account-box", 50));
        assert_eq!(hits.first().map(String::as_str), Some("account-box"));
    }

    #[test]
    fn max_five_terms() {
        let q = parse_query("a b c d e f g");
        assert_eq!(q.or.len() + q.and.len() + q.exclude.len(), MAX_TERMS);
    }

    #[test]
    fn empty_returns_all_sorted() {
        let m = meta();
        let hits = names(search(&m, "   ", 50));
        assert_eq!(hits.len(), 5);
        assert_eq!(hits.first().map(String::as_str), Some("account"));
    }
}
