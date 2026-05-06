use crate::settings::rmenu_data_dirs;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs::{self, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};
#[cfg(windows)]
use std::os::windows::{ffi::OsStrExt, process::CommandExt};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::Mutex;
use std::thread::sleep;
use std::time::{Duration, Instant};
#[cfg(windows)]
use windows::core::PCWSTR;
#[cfg(windows)]
use windows::Win32::System::Com::Urlmon::URLDownloadToFileW;

const DEFAULT_DEV_RTASKS_PATH: &str = "C:\\rTasks\\target\\release\\rtasks.exe";
const RTASKS_LATEST_EXE_URL: &str =
    "https://github.com/SynrgStudio/rtasks/releases/latest/download/rtasks.exe";
const RTASKS_PIPE_PATH: &str = r"\\.\pipe\rtasks";
const DEFAULT_CONNECT_TIMEOUT: Duration = Duration::from_millis(1_000);
const DEFAULT_START_TIMEOUT: Duration = Duration::from_millis(2_500);
const RETRY_DELAY: Duration = Duration::from_millis(50);
const DISCOVERY_CACHE_TTL: Duration = Duration::from_secs(5);
#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

#[derive(Debug, Clone)]
struct DiscoveryCacheEntry {
    checked_at: Instant,
    path: Option<PathBuf>,
}

