use crate::app_state::{
    ensure_selection_visible, AppState, LauncherItem, LauncherItemTone, RmodsInstallStatusView,
    RmodsPendingAction, RmodsUiItem, RtasksInputPriority, RtasksInputStatus, StartupUpdateNotice,
};
use crate::fuzzy::fuzzy_score;
use crate::launcher::{
    abbreviate_target, centered_text_y, compact_target_hint, launch_target,
    truncate_with_ellipsis_end,
};
use crate::modules::{
    input_accessory_text,
    types::{InputAccessoryKind, ModuleKeyEvent},
    ModuleRuntime,
};
use crate::ranking::update_matching_items_with_dataset;
use crate::rmods_registry::{
    download_verify_and_install_rmod, fetch_default_registry, install_status_for,
    read_registry_cache, scan_installed_rmods, uninstall_rmod, RmodsInstallStatus,
    RmodsRegistryItem, DEFAULT_RMODS_REGISTRY_URL,
};
use crate::rsnip_companion::install_rsnip_latest;
use crate::rtasks_companion::{
    install_rtasks_latest, RtasksCompanion, RtasksIpcResponse, RtasksPriority, RtasksTaskStatus,
};
use crate::settings::{CmdOptions, QuickSelectMode, RmenuConfig};
use crate::sources::persist_history_entry;
use std::ffi::OsStr;
use std::iter::once;
use std::os::windows::ffi::OsStrExt;
use std::os::windows::process::CommandExt;
use std::process::Command;
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::sync::Mutex;
use std::thread::sleep;
use std::time::{Duration, Instant};
use windows::{
    core::PCWSTR,
    Win32::{
        Foundation::{BOOL, COLORREF, HWND, LPARAM, LRESULT, RECT, WPARAM},
        Graphics::Gdi::{
            BeginPaint, CreateFontW, CreateSolidBrush, EndPaint, FillRect, InvalidateRect,
            SelectObject, SetBkColor, SetTextColor, TextOutW, PAINTSTRUCT,
        },
        System::LibraryLoader::GetModuleHandleW,
        UI::{
            Input::KeyboardAndMouse::{
                GetKeyState, VK_BACK, VK_CONTROL, VK_DOWN, VK_ESCAPE, VK_MENU, VK_RETURN, VK_SHIFT,
                VK_SPACE, VK_TAB, VK_UP,
            },
            WindowsAndMessaging::{
                CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetClientRect,
                GetSystemMetrics, GetWindowRect, IsWindow, KillTimer, LoadCursorW, MoveWindow,
                PeekMessageW, PostMessageW, PostQuitMessage, RegisterClassW, SetForegroundWindow,
                SetTimer, ShowWindow, TranslateMessage, CS_HREDRAW, CS_VREDRAW, IDC_ARROW, MSG,
                PM_REMOVE, SM_CXSCREEN, SM_CYSCREEN, SW_SHOW, WINDOW_EX_STYLE, WM_CHAR, WM_CREATE,
                WM_DESTROY, WM_KEYDOWN, WM_PAINT, WM_QUIT, WM_SYSKEYDOWN, WM_TIMER, WNDCLASSW,
                WS_EX_APPWINDOW, WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_POPUP, WS_VISIBLE,
            },
        },
    },
};

static APP_STATE: Mutex<Option<AppState>> = Mutex::new(None);
static CONFIG: Mutex<Option<RmenuConfig>> = Mutex::new(None);
static MODULE_RUNTIME: Mutex<Option<ModuleRuntime>> = Mutex::new(None);
static UI_MEASURE_STATE: Mutex<UiMeasureState> = Mutex::new(UiMeasureState::disabled());
static UI_RUN_TIMING_TRACE: Mutex<Option<UiRunTimingTrace>> = Mutex::new(None);
static UI_EMBEDDED_MODE: AtomicBool = AtomicBool::new(false);
static UI_EXIT_CODE: AtomicI32 = AtomicI32::new(0);
static SUPPRESS_NEXT_RMODS_SPACE_CHAR: AtomicBool = AtomicBool::new(false);
const INSTALL_CLOSE_TIMER_ID: usize = 42;
const INSTALL_START_TIMER_ID: usize = 43;
const WM_INSTALL_PROGRESS: u32 = 0x8000 + 1;
const WM_INSTALL_DONE: u32 = 0x8000 + 2;
const INPUT_PLACEHOLDER_TEXT: &str = concat!("rMenu ", env!("CARGO_PKG_VERSION"));
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

#[derive(Debug, Clone, Copy)]
pub struct UiLatencyMetrics {
    pub time_to_window_visible_ms: u128,
    pub time_to_first_paint_ms: u128,
    pub time_to_input_ready_ms: u128,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct UiRunTimings {
    pub pre_window_setup_ms: u128,
    pub module_on_load_ms: u128,
    pub initial_matching_update_ms: u128,
    pub register_class_ms: u128,
    pub create_window_ms: u128,
    pub time_to_window_visible_ms: u128,
    pub time_to_first_paint_ms: u128,
    pub time_to_input_ready_ms: u128,
    pub message_loop_ms: u128,
    pub total_ms: u128,
}

#[derive(Debug)]
struct UiRunTimingTrace {
    started_at: Instant,
    time_to_first_paint_ms: Option<u128>,
    time_to_input_ready_ms: Option<u128>,
}

#[derive(Debug)]
struct UiMeasureState {
    enabled: bool,
    started_at: Option<Instant>,
    time_to_window_visible_ms: Option<u128>,
    time_to_first_paint_ms: Option<u128>,
    time_to_input_ready_ms: Option<u128>,
}

impl UiMeasureState {
    const fn disabled() -> Self {
        Self {
            enabled: false,
            started_at: None,
            time_to_window_visible_ms: None,
            time_to_first_paint_ms: None,
            time_to_input_ready_ms: None,
        }
    }

