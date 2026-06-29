use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// 우리가 동봉/생성하는 augmented meta. 원본 @mdi/svg meta 에 path `d` 를 더한 형태.
#[derive(Serialize, Deserialize, Default)]
pub struct Meta {
    /// 이 meta 가 빌드된 @mdi/svg 버전.
    pub version: String,
    pub icons: Vec<Icon>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Icon {
    pub name: String,
    #[serde(default)]
    pub aliases: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    /// 아이콘이 추가된 @mdi/svg 버전 (증분 갱신 판단용).
    #[serde(default)]
    pub version: String,
    /// SVG `<path d="...">` 데이터. modifier 출력(SVG 전체/path)용.
    #[serde(default)]
    pub path: String,
}

impl Meta {
    pub fn read_file(p: &Path) -> Result<Meta> {
        let data =
            std::fs::read_to_string(p).with_context(|| format!("reading {}", p.display()))?;
        serde_json::from_str(&data).with_context(|| format!("parsing {}", p.display()))
    }
}

/// 원본 @mdi/svg meta.json 엔트리 (배열). path 데이터 없음.
#[derive(Deserialize)]
pub struct UpstreamIcon {
    pub name: String,
    #[serde(default)]
    pub aliases: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub version: String,
}
