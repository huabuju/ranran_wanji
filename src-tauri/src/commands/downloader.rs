use crate::adb::core::get_bin_root_dir;
use crate::utils::process::{
    spawn_tracked_async_command, wait_tracked_async_child, PROCESS_KIND_ARIA2C,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, Manager, State};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command as TokioCommand;
use tokio::sync::mpsc;

// ==============================
// 预检：GET Range:bytes=0-0
// 在 aria2c 启动前探测链接可访问性，提供明确错误信息
// ==============================
async fn preflight_check(url: &str, referer_val: &str) -> Result<(), String> {
    let mut header_map = reqwest::header::HeaderMap::new();
    let ua = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";
    header_map.insert(reqwest::header::USER_AGENT, ua.parse().unwrap());
    header_map.insert(reqwest::header::ACCEPT,            "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8".parse().unwrap());
    header_map.insert(
        reqwest::header::ACCEPT_LANGUAGE,
        "zh-CN,zh;q=0.9,en;q=0.8".parse().unwrap(),
    );
    header_map.insert(reqwest::header::CONNECTION, "keep-alive".parse().unwrap());
    if !referer_val.is_empty() {
        if let Ok(v) = referer_val.parse() {
            header_map.insert(reqwest::header::REFERER, v);
        }
    }

    let client = reqwest::Client::builder()
        .default_headers(header_map)
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;

    let resp = client
        .get(url)
        .header("Range", "bytes=0-0")
        .send()
        .await
        .map_err(|e| {
            if e.is_timeout() {
                "连接超时，请检查网络或链接是否有效".to_string()
            } else if e.is_connect() {
                format!("无法连接到服务器: {}", e)
            } else {
                format!("预检请求失败: {}", e)
            }
        })?;

    match resp.status().as_u16() {
        200 | 206 => Ok(()),
        403 => Err("服务器拒绝访问（403 Forbidden），请在高级选项中设置正确的 Referer，或确认链接是否需要登录".to_string()),
        404 => Err("资源不存在（404 Not Found），请确认链接是否正确或已失效".to_string()),
        401 => Err("需要身份验证（401 Unauthorized），此链接需要登录后才能访问".to_string()),
        410 => Err("资源已被删除（410 Gone），链接可能已过期".to_string()),
        code => Err(format!("服务器返回异常状态码 {}，链接可能无效或受限", code)),
    }
}

// ==============================
// 数据模型
// ==============================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum DownloadStatus {
    Pending,
    Downloading,
    Paused,
    Completed,
    Error,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadTask {
    pub id: String,
    pub url: String,
    pub file_name: String,
    pub save_dir: String,
    pub threads: u32,
    pub referer: Option<String>, // 自定义 Referer 头
    pub status: DownloadStatus,
    pub progress: f64,      // 0.0 ~ 100.0
    pub speed: String,      // 如 "1.2MB/s"
    pub downloaded: String, // 如 "1.6GB (42.0%)"
    pub eta: String,        // 如 "00:00:47"
    pub total_size: String, // 如 "3.8GB"
    pub completed_at: Option<String>,
    pub error_msg: Option<String>,
}

// 进度事件载荷
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadProgressEvent {
    pub id: String,
    pub status: DownloadStatus,
    pub progress: f64,
    pub speed: String,
    pub downloaded: String,
    pub eta: String,
    pub total_size: String,
    pub completed_at: Option<String>,
    pub error_msg: Option<String>,
}

// 全局下载任务管理状态
pub struct DownloadManager {
    pub tasks: Mutex<HashMap<String, DownloadTask>>,
}

impl DownloadManager {
    pub fn new() -> Self {
        Self {
            tasks: Mutex::new(HashMap::new()),
        }
    }
}

