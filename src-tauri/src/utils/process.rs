use std::collections::{HashMap, HashSet};
use std::io;
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;
#[cfg(target_os = "windows")]
use std::path::Component;
use std::path::{Path, PathBuf};
use std::process::{Child as StdChild, Command as StdCommand, Output, Stdio};
use std::sync::{Mutex, OnceLock};
use tokio::process::{Child as AsyncChild, Command as AsyncCommand};

#[cfg(target_os = "windows")]
use windows::core::PWSTR;
#[cfg(target_os = "windows")]
use windows::Win32::Foundation::{CloseHandle, HANDLE, INVALID_HANDLE_VALUE};
#[cfg(target_os = "windows")]
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
};
#[cfg(target_os = "windows")]
use windows::Win32::System::Threading::{
    OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_FORMAT, PROCESS_QUERY_LIMITED_INFORMATION,
};

pub const PROCESS_KIND_ADB_CLIENT: &str = "adb-client";
pub const PROCESS_KIND_ADB_SERVER: &str = "adb-server";
pub const PROCESS_KIND_FASTBOOT: &str = "fastboot";
pub const PROCESS_KIND_SCRCPY: &str = "scrcpy";
pub const PROCESS_KIND_ARIA2C: &str = "aria2c";
pub const PROCESS_KIND_LINK_DUMPER: &str = "link-dumper";
pub const PROCESS_KIND_LOCAL_PROGRAM: &str = "local-program";

static PROCESS_TRACKER: OnceLock<ProcessTracker> = OnceLock::new();

pub fn init_process_tracker(adb_path: &Path) {
    let _ = PROCESS_TRACKER.get_or_init(|| ProcessTracker {
        adb_path: adb_path.to_path_buf(),
        tracked_pids: Mutex::new(HashMap::new()),
        adb_client_snapshots: Mutex::new(HashMap::new()),
    });
}

fn tracker() -> Option<&'static ProcessTracker> {
    PROCESS_TRACKER.get()
}

struct ProcessTracker {
    adb_path: PathBuf,
    tracked_pids: Mutex<HashMap<&'static str, HashSet<u32>>>,
    adb_client_snapshots: Mutex<HashMap<u32, HashSet<u32>>>,
}

impl ProcessTracker {
    fn register_pid(&self, kind: &'static str, pid: u32) {
        let mut tracked = self.tracked_pids.lock().unwrap();
        tracked.entry(kind).or_default().insert(pid);
    }

    fn unregister_pid(&self, kind: &'static str, pid: u32) {
        let mut tracked = self.tracked_pids.lock().unwrap();
        if let Some(pids) = tracked.get_mut(kind) {
            pids.remove(&pid);
            if pids.is_empty() {
                tracked.remove(kind);
            }
        }
    }

    fn take_kind_pids(&self, kind: &'static str) -> Vec<u32> {
        let mut tracked = self.tracked_pids.lock().unwrap();
        tracked
            .remove(kind)
            .map(|pids| pids.into_iter().collect())
            .unwrap_or_default()
    }

    fn take_all_pids(&self) -> Vec<u32> {
        let mut tracked = self.tracked_pids.lock().unwrap();
        let mut all = Vec::new();
        for (_, pids) in tracked.drain() {
            all.extend(pids);
        }
        all
    }

    fn track_new_adb_server_for_client(&self, pid: u32) {
        let before = self.adb_client_snapshots.lock().unwrap().remove(&pid);
        let Some(before) = before else {
            return;
        };

        let current = list_process_ids_by_exact_path(&self.adb_path);
        for spawned_pid in current.difference(&before) {
            self.register_pid(PROCESS_KIND_ADB_SERVER, *spawned_pid);
        }
    }
}

pub fn spawn_tracked_std_command(
    command: &mut StdCommand,
    kind: &'static str,
) -> io::Result<StdChild> {
    let adb_snapshot = if kind == PROCESS_KIND_ADB_CLIENT {
        tracker().map(|process_tracker| list_process_ids_by_exact_path(&process_tracker.adb_path))
    } else {
        None
    };
    let child = command.spawn()?;
    if kind == PROCESS_KIND_ADB_CLIENT {
        if let (Some(process_tracker), Some(snapshot)) = (tracker(), adb_snapshot) {
            process_tracker
                .adb_client_snapshots
                .lock()
                .unwrap()
                .insert(child.id(), snapshot);
        }
    }
    register_child_pid(kind, child.id());
    Ok(child)
}

