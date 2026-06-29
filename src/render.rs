use anyhow::{Context, Result};
use resvg::{tiny_skia, usvg};
use std::path::Path;

/// 렌더링 전용 SVG 문자열(채움색 명시). PNG 굽기에만 사용.
fn render_svg(path_d: &str, color: &str) -> String {
    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"><path d="{path_d}" fill="{color}" /></svg>"#
    )
}

/// SVG path `d` → size×size PNG 파일.
pub fn render_png(path_d: &str, color: &str, size: u32, out: &Path) -> Result<()> {
    let svg = render_svg(path_d, color);
    let opt = usvg::Options::default();
    let tree = usvg::Tree::from_str(&svg, &opt).context("parsing generated SVG")?;

    let mut pixmap =
        tiny_skia::Pixmap::new(size, size).context("allocating pixmap")?;
    let scale = size as f32 / 24.0;
    let transform = tiny_skia::Transform::from_scale(scale, scale);
    resvg::render(&tree, transform, &mut pixmap.as_mut());

    if let Some(parent) = out.parent() {
        std::fs::create_dir_all(parent)?;
    }
    pixmap
        .save_png(out)
        .with_context(|| format!("writing {}", out.display()))?;
    Ok(())
}

/// MDI SVG 파일에서 첫 `path d="..."` 추출. MDI 아이콘은 단일 path.
pub fn extract_path_d(svg: &str) -> Option<String> {
    let start = svg.find(" d=\"")? + 4;
    let rest = &svg[start..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}
