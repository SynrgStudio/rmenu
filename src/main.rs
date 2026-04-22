mod app_state;
mod fuzzy;
mod launcher;
mod settings;
mod sources;

use atty;
use std::{
    ffi::OsStr,
    io::{self, Read},
    iter::once,
    os::windows::ffi::OsStrExt,
    path::Path,
    sync::Mutex,
    thread::sleep,
    time::{Duration, Instant},
};
use windows::{
    core::PCWSTR,
    Win32::{
        Foundation::{HWND, LPARAM, LRESULT, WPARAM, RECT, BOOL},
        System::LibraryLoader::GetModuleHandleW,
        UI::{
            Input::KeyboardAndMouse::{
                VK_ESCAPE, VK_RETURN, VK_DOWN, VK_UP, VK_TAB, VK_BACK
            },
            WindowsAndMessaging::{
                CreateWindowExW, DefWindowProcW, DispatchMessageW,
                LoadCursorW, RegisterClassW, ShowWindow,
                TranslateMessage, CS_HREDRAW, CS_VREDRAW, IDC_ARROW, MSG,
                SW_SHOW, WINDOW_EX_STYLE, WM_CREATE, WM_DESTROY, WM_KEYDOWN,
                WNDCLASSW, WS_POPUP, WS_VISIBLE, PostQuitMessage,
                GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN, WM_PAINT, WM_CHAR, 
                WS_EX_TOPMOST, WS_EX_TOOLWINDOW, GetClientRect, SetForegroundWindow,
                PeekMessageW, PM_REMOVE, WS_EX_APPWINDOW, WM_QUIT,
            },
        },
        Graphics::Gdi::{
            CreateFontW, SelectObject, SetBkColor, SetTextColor, 
            CreateSolidBrush, BeginPaint, EndPaint, PAINTSTRUCT,
            TextOutA, FillRect, InvalidateRect,
        },
    },
};

use app_state::{AppState, LauncherItem, LauncherSource, ensure_selection_visible, source_boost};
use fuzzy::{compact_lower_alnum, fuzzy_score, fuzzy_score_precomputed_lower};
use launcher::{abbreviate_target, centered_text_y, compact_target_hint, launch_target, truncate_with_ellipsis_end};
use settings::{CmdOptions, RmenuConfig, parse_args};
use sources::{index_cache_size_bytes, load_launcher_items, persist_history_entry};
static APP_STATE: Mutex<Option<AppState>> = Mutex::new(None);
static CONFIG: Mutex<Option<RmenuConfig>> = Mutex::new(None);

fn to_wstring(s: &str) -> Vec<u16> {
    OsStr::new(s).encode_wide().chain(once(0)).collect()
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

fn determine_window_geometry(cmd_opts: &CmdOptions, config: &RmenuConfig, num_items_to_show: usize, _silent_mode: bool) -> WindowGeometry {
    let screen_width = unsafe { GetSystemMetrics(SM_CXSCREEN) };
    let screen_height = unsafe { GetSystemMetrics(SM_CYSCREEN) };

    let final_layout_str = cmd_opts.layout.as_deref().unwrap_or_else(|| config.dimensions.default_layout.as_deref().unwrap_or("custom"));
    
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
            x = calculate_position_detailed(config.dimensions.x_position.as_deref().unwrap_or("r0.5"), screen_width, w);
            y = calculate_position_detailed(config.dimensions.y_position.as_deref().unwrap_or("r0.3"), screen_height, h);
        }
    }

    if let Some(cli_w_percent) = cmd_opts.cli_width_percent {
        w = (screen_width as f32 * cli_w_percent) as i32;
        if let Some(cli_max_w) = cmd_opts.cli_max_width.or(config.dimensions.max_width) {
             if w > cli_max_w { w = cli_max_w; }
        }
    } else if let Some(cli_max_w) = cmd_opts.cli_max_width { 
        if w > cli_max_w { w = cli_max_w; }
    }

    if let Some(cli_h) = cmd_opts.cli_height { h = cli_h; }

    if let Some(cli_x_str) = &cmd_opts.cli_x_pos {
        x = calculate_position_detailed(cli_x_str, screen_width, w);
    }
    if let Some(cli_y_str) = &cmd_opts.cli_y_pos {
        y = calculate_position_detailed(cli_y_str, screen_height, h);
    }
    
    WindowGeometry { x, y, width: w, height: h }
}