    fn enabled_now() -> Self {
        Self {
            enabled: true,
            started_at: Some(Instant::now()),
            time_to_window_visible_ms: None,
            time_to_first_paint_ms: None,
            time_to_input_ready_ms: None,
        }
    }
}

fn mark_window_visible_metric() {
    let mut state = UI_MEASURE_STATE.lock().unwrap();
    if !state.enabled || state.time_to_window_visible_ms.is_some() {
        return;
    }
    let Some(started_at) = state.started_at else {
        return;
    };
    state.time_to_window_visible_ms = Some(started_at.elapsed().as_millis());
}

fn mark_first_paint_and_input_ready_metrics() {
    {
        let mut state = UI_MEASURE_STATE.lock().unwrap();
        if state.enabled {
            if let Some(started_at) = state.started_at {
                let elapsed_ms = started_at.elapsed().as_millis();

                if state.time_to_first_paint_ms.is_none() {
                    state.time_to_first_paint_ms = Some(elapsed_ms);
                }
                if state.time_to_input_ready_ms.is_none() {
                    state.time_to_input_ready_ms = Some(elapsed_ms);
                }
            }
        }
    }

    let mut trace = UI_RUN_TIMING_TRACE.lock().unwrap();
    if let Some(trace) = trace.as_mut() {
        let elapsed_ms = trace.started_at.elapsed().as_millis();
        if trace.time_to_first_paint_ms.is_none() {
            trace.time_to_first_paint_ms = Some(elapsed_ms);
        }
        if trace.time_to_input_ready_ms.is_none() {
            trace.time_to_input_ready_ms = Some(elapsed_ms);
        }
    }
}

fn is_measure_mode_enabled() -> bool {
    UI_MEASURE_STATE.lock().unwrap().enabled
}

fn request_ui_exit(hwnd: HWND, exit_code: i32) {
    UI_EXIT_CODE.store(exit_code, Ordering::SeqCst);
    unsafe {
        DestroyWindow(hwnd);
    }
}

fn current_ui_exit_code() -> i32 {
    UI_EXIT_CODE.load(Ordering::SeqCst)
}

fn to_wstring(s: &str) -> Vec<u16> {
    OsStr::new(s).encode_wide().chain(once(0)).collect()
}

fn draw_text_w(hdc: windows::Win32::Graphics::Gdi::HDC, x: i32, y: i32, text: &str) {
    if text.is_empty() {
        return;
    }
    let utf16: Vec<u16> = text.encode_utf16().collect();
    unsafe {
        TextOutW(hdc, x, y, &utf16);
    }
}

fn blend_color(left: COLORREF, right: COLORREF, right_weight: u32) -> COLORREF {
    let left_weight = 100u32.saturating_sub(right_weight);
    let left_value = left.0;
    let right_value = right.0;
    let blend_channel = |shift: u32| -> u32 {
        let left_channel = (left_value >> shift) & 0xff;
        let right_channel = (right_value >> shift) & 0xff;
        ((left_channel * left_weight) + (right_channel * right_weight)) / 100
    };
    COLORREF(blend_channel(0) | (blend_channel(8) << 8) | (blend_channel(16) << 16))
}

fn placeholder_text_color(config: &RmenuConfig) -> COLORREF {
    blend_color(config.colors.background, config.colors.foreground, 45)
}

fn launcher_item_tone_color(tone: LauncherItemTone) -> COLORREF {
    match tone {
        LauncherItemTone::Success => COLORREF(0x0090D890),
        LauncherItemTone::Warning => COLORREF(0x0030B0E0),
        LauncherItemTone::Danger => COLORREF(0x004050E8),
    }
}

fn accessory_text_color(kind: InputAccessoryKind, config: &RmenuConfig) -> COLORREF {
    match kind {
        InputAccessoryKind::Info => config.colors.foreground,
        InputAccessoryKind::Hint => config.colors.foreground,
        InputAccessoryKind::Success => COLORREF(0x0090D890),
        InputAccessoryKind::Warning => COLORREF(0x0030B0E0),
        InputAccessoryKind::Error => COLORREF(0x004050E8),
    }
}

fn calculate_position_detailed(
    position_str: &str,
    screen_dimension: i32,
    window_dimension: i32,
) -> i32 {
    if position_str.starts_with('r') {
        if let Ok(relative) = position_str[1..].parse::<f32>() {
            return ((screen_dimension as f32 * relative) - (window_dimension as f32 / 2.0)) as i32;
        }
    }
    if let Ok(absolute) = position_str.parse::<i32>() {
        return absolute;
    }
    (screen_dimension - window_dimension) / 2
}

struct WindowGeometry {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

fn determine_window_geometry(
    cmd_opts: &CmdOptions,
    config: &RmenuConfig,
    num_items_to_show: usize,
) -> WindowGeometry {
    let screen_width = unsafe { GetSystemMetrics(SM_CXSCREEN) };
    let screen_height = unsafe { GetSystemMetrics(SM_CYSCREEN) };

    let final_layout_str = cmd_opts.layout.as_deref().unwrap_or_else(|| {
        config
            .dimensions
            .default_layout
            .as_deref()
            .unwrap_or("custom")
    });

    let mut x: i32;
    let mut y: i32;
    let mut w: i32;
    let mut h: i32;

    let input_bar_height = config.dimensions.height;
    let item_h = config.dimensions.item_height;
    let padding = config.dimensions.padding;
    let list_height = if num_items_to_show == 0 {
        0
    } else {
        (num_items_to_show as i32 * item_h) + (2 * padding)
    };
    let total_window_height = input_bar_height + list_height + config.dimensions.border_width * 2;

    match final_layout_str {
        "top-fullwidth" => {
            x = 0;
            y = 0;
            w = screen_width;
            h = total_window_height;
        }
        "bottom-fullwidth" => {
            x = 0;
            w = screen_width;
            h = total_window_height;
            y = screen_height - h;
        }
        "center-dialog" => {
            let target_width_percent = config.dimensions.width_percent.unwrap_or(0.6);
            let mut calculated_width = (screen_width as f32 * target_width_percent) as i32;
            if let Some(max_w) = config.dimensions.max_width {
                if calculated_width > max_w {
                    calculated_width = max_w;
                }
            }
            w = calculated_width;
            h = total_window_height;
            x = (screen_width - w) / 2;
            y = (screen_height - h) / 2;
        }
        "top-left" => {
            let target_width_percent = config.dimensions.width_percent.unwrap_or(0.3);
            let mut calculated_width = (screen_width as f32 * target_width_percent) as i32;
            if let Some(max_w) = config.dimensions.max_width {
                if calculated_width > max_w {
                    calculated_width = max_w;
                }
            }
            w = calculated_width;
            h = total_window_height;
            x = 0;
            y = 0;
        }
        "top-right" => {
            let target_width_percent = config.dimensions.width_percent.unwrap_or(0.3);
            let mut calculated_width = (screen_width as f32 * target_width_percent) as i32;
            if let Some(max_w) = config.dimensions.max_width {
                if calculated_width > max_w {
                    calculated_width = max_w;
                }
            }
            w = calculated_width;
            h = total_window_height;
            x = screen_width - w;
            y = 0;
        }
        "bottom-left" => {
            let target_width_percent = config.dimensions.width_percent.unwrap_or(0.3);
            let mut calculated_width = (screen_width as f32 * target_width_percent) as i32;
            if let Some(max_w) = config.dimensions.max_width {
                if calculated_width > max_w {
                    calculated_width = max_w;
                }
            }
            w = calculated_width;
            h = total_window_height;
            x = 0;
            y = screen_height - h;
        }
        "bottom-right" => {
            let target_width_percent = config.dimensions.width_percent.unwrap_or(0.3);
            let mut calculated_width = (screen_width as f32 * target_width_percent) as i32;
            if let Some(max_w) = config.dimensions.max_width {
                if calculated_width > max_w {
                    calculated_width = max_w;
                }
            }
            w = calculated_width;
            h = total_window_height;
            x = screen_width - w;
            y = screen_height - h;
        }
        _ => {
            let target_width_percent = config.dimensions.width_percent.unwrap_or(0.6);
            let mut calculated_width = (screen_width as f32 * target_width_percent) as i32;
            if let Some(max_w) = config.dimensions.max_width {
                if calculated_width > max_w {
                    calculated_width = max_w;
                }
            }
            w = calculated_width;
            h = total_window_height;
            x = calculate_position_detailed(
                config.dimensions.x_position.as_deref().unwrap_or("r0.5"),
                screen_width,
                w,
            );
            y = calculate_position_detailed(
                config.dimensions.y_position.as_deref().unwrap_or("r0.3"),
                screen_height,
                h,
            );
        }
    }

    if let Some(cli_w_percent) = cmd_opts.cli_width_percent {
        w = (screen_width as f32 * cli_w_percent) as i32;
        if let Some(cli_max_w) = cmd_opts.cli_max_width.or(config.dimensions.max_width) {
            if w > cli_max_w {
                w = cli_max_w;
            }
        }
    } else if let Some(cli_max_w) = cmd_opts.cli_max_width {
        if w > cli_max_w {
            w = cli_max_w;
        }
    }

    if let Some(cli_h) = cmd_opts.cli_height {
        h = cli_h;
    }

    if let Some(cli_x_str) = &cmd_opts.cli_x_pos {
        x = calculate_position_detailed(cli_x_str, screen_width, w);
    }
    if let Some(cli_y_str) = &cmd_opts.cli_y_pos {
        y = calculate_position_detailed(cli_y_str, screen_height, h);
    }

    WindowGeometry {
        x,
        y,
        width: w,
        height: h,
    }
}

fn visible_item_count(app_state: &AppState, config: &RmenuConfig) -> usize {
    if app_state.current_input.trim().is_empty() {
        return 0;
    }

    app_state
        .matching_items
        .len()
        .min(config.behavior.max_items.max(0) as usize)
}

fn resize_window_to_state(hwnd: HWND, app_state: &AppState) {
    let config_guard = CONFIG.lock().unwrap();
    let Some(config) = config_guard.as_ref() else {
        return;
    };

    let item_count = visible_item_count(app_state, config);
    let input_bar_height = config.dimensions.height;
    let list_height = if item_count == 0 {
        0
    } else {
        (item_count as i32 * config.dimensions.item_height) + (2 * config.dimensions.padding)
    };
    let height = input_bar_height + list_height + config.dimensions.border_width * 2;

    let mut rect = windows::Win32::Foundation::RECT::default();
    if unsafe { GetWindowRect(hwnd, &mut rect) }.as_bool() {
        let width = rect.right - rect.left;
        let _ = unsafe { MoveWindow(hwnd, rect.left, rect.top, width, height, true) };
    }
}

fn refresh_window(hwnd: HWND, app_state: &AppState) {
    resize_window_to_state(hwnd, app_state);
    unsafe {
        InvalidateRect(hwnd, None, true);
    }
}

fn is_rtasks_input(input: &str) -> bool {
    input == "t" || input.starts_with("t ")
}

fn is_rmods_input(input: &str) -> bool {
    let trimmed = input.trim_start();
    trimmed.eq_ignore_ascii_case("/rmods")
        || trimmed
            .get(..6)
            .is_some_and(|prefix| prefix.eq_ignore_ascii_case("/rmods"))
            && trimmed
                .as_bytes()
                .get(6)
                .is_some_and(|value| value.is_ascii_whitespace())
}

fn rmods_filter_query(input: &str) -> &str {
    let trimmed = input.trim_start();
    if trimmed
        .get(..6)
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case("/rmods"))
    {
        trimmed.get(6..).unwrap_or("").trim()
    } else {
        ""
    }
}

fn rtasks_task_text(input: &str) -> &str {
    input.strip_prefix("t ").unwrap_or("").trim()
}

fn toggle_rtasks_status(app_state: &mut AppState, status: RtasksInputStatus) {
    app_state.rtasks_status = if app_state.rtasks_status == Some(status) {
        None
    } else {
        Some(status)
    };
}

fn toggle_rtasks_priority(app_state: &mut AppState, priority: RtasksInputPriority) {
    app_state.rtasks_priority = if app_state.rtasks_priority == Some(priority) {
        None
    } else {
        Some(priority)
    };
}

fn rtasks_status_label(status: Option<RtasksInputStatus>) -> &'static str {
    match status.unwrap_or(RtasksInputStatus::Todo) {
        RtasksInputStatus::Todo => "TODO",
        RtasksInputStatus::Doing => "DOING",
        RtasksInputStatus::Done => "DONE",
    }
}

fn rtasks_priority_label(priority: Option<RtasksInputPriority>) -> &'static str {
    match priority.unwrap_or(RtasksInputPriority::Medium) {
        RtasksInputPriority::High => "prio:ALTA",
        RtasksInputPriority::Medium => "prio:MEDIA",
        RtasksInputPriority::Low => "prio:BAJA",
    }
}

fn rtasks_status_for_ipc(status: Option<RtasksInputStatus>) -> Option<RtasksTaskStatus> {
    status.map(|status| match status {
        RtasksInputStatus::Todo => RtasksTaskStatus::Todo,
        RtasksInputStatus::Doing => RtasksTaskStatus::Doing,
        RtasksInputStatus::Done => RtasksTaskStatus::Done,
    })
}

fn rtasks_priority_for_ipc(priority: Option<RtasksInputPriority>) -> Option<RtasksPriority> {
    priority.map(|priority| match priority {
        RtasksInputPriority::High => RtasksPriority::High,
        RtasksInputPriority::Medium => RtasksPriority::Medium,
        RtasksInputPriority::Low => RtasksPriority::Low,
    })
}

fn update_rmods_items(app_state: &mut AppState, force_refresh: bool) {
    if app_state.rmods.loaded && !force_refresh {
        render_rmods_matching_items(app_state);
        return;
    }

    let registry = fetch_default_registry(None).or_else(|_| read_registry_cache(None));
    let local_modules = scan_installed_rmods(None).unwrap_or_default();

    match registry {
        Ok(registry) => {
            app_state.rmods.items = registry
                .modules
                .into_iter()
                .map(|item| {
                    let local = local_modules.get(&item.id.to_ascii_lowercase());
                    let status = match install_status_for(&item, local) {
                        RmodsInstallStatus::NotInstalled => RmodsInstallStatusView::NotInstalled,
                        RmodsInstallStatus::Installed => RmodsInstallStatusView::Installed,
                        RmodsInstallStatus::UpdateAvailable => {
                            RmodsInstallStatusView::UpdateAvailable
                        }
                        RmodsInstallStatus::LocalNewer => RmodsInstallStatusView::LocalNewer,
                        RmodsInstallStatus::ChecksumMismatch => {
                            RmodsInstallStatusView::ChecksumMismatch
                        }
                    };
                    RmodsUiItem {
                        id: item.id,
                        name: item.name,
                        version: item.version,
                        description: item.description,
                        kind: item.kind,
                        download_url: item.download_url,
                        base_url: item.base_url,
                        sha256: item.sha256,
                        size: item.size,
                        files: item.files,
                        status,
                        pending_action: RmodsPendingAction::None,
                    }
                })
                .collect();
            app_state.rmods.error = None;
        }
        Err(error) => {
            app_state.rmods.items.clear();
            app_state.rmods.error = Some(error.message());
        }
    }

    app_state.rmods.loaded = true;
    render_rmods_matching_items(app_state);
}

