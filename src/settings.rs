use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use windows::Win32::Foundation::COLORREF;

// Estructura principal de configuración
#[derive(Debug, Clone)]
pub struct RmenuConfig {
    pub colors: ColorConfig,
    pub dimensions: DimensionConfig,
    pub font: FontConfig,
    pub behavior: BehaviorConfig,
}

#[derive(Debug, Clone)]
pub struct ColorConfig {
    pub background: COLORREF,
    pub foreground: COLORREF,
    pub selected_background: COLORREF,
    pub selected_foreground: COLORREF,
    pub border: COLORREF,
}

#[derive(Debug, Clone)]
pub struct DimensionConfig {
    pub default_layout: Option<String>,
    pub width_percent: Option<f32>,
    pub max_width: Option<i32>,
    pub height: i32,
    pub item_height: i32,
    pub x_position: Option<String>,
    pub y_position: Option<String>,
    pub padding: i32,
    pub border_width: i32,
}

#[derive(Debug, Clone)]
pub struct FontConfig {
    pub name: Option<String>,
    pub size: i32,
    pub weight: i32,
}

#[derive(Debug, Clone)]
pub struct BehaviorConfig {
    pub case_sensitive: bool,
    pub instant_selection: bool,
    pub max_items: i32,
    pub element_delimiter: char,
}

// Helper function to create COLORREF from R, G, B components
fn rgb_to_colorref(r: u8, g: u8, b: u8) -> COLORREF {
    COLORREF((b as u32) << 16 | (g as u32) << 8 | (r as u32))
}

impl Default for RmenuConfig {
    fn default() -> Self {
        RmenuConfig {
            colors: ColorConfig { 
                background:          rgb_to_colorref(0x28, 0x2C, 0x34), // #282C34 (Gris oscuro azulado)
                foreground:          rgb_to_colorref(0xAB, 0xB2, 0xBF), // #ABB2BF (Gris claro)
                selected_background: rgb_to_colorref(0x3A, 0x3F, 0x4B), // #3A3F4B (Gris medio)
                selected_foreground: rgb_to_colorref(0xE6, 0xE6, 0xE6), // #E6E6E6 (Casi blanco)
                border:              rgb_to_colorref(0x21, 0x25, 0x2B), // #21252B (Gris muy oscuro)
            },
            dimensions: DimensionConfig {
                default_layout: Some("custom".to_string()),
                width_percent: Some(0.6),
                max_width: Some(1000),
                height: 32, 
                item_height: 28, 
                x_position: Some("r0.5".to_string()),
                y_position: Some("r0.3".to_string()),
                padding: 8,
                border_width: 1,
            },
            font: FontConfig {
                name: Some("Consolas".to_string()),
                size: 15,
                weight: 400,
            },
            behavior: BehaviorConfig {
                case_sensitive: false,
                instant_selection: false,
                max_items: 10, 
                element_delimiter: ',',
            },
        }
    }
}

