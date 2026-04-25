use super::types::{
    ModuleCommandDef, ModuleInputAccessory, ModuleItem, ModuleMode, ModuleProviderDef,
};

#[derive(Debug, Clone)]
pub struct ModuleSnapshot {
    pub query: String,
    pub items: Vec<ModuleItem>,
    pub selected_index: usize,
    pub mode: ModuleMode,
}

#[derive(Debug, Clone)]
pub enum ModuleActionRequest {
    SetQuery(String),
    SetSelection(usize),
    MoveSelection(isize),
    Submit,
    Close,
    AddItems(Vec<ModuleItem>),
    ReplaceItems(Vec<ModuleItem>),
    SetInputAccessory(ModuleInputAccessory),
    ClearInputAccessory,
    RegisterCommand(ModuleCommandDef),
    RegisterProvider(ModuleProviderDef),
}

#[derive(Debug, Clone)]
pub struct ModuleCtx {
    module_name: String,
    snapshot: ModuleSnapshot,
    action_requests: Vec<ModuleActionRequest>,
    logs: Vec<String>,
    toasts: Vec<String>,
}

impl ModuleCtx {
    pub fn new(module_name: impl Into<String>, snapshot: ModuleSnapshot) -> Self {
        Self {
            module_name: module_name.into(),
            snapshot,
            action_requests: Vec::new(),
            logs: Vec::new(),
            toasts: Vec::new(),
        }
    }

    pub fn module_name(&self) -> &str {
        &self.module_name
    }

    pub fn query(&self) -> &str {
        &self.snapshot.query
    }

    pub fn items(&self) -> &[ModuleItem] {
        &self.snapshot.items
    }

    pub fn selected_item(&self) -> Option<&ModuleItem> {
        self.snapshot.items.get(self.snapshot.selected_index)
    }

    pub fn selected_index(&self) -> usize {
        self.snapshot.selected_index
    }

    pub fn mode(&self) -> ModuleMode {
        self.snapshot.mode
    }

    pub fn log(&mut self, message: impl Into<String>) {
        self.logs.push(message.into());
    }

    pub fn toast(&mut self, message: impl Into<String>) {
        self.toasts.push(message.into());
    }

    pub fn set_query(&mut self, text: impl Into<String>) {
        self.action_requests
            .push(ModuleActionRequest::SetQuery(text.into()));
    }

    pub fn set_selection(&mut self, index: usize) {
        self.action_requests
            .push(ModuleActionRequest::SetSelection(index));
    }

    pub fn move_selection(&mut self, offset: isize) {
        self.action_requests
            .push(ModuleActionRequest::MoveSelection(offset));
    }

    pub fn submit(&mut self) {
        self.action_requests.push(ModuleActionRequest::Submit);
    }

    pub fn close(&mut self) {
        self.action_requests.push(ModuleActionRequest::Close);
    }

    pub fn add_items(&mut self, items: Vec<ModuleItem>) {
        self.action_requests
            .push(ModuleActionRequest::AddItems(items));
    }

    pub fn replace_items(&mut self, items: Vec<ModuleItem>) {
        self.action_requests
            .push(ModuleActionRequest::ReplaceItems(items));
    }

    pub fn set_input_accessory(&mut self, accessory: ModuleInputAccessory) {
        self.action_requests
            .push(ModuleActionRequest::SetInputAccessory(accessory));
    }

    pub fn clear_input_accessory(&mut self) {
        self.action_requests
            .push(ModuleActionRequest::ClearInputAccessory);
    }

    pub fn register_command(&mut self, command: ModuleCommandDef) {
        self.action_requests
            .push(ModuleActionRequest::RegisterCommand(command));
    }

    pub fn register_provider(&mut self, provider: ModuleProviderDef) {
        self.action_requests
            .push(ModuleActionRequest::RegisterProvider(provider));
    }

    pub fn take_action_requests(&mut self) -> Vec<ModuleActionRequest> {
        std::mem::take(&mut self.action_requests)
    }

    pub fn take_logs(&mut self) -> Vec<String> {
        std::mem::take(&mut self.logs)
    }

    pub fn take_toasts(&mut self) -> Vec<String> {
        std::mem::take(&mut self.toasts)
    }
}
