pub const MODULE_API_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ModuleMode {
    #[default]
    Launcher,
    Stdin,
    Command,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BadgeKind {
    Shortcut,
    Status,
    Tag,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputAccessoryKind {
    Info,
    Success,
    Warning,
    Error,
    Hint,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleItemCapabilities {
    pub quick_select_key: Option<String>,
}

impl Default for ModuleItemCapabilities {
    fn default() -> Self {
        Self { quick_select_key: None }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleItemDecorations {
    pub badge: Option<String>,
    pub badge_kind: Option<BadgeKind>,
    pub hint: Option<String>,
    pub icon: Option<String>,
}

impl Default for ModuleItemDecorations {
    fn default() -> Self {
        Self {
            badge: None,
            badge_kind: None,
            hint: None,
            icon: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModuleAction {
    LaunchTarget { target: String },
    RunCommand { name: String, args: Vec<String> },
    Noop,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleItem {
    pub id: String,
    pub title: String,
    pub subtitle: Option<String>,
    pub source: Option<String>,
    pub action: ModuleAction,
    pub capabilities: ModuleItemCapabilities,
    pub decorations: ModuleItemDecorations,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleInputAccessory {
    pub text: String,
    pub kind: InputAccessoryKind,
    pub priority: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleKeyEvent {
    pub key: String,
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub meta: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleCommandDef {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleProviderDef {
    pub name: String,
    pub priority: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModuleSourceType {
    Directory,
    Rmod,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleDescriptor {
    pub source_type: ModuleSourceType,
    pub source_path: String,
    pub name: String,
    pub version: String,
    pub api_version: u32,
    pub kind: String,
    pub capabilities: Vec<String>,
    pub enabled: bool,
    pub priority: i32,
    pub description: Option<String>,
    pub author: Option<String>,
    pub homepage: Option<String>,
    pub entry_code: String,
    pub config_json: Option<String>,
    pub readme: Option<String>,
}
