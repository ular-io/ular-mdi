use anyhow::{Context, Result};
use flate2::read::GzDecoder;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::{ErrorKind, Read};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, SystemTime};
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

/// 갱신 중 표시 파일. 이보다 오래된 잠금은 비정상 종료로 보고 무시한다.
const LOCK_FILE: &str = ".updating";
const LOCK_STALE: Duration = Duration::from_secs(15 * 60);
/// 마지막 갱신 실패 메시지. 있으면 첫 검색이 자동 재시도하지 않는다(키 입력마다 재다운로드 방지).
const FAILED_FILE: &str = "update-failed.txt";
/// bootstrap 이 잠금을 잡은 뒤 넘겨준 자식 프로세스임을 표시하는 환경변수.
const LOCK_HELD_ENV: &str = "MDI_LOCK_HELD";

/// 잠금 파일. drop 시 삭제한다.
struct Lock(PathBuf);

impl Drop for Lock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

fn lock_held(path: &Path) -> bool {
    std::fs::metadata(path)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| SystemTime::now().duration_since(t).ok())
        .is_some_and(|age| age < LOCK_STALE)
}

fn try_lock(data: &Path) -> Result<Option<Lock>> {
    let path = data.join(LOCK_FILE);
    if lock_held(&path) {
        return Ok(None);
    }
    let _ = std::fs::remove_file(&path); // 오래된 잠금
    match OpenOptions::new().write(true).create_new(true).open(&path) {
        Ok(_) => Ok(Some(Lock(path))),
        Err(e) if e.kind() == ErrorKind::AlreadyExists => Ok(None),
        Err(e) => Err(e.into()),
    }
}

fn data_dir(paths: &Paths) -> Result<PathBuf> {
    let data = paths
        .data_dir()
        .context("alfred_workflow_data 가 설정되지 않음 (Alfred 환경 밖)")?
        .to_path_buf();
    std::fs::create_dir_all(data.join("icons"))?;
    Ok(data)
}

/// 아이콘이 하나도 없을 때(배포 패키지 첫 실행) Script Filter 출력.
///
/// 배포 패키지에는 아이콘이 없다(동기화 폴더 용량 절감). 갱신이 진행 중이 아니면
/// 백그라운드로 `mdi update` 를 시작하고, `rerun` 으로 완료되면 결과가 자동으로 바뀐다.
pub fn bootstrap(paths: &Paths) -> Result<serde_json::Value> {
    let data = data_dir(paths)?;
    if !lock_held(&data.join(LOCK_FILE)) {
        if let Ok(msg) = std::fs::read_to_string(data.join(FAILED_FILE)) {
            return Ok(status(
                "MDI 아이콘 내려받기 실패",
                &format!("{}  ·  mdi-update 로 다시 시도", msg.trim()),
                None,
            ));
        }
        if let Some(lock) = try_lock(&data)? {
            spawn_update()?;
            // 잠금은 자식이 넘겨받아 끝날 때 지운다.
            std::mem::forget(lock);
        }
    }
    Ok(status(
        "MDI 아이콘 내려받는 중…",
        "처음 한 번만 받는다. 완료되면 결과가 자동으로 표시된다",
        Some(1.0),
    ))
}

fn spawn_update() -> Result<()> {
    use std::os::unix::process::CommandExt;
    Command::new(std::env::current_exe()?)
        .arg("update")
        .env(LOCK_HELD_ENV, "1")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        // 입력이 바뀌면 Alfred 가 이전 Script Filter 를 종료한다. 갱신은 계속되도록 그룹을 분리한다.
        .process_group(0)
        .spawn()
        .context("mdi update 시작 실패")?;
    Ok(())
}

fn status(title: &str, subtitle: &str, rerun: Option<f64>) -> serde_json::Value {
    let mut v = serde_json::json!({
        "items": [{ "title": title, "subtitle": subtitle, "valid": false }]
    });
    if let Some(r) = rerun {
        v["rerun"] = serde_json::json!(r);
    }
    v
}

pub fn run(paths: &Paths, cfg: &Config) -> Result<()> {
    let data = data_dir(paths)?;
    let _lock = if std::env::var_os(LOCK_HELD_ENV).is_some() {
        Lock(data.join(LOCK_FILE))
    } else {
        match try_lock(&data)? {
            Some(l) => l,
            None => {
                notify::post("MDI", "이미 아이콘을 갱신하는 중입니다");
                return Ok(());
            }
        }
    };
    let _ = std::fs::remove_file(data.join(FAILED_FILE));
    let r = fetch_and_bake(&data, paths, cfg);
    if let Err(e) = &r {
        let _ = std::fs::write(data.join(FAILED_FILE), format!("{e:#}"));
        notify::post("MDI", &format!("갱신 실패: {e:#}"));
    }
    r
}

fn fetch_and_bake(data: &Path, paths: &Paths, cfg: &Config) -> Result<()> {
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

    // 검색은 meta.json 유무로 첫 내려받기 완료를 판단하므로 원자적으로 교체한다.
    let tmp = data.join("meta.json.tmp");
    std::fs::write(&tmp, serde_json::to_vec(&meta)?)?;
    std::fs::rename(&tmp, data.join("meta.json"))?;
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