fn render_rmods_matching_items(app_state: &mut AppState) {
    if let Some(error) = &app_state.rmods.error {
        let mut error_item = LauncherItem::new(
            format!("rMods registry error: {error}"),
            "rmods:error".to_string(),
            crate::app_state::LauncherSource::Direct,
        );
        error_item.trailing_hint = Some("Press R to retry".to_string());
        error_item.trailing_badge = Some("error".to_string());
        app_state.matching_items = vec![error_item];
        app_state.selected_index = 0;
        app_state.scroll_offset = 0;
        return;
    }

    let filter = rmods_filter_query(&app_state.current_input);
    let mut rendered = app_state
        .rmods
        .items
        .iter()
        .enumerate()
        .filter_map(|(index, item)| {
            let score = if filter.is_empty() {
                1
            } else {
                fuzzy_score(
                    filter,
                    &format!("{} {} {}", item.name, item.id, item.description),
                    false,
                )
            };
            if score <= 0 {
                return None;
            }

            let checkbox = rmods_checkbox(item.status, item.pending_action);
            let status = rmods_status_label(item.status);
            let label = format!("{checkbox} {} {}", item.name, item.version);
            let mut launcher_item = LauncherItem::new(
                label,
                format!("rmods:{}", item.id),
                crate::app_state::LauncherSource::Direct,
            );
            launcher_item.trailing_badge = Some(status.to_string());
            launcher_item.trailing_badge_tone = rmods_status_tone(item.status);
            launcher_item.trailing_hint = if item.description.trim().is_empty() {
                Some("Space mark | F5/Ctrl+R refresh | Ctrl+U updates".to_string())
            } else {
                Some(item.description.clone())
            };
            Some((index, score, launcher_item))
        })
        .collect::<Vec<_>>();
    if !filter.is_empty() {
        rendered.sort_by(
            |(left_index, left_score, _), (right_index, right_score, _)| {
                right_score
                    .cmp(left_score)
                    .then_with(|| left_index.cmp(right_index))
            },
        );
    }
    app_state.matching_items = rendered.into_iter().map(|(_, _, item)| item).collect();
    if app_state.matching_items.is_empty() {
        app_state.matching_items = vec![LauncherItem::new(
            if filter.is_empty() {
                "No rMods found in registry".to_string()
            } else {
                format!("No rMods match '{filter}'")
            },
            "rmods:empty".to_string(),
            crate::app_state::LauncherSource::Direct,
        )];
    }
    if app_state.selected_index >= app_state.matching_items.len() {
        app_state.selected_index = app_state.matching_items.len().saturating_sub(1);
    }
    ensure_selection_visible(app_state, 10);
}

fn rmods_badge_tone(item: &LauncherItem) -> Option<LauncherItemTone> {
    item.trailing_badge_tone.or_else(|| {
        if item.target.starts_with("rmods:") {
            rmods_status_label_tone(item.trailing_badge.as_deref()?)
        } else {
            None
        }
    })
}

fn rmods_status_label_tone(label: &str) -> Option<LauncherItemTone> {
    match label {
        "installed" => Some(LauncherItemTone::Success),
        "not installed" => Some(LauncherItemTone::Danger),
        "update available" | "local newer" => Some(LauncherItemTone::Warning),
        "checksum mismatch" => Some(LauncherItemTone::Danger),
        _ => None,
    }
}

fn rmods_status_tone(status: RmodsInstallStatusView) -> Option<LauncherItemTone> {
    match status {
        RmodsInstallStatusView::Installed => Some(LauncherItemTone::Success),
        RmodsInstallStatusView::NotInstalled => Some(LauncherItemTone::Danger),
        RmodsInstallStatusView::UpdateAvailable | RmodsInstallStatusView::LocalNewer => {
            Some(LauncherItemTone::Warning)
        }
        RmodsInstallStatusView::ChecksumMismatch => Some(LauncherItemTone::Danger),
    }
}

fn launch_update_changelog(notice: &StartupUpdateNotice) -> Result<(), String> {
    launch_target(&notice.release_url).map_err(|error| error.to_string())
}

fn launch_update_installer(notice: &StartupUpdateNotice) -> Result<(), String> {
    let installer_url = notice
        .installer_asset_url
        .as_ref()
        .ok_or_else(|| "update metadata is missing installer asset".to_string())?;
    let checksums_url = notice
        .checksums_asset_url
        .as_ref()
        .ok_or_else(|| "update metadata is missing SHA256SUMS asset".to_string())?;
    let updater_path = std::env::current_exe()
        .map_err(|error| error.to_string())?
        .parent()
        .ok_or_else(|| "could not resolve rmenu-updater.exe directory".to_string())?
        .join("rmenu-updater.exe");

    let mut command = Command::new(updater_path);
    command
        .arg("install")
        .arg("--version")
        .arg(&notice.version)
        .arg("--release-url")
        .arg(&notice.release_url)
        .arg("--installer-url")
        .arg(installer_url)
        .arg("--checksums-url")
        .arg(checksums_url);
    if let Some(data_dir) = &notice.data_dir {
        command.arg("--data-dir").arg(data_dir);
    }
    command
        .creation_flags(CREATE_NO_WINDOW)
        .spawn()
        .map(|_| ())
        .map_err(|error| format!("failed to launch updater: {error}"))
}

fn set_runtime_feedback(message: impl Into<String>, kind: InputAccessoryKind) {
    let mut runtime_guard = MODULE_RUNTIME.lock().unwrap();
    if let Some(runtime) = runtime_guard.as_mut() {
        runtime.set_runtime_feedback(message, kind);
    }
}

fn rmods_status_label(status: RmodsInstallStatusView) -> &'static str {
    match status {
        RmodsInstallStatusView::NotInstalled => "not installed",
        RmodsInstallStatusView::Installed => "installed",
        RmodsInstallStatusView::UpdateAvailable => "update available",
        RmodsInstallStatusView::LocalNewer => "local newer",
        RmodsInstallStatusView::ChecksumMismatch => "checksum mismatch",
    }
}

fn rmods_checkbox(status: RmodsInstallStatusView, action: RmodsPendingAction) -> &'static str {
    if action != RmodsPendingAction::None {
        "[/]"
    } else if rmods_status_is_installed(status) {
        "[x]"
    } else {
        "[ ]"
    }
}

fn rmods_status_is_installed(status: RmodsInstallStatusView) -> bool {
    matches!(
        status,
        RmodsInstallStatusView::Installed
            | RmodsInstallStatusView::UpdateAvailable
            | RmodsInstallStatusView::LocalNewer
            | RmodsInstallStatusView::ChecksumMismatch
    )
}

fn default_rmods_action(status: RmodsInstallStatusView) -> RmodsPendingAction {
    match status {
        RmodsInstallStatusView::NotInstalled => RmodsPendingAction::Install,
        RmodsInstallStatusView::UpdateAvailable => RmodsPendingAction::Update,
        RmodsInstallStatusView::Installed
        | RmodsInstallStatusView::LocalNewer
        | RmodsInstallStatusView::ChecksumMismatch => RmodsPendingAction::Uninstall,
    }
}

fn selected_rmods_item_id(app_state: &AppState) -> Option<String> {
    app_state
        .matching_items
        .get(app_state.selected_index)
        .and_then(|item| item.target.strip_prefix("rmods:"))
        .filter(|id| !id.is_empty() && *id != "empty" && *id != "error")
        .map(ToString::to_string)
}

fn restore_rmods_selection(app_state: &mut AppState, selected_id: &str) {
    if let Some(index) = app_state
        .matching_items
        .iter()
        .position(|item| item.target == format!("rmods:{selected_id}"))
    {
        app_state.selected_index = index;
        ensure_selection_visible(app_state, 10);
    }
}

fn toggle_selected_rmod(app_state: &mut AppState) {
    let Some(selected_id) = selected_rmods_item_id(app_state) else {
        return;
    };
    if let Some(item) = app_state
        .rmods
        .items
        .iter_mut()
        .find(|item| item.id == selected_id)
    {
        item.pending_action = if item.pending_action == RmodsPendingAction::None {
            default_rmods_action(item.status)
        } else {
            RmodsPendingAction::None
        };
        render_rmods_matching_items(app_state);
        restore_rmods_selection(app_state, &selected_id);
    }
}

fn select_rmods_updates(app_state: &mut AppState) {
    for item in &mut app_state.rmods.items {
        item.pending_action = if item.status == RmodsInstallStatusView::UpdateAvailable {
            RmodsPendingAction::Update
        } else {
            RmodsPendingAction::None
        };
    }
    render_rmods_matching_items(app_state);
}

fn rmods_registry_item_from_ui(item: &RmodsUiItem) -> RmodsRegistryItem {
    RmodsRegistryItem {
        id: item.id.clone(),
        name: item.name.clone(),
        version: item.version.clone(),
        description: item.description.clone(),
        kind: item.kind.clone(),
        download_url: item.download_url.clone(),
        base_url: item.base_url.clone(),
        sha256: item.sha256.clone(),
        size: item.size,
        files: item.files.clone(),
        tags: Vec::new(),
        requires_rmenu: None,
    }
}

#[derive(Debug, Default)]
struct RmodsApplySummary {
    installed: usize,
    updated: usize,
    uninstalled: usize,
}

