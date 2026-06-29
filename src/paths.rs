use anyhow::Result;
use std::path::{Path, PathBuf};

/// 런타임 자산 경로 해석.
///
/// - `bundle`: 워크플로 번들(읽기전용)에 동봉된 meta.json / icons. 검색 시 기본 소스.
/// - `data`: `$alfred_workflow_data` (쓰기 가능). mdi-update 가 증분 산출물을 기록하는 overlay.
///
/// 검색은 항상 overlay 우선, 없으면 bundle 로 폴백한다.
pub struct Paths {
    pub bundle: PathBuf,
    pub data: Option<PathBuf>,
}

impl Paths {
    pub fn discover() -> Result<Paths> {
        let cwd = std::env::current_dir()?;
        // 패키징 레이아웃: meta.json / icons 가 워크플로 루트에 위치.
        // 개발 레이아웃: assets/ 하위.
        let bundle = if cwd.join("meta.json").exists() {
            cwd
        } else {
            cwd.join("assets")
        };
        let data = std::env::var_os("alfred_workflow_data").map(PathBuf::from);
        Ok(Paths { bundle, data })
    }

    pub fn meta_bundle(&self) -> PathBuf {
        self.bundle.join("meta.json")
    }

    pub fn meta_overlay(&self) -> Option<PathBuf> {
        self.data.as_ref().map(|d| d.join("meta.json"))
    }

    /// 검색이 읽을 meta.json (overlay 우선).
    pub fn meta_active(&self) -> PathBuf {
        if let Some(p) = self.meta_overlay() {
            if p.exists() {
                return p;
            }
        }
        self.meta_bundle()
    }

    pub fn icon_overlay(&self, name: &str) -> Option<PathBuf> {
        self.data
            .as_ref()
            .map(|d| d.join("icons").join(format!("{name}.png")))
    }

    pub fn icon_bundle(&self, name: &str) -> PathBuf {
        self.bundle.join("icons").join(format!("{name}.png"))
    }

    /// Alfred item 의 icon.path 로 넣을 실제 PNG 경로 (overlay 우선, 없으면 bundle).
    pub fn icon_resolve(&self, name: &str) -> PathBuf {
        if let Some(p) = self.icon_overlay(name) {
            if p.exists() {
                return p;
            }
        }
        self.icon_bundle(name)
    }

    pub fn state(&self) -> Option<PathBuf> {
        self.data.as_ref().map(|d| d.join("state.json"))
    }

    pub fn data_dir(&self) -> Option<&Path> {
        self.data.as_deref()
    }
}
