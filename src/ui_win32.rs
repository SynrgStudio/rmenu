use crate::app_state::{ensure_selection_visible, AppState};
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
use crate::settings::{CmdOptions, QuickSelectMode, RmenuConfig};
use crate::sources::persist_history_entry;
use std::ffi::OsStr;
use std::iter::once;
use std::os::windows::ffi::OsStrExt;
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
                VK_TAB, VK_UP,
            },
            WindowsAndMessaging::{
                CreateWindowExW, DefWindowProcW, DispatchMessageW, GetClientRect, GetSystemMetrics,
                LoadCursorW, PeekMessageW, PostQuitMessage, RegisterClassW, SetForegroundWindow,
                ShowWindow, TranslateMessage, CS_HREDRAW, CS_VREDRAW, IDC_ARROW, MSG, PM_REMOVE,
                SM_CXSCREEN, SM_CYSCREEN, SW_SHOW, WINDOW_EX_STYLE, WM_CHAR, WM_CREATE, WM_DESTROY,
                WM_KEYDOWN, WM_PAINT, WM_QUIT, WNDCLASSW, WS_EX_APPWINDOW, WS_EX_TOOLWINDOW,
                WS_EX_TOPMOST, WS_POPUP, WS_VISIBLE,
            },
        },
    },
};

static APP_STATE: Mutex<Option<AppState>> = Mutex::new(None);
static CONFIG: Mutex<Option<RmenuConfig>> = Mutex::new(None);
static MODULE_RUNTIME: Mutex<Option<ModuleRuntime>> = Mutex::new(None);
static UI_MEASURE_STATE: Mutex<UiMeasureState> = Mutex::new(UiMeasureState::disabled());

#[derive(Debug, Clone, Copy)]
pub struct UiLatencyMetrics {
    pub time_to_window_visible_ms: u128,
    pub time_to_first_paint_ms: u128,
    pub time_to_input_ready_ms: u128,
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
    let mut state = UI_MEASURE_STATE.lock().unwrap();
    if !state.enabled {
        return;
    }
    let Some(started_at) = state.started_at else {
        return;
    };
    let elapsed_ms = started_at.elapsed().as_millis();

    if state.time_to_first_paint_ms.is_none() {
        state.time_to_first_paint_ms = Some(elapsed_ms);
    }
    if state.time_to_input_ready_ms.is_none() {
        state.time_to_input_ready_ms = Some(elapsed_ms);
    }
}

fn is_measure_mode_enabled() -> bool {
    UI_MEASURE_STATE.lock().unwrap().enabled
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
    let list_height = (num_items_to_show as i32 * item_h) + (2 * padding);
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

fn update_matching_items_from_config(app_state: &mut AppState) {
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

                if !app_state.current_input.is_empty() {
                    draw_text_w(hdc, x_offset, input_text_y, &app_state.current_input);
                }

                let accessory = {
                    let runtime_guard = MODULE_RUNTIME.lock().unwrap();
                    runtime_guard
                        .as_ref()
                        .and_then(|runtime| runtime.active_input_accessory())
                };

                if let Some(accessory) = accessory {
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
                        draw_text_w(hdc, row.chip_x.max(left_x + char_w), item_text_y, &chip);
                    }
                }

                SelectObject(hdc, old_font);
            }
            EndPaint(hwnd, &ps);

            if is_measure_mode_enabled() {
                mark_first_paint_and_input_ready_metrics();
                PostQuitMessage(0);
            }

            LRESULT(0)
        }
        WM_KEYDOWN => {
            let key_code = w_param.0 as i32;
            let mut app_state_guard = APP_STATE.lock().unwrap();
            if let Some(app_state) = app_state_guard.as_mut() {
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

                if key_code == VK_ESCAPE.0 as i32 {
                    PostQuitMessage(1);
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
                            PostQuitMessage(0);
                        } else {
                            InvalidateRect(hwnd, None, true);
                        }
                    }
                } else if key_code == VK_RETURN.0 as i32 {
                    if !app_state.matching_items.is_empty()
                        && app_state.selected_index < app_state.matching_items.len()
                    {
                        let selected = app_state.matching_items[app_state.selected_index].clone();
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
                    } else if !app_state.current_input.is_empty() {
                        let current_input = app_state.current_input.clone();
                        if let Some(raw_command) = current_input.strip_prefix('/') {
                            let parts = raw_command.split_whitespace().collect::<Vec<_>>();
                            if let Some((command, rest)) = parts.split_first() {
                                let args =
                                    rest.iter().map(|v| (*v).to_string()).collect::<Vec<_>>();
                                let mut runtime_guard = MODULE_RUNTIME.lock().unwrap();
                                if let Some(runtime) = runtime_guard.as_mut() {
                                    runtime.dispatch_command(
                                        app_state,
                                        command,
                                        &args,
                                        app_state.silent_mode,
                                    );
                                }
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
                    PostQuitMessage(0);
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
                        InvalidateRect(hwnd, None, true);
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
                        InvalidateRect(hwnd, None, true);
                    }
                }
            }
            drop(app_state_guard);
            LRESULT(0)
        }
        WM_CHAR => {
            let char_code = w_param.0 as u16;
            if char_code >= ' ' as u16 {
                if let Ok(mut app_state_guard) = APP_STATE.lock() {
                    if let Some(app_state) = app_state_guard.as_mut() {
                        if let Some(char_val) = std::char::from_u32(char_code as u32) {
                            app_state.current_input.push(char_val);
                        }
                        app_state.selected_index = 0;
                        update_matching_items_from_config(app_state);
                        InvalidateRect(hwnd, None, true);
                    }
                }
            }
            LRESULT(0)
        }
        WM_DESTROY => {
            let mut app_state_guard = APP_STATE.lock().unwrap();
            let mut runtime_guard = MODULE_RUNTIME.lock().unwrap();
            if let (Some(app_state), Some(runtime)) =
                (app_state_guard.as_mut(), runtime_guard.as_mut())
            {
                runtime.run_on_unload(app_state);
            }
            *runtime_guard = None;
            PostQuitMessage(0);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, w_param, l_param),
    }
}

