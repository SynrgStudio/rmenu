#![allow(dead_code)]

pub mod actions;
pub mod context;
pub mod hooks;
pub mod host_client;
pub mod ipc;
pub mod loader;
pub mod manifest;
pub mod rmod;
pub mod state;
pub mod types;

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use host_client::{ExternalModuleHost, HostClientError};
use ipc::{IpcAction, IpcInputAccessory, IpcItem, IpcKeyEvent, IpcSnapshot};
use loader::discover_module_descriptors;

use crate::app_state::{AppState, LauncherItem, LauncherSource};

use actions::{apply_action_request, ActionRuntimeView};
use context::{ModuleActionRequest, ModuleCtx, ModuleSnapshot};
use hooks::RuntimeModule;
use state::ModuleRuntimeState;
use types::{
    BadgeKind, InputAccessoryKind, ModuleAction, ModuleCommandDef, ModuleDescriptor, ModuleInputAccessory,
    ModuleItem, ModuleItemCapabilities, ModuleItemDecorations, ModuleKeyEvent, ModuleMode, ModuleProviderDef,
    MODULE_API_VERSION,
};

const MAX_RECENT_HOST_ERRORS: usize = 5;
const MAX_CONSECUTIVE_ERRORS_PER_MODULE: u32 = 5;
const MAX_CONSECUTIVE_TIMEOUTS_PER_MODULE: u32 = 3;
const HOT_RELOAD_CHECK_INTERVAL_MS: u64 = 500;
const HOT_RELOAD_DEBOUNCE_MS: u64 = 1200;

const IPC_ITEM_MAX_ID_LEN: usize = 256;
const IPC_ITEM_MAX_TITLE_LEN: usize = 256;
const IPC_ITEM_MAX_SUBTITLE_LEN: usize = 512;
const IPC_ITEM_MAX_SOURCE_LEN: usize = 64;
const IPC_ITEM_MAX_TARGET_LEN: usize = 4096;
const IPC_ITEM_MAX_BADGE_LEN: usize = 32;
const IPC_ITEM_MAX_HINT_LEN: usize = 512;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DedupeSourcePriority {
    CoreFirst,
    ProviderFirst,
}

#[derive(Debug, Clone)]
pub struct ModuleRuntimePolicy {
    pub provider_total_budget_ms: u128,
    pub provider_timeout_ms: u64,
    pub max_items_per_provider_host: usize,
    pub dedupe_source_priority: DedupeSourcePriority,
    pub host_restart_backoff_ms: u64,
    pub max_ipc_payload_bytes: usize,
}

