use anyhow::{Context, Result};
use serde::Deserialize;
use std::io::Read;

const REGISTRY: &str = "https://registry.npmjs.org/@mdi/svg/latest";

#[derive(Deserialize)]
struct LatestResp {
    version: String,
    dist: Dist,
}

#[derive(Deserialize)]
struct Dist {
    tarball: String,
}

/// 최신 @mdi/svg (version, tarball_url). 공개 익명 호출만 사용.
pub fn latest() -> Result<(String, String)> {
    let resp: LatestResp = ureq::get(REGISTRY)
        .call()
        .context("querying npm registry")?
        .into_json()
        .context("parsing registry response")?;
    Ok((resp.version, resp.dist.tarball))
}

/// tarball(gzip) 다운로드 → 바이트.
pub fn download(url: &str) -> Result<Vec<u8>> {
    let mut buf = Vec::new();
    ureq::get(url)
        .call()
        .with_context(|| format!("downloading {url}"))?
        .into_reader()
        .read_to_end(&mut buf)
        .context("reading tarball body")?;
    Ok(buf)
}
