use super::{
    context::ModuleCtx,
    types::{ModuleItem, ModuleKeyEvent},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleProviderOutput {
    pub provider_name: String,
    pub items: Vec<ModuleItem>,
}

pub trait RuntimeModule: Send {
    fn name(&self) -> &str;

    fn on_load(&mut self, _ctx: &mut ModuleCtx) {}

    fn on_unload(&mut self, _ctx: &mut ModuleCtx) {}

    fn on_query_change(&mut self, _query: &str, _ctx: &mut ModuleCtx) {}

    fn on_key(&mut self, _event: &ModuleKeyEvent, _ctx: &mut ModuleCtx) {}

    fn on_command(&mut self, _command: &str, _args: &[String], _ctx: &mut ModuleCtx) {}

    fn provide_items(&mut self, _query: &str, _ctx: &mut ModuleCtx) -> Vec<ModuleItem> {
        Vec::new()
    }

    fn decorate_items(&mut self, items: Vec<ModuleItem>, _ctx: &mut ModuleCtx) -> Vec<ModuleItem> {
        items
    }
}

pub fn dispatch_on_load(modules: &mut [Box<dyn RuntimeModule>], ctx: &mut ModuleCtx) {
    for module in modules {
        module.on_load(ctx);
    }
}

pub fn dispatch_on_unload(modules: &mut [Box<dyn RuntimeModule>], ctx: &mut ModuleCtx) {
    for module in modules {
        module.on_unload(ctx);
    }
}

pub fn dispatch_on_query_change(modules: &mut [Box<dyn RuntimeModule>], query: &str, ctx: &mut ModuleCtx) {
    for module in modules {
        module.on_query_change(query, ctx);
    }
}

pub fn dispatch_decorate_items(
    modules: &mut [Box<dyn RuntimeModule>],
    mut items: Vec<ModuleItem>,
    ctx: &mut ModuleCtx,
) -> Vec<ModuleItem> {
    for module in modules {
        items = module.decorate_items(items, ctx);
    }

    items
}
