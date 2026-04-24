use std::collections::BTreeMap;

use super::types::{ModuleCommandDef, ModuleInputAccessory, ModuleProviderDef};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedModuleInfo {
    pub name: String,
    pub version: String,
    pub enabled: bool,
}

#[derive(Debug, Default)]
pub struct ModuleRuntimeState {
    pub loaded_modules: Vec<LoadedModuleInfo>,
    pub registered_commands: BTreeMap<String, ModuleCommandDef>,
    pub registered_providers: BTreeMap<String, ModuleProviderDef>,
    pub active_input_accessory: Option<(String, ModuleInputAccessory)>,
    pub items_replaced_in_cycle: bool,
}

impl ModuleRuntimeState {
    pub fn register_module(&mut self, module_name: String, version: String, enabled: bool) {
        self.loaded_modules.push(LoadedModuleInfo {
            name: module_name,
            version,
            enabled,
        });
    }

    pub fn register_command(&mut self, module_name: &str, mut command: ModuleCommandDef) {
        command.name = command.name.trim().to_ascii_lowercase();
        if command.name.is_empty() {
            return;
        }

        let key = format!("{module_name}::{}", command.name);
        self.registered_commands.insert(key, command);
    }

    pub fn register_provider(&mut self, module_name: &str, provider: ModuleProviderDef) {
        let key = format!("{module_name}::{}", provider.name);
        self.registered_providers.insert(key, provider);
    }
}