fn apply_rmods_changes(app_state: &mut AppState) -> Result<RmodsApplySummary, String> {
    let pending = app_state
        .rmods
        .items
        .iter()
        .filter(|item| item.pending_action != RmodsPendingAction::None)
        .cloned()
        .collect::<Vec<_>>();
    if pending.is_empty() {
        return Ok(RmodsApplySummary::default());
    }

    let mut summary = RmodsApplySummary::default();
    let mut errors = Vec::new();
    for item in pending {
        match item.pending_action {
            RmodsPendingAction::None => {}
            RmodsPendingAction::Install => {
                let registry_item = rmods_registry_item_from_ui(&item);
                match download_verify_and_install_rmod(
                    &registry_item,
                    None,
                    DEFAULT_RMODS_REGISTRY_URL,
                ) {
                    Ok(_) => summary.installed += 1,
                    Err(error) => errors.push(format!("{}: {}", item.id, error.message())),
                }
            }
            RmodsPendingAction::Update => {
                let registry_item = rmods_registry_item_from_ui(&item);
                match download_verify_and_install_rmod(
                    &registry_item,
                    None,
                    DEFAULT_RMODS_REGISTRY_URL,
                ) {
                    Ok(_) => summary.updated += 1,
                    Err(error) => errors.push(format!("{}: {}", item.id, error.message())),
                }
            }
            RmodsPendingAction::Uninstall => match uninstall_rmod(&item.id, None) {
                Ok(_) => summary.uninstalled += 1,
                Err(error) => errors.push(format!("{}: {}", item.id, error.message())),
            },
        }
    }
    update_rmods_items(app_state, true);

    if errors.is_empty() {
        Ok(summary)
    } else {
        Err(format!(
            "installed {}, updated {}, removed {}; failed {} ({})",
            summary.installed,
            summary.updated,
            summary.uninstalled,
            errors.len(),
            errors.join("; ")
        ))
    }
}

fn startup_update_notice_item(notice: &StartupUpdateNotice) -> LauncherItem {
    let mut item = LauncherItem::new(
        format!("Update available: rMenu v{}", notice.version),
        "update:install".to_string(),
        crate::app_state::LauncherSource::Direct,
    );
    item.trailing_hint =
        Some("Enter install now | Ctrl+Enter view changelog | Any key continue".to_string());
    item.trailing_badge = Some("update available".to_string());
    item.trailing_badge_tone = Some(LauncherItemTone::Warning);
    item
}

fn render_startup_update_notice(app_state: &mut AppState) -> bool {
    let Some(notice) = app_state.startup_update_notice.as_ref() else {
        return false;
    };
    app_state.matching_items = vec![startup_update_notice_item(notice)];
    app_state.selected_index = 0;
    app_state.scroll_offset = 0;
    true
}

fn dismiss_startup_update_notice(app_state: &mut AppState) -> bool {
    if app_state.startup_update_notice.is_none() {
        return false;
    }
    app_state.startup_update_notice = None;
    app_state.selected_index = 0;
    update_matching_items_from_config(app_state);
    true
}

fn update_matching_items_from_config(app_state: &mut AppState) {
    if render_startup_update_notice(app_state) {
        return;
    }

    if !is_rtasks_input(&app_state.current_input) {
        app_state.rtasks_status = None;
        app_state.rtasks_priority = None;
    }

    if is_rmods_input(&app_state.current_input) {
        update_rmods_items(app_state, false);
        return;
    } else {
        app_state.rmods.loaded = false;
        app_state.rmods.items.clear();
        app_state.rmods.error = None;
    }

    if is_rtasks_input(&app_state.current_input) {
        app_state.matching_items.clear();
        app_state.selected_index = 0;
        app_state.scroll_offset = 0;
        return;
    }

    let mut provider_items = Vec::new();

    {
        let mut runtime_guard = MODULE_RUNTIME.lock().unwrap();
        if let Some(runtime) = runtime_guard.as_mut() {
            runtime.run_on_query_change(app_state);
            if runtime.items_replaced_in_cycle() {
                normalize_quick_select_items(app_state);
                return;
            }
            provider_items = runtime.collect_provider_items(app_state);
        }
    }

    let config_guard = CONFIG.lock().unwrap();
    let case_sensitive = config_guard
        .as_ref()
        .map_or(false, |c| c.behavior.case_sensitive);
    let max_visible_items = config_guard
        .as_ref()
        .map_or(10usize, |c| c.behavior.max_items.max(1) as usize);
    drop(config_guard);

    let mut dataset = app_state.all_items.clone();

    {
        let runtime_guard = MODULE_RUNTIME.lock().unwrap();
        if let Some(runtime) = runtime_guard.as_ref() {
            dataset = runtime.merge_rank_dataset(app_state.all_items.clone(), provider_items);
        } else {
            dataset.extend(provider_items);
        }
    }

    update_matching_items_with_dataset(app_state, dataset, case_sensitive, max_visible_items);

    {
        let mut runtime_guard = MODULE_RUNTIME.lock().unwrap();
        if let Some(runtime) = runtime_guard.as_mut() {
            let decorated = runtime.decorate_items(app_state, app_state.matching_items.clone());
            app_state.matching_items = decorated;
        }
    }

    normalize_quick_select_items(app_state);
    ensure_selection_visible(app_state, max_visible_items);
}

fn resolve_digit_from_key(key_code: i32) -> Option<char> {
    if (0x30..=0x39).contains(&key_code) {
        return char::from_u32(key_code as u32);
    }

    if (0x60..=0x69).contains(&key_code) {
        return char::from_u32((key_code as u32 - 0x60) + 0x30);
    }

    None
}

fn resolve_key_name(key_code: i32) -> String {
    if let Some(digit) = resolve_digit_from_key(key_code) {
        return digit.to_string();
    }

    match key_code {
        code if code == VK_RETURN.0 as i32 => "enter".to_string(),
        code if code == VK_ESCAPE.0 as i32 => "escape".to_string(),
        code if code == VK_TAB.0 as i32 => "tab".to_string(),
        code if code == VK_BACK.0 as i32 => "backspace".to_string(),
        code if code == VK_UP.0 as i32 => "up".to_string(),
        code if code == VK_DOWN.0 as i32 => "down".to_string(),
        code if (0x30..=0x39).contains(&code) => {
            char::from_u32(code as u32).unwrap_or('?').to_string()
        }
        code if (0x41..=0x5A).contains(&code) => char::from_u32((code + 32) as u32)
            .unwrap_or('?')
            .to_string(),
        _ => format!("vk_{key_code}"),
    }
}

fn is_key_pressed(vk: i32) -> bool {
    unsafe { (GetKeyState(vk) as u16 & 0x8000) != 0 }
}

fn build_module_key_event(key_code: i32) -> ModuleKeyEvent {
    ModuleKeyEvent {
        key: resolve_key_name(key_code),
        ctrl: is_key_pressed(VK_CONTROL.0 as i32),
        alt: is_key_pressed(VK_MENU.0 as i32),
        shift: is_key_pressed(VK_SHIFT.0 as i32),
        meta: false,
    }
}

fn find_quick_select_index(
    app_state: &AppState,
    key: char,
    max_visible_items: usize,
) -> Option<usize> {
    let visible_end =
        (app_state.scroll_offset + max_visible_items).min(app_state.matching_items.len());
    let key_str = key.to_string();

    for item_index in app_state.scroll_offset..visible_end {
        let item = &app_state.matching_items[item_index];
        if item.quick_select_key.as_deref() == Some(key_str.as_str()) {
            return Some(item_index);
        }
    }

    None
}

