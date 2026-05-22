use std::collections::{HashMap, HashSet};
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command as StdCommand, Output};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::Instant;
use tokio::process::Command as AsyncCommand;
use tokio::sync::{Mutex as AsyncMutex, MutexGuard as AsyncMutexGuard};
use tokio::time::{sleep, timeout, Duration};

use crate::utils::process::{
    output_tracked_async_command, output_tracked_std_command, PROCESS_KIND_ADB_CLIENT,
    PROCESS_KIND_FASTBOOT,
};

const FASTBOOT_DEVICE_SCAN_TIMEOUT_MS: u64 = 1200;
const FASTBOOT_MODE_DETECT_TIMEOUT_MS: u64 = 600;
const FASTBOOT_DEVICE_STICKY_MS: u64 = 1200;
const FASTBOOT_ACTIVE_DEVICE_STICKY_MS: u64 = 300000;
const FASTBOOT_MODE_CACHE_MS: u64 = 3000;

#[derive(Clone, Debug)]
pub struct DetectedDevice {
    pub serial: String,
    pub state: String,
    pub source: String,
}

#[derive(Clone, Debug)]
pub struct DeviceScanReport {
    pub raw_output: String,
    pub devices: Vec<DetectedDevice>,
}

#[derive(Clone, Debug)]
pub struct ConnectedDeviceScanSnapshot {
    pub devices: Vec<DetectedDevice>,
    pub adb_report: DeviceScanReport,
    pub fastboot_report: DeviceScanReport,
    pub duration_ms: u64,
}

#[derive(Clone, Debug)]
struct CachedFastbootDevice {
    state: String,
    last_seen: Instant,
}

#[derive(Default)]
struct FastbootCache {
    devices: HashMap<String, CachedFastbootDevice>,
}

static FASTBOOT_COMMAND_LOCK: OnceLock<AsyncMutex<()>> = OnceLock::new();
static FASTBOOT_CACHE: OnceLock<Mutex<FastbootCache>> = OnceLock::new();
static FASTBOOT_ACTIVE_COMMANDS: AtomicUsize = AtomicUsize::new(0);

struct FastbootActivityGuard;

pub struct FastbootCommandGuard {
    _lock: AsyncMutexGuard<'static, ()>,
    _activity: FastbootActivityGuard,
}

fn fastboot_command_lock() -> &'static AsyncMutex<()> {
    FASTBOOT_COMMAND_LOCK.get_or_init(|| AsyncMutex::new(()))
}

fn fastboot_cache() -> &'static Mutex<FastbootCache> {
    FASTBOOT_CACHE.get_or_init(|| Mutex::new(FastbootCache::default()))
}

fn is_fastboot_command_active() -> bool {
    FASTBOOT_ACTIVE_COMMANDS.load(Ordering::SeqCst) > 0
}

impl FastbootActivityGuard {
    fn new() -> Self {
        FASTBOOT_ACTIVE_COMMANDS.fetch_add(1, Ordering::SeqCst);
        Self
    }
}

impl Drop for FastbootActivityGuard {
    fn drop(&mut self) {
        FASTBOOT_ACTIVE_COMMANDS.fetch_sub(1, Ordering::SeqCst);
    }
}

pub async fn acquire_fastboot_command_guard() -> FastbootCommandGuard {
    FastbootCommandGuard {
        _lock: fastboot_command_lock().lock().await,
        _activity: FastbootActivityGuard::new(),
    }
}

/// 应用关键路径缓存结构，启动时注入 Tauri State
#[derive(Clone)]
pub struct AppPaths {
    pub adb: PathBuf,
    pub fastboot: PathBuf,
    pub scrcpy: PathBuf,
}

#[cfg(dev)]
fn project_bin_dir() -> PathBuf {
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest_dir.parent().unwrap_or(manifest_dir).join("bin")
}

#[cfg(not(dev))]
fn packaged_bin_dir(app: &tauri::AppHandle) -> PathBuf {
    use tauri::Manager;
    app.path().resource_dir().unwrap_or_default().join("bin")
}

