use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::thread;
use std::time::Duration;

use super::ipc::{
    HostRequest, HostRequestPayload, HostResponse, HostResponsePayload, IpcAction, IpcItem,
    IpcKeyEvent, IpcSnapshot, ModuleInitPayload,
};
use super::types::ModuleDescriptor;

#[derive(Debug)]
pub enum HostClientError {
    Io(String),
    Protocol(String),
    Timeout(String),
}

pub struct ExternalModuleHost {
    pub module_name: String,
    child: Child,
    stdin: ChildStdin,
    response_rx: Receiver<HostResponse>,
    next_id: u64,
    response_timeout_ms: u64,
    max_ipc_payload_bytes: usize,
}

impl ExternalModuleHost {
    pub fn start(
        descriptor: &ModuleDescriptor,
        response_timeout_ms: u64,
        max_ipc_payload_bytes: usize,
    ) -> Result<Self, HostClientError> {
        let host_bin = module_host_binary_path()?;

        let mut child = Command::new(host_bin)
            .env("RMODULE_MAX_IPC_BYTES", max_ipc_payload_bytes.to_string())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|err| HostClientError::Io(err.to_string()))?;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| HostClientError::Io("module-host stdin unavailable".to_string()))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| HostClientError::Io("module-host stdout unavailable".to_string()))?;

        let response_rx = spawn_response_reader(stdout, max_ipc_payload_bytes);

        let mut host = Self {
            module_name: descriptor.name.clone(),
            child,
            stdin,
            response_rx,
            next_id: 1,
            response_timeout_ms,
            max_ipc_payload_bytes,
        };

        let init = ModuleInitPayload {
            name: descriptor.name.clone(),
            version: descriptor.version.clone(),
            api_version: descriptor.api_version,
            capabilities: descriptor.capabilities.clone(),
            entry_code: descriptor.entry_code.clone(),
            config_json: descriptor.config_json.clone(),
        };

        match host.send_request(HostRequestPayload::Initialize(init))? {
            HostResponsePayload::Ack => {}
            HostResponsePayload::Error { message, .. } => {
                return Err(HostClientError::Protocol(format!(
                    "initialize failed: {message}"
                )))
            }
            other => {
                return Err(HostClientError::Protocol(format!(
                    "unexpected initialize response: {other:?}"
                )))
            }
        }

        match host.send_request(HostRequestPayload::OnLoad { snapshot: None })? {
            HostResponsePayload::Ack => {}
            HostResponsePayload::Error { message, .. } => {
                return Err(HostClientError::Protocol(format!(
                    "onLoad failed: {message}"
                )))
            }
            other => {
                return Err(HostClientError::Protocol(format!(
                    "unexpected onLoad response: {other:?}"
                )))
            }
        }

        Ok(host)
    }

    pub fn on_query_change(
        &mut self,
        query: &str,
        snapshot: IpcSnapshot,
    ) -> Result<Vec<IpcAction>, HostClientError> {
        actions_from_response(
            self.send_request(HostRequestPayload::OnQueryChange {
                query: query.to_string(),
                snapshot,
            })?,
            "OnQueryChange",
        )
    }

    pub fn on_key(
        &mut self,
        event: IpcKeyEvent,
        snapshot: IpcSnapshot,
    ) -> Result<Vec<IpcAction>, HostClientError> {
        actions_from_response(
            self.send_request(HostRequestPayload::OnKey { event, snapshot })?,
            "OnKey",
        )
    }

    pub fn provide_items(
        &mut self,
        query: &str,
        snapshot: IpcSnapshot,
    ) -> Result<Vec<IpcItem>, HostClientError> {
        match self.send_request(HostRequestPayload::ProvideItems {
            query: query.to_string(),
            snapshot,
        })? {
            HostResponsePayload::ProvideItemsResult { items } => Ok(items),
            HostResponsePayload::Error { message, .. } => Err(HostClientError::Protocol(message)),
            other => Err(HostClientError::Protocol(format!(
                "unexpected response for ProvideItems: {other:?}"
            ))),
        }
    }

    pub fn decorate_items(
        &mut self,
        items: Vec<IpcItem>,
        snapshot: IpcSnapshot,
    ) -> Result<Vec<IpcItem>, HostClientError> {
        match self.send_request(HostRequestPayload::DecorateItems { items, snapshot })? {
            HostResponsePayload::DecorateItemsResult { items } => Ok(items),
            HostResponsePayload::Error { message, .. } => Err(HostClientError::Protocol(message)),
            other => Err(HostClientError::Protocol(format!(
                "unexpected response for DecorateItems: {other:?}"
            ))),
        }
    }

    pub fn on_command(
        &mut self,
        command: &str,
        args: &[String],
        snapshot: IpcSnapshot,
    ) -> Result<Vec<IpcAction>, HostClientError> {
        actions_from_response(
            self.send_request(HostRequestPayload::OnCommand {
                command: command.to_string(),
                args: args.to_vec(),
                snapshot,
            })?,
            "OnCommand",
        )
    }

    pub fn shutdown(&mut self) {
        let _ = self.send_request(HostRequestPayload::OnUnload { snapshot: None });
        let _ = self.send_request(HostRequestPayload::Shutdown);
        self.force_kill();
    }

    fn send_request(
        &mut self,
        payload: HostRequestPayload,
    ) -> Result<HostResponsePayload, HostClientError> {
        let id = self.next_id;
        self.next_id = self.next_id.saturating_add(1);

        let request = HostRequest { id, payload };
        let encoded = serde_json::to_string(&request)
            .map_err(|err| HostClientError::Protocol(err.to_string()))?;
        if encoded.len() > self.max_ipc_payload_bytes {
            return Err(HostClientError::Protocol(format!(
                "request exceeds max_ipc_payload_bytes ({} > {})",
                encoded.len(),
                self.max_ipc_payload_bytes
            )));
        }

        self.stdin
            .write_all(encoded.as_bytes())
            .and_then(|_| self.stdin.write_all(b"\n"))
            .and_then(|_| self.stdin.flush())
            .map_err(|err| HostClientError::Io(err.to_string()))?;

        let timeout = Duration::from_millis(self.response_timeout_ms);
        let response = match self.response_rx.recv_timeout(timeout) {
            Ok(response) => response,
            Err(RecvTimeoutError::Timeout) => {
                self.force_kill();
                return Err(HostClientError::Timeout(format!(
                    "module-host timed out after {}ms for module '{}'",
                    self.response_timeout_ms, self.module_name
                )));
            }
            Err(RecvTimeoutError::Disconnected) => {
                self.force_kill();
                return Err(HostClientError::Protocol(
                    "module-host response channel disconnected".to_string(),
                ));
            }
        };

        if response.id != id {
            return Err(HostClientError::Protocol(format!(
                "mismatched response id: expected {id}, got {}",
                response.id
            )));
        }

        Ok(response.payload)
    }

    fn force_kill(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn actions_from_response(
    payload: HostResponsePayload,
    operation: &str,
) -> Result<Vec<IpcAction>, HostClientError> {
    match payload {
        HostResponsePayload::Ack => Ok(Vec::new()),
        HostResponsePayload::Actions { actions } => Ok(actions),
        HostResponsePayload::Error { message, .. } => Err(HostClientError::Protocol(message)),
        other => Err(HostClientError::Protocol(format!(
            "unexpected response for {operation}: {other:?}"
        ))),
    }
}

fn spawn_response_reader(
    stdout: ChildStdout,
    max_ipc_payload_bytes: usize,
) -> Receiver<HostResponse> {
    spawn_response_reader_from_reader(BufReader::new(stdout), max_ipc_payload_bytes)
}

fn spawn_response_reader_from_reader<R>(
    mut reader: R,
    max_ipc_payload_bytes: usize,
) -> Receiver<HostResponse>
where
    R: BufRead + Send + 'static,
{
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || loop {
        let mut line = String::new();
        let read = match reader.read_line(&mut line) {
            Ok(read) => read,
            Err(_) => break,
        };

        if read == 0 {
            break;
        }

        if line.len() > max_ipc_payload_bytes {
            break;
        }

        if let Ok(response) = serde_json::from_str::<HostResponse>(line.trim()) {
            if tx.send(response).is_err() {
                break;
            }
        }
    });

    rx
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;
    use std::sync::mpsc::RecvTimeoutError;
    use std::time::Duration;

    use std::process::{Command, Stdio};
    use std::sync::mpsc;

    use super::{spawn_response_reader_from_reader, ExternalModuleHost, HostClientError};
    use crate::modules::ipc::{HostRequestPayload, IpcSnapshot};

    #[cfg(windows)]
    fn test_host(response_timeout_ms: u64, max_ipc_payload_bytes: usize) -> ExternalModuleHost {
        let mut child = Command::new("powershell.exe")
            .args(["-NoProfile", "-Command", "Start-Sleep -Seconds 60"])
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn test child");
        let stdin = child.stdin.take().expect("test child stdin");
        let (tx, response_rx) = mpsc::channel();
        std::mem::forget(tx);

        ExternalModuleHost {
            module_name: "timeout-test".to_string(),
            child,
            stdin,
            response_rx,
            next_id: 1,
            response_timeout_ms,
            max_ipc_payload_bytes,
        }
    }

    fn snapshot() -> IpcSnapshot {
        IpcSnapshot {
            query: String::new(),
            items: Vec::new(),
            selected_index: 0,
            mode: "launcher".to_string(),
        }
    }

    #[test]
    fn response_reader_ignores_invalid_json_and_continues() {
        let input = b"not-json\n{\"id\":2,\"payload\":{\"type\":\"Ack\"}}\n".to_vec();
        let rx = spawn_response_reader_from_reader(Cursor::new(input), 1024);

        let response = rx
            .recv_timeout(Duration::from_millis(100))
            .expect("valid response should arrive");
        assert_eq!(response.id, 2);
    }

    #[test]
    fn response_reader_stops_on_payload_limit_exceeded() {
        let input = b"01234567890123456789\n{\"id\":2,\"payload\":{\"type\":\"Ack\"}}\n".to_vec();
        let rx = spawn_response_reader_from_reader(Cursor::new(input), 8);

        let err = rx
            .recv_timeout(Duration::from_millis(100))
            .expect_err("oversized payload should close reader");
        assert_eq!(err, RecvTimeoutError::Disconnected);
    }

    #[cfg(windows)]
    #[test]
    fn send_request_times_out_and_kills_unresponsive_host() {
        let mut host = test_host(10, 1024);

        let err = host
            .send_request(HostRequestPayload::Ping)
            .expect_err("unresponsive host should time out");

        assert!(matches!(err, HostClientError::Timeout(_)));
        assert!(host.child.try_wait().expect("child status").is_some());
    }

    #[cfg(windows)]
    #[test]
    fn send_request_rejects_oversized_request_before_waiting_for_response() {
        let mut host = test_host(1_000, 16);

        let err = host
            .send_request(HostRequestPayload::OnQueryChange {
                query: "long-query".repeat(16),
                snapshot: snapshot(),
            })
            .expect_err("oversized request should fail");

        assert!(
            matches!(err, HostClientError::Protocol(message) if message.contains("request exceeds max_ipc_payload_bytes"))
        );
        host.force_kill();
    }
}

fn module_host_binary_path() -> Result<PathBuf, HostClientError> {
    let current_exe =
        std::env::current_exe().map_err(|err| HostClientError::Io(err.to_string()))?;
    let exe_name = current_exe
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| HostClientError::Io("invalid current executable name".to_string()))?;

    let host_name = if exe_name.ends_with(".exe") {
        "rmenu-module-host.exe"
    } else {
        "rmenu-module-host"
    };

    let sibling = current_exe.with_file_name(host_name);
    if sibling.exists() {
        return Ok(sibling);
    }

    let parent_sibling = current_exe
        .parent()
        .and_then(|parent| parent.parent())
        .map(|parent| parent.join(host_name));
    if let Some(path) = parent_sibling {
        if path.exists() {
            return Ok(path);
        }
    }

    Ok(sibling)
}
