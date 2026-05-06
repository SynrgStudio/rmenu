#![windows_subsystem = "windows"]
#![allow(dead_code)]

#[cfg(not(test))]
mod app_state;
#[cfg(not(test))]
mod fuzzy;
#[cfg(not(test))]
mod launcher;
#[cfg(not(test))]
mod modules;
#[cfg(not(test))]
mod ranking;
#[cfg(not(test))]
mod rmods_registry;
#[cfg(not(test))]
mod rsnip_companion;
mod rtasks_companion;
mod settings;
#[cfg(not(test))]
mod sources;
#[cfg(not(test))]
mod ui_win32;

use std::env;
use std::ffi::OsStr;
use std::fs::OpenOptions;
use std::io::{self, Write};
use std::os::windows::ffi::OsStrExt;
#[cfg(not(test))]
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
#[cfg(not(test))]
use std::process::Command;
#[cfg(not(test))]
use std::time::{Duration, Instant};

#[cfg(not(test))]
use app_state::{AppState, LauncherItem};
#[cfg(not(test))]
use rsnip_companion::{RsnipCommand, RsnipCompanion};
#[cfg(not(test))]
use rtasks_companion::{RtasksCommand, RtasksCompanion};
#[cfg(not(test))]
use settings::{CmdOptions, RmenuConfig};
#[cfg(not(test))]
use sources::load_launcher_items;
use windows::core::PCWSTR;
#[cfg(not(test))]
use windows::Win32::Foundation::{GetLastError, ERROR_ALREADY_EXISTS};
use windows::Win32::Foundation::{ERROR_SUCCESS, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::Registry::{
    RegCloseKey, RegCreateKeyExW, RegDeleteValueW, RegSetValueExW, HKEY, HKEY_CURRENT_USER,
    KEY_SET_VALUE, REG_OPTION_NON_VOLATILE, REG_SZ,
};
#[cfg(not(test))]
use windows::Win32::System::Threading::CreateMutexW;
#[cfg(not(test))]
use windows::Win32::UI::Input::KeyboardAndMouse::RegisterHotKey;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    HOT_KEY_MODIFIERS, MOD_ALT, MOD_CONTROL, MOD_SHIFT, MOD_WIN, VK_F1, VK_F10, VK_F11, VK_F12,
    VK_F2, VK_F3, VK_F4, VK_F5, VK_F6, VK_F7, VK_F8, VK_F9, VK_SPACE,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, FindWindowExW, FindWindowW, PostMessageW,
    PostQuitMessage, RegisterClassW, HWND_MESSAGE, WINDOW_EX_STYLE, WINDOW_STYLE, WM_CLOSE,
    WM_DESTROY, WNDCLASSW,
};
#[cfg(not(test))]
use windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, GetForegroundWindow, GetMessageW, IsWindow, SetForegroundWindow,
    TranslateMessage, MSG, WM_HOTKEY,
};

const DAEMON_CLASS_NAME: &str = "rmenu_daemon_window";
const DAEMON_MUTEX_NAME: &str = "Local\\rmenu-daemon";
const HOTKEY_ID: i32 = 1;
const RTASKS_PANEL_HOTKEY_ID: i32 = 2;
const RUN_KEY_PATH: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";
const RUN_VALUE_NAME: &str = "rmenu-daemon";
#[cfg(not(test))]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

#[derive(Clone, Debug, PartialEq, Eq)]
struct DaemonOptions {
    hotkey: String,
    rmenu_path: Option<PathBuf>,
    modules_dir: Option<PathBuf>,
    data_dir: Option<PathBuf>,
    install_startup: bool,
    uninstall_startup: bool,
    quit: bool,
    help: bool,
}