pub fn get_bin_root_dir(_app: &tauri::AppHandle) -> PathBuf {
    #[cfg(dev)]
    {
        project_bin_dir()
    }
    #[cfg(not(dev))]
    {
        packaged_bin_dir(_app)
    }
}

/// 获取 adb.exe 的路径
pub fn get_adb_path(_app: &tauri::AppHandle) -> PathBuf {
    get_bin_root_dir(_app)
        .join("platform-tools")
        .join("adb.exe")
}

/// 获取 fastboot.exe 的路径
pub fn get_fastboot_path(_app: &tauri::AppHandle) -> PathBuf {
    get_bin_root_dir(_app)
        .join("platform-tools")
        .join("fastboot.exe")
}

/// 获取 scrcpy.exe 的路径
pub fn get_scrcpy_path(_app: &tauri::AppHandle) -> PathBuf {
    get_bin_root_dir(_app)
        .join("scrcpy-core")
        .join("scrcpy.exe")
}

/// 获取 bin/app-tools 目录下指定工具文件的路径
pub fn get_tool_path(_app: &tauri::AppHandle, filename: &str) -> PathBuf {
    get_bin_root_dir(_app).join("app-tools").join(filename)
}

/// 获取 bin/link-dumper/link-dumper.exe 的路径
pub fn get_link_dumper_path(_app: &tauri::AppHandle) -> PathBuf {
    get_bin_root_dir(_app)
        .join("link-dumper")
        .join("link-dumper.exe")
}

/// 创建隐藏黑框的同步 StdCommand
pub fn create_hidden_command(program: &PathBuf) -> StdCommand {
    let mut cmd = StdCommand::new(program);
    cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
    cmd
}

/// 创建隐藏黑框的异步 AsyncCommand
pub fn create_hidden_async_command(program: &PathBuf) -> AsyncCommand {
    let mut cmd = AsyncCommand::new(program);
    cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
    cmd
}

/// 执行 adb 命令 (同步)，返回 stdout 字符串
pub fn adb_run(adb: &PathBuf, args: &[&str]) -> Result<String, String> {
    let mut command = create_hidden_command(adb);
    let output = output_tracked_std_command(command.args(args), PROCESS_KIND_ADB_CLIENT)
        .map_err(|e| format!("执行 adb 失败: {}", e))?;
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

    if output.status.success() {
        Ok(stdout)
    } else if !stderr.is_empty() && !stdout.is_empty() {
        Err(format!("{}\n{}", stderr, stdout))
    } else if !stderr.is_empty() {
        Err(stderr)
    } else if !stdout.is_empty() {
        Err(stdout)
    } else {
        Err(format!("adb 退出码: {:?}", output.status.code()))
    }
}

/// 执行 adb 命令 (异步)，返回 stdout 字符串
pub async fn adb_run_async(adb: &PathBuf, args: &[&str]) -> Result<String, String> {
    let mut command = create_hidden_async_command(adb);
    let output = output_tracked_async_command(command.args(args), PROCESS_KIND_ADB_CLIENT)
        .await
        .map_err(|e| format!("执行 adb 失败: {}", e))?;
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    if output.status.success() {
        Ok(stdout)
    } else if !stderr.is_empty() && !stdout.is_empty() {
        Err(format!("{}\n{}", stderr, stdout))
    } else if !stderr.is_empty() {
        Err(stderr)
    } else if !stdout.is_empty() {
        Err(stdout)
    } else {
        Err(format!("adb 退出码: {:?}", output.status.code()))
    }
}