// ==============================
// 辅助：获取 aria2c 路径
// ==============================
#[allow(dead_code)]
fn get_aria2c_path(app: &AppHandle) -> Result<PathBuf, String> {
    // resource_dir 在打包后指向资源目录，开发环境下指向项目根
    let base = app
        .path()
        .resource_dir()
        .map_err(|e| format!("无法获取资源目录: {}", e))?;

    // 打包后：resources/bin/aria2-core/aria2c.exe
    // 开发时：../bin/aria2-core/aria2c.exe（相对 tauri 工作目录）
    let packaged = base.join("bin").join("aria2-core").join("aria2c.exe");
    if packaged.exists() {
        return Ok(packaged);
    }

    // 开发环境兜底：从 tauri 源码目录往上两级找 bin
    let dev_path = base
        .parent()
        .and_then(|p| p.parent())
        .map(|p| p.join("bin").join("aria2-core").join("aria2c.exe"));
    if let Some(p) = dev_path {
        if p.exists() {
            return Ok(p);
        }
    }

    Err("找不到 aria2c.exe，请确认 bin/aria2-core/aria2c.exe 存在".to_string())
}

fn get_runtime_aria2c_path(app: &AppHandle) -> Result<PathBuf, String> {
    let packaged = get_bin_root_dir(app).join("aria2-core").join("aria2c.exe");
    if packaged.exists() {
        Ok(packaged)
    } else {
        Err("找不到 aria2c.exe，请确认 bin/aria2-core/aria2c.exe 存在".to_string())
    }
}

// ==============================
// 解析 aria2c 输出行
// ==============================
// aria2c 下载中输出示例：
//   [#abc123  1.6GiB/3.8GiB(42%) CN:16 DL:19.6MiB ETA:47s]
// 下载完成输出：
//   Download complete: /path/to/file
fn parse_aria2_line(line: &str) -> Option<(f64, String, String, String, String)> {
    // 找到 [ ... ] 块
    let start = line.find('[')?;
    let end = line.rfind(']')?;
    if start >= end {
        return None;
    }
    let inner = &line[start + 1..end];

    // 提取百分比
    let pct_re = inner.find('(')?;
    let pct_end = inner[pct_re..].find('%')?;
    let pct_str = &inner[pct_re + 1..pct_re + pct_end];
    let progress: f64 = pct_str.trim().parse().ok()?;

    // 提取 downloaded/total：形如 "1.6GiB/3.8GiB"
    let size_part = inner[..pct_re].trim();
    // 去掉 GID 前缀（#xxxxxx  ）
    let size_part = if let Some(idx) = size_part.rfind("  ") {
        size_part[idx..].trim()
    } else {
        size_part
    };
    let downloaded = size_part.to_string();
    let total_size = if let Some(slash) = size_part.find('/') {
        size_part[slash + 1..].to_string()
    } else {
        String::new()
    };

    // 提取速度：DL:xxx
    let speed = if let Some(dl_pos) = inner.find("DL:") {
        let after = &inner[dl_pos + 3..];
        let end = after
            .find(|c: char| c == ' ' || c == ']')
            .unwrap_or(after.len());
        format_size_speed(&after[..end])
    } else {
        "0B/s".to_string()
    };

    // 提取 ETA
    let eta = if let Some(eta_pos) = inner.find("ETA:") {
        let after = &inner[eta_pos + 4..];
        let end = after
            .find(|c: char| c == ' ' || c == ']')
            .unwrap_or(after.len());
        format_eta(&after[..end])
    } else {
        "--".to_string()
    };

    Some((progress, speed, downloaded, eta, total_size))
}

fn format_size_speed(s: &str) -> String {
    // aria2 输出如 "19.6MiB" → "19.6MB/s"
    s.replace("GiB", "GB")
        .replace("MiB", "MB")
        .replace("KiB", "KB")
        + "/s"
}

fn format_eta(s: &str) -> String {
    // aria2 输出如 "47s" → "00:00:47", "2m3s" → "00:02:03", "1h5m" → "01:05:00"
    let s = s.trim();
    let mut hours = 0u64;
    let mut mins = 0u64;
    let mut secs = 0u64;

    let mut buf = String::new();
    for ch in s.chars() {
        if ch.is_ascii_digit() {
            buf.push(ch);
        } else {
            let num: u64 = buf.trim().parse().unwrap_or(0);
            buf.clear();
            match ch {
                'h' => hours = num,
                'm' => mins = num,
                's' => secs = num,
                _ => {}
            }
        }
    }
    format!("{:02}:{:02}:{:02}", hours, mins, secs)
}

