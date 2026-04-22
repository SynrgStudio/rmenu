use crate::app_state::{ensure_selection_visible, AppState};
use crate::launcher::{
    abbreviate_target, centered_text_y, compact_target_hint, launch_target, truncate_with_ellipsis_end,
};
use crate::ranking::update_matching_items;
use crate::settings::{CmdOptions, RmenuConfig};
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
        Foundation::{BOOL, HWND, LPARAM, LRESULT, RECT, WPARAM},
        Graphics::Gdi::{
            BeginPaint, CreateFontW, CreateSolidBrush, EndPaint, FillRect, InvalidateRect, PAINTSTRUCT,
            SelectObject, SetBkColor, SetTextColor, TextOutW,
        },
        System::LibraryLoader::GetModuleHandleW,
        UI::{
            Input::KeyboardAndMouse::{VK_BACK, VK_DOWN, VK_ESCAPE, VK_RETURN, VK_TAB, VK_UP},
            WindowsAndMessaging::{
                CreateWindowExW, DefWindowProcW, DispatchMessageW, GetClientRect, GetSystemMetrics,
                LoadCursorW, PeekMessageW, PostQuitMessage, RegisterClassW, SetForegroundWindow,
                ShowWindow, TranslateMessage, CS_HREDRAW, CS_VREDRAW, IDC_ARROW,
                MSG, PM_REMOVE, SM_CXSCREEN, SM_CYSCREEN, SW_SHOW, WINDOW_EX_STYLE, WM_CHAR, WM_CREATE,
                WM_DESTROY, WM_KEYDOWN, WM_PAINT, WM_QUIT, WNDCLASSW, WS_EX_APPWINDOW, WS_EX_TOOLWINDOW,
                WS_EX_TOPMOST, WS_POPUP, WS_VISIBLE,
            },
        },
    },
};

static APP_STATE: Mutex<Option<AppState>> = Mutex::new(None);
static CONFIG: Mutex<Option<RmenuConfig>> = Mutex::new(None);
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

fn calculate_position_detailed(position_str: &str, screen_dimension: i32, window_dimension: i32) -> i32 {
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

    let final_layout_str = cmd_opts
        .layout
        .as_deref()
        .unwrap_or_else(|| config.dimensions.default_layout.as_deref().unwrap_or("custom"));

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

    WindowGeometry { x, y, width: w, height: h }
}