pub async fn adb_run_async_combined(adb: &PathBuf, args: &[&str]) -> Result<String, String> {
    let mut command = create_hidden_async_command(adb);
    let output = output_tracked_async_command(command.args(args), PROCESS_KIND_ADB_CLIENT)
        .await
        .map_err(|e| format!("执行 adb 失败: {}", e))?;
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    if !output.status.success() {
        if !stderr.is_empty() && !stdout.is_empty() {
            Err(format!("{}\n{}", stderr, stdout))
        } else if !stderr.is_empty() {
            Err(stderr)
        } else if !stdout.is_empty() {
            Err(stdout)
        } else {
            Err(format!("adb 退出码: {:?}", output.status.code()))
        }
    } else if !stderr.is_empty() {
        if stdout.is_empty() {
            Ok(stderr)
        } else {
            Ok(format!("{}\n{}", stdout, stderr))
        }
    } else {
        Ok(stdout)
    }
}

/// 一次性导出所有 getprop 并解析为 HashMap (同步)
pub fn dump_props(adb: &PathBuf) -> HashMap<String, String> {
    let output = adb_run(adb, &["shell", "getprop"]).unwrap_or_default();
    let mut map = HashMap::new();
    for line in output.lines() {
        let line = line.trim();
        if !line.starts_with('[') {
            continue;
        }
        if let Some(sep) = line.find("]: [") {
            let key = line[1..sep].to_string();
            let rest = &line[sep + 4..];
            let value = if rest.ends_with(']') {
                rest[..rest.len() - 1].to_string()
            } else {
                rest.to_string()
            };
            map.insert(key, value);
        }
    }
    map
}

/// 一次性导出所有 getprop 并解析为 HashMap (异步)
pub async fn dump_props_async(adb: &PathBuf) -> HashMap<String, String> {
    let output = adb_run_async(adb, &["shell", "getprop"])
        .await
        .unwrap_or_default();
    let mut map = HashMap::new();
    for line in output.lines() {
        let line = line.trim();
        if !line.starts_with('[') {
            continue;
        }
        if let Some(sep) = line.find("]: [") {
            let key = line[1..sep].to_string();
            let rest = &line[sep + 4..];
            let value = if rest.ends_with(']') {
                rest[..rest.len() - 1].to_string()
            } else {
                rest.to_string()
            };
            map.insert(key, value);
        }
    }
    map
}

pub fn prop<'a>(map: &'a HashMap<String, String>, key: &str) -> &'a str {
    map.get(key).map(|s| s.as_str()).unwrap_or("--")
}

pub fn prop_or(map: &HashMap<String, String>, keys: &[&str]) -> String {
    for key in keys {
        let v = map.get(*key).map(|s| s.as_str()).unwrap_or("");
        if !v.is_empty() && v != "--" {
            return v.to_string();
        }
    }
    "--".to_string()
}

/// 执行 adb 命令 (异步)，支持指定设备 serial（传 Some("") 或 None 则退化为旧行为）
pub async fn adb_run_async_with_serial(
    adb: &PathBuf,
    serial: Option<&str>,
    args: &[&str],
) -> Result<String, String> {
    if let Some(s) = serial.filter(|s| !s.is_empty()) {
        let mut full_args = vec!["-s", s];
        full_args.extend_from_slice(args);
        adb_run_async(adb, &full_args).await
    } else {
        adb_run_async(adb, args).await
    }
}

pub async fn adb_run_async_with_serial_combined(
    adb: &PathBuf,
    serial: Option<&str>,
    args: &[&str],
) -> Result<String, String> {
    if let Some(s) = serial.filter(|s| !s.is_empty()) {
        let mut full_args = vec!["-s", s];
        full_args.extend_from_slice(args);
        adb_run_async_combined(adb, &full_args).await
    } else {
        adb_run_async_combined(adb, args).await
    }
}

