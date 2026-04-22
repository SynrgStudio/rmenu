use std::io;
use std::iter::once;
use std::os::windows::ffi::OsStrExt;
use std::path::Path;
use std::process::Command;

use windows::core::PCWSTR;
use windows::Win32::UI::Shell::ShellExecuteW;
use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

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

fn trim_outer_quotes(value: &str) -> &str {
    let trimmed = value.trim();
    if trimmed.len() >= 2 && trimmed.starts_with('"') && trimmed.ends_with('"') {
        &trimmed[1..trimmed.len() - 1]
    } else {
        trimmed
    }
}

fn split_executable_and_args(raw: &str) -> (&str, Option<&str>) {
    let input = raw.trim();
    if input.is_empty() {
        return ("", None);
    }

    if let Some(rest) = input.strip_prefix('"') {
        if let Some(end_quote) = rest.find('"') {
            let file = &rest[..end_quote];
            let after = rest[end_quote + 1..].trim();
            if after.is_empty() {
                return (file, None);
            }
            return (file, Some(after));
        }
    }

    if let Some((file, args)) = input.split_once(char::is_whitespace) {
        let args = args.trim();
        if args.is_empty() {
            return (file, None);
        }
        return (file, Some(args));
    }

    (input, None)
}

fn to_wstring(value: &str) -> Vec<u16> {
    std::ffi::OsStr::new(value)
        .encode_wide()
        .chain(once(0))
        .collect()
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

fn should_fallback_to_cmd(raw_target: &str, file_part: &str) -> bool {
    if looks_like_url(file_part) || looks_like_path(file_part) {
        return false;
    }

    let lowered = raw_target.to_ascii_lowercase();
    lowered.contains("&&")
        || lowered.contains("||")
        || lowered.contains('|')
        || lowered.contains('>')
        || lowered.starts_with("cmd ")
        || lowered.starts_with("cmd.exe ")
}

fn launch_with_shell_execute(file: &str, args: Option<&str>) -> io::Result<()> {
    let operation = to_wstring("open");
    let file_w = to_wstring(file);
    let args_w = args.map(to_wstring);

    let args_pcwstr = args_w
        .as_ref()
        .map_or(PCWSTR::null(), |value| PCWSTR(value.as_ptr()));

    let result = unsafe {
        ShellExecuteW(
            None,
            PCWSTR(operation.as_ptr()),
            PCWSTR(file_w.as_ptr()),
            args_pcwstr,
            PCWSTR::null(),
            SW_SHOWNORMAL,
        )
    };

    if result.0 as usize > 32 {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::Other,
            format!("ShellExecuteW failed for '{file}' with code {}", result.0 as usize),
        ))
    }
}

fn launch_with_cmd_start(raw_target: &str) -> io::Result<()> {
    Command::new("cmd")
        .arg("/C")
        .arg("start")
        .arg("")
        .arg(raw_target)
        .spawn()
        .map(|_| ())
        .map_err(|err| io::Error::new(err.kind(), format!("cmd start failed for '{raw_target}': {err}")))
}

pub fn launch_target(target: &str) -> io::Result<()> {
    let raw_target = trim_outer_quotes(target);
    if raw_target.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "target is empty",
        ));
    }

    let (file_part, args_part) = split_executable_and_args(raw_target);
    if file_part.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "target executable is empty",
        ));
    }

    if !looks_like_url(file_part) && looks_like_path(file_part) && !Path::new(file_part).exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("target path not found: {file_part}"),
        ));
    }

    match launch_with_shell_execute(file_part, args_part) {
        Ok(()) => Ok(()),
        Err(shell_err) => {
            if should_fallback_to_cmd(raw_target, file_part) {
                launch_with_cmd_start(raw_target)
            } else {
                Err(io::Error::new(
                    shell_err.kind(),
                    format!("failed to launch '{raw_target}': {shell_err}"),
                ))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        looks_like_path, looks_like_url, should_fallback_to_cmd, split_executable_and_args,
        trim_outer_quotes,
    };

    #[test]
    fn trim_outer_quotes_removes_outer_quotes() {
        assert_eq!(trim_outer_quotes("\"C:/Tools/app.exe\""), "C:/Tools/app.exe");
        assert_eq!(trim_outer_quotes("  \"cmd\"  "), "cmd");
        assert_eq!(trim_outer_quotes("notepad"), "notepad");
    }

    #[test]
    fn split_executable_and_args_parses_quoted_targets() {
        let (file, args) = split_executable_and_args("\"C:/Program Files/App/app.exe\" --flag 1");
        assert_eq!(file, "C:/Program Files/App/app.exe");
        assert_eq!(args, Some("--flag 1"));
    }

    #[test]
    fn split_executable_and_args_parses_unquoted_targets() {
        let (file, args) = split_executable_and_args("notepad.exe readme.txt");
        assert_eq!(file, "notepad.exe");
        assert_eq!(args, Some("readme.txt"));
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

    #[test]
    fn cmd_fallback_only_for_shell_like_commands() {
        assert!(should_fallback_to_cmd("cmd /k dir", "cmd"));
        assert!(should_fallback_to_cmd("echo hello && pause", "echo"));
        assert!(!should_fallback_to_cmd("notepad.exe", "notepad.exe"));
        assert!(!should_fallback_to_cmd("C:/Windows/notepad.exe", "C:/Windows/notepad.exe"));
    }
}