fn run_ui_internal(
    cmd_options: &CmdOptions,
    config: &RmenuConfig,
    initial_app_state: AppState,
    mut module_runtime: ModuleRuntime,
    measure_mode: bool,
) -> windows::core::Result<i32> {
    {
        let mut config_guard = CONFIG.lock().unwrap();
        *config_guard = Some(config.clone());
    }
    {
        let mut app_state_guard = APP_STATE.lock().unwrap();
        *app_state_guard = Some(initial_app_state);
    }
    {
        let mut app_state_guard = APP_STATE.lock().unwrap();
        if let Some(app_state) = app_state_guard.as_mut() {
            module_runtime.run_on_load(app_state);
            update_matching_items_from_config(app_state);
        }
    }
    {
        let mut runtime_guard = MODULE_RUNTIME.lock().unwrap();
        *runtime_guard = Some(module_runtime);
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
        let window_title_w = to_wstring("rmenu");

        let wc = WNDCLASSW {
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(window_proc),
            hInstance: GetModuleHandleW(None)?.into(),
            lpszClassName: PCWSTR(class_name_w.as_ptr()),
            hCursor: LoadCursorW(None, IDC_ARROW)?,
            ..Default::default()
        };
        RegisterClassW(&wc);

        let num_items_for_geom = {
            let app_state_guard = APP_STATE.lock().unwrap();
            app_state_guard.as_ref().map_or(0, |s| {
                s.matching_items
                    .len()
                    .min(config.behavior.max_items as usize)
            })
        };

        let geometry = determine_window_geometry(cmd_options, config, num_items_for_geom);

        let ex_style = if measure_mode {
            WINDOW_EX_STYLE(WS_EX_TOOLWINDOW.0)
        } else {
            WINDOW_EX_STYLE(WS_EX_TOPMOST.0 | WS_EX_TOOLWINDOW.0 | WS_EX_APPWINDOW.0)
        };

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

        if hwnd.0 == 0 {
            if !cmd_options.silent {
                eprintln!("Error creating window");
            }
            return Ok(0);
        }

        ShowWindow(hwnd, SW_SHOW);
        mark_window_visible_metric();
        if !measure_mode {
            SetForegroundWindow(hwnd);
        }
        InvalidateRect(hwnd, None, true);

        let mut msg = MSG::default();
        loop {
            match PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).into() {
                BOOL(0) => {
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
                        return Ok(msg.wParam.0 as i32);
                    }
                    TranslateMessage(&msg);
                    DispatchMessageW(&msg);
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
    )
}

pub fn measure_ui_latencies(
    cmd_options: &CmdOptions,
    config: &RmenuConfig,
    initial_app_state: AppState,
    module_runtime: ModuleRuntime,
) -> windows::core::Result<UiLatencyMetrics> {
    let _ = run_ui_internal(cmd_options, config, initial_app_state, module_runtime, true)?;

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
    use super::{compute_row_zones, find_quick_select_index, normalize_quick_select_items};
    use crate::app_state::{AppState, LauncherItem, LauncherSource};

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