pub async fn adb_pull_to_local_file(
    adb: &PathBuf,
    serial: Option<&str>,
    remote_path: &str,
    local_path: &Path,
) -> Result<String, String> {
    let local_dir = local_path
        .parent()
        .ok_or_else(|| "本地保存路径缺少目录".to_string())?;
    let local_file_name = local_path
        .file_name()
        .and_then(|value| value.to_str())
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| "本地保存路径缺少文件名".to_string())?;

    let mut command = create_hidden_async_command(adb);
    command.current_dir(local_dir);

    if let Some(serial) = serial.filter(|value| !value.is_empty()) {
        command.arg("-s").arg(serial);
    }

    let output = output_tracked_async_command(
        command.arg("pull").arg(remote_path).arg(local_file_name),
        PROCESS_KIND_ADB_CLIENT,
    )
    .await
    .map_err(|e| format!("执行 adb pull 失败: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

    if output.status.success() {
        Ok(if stdout.is_empty() { stderr } else { stdout })
    } else if !stderr.is_empty() && !stdout.is_empty() {
        Err(format!("{}\n{}", stderr, stdout))
    } else if !stderr.is_empty() {
        Err(stderr)
    } else if !stdout.is_empty() {
        Err(stdout)
    } else {
        Err(format!("adb 退出码: {:?}", output.status.code()))
    }
}

/// 一次性导出所有 getprop 并解析为 HashMap (异步)，支持指定 serial
pub async fn dump_props_async_with_serial(
    adb: &PathBuf,
    serial: Option<&str>,
) -> HashMap<String, String> {
    let output = adb_run_async_with_serial(adb, serial, &["shell", "getprop"])
        .await
        .unwrap_or_default();
    let mut map = HashMap::new();
    for line in output.lines() {
        let line = line.trim();
        if !line.starts_with('[') {
            continue;
        }
        if let Some(sep) = line.find("]: [") {
            let key = line[1..sep].to_string();
            let rest = &line[sep + 4..];
            let value = if rest.ends_with(']') {
                rest[..rest.len() - 1].to_string()
            } else {
                rest.to_string()
            };
            map.insert(key, value);
        }
    }
    map
}

fn build_combined_output(stdout: &str, stderr: &str) -> String {
    match (stdout.is_empty(), stderr.is_empty()) {
        (true, true) => String::new(),
        (false, true) => stdout.to_string(),
        (true, false) => stderr.to_string(),
        (false, false) => format!("{}\n{}", stdout, stderr),
    }
}

fn build_command_failure_message(tool: &str, exit_code: Option<i32>, raw_output: &str) -> String {
    if !raw_output.is_empty() {
        return raw_output.to_string();
    }

    match exit_code {
        Some(code) => format!("{} 退出码: {}", tool, code),
        None => format!("{} 进程已结束，但未返回退出码", tool),
    }
}

fn output_to_raw_text(output: Output, tool: &str) -> String {
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let raw_output = build_combined_output(&stdout, &stderr);
    if !raw_output.is_empty() || output.status.success() {
        return raw_output;
    }

    build_command_failure_message(tool, output.status.code(), &raw_output)
}

fn spawn_error_to_raw_text(tool: &str, error: String) -> String {
    format!("执行 {} 失败: {}", tool, error)
}

fn build_scan_report(raw_output: String, devices: Vec<DetectedDevice>) -> DeviceScanReport {
    DeviceScanReport { raw_output, devices }
}

async fn capture_adb_command(adb: &PathBuf, args: &[&str]) -> String {
    let mut command = create_hidden_async_command(adb);
    match output_tracked_async_command(command.args(args), PROCESS_KIND_ADB_CLIENT).await {
        Ok(output) => output_to_raw_text(output, "adb"),
        Err(error) => spawn_error_to_raw_text("adb", error.to_string()),
    }
}

async fn capture_fastboot_command(fastboot: &PathBuf, args: &[&str]) -> String {
    let _guard = acquire_fastboot_command_guard().await;
    let mut command = create_hidden_async_command(fastboot);
    match output_tracked_async_command(command.args(args), PROCESS_KIND_FASTBOOT).await {
        Ok(output) => output_to_raw_text(output, "fastboot"),
        Err(error) => spawn_error_to_raw_text("fastboot", error.to_string()),
    }
}

fn is_supported_adb_state(state: &str) -> bool {
    matches!(state, "device" | "recovery" | "sideload" | "unauthorized")
}

fn is_supported_fastboot_state(state: &str) -> bool {
    matches!(state, "fastboot" | "fastbootd")
}

fn prune_fastboot_cache(cache: &mut FastbootCache, ttl_ms: u64) {
    let now = Instant::now();
    cache
        .devices
        .retain(|_, device| now.duration_since(device.last_seen).as_millis() <= ttl_ms as u128);
}

fn update_fastboot_cache(serial: &str, state: &str) {
    let mut cache = fastboot_cache().lock().unwrap();
    cache.devices.insert(
        serial.to_string(),
        CachedFastbootDevice {
            state: state.to_string(),
            last_seen: Instant::now(),
        },
    );
    prune_fastboot_cache(&mut cache, FASTBOOT_ACTIVE_DEVICE_STICKY_MS);
}

fn get_cached_fastboot_mode(serial: &str) -> Option<String> {
    let mut cache = fastboot_cache().lock().unwrap();
    prune_fastboot_cache(&mut cache, FASTBOOT_ACTIVE_DEVICE_STICKY_MS);
    cache
        .devices
        .get(serial)
        .filter(|device| device.last_seen.elapsed().as_millis() <= FASTBOOT_MODE_CACHE_MS as u128)
        .map(|device| device.state.clone())
}

fn append_cached_fastboot_devices(
    devices: &mut Vec<DetectedDevice>,
    seen_serials: &HashSet<String>,
) {
    let ttl_ms = if is_fastboot_command_active() {
        FASTBOOT_ACTIVE_DEVICE_STICKY_MS
    } else {
        FASTBOOT_DEVICE_STICKY_MS
    };

    let mut cache = fastboot_cache().lock().unwrap();
    prune_fastboot_cache(&mut cache, ttl_ms);

    for (serial, device) in &cache.devices {
        if seen_serials.contains(serial) {
            continue;
        }

        devices.push(DetectedDevice {
            serial: serial.clone(),
            state: device.state.clone(),
            source: "fastboot".to_string(),
        });
    }
}

fn pick_device<'a>(
    devices: &'a [DetectedDevice],
    serial: Option<&str>,
) -> Option<&'a DetectedDevice> {
    if let Some(preferred) = serial.filter(|value| !value.is_empty()) {
        if let Some(device) = devices.iter().find(|device| device.serial == preferred) {
            return Some(device);
        }

        if devices.len() == 1 {
            return devices.first();
        }

        return None;
    }

    devices.first()
}

