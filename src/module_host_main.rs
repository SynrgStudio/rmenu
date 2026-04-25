#[path = "modules/ipc.rs"]
mod ipc;

use std::io::{self, BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

use ipc::{
    HostRequest, HostRequestPayload, HostResponse, HostResponsePayload, IpcAction, IpcItem,
    IpcKeyEvent, IpcSnapshot, ModuleInitPayload,
};
use serde::{Deserialize, Serialize};

const DEFAULT_MAX_IPC_PAYLOAD_BYTES: usize = 256 * 1024;

struct HostState {
    module: Option<ModuleInitPayload>,
    loaded: bool,
    runtime: Option<NodeRuntime>,
    max_ipc_payload_bytes: usize,
}

impl Default for HostState {
    fn default() -> Self {
        Self {
            module: None,
            loaded: false,
            runtime: None,
            max_ipc_payload_bytes: read_max_ipc_payload_bytes(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
enum WorkerRequest {
    #[serde(rename = "init")]
    Init {
        entry_code: String,
        config_json: Option<String>,
    },
    #[serde(rename = "hook")]
    Hook {
        hook: String,
        query: Option<String>,
        key_event: Option<IpcKeyEvent>,
        items: Option<Vec<IpcItem>>,
        command: Option<String>,
        args: Option<Vec<String>>,
        snapshot: Option<IpcSnapshot>,
    },
    #[serde(rename = "shutdown")]
    Shutdown,
}

#[derive(Debug, Serialize, Deserialize)]
struct WorkerResponse {
    ok: bool,
    items: Option<Vec<IpcItem>>,
    actions: Option<Vec<IpcAction>>,
    error: Option<String>,
}

struct NodeRuntime {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
}

impl NodeRuntime {
    fn start(module: &ModuleInitPayload, max_ipc_payload_bytes: usize) -> Result<Self, String> {
        let mut child = Command::new("node")
            .arg("--input-type=module")
            .arg("-e")
            .arg(node_bridge_script())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|err| format!("cannot spawn node: {err}"))?;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| "node runtime stdin unavailable".to_string())?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| "node runtime stdout unavailable".to_string())?;

        let mut runtime = Self {
            child,
            stdin,
            stdout: BufReader::new(stdout),
        };

        let init_response = runtime.send(
            WorkerRequest::Init {
                entry_code: module.entry_code.clone(),
                config_json: module.config_json.clone(),
            },
            max_ipc_payload_bytes,
        )?;

        if !init_response.ok {
            return Err(init_response
                .error
                .unwrap_or_else(|| "node init failed".to_string()));
        }

        Ok(runtime)
    }

    fn hook(
        &mut self,
        hook: &str,
        query: Option<String>,
        key_event: Option<IpcKeyEvent>,
        items: Option<Vec<IpcItem>>,
        command: Option<String>,
        args: Option<Vec<String>>,
        snapshot: Option<IpcSnapshot>,
        max_ipc_payload_bytes: usize,
    ) -> Result<WorkerResponse, String> {
        self.send(
            WorkerRequest::Hook {
                hook: hook.to_string(),
                query,
                key_event,
                items,
                command,
                args,
                snapshot,
            },
            max_ipc_payload_bytes,
        )
    }

    fn shutdown(&mut self, max_ipc_payload_bytes: usize) {
        let _ = self.send(WorkerRequest::Shutdown, max_ipc_payload_bytes);
        let _ = self.child.kill();
        let _ = self.child.wait();
    }

    fn send(
        &mut self,
        request: WorkerRequest,
        max_ipc_payload_bytes: usize,
    ) -> Result<WorkerResponse, String> {
        let encoded = serde_json::to_string(&request).map_err(|err| err.to_string())?;
        if encoded.len() > max_ipc_payload_bytes {
            return Err(format!(
                "worker request exceeds max_ipc_payload_bytes ({} > {})",
                encoded.len(),
                max_ipc_payload_bytes
            ));
        }

        self.stdin
            .write_all(encoded.as_bytes())
            .and_then(|_| self.stdin.write_all(b"\n"))
            .and_then(|_| self.stdin.flush())
            .map_err(|err| err.to_string())?;

        let mut line = String::new();
        let read = self
            .stdout
            .read_line(&mut line)
            .map_err(|err| err.to_string())?;
        if read == 0 {
            return Err("node runtime closed stdout".to_string());
        }

        if line.len() > max_ipc_payload_bytes {
            return Err(format!(
                "worker response exceeds max_ipc_payload_bytes ({} > {})",
                line.len(),
                max_ipc_payload_bytes
            ));
        }

        serde_json::from_str::<WorkerResponse>(line.trim()).map_err(|err| err.to_string())
    }
}

fn main() {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut state = HostState::default();

    for line in stdin.lock().lines() {
        let Ok(line) = line else {
            break;
        };

        let response = match serde_json::from_str::<HostRequest>(&line) {
            Ok(request) => handle_request(request, &mut state),
            Err(err) => HostResponse {
                id: 0,
                payload: HostResponsePayload::Error {
                    message: format!("invalid request json: {err}"),
                    recoverable: true,
                },
            },
        };

        let encoded = match serde_json::to_string(&response) {
            Ok(json) => json,
            Err(err) => format!(
                "{{\"id\":0,\"payload\":{{\"type\":\"Error\",\"data\":{{\"message\":\"serialize error: {}\",\"recoverable\":false}}}}}}",
                err
            ),
        };

        if stdout.write_all(encoded.as_bytes()).is_err() {
            break;
        }
        if stdout.write_all(b"\n").is_err() {
            break;
        }
        if stdout.flush().is_err() {
            break;
        }

        if matches!(response.payload, HostResponsePayload::Ack)
            && line.contains("\"type\":\"Shutdown\"")
        {
            break;
        }
    }

    if let Some(runtime) = state.runtime.as_mut() {
        runtime.shutdown(state.max_ipc_payload_bytes);
    }
}

fn handle_request(request: HostRequest, state: &mut HostState) -> HostResponse {
    let payload = match request.payload {
        HostRequestPayload::Ping => HostResponsePayload::Pong,
        HostRequestPayload::Initialize(module) => {
            if let Some(runtime) = state.runtime.as_mut() {
                runtime.shutdown(state.max_ipc_payload_bytes);
            }

            match NodeRuntime::start(&module, state.max_ipc_payload_bytes) {
                Ok(runtime) => {
                    state.module = Some(module);
                    state.runtime = Some(runtime);
                    HostResponsePayload::Ack
                }
                Err(message) => HostResponsePayload::Error {
                    message,
                    recoverable: true,
                },
            }
        }
        HostRequestPayload::OnLoad { snapshot } => {
            state.loaded = true;
            run_hook(state, "onLoad", None, None, None, None, None, snapshot)
                .unwrap_or(HostResponsePayload::Ack)
        }
        HostRequestPayload::OnQueryChange { query, snapshot } => {
            if !state.loaded {
                HostResponsePayload::Error {
                    message: "module not loaded".to_string(),
                    recoverable: true,
                }
            } else {
                run_hook(
                    state,
                    "onQueryChange",
                    Some(query),
                    None,
                    None,
                    None,
                    None,
                    Some(snapshot),
                )
                .unwrap_or(HostResponsePayload::Ack)
            }
        }
        HostRequestPayload::OnKey { event, snapshot } => {
            if !state.loaded {
                HostResponsePayload::Error {
                    message: "module not loaded".to_string(),
                    recoverable: true,
                }
            } else {
                run_hook(
                    state,
                    "onKey",
                    None,
                    Some(event),
                    None,
                    None,
                    None,
                    Some(snapshot),
                )
                .unwrap_or(HostResponsePayload::Ack)
            }
        }
        HostRequestPayload::ProvideItems { query, snapshot } => {
            if !state.loaded {
                HostResponsePayload::Error {
                    message: "module not loaded".to_string(),
                    recoverable: true,
                }
            } else {
                run_hook(
                    state,
                    "provideItems",
                    Some(query),
                    None,
                    None,
                    None,
                    None,
                    Some(snapshot),
                )
                .unwrap_or(HostResponsePayload::ProvideItemsResult { items: Vec::new() })
            }
        }
        HostRequestPayload::DecorateItems { items, snapshot } => {
            if !state.loaded {
                HostResponsePayload::Error {
                    message: "module not loaded".to_string(),
                    recoverable: true,
                }
            } else {
                run_hook(
                    state,
                    "decorateItems",
                    None,
                    None,
                    Some(items),
                    None,
                    None,
                    Some(snapshot),
                )
                .unwrap_or(HostResponsePayload::DecorateItemsResult { items: Vec::new() })
            }
        }
        HostRequestPayload::OnCommand {
            command,
            args,
            snapshot,
        } => {
            if !state.loaded {
                HostResponsePayload::Error {
                    message: "module not loaded".to_string(),
                    recoverable: true,
                }
            } else {
                run_hook(
                    state,
                    "onCommand",
                    None,
                    None,
                    None,
                    Some(command),
                    Some(args),
                    Some(snapshot),
                )
                .unwrap_or(HostResponsePayload::Ack)
            }
        }
        HostRequestPayload::OnUnload { snapshot } => {
            state.loaded = false;
            let _ = run_hook(state, "onUnload", None, None, None, None, None, snapshot);
            HostResponsePayload::Ack
        }
        HostRequestPayload::Shutdown => {
            state.loaded = false;
            if let Some(runtime) = state.runtime.as_mut() {
                runtime.shutdown(state.max_ipc_payload_bytes);
            }
            state.runtime = None;
            HostResponsePayload::Ack
        }
    };

    HostResponse {
        id: request.id,
        payload,
    }
}

fn run_hook(
    state: &mut HostState,
    hook: &str,
    query: Option<String>,
    key_event: Option<IpcKeyEvent>,
    items: Option<Vec<IpcItem>>,
    command: Option<String>,
    args: Option<Vec<String>>,
    snapshot: Option<IpcSnapshot>,
) -> Option<HostResponsePayload> {
    let runtime = state.runtime.as_mut()?;
    let response = runtime
        .hook(
            hook,
            query,
            key_event,
            items,
            command,
            args,
            snapshot,
            state.max_ipc_payload_bytes,
        )
        .ok()?;

    if !response.ok {
        return Some(HostResponsePayload::Error {
            message: response
                .error
                .unwrap_or_else(|| "script execution failed".to_string()),
            recoverable: true,
        });
    }

    match hook {
        "provideItems" => Some(HostResponsePayload::ProvideItemsResult {
            items: response.items.unwrap_or_default(),
        }),
        "decorateItems" => Some(HostResponsePayload::DecorateItemsResult {
            items: response.items.unwrap_or_default(),
        }),
        _ => {
            let actions = response.actions.unwrap_or_default();
            if actions.is_empty() {
                Some(HostResponsePayload::Ack)
            } else {
                Some(HostResponsePayload::Actions { actions })
            }
        }
    }
}

fn read_max_ipc_payload_bytes() -> usize {
    std::env::var("RMODULE_MAX_IPC_BYTES")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(DEFAULT_MAX_IPC_PAYLOAD_BYTES)
}

fn node_bridge_script() -> &'static str {
    r#"
import fs from 'node:fs';
import readline from 'node:readline';

let moduleInstance = null;
let moduleConfig = null;

function createCtx(configObj, snapshotObj) {
  const actions = [];
  const noop = () => {};
  const snapshot = snapshotObj && typeof snapshotObj === 'object' ? snapshotObj : {};
  const snapshotItems = Array.isArray(snapshot.items) ? snapshot.items : [];
  const selectedIndex = Number.isInteger(snapshot.selected_index) ? snapshot.selected_index : 0;
  return {
    takeActions: () => actions.splice(0),
    query: () => typeof snapshot.query === 'string' ? snapshot.query : '',
    items: () => snapshotItems.slice(),
    selectedItem: () => snapshotItems[selectedIndex] || null,
    selectedIndex: () => selectedIndex,
    mode: () => typeof snapshot.mode === 'string' ? snapshot.mode : 'launcher',
    hasCapability: () => false,
    log: noop,
    toast: noop,
    setQuery: (text) => actions.push({
      type: 'SetQuery',
      data: { text: typeof text === 'string' ? text : String(text ?? '') }
    }),
    setSelection: noop,
    moveSelection: noop,
    submit: noop,
    close: noop,
    addItems: noop,
    replaceItems: (items) => actions.push({
      type: 'ReplaceItems',
      data: { items: Array.isArray(items) ? items : [] }
    }),
    registerCommand: noop,
    registerProvider: noop,
    setInputAccessory: (accessory) => {
      if (!accessory || typeof accessory !== 'object') return;
      actions.push({
        type: 'SetInputAccessory',
        data: {
          text: typeof accessory.text === 'string' ? accessory.text : '',
          kind: typeof accessory.kind === 'string' ? accessory.kind : null,
          priority: Number.isInteger(accessory.priority) ? accessory.priority : null
        }
      });
    },
    clearInputAccessory: () => actions.push({ type: 'ClearInputAccessory' }),
    moduleConfig: () => configObj
  };
}

async function handleMessage(raw) {
  const message = JSON.parse(raw);

  if (message.type === 'init') {
    moduleConfig = message.config_json ? JSON.parse(message.config_json) : null;
    const moduleDataUrl = 'data:text/javascript;base64,' + Buffer.from(message.entry_code, 'utf8').toString('base64');
    const loaded = await import(moduleDataUrl);
    const createModule = loaded.default;

    if (typeof createModule !== 'function') {
      return { ok: false, error: 'module default export must be a function' };
    }

    moduleInstance = createModule();
    if (!moduleInstance || typeof moduleInstance !== 'object') {
      return { ok: false, error: 'createModule must return an object' };
    }

    return { ok: true };
  }

  if (message.type === 'shutdown') {
    return { ok: true };
  }

  if (!moduleInstance) {
    return { ok: false, error: 'module instance not initialized' };
  }

  const ctx = createCtx(moduleConfig, message.snapshot);
  const okWithActions = (extra = {}) => ({ ok: true, actions: ctx.takeActions(), ...extra });

  switch (message.hook) {
    case 'onLoad':
      if (typeof moduleInstance.onLoad === 'function') await moduleInstance.onLoad(ctx);
      return okWithActions();
    case 'onUnload':
      if (typeof moduleInstance.onUnload === 'function') await moduleInstance.onUnload(ctx);
      return okWithActions();
    case 'onQueryChange':
      if (typeof moduleInstance.onQueryChange === 'function') await moduleInstance.onQueryChange(message.query || '', ctx);
      return okWithActions();
    case 'onKey':
      if (typeof moduleInstance.onKey === 'function') {
        const keyEvent = message.key_event && typeof message.key_event === 'object' ? message.key_event : {};
        await moduleInstance.onKey({
          key: typeof keyEvent.key === 'string' ? keyEvent.key : '',
          ctrl: Boolean(keyEvent.ctrl),
          alt: Boolean(keyEvent.alt),
          shift: Boolean(keyEvent.shift),
          meta: Boolean(keyEvent.meta)
        }, ctx);
      }
      return okWithActions();
    case 'provideItems':
      if (typeof moduleInstance.provideItems === 'function') {
        const items = await moduleInstance.provideItems(message.query || '', ctx);
        return okWithActions({ items: Array.isArray(items) ? items : [] });
      }
      return okWithActions({ items: [] });
    case 'decorateItems':
      if (typeof moduleInstance.decorateItems === 'function') {
        const items = await moduleInstance.decorateItems(Array.isArray(message.items) ? message.items : [], ctx);
        return okWithActions({ items: Array.isArray(items) ? items : [] });
      }
      return okWithActions({ items: Array.isArray(message.items) ? message.items : [] });
    case 'onCommand':
      if (typeof moduleInstance.onCommand === 'function') {
        await moduleInstance.onCommand(message.command || '', Array.isArray(message.args) ? message.args : [], ctx);
      }
      return okWithActions();
    default:
      return { ok: false, error: 'unknown hook' };
  }
}

const rl = readline.createInterface({ input: process.stdin, crlfDelay: Infinity });
rl.on('line', async (line) => {
  try {
    const response = await handleMessage(line);
    process.stdout.write(JSON.stringify(response) + '\n');
  } catch (error) {
    process.stdout.write(JSON.stringify({ ok: false, error: String(error) }) + '\n');
  }
});
"#
}

#[cfg(test)]
mod tests {
    use super::ipc::{HostRequest, HostRequestPayload, HostResponsePayload, IpcSnapshot};
    use super::{
        handle_request, read_max_ipc_payload_bytes, HostState, NodeRuntime, WorkerRequest,
        DEFAULT_MAX_IPC_PAYLOAD_BYTES,
    };
    use std::io::BufReader;
    use std::process::{Command, Stdio};

    fn empty_snapshot() -> IpcSnapshot {
        IpcSnapshot {
            query: String::new(),
            items: Vec::new(),
            selected_index: 0,
            mode: "launcher".to_string(),
        }
    }

    #[test]
    fn max_ipc_payload_env_uses_default_for_missing_invalid_or_zero_values() {
        std::env::remove_var("RMODULE_MAX_IPC_BYTES");
        assert_eq!(read_max_ipc_payload_bytes(), DEFAULT_MAX_IPC_PAYLOAD_BYTES);

        std::env::set_var("RMODULE_MAX_IPC_BYTES", "not-a-number");
        assert_eq!(read_max_ipc_payload_bytes(), DEFAULT_MAX_IPC_PAYLOAD_BYTES);

        std::env::set_var("RMODULE_MAX_IPC_BYTES", "0");
        assert_eq!(read_max_ipc_payload_bytes(), DEFAULT_MAX_IPC_PAYLOAD_BYTES);

        std::env::set_var("RMODULE_MAX_IPC_BYTES", "128");
        assert_eq!(read_max_ipc_payload_bytes(), 128);

        std::env::remove_var("RMODULE_MAX_IPC_BYTES");
    }

    #[test]
    fn unloaded_module_requests_return_recoverable_errors() {
        let mut state = HostState::default();
        let response = handle_request(
            HostRequest {
                id: 7,
                payload: HostRequestPayload::ProvideItems {
                    query: "q".to_string(),
                    snapshot: empty_snapshot(),
                },
            },
            &mut state,
        );

        assert_eq!(response.id, 7);
        match response.payload {
            HostResponsePayload::Error {
                message,
                recoverable,
            } => {
                assert_eq!(message, "module not loaded");
                assert!(recoverable);
            }
            other => panic!("expected error response, got {other:?}"),
        }
    }

    #[cfg(windows)]
    #[test]
    fn node_runtime_send_rejects_oversized_worker_response() {
        let mut child = Command::new("powershell.exe")
            .args([
                "-NoProfile",
                "-Command",
                "Write-Output 'xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx'; Start-Sleep -Seconds 5",
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn oversized response child");
        let stdin = child.stdin.take().expect("child stdin");
        let stdout = child.stdout.take().expect("child stdout");
        let mut runtime = NodeRuntime {
            child,
            stdin,
            stdout: BufReader::new(stdout),
        };

        let err = runtime
            .send(WorkerRequest::Shutdown, 24)
            .expect_err("oversized worker response should fail");

        assert!(err.contains("worker response exceeds max_ipc_payload_bytes"));
        let _ = runtime.child.kill();
        let _ = runtime.child.wait();
    }

    #[test]
    fn shutdown_clears_loaded_state_without_runtime() {
        let mut state = HostState::default();
        state.loaded = true;

        let response = handle_request(
            HostRequest {
                id: 3,
                payload: HostRequestPayload::Shutdown,
            },
            &mut state,
        );

        assert_eq!(response.id, 3);
        assert!(matches!(response.payload, HostResponsePayload::Ack));
        assert!(!state.loaded);
        assert!(state.runtime.is_none());
    }
}
