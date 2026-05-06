use std::collections::BTreeMap;
use std::env;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::os::windows::process::CommandExt;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};

use crate::modules::types::{ModuleDescriptor, ModuleSourceType};

const CREATE_NO_WINDOW: u32 = 0x0800_0000;

#[derive(Debug)]
struct ActiveResidentHelper {
    module_name: String,
    command_path: PathBuf,
    signature: String,
    child: Child,
}

#[derive(Debug, Default)]
pub struct ResidentHelperManager {
    helpers: BTreeMap<String, ActiveResidentHelper>,
}

impl ResidentHelperManager {
    #[allow(dead_code)]
    pub fn start_from_descriptors(descriptors: &[ModuleDescriptor]) -> Self {
        let mut manager = Self::default();
        manager.sync(descriptors);
        manager
    }

    pub fn sync(&mut self, descriptors: &[ModuleDescriptor]) {
        let mut desired = BTreeMap::new();
        for descriptor in descriptors {
            let Some(resident) = descriptor.resident.as_ref() else {
                continue;
            };
            if !descriptor.enabled || !resident.enabled || !resident.autostart {
                continue;
            }
            desired.insert(descriptor.name.clone(), descriptor);
        }

        let active_names = self.helpers.keys().cloned().collect::<Vec<_>>();
        for name in active_names {
            if !desired.contains_key(&name) {
                self.stop_helper(&name);
            }
        }

        for (name, descriptor) in desired {
            let signature = helper_signature(descriptor);
            if self
                .helpers
                .get(&name)
                .map(|helper| helper.signature == signature)
                .unwrap_or(false)
            {
                continue;
            }
            if self.helpers.contains_key(&name) {
                self.stop_helper(&name);
            }
            self.start_helper(descriptor, signature);
        }
    }

    pub fn stop_all(&mut self) {
        let names = self.helpers.keys().cloned().collect::<Vec<_>>();
        for name in names {
            self.stop_helper(&name);
        }
    }

    #[cfg(test)]
    fn active_pid(&self, name: &str) -> Option<u32> {
        self.helpers.get(name).map(|helper| helper.child.id())
    }

    fn start_helper(&mut self, descriptor: &ModuleDescriptor, signature: String) {
        let Some(resident) = descriptor.resident.as_ref() else {
            return;
        };
        let Some(module_dir) = module_dir(descriptor) else {
            log_line(&format!(
                "resident helper skipped module={} reason=missing-module-dir",
                descriptor.name
            ));
            return;
        };
        let command_path = module_dir.join(&resident.command);
        if !command_path.exists() {
            log_line(&format!(
                "resident helper skipped module={} command={} reason=missing-command",
                descriptor.name,
                command_path.display()
            ));
            return;
        }

        let state_dir = module_state_dir(descriptor);
        if let Some(state_dir) = &state_dir {
            if let Err(err) = fs::create_dir_all(state_dir) {
                log_line(&format!(
                    "resident helper state dir failed module={} path={} error={err}",
                    descriptor.name,
                    state_dir.display()
                ));
            }
        }

        let mut command = Command::new(&command_path);
        command
            .current_dir(&module_dir)
            .arg("--module-name")
            .arg(&descriptor.name)
            .arg("--module-dir")
            .arg(&module_dir)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .stdin(Stdio::null())
            .creation_flags(CREATE_NO_WINDOW);

        if let Some(state_dir) = &state_dir {
            command.arg("--state-dir").arg(state_dir);
        }
        let config_path = module_dir.join("config.json");
        if config_path.exists() {
            command.arg("--config-path").arg(config_path);
        }
        command.args(&resident.args);

        match command.spawn() {
            Ok(child) => {
                log_line(&format!(
                    "resident helper started module={} pid={} command={}",
                    descriptor.name,
                    child.id(),
                    command_path.display()
                ));
                self.helpers.insert(
                    descriptor.name.clone(),
                    ActiveResidentHelper {
                        module_name: descriptor.name.clone(),
                        command_path,
                        signature,
                        child,
                    },
                );
            }
            Err(err) => {
                log_line(&format!(
                    "resident helper start failed module={} command={} error={err}",
                    descriptor.name,
                    command_path.display()
                ));
            }
        }
    }

    fn stop_helper(&mut self, name: &str) {
        let Some(mut helper) = self.helpers.remove(name) else {
            return;
        };
        match helper.child.try_wait() {
            Ok(Some(status)) => {
                log_line(&format!(
                    "resident helper already exited module={} command={} status={status}",
                    helper.module_name,
                    helper.command_path.display()
                ));
            }
            Ok(None) => {
                if let Err(err) = helper.child.kill() {
                    log_line(&format!(
                        "resident helper kill failed module={} command={} error={err}",
                        helper.module_name,
                        helper.command_path.display()
                    ));
                    return;
                }
                match helper.child.wait() {
                    Ok(status) => log_line(&format!(
                        "resident helper stopped module={} command={} status={status}",
                        helper.module_name,
                        helper.command_path.display()
                    )),
                    Err(err) => log_line(&format!(
                        "resident helper wait failed module={} command={} error={err}",
                        helper.module_name,
                        helper.command_path.display()
                    )),
                }
            }
            Err(err) => {
                log_line(&format!(
                    "resident helper status failed module={} command={} error={err}",
                    helper.module_name,
                    helper.command_path.display()
                ));
            }
        }
    }
}

impl Drop for ResidentHelperManager {
    fn drop(&mut self) {
        self.stop_all();
    }
}