impl Default for ModuleRuntimePolicy {
    fn default() -> Self {
        Self {
            provider_total_budget_ms: 35,
            provider_timeout_ms: 1500,
            max_items_per_provider_host: 24,
            dedupe_source_priority: DedupeSourcePriority::CoreFirst,
            host_restart_backoff_ms: 800,
            max_ipc_payload_bytes: 256 * 1024,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExternalModuleStatus {
    Loaded,
    Degraded,
    Disabled,
    Unloaded,
}

#[derive(Debug, Clone)]
struct HostHealth {
    status: ExternalModuleStatus,
    consecutive_errors: u32,
    consecutive_timeouts: u32,
}

impl Default for HostHealth {
    fn default() -> Self {
        Self {
            status: ExternalModuleStatus::Unloaded,
            consecutive_errors: 0,
            consecutive_timeouts: 0,
        }
    }
}

#[derive(Debug, Default, Clone)]
struct HostTelemetry {
    request_count: u64,
    error_count: u64,
    timeout_count: u64,
    restart_count: u64,
    total_latency_ms: u128,
    max_latency_ms: u128,
    recent_errors: VecDeque<String>,
}

#[derive(Debug, Clone)]
struct ResolvedCommandRoute {
    target_module: Option<String>,
    command: String,
}

impl ResolvedCommandRoute {
    fn matches_module(&self, module_name: &str) -> bool {
        self.target_module
            .as_deref()
            .map(|target| target.eq_ignore_ascii_case(module_name))
            .unwrap_or(true)
    }
}

pub struct ModuleRuntime {
    modules: Vec<Box<dyn RuntimeModule>>,
    state: ModuleRuntimeState,
    external_descriptors: Vec<ModuleDescriptor>,
    external_hosts: Vec<ExternalModuleHost>,
    host_telemetry: BTreeMap<String, HostTelemetry>,
    host_health: BTreeMap<String, HostHealth>,
    host_capabilities: BTreeMap<String, BTreeSet<String>>,
    last_restart_attempt: BTreeMap<String, Instant>,
    external_signatures: BTreeMap<String, u64>,
    last_hot_reload_check: Option<Instant>,
    last_hot_reload_at: Option<Instant>,
    policy: ModuleRuntimePolicy,
    modules_dir: PathBuf,
}

impl ModuleRuntime {
    pub fn new() -> Self {
        Self {
            modules: Vec::new(),
            state: ModuleRuntimeState::default(),
            external_descriptors: Vec::new(),
            external_hosts: Vec::new(),
            host_telemetry: BTreeMap::new(),
            host_health: BTreeMap::new(),
            host_capabilities: BTreeMap::new(),
            last_restart_attempt: BTreeMap::new(),
            external_signatures: BTreeMap::new(),
            last_hot_reload_check: None,
            last_hot_reload_at: None,
            policy: ModuleRuntimePolicy::default(),
            modules_dir: PathBuf::from("modules"),
        }
    }

    pub fn api_version(&self) -> u32 {
        MODULE_API_VERSION
    }

    pub fn register_builtin_module(&mut self, module: Box<dyn RuntimeModule>) {
        self.modules.push(module);
        self.sync_loaded_modules_state();
    }

    pub fn module_count(&self) -> usize {
        self.modules.len()
    }

    pub fn external_module_count(&self) -> usize {
        self.external_descriptors.len()
    }

    pub fn configure_modules_dir(&mut self, modules_dir: impl Into<PathBuf>) {
        self.modules_dir = modules_dir.into();
    }

    pub fn configure_policy(&mut self, policy: ModuleRuntimePolicy) {
        self.policy = policy;
    }

    pub fn load_external_descriptors(&mut self, modules_dir: &Path, silent_mode: bool) {
        self.modules_dir = modules_dir.to_path_buf();
        self.reload_external_descriptors(silent_mode);
    }

    pub fn reload_external_descriptors(&mut self, silent_mode: bool) {
        match discover_module_descriptors(&self.modules_dir) {
            Ok(descriptors) => {
                self.apply_external_descriptors(descriptors, silent_mode, true);
            }
            Err(err) => {
                if !silent_mode {
                    eprintln!("modules loader error: {err:?}");
                }
            }
        }
    }

    pub fn poll_hot_reload(&mut self, silent_mode: bool) -> bool {
        let now = Instant::now();

        if let Some(last_check) = self.last_hot_reload_check {
            if now.duration_since(last_check) < Duration::from_millis(HOT_RELOAD_CHECK_INTERVAL_MS) {
                return false;
            }
        }
        self.last_hot_reload_check = Some(now);

        match discover_module_descriptors(&self.modules_dir) {
            Ok(descriptors) => {
                let signatures = build_descriptor_signatures(&descriptors);
                if signatures == self.external_signatures {
                    return false;
                }

                if let Some(last_reload) = self.last_hot_reload_at {
                    if now.duration_since(last_reload) < Duration::from_millis(HOT_RELOAD_DEBOUNCE_MS) {
                        return false;
                    }
                }

                self.last_hot_reload_at = Some(now);
                self.apply_external_descriptors(descriptors, silent_mode, false);
                true
            }
            Err(err) => {
                if !silent_mode {
                    eprintln!("modules hot-reload poll error: {err:?}");
                }
                false
            }
        }
    }

    fn apply_external_descriptors(
        &mut self,
        descriptors: Vec<ModuleDescriptor>,
        silent_mode: bool,
        full_reload: bool,
    ) {
        if full_reload {
            self.shutdown_external_hosts();
            self.host_capabilities.clear();
            self.host_health.clear();
            self.external_descriptors = descriptors;

            for descriptor in &self.external_descriptors {
                self.host_capabilities.insert(
                    descriptor.name.clone(),
                    descriptor.capabilities.iter().map(|cap| cap.to_ascii_lowercase()).collect(),
                );
                self.host_health.insert(
                    descriptor.name.clone(),
                    HostHealth {
                        status: if descriptor.enabled {
                            ExternalModuleStatus::Unloaded
                        } else {
                            ExternalModuleStatus::Disabled
                        },
                        ..HostHealth::default()
                    },
                );
            }

            self.external_signatures = build_descriptor_signatures(&self.external_descriptors);
            self.sync_loaded_modules_state();
            self.start_external_hosts(silent_mode);
            return;
        }

        let old_map: BTreeMap<String, ModuleDescriptor> = self
            .external_descriptors
            .iter()
            .cloned()
            .map(|d| (d.name.clone(), d))
            .collect();
        let new_map: BTreeMap<String, ModuleDescriptor> = descriptors
            .iter()
            .cloned()
            .map(|d| (d.name.clone(), d))
            .collect();

        let removed = old_map
            .keys()
            .filter(|name| !new_map.contains_key(*name))
            .cloned()
            .collect::<Vec<_>>();

        for module_name in &removed {
            if let Some(index) = self.external_hosts.iter().position(|h| h.module_name == *module_name) {
                let mut host = self.external_hosts.remove(index);
                host.shutdown();
            }
            self.host_capabilities.remove(module_name);
            self.host_health.remove(module_name);
            self.host_telemetry.remove(module_name);
            self.external_signatures.remove(module_name);
        }

        let mut changed_or_new = Vec::new();
        for (name, new_desc) in &new_map {
            let changed = match old_map.get(name) {
                Some(old_desc) => old_desc != new_desc,
                None => true,
            };
            if changed {
                changed_or_new.push(name.clone());
            }
        }

        self.external_descriptors = descriptors;

        for name in changed_or_new {
            if let Some(index) = self.external_hosts.iter().position(|h| h.module_name == name) {
                let mut host = self.external_hosts.remove(index);
                host.shutdown();
            }

            let Some(descriptor) = self.external_descriptors.iter().find(|d| d.name == name).cloned() else {
                continue;
            };

            self.host_capabilities.insert(
                descriptor.name.clone(),
                descriptor.capabilities.iter().map(|cap| cap.to_ascii_lowercase()).collect(),
            );

            if !descriptor.enabled {
                self.host_health.insert(
                    descriptor.name.clone(),
                    HostHealth {
                        status: ExternalModuleStatus::Disabled,
                        ..HostHealth::default()
                    },
                );
                continue;
            }

            match ExternalModuleHost::start(
                &descriptor,
                self.policy.provider_timeout_ms,
                self.policy.max_ipc_payload_bytes,
            ) {
                Ok(host) => {
                    self.external_hosts.push(host);
                    self.host_health.insert(
                        descriptor.name.clone(),
                        HostHealth {
                            status: ExternalModuleStatus::Loaded,
                            ..HostHealth::default()
                        },
                    );
                }
                Err(err) => {
                    if !silent_mode {
                        eprintln!("module host reload error for '{}': {err:?}", descriptor.name);
                    }
                    self.record_host_error(
                        &descriptor.name,
                        0,
                        false,
                        format!("host_reload_error: {err:?}"),
                    );
                }
            }
        }

        self.external_signatures = build_descriptor_signatures(&self.external_descriptors);
        self.sync_loaded_modules_state();
    }

    pub fn runtime_command(&mut self, command: &str, silent_mode: bool) -> bool {
        match command {
            "modules.reload" => {
                self.reload_external_descriptors(silent_mode);
                self.state.active_input_accessory = Some((
                    "runtime".to_string(),
                    ModuleInputAccessory {
                        text: format!("modules reloaded: {} external", self.external_descriptors.len()),
                        kind: InputAccessoryKind::Info,
                        priority: 100,
                    },
                ));
                true
            }
            "modules.list" => {
                let mut names = self
                    .state
                    .loaded_modules
                    .iter()
                    .map(|module| module.name.as_str())
                    .collect::<Vec<_>>();
                names.sort_unstable();
                let preview = if names.is_empty() {
                    "none".to_string()
                } else {
                    names.join(", ")
                };

                self.state.active_input_accessory = Some((
                    "runtime".to_string(),
                    ModuleInputAccessory {
                        text: format!("modules: {preview}"),
                        kind: InputAccessoryKind::Hint,
                        priority: 100,
                    },
                ));
                true
            }
            "modules.telemetry.reset" => {
                self.host_telemetry.clear();
                self.state.active_input_accessory = Some((
                    "runtime".to_string(),
                    ModuleInputAccessory {
                        text: "modules telemetry reset".to_string(),
                        kind: InputAccessoryKind::Info,
                        priority: 100,
                    },
                ));
                true
            }
            _ => false,
        }
    }

    pub fn run_on_load(&mut self, app_state: &mut AppState) {
        for module in &mut self.modules {
            let module_name = module.name().to_string();
            let snapshot = snapshot_from_app_state(app_state);
            let mut ctx = ModuleCtx::new(module_name, snapshot);
            module.on_load(&mut ctx);
            Self::apply_ctx_requests(module.name(), &mut ctx, app_state, &mut self.state, None);
        }
    }

    pub fn run_on_unload(&mut self, app_state: &mut AppState) {
        for module in &mut self.modules {
            let module_name = module.name().to_string();
            let snapshot = snapshot_from_app_state(app_state);
            let mut ctx = ModuleCtx::new(module_name, snapshot);
            module.on_unload(&mut ctx);
            Self::apply_ctx_requests(module.name(), &mut ctx, app_state, &mut self.state, None);
        }

        self.shutdown_external_hosts();
    }

    pub fn run_on_query_change(&mut self, app_state: &mut AppState) {
        self.state.items_replaced_in_cycle = false;
        let query = app_state.current_input.clone();

        for module in &mut self.modules {
            let module_name = module.name().to_string();
            let snapshot = snapshot_from_app_state(app_state);
            let mut ctx = ModuleCtx::new(module_name, snapshot);
            module.on_query_change(&query, &mut ctx);
            Self::apply_ctx_requests(module.name(), &mut ctx, app_state, &mut self.state, None);
        }

        let mut failed_hosts: Vec<String> = Vec::new();
        let mut telemetry_events: Vec<(String, u128, bool, bool, Option<String>)> = Vec::new();
        for host in &mut self.external_hosts {
            let started = Instant::now();
            let snapshot = ipc_snapshot_from_app_state(app_state, false);
            match host.on_query_change(&query, snapshot) {
                Ok(actions) => {
                    Self::apply_ipc_actions(
                        &host.module_name,
                        actions,
                        app_state,
                        &mut self.state,
                        self.host_capabilities.get(&host.module_name),
                    );
                    telemetry_events.push((host.module_name.clone(), started.elapsed().as_millis(), false, false, None));
                }
                Err(err) => {
                    let is_timeout = matches!(err, HostClientError::Timeout(_));
                    telemetry_events.push((
                        host.module_name.clone(),
                        started.elapsed().as_millis(),
                        true,
                        is_timeout,
                        Some(host_error_message(&err)),
                    ));
                    failed_hosts.push(host.module_name.clone());
                }
            }
        }

        for (name, latency_ms, is_error, is_timeout, error_message) in telemetry_events {
            if is_error {
                self.record_host_error(
                    &name,
                    latency_ms,
                    is_timeout,
                    error_message.unwrap_or_else(|| "unknown host error".to_string()),
                );
            } else {
                self.record_host_success(&name, latency_ms);
            }
        }

        for module_name in failed_hosts {
            self.restart_external_host(&module_name, app_state.silent_mode);
        }
    }

    pub fn run_on_key(&mut self, app_state: &mut AppState, event: &ModuleKeyEvent) {
        if !should_dispatch_module_key_event(event) {
            return;
        }

        for module in &mut self.modules {
            let module_name = module.name().to_string();
            let snapshot = snapshot_from_app_state(app_state);
            let mut ctx = ModuleCtx::new(module_name, snapshot);
            module.on_key(event, &mut ctx);
            Self::apply_ctx_requests(module.name(), &mut ctx, app_state, &mut self.state, None);
        }

        let mut failed_hosts: Vec<String> = Vec::new();
        let mut telemetry_events: Vec<(String, u128, bool, bool, Option<String>)> = Vec::new();
        let host_capabilities = self.host_capabilities.clone();

        for host in &mut self.external_hosts {
            let has_capability = host_capabilities
                .get(&host.module_name)
                .map(|caps| caps.contains("keys"))
                .unwrap_or(false);
            if !has_capability {
                continue;
            }

            let started = Instant::now();
            let snapshot = ipc_snapshot_from_app_state(app_state, true);
            match host.on_key(module_key_event_to_ipc_key_event(event), snapshot) {
                Ok(actions) => {
                    Self::apply_ipc_actions(
                        &host.module_name,
                        actions,
                        app_state,
                        &mut self.state,
                        self.host_capabilities.get(&host.module_name),
                    );
                    telemetry_events.push((host.module_name.clone(), started.elapsed().as_millis(), false, false, None));
                }
                Err(err) => {
                    let is_timeout = matches!(err, HostClientError::Timeout(_));
                    telemetry_events.push((
                        host.module_name.clone(),
                        started.elapsed().as_millis(),
                        true,
                        is_timeout,
                        Some(host_error_message(&err)),
                    ));
                    failed_hosts.push(host.module_name.clone());
                }
            }
        }

        for (name, latency_ms, is_error, is_timeout, error_message) in telemetry_events {
            if is_error {
                self.record_host_error(
                    &name,
                    latency_ms,
                    is_timeout,
                    error_message.unwrap_or_else(|| "unknown host error".to_string()),
                );
            } else {
                self.record_host_success(&name, latency_ms);
            }
        }

        for module_name in failed_hosts {
            self.restart_external_host(&module_name, app_state.silent_mode);
        }
    }

    pub fn collect_provider_items(&mut self, app_state: &AppState) -> Vec<LauncherItem> {
        let mut provided: Vec<ModuleItem> = Vec::new();
        let query = app_state.current_input.clone();
        let mut runtime_view_state = app_state.clone();

        for module in &mut self.modules {
            let module_name = module.name().to_string();
            let snapshot = snapshot_from_app_state(&runtime_view_state);
            let mut ctx = ModuleCtx::new(module_name, snapshot);
            let mut items = module.provide_items(&query, &mut ctx);
            provided.append(&mut items);
            Self::apply_ctx_requests(
                module.name(),
                &mut ctx,
                &mut runtime_view_state,
                &mut self.state,
                None,
            );
        }

        let mut failed_hosts: Vec<String> = Vec::new();
        let mut telemetry_events: Vec<(String, u128, bool, bool, Option<String>)> = Vec::new();
        let host_capabilities = self.host_capabilities.clone();
        let providers_started = Instant::now();

        for host in &mut self.external_hosts {
            if providers_started.elapsed().as_millis() > self.policy.provider_total_budget_ms {
                let message = format!(
                    "provider_budget_exceeded total_budget_ms={} stopping remaining hosts",
                    self.policy.provider_total_budget_ms
                );
                telemetry_events.push((host.module_name.clone(), 0, true, false, Some(message)));
                break;
            }

            let has_capability = host_capabilities
                .get(&host.module_name)
                .map(|caps| caps.contains("providers"))
                .unwrap_or(false);
            if !has_capability {
                continue;
            }

            let started = Instant::now();
            let snapshot = ipc_snapshot_from_app_state(app_state, false);
            match host.provide_items(&query, snapshot) {
                Ok(mut items) => {
                    if items.len() > self.policy.max_items_per_provider_host {
                        items.truncate(self.policy.max_items_per_provider_host);
                    }
                    let sanitized = sanitize_ipc_items(items, &host.module_name, app_state.silent_mode);
                    provided.extend(sanitized.into_iter().map(module_item_from_ipc_item));
                    telemetry_events.push((host.module_name.clone(), started.elapsed().as_millis(), false, false, None));
                }
                Err(err) => {
                    let is_timeout = matches!(err, HostClientError::Timeout(_));
                    telemetry_events.push((
                        host.module_name.clone(),
                        started.elapsed().as_millis(),
                        true,
                        is_timeout,
                        Some(host_error_message(&err)),
                    ));
                    failed_hosts.push(host.module_name.clone());
                }
            }
        }

        for (name, latency_ms, is_error, is_timeout, error_message) in telemetry_events {
            if is_error {
                self.record_host_error(
                    &name,
                    latency_ms,
                    is_timeout,
                    error_message.unwrap_or_else(|| "unknown host error".to_string()),
                );
            } else {
                self.record_host_success(&name, latency_ms);
            }
        }

        for module_name in failed_hosts {
            self.restart_external_host(&module_name, app_state.silent_mode);
        }

        dedupe_module_items(provided)
            .into_iter()
            .map(launcher_item_from_module_item)
            .collect()
    }

    pub fn merge_rank_dataset(
        &self,
        core_items: Vec<LauncherItem>,
        provider_items: Vec<LauncherItem>,
    ) -> Vec<LauncherItem> {
        match self.policy.dedupe_source_priority {
            DedupeSourcePriority::CoreFirst => {
                dedupe_launcher_items_by_priority(vec![core_items, provider_items])
            }
            DedupeSourcePriority::ProviderFirst => {
                dedupe_launcher_items_by_priority(vec![provider_items, core_items])
            }
        }
    }

    pub fn dispatch_command(&mut self, app_state: &mut AppState, command: &str, args: &[String], silent_mode: bool) {
        if self.runtime_command(command, silent_mode) {
            return;
        }

        let Some(route) = self.resolve_command_route(command, silent_mode) else {
            return;
        };

        for module in &mut self.modules {
            if !route.matches_module(module.name()) {
                continue;
            }

            let module_name = module.name().to_string();
            let snapshot = snapshot_from_app_state(app_state);
            let mut ctx = ModuleCtx::new(module_name, snapshot);
            module.on_command(&route.command, args, &mut ctx);
            Self::apply_ctx_requests(module.name(), &mut ctx, app_state, &mut self.state, None);
        }

        let mut failed_hosts: Vec<String> = Vec::new();
        let mut telemetry_events: Vec<(String, u128, bool, bool, Option<String>)> = Vec::new();
        let host_capabilities = self.host_capabilities.clone();
        for host in &mut self.external_hosts {
            if !route.matches_module(&host.module_name) {
                continue;
            }

            let has_capability = host_capabilities
                .get(&host.module_name)
                .map(|caps| caps.contains("commands"))
                .unwrap_or(false);
            if !has_capability {
                continue;
            }

            let started = Instant::now();
            let snapshot = ipc_snapshot_from_app_state(app_state, true);
            match host.on_command(&route.command, args, snapshot) {
                Ok(actions) => {
                    Self::apply_ipc_actions(
                        &host.module_name,
                        actions,
                        app_state,
                        &mut self.state,
                        self.host_capabilities.get(&host.module_name),
                    );
                    telemetry_events.push((host.module_name.clone(), started.elapsed().as_millis(), false, false, None));
                }
                Err(err) => {
                    if !silent_mode {
                        eprintln!("module host command error '{}': {err:?}", host.module_name);
                    }
                    let is_timeout = matches!(err, HostClientError::Timeout(_));
                    telemetry_events.push((
                        host.module_name.clone(),
                        started.elapsed().as_millis(),
                        true,
                        is_timeout,
                        Some(host_error_message(&err)),
                    ));
                    failed_hosts.push(host.module_name.clone());
                }
            }
        }

        for (name, latency_ms, is_error, is_timeout, error_message) in telemetry_events {
            if is_error {
                self.record_host_error(
                    &name,
                    latency_ms,
                    is_timeout,
                    error_message.unwrap_or_else(|| "unknown host error".to_string()),
                );
            } else {
                self.record_host_success(&name, latency_ms);
            }
        }

        for module_name in failed_hosts {
            self.restart_external_host(&module_name, silent_mode);
        }
    }

    pub fn decorate_items(&mut self, app_state: &AppState, items: Vec<LauncherItem>) -> Vec<LauncherItem> {
        let mut module_items = items.into_iter().map(module_item_from_launcher_item).collect::<Vec<_>>();

        for module in &mut self.modules {
            let module_name = module.name().to_string();
            let snapshot = snapshot_from_app_state(app_state);
            let mut ctx = ModuleCtx::new(module_name, snapshot);
            module_items = module.decorate_items(module_items, &mut ctx);
            let mut shadow_state = app_state.clone();
            Self::apply_ctx_requests(module.name(), &mut ctx, &mut shadow_state, &mut self.state, None);
        }

        let mut ipc_items = module_items
            .into_iter()
            .map(module_item_to_ipc_item)
            .collect::<Vec<_>>();

        let mut failed_hosts: Vec<String> = Vec::new();
        let mut telemetry_events: Vec<(String, u128, bool, bool, Option<String>)> = Vec::new();
        let host_capabilities = self.host_capabilities.clone();
        for host in &mut self.external_hosts {
            let has_capability = host_capabilities
                .get(&host.module_name)
                .map(|caps| caps.contains("decorate-items"))
                .unwrap_or(false);
            if !has_capability {
                continue;
            }

            let started = Instant::now();
            let snapshot = ipc_snapshot_from_app_state(app_state, false);
            match host.decorate_items(ipc_items.clone(), snapshot) {
                Ok(next_items) => {
                    ipc_items = sanitize_ipc_items(next_items, &host.module_name, app_state.silent_mode);
                    telemetry_events.push((host.module_name.clone(), started.elapsed().as_millis(), false, false, None));
                }
                Err(err) => {
                    let is_timeout = matches!(err, HostClientError::Timeout(_));
                    telemetry_events.push((
                        host.module_name.clone(),
                        started.elapsed().as_millis(),
                        true,
                        is_timeout,
                        Some(host_error_message(&err)),
                    ));
                    failed_hosts.push(host.module_name.clone());
                }
            }
        }

        for (name, latency_ms, is_error, is_timeout, error_message) in telemetry_events {
            if is_error {
                self.record_host_error(
                    &name,
                    latency_ms,
                    is_timeout,
                    error_message.unwrap_or_else(|| "unknown host error".to_string()),
                );
            } else {
                self.record_host_success(&name, latency_ms);
            }
        }

        for module_name in failed_hosts {
            self.restart_external_host(&module_name, app_state.silent_mode);
        }

        ipc_items
            .into_iter()
            .map(module_item_from_ipc_item)
            .map(launcher_item_from_module_item)
            .collect()
    }

    pub fn active_input_accessory(&self) -> Option<ModuleInputAccessory> {
        self.state
            .active_input_accessory
            .as_ref()
            .map(|(_, accessory)| accessory.clone())
    }

    pub fn items_replaced_in_cycle(&self) -> bool {
        self.state.items_replaced_in_cycle
    }

    pub fn modules_debug_report(&self) -> String {
        let mut out = String::new();
        out.push_str("rmenu modules debug\n");
        out.push_str(&format!("- api_version: {}\n", self.api_version()));
        out.push_str(&format!("- builtin_modules: {}\n", self.modules.len()));
        out.push_str(&format!("- external_descriptors: {}\n", self.external_descriptors.len()));
        out.push_str(&format!("- running_hosts: {}\n", self.external_hosts.len()));

        if self.state.loaded_modules.is_empty() {
            out.push_str("- loaded_modules: none\n");
        } else {
            out.push_str("- loaded_modules:\n");
            for module in &self.state.loaded_modules {
                out.push_str(&format!(
                    "  - {}@{} enabled={}\n",
                    module.name, module.version, module.enabled
                ));
            }
        }

        if self.host_telemetry.is_empty() {
            out.push_str("- host_telemetry: none\n");
        } else {
            out.push_str("- host_telemetry:\n");
            for (name, telemetry) in &self.host_telemetry {
                let avg_latency = if telemetry.request_count == 0 {
                    0.0
                } else {
                    telemetry.total_latency_ms as f64 / telemetry.request_count as f64
                };
                let status = self
                    .host_health
                    .get(name)
                    .map(|health| match health.status {
                        ExternalModuleStatus::Loaded => "loaded",
                        ExternalModuleStatus::Degraded => "degraded",
                        ExternalModuleStatus::Disabled => "disabled",
                        ExternalModuleStatus::Unloaded => "unloaded",
                    })
                    .unwrap_or("unknown");
                out.push_str(&format!(
                    "  - {} status={} req={} err={} timeout={} restart={} avg_ms={:.2} max_ms={}\n",
                    name,
                    status,
                    telemetry.request_count,
                    telemetry.error_count,
                    telemetry.timeout_count,
                    telemetry.restart_count,
                    avg_latency,
                    telemetry.max_latency_ms
                ));

                if !telemetry.recent_errors.is_empty() {
                    out.push_str("    recent_errors:\n");
                    for err in &telemetry.recent_errors {
                        out.push_str(&format!("      - {}\n", err));
                    }
                }
            }
        }

        out
    }

    fn resolve_command_route(&self, command: &str, silent_mode: bool) -> Option<ResolvedCommandRoute> {
        let normalized = normalize_command_name(command);
        if normalized.is_empty() {
            if !silent_mode {
                eprintln!("modules command ignored: empty command");
            }
            return None;
        }

        if let Some((module_name, module_command)) = parse_namespaced_command(&normalized) {
            if let Some(module_name) = self.find_module_name(&module_name) {
                return Some(ResolvedCommandRoute {
                    target_module: Some(module_name),
                    command: module_command,
                });
            }

            if !silent_mode {
                eprintln!(
                    "modules command ignored: unknown module namespace '{}' for command '{}'",
                    module_name, command
                );
            }
            return None;
        }

        let alias_owners = self.command_alias_owners();
        if let Some(owners) = alias_owners.get(&normalized) {
            if owners.len() == 1 {
                return Some(ResolvedCommandRoute {
                    target_module: Some(owners[0].clone()),
                    command: normalized,
                });
            }

            if !silent_mode {
                eprintln!(
                    "modules command alias collision command='{}' owners='{}' (use /<module>::{})",
                    normalized,
                    owners.join(", "),
                    normalized
                );
            }
            return None;
        }

        Some(ResolvedCommandRoute {
            target_module: None,
            command: normalized,
        })
    }

    fn command_alias_owners(&self) -> BTreeMap<String, Vec<String>> {
        let mut owners: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();

        for (qualified, command) in &self.state.registered_commands {
            let Some((module_name, _)) = split_registered_command_key(qualified) else {
                continue;
            };

            let alias = normalize_command_name(&command.name);
            if alias.is_empty() {
                continue;
            }

            owners.entry(alias).or_default().insert(module_name.to_string());
        }

        owners
            .into_iter()
            .map(|(alias, module_names)| (alias, module_names.into_iter().collect()))
            .collect()
    }

    fn find_module_name(&self, module_name: &str) -> Option<String> {
        self
            .modules
            .iter()
            .find(|module| module.name().eq_ignore_ascii_case(module_name))
            .map(|module| module.name().to_string())
            .or_else(|| {
                self.external_descriptors
                    .iter()
                    .find(|descriptor| descriptor.name.eq_ignore_ascii_case(module_name))
                    .map(|descriptor| descriptor.name.clone())
            })
    }

    fn telemetry_entry_mut(&mut self, module_name: &str) -> &mut HostTelemetry {
        self.host_telemetry
            .entry(module_name.to_string())
            .or_default()
    }

    fn host_has_capability(&self, module_name: &str, capability: &str) -> bool {
        self.host_capabilities
            .get(module_name)
            .map(|caps| caps.contains(&capability.to_ascii_lowercase()))
            .unwrap_or(false)
    }

    fn record_host_success(&mut self, module_name: &str, latency_ms: u128) {
        let entry = self.telemetry_entry_mut(module_name);
        entry.request_count = entry.request_count.saturating_add(1);
        entry.total_latency_ms = entry.total_latency_ms.saturating_add(latency_ms);
        entry.max_latency_ms = entry.max_latency_ms.max(latency_ms);

        let health = self.host_health.entry(module_name.to_string()).or_default();
        health.consecutive_errors = 0;
        health.consecutive_timeouts = 0;
        health.status = ExternalModuleStatus::Loaded;
    }

    fn record_host_error(&mut self, module_name: &str, latency_ms: u128, is_timeout: bool, error_message: String) {
        let entry = self.telemetry_entry_mut(module_name);
        entry.request_count = entry.request_count.saturating_add(1);
        entry.error_count = entry.error_count.saturating_add(1);
        if is_timeout {
            entry.timeout_count = entry.timeout_count.saturating_add(1);
        }
        entry.total_latency_ms = entry.total_latency_ms.saturating_add(latency_ms);
        entry.max_latency_ms = entry.max_latency_ms.max(latency_ms);

        entry.recent_errors.push_back(error_message);
        while entry.recent_errors.len() > MAX_RECENT_HOST_ERRORS {
            let _ = entry.recent_errors.pop_front();
        }

        let health = self.host_health.entry(module_name.to_string()).or_default();
        health.consecutive_errors = health.consecutive_errors.saturating_add(1);
        if is_timeout {
            health.consecutive_timeouts = health.consecutive_timeouts.saturating_add(1);
        }

        let should_disable = health.consecutive_errors >= MAX_CONSECUTIVE_ERRORS_PER_MODULE
            || health.consecutive_timeouts >= MAX_CONSECUTIVE_TIMEOUTS_PER_MODULE;
        health.status = if should_disable {
            ExternalModuleStatus::Disabled
        } else {
            ExternalModuleStatus::Degraded
        };
    }

    fn host_is_disabled(&self, module_name: &str) -> bool {
        self.host_health
            .get(module_name)
            .map(|health| health.status == ExternalModuleStatus::Disabled)
            .unwrap_or(false)
    }

    fn sync_loaded_modules_state(&mut self) {
        self.state.loaded_modules.clear();

        for module in &self.modules {
            self.state
                .register_module(module.name().to_string(), "builtin".to_string(), true);
        }

        for descriptor in &self.external_descriptors {
            self.state.register_module(
                descriptor.name.clone(),
                descriptor.version.clone(),
                descriptor.enabled,
            );
        }
    }

    fn start_external_hosts(&mut self, silent_mode: bool) {
        self.external_hosts.clear();

        let descriptors = self.external_descriptors.clone();
        for descriptor in descriptors {
            if !descriptor.enabled || self.host_is_disabled(&descriptor.name) {
                continue;
            }

            match ExternalModuleHost::start(
                &descriptor,
                self.policy.provider_timeout_ms,
                self.policy.max_ipc_payload_bytes,
            ) {
                Ok(host) => {
                    self.external_hosts.push(host);
                    self.record_host_success(&descriptor.name, 0);
                }
                Err(err) => {
                    if !silent_mode {
                        eprintln!("module host start error for '{}': {err:?}", descriptor.name);
                    }
                    self.record_host_error(
                        &descriptor.name,
                        0,
                        false,
                        format!("host_start_error: {err:?}"),
                    );
                }
            }
        }
    }

    fn restart_external_host(&mut self, module_name: &str, silent_mode: bool) {
        {
            let entry = self.telemetry_entry_mut(module_name);
            entry.restart_count = entry.restart_count.saturating_add(1);
        }

        let now = Instant::now();
        if let Some(last_attempt) = self.last_restart_attempt.get(module_name) {
            if now.duration_since(*last_attempt).as_millis() < self.policy.host_restart_backoff_ms as u128 {
                return;
            }
        }
        self.last_restart_attempt.insert(module_name.to_string(), now);

        if let Some(index) = self.external_hosts.iter().position(|host| host.module_name == module_name) {
            let mut host = self.external_hosts.remove(index);
            host.shutdown();
        }

        if self.host_is_disabled(module_name) {
            if !silent_mode {
                eprintln!("module '{}' disabled after repeated failures", module_name);
            }
            return;
        }

        let descriptor = self
            .external_descriptors
            .iter()
            .find(|descriptor| descriptor.enabled && descriptor.name == module_name)
            .cloned();

        if let Some(descriptor) = descriptor {
            match ExternalModuleHost::start(
                &descriptor,
                self.policy.provider_timeout_ms,
                self.policy.max_ipc_payload_bytes,
            ) {
                Ok(host) => {
                    self.external_hosts.push(host);
                    self.record_host_success(module_name, 0);
                }
                Err(err) => {
                    if !silent_mode {
                        eprintln!("module host restart error for '{}': {err:?}", module_name);
                    }
                    self.record_host_error(module_name, 0, false, format!("host_restart_error: {err:?}"));
                }
            }
        }
    }

    fn shutdown_external_hosts(&mut self) {
        for host in &mut self.external_hosts {
            host.shutdown();
        }
        self.external_hosts.clear();
    }

    fn apply_ipc_actions(
        module_name: &str,
        actions: Vec<IpcAction>,
        app_state: &mut AppState,
        state: &mut ModuleRuntimeState,
        allowed_capabilities: Option<&BTreeSet<String>>,
    ) {
        if actions.is_empty() {
            return;
        }

        let snapshot = snapshot_from_app_state(app_state);
        let mut ctx = ModuleCtx::new(module_name.to_string(), snapshot);
        for action in actions {
            match action {
                IpcAction::SetQuery { text } => ctx.set_query(text),
                IpcAction::SetInputAccessory(accessory) => {
                    ctx.set_input_accessory(module_input_accessory_from_ipc(accessory));
                }
                IpcAction::ClearInputAccessory => ctx.clear_input_accessory(),
                IpcAction::ReplaceItems { items } => {
                    let sanitized = sanitize_ipc_items(items, module_name, app_state.silent_mode)
                        .into_iter()
                        .map(module_item_from_ipc_item)
                        .collect();
                    ctx.replace_items(sanitized);
                }
            }
        }
        Self::apply_ctx_requests(module_name, &mut ctx, app_state, state, allowed_capabilities);
    }

    fn apply_ctx_requests(
        module_name: &str,
        ctx: &mut ModuleCtx,
        app_state: &mut AppState,
        state: &mut ModuleRuntimeState,
        allowed_capabilities: Option<&BTreeSet<String>>,
    ) {
        let mut view = ActionRuntimeView {
            query: app_state.current_input.clone(),
            items: app_state
                .matching_items
                .iter()
                .cloned()
                .map(module_item_from_launcher_item)
                .collect(),
            selected_index: app_state.selected_index,
        };

        for request in ctx.take_action_requests() {
            if let Some(capability) = required_capability_for_action(&request) {
                if let Some(allowed) = allowed_capabilities {
                    if !allowed.contains(capability) {
                        if !app_state.silent_mode {
                            eprintln!(
                                "permission_denied module='{}' operation='{}' capability='{}'",
                                module_name,
                                action_name(&request),
                                capability
                            );
                        }
                        continue;
                    }
                }
            }

            let _ = apply_action_request(module_name, request, &mut view, state);
        }

        app_state.current_input = view.query;
        app_state.selected_index = view.selected_index;
        app_state.matching_items = view.items.into_iter().map(launcher_item_from_module_item).collect();
    }
}

fn descriptor_signature(descriptor: &ModuleDescriptor) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    descriptor.name.hash(&mut hasher);
    descriptor.version.hash(&mut hasher);
    descriptor.api_version.hash(&mut hasher);
    descriptor.kind.hash(&mut hasher);
    descriptor.enabled.hash(&mut hasher);
    descriptor.priority.hash(&mut hasher);
    descriptor.capabilities.hash(&mut hasher);
    descriptor.entry_code.hash(&mut hasher);
    descriptor.config_json.hash(&mut hasher);
    descriptor.readme.hash(&mut hasher);
    hasher.finish()
}

fn build_descriptor_signatures(descriptors: &[ModuleDescriptor]) -> BTreeMap<String, u64> {
    descriptors
        .iter()
        .map(|descriptor| (descriptor.name.clone(), descriptor_signature(descriptor)))
        .collect()
}

fn host_error_message(err: &HostClientError) -> String {
    match err {
        HostClientError::Io(message) => format!("io: {message}"),
        HostClientError::Protocol(message) => format!("protocol: {message}"),
        HostClientError::Timeout(message) => format!("timeout: {message}"),
    }
}

fn normalize_command_name(value: &str) -> String {
    value.trim().to_ascii_lowercase()
}

fn parse_namespaced_command(command: &str) -> Option<(String, String)> {
    let (module_name, module_command) = command.split_once("::")?;
    let module_name = normalize_command_name(module_name);
    let module_command = normalize_command_name(module_command);

    if module_name.is_empty() || module_command.is_empty() {
        return None;
    }

    Some((module_name, module_command))
}

fn split_registered_command_key(key: &str) -> Option<(&str, &str)> {
    key.split_once("::")
}

fn launcher_item_dedupe_key(item: &LauncherItem) -> String {
    let target = item.target.trim();
    if !target.is_empty() {
        return target.to_ascii_lowercase();
    }

    item.label.trim().to_ascii_lowercase()
}

fn module_key_event_to_ipc_key_event(event: &ModuleKeyEvent) -> IpcKeyEvent {
    IpcKeyEvent {
        key: event.key.clone(),
        ctrl: event.ctrl,
        alt: event.alt,
        shift: event.shift,
        meta: event.meta,
    }
}

fn should_dispatch_module_key_event(event: &ModuleKeyEvent) -> bool {
    if event.ctrl || event.alt || event.meta {
        return true;
    }

    matches!(
        event.key.as_str(),
        "enter" | "escape" | "tab" | "backspace" | "up" | "down"
    )
}

fn dedupe_launcher_items_by_priority(groups: Vec<Vec<LauncherItem>>) -> Vec<LauncherItem> {
    let mut seen: BTreeSet<String> = BTreeSet::new();
    let mut output = Vec::new();

    for group in groups {
        for item in group {
            let key = launcher_item_dedupe_key(&item);
            if seen.insert(key) {
                output.push(item);
            }
        }
    }

    output
}

fn action_name(request: &ModuleActionRequest) -> &'static str {
    match request {
        ModuleActionRequest::SetQuery(_) => "set_query",
        ModuleActionRequest::SetSelection(_) => "set_selection",
        ModuleActionRequest::MoveSelection(_) => "move_selection",
        ModuleActionRequest::Submit => "submit",
        ModuleActionRequest::Close => "close",
        ModuleActionRequest::AddItems(_) => "add_items",
        ModuleActionRequest::ReplaceItems(_) => "replace_items",
        ModuleActionRequest::SetInputAccessory(_) => "set_input_accessory",
        ModuleActionRequest::ClearInputAccessory => "clear_input_accessory",
        ModuleActionRequest::RegisterCommand(_) => "register_command",
        ModuleActionRequest::RegisterProvider(_) => "register_provider",
    }
}

fn required_capability_for_action(request: &ModuleActionRequest) -> Option<&'static str> {
    match request {
        ModuleActionRequest::RegisterProvider(_) => Some("providers"),
        ModuleActionRequest::RegisterCommand(_) => Some("commands"),
        ModuleActionRequest::SetInputAccessory(_) | ModuleActionRequest::ClearInputAccessory => {
            Some("input-accessory")
        }
        _ => None,
    }
}