impl Default for DaemonOptions {
    fn default() -> Self {
        Self {
            hotkey: "ctrl+shift+space".to_string(),
            rmenu_path: None,
            modules_dir: None,
            data_dir: None,
            install_startup: false,
            uninstall_startup: false,
            quit: false,
            help: false,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ParsedHotkey {
    modifiers: HOT_KEY_MODIFIERS,
    vk: u32,
}

#[cfg(not(test))]
struct PreparedRmenu {
    cmd_options: CmdOptions,
    config: RmenuConfig,
    modules_dir: PathBuf,
    launcher_items: Vec<LauncherItem>,
}

fn to_wstring(value: &str) -> Vec<u16> {
    OsStr::new(value).encode_wide().chain(Some(0)).collect()
}

fn utf16_bytes_with_nul(value: &str) -> Vec<u8> {
    let wide = to_wstring(value);
    let mut bytes = Vec::with_capacity(wide.len() * 2);
    for unit in wide {
        bytes.extend_from_slice(&unit.to_le_bytes());
    }
    bytes
}

fn quote_arg(value: &Path) -> String {
    format!("\"{}\"", value.display())
}

fn quote_string_arg(value: &str) -> String {
    format!("\"{}\"", value.replace('"', "\\\""))
}

fn log_path() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(env::temp_dir)
        .join("rmenu")
        .join("rmenu-daemon.log")
}

fn log_line(message: &str) {
    let path = log_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
        let _ = writeln!(file, "{message}");
    }
}

fn parse_args_from<I>(args: I) -> DaemonOptions
where
    I: IntoIterator<Item = String>,
{
    let mut options = DaemonOptions::default();
    let args: Vec<String> = args.into_iter().collect();
    let mut i = 1;

    while i < args.len() {
        match args[i].as_str() {
            "--hotkey" => {
                if i + 1 < args.len() {
                    options.hotkey = args[i + 1].clone();
                    i += 1;
                }
            }
            "--rmenu" => {
                if i + 1 < args.len() {
                    options.rmenu_path = Some(PathBuf::from(&args[i + 1]));
                    i += 1;
                }
            }
            "--modules-dir" => {
                if i + 1 < args.len() {
                    options.modules_dir = Some(PathBuf::from(&args[i + 1]));
                    i += 1;
                }
            }
            "--data-dir" => {
                if i + 1 < args.len() {
                    options.data_dir = Some(PathBuf::from(&args[i + 1]));
                    i += 1;
                }
            }
            "--install-startup" => options.install_startup = true,
            "--uninstall-startup" => options.uninstall_startup = true,
            "--quit" => options.quit = true,
            "-h" | "--help" => options.help = true,
            _ => {}
        }
        i += 1;
    }

    options
}

fn effective_rmenu_path(options: &DaemonOptions) -> PathBuf {
    if let Some(path) = &options.rmenu_path {
        return path.clone();
    }

    env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(|parent| parent.join("rmenu.exe")))
        .unwrap_or_else(|| PathBuf::from("rmenu.exe"))
}

fn effective_modules_dir(options: &DaemonOptions) -> PathBuf {
    settings::resolve_modules_dir(
        options.modules_dir.as_ref().and_then(|path| path.to_str()),
        options.data_dir.as_ref().and_then(|path| path.to_str()),
    )
}