fn helper_signature(descriptor: &ModuleDescriptor) -> String {
    let Some(resident) = descriptor.resident.as_ref() else {
        return String::new();
    };
    format!(
        "{}\0{}\0{}\0{}\0{}\0{}",
        descriptor.source_path,
        descriptor.version,
        resident.command,
        resident.args.join("\0"),
        resident.autostart,
        resident.shutdown
    )
}

fn module_dir(descriptor: &ModuleDescriptor) -> Option<PathBuf> {
    match descriptor.source_type {
        ModuleSourceType::Directory => Some(PathBuf::from(&descriptor.source_path)),
        ModuleSourceType::Rmod => None,
    }
}

fn module_state_dir(descriptor: &ModuleDescriptor) -> Option<PathBuf> {
    let module_dir = module_dir(descriptor)?;
    let modules_dir = module_dir.parent()?;
    let data_dir = modules_dir.parent()?;
    Some(
        data_dir
            .join("state")
            .join("modules")
            .join(&descriptor.name),
    )
}

fn log_path() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(env::temp_dir)
        .join("rmenu")
        .join("rmenu-daemon.log")
}

fn log_line(message: &str) {
    let path = log_path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
        let _ = writeln!(file, "{message}");
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::process::Command;
    use std::thread;
    use std::time::{Duration, Instant};

    use super::{module_dir, module_state_dir, ResidentHelperManager};
    use crate::modules::types::{
        ModuleDescriptor, ModuleSourceType, ResidentHelperDescriptor, MODULE_API_VERSION,
    };

    fn descriptor_at(source_path: String, args: Vec<String>) -> ModuleDescriptor {
        ModuleDescriptor {
            source_type: ModuleSourceType::Directory,
            source_path,
            name: "demo".to_string(),
            version: "1.0.0".to_string(),
            api_version: MODULE_API_VERSION,
            kind: "script".to_string(),
            capabilities: Vec::new(),
            enabled: true,
            priority: 0,
            description: None,
            author: None,
            homepage: None,
            entry_code: String::new(),
            config_json: None,
            readme: None,
            resident: Some(ResidentHelperDescriptor {
                enabled: true,
                command: "bin/helper.exe".to_string(),
                args,
                autostart: true,
                shutdown: "kill".to_string(),
            }),
        }
    }

    fn descriptor() -> ModuleDescriptor {
        descriptor_at("C:\\rMenuData\\modules\\demo".to_string(), Vec::new())
    }

    #[test]
    fn resident_helper_paths_use_module_and_state_dirs() {
        let descriptor = descriptor();

        assert_eq!(
            module_dir(&descriptor)
                .expect("module dir")
                .to_string_lossy(),
            "C:\\rMenuData\\modules\\demo"
        );
        assert_eq!(
            module_state_dir(&descriptor)
                .expect("state dir")
                .to_string_lossy(),
            "C:\\rMenuData\\state\\modules\\demo"
        );
    }

    #[test]
    fn sync_starts_restarts_and_stops_helpers() {
        let root = temp_root("sync");
        let module_dir = root.join("data").join("modules").join("demo");
        let bin_dir = module_dir.join("bin");
        fs::create_dir_all(&bin_dir).expect("create helper bin dir");
        let helper_src = root.join("helper.rs");
        let helper_exe = bin_dir.join("helper.exe");
        fs::write(
            &helper_src,
            r#"
use std::{thread, time::Duration};
fn main() {
    loop { thread::sleep(Duration::from_millis(500)); }
}
"#,
        )
        .expect("write helper source");
        let status = Command::new("rustc")
            .arg(&helper_src)
            .arg("-O")
            .arg("-o")
            .arg(&helper_exe)
            .status()
            .expect("run rustc for fake helper");
        assert!(status.success(), "rustc fake helper failed: {status}");

        let descriptor_v1 = descriptor_at(module_dir.to_string_lossy().to_string(), Vec::new());
        let descriptor_v2 = descriptor_at(
            module_dir.to_string_lossy().to_string(),
            vec!["--changed".to_string()],
        );

        let mut manager = ResidentHelperManager::default();
        manager.sync(&[descriptor_v1]);
        let first_pid = manager.active_pid("demo").expect("helper started");
        assert!(wait_for_process(first_pid, true));

        manager.sync(&[descriptor_v2]);
        let second_pid = manager.active_pid("demo").expect("helper restarted");
        assert_ne!(first_pid, second_pid);
        assert!(wait_for_process(first_pid, false));
        assert!(wait_for_process(second_pid, true));

        manager.sync(&[]);
        assert_eq!(manager.active_pid("demo"), None);
        assert!(wait_for_process(second_pid, false));

        let _ = fs::remove_dir_all(root);
    }

    fn temp_root(name: &str) -> std::path::PathBuf {
        let mut dir = std::env::temp_dir();
        dir.push(format!(
            "rmenu-resident-helper-test-{}-{}",
            name,
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("create temp root");
        dir
    }

    fn wait_for_process(pid: u32, should_exist: bool) -> bool {
        let started = Instant::now();
        while started.elapsed() < Duration::from_secs(5) {
            if process_exists(pid) == should_exist {
                return true;
            }
            thread::sleep(Duration::from_millis(100));
        }
        process_exists(pid) == should_exist
    }

    fn process_exists(pid: u32) -> bool {
        Command::new("powershell")
            .args([
                "-NoProfile",
                "-Command",
                &format!(
                    "if (Get-Process -Id {pid} -ErrorAction SilentlyContinue) {{ exit 0 }} else {{ exit 1 }}"
                ),
            ])
            .status()
            .map(|status| status.success())
            .unwrap_or(false)
    }
}
