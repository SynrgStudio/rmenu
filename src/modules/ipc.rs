use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostRequest {
    pub id: u64,
    pub payload: HostRequestPayload,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum HostRequestPayload {
    Ping,
    Initialize(ModuleInitPayload),
    OnLoad {
        snapshot: Option<IpcSnapshot>,
    },
    OnQueryChange {
        query: String,
        snapshot: IpcSnapshot,
    },
    OnKey {
        event: IpcKeyEvent,
        snapshot: IpcSnapshot,
    },
    ProvideItems {
        query: String,
        snapshot: IpcSnapshot,
    },
    DecorateItems {
        items: Vec<IpcItem>,
        snapshot: IpcSnapshot,
    },
    OnCommand {
        command: String,
        args: Vec<String>,
        snapshot: IpcSnapshot,
    },
    OnUnload {
        snapshot: Option<IpcSnapshot>,
    },
    Shutdown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleInitPayload {
    pub name: String,
    pub version: String,
    pub api_version: u32,
    pub capabilities: Vec<String>,
    pub entry_code: String,
    pub config_json: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostResponse {
    pub id: u64,
    pub payload: HostResponsePayload,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum HostResponsePayload {
    Pong,
    Ack,
    Actions { actions: Vec<IpcAction> },
    ProvideItemsResult { items: Vec<IpcItem> },
    DecorateItemsResult { items: Vec<IpcItem> },
    Error { message: String, recoverable: bool },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum IpcAction {
    SetQuery { text: String },
    SetInputAccessory(IpcInputAccessory),
    ClearInputAccessory,
    ReplaceItems { items: Vec<IpcItem> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcSnapshot {
    pub query: String,
    pub items: Vec<IpcItem>,
    pub selected_index: usize,
    pub mode: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcInputAccessory {
    pub text: String,
    pub kind: Option<String>,
    pub priority: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcKeyEvent {
    pub key: String,
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub meta: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcItem {
    pub id: String,
    pub title: String,
    pub subtitle: Option<String>,
    pub source: Option<String>,
    pub target: Option<String>,
    pub quick_select_key: Option<String>,
    pub badge: Option<String>,
    pub hint: Option<String>,
}