pub fn output_tracked_std_command(
    command: &mut StdCommand,
    kind: &'static str,
) -> io::Result<Output> {
    command.stdin(Stdio::null());
    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());
    let child = spawn_tracked_std_command(command, kind)?;
    let pid = child.id();
    let output = child.wait_with_output();
    unregister_child_pid(kind, pid);
    if kind == PROCESS_KIND_ADB_CLIENT {
        track_new_adb_server_for_client(pid);
    }
    output
}

pub fn wait_tracked_std_child(
    child: &mut StdChild,
    kind: &'static str,
) -> io::Result<std::process::ExitStatus> {
    let pid = child.id();
    let status = child.wait();
    unregister_child_pid(kind, pid);
    if kind == PROCESS_KIND_ADB_CLIENT {
        track_new_adb_server_for_client(pid);
    }
    status
}

pub fn spawn_tracked_async_command(
    command: &mut AsyncCommand,
    kind: &'static str,
) -> io::Result<AsyncChild> {
    let adb_snapshot = if kind == PROCESS_KIND_ADB_CLIENT {
        tracker().map(|process_tracker| list_process_ids_by_exact_path(&process_tracker.adb_path))
    } else {
        None
    };
    let child = command.spawn()?;
    if let Some(pid) = child.id() {
        if kind == PROCESS_KIND_ADB_CLIENT {
            if let (Some(process_tracker), Some(snapshot)) = (tracker(), adb_snapshot) {
                process_tracker
                    .adb_client_snapshots
                    .lock()
                    .unwrap()
                    .insert(pid, snapshot);
            }
        }
        register_child_pid(kind, pid);
    }
    Ok(child)
}

pub async fn output_tracked_async_command(
    command: &mut AsyncCommand,
    kind: &'static str,
) -> io::Result<Output> {
    command.stdin(Stdio::null());
    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());
    let child = spawn_tracked_async_command(command, kind)?;
    let pid = child.id();
    let output = child.wait_with_output().await;
    if let Some(pid) = pid {
        unregister_child_pid(kind, pid);
        if kind == PROCESS_KIND_ADB_CLIENT {
            track_new_adb_server_for_client(pid);
        }
    }
    output
}

pub async fn wait_tracked_async_child(
    child: &mut AsyncChild,
    kind: &'static str,
) -> io::Result<std::process::ExitStatus> {
    let pid = child.id();
    let status = child.wait().await;
    if let Some(pid) = pid {
        unregister_child_pid(kind, pid);
        if kind == PROCESS_KIND_ADB_CLIENT {
            track_new_adb_server_for_client(pid);
        }
    }
    status
}

pub fn refresh_adb_server_tracking() {
    if let Some(process_tracker) = tracker() {
        let current = list_process_ids_by_exact_path(&process_tracker.adb_path);
        let tracked = process_tracker.take_kind_pids(PROCESS_KIND_ADB_SERVER);
        let alive_tracked: Vec<u32> = tracked
            .into_iter()
            .filter(|pid| current.contains(pid))
            .collect();
        for pid in alive_tracked {
            process_tracker.register_pid(PROCESS_KIND_ADB_SERVER, pid);
        }
    }
}

pub fn terminate_tracked_kind(kind: &'static str) {
    if let Some(process_tracker) = tracker() {
        terminate_process_ids(process_tracker.take_kind_pids(kind));
    }
}

pub fn terminate_all_tracked_processes() {
    if let Some(process_tracker) = tracker() {
        terminate_process_ids(process_tracker.take_all_pids());
    }
}

pub fn terminate_processes_in_dir(root: &Path) {
    terminate_process_ids(list_process_ids_in_dir(root));
}

fn register_child_pid(kind: &'static str, pid: u32) {
    if let Some(process_tracker) = tracker() {
        process_tracker.register_pid(kind, pid);
    }
}

fn unregister_child_pid(kind: &'static str, pid: u32) {
    if let Some(process_tracker) = tracker() {
        process_tracker.unregister_pid(kind, pid);
    }
}

fn track_new_adb_server_for_client(pid: u32) {
    if let Some(process_tracker) = tracker() {
        process_tracker.track_new_adb_server_for_client(pid);
    }
}