impl RmenuConfig {
    pub fn load(path: Option<&Path>) -> io::Result<Self> {
        let config_path = match path {
            Some(p) => p.to_path_buf(),
            None => Self::default_config_path()?,
        };
        
        if !config_path.exists() {
            let default_config = RmenuConfig::default();
            // No imprimir nada aquí sobre guardar/crear config.ini, se maneja en main si es necesario (o no)
            // Solo intenta guardarlo silenciosamente.
            let save_path = path.map_or_else(|| Self::default_config_path().unwrap_or_else(|_| PathBuf::from("config.ini")), |p| p.to_path_buf());
            let _ = default_config.save(Some(&save_path)); // Ignorar resultado del guardado aquí
            return Ok(default_config);
        }

        let mut file = fs::File::open(&config_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        Self::parse_config(&contents)
    }

    pub fn save(&self, path: Option<&Path>) -> io::Result<()> {
        let config_path = match path {
            Some(p) => p.to_path_buf(),
            None => Self::default_config_path()?,
        };
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut file = fs::File::create(&config_path)?;
        file.write_all(self.to_string().as_bytes())?;
        Ok(())
    }

    fn default_config_path() -> io::Result<PathBuf> {
        Self::get_home_config_path()
    }

    fn to_string(&self) -> String {
        let mut s = String::new();
        s.push_str("[Colors]\n");

        let bg_r = (self.colors.background.0 & 0x0000FF) as u8;
        let bg_g = ((self.colors.background.0 & 0x00FF00) >> 8) as u8;
        let bg_b = ((self.colors.background.0 & 0xFF0000) >> 16) as u8;
        s.push_str(&format!("background = #{:02X}{:02X}{:02X}\n", bg_r, bg_g, bg_b));

        let fg_r = (self.colors.foreground.0 & 0x0000FF) as u8;
        let fg_g = ((self.colors.foreground.0 & 0x00FF00) >> 8) as u8;
        let fg_b = ((self.colors.foreground.0 & 0xFF0000) >> 16) as u8;
        s.push_str(&format!("foreground = #{:02X}{:02X}{:02X}\n", fg_r, fg_g, fg_b));

        let sel_bg_r = (self.colors.selected_background.0 & 0x0000FF) as u8;
        let sel_bg_g = ((self.colors.selected_background.0 & 0x00FF00) >> 8) as u8;
        let sel_bg_b = ((self.colors.selected_background.0 & 0xFF0000) >> 16) as u8;
        s.push_str(&format!("selected_background = #{:02X}{:02X}{:02X}\n", sel_bg_r, sel_bg_g, sel_bg_b));

        let sel_fg_r = (self.colors.selected_foreground.0 & 0x0000FF) as u8;
        let sel_fg_g = ((self.colors.selected_foreground.0 & 0x00FF00) >> 8) as u8;
        let sel_fg_b = ((self.colors.selected_foreground.0 & 0xFF0000) >> 16) as u8;
        s.push_str(&format!("selected_foreground = #{:02X}{:02X}{:02X}\n", sel_fg_r, sel_fg_g, sel_fg_b));

        let border_r = (self.colors.border.0 & 0x0000FF) as u8;
        let border_g = ((self.colors.border.0 & 0x00FF00) >> 8) as u8;
        let border_b = ((self.colors.border.0 & 0xFF0000) >> 16) as u8;
        s.push_str(&format!("border = #{:02X}{:02X}{:02X}\n\n", border_r, border_g, border_b));

        s.push_str("[Dimensions]\n");
        s.push_str("# Opciones para default_layout: custom, top-fullwidth, bottom-fullwidth, center-dialog, top-left, top-right, bottom-left, bottom-right\n");
        s.push_str(&format!("default_layout = {}\n\n", self.dimensions.default_layout.as_deref().unwrap_or("custom")));
        s.push_str("# Los siguientes valores se usan si default_layout es 'custom' o no está definido,\n");
        s.push_str("# y si no son sobrescritos por argumentos de línea de comandos.\n");
        if let Some(wp) = self.dimensions.width_percent {
            s.push_str(&format!("width_percent = {}\n", wp));
        }
        if let Some(mw) = self.dimensions.max_width {
            s.push_str(&format!("max_width = {}\n", mw));
        }
        s.push_str(&format!("height = {}\n", self.dimensions.height));
        s.push_str(&format!("item_height = {}\n", self.dimensions.item_height));
        if let Some(xp) = &self.dimensions.x_position {
            s.push_str(&format!("x_position = {}\n", xp));
        }
        if let Some(yp) = &self.dimensions.y_position {
            s.push_str(&format!("y_position = {}\n", yp));
        }
        s.push_str(&format!("padding = {}\n", self.dimensions.padding));
        s.push_str(&format!("border_width = {}\n\n", self.dimensions.border_width));

        s.push_str("[Font]\n");
        if let Some(name) = &self.font.name {
            s.push_str(&format!("name = {}\n", name));
        }
        s.push_str(&format!("size = {}\n", self.font.size));
        s.push_str(&format!("weight = {}\n\n", self.font.weight));

        s.push_str("[Behavior]\n");
        s.push_str(&format!("case_sensitive = {}\n", self.behavior.case_sensitive));
        s.push_str(&format!("instant_selection = {}\n", self.behavior.instant_selection));
        s.push_str(&format!("max_items = {}\n", self.behavior.max_items));
        s.push_str(&format!("element_delimiter = {}\n", self.behavior.element_delimiter));
        
        s
    }

    fn parse_config(content: &str) -> io::Result<Self> {
        let mut config = RmenuConfig::default(); // Esta llamada es crucial
        let mut current_section = String::new();
        let mut properties: HashMap<String, HashMap<String, String>> = HashMap::new();

        for line in content.lines() {
            let line = line.trim();
            if line.starts_with('#') || line.is_empty() {
                continue;
            }

            if line.starts_with('[') && line.ends_with(']') {
                current_section = line[1..line.len() - 1].to_string();
                properties.entry(current_section.clone()).or_insert_with(HashMap::new);
            } else if let Some(eq_idx) = line.find('=') {
                let key = line[..eq_idx].trim().to_string();
                let value = line[eq_idx + 1..].trim().to_string();
                if let Some(section_props) = properties.get_mut(&current_section) {
                    section_props.insert(key, value);
                }
            }
        }
        Self::apply_properties(&mut config, properties)?;
        Ok(config)
    }

    fn apply_properties(config: &mut RmenuConfig, properties: HashMap<String, HashMap<String, String>>) -> io::Result<()> {
        if let Some(colors_props) = properties.get("Colors") {
            if let Some(val) = colors_props.get("background") { config.colors.background = parse_color_hex(val)?; }
            if let Some(val) = colors_props.get("foreground") { config.colors.foreground = parse_color_hex(val)?; }
            if let Some(val) = colors_props.get("selected_background") { config.colors.selected_background = parse_color_hex(val)?; }
            if let Some(val) = colors_props.get("selected_foreground") { config.colors.selected_foreground = parse_color_hex(val)?; }
            if let Some(val) = colors_props.get("border") { config.colors.border = parse_color_hex(val)?; }
        }

        if let Some(dim_props) = properties.get("Dimensions") {
            if let Some(val) = dim_props.get("default_layout") { config.dimensions.default_layout = Some(val.clone()); }
            if let Some(val) = dim_props.get("width_percent") { config.dimensions.width_percent = val.parse().ok(); }
            if let Some(val) = dim_props.get("max_width") { config.dimensions.max_width = val.parse().ok(); }
            if let Some(val) = dim_props.get("height") { config.dimensions.height = val.parse().unwrap_or(config.dimensions.height); }
            if let Some(val) = dim_props.get("item_height") { config.dimensions.item_height = val.parse().unwrap_or(config.dimensions.item_height); }
            if let Some(val) = dim_props.get("x_position") { config.dimensions.x_position = Some(val.clone()); }
            if let Some(val) = dim_props.get("y_position") { config.dimensions.y_position = Some(val.clone()); }
            if let Some(val) = dim_props.get("padding") { config.dimensions.padding = val.parse().unwrap_or(config.dimensions.padding); }
            if let Some(val) = dim_props.get("border_width") { config.dimensions.border_width = val.parse().unwrap_or(config.dimensions.border_width); }
        }

        if let Some(font_props) = properties.get("Font") {
            if let Some(val) = font_props.get("name") { config.font.name = Some(val.clone()); }
            if let Some(val) = font_props.get("size") { config.font.size = val.parse().unwrap_or(config.font.size); }
            if let Some(val) = font_props.get("weight") { config.font.weight = val.parse().unwrap_or(config.font.weight); }
        }

        if let Some(behavior_props) = properties.get("Behavior") {
            if let Some(val) = behavior_props.get("case_sensitive") { config.behavior.case_sensitive = val.parse().unwrap_or(config.behavior.case_sensitive); }
            if let Some(val) = behavior_props.get("instant_selection") { config.behavior.instant_selection = val.parse().unwrap_or(config.behavior.instant_selection); }
            if let Some(val) = behavior_props.get("max_items") { config.behavior.max_items = val.parse().unwrap_or(config.behavior.max_items); }
            if let Some(val) = behavior_props.get("element_delimiter") { config.behavior.element_delimiter = val.chars().next().unwrap_or(config.behavior.element_delimiter); }
        }
        Ok(())
    }

    fn get_home_config_path() -> io::Result<PathBuf> {
        dirs::config_dir()
            .map(|mut path| {
                path.push("rmenu");
                path.push("config.ini");
                path
            })
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "No se pudo encontrar el directorio de configuración del usuario"))
    }

    pub fn apply_cli_overrides(&mut self, cmd_opts: &CmdOptions) {
        if let Some(val) = cmd_opts.cli_width_percent { self.dimensions.width_percent = Some(val); }
        if let Some(val) = cmd_opts.cli_max_width { self.dimensions.max_width = Some(val); }
        if let Some(val) = cmd_opts.cli_height { self.dimensions.height = val; }
        if let Some(val) = cmd_opts.cli_item_height { self.dimensions.item_height = val; }
        if let Some(val) = &cmd_opts.cli_x_pos { self.dimensions.x_position = Some(val.clone()); }
        if let Some(val) = &cmd_opts.cli_y_pos { self.dimensions.y_position = Some(val.clone()); }
        if let Some(val) = cmd_opts.cli_padding { self.dimensions.padding = val; }
        if let Some(val) = cmd_opts.cli_border_width { self.dimensions.border_width = val; }
    }
}