static RTASKS_DISCOVERY_CACHE: Mutex<Option<DiscoveryCacheEntry>> = Mutex::new(None);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RtasksCommand {
    Panel,
    AddTask,
    Shutdown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RtasksIpcRequest {
    pub cmd: RtasksCommand,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<RtasksTaskStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<RtasksPriority>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum RtasksTaskStatus {
    Todo,
    Doing,
    Done,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum RtasksPriority {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RtasksIpcResponse {
    Ok { message: String },
    Error { message: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RtasksError {
    NotInstalled,
    Io(String),
    Protocol(String),
    Timeout(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RtasksCompanion {
    pub exe_path: PathBuf,
}

impl RtasksCompanion {
    pub fn discover() -> Result<Self, RtasksError> {
        discover_rtasks_path_cached()
            .map(|exe_path| Self { exe_path })
            .ok_or(RtasksError::NotInstalled)
    }

    pub fn ensure_daemon(&self) -> Result<(), RtasksError> {
        if self.ping(DEFAULT_CONNECT_TIMEOUT).is_ok() {
            return Ok(());
        }

        start_rtasks_daemon(&self.exe_path)?;
        self.wait_until_reachable(DEFAULT_START_TIMEOUT)
    }

    pub fn send(&self, command: RtasksCommand) -> Result<RtasksIpcResponse, RtasksError> {
        send_command(command, DEFAULT_CONNECT_TIMEOUT)
    }

    pub fn ensure_and_send(
        &self,
        command: RtasksCommand,
    ) -> Result<RtasksIpcResponse, RtasksError> {
        self.ensure_daemon()?;
        self.send(command)
    }

    pub fn ensure_and_add_task(
        &self,
        input: String,
        status: Option<RtasksTaskStatus>,
        priority: Option<RtasksPriority>,
    ) -> Result<RtasksIpcResponse, RtasksError> {
        self.ensure_daemon()?;
        send_add_task(input, status, priority, DEFAULT_CONNECT_TIMEOUT)
    }

    pub fn ping(&self, timeout: Duration) -> Result<(), RtasksError> {
        open_pipe_with_timeout(timeout).map(|_| ())
    }

    fn wait_until_reachable(&self, timeout: Duration) -> Result<(), RtasksError> {
        self.ping(timeout)
    }
}

pub fn companion_rtasks_path_from_data_dir(cli_data_dir: Option<&str>) -> PathBuf {
    rmenu_data_dirs(cli_data_dir)
        .companions_dir
        .join("rtasks")
        .join("rtasks.exe")
}

pub fn install_rtasks_latest() -> Result<PathBuf, RtasksError> {
    let destination = companion_rtasks_path_from_data_dir(None);
    prepare_rtasks_install_dirs(&destination)?;

    let _ = send_command(RtasksCommand::Shutdown, Duration::from_millis(500));
    sleep(Duration::from_millis(250));

    download_rtasks_latest(&destination).or_else(|download_err| {
        let source = PathBuf::from(DEFAULT_DEV_RTASKS_PATH);
        if !source.exists() {
            return Err(download_err);
        }
        fs::copy(&source, &destination).map_err(io_error)?;
        Ok(())
    })?;

    set_rtasks_discovery_cache(Some(destination.clone()));
    RtasksCompanion {
        exe_path: destination.clone(),
    }
    .ensure_daemon()?;
    Ok(destination)
}

fn prepare_rtasks_install_dirs(destination: &Path) -> Result<(), RtasksError> {
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent).map_err(io_error)?;
        fs::create_dir_all(parent.join("config")).map_err(io_error)?;
        fs::create_dir_all(parent.join("state")).map_err(io_error)?;
        fs::create_dir_all(parent.join("logs")).map_err(io_error)?;
    }
    Ok(())
}

#[cfg(windows)]
fn download_rtasks_latest(destination: &Path) -> Result<(), RtasksError> {
    let url = wide_null(RTASKS_LATEST_EXE_URL);
    let path = wide_null(&destination.to_string_lossy());
    unsafe { URLDownloadToFileW(None, PCWSTR(url.as_ptr()), PCWSTR(path.as_ptr()), 0, None) }
        .map_err(|err| RtasksError::Io(format!("failed to download RTasks latest release: {err}")))
}

#[cfg(not(windows))]
fn download_rtasks_latest(_destination: &Path) -> Result<(), RtasksError> {
    Err(RtasksError::Io(
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

fn set_rtasks_discovery_cache(path: Option<PathBuf>) {
    let mut cache = RTASKS_DISCOVERY_CACHE.lock().unwrap();
    *cache = Some(DiscoveryCacheEntry {
        checked_at: Instant::now(),
        path,
    });
}

fn discover_rtasks_path_cached() -> Option<PathBuf> {
    {
        let cache = RTASKS_DISCOVERY_CACHE.lock().unwrap();
        if let Some(entry) = cache.as_ref() {
            if entry.checked_at.elapsed() <= DISCOVERY_CACHE_TTL {
                return entry.path.clone();
            }
        }
    }

    let path = discover_rtasks_path();
    set_rtasks_discovery_cache(path.clone());
    path
}

pub fn discover_rtasks_path() -> Option<PathBuf> {
    let managed_path = companion_rtasks_path_from_data_dir(None);
    if managed_path.exists() {
        return Some(managed_path);
    }

    if let Ok(value) = env::var("RMENU_RTASKS_PATH") {
        let trimmed = value.trim();
        if !trimmed.is_empty() {
            let path = PathBuf::from(trimmed);
            if path.exists() {
                return Some(path);
            }
        }
    }

    let dev_path = PathBuf::from(DEFAULT_DEV_RTASKS_PATH);
    if dev_path.exists() {
        return Some(dev_path);
    }

    find_rtasks_on_path()
}

fn find_rtasks_on_path() -> Option<PathBuf> {
    let path_value = env::var_os("PATH")?;
    let mut matches = env::split_paths(&path_value)
        .map(|dir| dir.join("rtasks.exe"))
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

#[cfg(test)]
pub fn request_json_line(command: RtasksCommand) -> Result<String, RtasksError> {
    request_json_line_for(RtasksIpcRequest {
        cmd: command,
        input: None,
        status: None,
        priority: None,
    })
}

fn request_json_line_for(request: RtasksIpcRequest) -> Result<String, RtasksError> {
    let mut line = serde_json::to_string(&request)
        .map_err(|error| RtasksError::Protocol(format!("serialize request failed: {error}")))?;
    line.push('\n');
    Ok(line)
}

pub fn parse_response_json_line(line: &str) -> Result<RtasksIpcResponse, RtasksError> {
    serde_json::from_str(line.trim())
        .map_err(|error| RtasksError::Protocol(format!("parse response failed: {error}")))
}

pub fn send_command(
    command: RtasksCommand,
    timeout: Duration,
) -> Result<RtasksIpcResponse, RtasksError> {
    send_request(
        RtasksIpcRequest {
            cmd: command,
            input: None,
            status: None,
            priority: None,
        },
        timeout,
    )
}

pub fn send_add_task(
    input: String,
    status: Option<RtasksTaskStatus>,
    priority: Option<RtasksPriority>,
    timeout: Duration,
) -> Result<RtasksIpcResponse, RtasksError> {
    send_request(
        RtasksIpcRequest {
            cmd: RtasksCommand::AddTask,
            input: Some(input),
            status,
            priority,
        },
        timeout,
    )
}

fn send_request(
    request: RtasksIpcRequest,
    timeout: Duration,
) -> Result<RtasksIpcResponse, RtasksError> {
    let pipe = open_pipe_with_timeout(timeout)?;
    let mut reader = BufReader::new(pipe);
    let request = request_json_line_for(request)?;
    reader
        .get_mut()
        .write_all(request.as_bytes())
        .and_then(|_| reader.get_mut().flush())
        .map_err(io_error)?;

    let mut response = String::new();
    reader.read_line(&mut response).map_err(io_error)?;
    if response.trim().is_empty() {
        return Err(RtasksError::Protocol(
            "empty response from rtasks daemon".to_string(),
        ));
    }

    parse_response_json_line(&response)
}

fn open_pipe_with_timeout(timeout: Duration) -> Result<std::fs::File, RtasksError> {
    let started = Instant::now();
    loop {
        match OpenOptions::new()
            .read(true)
            .write(true)
            .open(RTASKS_PIPE_PATH)
        {
            Ok(file) => return Ok(file),
            Err(error) => {
                if started.elapsed() >= timeout {
                    return Err(RtasksError::Timeout(format!(
                        "rtasks daemon pipe was not reachable within {}ms: {error}",
                        timeout.as_millis()
                    )));
                }
                sleep(RETRY_DELAY);
            }
        }
    }
}

fn start_rtasks_daemon(exe_path: &Path) -> Result<(), RtasksError> {
    let mut command = Command::new(exe_path);
    command
        .args(["daemon", "--no-hotkeys"])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    #[cfg(windows)]
    command.creation_flags(CREATE_NO_WINDOW);

    command.spawn().map(|_| ()).map_err(|error| {
        RtasksError::Io(format!(
            "failed to start rtasks daemon '{}': {error}",
            exe_path.display()
        ))
    })
}

fn io_error(error: io::Error) -> RtasksError {
    RtasksError::Io(error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_json_matches_rtasks_protocol() {
        assert_eq!(
            request_json_line(RtasksCommand::Panel).expect("serialize"),
            "{\"cmd\":\"panel\"}\n"
        );
        assert_eq!(
            request_json_line(RtasksCommand::Shutdown).expect("serialize"),
            "{\"cmd\":\"shutdown\"}\n"
        );
        assert_eq!(
            request_json_line_for(RtasksIpcRequest {
                cmd: RtasksCommand::AddTask,
                input: Some("comprar pan".to_string()),
                status: Some(RtasksTaskStatus::Todo),
                priority: Some(RtasksPriority::High),
            })
            .expect("serialize"),
            "{\"cmd\":\"add_task\",\"input\":\"comprar pan\",\"status\":\"Todo\",\"priority\":\"High\"}\n"
        );
    }

    #[test]
    fn companion_path_uses_data_dir_layout() {
        assert_eq!(
            companion_rtasks_path_from_data_dir(Some("C:\\rMenuData")),
            PathBuf::from("C:\\rMenuData\\companions\\rtasks\\rtasks.exe")
        );
    }
}
