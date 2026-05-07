use std::env;
use std::ffi::OsStr;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};
use std::thread;
use std::time::{Duration, Instant};

use sha2::{Digest, Sha256};

#[cfg(windows)]
use std::os::windows::process::CommandExt;
#[cfg(windows)]
use windows::Win32::System::Threading::CREATE_NO_WINDOW;

#[derive(Debug, Clone, PartialEq, Eq)]
struct InstallArgs {
    version: String,
    release_url: String,
    installer_url: String,
    checksums_url: String,
    data_dir: Option<String>,
    dry_run: bool,
    no_quit: bool,
    no_restart: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum UpdaterCommand {
    Install(InstallArgs),
    Help,
}

#[derive(Debug)]
enum UpdaterError {
    Usage(String),
    Io(String),
    Download(String),
    Checksum(String),
    Process(String),
}

impl UpdaterError {
    fn message(&self) -> String {
        match self {
            Self::Usage(message)
            | Self::Io(message)
            | Self::Download(message)
            | Self::Checksum(message)
            | Self::Process(message) => message.clone(),
        }
    }
}

fn main() -> ExitCode {
    match run(env::args().skip(1).collect()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("rmenu-updater: {}", error.message());
            ExitCode::from(1)
        }
    }
}

fn run(args: Vec<String>) -> Result<(), UpdaterError> {
    match parse_args(&args)? {
        UpdaterCommand::Help => {
            print_help();
            Ok(())
        }
        UpdaterCommand::Install(args) => run_install(args),
    }
}

fn parse_args(args: &[String]) -> Result<UpdaterCommand, UpdaterError> {
    if args.is_empty() || args.iter().any(|arg| arg == "--help" || arg == "-h") {
        return Ok(UpdaterCommand::Help);
    }
    if args[0] != "install" {
        return Err(UpdaterError::Usage(format!(
            "unknown command '{}'; expected 'install'",
            args[0]
        )));
    }

    let mut version = None;
    let mut release_url = None;
    let mut installer_url = None;
    let mut checksums_url = None;
    let mut data_dir = None;
    let mut dry_run = false;
    let mut no_quit = false;
    let mut no_restart = false;

    let mut index = 1;
    while index < args.len() {
        match args[index].as_str() {
            "--version" => version = Some(take_value(args, &mut index, "--version")?),
            "--release-url" => release_url = Some(take_value(args, &mut index, "--release-url")?),
            "--installer-url" => {
                installer_url = Some(take_value(args, &mut index, "--installer-url")?)
            }
            "--checksums-url" => {
                checksums_url = Some(take_value(args, &mut index, "--checksums-url")?)
            }
            "--data-dir" => data_dir = Some(take_value(args, &mut index, "--data-dir")?),
            "--dry-run" => dry_run = true,
            "--no-quit" => no_quit = true,
            "--no-restart" => no_restart = true,
            flag => return Err(UpdaterError::Usage(format!("unknown argument '{flag}'"))),
        }
        index += 1;
    }

    Ok(UpdaterCommand::Install(InstallArgs {
        version: required(version, "--version")?,
        release_url: required(release_url, "--release-url")?,
        installer_url: required(installer_url, "--installer-url")?,
        checksums_url: required(checksums_url, "--checksums-url")?,
        data_dir,
        dry_run,
        no_quit,
        no_restart,
    }))
}

fn take_value(args: &[String], index: &mut usize, flag: &str) -> Result<String, UpdaterError> {
    *index += 1;
    args.get(*index)
        .filter(|value| !value.starts_with("--"))
        .cloned()
        .ok_or_else(|| UpdaterError::Usage(format!("missing value for {flag}")))
}

fn required(value: Option<String>, flag: &str) -> Result<String, UpdaterError> {
    value.ok_or_else(|| UpdaterError::Usage(format!("missing required {flag}")))
}