fn normalize_quick_select_items(app_state: &mut AppState) {
    use std::collections::BTreeSet;

    let mut seen: BTreeSet<String> = BTreeSet::new();

    for item in &mut app_state.matching_items {
        let Some(key) = item.quick_select_key.clone() else {
            continue;
        };

        if key.is_empty() {
            item.quick_select_key = None;
            continue;
        }

        if seen.insert(key.clone()) {
            continue;
        }

        if !app_state.silent_mode {
            eprintln!(
                "modules quick-select conflict: key '{}' duplicated, keeping first item and clearing duplicate on '{}'",
                key, item.label
            );
        }

        item.quick_select_key = None;
        if item.trailing_badge.as_deref() == Some(key.as_str()) {
            item.trailing_badge = None;
            item.trailing_badge_tone = None;
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RowZones {
    left_text: String,
    right_text: String,
    chip_text: Option<String>,
    right_x: i32,
    chip_x: i32,
}

fn compute_row_zones(
    label: &str,
    hint: &str,
    chip: Option<String>,
    left_x: i32,
    row_right_bound: i32,
    char_w: i32,
) -> RowZones {
    if row_right_bound <= left_x || char_w <= 0 {
        return RowZones {
            left_text: String::new(),
            right_text: String::new(),
            chip_text: None,
            right_x: left_x,
            chip_x: left_x,
        };
    }

    let total_w = row_right_bound - left_x;
    let min_gap = char_w * 2;
    let zone_gap = char_w;
    let min_label_w = char_w * 4;
    let min_hint_chars = 4;

    let mut chip_text = chip.filter(|value| !value.is_empty());
    let mut chip_w = chip_text
        .as_ref()
        .map(|text| text.chars().count() as i32 * char_w)
        .unwrap_or(0);

    if chip_text.is_some() && total_w < min_label_w + zone_gap + chip_w {
        chip_text = None;
        chip_w = 0;
    }

    let hint_right_bound = if chip_text.is_some() {
        row_right_bound - chip_w - zone_gap
    } else {
        row_right_bound
    };

    let mut right_text = String::new();
    let mut right_w = 0;
    let available_for_hint = hint_right_bound - left_x - min_label_w - min_gap;
    if !hint.is_empty() && available_for_hint >= char_w * min_hint_chars {
        let max_hint_chars = (available_for_hint / char_w).max(1) as usize;
        right_text = abbreviate_target(hint, max_hint_chars);
        right_w = right_text.chars().count() as i32 * char_w;
    }

    let right_x = if right_text.is_empty() {
        hint_right_bound
    } else {
        hint_right_bound - right_w
    };

    let label_right_limit = if right_text.is_empty() {
        hint_right_bound
    } else {
        right_x - min_gap
    };
    let left_max_w = (label_right_limit - left_x).max(char_w);
    let left_max_chars = (left_max_w / char_w).max(1) as usize;
    let left_text = truncate_with_ellipsis_end(label, left_max_chars);

    let chip_x = if chip_text.is_some() {
        row_right_bound - chip_w
    } else {
        hint_right_bound
    };

    RowZones {
        left_text,
        right_text,
        chip_text,
        right_x,
        chip_x,
    }
}

unsafe extern "system" fn window_proc(
    hwnd: HWND,
    msg: u32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    match msg {
        WM_CREATE => {
            if !is_measure_mode_enabled() {
                sleep(Duration::from_millis(50));
                SetForegroundWindow(hwnd);
            }
            LRESULT(0)
        }
        WM_PAINT => {
            let mut ps = PAINTSTRUCT::default();
            let hdc = BeginPaint(hwnd, &mut ps);

            let config_guard = CONFIG.lock().unwrap();
            let app_state_guard = APP_STATE.lock().unwrap();

            if let (Some(config), Some(app_state)) = (&*config_guard, &*app_state_guard) {
                let config = config.clone();
                let app_state = app_state.clone();
                drop(config_guard);
                drop(app_state_guard);

                let mut rect = RECT::default();
                GetClientRect(hwnd, &mut rect);

                SetBkColor(hdc, config.colors.background);
                SetTextColor(hdc, config.colors.foreground);

                let bg_brush = CreateSolidBrush(config.colors.background);
                FillRect(hdc, &rect, bg_brush);

                let final_border_width = config.dimensions.border_width;

                if final_border_width > 0 {
                    let border_brush = CreateSolidBrush(config.colors.border);
                    let bw = final_border_width;
                    FillRect(
                        hdc,
                        &RECT {
                            left: 0,
                            top: 0,
                            right: rect.right,
                            bottom: bw,
                        },
                        border_brush,
                    );
                    FillRect(
                        hdc,
                        &RECT {
                            left: 0,
                            top: rect.bottom - bw,
                            right: rect.right,
                            bottom: rect.bottom,
                        },
                        border_brush,
                    );
                    FillRect(
                        hdc,
                        &RECT {
                            left: 0,
                            top: bw,
                            right: bw,
                            bottom: rect.bottom - bw,
                        },
                        border_brush,
                    );
                    FillRect(
                        hdc,
                        &RECT {
                            left: rect.right - bw,
                            top: bw,
                            right: rect.right,
                            bottom: rect.bottom - bw,
                        },
                        border_brush,
                    );
                }

                let font = CreateFontW(
                    config.font.size,
                    0,
                    0,
                    0,
                    config.font.weight as i32,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    config
                        .font
                        .name
                        .as_ref()
                        .map_or(PCWSTR::null(), |name| PCWSTR(to_wstring(name).as_ptr())),
                );
                let old_font = SelectObject(hdc, font);

                let current_padding = config.dimensions.padding;
                let char_w = (config.font.size / 2).max(6);

                let input_bar_actual_height = config.dimensions.height;
                let input_text_y = centered_text_y(0, input_bar_actual_height, config.font.size);

                let mut x_offset = current_padding;
                if let Some(prompt) = &app_state.prompt {
                    let prompt_text = format!("{}: ", prompt);
                    draw_text_w(hdc, x_offset, input_text_y, &prompt_text);
                    x_offset += prompt_text.len() as i32 * char_w;
                }

                if app_state.current_input.is_empty() {
                    SetTextColor(hdc, placeholder_text_color(&config));
                    draw_text_w(hdc, x_offset, input_text_y, INPUT_PLACEHOLDER_TEXT);
                    SetTextColor(hdc, config.colors.foreground);
                } else {
                    draw_text_w(hdc, x_offset, input_text_y, &app_state.current_input);
                }

                let accessory = {
                    let runtime_guard = MODULE_RUNTIME.lock().unwrap();
                    runtime_guard
                        .as_ref()
                        .and_then(|runtime| runtime.active_input_accessory())
                };

                if is_rtasks_input(&app_state.current_input) {
                    let status_text = rtasks_status_label(app_state.rtasks_status);
                    let priority_text = rtasks_priority_label(app_state.rtasks_priority);
                    let gap = char_w * 2;
                    let priority_w = priority_text.chars().count() as i32 * char_w;
                    let status_w = status_text.chars().count() as i32 * char_w;
                    let priority_x = rect.right - current_padding - priority_w;
                    let status_x = priority_x - gap - status_w;
                    SetTextColor(
                        hdc,
                        if app_state.rtasks_status.is_some() {
                            COLORREF(0x0079C398)
                        } else {
                            config.colors.foreground
                        },
                    );
                    draw_text_w(
                        hdc,
                        status_x.max(x_offset + current_padding),
                        input_text_y,
                        status_text,
                    );
                    SetTextColor(
                        hdc,
                        if app_state.rtasks_priority.is_some() {
                            COLORREF(0x0079C398)
                        } else {
                            config.colors.foreground
                        },
                    );
                    draw_text_w(
                        hdc,
                        priority_x.max(x_offset + current_padding),
                        input_text_y,
                        priority_text,
                    );
                    SetTextColor(hdc, config.colors.foreground);
                } else if let Some(accessory) = accessory {
                    let accessory_text = input_accessory_text(&accessory);
                    let max_chars =
                        ((rect.right - (x_offset + current_padding * 2)) / char_w).max(0) as usize;
                    if max_chars >= 6 {
                        let accessory_draw = truncate_with_ellipsis_end(&accessory_text, max_chars);
                        let accessory_w = accessory_draw.chars().count() as i32 * char_w;
                        let accessory_x = (rect.right - current_padding - accessory_w)
                            .max(x_offset + current_padding);
                        let old_color = accessory_text_color(accessory.kind, &config);
                        SetTextColor(hdc, old_color);
                        draw_text_w(hdc, accessory_x, input_text_y, &accessory_draw);
                        SetTextColor(hdc, config.colors.foreground);
                    }
                }

                let current_item_height = config.dimensions.item_height;
                let max_items_to_display = config.behavior.max_items.max(1) as usize;

                let visible_end = (app_state.scroll_offset + max_items_to_display)
                    .min(app_state.matching_items.len());

                for (visible_row, item_index) in (app_state.scroll_offset..visible_end).enumerate()
                {
                    let item = &app_state.matching_items[item_index];
                    let y = input_bar_actual_height + (current_item_height * visible_row as i32);

                    if item_index == app_state.selected_index {
                        SetBkColor(hdc, config.colors.selected_background);
                        SetTextColor(hdc, config.colors.selected_foreground);
                        let select_rect = RECT {
                            left: 0,
                            top: y,
                            right: rect.right,
                            bottom: y + current_item_height,
                        };
                        let select_brush = CreateSolidBrush(config.colors.selected_background);
                        FillRect(hdc, &select_rect, select_brush);
                    } else {
                        SetBkColor(hdc, config.colors.background);
                        SetTextColor(hdc, config.colors.foreground);
                    }

                    let left_x = current_padding;
                    let row_right_bound = rect.right - current_padding;

                    let chip_text = item
                        .quick_select_key
                        .as_ref()
                        .map(|key| format!("[{key}]"))
                        .or_else(|| item.trailing_badge.clone());
                    let default_hint = compact_target_hint(&item.target);
                    let row = compute_row_zones(
                        &item.label,
                        item.trailing_hint
                            .as_deref()
                            .unwrap_or(default_hint.as_str()),
                        chip_text,
                        left_x,
                        row_right_bound,
                        char_w,
                    );

                    let item_text_y = centered_text_y(y, current_item_height, config.font.size);
                    draw_text_w(hdc, left_x, item_text_y, &row.left_text);
                    if !row.right_text.is_empty() {
                        draw_text_w(hdc, row.right_x, item_text_y, &row.right_text);
                    }
                    if let Some(chip) = row.chip_text {
                        let badge_tone = rmods_badge_tone(item);
                        let previous_color = if let Some(tone) = badge_tone {
                            SetTextColor(hdc, launcher_item_tone_color(tone))
                        } else {
                            COLORREF(0)
                        };
                        draw_text_w(hdc, row.chip_x.max(left_x + char_w), item_text_y, &chip);
                        if badge_tone.is_some() {
                            SetTextColor(hdc, previous_color);
                        }
                    }
                }

                SelectObject(hdc, old_font);
            }
            EndPaint(hwnd, &ps);

            mark_first_paint_and_input_ready_metrics();
            if is_measure_mode_enabled() {
                request_ui_exit(hwnd, 0);
            }

            LRESULT(0)
        }
        WM_KEYDOWN | WM_SYSKEYDOWN => {
            let key_code = w_param.0 as i32;
            let mut app_state_guard = APP_STATE.lock().unwrap();
            if let Some(app_state) = app_state_guard.as_mut() {
                let ctrl_down = unsafe { (GetKeyState(VK_CONTROL.0 as i32) as u16 & 0x8000) != 0 };
                if let Some(notice) = app_state.startup_update_notice.clone() {
                    if key_code == VK_RETURN.0 as i32 && ctrl_down {
                        let result = launch_update_changelog(&notice);
                        dismiss_startup_update_notice(app_state);
                        if let Err(error) = result {
                            set_runtime_feedback(
                                format!("Failed to open changelog: {error}"),
                                InputAccessoryKind::Error,
                            );
                        }
                        refresh_window(hwnd, app_state);
                        return LRESULT(0);
                    }
                    if key_code == VK_RETURN.0 as i32 {
                        let result = launch_update_installer(&notice);
                        dismiss_startup_update_notice(app_state);
                        match result {
                            Ok(()) => request_ui_exit(hwnd, 0),
                            Err(error) => {
                                set_runtime_feedback(error, InputAccessoryKind::Error);
                                refresh_window(hwnd, app_state);
                            }
                        }
                        return LRESULT(0);
                    }
                    dismiss_startup_update_notice(app_state);
                    InvalidateRect(hwnd, None, true);
                    return LRESULT(0);
                }

                let module_key_event = build_module_key_event(key_code);
                let input_before_modules = app_state.current_input.clone();
                {
                    let mut runtime_guard = MODULE_RUNTIME.lock().unwrap();
                    if let Some(runtime) = runtime_guard.as_mut() {
                        runtime.run_on_key(app_state, &module_key_event);
                    }
                }
                if app_state.current_input != input_before_modules {
                    update_matching_items_from_config(app_state);
                    InvalidateRect(hwnd, None, true);
                }

                let alt_down = unsafe { (GetKeyState(VK_MENU.0 as i32) as u16 & 0x8000) != 0 };
                if is_rtasks_input(&app_state.current_input) && alt_down {
                    match key_code {
                        code if code == '1' as i32 => {
                            toggle_rtasks_status(app_state, RtasksInputStatus::Todo)
                        }
                        code if code == '2' as i32 => {
                            toggle_rtasks_status(app_state, RtasksInputStatus::Doing)
                        }
                        code if code == '3' as i32 => {
                            toggle_rtasks_status(app_state, RtasksInputStatus::Done)
                        }
                        code if code == 'Q' as i32 => {
                            toggle_rtasks_priority(app_state, RtasksInputPriority::High)
                        }
                        code if code == 'W' as i32 => {
                            toggle_rtasks_priority(app_state, RtasksInputPriority::Medium)
                        }
                        code if code == 'E' as i32 => {
                            toggle_rtasks_priority(app_state, RtasksInputPriority::Low)
                        }
                        _ => {}
                    }
                    refresh_window(hwnd, app_state);
                    return LRESULT(0);
                }

                if is_rmods_input(&app_state.current_input) {
                    match key_code {
                        code if code == VK_SPACE.0 as i32 => {
                            toggle_selected_rmod(app_state);
                            SUPPRESS_NEXT_RMODS_SPACE_CHAR.store(true, Ordering::Relaxed);
                            refresh_window(hwnd, app_state);
                            return LRESULT(0);
                        }
                        code if code == 0x74 || (ctrl_down && code == 'R' as i32) => {
                            update_rmods_items(app_state, true);
                            refresh_window(hwnd, app_state);
                            return LRESULT(0);
                        }
                        code if ctrl_down && code == 'U' as i32 => {
                            select_rmods_updates(app_state);
                            refresh_window(hwnd, app_state);
                            return LRESULT(0);
                        }
                        _ => {}
                    }
                }

                if key_code == VK_ESCAPE.0 as i32 {
                    request_ui_exit(hwnd, 1);
                } else if let Some(digit_key) = resolve_digit_from_key(key_code) {
                    let (max_visible, quick_select_mode) = {
                        let config_guard = CONFIG.lock().unwrap();
                        let max_visible = config_guard
                            .as_ref()
                            .map_or(10usize, |c| c.behavior.max_items.max(1) as usize);
                        let mode = config_guard
                            .as_ref()
                            .map_or(QuickSelectMode::Submit, |c| c.behavior.quick_select_mode);
                        (max_visible, mode)
                    };

                    if let Some(item_index) =
                        find_quick_select_index(app_state, digit_key, max_visible)
                    {
                        app_state.selected_index = item_index;
                        ensure_selection_visible(app_state, max_visible);

                        if quick_select_mode == QuickSelectMode::Submit {
                            let selected =
                                app_state.matching_items[app_state.selected_index].clone();
                            if app_state.launcher_mode {
                                if let Err(e) = launch_target(&selected.target) {
                                    if !app_state.silent_mode {
                                        eprintln!(
                                            "Error launching target '{}': {}",
                                            selected.target, e
                                        );
                                    }
                                } else {
                                    persist_history_entry(
                                        &selected.target,
                                        app_state.silent_mode,
                                        app_state.history_max_items,
                                    );
                                }
                            } else {
                                println!("{}", selected.label);
                            }
                            request_ui_exit(hwnd, 0);
                        } else {
                            InvalidateRect(hwnd, None, true);
                        }
                    }
                } else if key_code == VK_RETURN.0 as i32 {
                    if is_rmods_input(&app_state.current_input) {
                        let result = apply_rmods_changes(app_state);
                        let mut runtime_guard = MODULE_RUNTIME.lock().unwrap();
                        if let Some(runtime) = runtime_guard.as_mut() {
                            match result {
                                Ok(summary)
                                    if summary.installed == 0
                                        && summary.updated == 0
                                        && summary.uninstalled == 0 =>
                                {
                                    runtime.set_runtime_feedback(
                                        "No rMods changes",
                                        InputAccessoryKind::Hint,
                                    );
                                }
                                Ok(summary) => {
                                    runtime.reload_external_descriptors(app_state.silent_mode);
                                    runtime.set_runtime_feedback(
                                        format!(
                                            "rMods: installed {}, updated {}, removed {}",
                                            summary.installed, summary.updated, summary.uninstalled
                                        ),
                                        InputAccessoryKind::Success,
                                    );
                                }
                                Err(error) => {
                                    runtime.reload_external_descriptors(app_state.silent_mode);
                                    runtime.set_runtime_feedback(
                                        format!("rMods changes failed: {error}"),
                                        InputAccessoryKind::Error,
                                    );
                                }
                            }
                        }
                        refresh_window(hwnd, app_state);
                        return LRESULT(0);
                    } else if is_rtasks_input(&app_state.current_input) {
                        let task_input = rtasks_task_text(&app_state.current_input).to_string();
                        if !task_input.is_empty() {
                            let result = RtasksCompanion::discover().and_then(|companion| {
                                companion.ensure_and_add_task(
                                    task_input,
                                    rtasks_status_for_ipc(app_state.rtasks_status),
                                    rtasks_priority_for_ipc(app_state.rtasks_priority),
                                )
                            });
                            let mut runtime_guard = MODULE_RUNTIME.lock().unwrap();
                            if let Some(runtime) = runtime_guard.as_mut() {
                                match result {
                                    Ok(RtasksIpcResponse::Ok { .. }) => runtime
                                        .set_runtime_feedback(
                                            "RTasks task added",
                                            InputAccessoryKind::Success,
                                        ),
                                    Ok(RtasksIpcResponse::Error { message }) => runtime
                                        .set_runtime_feedback(
                                            format!("RTasks add failed: {message}"),
                                            InputAccessoryKind::Error,
                                        ),
                                    Err(err) => runtime.set_runtime_feedback(
                                        format!("RTasks add failed: {err:?}"),
                                        InputAccessoryKind::Error,
                                    ),
                                }
                            }
                        }
                        request_ui_exit(hwnd, 0);
                    } else if !app_state.current_input.is_empty() {
                        let current_input = app_state.current_input.clone();
                        if let Some(raw_command) = current_input.strip_prefix('/') {
                            let parts = raw_command.split_whitespace().collect::<Vec<_>>();
                            if let Some((command, rest)) = parts.split_first() {
                                let args =
                                    rest.iter().map(|v| (*v).to_string()).collect::<Vec<_>>();
                                if command.eq_ignore_ascii_case("install")
                                    && args.first().is_some_and(|value| {
                                        value.eq_ignore_ascii_case("rsnip")
                                            || value.eq_ignore_ascii_case("rtasks")
                                    })
                                {
                                    let companion_name = args
                                        .first()
                                        .map(|value| value.to_ascii_lowercase())
                                        .unwrap_or_default();
                                    {
                                        let mut runtime_guard = MODULE_RUNTIME.lock().unwrap();
                                        if let Some(runtime) = runtime_guard.as_mut() {
                                            runtime.set_runtime_feedback(
                                                format!(
                                                    "Fetching {} from GitHub latest release",
                                                    if companion_name == "rtasks" {
                                                        "RTasks"
                                                    } else {
                                                        "rSnip"
                                                    }
                                                ),
                                                InputAccessoryKind::Info,
                                            );
                                        }
                                    }
                                    refresh_window(hwnd, app_state);
                                    unsafe {
                                        SetTimer(hwnd, INSTALL_START_TIMER_ID, 350, None);
                                    }
                                    return LRESULT(0);
                                }

                                let mut runtime_guard = MODULE_RUNTIME.lock().unwrap();
                                if let Some(runtime) = runtime_guard.as_mut() {
                                    if runtime.dispatch_command(
                                        app_state,
                                        command,
                                        &args,
                                        app_state.silent_mode,
                                    ) {
                                        refresh_window(hwnd, app_state);
                                        return LRESULT(0);
                                    }
                                }
                            }
                        } else if !app_state.matching_items.is_empty()
                            && app_state.selected_index < app_state.matching_items.len()
                        {
                            let selected =
                                app_state.matching_items[app_state.selected_index].clone();
                            if app_state.launcher_mode {
                                if let Err(e) = launch_target(&selected.target) {
                                    if !app_state.silent_mode {
                                        eprintln!(
                                            "Error launching target '{}': {}",
                                            selected.target, e
                                        );
                                    }
                                } else {
                                    persist_history_entry(
                                        &selected.target,
                                        app_state.silent_mode,
                                        app_state.history_max_items,
                                    );
                                }
                            } else {
                                println!("{}", selected.label);
                            }
                        } else if app_state.launcher_mode {
                            if let Err(e) = launch_target(&app_state.current_input) {
                                if !app_state.silent_mode {
                                    eprintln!(
                                        "Error launching input '{}': {}",
                                        app_state.current_input, e
                                    );
                                }
                            } else {
                                persist_history_entry(
                                    &app_state.current_input,
                                    app_state.silent_mode,
                                    app_state.history_max_items,
                                );
                            }
                        } else if app_state.all_items.is_empty() {
                            println!("{}", app_state.current_input);
                        }
                    }
                    request_ui_exit(hwnd, 0);
                } else if key_code == VK_DOWN.0 as i32 {
                    if !app_state.matching_items.is_empty() {
                        app_state.selected_index =
                            (app_state.selected_index + 1) % app_state.matching_items.len();
                        let max_visible = {
                            let config_guard = CONFIG.lock().unwrap();
                            config_guard
                                .as_ref()
                                .map_or(10usize, |c| c.behavior.max_items.max(1) as usize)
                        };
                        ensure_selection_visible(app_state, max_visible);
                        InvalidateRect(hwnd, None, true);
                    }
                } else if key_code == VK_UP.0 as i32 {
                    if !app_state.matching_items.is_empty() {
                        app_state.selected_index =
                            (app_state.selected_index + app_state.matching_items.len() - 1)
                                % app_state.matching_items.len();
                        let max_visible = {
                            let config_guard = CONFIG.lock().unwrap();
                            config_guard
                                .as_ref()
                                .map_or(10usize, |c| c.behavior.max_items.max(1) as usize)
                        };
                        ensure_selection_visible(app_state, max_visible);
                        InvalidateRect(hwnd, None, true);
                    }
                } else if key_code == VK_BACK.0 as i32 {
                    if !app_state.current_input.is_empty() {
                        app_state.current_input.pop();
                        app_state.selected_index = 0;
                        update_matching_items_from_config(app_state);
                        refresh_window(hwnd, app_state);
                    }
                } else if key_code == VK_TAB.0 as i32 {
                    if !app_state.matching_items.is_empty()
                        && app_state.selected_index < app_state.matching_items.len()
                    {
                        app_state.current_input = app_state.matching_items
                            [app_state.selected_index]
                            .label
                            .clone();
                        update_matching_items_from_config(app_state);
                        refresh_window(hwnd, app_state);
                    }
                }
            }
            drop(app_state_guard);
            LRESULT(0)
        }
        WM_CHAR => {
            let char_code = w_param.0 as u16;
            if char_code >= ' ' as u16 {
                if char_code == ' ' as u16
                    && SUPPRESS_NEXT_RMODS_SPACE_CHAR.swap(false, Ordering::Relaxed)
                {
                    return LRESULT(0);
                }
                if let Ok(mut app_state_guard) = APP_STATE.lock() {
                    if let Some(app_state) = app_state_guard.as_mut() {
                        let selected_rmod_id = if is_rmods_input(&app_state.current_input) {
                            selected_rmods_item_id(app_state)
                        } else {
                            None
                        };
                        if let Some(char_val) = std::char::from_u32(char_code as u32) {
                            if app_state.current_input.eq_ignore_ascii_case("/rmods")
                                && !char_val.is_whitespace()
                            {
                                app_state.current_input.push(' ');
                            }
                            app_state.current_input.push(char_val);
                        }
                        app_state.selected_index = 0;
                        update_matching_items_from_config(app_state);
                        if let Some(selected_id) = selected_rmod_id {
                            restore_rmods_selection(app_state, &selected_id);
                        }
                        refresh_window(hwnd, app_state);
                    }
                }
            }
            LRESULT(0)
        }
        WM_INSTALL_PROGRESS => {
            if let Ok(app_state_guard) = APP_STATE.lock() {
                if let Some(app_state) = app_state_guard.as_ref() {
                    refresh_window(hwnd, app_state);
                }
            }
            LRESULT(0)
        }
        WM_INSTALL_DONE => {
            if let Ok(app_state_guard) = APP_STATE.lock() {
                if let Some(app_state) = app_state_guard.as_ref() {
                    refresh_window(hwnd, app_state);
                }
            }
            unsafe {
                SetTimer(hwnd, INSTALL_CLOSE_TIMER_ID, 1_000, None);
            }
            LRESULT(0)
        }
        WM_TIMER => {
            if w_param.0 == INSTALL_START_TIMER_ID {
                unsafe {
                    KillTimer(hwnd, INSTALL_START_TIMER_ID);
                }
                let companion_name = if let Ok(app_state_guard) = APP_STATE.lock() {
                    if let Some(app_state) = app_state_guard.as_ref() {
                        let companion_name = app_state
                            .current_input
                            .split_whitespace()
                            .nth(1)
                            .map(|value| value.to_ascii_lowercase())
                            .unwrap_or_else(|| "rsnip".to_string());
                        {
                            let mut runtime_guard = MODULE_RUNTIME.lock().unwrap();
                            if let Some(runtime) = runtime_guard.as_mut() {
                                let installing_text = if companion_name == "rtasks" {
                                    "Installing RTasks"
                                } else {
                                    "Installing rSnip"
                                };
                                runtime.set_runtime_feedback(
                                    installing_text,
                                    InputAccessoryKind::Info,
                                );
                            }
                        }
                        refresh_window(hwnd, app_state);
                        companion_name
                    } else {
                        "rsnip".to_string()
                    }
                } else {
                    "rsnip".to_string()
                };

                let hwnd_raw = hwnd.0;
                std::thread::spawn(move || {
                    let hwnd = HWND(hwnd_raw);
                    let install_result = if companion_name == "rtasks" {
                        install_rtasks_latest()
                            .map(|_| "RTasks installed as rMenu companion")
                            .map_err(|err| format!("RTasks install failed: {err:?}"))
                    } else {
                        install_rsnip_latest()
                            .map(|_| "rSnip installed as rMenu companion")
                            .map_err(|err| format!("rSnip install failed: {err:?}"))
                    };
                    {
                        let mut runtime_guard = MODULE_RUNTIME.lock().unwrap();
                        if let Some(runtime) = runtime_guard.as_mut() {
                            match install_result {
                                Ok(message) => runtime
                                    .set_runtime_feedback(message, InputAccessoryKind::Success),
                                Err(message) => {
                                    runtime.set_runtime_feedback(message, InputAccessoryKind::Error)
                                }
                            }
                        }
                    }
                    let _ = unsafe { PostMessageW(hwnd, WM_INSTALL_DONE, WPARAM(0), LPARAM(0)) };
                });
            } else if w_param.0 == INSTALL_CLOSE_TIMER_ID {
                request_ui_exit(hwnd, 0);
            }
            LRESULT(0)
        }
        WM_DESTROY => {
            if UI_EMBEDDED_MODE.load(Ordering::SeqCst) {
                return LRESULT(0);
            }

            let mut app_state_guard = APP_STATE.lock().unwrap();
            let mut runtime_guard = MODULE_RUNTIME.lock().unwrap();
            if let (Some(app_state), Some(runtime)) =
                (app_state_guard.as_mut(), runtime_guard.as_mut())
            {
                runtime.run_on_unload(app_state);
            }
            *runtime_guard = None;
            PostQuitMessage(current_ui_exit_code());
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, w_param, l_param),
    }
}

fn finish_ui_run_timing(
    timings: Option<&mut UiRunTimings>,
    run_started_at: Instant,
    message_loop_started_at: Option<Instant>,
) {
    let Some(timings) = timings else {
        let mut trace_guard = UI_RUN_TIMING_TRACE.lock().unwrap();
        *trace_guard = None;
        return;
    };

    if let Some(loop_started_at) = message_loop_started_at {
        timings.message_loop_ms = loop_started_at.elapsed().as_millis();
    }
    timings.total_ms = run_started_at.elapsed().as_millis();

    let mut trace_guard = UI_RUN_TIMING_TRACE.lock().unwrap();
    if let Some(trace) = trace_guard.as_ref() {
        timings.time_to_first_paint_ms = trace.time_to_first_paint_ms.unwrap_or(0);
        timings.time_to_input_ready_ms = trace.time_to_input_ready_ms.unwrap_or(0);
    }
    *trace_guard = None;
}

fn run_ui_internal(
    cmd_options: &CmdOptions,
    config: &RmenuConfig,
    initial_app_state: AppState,
    mut module_runtime: ModuleRuntime,
    measure_mode: bool,
    embedded_mode: bool,
    mut run_timings: Option<&mut UiRunTimings>,
) -> windows::core::Result<i32> {
    let run_started_at = Instant::now();
    if run_timings.is_some() {
        let mut trace_guard = UI_RUN_TIMING_TRACE.lock().unwrap();
        *trace_guard = Some(UiRunTimingTrace {
            started_at: run_started_at,
            time_to_first_paint_ms: None,
            time_to_input_ready_ms: None,
        });
    }

    UI_EMBEDDED_MODE.store(embedded_mode, Ordering::SeqCst);
    UI_EXIT_CODE.store(0, Ordering::SeqCst);
    {
        let mut config_guard = CONFIG.lock().unwrap();
        *config_guard = Some(config.clone());
    }
    {
        let mut app_state_guard = APP_STATE.lock().unwrap();
        *app_state_guard = Some(initial_app_state);
    }
    module_runtime.clear_runtime_feedback();
    {
        let mut app_state_guard = APP_STATE.lock().unwrap();
        if let Some(app_state) = app_state_guard.as_mut() {
            let on_load_started_at = Instant::now();
            module_runtime.run_on_load(app_state);
            if let Some(timings) = run_timings.as_deref_mut() {
                timings.module_on_load_ms = on_load_started_at.elapsed().as_millis();
            }

            let matching_started_at = Instant::now();
            if app_state.current_input.trim().is_empty() {
                if !render_startup_update_notice(app_state) {
                    app_state.matching_items.clear();
                    app_state.selected_index = 0;
                    app_state.scroll_offset = 0;
                }
            } else {
                update_matching_items_from_config(app_state);
            }
            if let Some(timings) = run_timings.as_deref_mut() {
                timings.initial_matching_update_ms = matching_started_at.elapsed().as_millis();
            }
        }
    }
    {
        let mut runtime_guard = MODULE_RUNTIME.lock().unwrap();
        *runtime_guard = Some(module_runtime);
    }
    if let Some(timings) = run_timings.as_deref_mut() {
        timings.pre_window_setup_ms = run_started_at.elapsed().as_millis();
    }
    {
        let mut measure_guard = UI_MEASURE_STATE.lock().unwrap();
        *measure_guard = if measure_mode {
            UiMeasureState::enabled_now()
        } else {
            UiMeasureState::disabled()
        };
    }

    unsafe {
        let class_name_w = to_wstring("rmenu_class_layout");
        let window_title_w = to_wstring("rMenu");

        let register_started_at = Instant::now();
        let wc = WNDCLASSW {
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(window_proc),
            hInstance: GetModuleHandleW(None)?.into(),
            lpszClassName: PCWSTR(class_name_w.as_ptr()),
            hCursor: LoadCursorW(None, IDC_ARROW)?,
            ..Default::default()
        };
        RegisterClassW(&wc);
        if let Some(timings) = run_timings.as_deref_mut() {
            timings.register_class_ms = register_started_at.elapsed().as_millis();
        }

        let num_items_for_geom = {
            let app_state_guard = APP_STATE.lock().unwrap();
            app_state_guard
                .as_ref()
                .map_or(0, |state| visible_item_count(state, config))
        };

        let geometry = determine_window_geometry(cmd_options, config, num_items_for_geom);

        let ex_style = if measure_mode {
            WINDOW_EX_STYLE(WS_EX_TOOLWINDOW.0)
        } else {
            WINDOW_EX_STYLE(WS_EX_TOPMOST.0 | WS_EX_TOOLWINDOW.0 | WS_EX_APPWINDOW.0)
        };

        let create_window_started_at = Instant::now();
        let hwnd = CreateWindowExW(
            ex_style,
            PCWSTR(class_name_w.as_ptr()),
            PCWSTR(window_title_w.as_ptr()),
            WS_POPUP | WS_VISIBLE,
            geometry.x,
            geometry.y,
            geometry.width,
            geometry.height,
            None,
            None,
            GetModuleHandleW(None)?,
            None,
        );

        if let Some(timings) = run_timings.as_deref_mut() {
            timings.create_window_ms = create_window_started_at.elapsed().as_millis();
        }

        if hwnd.0 == 0 {
            if !cmd_options.silent {
                eprintln!("Error creating window");
            }
            finish_ui_run_timing(run_timings.as_deref_mut(), run_started_at, None);
            return Ok(0);
        }

        ShowWindow(hwnd, SW_SHOW);
        if let Some(timings) = run_timings.as_deref_mut() {
            timings.time_to_window_visible_ms = run_started_at.elapsed().as_millis();
        }
        mark_window_visible_metric();
        if !measure_mode {
            SetForegroundWindow(hwnd);
        }
        InvalidateRect(hwnd, None, true);

        let message_loop_started_at = Instant::now();
        let mut msg = MSG::default();
        loop {
            match PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).into() {
                BOOL(0) => {
                    if embedded_mode && IsWindow(hwnd).0 == 0 {
                        let exit_code = current_ui_exit_code();
                        finish_ui_run_timing(
                            run_timings.as_deref_mut(),
                            run_started_at,
                            Some(message_loop_started_at),
                        );
                        return Ok(exit_code);
                    }

                    let mut should_repaint = false;
                    {
                        let mut runtime_guard = MODULE_RUNTIME.lock().unwrap();
                        if let Some(runtime) = runtime_guard.as_mut() {
                            if runtime.poll_hot_reload(cmd_options.silent) {
                                should_repaint = true;
                            }
                        }
                    }

                    if should_repaint {
                        let mut app_state_guard = APP_STATE.lock().unwrap();
                        if let Some(app_state) = app_state_guard.as_mut() {
                            update_matching_items_from_config(app_state);
                        }
                        InvalidateRect(hwnd, None, true);
                    }

                    std::thread::yield_now();
                    continue;
                }
                _ => {
                    if msg.message == WM_QUIT {
                        let exit_code = msg.wParam.0 as i32;
                        finish_ui_run_timing(
                            run_timings.as_deref_mut(),
                            run_started_at,
                            Some(message_loop_started_at),
                        );
                        return Ok(exit_code);
                    }
                    TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                    if embedded_mode && IsWindow(hwnd).0 == 0 {
                        let exit_code = current_ui_exit_code();
                        finish_ui_run_timing(
                            run_timings.as_deref_mut(),
                            run_started_at,
                            Some(message_loop_started_at),
                        );
                        return Ok(exit_code);
                    }
                }
            }
        }
    }
}

pub fn run_ui(
    cmd_options: &CmdOptions,
    config: &RmenuConfig,
    initial_app_state: AppState,
    module_runtime: ModuleRuntime,
) -> windows::core::Result<i32> {
    run_ui_internal(
        cmd_options,
        config,
        initial_app_state,
        module_runtime,
        false,
        false,
        None,
    )
}

#[allow(dead_code)]
pub fn run_ui_embedded(
    cmd_options: &CmdOptions,
    config: &RmenuConfig,
    initial_app_state: AppState,
    module_runtime: ModuleRuntime,
) -> windows::core::Result<i32> {
    run_ui_internal(
        cmd_options,
        config,
        initial_app_state,
        module_runtime,
        false,
        true,
        None,
    )
}

#[allow(dead_code)]
pub fn run_ui_embedded_timed(
    cmd_options: &CmdOptions,
    config: &RmenuConfig,
    initial_app_state: AppState,
    module_runtime: ModuleRuntime,
) -> windows::core::Result<(i32, UiRunTimings)> {
    let mut timings = UiRunTimings::default();
    let exit_code = run_ui_internal(
        cmd_options,
        config,
        initial_app_state,
        module_runtime,
        false,
        true,
        Some(&mut timings),
    )?;
    Ok((exit_code, timings))
}

#[allow(dead_code)]
pub fn take_module_runtime() -> Option<ModuleRuntime> {
    let mut runtime_guard = MODULE_RUNTIME.lock().unwrap();
    runtime_guard.take()
}

pub fn measure_ui_latencies(
    cmd_options: &CmdOptions,
    config: &RmenuConfig,
    initial_app_state: AppState,
    module_runtime: ModuleRuntime,
) -> windows::core::Result<UiLatencyMetrics> {
    let _ = run_ui_internal(
        cmd_options,
        config,
        initial_app_state,
        module_runtime,
        true,
        false,
        None,
    )?;

    let mut measure_guard = UI_MEASURE_STATE.lock().unwrap();
    let metrics = UiLatencyMetrics {
        time_to_window_visible_ms: measure_guard.time_to_window_visible_ms.unwrap_or(0),
        time_to_first_paint_ms: measure_guard.time_to_first_paint_ms.unwrap_or(0),
        time_to_input_ready_ms: measure_guard.time_to_input_ready_ms.unwrap_or(0),
    };
    *measure_guard = UiMeasureState::disabled();

    Ok(metrics)
}

#[cfg(test)]
mod tests {
    use super::{
        compute_row_zones, dismiss_startup_update_notice, find_quick_select_index,
        normalize_quick_select_items, render_startup_update_notice, rmods_badge_tone,
        rmods_status_tone,
    };
    use crate::app_state::{
        AppState, LauncherItem, LauncherItemTone, LauncherSource, RmodsInstallStatusView,
        StartupUpdateNotice,
    };

    #[test]
    fn startup_update_notice_renders_and_dismisses_for_current_open() {
        let mut state = AppState {
            startup_update_notice: Some(StartupUpdateNotice {
                version: "0.3.1".to_string(),
                release_url: "https://github.com/SynrgStudio/rmenu/releases/tag/v0.3.1".to_string(),
                installer_asset_url: Some(
                    "https://example.test/rmenu-setup-v0.3.1.exe".to_string(),
                ),
                checksums_asset_url: Some("https://example.test/SHA256SUMS.txt".to_string()),
                data_dir: Some(r"C:\rMenuData".to_string()),
            }),
            ..Default::default()
        };

        assert!(render_startup_update_notice(&mut state));
        assert_eq!(state.matching_items.len(), 1);
        assert_eq!(state.matching_items[0].target, "update:install");
        assert_eq!(
            state.matching_items[0].trailing_badge_tone,
            Some(LauncherItemTone::Warning)
        );

        assert!(dismiss_startup_update_notice(&mut state));
        assert_eq!(state.startup_update_notice, None);
    }

    #[test]
    fn quick_select_conflicts_keep_first_visible_item() {
        let mut first = LauncherItem::new(
            "First".to_string(),
            "t1".to_string(),
            LauncherSource::Direct,
        );
        first.quick_select_key = Some("1".to_string());
        first.trailing_badge = Some("1".to_string());

        let mut second = LauncherItem::new(
            "Second".to_string(),
            "t2".to_string(),
            LauncherSource::Direct,
        );
        second.quick_select_key = Some("1".to_string());
        second.trailing_badge = Some("1".to_string());
        second.trailing_badge_tone = Some(LauncherItemTone::Warning);

        let mut state = AppState {
            matching_items: vec![first, second],
            ..Default::default()
        };

        normalize_quick_select_items(&mut state);

        assert_eq!(
            state.matching_items[0].quick_select_key.as_deref(),
            Some("1")
        );
        assert_eq!(state.matching_items[1].quick_select_key, None);
        assert_eq!(state.matching_items[1].trailing_badge, None);
        assert_eq!(state.matching_items[1].trailing_badge_tone, None);
    }

    #[test]
    fn rmods_badge_tone_is_derived_from_rmods_badge_text_after_interaction() {
        let mut item = LauncherItem::new(
            "[x] calculator 0.1.0".to_string(),
            "rmods:calculator".to_string(),
            LauncherSource::Direct,
        );
        item.trailing_badge = Some("installed".to_string());

        assert_eq!(rmods_badge_tone(&item), Some(LauncherItemTone::Success));

        item.trailing_badge = Some("not installed".to_string());
        assert_eq!(rmods_badge_tone(&item), Some(LauncherItemTone::Danger));

        item.trailing_badge = Some("update available".to_string());
        assert_eq!(rmods_badge_tone(&item), Some(LauncherItemTone::Warning));
    }

    #[test]
    fn rmods_status_tones_match_install_state_colors() {
        assert_eq!(
            rmods_status_tone(RmodsInstallStatusView::Installed),
            Some(LauncherItemTone::Success)
        );
        assert_eq!(
            rmods_status_tone(RmodsInstallStatusView::NotInstalled),
            Some(LauncherItemTone::Danger)
        );
        assert_eq!(
            rmods_status_tone(RmodsInstallStatusView::UpdateAvailable),
            Some(LauncherItemTone::Warning)
        );
    }

    #[test]
    fn quick_select_index_is_resolved_from_visible_window() {
        let mut items = Vec::new();
        for idx in 0..6 {
            let mut item = LauncherItem::new(
                format!("Item {idx}"),
                format!("target-{idx}"),
                LauncherSource::Direct,
            );
            if idx == 4 {
                item.quick_select_key = Some("5".to_string());
            }
            items.push(item);
        }

        let state = AppState {
            matching_items: items,
            scroll_offset: 2,
            ..Default::default()
        };

        let index = find_quick_select_index(&state, '5', 3);
        assert_eq!(index, Some(4));

        let hidden = find_quick_select_index(&state, '5', 2);
        assert_eq!(hidden, None);
    }

    #[test]
    fn row_zones_hide_hint_and_chip_on_extreme_width() {
        let row = compute_row_zones(
            "Very long label value",
            "C:/a/very/long/path/hint",
            Some("[1]".to_string()),
            0,
            10,
            2,
        );

        assert!(row.right_text.is_empty());
        assert!(row.chip_text.is_none());
        assert!(!row.left_text.is_empty());
    }

    #[test]
    fn row_zones_keep_all_zones_when_width_is_enough() {
        let row = compute_row_zones(
            "Calculator",
            "\\Windows\\System32\\calc.exe",
            Some("[1]".to_string()),
            0,
            120,
            2,
        );

        assert!(!row.left_text.is_empty());
        assert!(!row.right_text.is_empty());
        assert_eq!(row.chip_text.as_deref(), Some("[1]"));
        assert!(row.right_x >= 0);
        assert!(row.chip_x >= 0);
    }
}
