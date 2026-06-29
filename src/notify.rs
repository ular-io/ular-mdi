/// macOS 알림 표시. 사용자 텍스트는 셸을 거치지 않고 osascript 인자로 직접 전달.
pub fn post(title: &str, message: &str) {
    let script = format!(
        "display notification {} with title {}",
        applescript_quote(message),
        applescript_quote(title)
    );
    let _ = std::process::Command::new("osascript")
        .arg("-e")
        .arg(script)
        .status();
}

/// AppleScript 문자열 리터럴 이스케이프.
fn applescript_quote(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            _ => out.push(c),
        }
    }
    out.push('"');
    out
}