fn run_install(args: InstallArgs) -> Result<(), UpdaterError> {
    let log_path = updater_log_path(args.data_dir.as_deref())?;
    log_line(&log_path, format!("starting update to {}", args.version))?;
    log_line(&log_path, format!("release: {}", args.release_url))?;

    let downloads_dir = rmenu_state_dir(args.data_dir.as_deref())
        .join("updates")
        .join("downloads");
    fs::create_dir_all(&downloads_dir).map_err(io_error)?;

    let installer_name = asset_file_name(&args.installer_url, &args.version)?;
    let checksums_path = downloads_dir.join(format!("SHA256SUMS-v{}.txt", args.version));
    let installer_path = downloads_dir.join(installer_name);

    download_to_path(&args.checksums_url, &checksums_path)?;
    download_to_path(&args.installer_url, &installer_path)?;

    let checksums = fs::read_to_string(&checksums_path).map_err(io_error)?;
    let expected = expected_sha256_for_file(&checksums, &installer_path)?;
    let actual = sha256_file(&installer_path)?;
    if !expected.eq_ignore_ascii_case(&actual) {
        log_line(
            &log_path,
            format!("hash mismatch: expected {expected}, actual {actual}"),
        )?;
        return Err(UpdaterError::Checksum(format!(
            "hash mismatch for {}",
            installer_path.display()
        )));
    }
    log_line(&log_path, format!("verified sha256 {actual}"))?;

    if args.dry_run {
        log_line(
            &log_path,
            format!("dry-run: would run installer {}", installer_path.display()),
        )?;
        return Ok(());
    }

    if !args.no_quit {
        request_daemon_quit(&log_path)?;
        wait_for_processes(
            &["rmenu.exe", "rmenu-daemon.exe"],
            Duration::from_secs(10),
            &log_path,
        )?;
    }

    run_installer(&installer_path, &log_path)?;

    if !args.no_restart {
        restart_daemon(args.data_dir.as_deref(), &log_path)?;
    }

    log_line(&log_path, "update completed")?;
    Ok(())
}

fn updater_log_path(cli_data_dir: Option<&str>) -> Result<PathBuf, UpdaterError> {
    let dir = rmenu_state_dir(cli_data_dir).join("updates");
    fs::create_dir_all(&dir).map_err(io_error)?;
    Ok(dir.join("updater.log"))
}

fn rmenu_state_dir(cli_data_dir: Option<&str>) -> PathBuf {
    if let Some(data_dir) = cli_data_dir.filter(|value| !value.trim().is_empty()) {
        return PathBuf::from(data_dir).join("state");
    }
    #[cfg(windows)]
    {
        PathBuf::from(r"C:\rMenuData").join("state")
    }
    #[cfg(not(windows))]
    {
        dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("rmenu")
            .join("state")
    }
}

fn log_line(path: &Path, message: impl AsRef<str>) -> Result<(), UpdaterError> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(io_error)?;
    writeln!(file, "{}", message.as_ref()).map_err(io_error)
}

fn asset_file_name(url: &str, version: &str) -> Result<String, UpdaterError> {
    let fallback = format!("rmenu-setup-v{version}.exe");
    let raw_name = url
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or_default()
        .split('?')
        .next()
        .unwrap_or_default()
        .trim();
    let name = if raw_name.is_empty() {
        fallback.as_str()
    } else {
        raw_name
    };
    if !name.to_ascii_lowercase().ends_with(".exe") || name.contains("..") {
        return Err(UpdaterError::Download(format!(
            "invalid installer asset name: {name}"
        )));
    }
    Ok(name.to_string())
}

fn download_to_path(url: &str, path: &Path) -> Result<(), UpdaterError> {
    if let Some(source) = url.strip_prefix("file://") {
        fs::copy(file_url_path(source), path)
            .map(|_| ())
            .map_err(io_error)
    } else {
        download_http_to_path(url, path)
    }
}

#[cfg(windows)]
fn download_http_to_path(url: &str, path: &Path) -> Result<(), UpdaterError> {
    use windows::core::PCWSTR;
    use windows::Win32::System::Com::Urlmon::URLDownloadToFileW;

    if !url.starts_with("https://") && !url.starts_with("http://") {
        return Err(UpdaterError::Download(format!(
            "unsupported URL scheme: {url}"
        )));
    }
    let wide_url = wide_null(url);
    let wide_path = wide_null(&path.to_string_lossy());
    unsafe {
        URLDownloadToFileW(
            None,
            PCWSTR(wide_url.as_ptr()),
            PCWSTR(wide_path.as_ptr()),
            0,
            None,
        )
    }
    .map_err(|error| UpdaterError::Download(format!("download failed: {error}")))
}

#[cfg(not(windows))]
fn download_http_to_path(url: &str, _path: &Path) -> Result<(), UpdaterError> {
    Err(UpdaterError::Download(format!(
        "HTTP downloads are only implemented on Windows: {url}"
    )))
}

fn file_url_path(raw_path: &str) -> PathBuf {
    #[cfg(windows)]
    {
        let normalized = raw_path.replace('/', "\\");
        let bytes = normalized.as_bytes();
        if normalized.starts_with('\\')
            && bytes.len() >= 4
            && bytes[2] == b':'
            && bytes[1].is_ascii_alphabetic()
        {
            return PathBuf::from(&normalized[1..]);
        }
        PathBuf::from(normalized)
    }
    #[cfg(not(windows))]
    {
        PathBuf::from(raw_path)
    }
}