// ==============================
// Command: 开始下载
// ==============================
#[tauri::command]
pub async fn start_download(
    app: AppHandle,
    manager: State<'_, Arc<DownloadManager>>,
    id: String,
    url: String,
    save_dir: String,
    file_name: String,
    threads: u32,
    referer: Option<String>,
) -> Result<(), String> {
    let aria2c = get_runtime_aria2c_path(&app)?;

    let mut file_name = file_name.trim().split('?').next().unwrap_or("").to_string();
    if file_name.is_empty() {
        file_name = url
            .split('?')
            .next()
            .unwrap_or(&url)
            .split('/')
            .last()
            .unwrap_or("download")
            .to_string();
    }

    // 确保保存目录存在
    std::fs::create_dir_all(&save_dir).map_err(|e| format!("创建目录失败: {}", e))?;

    // ---- 预检：用 GET Range:bytes=0-0 探测链接 ----
    // 在任务入队前完成，失败直接返回 Err，前端可展示明确原因
    let preflight_referer = referer
        .as_deref()
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.to_string())
        .unwrap_or_else(|| {
            if let Ok(u) = url::Url::parse(&url) {
                format!("{}://{}/", u.scheme(), u.host_str().unwrap_or(""))
            } else {
                String::new()
            }
        });
    preflight_check(&url, &preflight_referer).await?;
    // ---- 预检通过 ----

    // 构造任务
    let task = DownloadTask {
        id: id.clone(),
        url: url.clone(),
        file_name: file_name.clone(),
        save_dir: save_dir.clone(),
        threads,
        referer: referer.clone(),
        status: DownloadStatus::Downloading,
        progress: 0.0,
        speed: "0B/s".to_string(),
        downloaded: "0B".to_string(),
        eta: "--".to_string(),
        total_size: "--".to_string(),
        completed_at: None,
        error_msg: None,
    };

    {
        let mut tasks = manager.tasks.lock().unwrap();
        tasks.insert(id.clone(), task);
    }

    // 发布初始状态
    let _ = app.emit(
        "download-progress",
        DownloadProgressEvent {
            id: id.clone(),
            status: DownloadStatus::Downloading,
            progress: 0.0,
            speed: "0B/s".to_string(),
            downloaded: "0B".to_string(),
            eta: "--".to_string(),
            total_size: "--".to_string(),
            completed_at: None,
            error_msg: None,
        },
    );

    let manager_clone = Arc::clone(&manager);
    let app_clone = app.clone();
    let id_clone = id.clone();

    // 异步执行下载
    tokio::spawn(async move {
        let thread_str = threads.to_string();
        let mut cmd = TokioCommand::new(&aria2c);

        // 构造完整浏览器请求头，以应对部分服务器的反爬/403 拦截
        let referer_val = referer
            .as_deref()
            .filter(|s| !s.trim().is_empty())
            .map(|s| s.to_string())
            .unwrap_or_else(|| {
                // 自动从 URL 提取 origin 作为默认 Referer
                if let Ok(u) = url::Url::parse(&url) {
                    format!("{}://{}/", u.scheme(), u.host_str().unwrap_or(""))
                } else {
                    String::new()
                }
            });

        cmd.args([
            "--dir", &save_dir,
            "--out", &file_name,
            "--split", &thread_str,
            "--max-connection-per-server", &thread_str,
            "--min-split-size", "1M",
            "--console-log-level=notice",
            "--summary-interval=1",
            "--show-console-readout=true",
            "--check-certificate=false",
            "--retry-wait=3",
            "--max-tries=5",
            "--user-agent",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
            "--header", "Accept: text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7",
            "--header", "Accept-Language: zh-CN,zh;q=0.9,en;q=0.8",
            "--header", "Accept-Encoding: identity",
            "--header", "Connection: keep-alive",
        ]);

        // 仅在 referer 非空时添加
        if !referer_val.is_empty() {
            cmd.args(["--header", &format!("Referer: {}", referer_val)]);
        }

        cmd.arg(&url).stdout(Stdio::piped()).stderr(Stdio::piped());

        // Windows 下必须设置 CREATE_NO_WINDOW，防止弹出黑色控制台窗口
        // 弹窗不仅影响体验，还会导致 aria2c 检测到自己在「交互式控制台」而改变输出行为
        // tokio::process::Command 在 Windows 上自带 creation_flags 方法，无需额外 import trait
        #[cfg(target_os = "windows")]
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW

        let mut child = match spawn_tracked_async_command(&mut cmd, PROCESS_KIND_ARIA2C) {
            Ok(c) => c,
            Err(e) => {
                let _ = app_clone.emit(
                    "download-progress",
                    DownloadProgressEvent {
                        id: id_clone.clone(),
                        status: DownloadStatus::Error,
                        progress: 0.0,
                        speed: "".to_string(),
                        downloaded: "".to_string(),
                        eta: "".to_string(),
                        total_size: "".to_string(),
                        completed_at: None,
                        error_msg: Some(format!("启动 aria2c 失败: {}", e)),
                    },
                );
                return;
            }
        };

        // 同时读取 stdout 和 stderr
        // aria2c 在非 TTY 环境下：进度行 → stderr，日志行有时会到 stdout
        // 使用 mpsc channel 合并两个流
        let (tx, mut rx) = mpsc::unbounded_channel::<String>();

        let stdout = child.stdout.take().expect("stdout not captured");
        let stderr = child.stderr.take().expect("stderr not captured");

        let tx_out = tx.clone();
        tokio::spawn(async move {
            let mut lines = BufReader::new(stdout).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                let _ = tx_out.send(line);
            }
        });

        let tx_err = tx.clone();
        tokio::spawn(async move {
            let mut lines = BufReader::new(stderr).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                let _ = tx_err.send(line);
            }
        });
        drop(tx); // 让两个子任务持有发送端，主循环持有接收端

        while let Some(line) = rx.recv().await {
            let should_kill = manager_clone
                .tasks
                .lock()
                .unwrap()
                .get(&id_clone)
                .map_or(false, |t| t.status == DownloadStatus::Cancelled);
            if should_kill {
                let _ = child.kill().await;
                let _ = wait_tracked_async_child(&mut child, PROCESS_KIND_ARIA2C).await;

                let file_path = std::path::Path::new(&save_dir).join(&file_name);
                let aria2_file_path =
                    std::path::Path::new(&save_dir).join(format!("{}.aria2", file_name));
                let _ = std::fs::remove_file(&file_path);
                let _ = std::fs::remove_file(&aria2_file_path);
                return;
            }

            if line.contains("Download complete") || line.contains("NOTICE - Download complete") {
                let now = chrono_now();
                {
                    let mut tasks = manager_clone.tasks.lock().unwrap();
                    if let Some(t) = tasks.get_mut(&id_clone) {
                        t.status = DownloadStatus::Completed;
                        t.progress = 100.0;
                        t.completed_at = Some(now.clone());
                    }
                }
                let _ = app_clone.emit(
                    "download-progress",
                    DownloadProgressEvent {
                        id: id_clone.clone(),
                        status: DownloadStatus::Completed,
                        progress: 100.0,
                        speed: "".to_string(),
                        downloaded: "".to_string(),
                        eta: "".to_string(),
                        total_size: "".to_string(),
                        completed_at: Some(now),
                        error_msg: None,
                    },
                );
                break;
            }

            if let Some((progress, speed, downloaded, eta, total_size)) = parse_aria2_line(&line) {
                {
                    let mut tasks = manager_clone.tasks.lock().unwrap();
                    if let Some(t) = tasks.get_mut(&id_clone) {
                        t.progress = progress;
                        t.speed = speed.clone();
                        t.downloaded = downloaded.clone();
                        t.eta = eta.clone();
                        t.total_size = total_size.clone();
                    }
                }
                let _ = app_clone.emit(
                    "download-progress",
                    DownloadProgressEvent {
                        id: id_clone.clone(),
                        status: DownloadStatus::Downloading,
                        progress,
                        speed,
                        downloaded,
                        eta,
                        total_size,
                        completed_at: None,
                        error_msg: None,
                    },
                );
            }
        }

        // 进程结束检查退出码
        if let Ok(status) = wait_tracked_async_child(&mut child, PROCESS_KIND_ARIA2C).await {
            let current_status = {
                let tasks_guard = manager_clone.tasks.lock().unwrap();
                tasks_guard
                    .get(&id_clone)
                    .map(|t| t.status.clone())
                    .unwrap_or(DownloadStatus::Error)
            };

            if current_status == DownloadStatus::Cancelled {
                // 如果进程结束是因为被取消，确保文件被删除
                let file_path = std::path::Path::new(&save_dir).join(&file_name);
                let aria2_file_path =
                    std::path::Path::new(&save_dir).join(format!("{}.aria2", file_name));
                let _ = std::fs::remove_file(&file_path);
                let _ = std::fs::remove_file(&aria2_file_path);
            } else if !status.success() && current_status != DownloadStatus::Completed {
                // 仅在非完成、非取消时标记错误
                {
                    let mut tasks = manager_clone.tasks.lock().unwrap();
                    if let Some(t) = tasks.get_mut(&id_clone) {
                        t.status = DownloadStatus::Error;
                        t.error_msg = Some(format!("aria2c 退出码: {}", status));
                    }
                }
                let _ = app_clone.emit(
                    "download-progress",
                    DownloadProgressEvent {
                        id: id_clone.clone(),
                        status: DownloadStatus::Error,
                        progress: 0.0,
                        speed: "".to_string(),
                        downloaded: "".to_string(),
                        eta: "".to_string(),
                        total_size: "".to_string(),
                        completed_at: None,
                        error_msg: Some(format!("下载失败，aria2c 退出码: {}", status)),
                    },
                );
            }
        }
    });

    Ok(())
}

