use anyhow::{Context, Result};
use flate2::read::GzDecoder;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Read;
use std::path::Path;
use tar::Archive;

use crate::config::Config;
use crate::meta::{Icon, Meta, UpstreamIcon};
use crate::paths::Paths;
use crate::{notify, npm, render};

#[derive(Serialize, Deserialize, Default)]
struct State {
    version: String,
}

/// tarball 에서 추출한 원본 데이터.
struct Upstream {
    icons: Vec<UpstreamIcon>,
    svgs: HashMap<String, String>,
}

pub fn run(paths: &Paths, cfg: &Config) -> Result<()> {
    let data = paths
        .data_dir()
        .context("alfred_workflow_data 가 설정되지 않음 (Alfred 환경 밖)")?
        .to_path_buf();
    std::fs::create_dir_all(data.join("icons"))?;

    let current = current_version(paths);
    let (latest, tarball_url) = npm::latest()?;

    if !is_newer(&latest, &current) {
        notify::post("MDI", &format!("이미 최신입니다 (v{current})"));
        return Ok(());
    }

    let bytes = npm::download(&tarball_url)?;
    let upstream = extract_tarball(&bytes)?;

    let icons_dir = data.join("icons");
    let mut icons = Vec::with_capacity(upstream.icons.len());
    let mut new_count = 0usize;

    for u in &upstream.icons {
        let d = upstream
            .svgs
            .get(&u.name)
            .and_then(|s| render::extract_path_d(s))
            .unwrap_or_default();

        let png = icons_dir.join(format!("{}.png", u.name));
        let is_new = is_newer(&u.version, &current);
        // 신규 아이콘 또는 overlay 에 아직 PNG 가 없는 경우에만 굽는다.
        if is_new || !png.exists() {
            render::render_png(&d, &cfg.color, cfg.size, &png)?;
            if is_new {
                new_count += 1;
            }
        }

        icons.push(Icon {
            name: u.name.clone(),
            aliases: u.aliases.clone(),
            tags: u.tags.clone(),
            version: u.version.clone(),
            path: d,
        });
    }

    let meta = Meta {
        version: latest.clone(),
        icons,
    };

    prune_orphans(&icons_dir, &meta)?;

    std::fs::write(data.join("meta.json"), serde_json::to_vec(&meta)?)?;
    if let Some(state_path) = paths.state() {
        std::fs::write(
            state_path,
            serde_json::to_vec(&State {
                version: latest.clone(),
            })?,
        )?;
    }

    notify::post("MDI", &format!("v{latest} 로 갱신 (+{new_count} 신규)"));
    Ok(())
}

/// 현재 빌드된 버전 = state.json → overlay meta → bundle meta 순.
fn current_version(paths: &Paths) -> String {
    if let Some(sp) = paths.state() {
        if let Ok(s) = std::fs::read_to_string(&sp) {
            if let Ok(st) = serde_json::from_str::<State>(&s) {
                if !st.version.is_empty() {
                    return st.version;
                }
            }
        }
    }
    for candidate in [paths.meta_overlay(), Some(paths.meta_bundle())]
        .into_iter()
        .flatten()
    {
        if let Ok(m) = Meta::read_file(&candidate) {
            if !m.version.is_empty() {
                return m.version;
            }
        }
    }
    "0.0.0".to_string()
}

/// tarball(gzip tar) 에서 package/meta.json 과 package/svg/*.svg 추출.
fn extract_tarball(bytes: &[u8]) -> Result<Upstream> {
    let mut archive = Archive::new(GzDecoder::new(bytes));
    let mut meta_json: Option<String> = None;
    let mut svgs = HashMap::new();

    for entry in archive.entries()? {
        let mut entry = entry?;
        let path = entry.path()?.to_path_buf();
        let rel = path.strip_prefix("package").unwrap_or(&path);

        if rel == Path::new("meta.json") {
            let mut s = String::new();
            entry.read_to_string(&mut s)?;
            meta_json = Some(s);
        } else if rel.extension().and_then(|e| e.to_str()) == Some("svg")
            && rel.starts_with("svg")
        {
            if let Some(stem) = rel.file_stem().and_then(|s| s.to_str()) {
                let mut s = String::new();
                entry.read_to_string(&mut s)?;
                svgs.insert(stem.to_string(), s);
            }
        }
    }

    let meta_json = meta_json.context("tarball 에 meta.json 이 없음")?;
    let icons: Vec<UpstreamIcon> =
        serde_json::from_str(&meta_json).context("원본 meta.json 파싱 실패")?;
    Ok(Upstream { icons, svgs })
}

/// 현재 meta 에 없는 overlay PNG 삭제.
fn prune_orphans(icons_dir: &Path, meta: &Meta) -> Result<()> {
    let keep: std::collections::HashSet<&str> =
        meta.icons.iter().map(|i| i.name.as_str()).collect();
    let Ok(rd) = std::fs::read_dir(icons_dir) else {
        return Ok(());
    };
    for entry in rd.flatten() {
        let p = entry.path();
        if p.extension().and_then(|e| e.to_str()) != Some("png") {
            continue;
        }
        if let Some(stem) = p.file_stem().and_then(|s| s.to_str()) {
            if !keep.contains(stem) {
                let _ = std::fs::remove_file(&p);
            }
        }
    }
    Ok(())
}

/// semver 비교: a > b 인지. 숫자 컴포넌트 비교(프리릴리스 무시).
fn is_newer(a: &str, b: &str) -> bool {
    fn parts(v: &str) -> Vec<u64> {
        v.split('-')
            .next()
            .unwrap_or("")
            .split('.')
            .map(|p| p.parse().unwrap_or(0))
            .collect()
    }
    let (pa, pb) = (parts(a), parts(b));
    for i in 0..pa.len().max(pb.len()) {
        let x = pa.get(i).copied().unwrap_or(0);
        let y = pb.get(i).copied().unwrap_or(0);
        if x != y {
            return x > y;
        }
    }
    false
}