fn terminate_process_ids(pids: Vec<u32>) {
    for pid in pids {
        let _ = StdCommand::new("taskkill")
            .args(["/F", "/PID", &pid.to_string(), "/T"])
            .creation_flags(0x08000000)
            .status();
    }
}

#[cfg(target_os = "windows")]
fn list_process_ids_in_dir(root: &Path) -> Vec<u32> {
    let normalized_root = normalize_windows_dir_path(root);
    if normalized_root.is_empty() {
        return Vec::new();
    }

    let snapshot = match unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) } {
        Ok(handle) => handle,
        Err(_) => return Vec::new(),
    };

    if snapshot == INVALID_HANDLE_VALUE {
        return Vec::new();
    }

    let mut matched = Vec::new();
    let mut entry = PROCESSENTRY32W {
        dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
        ..Default::default()
    };

    let current_pid = std::process::id();
    let mut has_entry = unsafe { Process32FirstW(snapshot, &mut entry).is_ok() };
    while has_entry {
        let pid = entry.th32ProcessID;
        if pid != 0 && pid != current_pid {
            if let Some(executable_path) = query_process_image_path(pid) {
                let normalized_process_path = normalize_windows_path(&executable_path);
                if normalized_process_path.starts_with(&normalized_root) {
                    matched.push(pid);
                }
            }
        }
        has_entry = unsafe { Process32NextW(snapshot, &mut entry).is_ok() };
    }

    let _ = unsafe { CloseHandle(snapshot) };
    matched
}

#[cfg(not(target_os = "windows"))]
fn list_process_ids_in_dir(_root: &Path) -> Vec<u32> {
    Vec::new()
}

#[cfg(target_os = "windows")]
fn list_process_ids_by_exact_path(target_path: &Path) -> HashSet<u32> {
    let normalized_target = normalize_windows_path(target_path);
    let snapshot = match unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) } {
        Ok(handle) => handle,
        Err(_) => return HashSet::new(),
    };

    if snapshot == INVALID_HANDLE_VALUE {
        return HashSet::new();
    }

    let mut matched = HashSet::new();
    let mut entry = PROCESSENTRY32W {
        dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
        ..Default::default()
    };

    let mut has_entry = unsafe { Process32FirstW(snapshot, &mut entry).is_ok() };
    while has_entry {
        let pid = entry.th32ProcessID;
        if pid != 0 {
            if let Some(executable_path) = query_process_image_path(pid) {
                if normalize_windows_path(&executable_path) == normalized_target {
                    matched.insert(pid);
                }
            }
        }
        has_entry = unsafe { Process32NextW(snapshot, &mut entry).is_ok() };
    }

    let _ = unsafe { CloseHandle(snapshot) };
    matched
}

#[cfg(not(target_os = "windows"))]
fn list_process_ids_by_exact_path(_target_path: &Path) -> HashSet<u32> {
    HashSet::new()
}

#[cfg(target_os = "windows")]
fn query_process_image_path(pid: u32) -> Option<PathBuf> {
    let handle = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) }.ok()?;
    let result = query_process_image_path_from_handle(handle);
    let _ = unsafe { CloseHandle(handle) };
    result
}

#[cfg(target_os = "windows")]
fn query_process_image_path_from_handle(handle: HANDLE) -> Option<PathBuf> {
    let mut buffer = vec![0u16; 32768];
    let mut size = buffer.len() as u32;
    let ok = unsafe {
        QueryFullProcessImageNameW(
            handle,
            PROCESS_NAME_FORMAT(0),
            PWSTR(buffer.as_mut_ptr()),
            &mut size,
        )
        .is_ok()
    };
    if !ok || size == 0 {
        return None;
    }
    Some(PathBuf::from(String::from_utf16_lossy(
        &buffer[..size as usize],
    )))
}

#[cfg(target_os = "windows")]
fn normalize_windows_path(path: &Path) -> String {
    path.to_string_lossy()
        .replace('/', "\\")
        .to_ascii_lowercase()
}

#[cfg(target_os = "windows")]
fn normalize_windows_dir_path(path: &Path) -> String {
    let normalized = normalize_windows_path(path);
    if normalized.is_empty() {
        return normalized;
    }

    let has_root = path
        .components()
        .any(|component| matches!(component, Component::Prefix(_) | Component::RootDir));
    if !has_root {
        return String::new();
    }

    normalized.trim_end_matches('\\').to_string() + "\\"
}