// ==============================
// Command: 重试下载
// ==============================
#[tauri::command]
pub async fn retry_download(
    app: AppHandle,
    manager: State<'_, Arc<DownloadManager>>,
    id: String,
) -> Result<(), String> {
    // 1. 获取原任务的信息
    let (url, save_dir, file_name, threads, referer) = {
        let tasks = manager.tasks.lock().unwrap();
        if let Some(task) = tasks.get(&id) {
            // 只有失败或取消的任务允许重试
            if task.status != DownloadStatus::Error && task.status != DownloadStatus::Cancelled {
                return Err("只能重试失败或已取消的任务".to_string());
            }
            (
                task.url.clone(),
                task.save_dir.clone(),
                task.file_name.clone(),
                task.threads,
                task.referer.clone(),
            )
        } else {
            return Err(format!("任务 {} 不存在", id));
        }
    };

    // 2. 重新调用 start_download
    // 注意这里由于 state 是注入的，代码复用可以使用同一个逻辑
    start_download(app, manager, id, url, save_dir, file_name, threads, referer).await
}

// ==============================
// Command: 取消下载
// ==============================
#[tauri::command]
pub fn cancel_download(
    app: AppHandle,
    manager: State<'_, Arc<DownloadManager>>,
    id: String,
) -> Result<(), String> {
    let (save_dir, file_name) = {
        let mut tasks = manager.tasks.lock().unwrap();
        if let Some(task) = tasks.get_mut(&id) {
            task.status = DownloadStatus::Cancelled;
            let _ = app.emit(
                "download-progress",
                DownloadProgressEvent {
                    id: id.clone(),
                    status: DownloadStatus::Cancelled,
                    progress: task.progress,
                    speed: "".to_string(),
                    downloaded: task.downloaded.clone(),
                    eta: "".to_string(),
                    total_size: task.total_size.clone(),
                    completed_at: None,
                    error_msg: None,
                },
            );
            (task.save_dir.clone(), task.file_name.clone())
        } else {
            return Err(format!("任务 {} 不存在", id));
        }
    };

    // 尝试同步删除文件（如果此时 aria2c 未在运行或无锁，可直接成功。否则在 start_download 的退出流程中也会删除）
    let file_path = std::path::Path::new(&save_dir).join(&file_name);
    let aria2_file_path = std::path::Path::new(&save_dir).join(format!("{}.aria2", file_name));
    let _ = std::fs::remove_file(&file_path);
    let _ = std::fs::remove_file(&aria2_file_path);

    Ok(())
}