fn snapshot_from_app_state(app_state: &AppState) -> ModuleSnapshot {
    ModuleSnapshot {
        query: app_state.current_input.clone(),
        items: app_state
            .matching_items
            .iter()
            .cloned()
            .map(module_item_from_launcher_item)
            .collect(),
        selected_index: app_state.selected_index,
        mode: if app_state.launcher_mode {
            ModuleMode::Launcher
        } else {
            ModuleMode::Stdin
        },
    }
}

fn ipc_snapshot_from_app_state(app_state: &AppState, include_items: bool) -> IpcSnapshot {
    let mode = if app_state.launcher_mode { "launcher" } else { "stdin" }.to_string();
    let items = if include_items {
        app_state
            .matching_items
            .iter()
            .cloned()
            .map(module_item_from_launcher_item)
            .map(module_item_to_ipc_item)
            .collect()
    } else {
        Vec::new()
    };

    IpcSnapshot {
        query: app_state.current_input.clone(),
        items,
        selected_index: app_state.selected_index,
        mode,
    }
}

fn source_to_name(source: LauncherSource) -> &'static str {
    match source {
        LauncherSource::Direct => "direct",
        LauncherSource::History => "history",
        LauncherSource::StartMenu => "start_menu",
        LauncherSource::Path => "path",
    }
}

