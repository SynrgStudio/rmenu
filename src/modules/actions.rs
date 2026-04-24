use super::{
    context::ModuleActionRequest,
    state::ModuleRuntimeState,
    types::{ModuleCommandDef, ModuleInputAccessory, ModuleItem, ModuleProviderDef},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActionError {
    InvalidSelectionIndex { requested: usize, len: usize },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActionEffect {
    QueryUpdated(String),
    SelectionUpdated(usize),
    Submitted,
    Closed,
    ItemsAdded(usize),
    ItemsReplaced(usize),
    InputAccessorySet(ModuleInputAccessory),
    InputAccessoryCleared,
    CommandRegistered(ModuleCommandDef),
    ProviderRegistered(ModuleProviderDef),
}

#[derive(Debug, Default)]
pub struct ActionRuntimeView {
    pub query: String,
    pub items: Vec<ModuleItem>,
    pub selected_index: usize,
}

pub fn apply_action_request(
    module_name: &str,
    request: ModuleActionRequest,
    view: &mut ActionRuntimeView,
    state: &mut ModuleRuntimeState,
) -> Result<ActionEffect, ActionError> {
    match request {
        ModuleActionRequest::SetQuery(text) => {
            view.query = text.clone();
            Ok(ActionEffect::QueryUpdated(text))
        }
        ModuleActionRequest::SetSelection(index) => {
            if view.items.is_empty() {
                view.selected_index = 0;
                return Ok(ActionEffect::SelectionUpdated(0));
            }

            if index >= view.items.len() {
                return Err(ActionError::InvalidSelectionIndex {
                    requested: index,
                    len: view.items.len(),
                });
            }

            view.selected_index = index;
            Ok(ActionEffect::SelectionUpdated(index))
        }
        ModuleActionRequest::MoveSelection(offset) => {
            if view.items.is_empty() {
                view.selected_index = 0;
                return Ok(ActionEffect::SelectionUpdated(0));
            }

            let len = view.items.len() as isize;
            let current = view.selected_index as isize;
            let next = (current + offset).clamp(0, len - 1) as usize;
            view.selected_index = next;
            Ok(ActionEffect::SelectionUpdated(next))
        }
        ModuleActionRequest::Submit => Ok(ActionEffect::Submitted),
        ModuleActionRequest::Close => Ok(ActionEffect::Closed),
        ModuleActionRequest::AddItems(mut items) => {
            let added = items.len();
            view.items.append(&mut items);
            Ok(ActionEffect::ItemsAdded(added))
        }
        ModuleActionRequest::ReplaceItems(items) => {
            let replaced = items.len();
            state.items_replaced_in_cycle = true;
            view.items = items;
            if view.items.is_empty() {
                view.selected_index = 0;
            } else if view.selected_index >= view.items.len() {
                view.selected_index = view.items.len() - 1;
            }
            Ok(ActionEffect::ItemsReplaced(replaced))
        }
        ModuleActionRequest::SetInputAccessory(accessory) => {
            let should_replace = match &state.active_input_accessory {
                Some((_, current)) => accessory.priority >= current.priority,
                None => true,
            };

            if should_replace {
                state.active_input_accessory = Some((module_name.to_string(), accessory.clone()));
                Ok(ActionEffect::InputAccessorySet(accessory))
            } else {
                Ok(ActionEffect::InputAccessoryCleared)
            }
        }
        ModuleActionRequest::ClearInputAccessory => {
            state.active_input_accessory = None;
            Ok(ActionEffect::InputAccessoryCleared)
        }
        ModuleActionRequest::RegisterCommand(command) => {
            state.register_command(module_name, command.clone());
            Ok(ActionEffect::CommandRegistered(command))
        }
        ModuleActionRequest::RegisterProvider(provider) => {
            state.register_provider(module_name, provider.clone());
            Ok(ActionEffect::ProviderRegistered(provider))
        }
    }
}
