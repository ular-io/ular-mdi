use crate::config::Config;
use crate::meta::Icon;
use crate::paths::Paths;
use serde::Serialize;

/// Alfred Script Filter 최상위 출력.
#[derive(Serialize)]
pub struct ScriptFilter {
    pub items: Vec<Item>,
}

#[derive(Serialize)]
pub struct Item {
    pub uid: String,
    pub title: String,
    pub subtitle: String,
    /// Enter(기본) 출력: 아이콘 이름 `mdi-<name>`.
    pub arg: String,
    pub icon: Icon_,
    pub mods: Mods,
    /// 큰따옴표 미리보기/복사 텍스트.
    pub text: Text,
    #[serde(rename = "quicklookurl", skip_serializing_if = "Option::is_none")]
    pub quicklook: Option<String>,
}

#[derive(Serialize)]
pub struct Icon_ {
    pub path: String,
}

#[derive(Serialize)]
pub struct Mods {
    /// Cmd: SVG 코드 전체.
    pub cmd: Mod,
    /// Opt(alt): path data.
    pub alt: Mod,
    /// Ctrl: PNG 파일(파일 복사 액션으로 연결).
    pub ctrl: Mod,
}

#[derive(Serialize)]
pub struct Mod {
    pub valid: bool,
    pub arg: String,
    pub subtitle: String,
}

#[derive(Serialize)]
pub struct Text {
    pub copy: String,
    pub largetype: String,
}

/// 표준 MDI 스타일 SVG 마크업(채움색 미지정 → currentColor 상속).
pub fn svg_markup(path_d: &str) -> String {
    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d="{path_d}" /></svg>"#
    )
}

fn icon_name(icon: &Icon) -> String {
    format!("mdi-{}", icon.name)
}

pub fn build(hits: &[&Icon], paths: &Paths, _cfg: &Config) -> ScriptFilter {
    let items = hits
        .iter()
        .map(|icon| {
            let name = icon_name(icon);
            let svg = svg_markup(&icon.path);
            let png = paths.icon_resolve(&icon.name).to_string_lossy().into_owned();
            let subtitle = if icon.tags.is_empty() {
                name.clone()
            } else {
                format!("{}  ·  {}", name, icon.tags.join(", "))
            };
            Item {
                uid: icon.name.clone(),
                title: icon.name.clone(),
                subtitle,
                arg: name.clone(),
                icon: Icon_ { path: png.clone() },
                mods: Mods {
                    cmd: Mod {
                        valid: true,
                        arg: svg.clone(),
                        subtitle: "Copy SVG code".to_string(),
                    },
                    alt: Mod {
                        valid: true,
                        arg: icon.path.clone(),
                        subtitle: "Copy SVG path data".to_string(),
                    },
                    ctrl: Mod {
                        valid: true,
                        arg: png,
                        subtitle: "Copy PNG file".to_string(),
                    },
                },
                text: Text {
                    copy: name.clone(),
                    largetype: name,
                },
                quicklook: None,
            }
        })
        .collect();

    ScriptFilter { items }
}