fn parse_hotkey(value: &str) -> Result<ParsedHotkey, String> {
    let mut modifiers = HOT_KEY_MODIFIERS(0);
    let mut key: Option<u32> = None;

    for token in value
        .split('+')
        .map(|part| part.trim().to_ascii_lowercase())
        .filter(|part| !part.is_empty())
    {
        match token.as_str() {
            "alt" => modifiers.0 |= MOD_ALT.0,
            "ctrl" | "control" => modifiers.0 |= MOD_CONTROL.0,
            "shift" => modifiers.0 |= MOD_SHIFT.0,
            "win" | "windows" | "super" => modifiers.0 |= MOD_WIN.0,
            "space" | "spacebar" => key = Some(VK_SPACE.0 as u32),
            "f1" => key = Some(VK_F1.0 as u32),
            "f2" => key = Some(VK_F2.0 as u32),
            "f3" => key = Some(VK_F3.0 as u32),
            "f4" => key = Some(VK_F4.0 as u32),
            "f5" => key = Some(VK_F5.0 as u32),
            "f6" => key = Some(VK_F6.0 as u32),
            "f7" => key = Some(VK_F7.0 as u32),
            "f8" => key = Some(VK_F8.0 as u32),
            "f9" => key = Some(VK_F9.0 as u32),
            "f10" => key = Some(VK_F10.0 as u32),
            "f11" => key = Some(VK_F11.0 as u32),
            "f12" => key = Some(VK_F12.0 as u32),
            _ => {
                if token.len() == 1 {
                    let ch = token.chars().next().expect("single-char token");
                    if ch.is_ascii_alphanumeric() {
                        key = Some(ch.to_ascii_uppercase() as u32);
                    } else {
                        return Err(format!("unsupported hotkey key: {token}"));
                    }
                } else {
                    return Err(format!("unsupported hotkey token: {token}"));
                }
            }
        }
    }

    let vk = key.ok_or_else(|| "hotkey is missing a key".to_string())?;
    if modifiers.0 == 0 {
        return Err("hotkey must include at least one modifier".to_string());
    }

    Ok(ParsedHotkey { modifiers, vk })
}

fn build_startup_command(options: &DaemonOptions) -> io::Result<String> {
    let daemon_path = env::current_exe()?;
    let rmenu_path = effective_rmenu_path(options);
    let modules_dir = effective_modules_dir(options);
    let mut command = quote_arg(&daemon_path);
    command.push_str(" --hotkey ");
    command.push_str(&quote_string_arg(&options.hotkey));
    command.push_str(" --rmenu ");
    command.push_str(&quote_arg(&rmenu_path));
    command.push_str(" --modules-dir ");
    command.push_str(&quote_arg(&modules_dir));
    if let Some(data_dir) = &options.data_dir {
        command.push_str(" --data-dir ");
        command.push_str(&quote_arg(data_dir));
    }
    Ok(command)
}

fn open_run_key() -> Result<HKEY, String> {
    let subkey = to_wstring(RUN_KEY_PATH);
    let mut key = HKEY::default();
    let status = unsafe {
        RegCreateKeyExW(
            HKEY_CURRENT_USER,
            PCWSTR(subkey.as_ptr()),
            0,
            PCWSTR::null(),
            REG_OPTION_NON_VOLATILE,
            KEY_SET_VALUE,
            None,
            &mut key,
            None,
        )
    };

    if status != ERROR_SUCCESS {
        return Err(format!("RegCreateKeyExW failed: {}", status.0));
    }

    Ok(key)
}

fn install_startup(options: &DaemonOptions) -> Result<(), String> {
    let command = build_startup_command(options).map_err(|err| err.to_string())?;
    let key = open_run_key()?;
    let value_name = to_wstring(RUN_VALUE_NAME);
    let data = utf16_bytes_with_nul(&command);
    let status =
        unsafe { RegSetValueExW(key, PCWSTR(value_name.as_ptr()), 0, REG_SZ, Some(&data)) };
    let _ = unsafe { RegCloseKey(key) };

    if status != ERROR_SUCCESS {
        return Err(format!("RegSetValueExW failed: {}", status.0));
    }

    log_line(&format!("installed startup: {command}"));
    Ok(())
}

fn uninstall_startup() -> Result<(), String> {
    let key = open_run_key()?;
    let value_name = to_wstring(RUN_VALUE_NAME);
    let status = unsafe { RegDeleteValueW(key, PCWSTR(value_name.as_ptr())) };
    let _ = unsafe { RegCloseKey(key) };

    if status != ERROR_SUCCESS {
        return Err(format!("RegDeleteValueW failed: {}", status.0));
    }

    log_line("uninstalled startup");
    Ok(())
}