fn parse_adb_devices_output(output: &str) -> Vec<DetectedDevice> {
    output
        .lines()
        .filter_map(|raw_line| {
            let line = raw_line.trim();
            if line.is_empty()
                || line.starts_with("List of devices attached")
                || line.starts_with("* ")
            {
                return None;
            }

            let mut parts = line.split_whitespace();
            let serial = parts.next()?.trim();
            let state = parts.next()?.trim();

            if serial.is_empty() || !is_supported_adb_state(state) {
                return None;
            }

            Some(DetectedDevice {
                serial: serial.to_string(),
                state: state.to_string(),
                source: "adb".to_string(),
            })
        })
        .collect()
}

fn parse_fastboot_devices_output(output: &str) -> Vec<(String, String)> {
    output
        .lines()
        .filter_map(|raw_line| {
            let line = raw_line.trim();
            if line.is_empty() {
                return None;
            }

            let mut parts = line.split_whitespace();
            let serial = parts.next()?.trim();
            let state = parts.next()?.trim();
            if serial.is_empty() || !is_supported_fastboot_state(state) {
                return None;
            }

            Some((serial.to_string(), state.to_string()))
        })
        .collect()
}

fn should_retry_adb_scan(raw_output: &str, devices: &[DetectedDevice]) -> bool {
    if !devices.is_empty() {
        return false;
    }

    raw_output.is_empty() || raw_output.to_ascii_lowercase().contains("daemon")
}

