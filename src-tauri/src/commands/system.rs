use crate::adb::core::AppPaths;
use crate::utils::process::{
    spawn_tracked_std_command, terminate_processes_in_dir, terminate_tracked_kind,
    wait_tracked_std_child,
    PROCESS_KIND_SCRCPY,
};
use serde::Serialize;
use std::fs;
use std::os::windows::process::CommandExt; // 引入 Windows 扩展
use std::path::{Path, PathBuf};
use std::process::Command as StdCommand;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager, State};

#[derive(Default)]
pub struct ExitCleanupState {
    pub skip_cleanup_on_exit: AtomicBool,
}

#[derive(Debug, Serialize)]
pub struct CleanupEntry {
    pub path: String,
    pub status: String,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct CleanupSummary {
    pub success_count: usize,
    pub skipped_count: usize,
    pub failed_count: usize,
    pub entries: Vec<CleanupEntry>,
}

#[derive(Debug, Clone)]
enum CleanupTarget {
    Path(PathBuf),
    #[allow(dead_code)]
    DirContents(PathBuf),
    DirContentsExcept(PathBuf, Vec<&'static str>),
    TempPrefix(String),
}

const CMD_MENU_SCRIPT_NAME: &str = "ranran_toolkit_cmd_menu.bat";

// ==============================
// Command: 打开 bin 目录下的 CMD 菜单
// ==============================
#[tauri::command]
pub fn open_platform_tools_cmd(paths: State<'_, AppPaths>) -> Result<(), String> {
    let bin_dir = paths
        .adb
        .parent()
        .and_then(|p| p.parent())
        .ok_or_else(|| "无法获取 bin 目录".to_string())?;
    let menu_script = bin_dir.join(CMD_MENU_SCRIPT_NAME);

    if !menu_script.is_file() {
        return Err(format!("未找到 CMD 菜单脚本: {}", menu_script.display()));
    }

    StdCommand::new("cmd")
        .args(["/K", CMD_MENU_SCRIPT_NAME])
        .current_dir(bin_dir)
        .spawn()
        .map_err(|e| format!("打开 CMD 菜单失败: {}", e))?;
    Ok(())
}

// ==============================
// Command: 打开 Windows 设备管理器
// ==============================
#[tauri::command]
pub fn open_device_manager() -> Result<(), String> {
    let mut cmd = StdCommand::new("cmd");
    cmd.args(&["/C", "start devmgmt.msc"])
        .creation_flags(0x08000000) // CREATE_NO_WINDOW，仅用于中转 start 命令
        .spawn()
        .map_err(|e| format!("打开设备管理器失败: {}", e))?;
    Ok(())
}

// ==============================
// Command: 启动 Scrcpy 投屏
// ==============================
#[tauri::command]
pub fn start_scrcpy(
    app: AppHandle,
    paths: State<'_, AppPaths>,
    serial: Option<String>,
    args: Vec<String>,
) -> Result<(), String> {
    let mut cmd = StdCommand::new(&paths.scrcpy);

    cmd.env("ADB", &paths.adb);

    // 当指定了 serial 时，将 -s <serial> 注入到 scrcpy 参数中
    if let Some(ref s) = serial {
        if !s.is_empty() {
            cmd.args(["-s", s.as_str()]);
        }
    }

    let mut child = spawn_tracked_std_command(
        cmd.args(&args).creation_flags(0x08000000),
        PROCESS_KIND_SCRCPY,
    )
    .map_err(|e| format!("启动 Scrcpy 失败: {}", e))?;

    std::thread::spawn(move || {
        let _ = wait_tracked_std_child(&mut child, PROCESS_KIND_SCRCPY);
        let _ = app.emit("scrcpy-exited", ());
    });

    Ok(())
}

// ==============================
// Command: 停止 Scrcpy 投屏
// ==============================
#[tauri::command]
pub fn stop_scrcpy() -> Result<(), String> {
    terminate_tracked_kind(PROCESS_KIND_SCRCPY);
    Ok(())
}


fn open_folder_in_explorer(path: &Path, folder_label: &str) -> Result<(), String> {
    if !path.is_dir() {
        return Err(format!("{}不存在: {:?}", folder_label, path));
    }

    let mut cmd = StdCommand::new("explorer");
    cmd.arg(path)
        .creation_flags(0x08000000) // CREATE_NO_WINDOW
        .spawn()
        .map_err(|e| format!("打开{}失败: {}", folder_label, e))?;

    Ok(())
}

// ==============================
// Command: 打开驱动文件夹
// ==============================
#[tauri::command]
pub fn open_driver_folder(paths: State<'_, AppPaths>) -> Result<(), String> {
    let bin_dir = paths
        .adb
        .parent()
        .and_then(|p| p.parent())
        .ok_or_else(|| "无法获取 bin 目录".to_string())?;

    let driver_path = bin_dir.join("usb-driver");
    open_folder_in_explorer(&driver_path, "驱动文件夹")
}

// ==============================
// Command: 打开工具资源包文件夹
// ==============================
// Command: 打开工具资源包文件夹
#[tauri::command]
pub fn open_boot_patch_folder(paths: State<'_, AppPaths>) -> Result<(), String> {
    let bin_dir = paths
        .adb
        .parent()
        .and_then(|p| p.parent())
        .ok_or_else(|| "无法获取 bin 目录".to_string())?;

    let boot_patch_path = bin_dir.join("boot-patch");
    open_folder_in_explorer(&boot_patch_path, "Boot 修补资源文件夹")
}

#[tauri::command]
pub fn open_tool_dependency_folder(paths: State<'_, AppPaths>) -> Result<(), String> {
    let dependency_path = paths
        .adb
        .parent()
        .and_then(|p| p.parent())
        .ok_or_else(|| "无法获取工具依赖目录".to_string())?;

    open_folder_in_explorer(dependency_path, "资源包文件夹")
}

#[tauri::command]
pub fn open_folder_path(path: String) -> Result<(), String> {
    let folder_path = PathBuf::from(path.trim());
    open_folder_in_explorer(&folder_path, "文件夹")
}

fn remove_file_or_dir(path: &Path) -> Result<(), String> {
    if path.is_dir() {
        fs::remove_dir_all(path)
    } else {
        fs::remove_file(path)
    }
    .map_err(|e| format!("删除失败: {}", e))
}

fn remove_file_or_dir_with_retry(path: &Path) -> Result<(), String> {
    const REMOVE_RETRY_TIMES: usize = 30;
    const REMOVE_RETRY_DELAY_MS: u64 = 400;

    for attempt in 0..REMOVE_RETRY_TIMES {
        if !path.exists() {
            return Ok(());
        }

        match remove_file_or_dir(path) {
            Ok(_) => return Ok(()),
            Err(error) => {
                if attempt + 1 == REMOVE_RETRY_TIMES {
                    return Err(error);
                }
                std::thread::sleep(Duration::from_millis(REMOVE_RETRY_DELAY_MS));
            }
        }
    }

    Ok(())
}

fn push_cleanup_result(entries: &mut Vec<CleanupEntry>, path: PathBuf) {
    let path_text = path.display().to_string();
    if !path.exists() {
        entries.push(CleanupEntry {
            path: path_text,
            status: "skipped".to_string(),
            message: "路径不存在，已跳过".to_string(),
        });
        return;
    }

    match remove_file_or_dir_with_retry(&path) {
        Ok(_) => entries.push(CleanupEntry {
            path: path_text,
            status: "success".to_string(),
            message: "已清理".to_string(),
        }),
        Err(error) => entries.push(CleanupEntry {
            path: path_text,
            status: "failed".to_string(),
            message: error,
        }),
    }
}

fn push_cleanup_dir_contents(
    entries: &mut Vec<CleanupEntry>,
    dir: PathBuf,
    excluded_names: &[&str],
) {
    let dir_text = dir.display().to_string();
    if !dir.exists() {
        entries.push(CleanupEntry {
            path: dir_text,
            status: "skipped".to_string(),
            message: "目录不存在，已跳过".to_string(),
        });
        return;
    }

    let read_dir = match fs::read_dir(&dir) {
        Ok(read_dir) => read_dir,
        Err(error) => {
            entries.push(CleanupEntry {
                path: dir_text,
                status: "failed".to_string(),
                message: format!("读取目录失败: {}", error),
            });
            return;
        }
    };

    let mut removed = 0usize;
    let mut skipped = 0usize;
    let mut failed = Vec::new();
    for entry in read_dir {
        match entry {
            Ok(entry) => {
                let path = entry.path();
                let should_skip = path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .map(|name| excluded_names.iter().any(|excluded| excluded == &name))
                    .unwrap_or(false);
                if should_skip {
                    skipped += 1;
                    continue;
                }
                match remove_file_or_dir_with_retry(&path) {
                    Ok(_) => removed += 1,
                    Err(error) => failed.push(format!("{} ({})", path.display(), error)),
                }
            }
            Err(error) => failed.push(format!("读取目录项失败: {}", error)),
        }
    }

    if failed.is_empty() {
        entries.push(CleanupEntry {
            path: dir_text,
            status: "success".to_string(),
            message: if skipped > 0 {
                format!("已清理 {} 个项目，跳过 {} 个排除项", removed, skipped)
            } else {
                format!("已清理 {} 个项目", removed)
            },
        });
    } else {
        entries.push(CleanupEntry {
            path: dir_text,
            status: "failed".to_string(),
            message: failed.join("; "),
        });
    }
}

fn push_temp_prefix_cleanup(entries: &mut Vec<CleanupEntry>, prefix: &str) {
    let temp_dir = std::env::temp_dir();
    let read_dir = match fs::read_dir(&temp_dir) {
        Ok(read_dir) => read_dir,
        Err(error) => {
            entries.push(CleanupEntry {
                path: temp_dir.display().to_string(),
                status: "failed".to_string(),
                message: format!("读取系统临时目录失败: {}", error),
            });
            return;
        }
    };

    let mut matched = 0usize;
    for entry in read_dir.flatten() {
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if name.starts_with(prefix) {
            matched += 1;
            push_cleanup_result(entries, path);
        }
    }

    if matched == 0 {
        entries.push(CleanupEntry {
            path: temp_dir.join(format!("{}*", prefix)).display().to_string(),
            status: "skipped".to_string(),
            message: "未发现匹配的临时目录".to_string(),
        });
    }
}

// ==============================
// Command: 清理本工具缓存与临时资源
// ==============================
fn collect_pending_cleanup_targets(app: &AppHandle) -> Vec<CleanupTarget> {
    let resolver = app.path();
    let mut targets = Vec::new();

    if let Ok(local_data_dir) = resolver.app_local_data_dir() {
        targets.push(CleanupTarget::Path(local_data_dir.join("runtime")));
        targets.push(CleanupTarget::Path(local_data_dir.join("boot-patch-temp")));
        targets.push(CleanupTarget::Path(local_data_dir.join("link-dumper-temp")));
    }

    targets.push(CleanupTarget::Path(
        std::env::temp_dir().join("ranran-boot-patch-payload"),
    ));
    targets.push(CleanupTarget::TempPrefix("apk_extract_".to_string()));
    targets
}

fn collect_exit_cleanup_targets(app: &AppHandle) -> Vec<CleanupTarget> {
    let resolver = app.path();
    let mut targets = Vec::new();

    if let Ok(cache_dir) = resolver.app_cache_dir() {
        targets.push(CleanupTarget::DirContentsExcept(
            cache_dir,
            vec!["EBWebView"],
        ));
    }

    if let Ok(log_dir) = resolver.app_log_dir() {
        targets.push(CleanupTarget::DirContents(log_dir));
    }

    targets
}

fn execute_cleanup_targets(targets: Vec<CleanupTarget>) -> CleanupSummary {
    let mut entries = Vec::new();
    for target in targets {
        match target {
            CleanupTarget::Path(path) => push_cleanup_result(&mut entries, path),
            CleanupTarget::DirContents(path) => {
                push_cleanup_dir_contents(&mut entries, path, &[])
            }
            CleanupTarget::DirContentsExcept(path, excluded_names) => {
                push_cleanup_dir_contents(&mut entries, path, &excluded_names)
            }
            CleanupTarget::TempPrefix(prefix) => push_temp_prefix_cleanup(&mut entries, &prefix),
        }
    }

    let success_count = entries
        .iter()
        .filter(|entry| entry.status == "success")
        .count();
    let skipped_count = entries
        .iter()
        .filter(|entry| entry.status == "skipped")
        .count();
    let failed_count = entries
        .iter()
        .filter(|entry| entry.status == "failed")
        .count();

    CleanupSummary {
        success_count,
        skipped_count,
        failed_count,
        entries,
    }
}

fn execute_exit_cache_log_cleanup(app: &AppHandle) -> CleanupSummary {
    execute_cleanup_targets(collect_exit_cleanup_targets(app))
}

fn merge_cleanup_summaries(summaries: Vec<CleanupSummary>) -> CleanupSummary {
    let mut success_count = 0;
    let mut skipped_count = 0;
    let mut failed_count = 0;
    let mut entries = Vec::new();

    for summary in summaries {
        success_count += summary.success_count;
        skipped_count += summary.skipped_count;
        failed_count += summary.failed_count;
        entries.extend(summary.entries);
    }

    CleanupSummary {
        success_count,
        skipped_count,
        failed_count,
        entries,
    }
}

fn build_cleanup_failure_message(summary: &CleanupSummary) -> String {
    let failed_paths = summary
        .entries
        .iter()
        .filter(|entry| entry.status == "failed")
        .take(3)
        .map(|entry| entry.path.clone())
        .collect::<Vec<_>>();

    if failed_paths.is_empty() {
        return "工具缓存清理未完成，请关闭占用相关文件的外部程序后重试".to_string();
    }

    format!(
        "工具缓存清理未完成，仍有 {} 项删除失败：{}",
        summary.failed_count,
        failed_paths.join("；")
    )
}

pub fn force_stop_runtime_processes(app: &AppHandle) {
    let Ok(local_data_dir) = app.path().app_local_data_dir() else {
        return;
    };

    let runtime_root = local_data_dir.join("runtime");
    terminate_processes_in_dir(&runtime_root);
    std::thread::sleep(Duration::from_millis(600));
}

pub fn force_stop_runtime_processes_by_paths(paths: &AppPaths) {
    let Some(runtime_root) = paths
        .adb
        .parent()
        .and_then(|platform_tools_dir| platform_tools_dir.parent())
        .and_then(|bin_dir| bin_dir.parent())
    else {
        return;
    };

    terminate_processes_in_dir(runtime_root);
    std::thread::sleep(Duration::from_millis(600));
}

#[tauri::command]
pub async fn clear_tool_cache(
    app: AppHandle,
    paths: State<'_, AppPaths>,
    exit_cleanup_state: State<'_, ExitCleanupState>,
    manager: State<'_, Arc<crate::commands::downloader::DownloadManager>>,
) -> Result<(), String> {
    cleanup_processes(paths.inner());
    crate::commands::downloader::cleanup_on_exit(&manager);

    let cleanup_summary = merge_cleanup_summaries(vec![
        execute_exit_cache_log_cleanup(&app),
        execute_cleanup_targets(collect_pending_cleanup_targets(&app)),
    ]);

    if cleanup_summary.failed_count > 0 {
        exit_cleanup_state
            .skip_cleanup_on_exit
            .store(false, Ordering::SeqCst);
        return Err(build_cleanup_failure_message(&cleanup_summary));
    }

    exit_cleanup_state
        .skip_cleanup_on_exit
        .store(true, Ordering::SeqCst);

    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(Duration::from_millis(150)).await;
        app.exit(0);
    });