fn request_quit() -> Result<(), String> {
    let class_name = to_wstring(DAEMON_CLASS_NAME);
    let mut hwnd = unsafe { FindWindowW(PCWSTR(class_name.as_ptr()), PCWSTR::null()) };
    if hwnd.0 == 0 {
        hwnd = unsafe {
            FindWindowExW(
                HWND_MESSAGE,
                HWND(0),
                PCWSTR(class_name.as_ptr()),
                PCWSTR::null(),
            )
        };
    }
    if hwnd.0 == 0 {
        log_line("quit requested but daemon window was not found");
        return Ok(());
    }

    let ok = unsafe { PostMessageW(hwnd, WM_CLOSE, WPARAM(0), LPARAM(0)) };
    if !ok.as_bool() {
        return Err("PostMessageW(WM_CLOSE) failed".to_string());
    }

    log_line("quit requested");
    Ok(())
}

#[cfg(not(test))]
fn configure_runtime(
    config: &RmenuConfig,
    modules_dir: &Path,
    silent_mode: bool,
) -> modules::ModuleRuntime {
    let mut runtime = modules::ModuleRuntime::new();
    runtime.configure_policy(modules::ModuleRuntimePolicy {
        provider_total_budget_ms: config.modules.provider_total_budget_ms,
        provider_timeout_ms: config.modules.provider_timeout_ms,
        max_items_per_provider_host: config.modules.max_items_per_provider_host,
        dedupe_source_priority: match config.modules.dedupe_source_priority {
            settings::DedupeSourcePriority::CoreFirst => modules::DedupeSourcePriority::CoreFirst,
            settings::DedupeSourcePriority::ProviderFirst => {
                modules::DedupeSourcePriority::ProviderFirst
            }
        },
        host_restart_backoff_ms: config.modules.host_restart_backoff_ms,
        max_ipc_payload_bytes: config.modules.max_ipc_payload_bytes,
    });
    runtime.register_builtin_module(Box::new(modules::BuiltinLifecycleModule));
    runtime.register_builtin_module(Box::new(modules::BuiltinQueryProviderModule));
    runtime.register_builtin_module(Box::new(modules::BuiltinRsnipCompanionModule));
    runtime.register_builtin_module(Box::new(modules::BuiltinRtasksCompanionModule));
    runtime.load_external_descriptors(modules_dir, silent_mode);
    runtime
}

#[cfg(not(test))]
fn prepare_rmenu(
    options: &DaemonOptions,
) -> Result<(PreparedRmenu, modules::ModuleRuntime), String> {
    let started = Instant::now();
    let modules_dir = effective_modules_dir(options);
    let mut cmd_options = CmdOptions {
        modules_dir: Some(modules_dir.display().to_string()),
        silent: true,
        ..Default::default()
    };
    let mut config = RmenuConfig::load(None).unwrap_or_else(|err| {
        log_line(&format!("config load failed, using defaults: {err}"));
        RmenuConfig::default()
    });
    config.apply_cli_overrides(&cmd_options);

    cmd_options.silent = true;
    let launcher_items = load_launcher_items(&config.launcher, true, false);
    let runtime = configure_runtime(&config, &modules_dir, true);
    let prepared = PreparedRmenu {
        cmd_options,
        config,
        modules_dir,
        launcher_items,
    };

    log_line(&format!(
        "prewarmed rmenu modules={} items={} elapsed_ms={}",
        prepared.modules_dir.display(),
        prepared.launcher_items.len(),
        started.elapsed().as_millis()
    ));

    Ok((prepared, runtime))
}

#[cfg(not(test))]
fn initial_app_state(prepared: &PreparedRmenu) -> AppState {
    AppState {
        current_input: String::new(),
        selected_index: 0,
        scroll_offset: 0,
        matching_items: Vec::new(),
        all_items: prepared.launcher_items.clone(),
        prompt: prepared.cmd_options.prompt.clone(),
        launcher_mode: true,
        silent_mode: prepared.cmd_options.silent,
        history_max_items: prepared.config.launcher.history_max_items,
        source_boost_history: prepared.config.launcher.source_boost_history,
        source_boost_start_menu: prepared.config.launcher.source_boost_start_menu,
        source_boost_path: prepared.config.launcher.source_boost_path,
        rtasks_status: None,
        rtasks_priority: None,
        rmods: Default::default(),
    }
}