fn name_to_source(value: &str) -> LauncherSource {
    match value {
        "history" => LauncherSource::History,
        "start_menu" => LauncherSource::StartMenu,
        "path" => LauncherSource::Path,
        _ => LauncherSource::Direct,
    }
}

fn module_item_from_launcher_item(item: LauncherItem) -> ModuleItem {
    ModuleItem {
        id: item.target.clone(),
        title: item.label,
        subtitle: Some(item.target.clone()),
        source: Some(source_to_name(item.source).to_string()),
        action: ModuleAction::LaunchTarget {
            target: item.target,
        },
        capabilities: ModuleItemCapabilities {
            quick_select_key: item.quick_select_key,
        },
        decorations: ModuleItemDecorations {
            badge: item.trailing_badge,
            badge_kind: Some(BadgeKind::Shortcut),
            hint: item.trailing_hint,
            icon: None,
        },
    }
}

fn dedupe_module_items(items: Vec<ModuleItem>) -> Vec<ModuleItem> {
    let mut seen: BTreeSet<String> = BTreeSet::new();
    let mut output = Vec::with_capacity(items.len());

    for item in items {
        let key = format!("{}::{}", item.id, item.title.to_lowercase());
        if seen.insert(key) {
            output.push(item);
        }
    }

    output
}