async fn scan_adb_devices_with_report(adb: &PathBuf) -> DeviceScanReport {
    let primary = capture_adb_command(adb, &["devices"]).await;
    let primary_devices = parse_adb_devices_output(&primary);

    if !should_retry_adb_scan(&primary, &primary_devices) {
        return build_scan_report(primary, primary_devices);
    }

    sleep(Duration::from_millis(250)).await;
    let retry = capture_adb_command(adb, &["devices"]).await;
    let retry_devices = parse_adb_devices_output(&retry);
    build_scan_report(retry, retry_devices)
}

pub async fn list_adb_devices(adb: &PathBuf) -> Vec<DetectedDevice> {
    scan_adb_devices_with_report(adb).await.devices
}

async fn resolve_fastboot_mode(fastboot: &PathBuf, serial: &str, listed_state: &str) -> String {
    if listed_state == "fastbootd" {
        update_fastboot_cache(serial, "fastbootd");
        return "fastbootd".to_string();
    }

    if let Some(cached_state) = get_cached_fastboot_mode(serial) {
        return cached_state;
    }

    let output = timeout(
        Duration::from_millis(FASTBOOT_MODE_DETECT_TIMEOUT_MS),
        fastboot_run_async_with_serial(fastboot, Some(serial), &["getvar", "is-userspace"]),
    )
    .await
    .unwrap_or(Ok(String::new()))
    .unwrap_or_default();

    let detected_state = if output.to_ascii_lowercase().contains("yes") {
        "fastbootd".to_string()
    } else {
        listed_state.to_string()
    };

    update_fastboot_cache(serial, &detected_state);
    detected_state
}

async fn scan_fastboot_devices_with_report(fastboot: &PathBuf) -> DeviceScanReport {
    let primary = match timeout(
        Duration::from_millis(FASTBOOT_DEVICE_SCAN_TIMEOUT_MS),
        capture_fastboot_command(fastboot, &["devices"]),
    )
    .await
    {
        Ok(capture) => capture,
        Err(_) => {
            return DeviceScanReport {
                raw_output: format!(
                    
                    "fastboot devices 执行超时（>{}ms）",
                    FASTBOOT_DEVICE_SCAN_TIMEOUT_MS
                ),
                devices: Vec::new(),
            };
        }
    };

    let mut devices = Vec::new();
    let mut seen_serials = HashSet::new();
    for (serial, listed_state) in parse_fastboot_devices_output(&primary) {
        if !seen_serials.insert(serial.clone()) {
            continue;
        }

        let state = resolve_fastboot_mode(fastboot, &serial, &listed_state).await;
        devices.push(DetectedDevice {
            serial,
            state,
            source: "fastboot".to_string(),
        });
    }

    append_cached_fastboot_devices(&mut devices, &seen_serials);
    build_scan_report(primary, devices)
}

pub async fn list_fastboot_devices(fastboot: &PathBuf) -> Vec<DetectedDevice> {
    scan_fastboot_devices_with_report(fastboot).await.devices
}

fn merge_detected_devices(
    adb_devices: Vec<DetectedDevice>,
    fastboot_devices: Vec<DetectedDevice>,
) -> Vec<DetectedDevice> {
    let mut devices = adb_devices;
    let mut seen_serials: HashSet<String> =
        devices.iter().map(|device| device.serial.clone()).collect();

    for device in fastboot_devices {
        if seen_serials.insert(device.serial.clone()) {
            devices.push(device);
        }
    }

    devices
}

pub async fn scan_connected_devices_with_report(
    adb: &PathBuf,
    fastboot: &PathBuf,
) -> ConnectedDeviceScanSnapshot {
    let started_at = Instant::now();
    let (adb_report, fastboot_report) = tokio::join!(
        scan_adb_devices_with_report(adb),
        scan_fastboot_devices_with_report(fastboot)
    );
    let devices = merge_detected_devices(
        adb_report.devices.clone(),
        fastboot_report.devices.clone(),
    );

    ConnectedDeviceScanSnapshot {
        devices,
        adb_report,
        fastboot_report,
        duration_ms: started_at.elapsed().as_millis() as u64,
    }
}

