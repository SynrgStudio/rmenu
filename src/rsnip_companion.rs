use crate::settings::rmenu_data_dirs;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs::{self, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};
#[cfg(windows)]
use std::os::windows::{ffi::OsStrExt, process::CommandExt};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread::sleep;
use std::time::{Duration, Instant};
#[cfg(windows)]
use windows::core::PCWSTR;
#[cfg(windows)]
use windows::Win32::System::Com::Urlmon::URLDownloadToFileW;

const DEFAULT_DEV_RSNIP_PATH: &str = "C:\\rSnip\\target\\release\\rsnip.exe";
const RSNIP_LATEST_EXE_URL: &str =
    "https://github.com/SynrgStudio/rSnip/releases/latest/download/rsnip.exe";
const RSNIP_PIPE_PATH: &str = r"\\.\pipe\rsnip";
const DEFAULT_CONNECT_TIMEOUT: Duration = Duration::from_millis(1_000);
const DEFAULT_START_TIMEOUT: Duration = Duration::from_millis(2_000);
const RETRY_DELAY: Duration = Duration::from_millis(50);
#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RsnipCommand {
    Snip,
    Record,
    Ocr,
    Shutdown,
}

impl RsnipCommand {
    pub fn as_cli_arg(self) -> &'static str {
        match self {
            Self::Snip => "snip",
            Self::Record => "record",
            Self::Ocr => "ocr",
            Self::Shutdown => "stop",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RsnipIpcRequest {
    pub cmd: RsnipCommand,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RsnipIpcResponse {
    Ok { message: String },
    Error { message: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RsnipError {
    NotInstalled,
    Io(String),
    Protocol(String),
    Timeout(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RsnipCompanion {
    pub exe_path: PathBuf,
}

impl RsnipCompanion {
    pub fn discover() -> Result<Self, RsnipError> {
        discover_rsnip_path()
            .map(|exe_path| Self { exe_path })
            .ok_or(RsnipError::NotInstalled)
    }

    pub fn ensure_daemon(&self) -> Result<(), RsnipError> {
        if self.ping(DEFAULT_CONNECT_TIMEOUT).is_ok() {
            return Ok(());
        }

        start_rsnip_daemon(&self.exe_path)?;
        self.wait_until_reachable(DEFAULT_START_TIMEOUT)
    }

    pub fn send(&self, command: RsnipCommand) -> Result<RsnipIpcResponse, RsnipError> {
        send_command(command, DEFAULT_CONNECT_TIMEOUT)
    }

    pub fn ensure_and_send(&self, command: RsnipCommand) -> Result<RsnipIpcResponse, RsnipError> {
        self.ensure_daemon()?;
        self.send(command)
    }

    pub fn ping(&self, timeout: Duration) -> Result<(), RsnipError> {
        open_pipe_with_timeout(timeout).map(|_| ())
    }

    fn wait_until_reachable(&self, timeout: Duration) -> Result<(), RsnipError> {
        self.ping(timeout)
    }
}

pub fn companion_rsnip_path_from_data_dir(cli_data_dir: Option<&str>) -> PathBuf {
    rmenu_data_dirs(cli_data_dir)
        .companions_dir
        .join("rsnip")
        .join("rsnip.exe")
}

pub fn install_rsnip_latest() -> Result<PathBuf, RsnipError> {
    let destination = companion_rsnip_path_from_data_dir(None);
    prepare_rsnip_install_dirs(&destination)?;

    let _ = send_command(RsnipCommand::Shutdown, Duration::from_millis(500));
    sleep(Duration::from_millis(250));

    download_rsnip_latest(&destination).or_else(|download_err| {
        let source = PathBuf::from(DEFAULT_DEV_RSNIP_PATH);
        if !source.exists() {
            return Err(download_err);
        }
        fs::copy(&source, &destination).map_err(io_error)?;
        Ok(())
    })?;

    RsnipCompanion {
        exe_path: destination.clone(),
    }
    .ensure_daemon()?;
    Ok(destination)
}

pub fn install_rsnip_from_dev() -> Result<PathBuf, RsnipError> {
    let source = PathBuf::from(DEFAULT_DEV_RSNIP_PATH);
    if !source.exists() {
        return Err(RsnipError::Io(format!(
            "local RSnip source not found: {}",
            source.display()
        )));
    }

    let destination = companion_rsnip_path_from_data_dir(None);
    prepare_rsnip_install_dirs(&destination)?;
    fs::copy(&source, &destination).map_err(io_error)?;
    RsnipCompanion {
        exe_path: destination.clone(),
    }
    .ensure_daemon()?;
    Ok(destination)
}

fn prepare_rsnip_install_dirs(destination: &Path) -> Result<(), RsnipError> {
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent).map_err(io_error)?;
        fs::create_dir_all(parent.join("config")).map_err(io_error)?;
        fs::create_dir_all(parent.join("state")).map_err(io_error)?;
        fs::create_dir_all(parent.join("logs")).map_err(io_error)?;
    }
    Ok(())
}

#[cfg(windows)]
fn download_rsnip_latest(destination: &Path) -> Result<(), RsnipError> {
    let url = wide_null(RSNIP_LATEST_EXE_URL);
    let path = wide_null(&destination.to_string_lossy());
    unsafe { URLDownloadToFileW(None, PCWSTR(url.as_ptr()), PCWSTR(path.as_ptr()), 0, None) }
        .map_err(|err| RsnipError::Io(format!("failed to download RSnip latest release: {err}")))
}

#[cfg(not(windows))]
fn download_rsnip_latest(_destination: &Path) -> Result<(), RsnipError> {
    Err(RsnipError::Io(
        "GitHub release download is only supported on Windows".to_string(),
    ))
}

#[cfg(windows)]
fn wide_null(value: &str) -> Vec<u16> {
    std::ffi::OsStr::new(value)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

pub fn discover_rsnip_path() -> Option<PathBuf> {
    let managed_path = companion_rsnip_path_from_data_dir(None);
    if managed_path.exists() {
        return Some(managed_path);
    }

    if let Ok(value) = env::var("RMENU_RSNIP_PATH") {
        let trimmed = value.trim();
        if !trimmed.is_empty() {
            let path = PathBuf::from(trimmed);
            if path.exists() {
                return Some(path);
            }
        }
    }

    let dev_path = PathBuf::from(DEFAULT_DEV_RSNIP_PATH);
    if dev_path.exists() {
        return Some(dev_path);
    }

    find_rsnip_on_path()
}

fn find_rsnip_on_path() -> Option<PathBuf> {
    let path_value = env::var_os("PATH")?;
    let mut matches = env::split_paths(&path_value)
        .map(|dir| dir.join("rsnip.exe"))
        .filter(|candidate| candidate.exists())
        .collect::<Vec<_>>();
    matches.sort();
    matches.dedup();

    if matches.len() == 1 {
        matches.into_iter().next()
    } else {
        None
    }
}

pub fn request_json_line(command: RsnipCommand) -> Result<String, RsnipError> {
    let mut line = serde_json::to_string(&RsnipIpcRequest { cmd: command })
        .map_err(|error| RsnipError::Protocol(format!("serialize request failed: {error}")))?;
    line.push('\n');
    Ok(line)
}

pub fn parse_response_json_line(line: &str) -> Result<RsnipIpcResponse, RsnipError> {
    serde_json::from_str(line.trim())
        .map_err(|error| RsnipError::Protocol(format!("parse response failed: {error}")))
}

pub fn send_command(
    command: RsnipCommand,
    timeout: Duration,
) -> Result<RsnipIpcResponse, RsnipError> {
    let pipe = open_pipe_with_timeout(timeout)?;
    let mut reader = BufReader::new(pipe);
    let request = request_json_line(command)?;
    reader
        .get_mut()
        .write_all(request.as_bytes())
        .and_then(|_| reader.get_mut().flush())
        .map_err(io_error)?;

    let mut response = String::new();
    reader.read_line(&mut response).map_err(io_error)?;
    if response.trim().is_empty() {
        return Err(RsnipError::Protocol(
            "empty response from rsnip daemon".to_string(),
        ));
    }

    parse_response_json_line(&response)
}

fn open_pipe_with_timeout(timeout: Duration) -> Result<std::fs::File, RsnipError> {
    let started = Instant::now();
    loop {
        match OpenOptions::new()
            .read(true)
            .write(true)
            .open(RSNIP_PIPE_PATH)
        {
            Ok(file) => return Ok(file),
            Err(error) => {
                if started.elapsed() >= timeout {
                    return Err(RsnipError::Timeout(format!(
                        "rsnip daemon pipe was not reachable within {}ms: {error}",
                        timeout.as_millis()
                    )));
                }
                sleep(RETRY_DELAY);
            }
        }
    }
}

fn start_rsnip_daemon(exe_path: &Path) -> Result<(), RsnipError> {
    let mut command = Command::new(exe_path);
    command
        .arg("daemon")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    #[cfg(windows)]
    command.creation_flags(CREATE_NO_WINDOW);

    command.spawn().map(|_| ()).map_err(|error| {
        RsnipError::Io(format!(
            "failed to start rsnip daemon '{}': {error}",
            exe_path.display()
        ))
    })
}

fn io_error(error: io::Error) -> RsnipError {
    RsnipError::Io(error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_json_matches_rsnip_protocol() {
        assert_eq!(
            request_json_line(RsnipCommand::Snip).expect("serialize"),
            "{\"cmd\":\"snip\"}\n"
        );
        assert_eq!(
            request_json_line(RsnipCommand::Record).expect("serialize"),
            "{\"cmd\":\"record\"}\n"
        );
        assert_eq!(
            request_json_line(RsnipCommand::Ocr).expect("serialize"),
            "{\"cmd\":\"ocr\"}\n"
        );
        assert_eq!(
            request_json_line(RsnipCommand::Shutdown).expect("serialize"),
            "{\"cmd\":\"shutdown\"}\n"
        );
    }

    #[test]
    fn response_json_matches_rsnip_protocol() {
        assert_eq!(
            parse_response_json_line("{\"ok\":{\"message\":\"accepted\"}}\n").expect("parse"),
            RsnipIpcResponse::Ok {
                message: "accepted".to_string()
            }
        );
        assert_eq!(
            parse_response_json_line("{\"error\":{\"message\":\"nope\"}}\n").expect("parse"),
            RsnipIpcResponse::Error {
                message: "nope".to_string()
            }
        );
    }

    #[test]
    fn companion_path_uses_data_dir_layout() {
        assert_eq!(
            companion_rsnip_path_from_data_dir(Some("C:\\rMenuData")),
            PathBuf::from("C:\\rMenuData\\companions\\rsnip\\rsnip.exe")
        );
    }

    #[test]
    fn cli_args_match_commands() {
        assert_eq!(RsnipCommand::Snip.as_cli_arg(), "snip");
        assert_eq!(RsnipCommand::Record.as_cli_arg(), "record");
        assert_eq!(RsnipCommand::Ocr.as_cli_arg(), "ocr");
        assert_eq!(RsnipCommand::Shutdown.as_cli_arg(), "stop");
    }
}
