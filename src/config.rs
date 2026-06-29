/// Alfred Workflow Configuration 으로 노출되는 환경변수 기반 설정.
pub struct Config {
    /// PNG 굽는 크기(px). 기본 88.
    pub size: u32,
    /// 아이콘 채움 색. 다크모드 대응 시 흰색 등. 기본 검정.
    pub color: String,
    /// 검색 결과 최대 개수.
    pub limit: usize,
}

impl Config {
    pub fn from_env() -> Config {
        let size = std::env::var("mdi_size")
            .ok()
            .and_then(|s| s.parse().ok())
            .filter(|&n| n > 0)
            .unwrap_or(88);
        let color = std::env::var("mdi_color")
            .ok()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "#000000".to_string());
        let limit = std::env::var("mdi_limit")
            .ok()
            .and_then(|s| s.parse().ok())
            .filter(|&n| n > 0)
            .unwrap_or(30);
        Config { size, color, limit }
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            size: 88,
            color: "#000000".to_string(),
            limit: 30,
        }
    }
}