#[cfg(not(test))]
fn show_warm_rmenu(
    prepared: &PreparedRmenu,
    runtime: modules::ModuleRuntime,
) -> modules::ModuleRuntime {
    let started = Instant::now();
    let app_state = initial_app_state(prepared);
    match ui_win32::run_ui_embedded(&prepared.cmd_options, &prepared.config, app_state, runtime) {
        Ok(exit_code) => {
            log_line(&format!(
                "rmenu closed exit_code={} elapsed_ms={}",
                exit_code,
                started.elapsed().as_millis()
            ));
        }
        Err(err) => {
            log_line(&format!("rmenu ui error: {err}"));
        }
    }

    ui_win32::take_module_runtime()
        .unwrap_or_else(|| configure_runtime(&prepared.config, &prepared.modules_dir, true))
}

unsafe extern "system" fn daemon_window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_CLOSE => {
            DestroyWindow(hwnd);
            LRESULT(0)
        }
        WM_DESTROY => {
            PostQuitMessage(0);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

#[cfg(not(test))]
struct ActiveRsnipCompanion {
    companion: RsnipCompanion,
    started_by_rmenu: bool,
}

#[cfg(not(test))]
struct ActiveRtasksCompanion {
    companion: RtasksCompanion,
    started_by_rmenu: bool,
}

#[cfg(not(test))]
fn start_rsnip_daemon_if_available() -> Option<ActiveRsnipCompanion> {
    match RsnipCompanion::discover() {
        Ok(companion) => {
            let was_running = companion
                .ping(std::time::Duration::from_millis(100))
                .is_ok();
            match companion.ensure_daemon() {
                Ok(()) => {
                    log_line(&format!(
                        "started/confirmed rsnip daemon: {} owner={}",
                        companion.exe_path.display(),
                        if was_running { "rsnip" } else { "rmenu" }
                    ));
                    Some(ActiveRsnipCompanion {
                        companion,
                        started_by_rmenu: !was_running,
                    })
                }
                Err(err) => {
                    log_line(&format!(
                        "failed to start/confirm rsnip daemon {}: {err:?}",
                        companion.exe_path.display()
                    ));
                    None
                }
            }
        }
        Err(err) => {
            log_line(&format!("rsnip not available: {err:?}"));
            None
        }
    }
}

#[cfg(not(test))]
fn start_rtasks_daemon_if_available() -> Option<ActiveRtasksCompanion> {
    match RtasksCompanion::discover() {
        Ok(companion) => {
            let was_running = companion.ping(Duration::from_millis(100)).is_ok();
            match companion.ensure_daemon() {
                Ok(()) => {
                    log_line(&format!(
                        "started/confirmed rtasks daemon: {} owner={}",
                        companion.exe_path.display(),
                        if was_running { "rtasks" } else { "rmenu" }
                    ));
                    Some(ActiveRtasksCompanion {
                        companion,
                        started_by_rmenu: !was_running,
                    })
                }
                Err(err) => {
                    log_line(&format!(
                        "failed to start/confirm rtasks daemon {}: {err:?}",
                        companion.exe_path.display()
                    ));
                    None
                }
            }
        }
        Err(err) => {
            log_line(&format!("rtasks not available: {err:?}"));
            None
        }
    }
}

#[cfg(not(test))]
fn stop_rsnip_daemon(active: Option<ActiveRsnipCompanion>) {
    let (companion, started_by_rmenu) = match active {
        Some(active) => (active.companion, active.started_by_rmenu),
        None => match RsnipCompanion::discover() {
            Ok(companion) => (companion, false),
            Err(err) => {
                log_line(&format!(
                    "rsnip shutdown skipped, companion not discovered: {err:?}"
                ));
                return;
            }
        },
    };

    match companion.send(RsnipCommand::Shutdown) {
        Ok(response) => log_line(&format!(
            "requested rsnip daemon shutdown owner_started_by_rmenu={} response={response:?}",
            started_by_rmenu
        )),
        Err(err) => log_line(&format!("failed to request rsnip daemon shutdown: {err:?}")),
    }

    if wait_until_rsnip_stopped(&companion, Duration::from_millis(2_000)) {
        log_line("rsnip daemon stopped cleanly");
        return;
    }

    log_line("rsnip daemon still reachable after shutdown request; forcing rsnip.exe termination");
    force_kill_rsnip_processes();
}

#[cfg(not(test))]
fn wait_until_rsnip_stopped(companion: &RsnipCompanion, timeout: Duration) -> bool {
    let started = Instant::now();
    while started.elapsed() < timeout {
        if companion.ping(Duration::from_millis(100)).is_err() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    companion.ping(Duration::from_millis(100)).is_err()
}

#[cfg(not(test))]
fn stop_rtasks_daemon(active: Option<ActiveRtasksCompanion>) {
    let (companion, started_by_rmenu) = match active {
        Some(active) => (active.companion, active.started_by_rmenu),
        None => match RtasksCompanion::discover() {
            Ok(companion) => (companion, false),
            Err(err) => {
                log_line(&format!(
                    "rtasks shutdown skipped, companion not discovered: {err:?}"
                ));
                return;
            }
        },
    };

    match companion.send(RtasksCommand::Shutdown) {
        Ok(response) => log_line(&format!(
            "requested rtasks daemon shutdown owner_started_by_rmenu={} response={response:?}",
            started_by_rmenu
        )),
        Err(err) => log_line(&format!(
            "failed to request rtasks daemon shutdown: {err:?}"
        )),
    }

    if wait_until_rtasks_stopped(&companion, Duration::from_millis(2_000)) {
        log_line("rtasks daemon stopped cleanly");
        return;
    }

    log_line(
        "rtasks daemon still reachable after shutdown request; forcing rtasks.exe termination",
    );
    force_kill_processes("rtasks.exe");
}

#[cfg(not(test))]
fn wait_until_rtasks_stopped(companion: &RtasksCompanion, timeout: Duration) -> bool {
    let started = Instant::now();
    while started.elapsed() < timeout {
        if companion.ping(Duration::from_millis(100)).is_err() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    companion.ping(Duration::from_millis(100)).is_err()
}

#[cfg(not(test))]
fn force_kill_processes(image_name: &str) {
    match Command::new("taskkill")
        .args(["/IM", image_name, "/F"])
        .creation_flags(CREATE_NO_WINDOW)
        .status()
    {
        Ok(status) => log_line(&format!("taskkill {image_name} exit_status={status}")),
        Err(err) => log_line(&format!("failed to run taskkill for {image_name}: {err}")),
    }
}

#[cfg(not(test))]
fn force_kill_rsnip_processes() {
    force_kill_processes("rsnip.exe");
}

fn create_daemon_window() -> Result<HWND, String> {
    let class_name = to_wstring(DAEMON_CLASS_NAME);
    let instance = unsafe { GetModuleHandleW(None).map_err(|err| err.to_string())? };
    let wc = WNDCLASSW {
        lpfnWndProc: Some(daemon_window_proc),
        hInstance: instance.into(),
        lpszClassName: PCWSTR(class_name.as_ptr()),
        ..Default::default()
    };

    unsafe { RegisterClassW(&wc) };

    let hwnd = unsafe {
        CreateWindowExW(
            WINDOW_EX_STYLE(0),
            PCWSTR(class_name.as_ptr()),
            PCWSTR(class_name.as_ptr()),
            WINDOW_STYLE(0),
            0,
            0,
            0,
            0,
            HWND_MESSAGE,
            None,
            instance,
            None,
        )
    };

    if hwnd.0 == 0 {
        return Err("CreateWindowExW failed".to_string());
    }

    Ok(hwnd)
}

#[cfg(not(test))]
fn run_daemon(options: DaemonOptions) -> Result<(), String> {
    if let Some(data_dir) = &options.data_dir {
        env::set_var("RMENU_DATA_DIR", data_dir);
    }

    let mutex_name = to_wstring(DAEMON_MUTEX_NAME);
    let _mutex = unsafe { CreateMutexW(None, true, PCWSTR(mutex_name.as_ptr())) }
        .map_err(|err| err.to_string())?;
    if unsafe { GetLastError() } == ERROR_ALREADY_EXISTS {
        log_line("daemon already running");
        return Ok(());
    }

    let hotkey = parse_hotkey(&options.hotkey)?;
    let hwnd = create_daemon_window()?;
    let registered = unsafe { RegisterHotKey(hwnd, HOTKEY_ID, hotkey.modifiers, hotkey.vk) };
    if !registered.as_bool() {
        return Err(format!("failed to register hotkey {}", options.hotkey));
    }

    let active_rsnip = start_rsnip_daemon_if_available();
    let active_rtasks = start_rtasks_daemon_if_available();
    match parse_hotkey("ctrl+space") {
        Ok(rtasks_hotkey) => {
            let registered = unsafe {
                RegisterHotKey(
                    hwnd,
                    RTASKS_PANEL_HOTKEY_ID,
                    rtasks_hotkey.modifiers,
                    rtasks_hotkey.vk,
                )
            };
            if registered.as_bool() {
                log_line("registered RTasks panel hotkey ctrl+space");
            } else {
                log_line("failed to register RTasks panel hotkey ctrl+space");
            }
        }
        Err(err) => log_line(&format!("failed to parse RTasks panel hotkey: {err}")),
    }

    let (prepared, mut runtime) = prepare_rmenu(&options)?;
    log_line(&format!(
        "daemon started hotkey={} mode=resident-prewarmed modules_dir={} rmenu_arg={}",
        options.hotkey,
        prepared.modules_dir.display(),
        effective_rmenu_path(&options).display()
    ));

    let mut rtasks_panel_open = false;
    let mut rtasks_panel_restore_hwnd: Option<HWND> = None;
    let mut msg = MSG::default();
    loop {
        let result = unsafe { GetMessageW(&mut msg, None, 0, 0) };
        if result.0 <= 0 {
            break;
        }

        if msg.message == WM_HOTKEY && msg.wParam.0 == HOTKEY_ID as usize {
            runtime = show_warm_rmenu(&prepared, runtime);
        } else if msg.message == WM_HOTKEY && msg.wParam.0 == RTASKS_PANEL_HOTKEY_ID as usize {
            if let Ok(companion) = RtasksCompanion::discover() {
                if rtasks_panel_open {
                    match companion.ensure_and_send(RtasksCommand::Panel) {
                        Ok(_) => {
                            rtasks_panel_open = false;
                            if let Some(hwnd) = rtasks_panel_restore_hwnd.take() {
                                unsafe {
                                    if IsWindow(hwnd).as_bool() {
                                        let _ = SetForegroundWindow(hwnd);
                                    }
                                }
                            }
                        }
                        Err(err) => {
                            log_line(&format!(
                                "failed to close RTasks panel from hotkey: {err:?}"
                            ));
                        }
                    }
                } else {
                    let foreground = unsafe { GetForegroundWindow() };
                    rtasks_panel_restore_hwnd = if foreground.0 == 0 {
                        None
                    } else {
                        Some(foreground)
                    };
                    match companion.ensure_and_send(RtasksCommand::Panel) {
                        Ok(_) => rtasks_panel_open = true,
                        Err(err) => {
                            rtasks_panel_restore_hwnd = None;
                            log_line(&format!("failed to open RTasks panel from hotkey: {err:?}"));
                        }
                    }
                }
            }
        } else {
            unsafe {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
    }

    stop_rsnip_daemon(active_rsnip);
    stop_rtasks_daemon(active_rtasks);
    log_line("daemon stopped");
    Ok(())
}

fn print_help() {
    log_line(
        "usage: rmenu-daemon.exe [--hotkey ctrl+shift+space] [--rmenu PATH] [--modules-dir PATH] [--data-dir PATH] [--install-startup] [--uninstall-startup] [--quit]",
    );
}

#[cfg(not(test))]
fn main() {
    let options = parse_args_from(env::args());

    let result = if options.help {
        print_help();
        Ok(())
    } else if options.uninstall_startup {
        uninstall_startup()
    } else if options.install_startup {
        install_startup(&options)
    } else if options.quit {
        request_quit()
    } else {
        run_daemon(options)
    };

    if let Err(err) = result {
        log_line(&format!("error: {err}"));
    }
}

#[cfg(test)]
mod tests {
    use super::{build_startup_command, parse_args_from, parse_hotkey, DaemonOptions};
    use std::path::PathBuf;

    #[test]
    fn parse_args_accepts_daemon_options() {
        let options = parse_args_from([
            "rmenu-daemon".to_string(),
            "--hotkey".to_string(),
            "ctrl+space".to_string(),
            "--rmenu".to_string(),
            "C:\\rMenu\\rmenu.exe".to_string(),
            "--modules-dir".to_string(),
            "C:\\rMenu\\modules".to_string(),
            "--data-dir".to_string(),
            "C:\\rMenuData".to_string(),
            "--install-startup".to_string(),
        ]);

        assert_eq!(options.hotkey, "ctrl+space");
        assert_eq!(
            options.rmenu_path,
            Some(PathBuf::from("C:\\rMenu\\rmenu.exe"))
        );
        assert_eq!(
            options.modules_dir,
            Some(PathBuf::from("C:\\rMenu\\modules"))
        );
        assert_eq!(options.data_dir, Some(PathBuf::from("C:\\rMenuData")));
        assert!(options.install_startup);
    }

    #[test]
    fn default_hotkey_is_ctrl_shift_space() {
        let options = DaemonOptions::default();
        assert_eq!(options.hotkey, "ctrl+shift+space");
    }

    #[test]
    fn parse_hotkey_supports_alt_spacebar() {
        let hotkey = parse_hotkey("alt+spacebar").expect("valid hotkey");
        assert_ne!(hotkey.modifiers.0, 0);
        assert_eq!(hotkey.vk, 32);
    }

    #[test]
    fn parse_hotkey_supports_function_keys() {
        let hotkey = parse_hotkey("ctrl+alt+shift+f12").expect("valid hotkey");
        assert_ne!(hotkey.modifiers.0, 0);
        assert_eq!(hotkey.vk, 123);
    }

    #[test]
    fn parse_hotkey_rejects_missing_modifier() {
        assert!(parse_hotkey("space").is_err());
    }

    #[test]
    fn startup_command_persists_rmenu_and_modules_args() {
        let mut options = DaemonOptions::default();
        options.rmenu_path = Some(PathBuf::from("C:\\rMenu\\rmenu.exe"));
        options.modules_dir = Some(PathBuf::from("C:\\rMenu\\modules"));
        options.data_dir = Some(PathBuf::from("C:\\rMenuData"));

        let command = build_startup_command(&options).expect("startup command");

        assert!(command.contains("--hotkey \"ctrl+shift+space\""));
        assert!(command.contains("--rmenu \"C:\\rMenu\\rmenu.exe\""));
        assert!(command.contains("--modules-dir \"C:\\rMenu\\modules\""));
        assert!(command.contains("--data-dir \"C:\\rMenuData\""));
    }
}