fn module_item_to_ipc_item(item: ModuleItem) -> IpcItem {
    let target = match item.action {
        ModuleAction::LaunchTarget { target } => Some(target),
        ModuleAction::RunCommand { name, args } => {
            if args.is_empty() {
                Some(name)
            } else {
                Some(format!("{} {}", name, args.join(" ")))
            }
        }
        ModuleAction::Noop => None,
    };

    IpcItem {
        id: item.id,
        title: item.title,
        subtitle: item.subtitle,
        source: item.source,
        target,
        quick_select_key: item.capabilities.quick_select_key,
        badge: item.decorations.badge,
        hint: item.decorations.hint,
    }
}

fn sanitize_ipc_items(items: Vec<IpcItem>, module_name: &str, silent_mode: bool) -> Vec<IpcItem> {
    items
        .into_iter()
        .enumerate()
        .filter_map(|(index, item)| match sanitize_ipc_item(item) {
            Ok(item) => Some(item),
            Err(reason) => {
                if !silent_mode {
                    eprintln!(
                        "module ipc item dropped module='{}' index={} reason='{}'",
                        module_name, index, reason
                    );
                }
                None
            }
        })
        .collect()
}

fn sanitize_ipc_item(item: IpcItem) -> Result<IpcItem, String> {
    let id = sanitize_required_single_line(item.id, IPC_ITEM_MAX_ID_LEN, "id")?;
    let title = sanitize_required_single_line(item.title, IPC_ITEM_MAX_TITLE_LEN, "title")?;

    let subtitle = sanitize_optional_single_line(item.subtitle, IPC_ITEM_MAX_SUBTITLE_LEN);

    let source = sanitize_optional_single_line(item.source, IPC_ITEM_MAX_SOURCE_LEN)
        .and_then(|value| sanitize_source(value));

    let target = sanitize_optional_multiline(item.target, IPC_ITEM_MAX_TARGET_LEN);
    let quick_select_key = sanitize_quick_select_key(item.quick_select_key);
    let badge = sanitize_optional_single_line(item.badge, IPC_ITEM_MAX_BADGE_LEN);
    let hint = sanitize_optional_single_line(item.hint, IPC_ITEM_MAX_HINT_LEN);

    Ok(IpcItem {
        id,
        title,
        subtitle,
        source,
        target,
        quick_select_key,
        badge,
        hint,
    })
}