unsafe extern "system" fn window_proc(
    hwnd: HWND,
    msg: u32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    match msg {
        WM_CREATE => {
            sleep(Duration::from_millis(50));
            SetForegroundWindow(hwnd);
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
                    FillRect(hdc, &RECT { left: 0, top: rect.bottom - bw, right: rect.right, bottom: rect.bottom }, border_brush);
                    FillRect(hdc, &RECT { left: 0, top: bw, right: bw, bottom: rect.bottom - bw }, border_brush);
                    FillRect(hdc, &RECT { left: rect.right - bw, top: bw, right: rect.right, bottom: rect.bottom - bw }, border_brush);
                }
                
                let font = CreateFontW(
                    config.font.size, 0, 0, 0,
                    config.font.weight as i32, 0, 0, 0, 
                    0, 0, 0, 0, 0, 
                    config.font.name.as_ref().map_or(PCWSTR::null(), |name| PCWSTR(to_wstring(name).as_ptr()))
                );
                let old_font = SelectObject(hdc, font);
                
                let current_padding = config.dimensions.padding;
                let char_w = (config.font.size / 2).max(6);

                let input_bar_actual_height = config.dimensions.height;
                let input_text_y = centered_text_y(0, input_bar_actual_height, config.font.size);

                let mut x_offset = current_padding;
                if let Some(prompt) = &app_state.prompt {
                    let prompt_text = format!("{}: ", prompt);
                    TextOutA(hdc, x_offset, input_text_y, prompt_text.as_bytes());
                    x_offset += prompt_text.len() as i32 * char_w;
                }
                
                if !app_state.current_input.is_empty() {
                    TextOutA(hdc, x_offset, input_text_y, app_state.current_input.as_bytes());
                }
                
                let current_item_height = config.dimensions.item_height;
                let max_items_to_display = config.behavior.max_items.max(1) as usize;

                let visible_end = (app_state.scroll_offset + max_items_to_display)
                    .min(app_state.matching_items.len());

                for (visible_row, item_index) in (app_state.scroll_offset..visible_end).enumerate() {
                    let item = &app_state.matching_items[item_index];
                    let y = input_bar_actual_height + (current_item_height * visible_row as i32);
                    
                    if item_index == app_state.selected_index {
                        SetBkColor(hdc, config.colors.selected_background);
                        SetTextColor(hdc, config.colors.selected_foreground);
                        let select_rect = RECT { left: 0, top: y, right: rect.right, bottom: y + current_item_height };
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
                    TextOutA(hdc, left_x, item_text_y, left_text.as_bytes());
                    if !right_text.is_empty() {
                        TextOutA(hdc, right_x, item_text_y, right_text.as_bytes());
                    }
                }
                
                SelectObject(hdc, old_font);
            }
            EndPaint(hwnd, &ps);
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
                                persist_history_entry(&app_state.current_input, app_state.silent_mode, app_state.history_max_items);
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
                        app_state.selected_index = (app_state.selected_index + app_state.matching_items.len() - 1) % app_state.matching_items.len();
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
                        update_matching_items_refactored(app_state);
                        InvalidateRect(hwnd, None, true);
                    }
                } else if key_code == VK_TAB.0 as i32 {
                    if !app_state.matching_items.is_empty() && app_state.selected_index < app_state.matching_items.len() {
                        app_state.current_input = app_state.matching_items[app_state.selected_index].label.clone();
                        update_matching_items_refactored(app_state);
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
                        update_matching_items_refactored(app_state);
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

#[derive(Debug, Clone)]
struct RankedItem {
    item: LauncherItem,
    fuzzy_score: i64,
    source_boost: i64,
    total_score: i64,
}

const PARTIAL_TOPK_THRESHOLD: usize = 1200;
const PARTIAL_TOPK_LIMIT: usize = 400;

fn rank_compare_desc(a: &RankedItem, b: &RankedItem) -> std::cmp::Ordering {
    b.total_score
        .cmp(&a.total_score)
        .then_with(|| a.item.label.cmp(&b.item.label))
}

fn source_name(source: LauncherSource) -> &'static str {
    match source {
        LauncherSource::Direct => "direct",
        LauncherSource::History => "history",
        LauncherSource::StartMenu => "start_menu",
        LauncherSource::Path => "path",
    }
}

fn rank_items(app_state: &AppState, query: &str, case_sensitive: bool) -> Vec<RankedItem> {
    let query_norm = if case_sensitive {
        String::new()
    } else {
        query.to_lowercase()
    };
    let query_compact = if case_sensitive {
        String::new()
    } else {
        compact_lower_alnum(&query_norm)
    };

    let mut ranked: Vec<RankedItem> = app_state
        .all_items
        .iter()
        .filter_map(|item| {
            let fuzzy = if case_sensitive {
                fuzzy_score(query, &item.label, true)
            } else {
                fuzzy_score_precomputed_lower(
                    &query_norm,
                    &query_compact,
                    &item.label_lc,
                    &item.label_compact,
                )
            };

            if fuzzy <= 0 {
                return None;
            }
            let boost = source_boost(app_state, item.source);
            let total = fuzzy + boost;
            Some(RankedItem {
                item: item.clone(),
                fuzzy_score: fuzzy,
                source_boost: boost,
                total_score: total,
            })
        })
        .collect();

    if ranked.len() > PARTIAL_TOPK_THRESHOLD {
        let keep = PARTIAL_TOPK_LIMIT.min(ranked.len());
        ranked.select_nth_unstable_by(keep - 1, rank_compare_desc);
        ranked.truncate(keep);
    }

    ranked.sort_unstable_by(rank_compare_desc);

    ranked
}

fn update_matching_items_refactored(app_state: &mut AppState) {
    let config_guard = CONFIG.lock().unwrap();
    let case_sensitive = config_guard.as_ref().map_or(false, |c| c.behavior.case_sensitive);
    let max_visible_items = config_guard
        .as_ref()
        .map_or(10usize, |c| c.behavior.max_items.max(1) as usize);
    drop(config_guard);

    if app_state.current_input.is_empty() {
        app_state.matching_items = app_state.all_items.clone();
        ensure_selection_visible(app_state, max_visible_items);
        return;
    }

    let ranked = rank_items(app_state, &app_state.current_input, case_sensitive);
    app_state.matching_items = ranked.into_iter().map(|entry| entry.item).collect();
    ensure_selection_visible(app_state, max_visible_items);
}

fn p95_duration_ms(samples: &mut [u128]) -> u128 {
    if samples.is_empty() {
        return 0;
    }
    samples.sort_unstable();
    let idx = ((samples.len() as f64 * 0.95).ceil() as usize)
        .saturating_sub(1)
        .min(samples.len() - 1);
    samples[idx]
}

fn estimated_dataset_bytes(items: &[LauncherItem]) -> usize {
    items
        .iter()
        .map(|item| item.label.len() + item.target.len())
        .sum()
}

fn print_metrics(app_state: &AppState, case_sensitive: bool, startup_ms: u128) {
    let mut queries: Vec<String> = vec![
        "pow".to_string(),
        "not".to_string(),
        "calc".to_string(),
        "code".to_string(),
        "expl".to_string(),
    ];

    for label in app_state.all_items.iter().take(15).map(|item| item.label.as_str()) {
        let q: String = label.chars().take(3).collect::<String>().to_lowercase();
        if q.len() >= 2 {
            queries.push(q);
        }
    }

    let mut search_samples_ms: Vec<u128> = Vec::new();
    for query in queries {
        let t0 = Instant::now();
        let _ = rank_items(app_state, &query, case_sensitive);
        search_samples_ms.push(t0.elapsed().as_micros());
    }

    let p95_us = p95_duration_ms(&mut search_samples_ms);
    let p95_ms = p95_us as f64 / 1000.0;
    let cache_size = index_cache_size_bytes().unwrap_or(0);

    println!("rmenu metrics");
    println!("- startup_prepare_ms: {}", startup_ms);
    println!("- search_p95_ms: {:.3}", p95_ms);
    println!("- dataset_items: {}", app_state.all_items.len());
    println!("- dataset_estimated_bytes: {}", estimated_dataset_bytes(&app_state.all_items));
    println!("- index_cache_bytes: {}", cache_size);
}

fn main() -> windows::core::Result<()> {
    let startup_t0 = Instant::now();

    let cmd_options: CmdOptions = parse_args();
    let silent_mode = cmd_options.silent; // Conservar silent_mode por si se usa para errores genuinos

    let config_path_from_cli = cmd_options.config_path.as_ref().map(Path::new);

    let mut app_config = match RmenuConfig::load(config_path_from_cli) {
        Ok(cfg) => cfg,
        Err(e) => {
            if !silent_mode { // Este es un error genuino, no un mensaje de debug
                eprintln!("Error loading configuration: {}. Using default config.", e);
            }
            RmenuConfig::default()
        }
    };
    
    app_config.apply_cli_overrides(&cmd_options);

    {
        let mut config_global_guard = CONFIG.lock().unwrap();
        *config_global_guard = Some(app_config.clone());
        drop(config_global_guard);
    }
    
    let launcher_config = app_config.launcher.clone();

    let mut initial_items: Vec<LauncherItem> = Vec::new();
    let mut launcher_mode = false;

    if let Some(elements_str) = &cmd_options.elements_str {
        initial_items = elements_str
            .split(app_config.behavior.element_delimiter)
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(|s| {
                LauncherItem::new(
                    s.to_string(),
                    s.to_string(),
                    LauncherSource::Direct,
                )
            })
            .collect();
    } else if !atty::is(atty::Stream::Stdin) {
        // Solo leer de stdin si no es una TTY (es decir, hay datos redirigidos)
        let mut buffer = String::new();
        match io::stdin().read_to_string(&mut buffer) {
            Ok(bytes_read) => {
                if bytes_read > 0 {
                    initial_items = buffer
                        .lines()
                        .map(str::trim)
                        .filter(|s| !s.is_empty())
                        .map(|s| {
                            LauncherItem::new(
                                s.to_string(),
                                s.to_string(),
                                LauncherSource::Direct,
                            )
                        })
                        .collect();
                }
            }
            Err(e) => {
                if !silent_mode {
                    eprintln!("Error reading from stdin: {}", e);
                }
            }
        }
    } else if launcher_config.launcher_mode_default {
        launcher_mode = true;
        initial_items = load_launcher_items(&launcher_config, silent_mode);
    }

    let final_initial_items = initial_items;
    let initial_app_state = AppState {
        current_input: String::new(),
        selected_index: 0,
        scroll_offset: 0,
        matching_items: final_initial_items.clone(),
        all_items: final_initial_items,
        prompt: cmd_options.prompt.clone(),
        launcher_mode,
        silent_mode,
        history_max_items: launcher_config.history_max_items,
        source_boost_history: launcher_config.source_boost_history,
        source_boost_start_menu: launcher_config.source_boost_start_menu,
        source_boost_path: launcher_config.source_boost_path,
    };

    let case_sensitive = app_config.behavior.case_sensitive;

    if cmd_options.metrics {
        let startup_ms = startup_t0.elapsed().as_millis();
        print_metrics(&initial_app_state, case_sensitive, startup_ms);
        return Ok(());
    }

    if let Some(debug_query) = &cmd_options.debug_ranking {
        let ranked = rank_items(&initial_app_state, debug_query, case_sensitive);

        println!("Debug ranking for query: '{}'", debug_query);
        println!(
            "Dataset size: {} | case_sensitive={} | launcher_mode={}",
            initial_app_state.all_items.len(),
            case_sensitive,
            initial_app_state.launcher_mode
        );

        for (i, entry) in ranked.iter().take(20).enumerate() {
            println!(
                "{:>2}. total={:<5} fuzzy={:<5} boost={:<5} source={:<10} label={} target={}",
                i + 1,
                entry.total_score,
                entry.fuzzy_score,
                entry.source_boost,
                source_name(entry.item.source),
                entry.item.label,
                entry.item.target
            );
        }

        return Ok(());
    }
    
    {
        let mut app_state_global_guard = APP_STATE.lock().unwrap();
        *app_state_global_guard = Some(initial_app_state.clone());
        drop(app_state_global_guard);
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
        
        let config_for_geom_guard = CONFIG.lock().unwrap();
        let final_config_for_geom = config_for_geom_guard.as_ref().expect("CONFIG no inicializado").clone();
        drop(config_for_geom_guard);

        let app_state_for_geom_guard = APP_STATE.lock().unwrap();
        let num_items_for_geom = app_state_for_geom_guard.as_ref().map_or(0, |s| {
            s.matching_items.len().min(final_config_for_geom.behavior.max_items as usize)
        });
        drop(app_state_for_geom_guard);
        
        let geometry = determine_window_geometry(&cmd_options, &final_config_for_geom, num_items_for_geom, silent_mode);
        
        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE(WS_EX_TOPMOST.0 | WS_EX_TOOLWINDOW.0 | WS_EX_APPWINDOW.0),
            PCWSTR(class_name_w.as_ptr()),
            PCWSTR(window_title_w.as_ptr()),
            WS_POPUP | WS_VISIBLE,
            geometry.x, geometry.y, geometry.width, geometry.height,
            None, None, GetModuleHandleW(None)?, None,
        );
        
        if hwnd.0 == 0 {
            if !silent_mode { // Error genuino
                eprintln!("Error creating window");
            }
            return Ok(());
        }
        
        ShowWindow(hwnd, SW_SHOW);
        SetForegroundWindow(hwnd);
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
                        std::process::exit(msg.wParam.0 as i32);
                    }
                    TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }
            }
        }
    }
}

