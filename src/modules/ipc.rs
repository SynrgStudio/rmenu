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
    OnLoad,
    OnQueryChange { query: String },
    OnKey { event: IpcKeyEvent },
    ProvideItems { query: String },
    DecorateItems { items: Vec<IpcItem> },
    OnCommand { command: String, args: Vec<String> },
    OnUnload,
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
    ProvideItemsResult { items: Vec<IpcItem> },
    DecorateItemsResult { items: Vec<IpcItem> },
    Error { message: String, recoverable: bool },
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