fn sanitize_required_single_line(value: String, max_len: usize, field_name: &str) -> Result<String, String> {
    let sanitized = sanitize_single_line_string(value, max_len);
    if sanitized.is_empty() {
        return Err(format!("{field_name} is required"));
    }

    Ok(sanitized)
}

fn sanitize_optional_single_line(value: Option<String>, max_len: usize) -> Option<String> {
    value.and_then(|value| {
        let sanitized = sanitize_single_line_string(value, max_len);
        if sanitized.is_empty() {
            None
        } else {
            Some(sanitized)
        }
    })
}

fn sanitize_optional_multiline(value: Option<String>, max_len: usize) -> Option<String> {
    value.and_then(|value| {
        let mut sanitized = value
            .trim()
            .chars()
            .map(|ch| if ch == '\0' { ' ' } else { ch })
            .collect::<String>();

        if sanitized.len() > max_len {
            sanitized.truncate(max_len);
        }

        if sanitized.is_empty() {
            None
        } else {
            Some(sanitized)
        }
    })
}

fn sanitize_single_line_string(value: String, max_len: usize) -> String {
    let mut out = String::new();
    let mut previous_is_whitespace = false;

    for ch in value.trim().chars() {
        let normalized = if ch.is_control() { ' ' } else { ch };
        if normalized.is_whitespace() {
            if !previous_is_whitespace {
                out.push(' ');
                previous_is_whitespace = true;
            }
        } else {
            out.push(normalized);
            previous_is_whitespace = false;
        }

        if out.len() >= max_len {
            break;
        }
    }

    out.trim().to_string()
}

fn sanitize_source(value: String) -> Option<String> {
    let lowered = value.to_ascii_lowercase();
    if lowered
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-')
    {
        Some(lowered)
    } else {
        None
    }
}

fn sanitize_quick_select_key(value: Option<String>) -> Option<String> {
    let value = sanitize_optional_single_line(value, 1)?;
    let mut chars = value.chars();
    let ch = chars.next()?;

    if chars.next().is_some() {
        return None;
    }

    if ch.is_ascii_digit() {
        Some(ch.to_string())
    } else {
        None
    }
}

fn module_item_from_ipc_item(item: IpcItem) -> ModuleItem {
    let action = item
        .target
        .map(|target| ModuleAction::LaunchTarget { target })
        .unwrap_or(ModuleAction::Noop);

    ModuleItem {
        id: item.id,
        title: item.title,
        subtitle: item.subtitle,
        source: item.source,
        action,
        capabilities: ModuleItemCapabilities {
            quick_select_key: item.quick_select_key,
        },
        decorations: ModuleItemDecorations {
            badge: item.badge,
            badge_kind: Some(BadgeKind::Shortcut),
            hint: item.hint,
            icon: None,
        },
    }
}

fn launcher_item_from_module_item(item: ModuleItem) -> LauncherItem {
    let target = match item.action {
        ModuleAction::LaunchTarget { target } => target,
        ModuleAction::RunCommand { name, args } => {
            if args.is_empty() {
                name
            } else {
                format!("{} {}", name, args.join(" "))
            }
        }
        ModuleAction::Noop => item.subtitle.clone().unwrap_or_else(|| item.title.clone()),
    };

    let source = item
        .source
        .as_deref()
        .map(name_to_source)
        .unwrap_or(LauncherSource::Direct);

    let mut launcher_item = LauncherItem::new(item.title, target, source);
    launcher_item.quick_select_key = item.capabilities.quick_select_key;
    launcher_item.trailing_badge = item.decorations.badge;
    launcher_item.trailing_hint = item.decorations.hint;
    launcher_item
}

fn module_input_accessory_from_ipc(accessory: IpcInputAccessory) -> ModuleInputAccessory {
    ModuleInputAccessory {
        text: sanitize_single_line_string(accessory.text, IPC_ITEM_MAX_HINT_LEN),
        kind: input_accessory_kind_from_str(accessory.kind.as_deref()),
        priority: accessory.priority.unwrap_or(0),
    }
}

fn input_accessory_kind_from_str(kind: Option<&str>) -> InputAccessoryKind {
    match kind.unwrap_or("hint").trim().to_ascii_lowercase().as_str() {
        "info" => InputAccessoryKind::Info,
        "success" => InputAccessoryKind::Success,
        "warning" => InputAccessoryKind::Warning,
        "error" => InputAccessoryKind::Error,
        _ => InputAccessoryKind::Hint,
    }
}

pub fn input_accessory_text(accessory: &ModuleInputAccessory) -> String {
    accessory.text.clone()
}

pub fn quick_select_badge_text(capabilities: &ModuleItemCapabilities, decorations: &ModuleItemDecorations) -> Option<String> {
    if let Some(key) = &capabilities.quick_select_key {
        return Some(key.clone());
    }

    match decorations.badge_kind {
        Some(BadgeKind::Shortcut) | None => decorations.badge.clone(),
        Some(BadgeKind::Status) | Some(BadgeKind::Tag) => decorations.badge.clone(),
    }
}

#[derive(Default)]
pub struct BuiltinLifecycleModule;

impl RuntimeModule for BuiltinLifecycleModule {
    fn name(&self) -> &str {
        "builtin.lifecycle"
    }

    fn on_load(&mut self, ctx: &mut ModuleCtx) {
        ctx.register_command(ModuleCommandDef {
            name: "modules.list".to_string(),
            description: Some("Lista módulos cargados".to_string()),
        });
        ctx.log("builtin.lifecycle loaded");
    }