pub async fn list_connected_devices(adb: &PathBuf, fastboot: &PathBuf) -> Vec<DetectedDevice> {
    scan_connected_devices_with_report(adb, fastboot).await.devices
}

pub async fn detect_device_state(
    adb: &PathBuf,
    fastboot: &PathBuf,
    serial: Option<&str>,
) -> Result<String, String> {
    let adb_devices = list_adb_devices(adb).await;
    if let Some(device) = pick_device(&adb_devices, serial) {
        return Ok(device.state.clone());
    }

    let fastboot_devices = list_fastboot_devices(fastboot).await;
    if let Some(device) = pick_device(&fastboot_devices, serial) {
        return Ok(device.state.clone());
    }

    Ok("none".to_string())
}

/// 执行 fastboot 命令 (异步)，支持指定设备 serial
pub async fn fastboot_run_async_with_serial(
    fastboot: &PathBuf,
    serial: Option<&str>,
    args: &[&str],
) -> Result<String, String> {
    if let Some(s) = serial.filter(|s| !s.is_empty()) {
        let mut full_args = vec!["-s", s];
        full_args.extend_from_slice(args);
        fastboot_run_async(fastboot, &full_args).await
    } else {
        fastboot_run_async(fastboot, args).await
    }
}

/// 一次性获取所有 fastboot 变量并解析 (异步)，支持指定 serial
pub async fn dump_fastboot_vars_async_with_serial(
    fastboot: &PathBuf,
    serial: Option<&str>,
) -> HashMap<String, String> {
    let output = fastboot_run_async_with_serial(fastboot, serial, &["getvar", "all"])
        .await
        .unwrap_or_default();
    let mut map = HashMap::new();
    for line in output.lines() {
        let line = line.trim();
        let clean_line = if let Some(stripped) = line.strip_prefix("(bootloader) ") {
            stripped.trim()
        } else {
            line
        };
        if let Some((k, v)) = clean_line.split_once(':') {
            let key = k.trim().to_string();
            let val = v.trim().to_string();
            map.insert(key, val);
        }
    }
    map
}

/// 执行 fastboot 命令 (异步)
pub async fn fastboot_run_async(fastboot: &PathBuf, args: &[&str]) -> Result<String, String> {
    let _guard = acquire_fastboot_command_guard().await;
    let mut command = create_hidden_async_command(fastboot);
    let output = output_tracked_async_command(command.args(args), PROCESS_KIND_FASTBOOT)
        .await
        .map_err(|e| format!("执行 fastboot 失败: {}", e))?;
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    if !stderr.is_empty() {
        Ok(format!("{}\n{}", stdout, stderr))
    } else {
        Ok(stdout)
    }
}

/// 一次性获取所有 fastboot 变量并解析 (异步)
pub async fn dump_fastboot_vars_async(fastboot: &PathBuf) -> HashMap<String, String> {
    let output = fastboot_run_async(fastboot, &["getvar", "all"])
        .await
        .unwrap_or_default();
    let mut map = HashMap::new();
    for line in output.lines() {
        let line = line.trim();
        let clean_line = if let Some(stripped) = line.strip_prefix("(bootloader) ") {
            stripped.trim()
        } else {
            line
        };
        if let Some((k, v)) = clean_line.split_once(':') {
            let key = k.trim().to_string();
            let val = v.trim().to_string();
            map.insert(key.clone(), val);

            // 针对 partition-size:name 或 partition-type:name 这种带冒号的 key 进行特殊处理
            // 使得后续可以通过 key.starts_with("partition-size") 并再次 split 获取 name
            // 虽然 dump_fastboot_vars_async 主要是存原始 map，但为了兼容多级冒号，
            // 我们可以考虑在 get_partition_info 里直接处理原始字符串，
            // 或者在这里保证 key 的完整性。
        }
    }
    map
}