#[derive(Debug, Default, Clone)]
pub struct CmdOptions {
    pub elements_str: Option<String>,
    pub prompt: Option<String>,
    pub config_path: Option<String>,
    pub silent: bool,
    pub layout: Option<String>,
    pub cli_width_percent: Option<f32>,
    pub cli_max_width: Option<i32>,
    pub cli_height: Option<i32>,
    pub cli_item_height: Option<i32>,
    pub cli_x_pos: Option<String>,
    pub cli_y_pos: Option<String>,
    pub cli_padding: Option<i32>,
    pub cli_border_width: Option<i32>,
}

fn parse_color_hex(hex: &str) -> io::Result<COLORREF> {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "Color hex debe tener 6 dígitos"));
    }
    let r = u8::from_str_radix(&hex[0..2], 16).map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "Componente R inválido"))?;
    let g = u8::from_str_radix(&hex[2..4], 16).map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "Componente G inválido"))?;
    let b = u8::from_str_radix(&hex[4..6], 16).map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "Componente B inválido"))?;
    Ok(COLORREF((b as u32) << 16 | (g as u32) << 8 | (r as u32)))
}

pub fn parse_args() -> CmdOptions {
    let mut options = CmdOptions::default();
    let args: Vec<String> = std::env::args().collect();
    let mut i = 1;

    while i < args.len() {
        let arg = &args[i];
        match arg.as_str() {
            "-e" | "--elements" => {
                if i + 1 < args.len() { options.elements_str = Some(args[i + 1].clone()); i += 1; }
            }
            "-p" | "--prompt" => {
                if i + 1 < args.len() { options.prompt = Some(args[i + 1].clone()); i += 1; }
            }
            "-c" | "--config" => {
                if i + 1 < args.len() { options.config_path = Some(args[i + 1].clone()); i += 1; }
            }
            "-s" | "--silent" => { options.silent = true; }
            "-h" | "--help" => { print_help(); std::process::exit(0); }
            "--layout" => {
                if i + 1 < args.len() { options.layout = Some(args[i + 1].to_lowercase().clone()); i += 1; }
            }
            "--x-pos" => {
                if i + 1 < args.len() { options.cli_x_pos = Some(args[i + 1].clone()); i += 1; }
            }
            "--y-pos" => {
                if i + 1 < args.len() { options.cli_y_pos = Some(args[i + 1].clone()); i += 1; }
            }
            "--width-percent" => {
                if i + 1 < args.len() { options.cli_width_percent = args[i + 1].parse().ok(); i += 1; }
            }
            "--max-width" => {
                if i + 1 < args.len() { options.cli_max_width = args[i + 1].parse().ok(); i += 1; }
            }
            "--height" => {
                if i + 1 < args.len() { options.cli_height = args[i + 1].parse().ok(); i += 1; }
            }
            "--item-height" => {
                if i + 1 < args.len() { options.cli_item_height = args[i + 1].parse().ok(); i += 1; }
            }
            "--padding" => {
                if i + 1 < args.len() { options.cli_padding = args[i + 1].parse().ok(); i += 1; }
            }
            "--border-width" => {
                if i + 1 < args.len() { options.cli_border_width = args[i + 1].parse().ok(); i += 1; }
            }
            _ => { /* Ignorar argumentos desconocidos */ }
        }
        i += 1;
    }
    options
}