    fn on_unload(&mut self, ctx: &mut ModuleCtx) {
        ctx.log("builtin.lifecycle unloaded");
    }

    fn on_command(&mut self, command: &str, _args: &[String], ctx: &mut ModuleCtx) {
        if command == "modules.list" {
            ctx.toast("modules.list ejecutado (builtin.lifecycle)");
        }
    }
}

#[derive(Default)]
pub struct BuiltinQueryProviderModule;

impl RuntimeModule for BuiltinQueryProviderModule {
    fn name(&self) -> &str {
        "builtin.query-provider"
    }

    fn on_load(&mut self, ctx: &mut ModuleCtx) {
        ctx.register_provider(ModuleProviderDef {
            name: "query-provider".to_string(),
            priority: 0,
        });
    }

    fn provide_items(&mut self, query: &str, _ctx: &mut ModuleCtx) -> Vec<ModuleItem> {
        let trimmed = query.trim();
        if trimmed.is_empty() {
            return Vec::new();
        }

        if let Some(rest) = trimmed.strip_prefix("=") {
            let expr = rest.trim();
            if !expr.is_empty() {
                return vec![ModuleItem {
                    id: format!("calc::{expr}"),
                    title: format!("Calc: {expr}"),
                    subtitle: Some("builtin provider".to_string()),
                    source: Some("module_provider".to_string()),
                    action: ModuleAction::Noop,
                    capabilities: ModuleItemCapabilities {
                        quick_select_key: Some("1".to_string()),
                    },
                    decorations: ModuleItemDecorations {
                        badge: Some("1".to_string()),
                        badge_kind: Some(BadgeKind::Shortcut),
                        hint: Some("presiona Enter para copiar en futuro".to_string()),
                        icon: None,
                    },
                }];
            }
        }

        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::{
        dedupe_launcher_items_by_priority, parse_namespaced_command, sanitize_ipc_item, sanitize_ipc_items,
        BuiltinLifecycleModule, DedupeSourcePriority, ExternalModuleStatus, HostTelemetry, IpcAction, ModuleRuntime,
        ResolvedCommandRoute, IPC_ITEM_MAX_BADGE_LEN, IPC_ITEM_MAX_HINT_LEN, IPC_ITEM_MAX_ID_LEN,
        IPC_ITEM_MAX_SOURCE_LEN, IPC_ITEM_MAX_SUBTITLE_LEN, IPC_ITEM_MAX_TARGET_LEN,
        IPC_ITEM_MAX_TITLE_LEN,
    };
    use crate::app_state::{AppState, LauncherItem, LauncherSource};
    use crate::modules::ipc::{IpcInputAccessory, IpcItem};
    use crate::modules::types::{
        BadgeKind, InputAccessoryKind, ModuleCommandDef, ModuleInputAccessory, ModuleItemCapabilities,
        ModuleItemDecorations,
    };

    #[test]
    fn sanitize_ipc_item_rejects_missing_required_fields() {
        let missing_id = IpcItem {
            id: " \n ".to_string(),
            title: "valid".to_string(),
            subtitle: None,
            source: None,
            target: None,
            quick_select_key: None,
            badge: None,
            hint: None,
        };
        assert!(sanitize_ipc_item(missing_id).is_err());

        let missing_title = IpcItem {
            id: "valid".to_string(),
            title: "\t\n".to_string(),
            subtitle: None,
            source: None,
            target: None,
            quick_select_key: None,
            badge: None,
            hint: None,
        };
        assert!(sanitize_ipc_item(missing_title).is_err());
    }

    #[test]
    fn sanitize_ipc_item_normalizes_and_truncates_fields() {
        let raw = IpcItem {
            id: "  item\n\tid  ".to_string(),
            title: "  title\n\tvalue ".to_string(),
            subtitle: Some(format!("{}x", "a".repeat(IPC_ITEM_MAX_SUBTITLE_LEN))),
            source: Some(" Module_Source ".to_string()),
            target: Some(format!("  {}x  ", "b".repeat(IPC_ITEM_MAX_TARGET_LEN))),
            quick_select_key: Some(" 2 ".to_string()),
            badge: Some(format!("{}x", "c".repeat(IPC_ITEM_MAX_BADGE_LEN))),
            hint: Some(format!("{}x", "d".repeat(IPC_ITEM_MAX_HINT_LEN))),
        };

        let sanitized = sanitize_ipc_item(raw).expect("item should sanitize");

        assert_eq!(sanitized.id, "item id");
        assert_eq!(sanitized.title, "title value");
        assert_eq!(sanitized.subtitle.as_deref().map(str::len), Some(IPC_ITEM_MAX_SUBTITLE_LEN));
        assert_eq!(sanitized.source.as_deref(), Some("module_source"));
        assert_eq!(sanitized.target.as_deref().map(str::len), Some(IPC_ITEM_MAX_TARGET_LEN));
        assert_eq!(sanitized.quick_select_key.as_deref(), Some("2"));
        assert_eq!(sanitized.badge.as_deref().map(str::len), Some(IPC_ITEM_MAX_BADGE_LEN));
        assert_eq!(sanitized.hint.as_deref().map(str::len), Some(IPC_ITEM_MAX_HINT_LEN));
    }

    #[test]
    fn sanitize_ipc_item_drops_invalid_optional_fields() {
        let raw = IpcItem {
            id: "valid".to_string(),
            title: "valid".to_string(),
            subtitle: Some("\n\t".to_string()),
            source: Some("bad source!".to_string()),
            target: Some("\t\n".to_string()),
            quick_select_key: Some("x".to_string()),
            badge: Some("\n".to_string()),
            hint: Some("\t".to_string()),
        };

        let sanitized = sanitize_ipc_item(raw).expect("item should sanitize");

        assert_eq!(sanitized.subtitle, None);
        assert_eq!(sanitized.source, None);
        assert_eq!(sanitized.target, None);
        assert_eq!(sanitized.quick_select_key, None);
        assert_eq!(sanitized.badge, None);
        assert_eq!(sanitized.hint, None);
    }

    #[test]
    fn sanitize_ipc_items_drops_invalid_entries_safely() {
        let items = vec![
            IpcItem {
                id: "ok-1".to_string(),
                title: "Title 1".to_string(),
                subtitle: None,
                source: None,
                target: None,
                quick_select_key: None,
                badge: None,
                hint: None,
            },
            IpcItem {
                id: "\n".to_string(),
                title: "Title 2".to_string(),
                subtitle: None,
                source: None,
                target: None,
                quick_select_key: None,
                badge: None,
                hint: None,
            },
            IpcItem {
                id: "ok-3".to_string(),
                title: "Title 3".to_string(),
                subtitle: None,
                source: None,
                target: None,
                quick_select_key: None,
                badge: None,
                hint: None,
            },
        ];

        let sanitized = sanitize_ipc_items(items, "test.module", true);
        assert_eq!(sanitized.len(), 2);
        assert_eq!(sanitized[0].id, "ok-1");
        assert_eq!(sanitized[1].id, "ok-3");
    }

    #[test]
    fn sanitize_ipc_item_enforces_required_field_limits() {
        let raw = IpcItem {
            id: format!("{}x", "a".repeat(IPC_ITEM_MAX_ID_LEN)),
            title: format!("{}x", "b".repeat(IPC_ITEM_MAX_TITLE_LEN)),
            subtitle: None,
            source: Some(format!("{}x", "c".repeat(IPC_ITEM_MAX_SOURCE_LEN))),
            target: None,
            quick_select_key: None,
            badge: None,
            hint: None,
        };

        let sanitized = sanitize_ipc_item(raw).expect("item should sanitize");
        assert_eq!(sanitized.id.len(), IPC_ITEM_MAX_ID_LEN);
        assert_eq!(sanitized.title.len(), IPC_ITEM_MAX_TITLE_LEN);
        assert_eq!(sanitized.source.as_deref().map(str::len), Some(IPC_ITEM_MAX_SOURCE_LEN));
    }

    #[test]
    fn dedupe_launcher_items_respects_priority_order() {
        let core = vec![
            LauncherItem::new("Core A".to_string(), "same-target".to_string(), LauncherSource::Direct),
            LauncherItem::new("Core B".to_string(), "core-only".to_string(), LauncherSource::Direct),
        ];
        let provider = vec![
            LauncherItem::new(
                "Provider A".to_string(),
                "same-target".to_string(),
                LauncherSource::Direct,
            ),
            LauncherItem::new(
                "Provider B".to_string(),
                "provider-only".to_string(),
                LauncherSource::Direct,
            ),
        ];

        let core_first = dedupe_launcher_items_by_priority(vec![core.clone(), provider.clone()]);
        assert_eq!(core_first.len(), 3);
        assert_eq!(core_first[0].label, "Core A");
        assert_eq!(core_first[1].label, "Core B");
        assert_eq!(core_first[2].label, "Provider B");

        let provider_first = dedupe_launcher_items_by_priority(vec![provider, core]);
        assert_eq!(provider_first.len(), 3);
        assert_eq!(provider_first[0].label, "Provider A");
        assert_eq!(provider_first[1].label, "Provider B");
        assert_eq!(provider_first[2].label, "Core B");
    }

    #[test]
    fn plain_text_keys_are_not_dispatched_to_modules() {
        assert!(!super::should_dispatch_module_key_event(&super::ModuleKeyEvent {
            key: "a".to_string(),
            ctrl: false,
            alt: false,
            shift: false,
            meta: false,
        }));
        assert!(!super::should_dispatch_module_key_event(&super::ModuleKeyEvent {
            key: "1".to_string(),
            ctrl: false,
            alt: false,
            shift: false,
            meta: false,
        }));
        assert!(super::should_dispatch_module_key_event(&super::ModuleKeyEvent {
            key: "b".to_string(),
            ctrl: true,
            alt: false,
            shift: false,
            meta: false,
        }));
        assert!(super::should_dispatch_module_key_event(&super::ModuleKeyEvent {
            key: "enter".to_string(),
            ctrl: false,
            alt: false,
            shift: false,
            meta: false,
        }));
    }

    #[test]
    fn ipc_set_query_updates_app_input() {
        let mut app_state = AppState {
            current_input: "before".to_string(),
            ..Default::default()
        };
        let mut state = super::ModuleRuntimeState::default();
        let actions = vec![IpcAction::SetQuery {
            text: "/shortcuts::bind ".to_string(),
        }];

        ModuleRuntime::apply_ipc_actions("shortcuts", actions, &mut app_state, &mut state, None);

        assert_eq!(app_state.current_input, "/shortcuts::bind ");
    }

    #[test]
    fn ipc_replace_items_updates_visible_app_items() {
        let mut app_state = AppState {
            matching_items: vec![LauncherItem::new(
                "Core Item".to_string(),
                "core-target".to_string(),
                LauncherSource::Direct,
            )],
            ..Default::default()
        };
        let mut state = super::ModuleRuntimeState::default();
        let actions = vec![IpcAction::ReplaceItems {
            items: vec![IpcItem {
                id: "local-scripts::build".to_string(),
                title: "build".to_string(),
                subtitle: Some("modules/local-scripts/scripts/build.ps1".to_string()),
                source: Some("local-scripts".to_string()),
                target: Some("powershell.exe -NoProfile -File modules/local-scripts/scripts/build.ps1".to_string()),
                quick_select_key: None,
                badge: Some("ps1".to_string()),
                hint: Some("modules/local-scripts/scripts/build.ps1".to_string()),
            }],
        }];

        ModuleRuntime::apply_ipc_actions("local-scripts", actions, &mut app_state, &mut state, None);

        assert!(state.items_replaced_in_cycle);
        assert_eq!(app_state.matching_items.len(), 1);
        assert_eq!(app_state.matching_items[0].label, "build");
        assert_eq!(app_state.matching_items[0].trailing_badge.as_deref(), Some("ps1"));
    }

    #[test]
    fn merge_rank_dataset_uses_configurable_source_priority() {
        let mut runtime = ModuleRuntime::new();

        let core = vec![LauncherItem::new(
            "Core Item".to_string(),
            "same-target".to_string(),
            LauncherSource::Direct,
        )];
        let provider = vec![LauncherItem::new(
            "Provider Item".to_string(),
            "same-target".to_string(),
            LauncherSource::Direct,
        )];

        runtime.configure_policy(super::ModuleRuntimePolicy {
            dedupe_source_priority: DedupeSourcePriority::CoreFirst,
            ..Default::default()
        });
        let merged_core_first = runtime.merge_rank_dataset(core.clone(), provider.clone());
        assert_eq!(merged_core_first.len(), 1);
        assert_eq!(merged_core_first[0].label, "Core Item");

        runtime.configure_policy(super::ModuleRuntimePolicy {
            dedupe_source_priority: DedupeSourcePriority::ProviderFirst,
            ..Default::default()
        });
        let merged_provider_first = runtime.merge_rank_dataset(core, provider);
        assert_eq!(merged_provider_first.len(), 1);
        assert_eq!(merged_provider_first[0].label, "Provider Item");
    }

    #[test]
    fn command_route_requires_namespace_for_colliding_aliases() {
        let mut runtime = ModuleRuntime::new();
        runtime.state.register_command(
            "module.a",
            ModuleCommandDef {
                name: "open".to_string(),
                description: None,
            },
        );
        runtime.state.register_command(
            "module.b",
            ModuleCommandDef {
                name: "open".to_string(),
                description: None,
            },
        );

        let route = runtime.resolve_command_route("open", true);
        assert!(route.is_none());

        let namespaced = parse_namespaced_command("module.a::open").expect("must parse");
        assert_eq!(namespaced.0, "module.a");
        assert_eq!(namespaced.1, "open");
    }

    #[test]
    fn resolved_command_route_matches_target_module() {
        let targeted = ResolvedCommandRoute {
            target_module: Some("module.alpha".to_string()),
            command: "run".to_string(),
        };
        assert!(targeted.matches_module("module.alpha"));
        assert!(!targeted.matches_module("module.beta"));

        let broadcast = ResolvedCommandRoute {
            target_module: None,
            command: "run".to_string(),
        };
        assert!(broadcast.matches_module("module.alpha"));
        assert!(broadcast.matches_module("module.beta"));
    }

    #[test]
    fn runtime_command_list_sets_accessory_with_loaded_modules() {
        let mut runtime = ModuleRuntime::new();
        runtime.register_builtin_module(Box::new(BuiltinLifecycleModule));

        let handled = runtime.runtime_command("modules.list", true);
        assert!(handled);

        let text = runtime
            .active_input_accessory()
            .map(|value| value.text)
            .unwrap_or_default();
        assert!(text.contains("builtin.lifecycle"));
    }

    #[test]
    fn runtime_command_telemetry_reset_clears_state() {
        let mut runtime = ModuleRuntime::new();
        runtime
            .host_telemetry
            .insert("mod.host".to_string(), HostTelemetry::default());

        let handled = runtime.runtime_command("modules.telemetry.reset", true);
        assert!(handled);
        assert!(runtime.host_telemetry.is_empty());
    }

    #[test]
    fn host_health_transitions_to_disabled_after_timeouts() {
        let mut runtime = ModuleRuntime::new();

        runtime.record_host_error("mod.host", 10, true, "timeout-1".to_string());
        let status_after_first = runtime
            .host_health
            .get("mod.host")
            .map(|value| value.status)
            .unwrap_or(ExternalModuleStatus::Unloaded);
        assert_eq!(status_after_first, ExternalModuleStatus::Degraded);

        runtime.record_host_error("mod.host", 10, true, "timeout-2".to_string());
        runtime.record_host_error("mod.host", 10, true, "timeout-3".to_string());

        let status_after_disable = runtime
            .host_health
            .get("mod.host")
            .map(|value| value.status)
            .unwrap_or(ExternalModuleStatus::Unloaded);
        assert_eq!(status_after_disable, ExternalModuleStatus::Disabled);

        runtime.record_host_success("mod.host", 1);
        let status_after_success = runtime
            .host_health
            .get("mod.host")
            .map(|value| value.status)
            .unwrap_or(ExternalModuleStatus::Unloaded);
        assert_eq!(status_after_success, ExternalModuleStatus::Loaded);
    }

    #[test]
    fn runtime_command_reload_sets_feedback_accessory() {
        let mut runtime = ModuleRuntime::new();

        let handled = runtime.runtime_command("modules.reload", true);
        assert!(handled);

        let text = runtime
            .active_input_accessory()
            .map(|value| value.text)
            .unwrap_or_default();
        assert!(text.contains("modules reloaded"));
    }

    #[test]
    fn quick_select_badge_prefers_shortcut_key() {
        let capabilities = ModuleItemCapabilities {
            quick_select_key: Some("7".to_string()),
        };
        let decorations = ModuleItemDecorations {
            badge: Some("HOT".to_string()),
            badge_kind: Some(BadgeKind::Status),
            hint: None,
            icon: None,
        };

        let badge = super::quick_select_badge_text(&capabilities, &decorations);
        assert_eq!(badge.as_deref(), Some("7"));
    }

    #[test]
    fn input_accessory_text_omits_kind_prefix() {
        let text = super::input_accessory_text(&ModuleInputAccessory {
            text: "ready".to_string(),
            kind: InputAccessoryKind::Success,
            priority: 1,
        });
        assert_eq!(text, "ready");
    }

    #[test]
    fn ipc_input_accessory_maps_kind_and_priority() {
        let accessory = super::module_input_accessory_from_ipc(IpcInputAccessory {
            text: "=4".to_string(),
            kind: Some("success".to_string()),
            priority: Some(100),
        });

        assert_eq!(accessory.text, "=4");
        assert_eq!(accessory.kind, InputAccessoryKind::Success);
        assert_eq!(accessory.priority, 100);
    }
}
