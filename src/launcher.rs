use std::io;
use std::path::Path;
use std::process::Command;

pub fn centered_text_y(row_top: i32, row_height: i32, font_size: i32) -> i32 {
    row_top + ((row_height - font_size).max(0) / 2)
}

pub fn abbreviate_target(target: &str, max_len: usize) -> String {
    if target.len() <= max_len {
        return target.to_string();
    }

    let keep = max_len.saturating_sub(3);
    let tail = target
        .chars()
        .rev()
        .take(keep)
        .collect::<Vec<char>>()
        .into_iter()
        .rev()
        .collect::<String>();
    format!("...{}", tail)
}

pub fn truncate_with_ellipsis_end(value: &str, max_chars: usize) -> String {
    let count = value.chars().count();
    if count <= max_chars {
        return value.to_string();
    }
    if max_chars <= 3 {
        return ".".repeat(max_chars);
    }

    let keep = max_chars - 3;
    let prefix: String = value.chars().take(keep).collect();
    format!("{}...", prefix)
}

pub fn compact_target_hint(target: &str) -> String {
    let path = Path::new(target);

    if let Some(file_name) = path.file_name().and_then(|value| value.to_str()) {
        if let Some(parent_name) = path
            .parent()
            .and_then(|parent| parent.file_name())
            .and_then(|value| value.to_str())
        {
            return format!("\\{}\\{}", parent_name, file_name);
        }
        return file_name.to_string();
    }

    abbreviate_target(target, 44)
}

fn strip_wrapping_quotes(value: &str) -> &str {
    let trimmed = value.trim();
    if trimmed.len() >= 2 && trimmed.starts_with('"') && trimmed.ends_with('"') {
        &trimmed[1..trimmed.len() - 1]
    } else {
        trimmed
    }
}

fn looks_like_url(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    lower.starts_with("http://")
        || lower.starts_with("https://")
        || lower.starts_with("file://")
        || lower.starts_with("mailto:")
}

fn looks_like_path(value: &str) -> bool {
    value.contains('\\')
        || value.contains('/')
        || (value.len() >= 2 && value.as_bytes()[1] == b':')
}

pub fn launch_target(target: &str) -> io::Result<()> {
    let target = strip_wrapping_quotes(target);
    if target.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "target is empty",
        ));
    }

    if !looks_like_url(target) && looks_like_path(target) && !Path::new(target).exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("target path not found: {target}"),
        ));
    }

    Command::new("cmd")
        .arg("/C")
        .arg("start")
        .arg("")
        .arg(target)
        .spawn()
        .map(|_| ())
        .map_err(|err| io::Error::new(err.kind(), format!("failed to launch '{target}': {err}")))
}

#[cfg(test)]
mod tests {
    use super::{looks_like_path, looks_like_url, strip_wrapping_quotes};

    #[test]
    fn strip_wrapping_quotes_removes_outer_quotes() {
        assert_eq!(strip_wrapping_quotes("\"C:/Tools/app.exe\""), "C:/Tools/app.exe");
        assert_eq!(strip_wrapping_quotes("  \"cmd\"  "), "cmd");
        assert_eq!(strip_wrapping_quotes("notepad"), "notepad");
    }

    #[test]
    fn url_detection_works() {
        assert!(looks_like_url("https://example.com"));
        assert!(looks_like_url("HTTP://example.com"));
        assert!(!looks_like_url("C:/Windows/notepad.exe"));
    }

    #[test]
    fn path_detection_works() {
        assert!(looks_like_path("C:/Windows/notepad.exe"));
        assert!(looks_like_path("C:\\Windows\\notepad.exe"));
        assert!(looks_like_path(".\\script.bat"));
        assert!(!looks_like_path("notepad"));
    }
}