fn expected_sha256_for_file(content: &str, installer_path: &Path) -> Result<String, UpdaterError> {
    let file_name = installer_path
        .file_name()
        .and_then(OsStr::to_str)
        .ok_or_else(|| UpdaterError::Checksum("installer path has no file name".to_string()))?;
    for line in content.lines() {
        let trimmed = line.trim().trim_start_matches('\u{feff}');
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let mut parts = trimmed.split_whitespace();
        let Some(hash) = parts.next() else { continue };
        let Some(path) = parts.next() else { continue };
        let normalized = path.replace('\\', "/");
        if normalized.eq_ignore_ascii_case(file_name)
            || normalized.ends_with(&format!("/{file_name}"))
        {
            if hash.len() == 64 && hash.chars().all(|ch| ch.is_ascii_hexdigit()) {
                return Ok(hash.to_string());
            }
            return Err(UpdaterError::Checksum(format!(
                "invalid SHA256 for {file_name}"
            )));
        }
    }
    Err(UpdaterError::Checksum(format!(
        "no checksum entry found for {file_name}"
    )))
}

fn sha256_file(path: &Path) -> Result<String, UpdaterError> {
    let bytes = fs::read(path).map_err(io_error)?;
    let digest = Sha256::digest(bytes);
    Ok(format!("{digest:x}"))
}

fn request_daemon_quit(log_path: &Path) -> Result<(), UpdaterError> {
    let daemon = current_exe_sibling("rmenu-daemon.exe")?;
    if !daemon.exists() {
        log_line(log_path, format!("daemon not found: {}", daemon.display()))?;
        return Ok(());
    }
    log_line(log_path, "requesting daemon quit")?;
    hidden_command(Command::new(daemon).arg("--quit"))
        .status()
        .map_err(process_error)?;
    Ok(())
}

fn run_installer(installer_path: &Path, log_path: &Path) -> Result<(), UpdaterError> {
    log_line(
        log_path,
        format!("running installer {}", installer_path.display()),
    )?;
    Command::new(installer_path)
        .arg("/NORESTART")
        .spawn()
        .map_err(process_error)?
        .wait()
        .map_err(process_error)?;
    Ok(())
}

fn restart_daemon(cli_data_dir: Option<&str>, log_path: &Path) -> Result<(), UpdaterError> {
    let daemon = current_exe_sibling("rmenu-daemon.exe")?;
    let rmenu = current_exe_sibling("rmenu.exe")?;
    if !daemon.exists() || !rmenu.exists() {
        log_line(log_path, "restart skipped: installed binaries not found")?;
        return Ok(());
    }
    let mut command = Command::new(daemon);
    command
        .arg("--hotkey")
        .arg("ctrl+shift+space")
        .arg("--rmenu")
        .arg(rmenu);
    if let Some(data_dir) = cli_data_dir {
        command.arg("--data-dir").arg(data_dir);
    }
    hidden_command(&mut command)
        .spawn()
        .map_err(process_error)?;
    Ok(())
}

fn current_exe_sibling(file_name: &str) -> Result<PathBuf, UpdaterError> {
    let exe = env::current_exe().map_err(io_error)?;
    let dir = exe.parent().ok_or_else(|| {
        UpdaterError::Process("could not resolve executable directory".to_string())
    })?;
    Ok(dir.join(file_name))
}

#[cfg(windows)]
fn hidden_command(command: &mut Command) -> &mut Command {
    command.creation_flags(CREATE_NO_WINDOW.0)
}

#[cfg(not(windows))]
fn hidden_command(command: &mut Command) -> &mut Command {
    command
}

fn wait_for_processes(
    process_names: &[&str],
    timeout: Duration,
    log_path: &Path,
) -> Result<(), UpdaterError> {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        let running = running_processes(process_names)?;
        if running.is_empty() {
            return Ok(());
        }
        thread::sleep(Duration::from_millis(250));
    }
    log_line(
        log_path,
        "process wait timed out; continuing installer launch",
    )?;
    Ok(())
}

#[cfg(windows)]
fn running_processes(process_names: &[&str]) -> Result<Vec<String>, UpdaterError> {
    let mut command = Command::new("tasklist");
    let output = hidden_command(&mut command)
        .output()
        .map_err(process_error)?;
    let text = String::from_utf8_lossy(&output.stdout).to_ascii_lowercase();
    Ok(process_names
        .iter()
        .filter(|name| text.contains(&name.to_ascii_lowercase()))
        .map(|name| (*name).to_string())
        .collect())
}