    Ok(())
}

#[tauri::command]
pub async fn restart_app(
    app: AppHandle,
    exit_cleanup_state: State<'_, ExitCleanupState>,
) -> Result<(), String> {
    exit_cleanup_state
        .skip_cleanup_on_exit
        .store(true, Ordering::SeqCst);

    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(Duration::from_millis(80)).await;
        app.request_restart();
    });

    Ok(())
}
// ==============================
// 导出给 lib.rs 使用：清理所有相关进程
// ==============================
pub fn cleanup_processes(paths: &AppPaths) {
    // 先刷新一次 adb server 跟踪，确保只处理本应用运行期间派生出的 server。
    force_stop_runtime_processes_by_paths(paths);
}

// ==============================
// Command: 获取在线更新 JSON (绕过 CORS)
// ==============================
#[tauri::command]
pub async fn get_online_update_json() -> Result<String, String> {
    let output = StdCommand::new("curl")
        .args(&[
            "-L",
            "https://gitee.com/xiaowan12/toolkit-tauri-app/raw/master/update.json",
        ])
        .creation_flags(0x08000000) // CREATE_NO_WINDOW
        .output()
        .map_err(|e| format!("请求失败: {}", e))?;

    if output.status.success() {
        String::from_utf8(output.stdout).map_err(|e| format!("解析失败: {}", e))
    } else {
        Err(format!("网络请求失败 (状态码: {})", output.status))
    }
}