pub fn print_help() {
    println!("rmenu - Un lanzador de menús simple al estilo dmenu para Windows");
    println!("Uso: rmenu [OPCIONES]");
    println!("");
    println!("Opciones de Entrada:");
    println!("  -e, --elements <LIST>   Lista de elementos (delimitador en config.ini, defecto: ',').");
    println!("                            Si no se provee, rmenu lee de stdin (uno por línea).");
    println!("  -p, --prompt <TEXT>     Texto a mostrar como prompt.");
    println!("");
    println!("Opciones de Configuración y Comportamiento:");
    println!("  -c, --config <PATH>     Ruta al archivo de configuración (config.ini).");
    println!("  -s, --silent            Suprime todos los mensajes de error/diagnóstico (stderr).");
    println!("  -h, --help              Muestra esta ayuda.");
    println!("");
    println!("Opciones de Geometría y Layout (sobrescriben config.ini):");
    println!("  --layout <NAME>         Aplica un layout predefinido. Opciones:");
    println!("                            custom, top-fullwidth, bottom-fullwidth, center-dialog,");
    println!("                            top-left, top-right, bottom-left, bottom-right.");
    println!("                            Si es 'custom' o se omite, se usan los valores detallados.");
    println!("  --x-pos <POS>           Posición X. Ej: '100' (píxeles) o 'r0.5' (relativo).");
    println!("  --y-pos <POS>           Posición Y. Ej: '0' o 'r0.3'.");
    println!("  --width-percent <FLOAT> Ancho como porcentaje de pantalla (0.0-1.0).");
    println!("  --max-width <PX>        Ancho máximo en píxeles.");
    println!("  --height <PX>           Altura de la barra de entrada en píxeles.");
    println!("  --item-height <PX>      Altura de cada ítem de la lista en píxeles.");
    println!("  --padding <PX>          Relleno interno en píxeles.");
    println!("  --border-width <PX>     Ancho del borde en píxeles.");
    println!("");
    println!("Para más detalles sobre colores, fuentes y comportamiento, ver el archivo config.ini.");
    println!("Ubicación por defecto: %APPDATA%\\rmenu\\config.ini"); // Barras dobles para literal
} 