#[cfg(not(windows))]
fn running_processes(_process_names: &[&str]) -> Result<Vec<String>, UpdaterError> {
    Ok(Vec::new())
}

#[cfg(windows)]
fn wide_null(value: &str) -> Vec<u16> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    OsStr::new(value).encode_wide().chain(Some(0)).collect()
}

fn io_error(error: io::Error) -> UpdaterError {
    UpdaterError::Io(error.to_string())
}

fn process_error(error: io::Error) -> UpdaterError {
    UpdaterError::Process(error.to_string())
}

fn print_help() {
    println!(
        "rmenu-updater\n\nUsage:\n  rmenu-updater install --version <VERSION> --release-url <URL> --installer-url <URL> --checksums-url <URL> [--data-dir <PATH>] [--dry-run]\n"
    );
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;
    use std::process::Command;

    use super::{expected_sha256_for_file, parse_args, sha256_file, UpdaterCommand};

    #[test]
    fn parse_install_args_requires_update_metadata() {
        let args = vec![
            "install".to_string(),
            "--version".to_string(),
            "0.3.1".to_string(),
            "--release-url".to_string(),
            "https://example.test/release".to_string(),
            "--installer-url".to_string(),
            "file:///tmp/rmenu-setup-v0.3.1.exe".to_string(),
            "--checksums-url".to_string(),
            "file:///tmp/SHA256SUMS.txt".to_string(),
            "--dry-run".to_string(),
        ];

        let parsed = parse_args(&args).expect("parse args");
        let UpdaterCommand::Install(args) = parsed else {
            panic!("expected install command")
        };
        assert_eq!(args.version, "0.3.1");
        assert!(args.dry_run);
    }

    #[test]
    fn checksum_parser_matches_nested_installer_path() {
        let content = "abcdabcdabcdabcdabcdabcdabcdabcdabcdabcdabcdabcdabcdabcdabcdabcd  installers/rmenu-setup-v0.3.1.exe\n";
        let expected = expected_sha256_for_file(
            content,
            Path::new("C:/tmp/downloads/rmenu-setup-v0.3.1.exe"),
        )
        .expect("checksum");
        assert_eq!(
            expected,
            "abcdabcdabcdabcdabcdabcdabcdabcdabcdabcdabcdabcdabcdabcdabcdabcd"
        );
    }

    #[test]
    fn dry_run_downloads_and_verifies_file_url_fixture() {
        let temp = std::env::temp_dir().join(format!("rmenu-updater-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp);
        fs::create_dir_all(&temp).expect("temp dir");
        let installer = temp.join("rmenu-setup-v9.9.9.exe");
        fs::write(&installer, b"fake installer bytes").expect("installer fixture");
        let hash = sha256_file(&installer).expect("hash");
        let sums = temp.join("SHA256SUMS.txt");
        fs::write(
            &sums,
            format!("{hash}  installers/rmenu-setup-v9.9.9.exe\n"),
        )
        .expect("checksum fixture");
        let data_dir = temp.join("data");

        let status = Command::new(std::env::current_exe().expect("test exe"))
            .arg("tests::updater_dry_run_child")
            .arg("--exact")
            .env(
                "RMENU_UPDATER_TEST_INSTALLER",
                installer.to_string_lossy().to_string(),
            )
            .env(
                "RMENU_UPDATER_TEST_SUMS",
                sums.to_string_lossy().to_string(),
            )
            .env(
                "RMENU_UPDATER_TEST_DATA_DIR",
                data_dir.to_string_lossy().to_string(),
            )
            .status()
            .expect("child status");
        assert!(status.success());
        assert!(data_dir.join("state/updates/updater.log").exists());
        let _ = fs::remove_dir_all(temp);
    }

    #[test]
    fn updater_dry_run_child() {
        if std::env::var("RMENU_UPDATER_TEST_INSTALLER").is_err() {
            return;
        }
        let installer = std::env::var("RMENU_UPDATER_TEST_INSTALLER").expect("installer env");
        let sums = std::env::var("RMENU_UPDATER_TEST_SUMS").expect("sums env");
        let data_dir = std::env::var("RMENU_UPDATER_TEST_DATA_DIR").expect("data env");
        super::run(vec![
            "install".to_string(),
            "--version".to_string(),
            "9.9.9".to_string(),
            "--release-url".to_string(),
            "https://example.test/release".to_string(),
            "--installer-url".to_string(),
            format!("file://{installer}"),
            "--checksums-url".to_string(),
            format!("file://{sums}"),
            "--data-dir".to_string(),
            data_dir,
            "--dry-run".to_string(),
        ])
        .expect("dry run");
        std::process::exit(0);
    }
}