fn update_matching_items_from_config(app_state: &mut AppState) {
    let config_guard = CONFIG.lock().unwrap();
    let case_sensitive = config_guard.as_ref().map_or(false, |c| c.behavior.case_sensitive);
    let max_visible_items = config_guard
        .as_ref()
        .map_or(10usize, |c| c.behavior.max_items.max(1) as usize);
    drop(config_guard);

    update_matching_items(app_state, case_sensitive, max_visible_items);
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
                    FillRect(hdc, &RECT { left: 0, top: 0, right: rect.right, bottom: bw }, border_brush);
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

                let current_item_height = config.dimensions.item_height;
                let max_items_to_display = config.behavior.max_items.max(1) as usize;

                let visible_end = (app_state.scroll_offset + max_items_to_display).min(app_state.matching_items.len());

                for (visible_row, item_index) in (app_state.scroll_offset..visible_end).enumerate() {
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

                    let min_gap = char_w * 2;
                    let left_x = current_padding;
                    let row_right_bound = rect.right - current_padding;

                    let mut right_text = compact_target_hint(&item.target);
                    let mut right_chars = right_text.chars().count() as i32;
                    let mut right_w = right_chars * char_w;
                    let mut right_x = row_right_bound - right_w;

                    let max_left_w = (right_x - min_gap - left_x).max(char_w * 8);
                    let left_max_chars = (max_left_w / char_w).max(1) as usize;
                    let left_text = truncate_with_ellipsis_end(&item.label, left_max_chars);
                    let left_w = left_text.chars().count() as i32 * char_w;

                    if right_x <= left_x + left_w + min_gap {
                        let available_right_w = row_right_bound - (left_x + left_w + min_gap);
                        let available_right_chars = (available_right_w / char_w).max(0) as usize;
                        if available_right_chars >= 4 {
                            right_text = abbreviate_target(&right_text, available_right_chars);
                            right_chars = right_text.chars().count() as i32;
                            right_w = right_chars * char_w;
                            right_x = row_right_bound - right_w;
                        } else {
                            right_text.clear();
                        }
                    }

                    let item_text_y = centered_text_y(y, current_item_height, config.font.size);
                    draw_text_w(hdc, left_x, item_text_y, &left_text);
                    if !right_text.is_empty() {
                        draw_text_w(hdc, right_x, item_text_y, &right_text);
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
                if key_code == VK_ESCAPE.0 as i32 {
                    PostQuitMessage(1);
                } else if key_code == VK_RETURN.0 as i32 {
                    if !app_state.matching_items.is_empty() && app_state.selected_index < app_state.matching_items.len() {
                        let selected = app_state.matching_items[app_state.selected_index].clone();
                        if app_state.launcher_mode {
                            if let Err(e) = launch_target(&selected.target) {
                                if !app_state.silent_mode {
                                    eprintln!("Error launching target '{}': {}", selected.target, e);
                                }
                            } else {
                                persist_history_entry(&selected.target, app_state.silent_mode, app_state.history_max_items);
                            }
                        } else {
                            println!("{}", selected.label);
                        }
                    } else if !app_state.current_input.is_empty() {
                        if app_state.launcher_mode {
                            if let Err(e) = launch_target(&app_state.current_input) {
                                if !app_state.silent_mode {
                                    eprintln!("Error launching input '{}': {}", app_state.current_input, e);
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
                        app_state.selected_index = (app_state.selected_index + 1) % app_state.matching_items.len();
                        let max_visible = {
                            let config_guard = CONFIG.lock().unwrap();
                            config_guard.as_ref().map_or(10usize, |c| c.behavior.max_items.max(1) as usize)
                        };
                        ensure_selection_visible(app_state, max_visible);
                        InvalidateRect(hwnd, None, true);
                    }
                } else if key_code == VK_UP.0 as i32 {
                    if !app_state.matching_items.is_empty() {
                        app_state.selected_index =
                            (app_state.selected_index + app_state.matching_items.len() - 1) % app_state.matching_items.len();
                        let max_visible = {
                            let config_guard = CONFIG.lock().unwrap();
                            config_guard.as_ref().map_or(10usize, |c| c.behavior.max_items.max(1) as usize)
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
                    if !app_state.matching_items.is_empty() && app_state.selected_index < app_state.matching_items.len() {
                        app_state.current_input = app_state.matching_items[app_state.selected_index].label.clone();
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
            app_state_guard
                .as_ref()
                .map_or(0, |s| s.matching_items.len().min(config.behavior.max_items as usize))
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

pub fn run_ui(cmd_options: &CmdOptions, config: &RmenuConfig, initial_app_state: AppState) -> windows::core::Result<i32> {
    run_ui_internal(cmd_options, config, initial_app_state, false)
}

pub fn measure_ui_latencies(
    cmd_options: &CmdOptions,
    config: &RmenuConfig,
    initial_app_state: AppState,
) -> windows::core::Result<UiLatencyMetrics> {
    let _ = run_ui_internal(cmd_options, config, initial_app_state, true)?;

    let mut measure_guard = UI_MEASURE_STATE.lock().unwrap();
    let metrics = UiLatencyMetrics {
        time_to_window_visible_ms: measure_guard.time_to_window_visible_ms.unwrap_or(0),
        time_to_first_paint_ms: measure_guard.time_to_first_paint_ms.unwrap_or(0),
        time_to_input_ready_ms: measure_guard.time_to_input_ready_ms.unwrap_or(0),
    };
    *measure_guard = UiMeasureState::disabled();

    Ok(metrics)
}