// ==============================
// Command: 获取所有任务
// ==============================
#[tauri::command]
pub fn get_download_tasks(manager: State<'_, Arc<DownloadManager>>) -> Vec<DownloadTask> {
    let tasks = manager.tasks.lock().unwrap();
    tasks.values().cloned().collect()
}

// ==============================
// Command: 移除已完成/错误/取消任务
// ==============================
#[tauri::command]
pub fn remove_download_task(
    manager: State<'_, Arc<DownloadManager>>,
    id: String,
) -> Result<(), String> {
    let mut tasks = manager.tasks.lock().unwrap();
    if tasks.remove(&id).is_some() {
        Ok(())
    } else {
        Err(format!("任务 {} 不存在", id))
    }
}

// ==============================
// Command: 打开下载文件夹
// ==============================
#[tauri::command]
pub fn open_download_folder(save_dir: String) -> Result<(), String> {
    use std::process::Command as StdCommand;
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        StdCommand::new("explorer")
            .arg(&save_dir)
            .creation_flags(0x08000000)
            .spawn()
            .map_err(|e| format!("打开文件夹失败: {}", e))?;
    }
    Ok(())
}

// ==============================
// 暴露给外部（如 lib.rs 应用退出时）的清理方法
// ==============================
pub fn cleanup_on_exit(manager: &Arc<DownloadManager>) {
    let tasks = manager.tasks.lock().unwrap();
    // 强制清理所有未完成的状态文件。
    // 因为 taskkill 强杀进程后，异步协程极有可能抢先将状态改为 Error，
    // 而且 Windows 下文件句柄的释放需要几百毫秒，因此加入一个轮询重试机制。
    for task in tasks.values() {
        if task.status != DownloadStatus::Completed {
            let file_path = std::path::Path::new(&task.save_dir).join(&task.file_name);
            let aria2_file_path =
                std::path::Path::new(&task.save_dir).join(format!("{}.aria2", task.file_name));

            // 重试 5 次（约 1 秒），等待 aria2c.exe 完全释放文件锁
            for _ in 0..5 {
                let r1 = std::fs::remove_file(&file_path);
                let r2 = std::fs::remove_file(&aria2_file_path);

                let ok1 = r1.is_ok()
                    || r1
                        .as_ref()
                        .err()
                        .map_or(false, |e| e.kind() == std::io::ErrorKind::NotFound);
                let ok2 = r2.is_ok()
                    || r2
                        .as_ref()
                        .err()
                        .map_or(false, |e| e.kind() == std::io::ErrorKind::NotFound);

                if ok1 && ok2 {
                    break;
                }
                std::thread::sleep(std::time::Duration::from_millis(200));
            }
        }
    }
}

// ==============================
// 辅助：获取当前时间字符串
// ==============================
fn chrono_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    // 简单格式化为 YYYY-MM-DD HH:mm（UTC+8）
    let secs_cst = secs + 8 * 3600;
    let days_total = secs_cst / 86400;
    let time_of_day = secs_cst % 86400;
    let hh = time_of_day / 3600;
    let mm = (time_of_day % 3600) / 60;

    // 简单年月日计算（足够精度用于显示）
    let mut year = 1970u64;
    let mut remaining_days = days_total;
    loop {
        let days_in_year = if is_leap(year) { 366 } else { 365 };
        if remaining_days < days_in_year {
            break;
        }
        remaining_days -= days_in_year;
        year += 1;
    }
    let months = [
        31,
        if is_leap(year) { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    let mut month = 1u64;
    for m in months {
        if remaining_days < m {
            break;
        }
        remaining_days -= m;
        month += 1;
    }
    let day = remaining_days + 1;

    format!("{}-{:02}-{:02} {:02}:{:02}", year, month, day, hh, mm)
}

fn is_leap(y: u64) -> bool {
    (y % 4 == 0 && y % 100 != 0) || y % 400 == 0
}
