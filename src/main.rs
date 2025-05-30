mod settings;

use atty;
use std::{
    ffi::OsStr,
    iter::once,
    os::windows::ffi::OsStrExt,
    sync::Mutex,
    thread::sleep,
    time::Duration,
    path::Path,
    io::{self, Read},
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

use settings::{RmenuConfig, CmdOptions, parse_args};

#[derive(Debug, Default, Clone)]
struct AppState {
    current_input: String,
    selected_index: usize,
    matching_items: Vec<String>,
    all_items: Vec<String>,
    prompt: Option<String>,
}

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

                let mut x_offset = current_padding;
                if let Some(prompt) = &app_state.prompt {
                    let prompt_text = format!("{}: ", prompt);
                    TextOutA(hdc, x_offset, current_padding, prompt_text.as_bytes());
                    x_offset += prompt_text.len() as i32 * (config.font.size / 2);
                }
                
                if !app_state.current_input.is_empty() {
                    TextOutA(hdc, x_offset, current_padding, app_state.current_input.as_bytes());
                }
                
                let current_item_height = config.dimensions.item_height;
                let input_bar_actual_height = config.dimensions.height;

                let max_items_to_display = config.behavior.max_items.min(app_state.matching_items.len() as i32);
                for (i, item) in app_state.matching_items.iter().take(max_items_to_display as usize).enumerate() {
                    let y = input_bar_actual_height + (current_item_height * i as i32);
                    
                    if i == app_state.selected_index {
                        SetBkColor(hdc, config.colors.selected_background);
                        SetTextColor(hdc, config.colors.selected_foreground);
                        let select_rect = RECT { left: 0, top: y, right: rect.right, bottom: y + current_item_height };
                        let select_brush = CreateSolidBrush(config.colors.selected_background);
                        FillRect(hdc, &select_rect, select_brush);
                    } else {
                        SetBkColor(hdc, config.colors.background);
                        SetTextColor(hdc, config.colors.foreground);
                    }
                    TextOutA(hdc, current_padding, y + current_padding / 2, item.as_bytes());
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
                        println!("{}", selected);
                    } else if app_state.all_items.is_empty() && !app_state.current_input.is_empty() {
                        println!("{}", app_state.current_input);
                    }
                    PostQuitMessage(0);
                } else if key_code == VK_DOWN.0 as i32 {
                    if !app_state.matching_items.is_empty() {
                        app_state.selected_index = (app_state.selected_index + 1) % app_state.matching_items.len();
                        InvalidateRect(hwnd, None, true);
                    }
                } else if key_code == VK_UP.0 as i32 {
                    if !app_state.matching_items.is_empty() {
                        app_state.selected_index = (app_state.selected_index + app_state.matching_items.len() - 1) % app_state.matching_items.len();
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
                        app_state.current_input = app_state.matching_items[app_state.selected_index].clone();
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

fn update_matching_items_refactored(app_state: &mut AppState) {
    let config_guard = CONFIG.lock().unwrap();
    let case_sensitive = config_guard.as_ref().map_or(false, |c| c.behavior.case_sensitive);
    drop(config_guard);

    if app_state.current_input.is_empty() {
        app_state.matching_items = app_state.all_items.clone();
    } else {
        app_state.matching_items = app_state.all_items.iter().filter(|item| {
            if case_sensitive {
                item.contains(&app_state.current_input)
            } else {
                item.to_lowercase().contains(&app_state.current_input.to_lowercase())
            }
        }).cloned().collect();
    }
}

fn main() -> windows::core::Result<()> {
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
    
    let mut initial_elements: Vec<String> = Vec::new();
    if let Some(elements_str) = &cmd_options.elements_str {
        initial_elements = elements_str.split(app_config.behavior.element_delimiter).map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
    } else {
        if !atty::is(atty::Stream::Stdin) { // Solo leer de stdin si no es una TTY (es decir, hay datos redirigidos)
            let mut buffer = String::new();
            match io::stdin().read_to_string(&mut buffer) {
                Ok(bytes_read) => {
                    if bytes_read > 0 {
                        initial_elements = buffer.lines().map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
                    }
                }
                Err(e) => {
                    if !silent_mode { // Error genuino
                        eprintln!("Error reading from stdin: {}", e);
                    }
                }
            }
        }
    }

    let final_initial_elements = initial_elements;
    let initial_app_state = AppState {
        current_input: String::new(),
        selected_index: 0,
        matching_items: final_initial_elements.clone(),
        all_items: final_initial_elements,
        prompt: cmd_options.prompt.clone(),
    };
    
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

