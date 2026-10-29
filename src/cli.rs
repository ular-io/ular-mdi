use anyhow::{bail, Context, Result};
use std::path::PathBuf;

use crate::config::Config;
use crate::meta::{Icon, Meta, UpstreamIcon};
use crate::paths::Paths;
use crate::{alfred, render, search, update};

/// `mdi search <query...>` — Alfred Script Filter.
pub fn search(args: &[String]) -> Result<()> {
    let query = args.join(" ");
    let paths = Paths::discover()?;
    let cfg = Config::from_env();
    let meta_path = paths.meta_active();
    if !meta_path.exists() {
        println!("{}", serde_json::to_string(&update::bootstrap(&paths)?)?);
        return Ok(());
    }
    let meta = Meta::read_file(&meta_path)
        .with_context(|| "meta.json 로드 실패 (워크플로 자산 누락?)")?;
    let hits = search::search(&meta, &query, cfg.limit);
    let out = alfred::build(&hits, &paths, &cfg);
    println!("{}", serde_json::to_string(&out)?);
    Ok(())
}

/// `mdi update` — 증분 갱신.
pub fn update(_args: &[String]) -> Result<()> {
    let paths = Paths::discover()?;
    let cfg = Config::from_env();
    update::run(&paths, &cfg)
}

/// `mdi bake --svg-dir <d> --meta <f> --out-dir <d> --version <v> [--size N] [--color #hex]`
/// 릴리스(CI) 시 augmented meta.json + PNG 캐시 생성.
pub fn bake(args: &[String]) -> Result<()> {
    let mut svg_dir = None;
    let mut meta_in = None;
    let mut out_dir = None;
    let mut version = None;
    let mut size = 88u32;
    let mut color = "#000000".to_string();

    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--svg-dir" => svg_dir = it.next().map(PathBuf::from),
            "--meta" => meta_in = it.next().map(PathBuf::from),
            "--out-dir" => out_dir = it.next().map(PathBuf::from),
            "--version" => version = it.next().cloned(),
            "--size" => {
                size = it.next().and_then(|s| s.parse().ok()).unwrap_or(88);
            }
            "--color" => {
                if let Some(c) = it.next() {
                    color = c.clone();
                }
            }
            other => bail!("알 수 없는 인자: {other}"),
        }
    }

    let svg_dir = svg_dir.context("--svg-dir 필요")?;
    let meta_in = meta_in.context("--meta 필요")?;
    let out_dir = out_dir.context("--out-dir 필요")?;
    let version = version.context("--version 필요")?;

    let upstream: Vec<UpstreamIcon> =
        serde_json::from_str(&std::fs::read_to_string(&meta_in)?).context("원본 meta 파싱 실패")?;

    std::fs::create_dir_all(out_dir.join("icons"))?;
    let mut icons = Vec::with_capacity(upstream.len());

    for u in &upstream {
        let svg_path = svg_dir.join(format!("{}.svg", u.name));
        let svg = std::fs::read_to_string(&svg_path)
            .with_context(|| format!("reading {}", svg_path.display()))?;
        let d = render::extract_path_d(&svg).unwrap_or_default();
        render::render_png(
            &d,
            &color,
            size,
            &out_dir.join("icons").join(format!("{}.png", u.name)),
        )?;
        icons.push(Icon {
            name: u.name.clone(),
            aliases: u.aliases.clone(),
            tags: u.tags.clone(),
            version: u.version.clone(),
            path: d,
        });
    }

    let meta = Meta { version, icons };
    std::fs::write(out_dir.join("meta.json"), serde_json::to_vec(&meta)?)?;
    eprintln!("baked {} icons → {}", meta.icons.len(), out_dir.display());
    Ok(())
}
