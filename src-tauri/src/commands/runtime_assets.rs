#![cfg_attr(dev, allow(dead_code))]

use crate::adb::core::{adb_run_async, AppPaths};
#[cfg(dev)]
use crate::adb::core::get_bin_root_dir;
use hex::encode as hex_encode;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use tauri::{AppHandle, Emitter, Manager, State};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::sync::Mutex;
use zip::ZipArchive;

const RUNTIME_PROGRESS_EVENT: &str = "runtime-assets-progress";
const RUNTIME_FS_RETRY_TIMES: usize = 30;
const RUNTIME_FS_RETRY_DELAY_MS: u64 = 400;

static RUNTIME_PREPARE_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct RuntimePart {
    name: String,
    size: Option<u64>,
    sha256: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct RuntimeManifest {
    version: String,
    archive_name: String,
    archive_sha256: Option<String>,
    archive_size: Option<u64>,
    base_url: Option<String>,
    required_files: Option<Vec<String>>,
    parts: Vec<RuntimePart>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct RuntimeInstallState {
    version: String,
    archive_sha256: Option<String>,
    required_files: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeAssetsProgress {
    pub phase: String,
    pub message: String,
    pub progress: f64,
    pub part_name: Option<String>,
    pub part_index: Option<usize>,
    pub part_total: Option<usize>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeAssetsResult {
    pub source: String,
    pub version: String,
    pub runtime_dir: String,
}

fn manifest_url() -> &'static str {
    option_env!("RANRAN_RUNTIME_MANIFEST_URL").unwrap_or(
        "https://gitee.com/xiaowan12/ranran-toolkit-bin/raw/master/cloud-parts/runtime-manifest.json",
    )
}

fn prepare_lock() -> &'static Mutex<()> {
    RUNTIME_PREPARE_LOCK.get_or_init(|| Mutex::new(()))
}

fn emit_progress(
    app: &AppHandle,
    phase: &str,
    message: impl Into<String>,
    progress: f64,
    part_name: Option<String>,
    part_index: Option<usize>,
    part_total: Option<usize>,
) {
    let _ = app.emit(
        RUNTIME_PROGRESS_EVENT,
        RuntimeAssetsProgress {
            phase: phase.to_string(),
            message: message.into(),
            progress,
            part_name,
            part_index,
            part_total,
        },
    );
}

fn runtime_root_dir(app: &AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_local_data_dir()
        .map(|dir| dir.join("runtime"))
        .map_err(|e| format!("获取运行时目录失败: {}", e))
}

fn runtime_bin_dir(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(runtime_root_dir(app)?.join("bin"))
}

fn runtime_state_path(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(runtime_root_dir(app)?.join("install-state.json"))
}

fn default_required_files() -> Vec<&'static str> {
    vec![
        "platform-tools/adb.exe",
        "platform-tools/fastboot.exe",
        "platform-tools/magiskboot.exe",
        "aria2-core/aria2c.exe",
        "scrcpy-core/scrcpy.exe",
        "link-dumper/link-dumper.exe",
    ]
}

fn required_files_from_manifest(manifest: &RuntimeManifest) -> Vec<String> {
    manifest.required_files.clone().unwrap_or_else(|| {
        default_required_files()
            .into_iter()
            .map(str::to_string)
            .collect()
    })
}

fn required_files_from_install_state(state: &RuntimeInstallState) -> Vec<String> {
    state.required_files.clone().unwrap_or_else(|| {
        default_required_files()
            .into_iter()
            .map(str::to_string)
            .collect()
    })
}

fn has_required_files(bin_dir: &Path, required_files: &[String]) -> bool {
    required_files
        .iter()
        .all(|relative| bin_dir.join(relative).is_file())
}

fn load_install_state(app: &AppHandle) -> Option<RuntimeInstallState> {
    let path = runtime_state_path(app).ok()?;
    let content = fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}

fn write_install_state(app: &AppHandle, manifest: &RuntimeManifest) -> Result<(), String> {
    let path = runtime_state_path(app)?;
    let state = RuntimeInstallState {
        version: manifest.version.clone(),
        archive_sha256: manifest.archive_sha256.clone(),
        required_files: Some(required_files_from_manifest(manifest)),
    };
    let content =
        serde_json::to_string_pretty(&state).map_err(|e| format!("写入运行时状态失败: {}", e))?;
    fs::write(path, content).map_err(|e| format!("写入运行时状态失败: {}", e))
}

fn local_runtime_state_if_ready(app: &AppHandle) -> Option<RuntimeInstallState> {
    let Some(state) = load_install_state(app) else {
        return None;
    };

    let Ok(bin_dir) = runtime_bin_dir(app) else {
        return None;
    };

    if has_required_files(&bin_dir, &required_files_from_install_state(&state)) {
        Some(state)
    } else {
        None
    }
}

fn is_runtime_ready(app: &AppHandle, manifest: &RuntimeManifest) -> bool {
    let Some(state) = local_runtime_state_if_ready(app) else {
        return false;
    };

    state.version == manifest.version && state.archive_sha256 == manifest.archive_sha256
}

async fn fetch_manifest(app: &AppHandle) -> Result<RuntimeManifest, String> {
    emit_progress(
        app,
        "manifest",
        "正在读取运行时清单...",
        6.0,
        None,
        None,
        None,
    );

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| format!("创建网络客户端失败: {}", e))?;

    let text = client
        .get(manifest_url())
        .send()
        .await
        .map_err(|e| format!("下载运行时清单失败: {}", e))?
        .error_for_status()
        .map_err(|e| format!("运行时清单响应异常: {}", e))?
        .text()
        .await
        .map_err(|e| format!("读取运行时清单失败: {}", e))?;

    serde_json::from_str(&text).map_err(|e| format!("解析运行时清单失败: {}", e))
}

fn normalize_base_url(manifest: &RuntimeManifest) -> String {
    manifest
        .base_url
        .clone()
        .unwrap_or_else(|| {
            manifest_url()
                .rsplit_once('/')
                .map(|(base, _)| base.to_string())
                .unwrap_or_else(|| manifest_url().to_string())
        })
        .trim_end_matches('/')
        .to_string()
}

async fn sha256_file(path: PathBuf) -> Result<String, String> {
    tokio::task::spawn_blocking(move || -> Result<String, String> {
        let mut file = fs::File::open(&path).map_err(|e| format!("读取校验文件失败: {}", e))?;
        let mut hasher = Sha256::new();
        let mut buffer = [0u8; 1024 * 64];

        loop {
            let read = file
                .read(&mut buffer)
                .map_err(|e| format!("读取校验文件失败: {}", e))?;
            if read == 0 {
                break;
            }
            hasher.update(&buffer[..read]);
        }

        Ok(hex_encode(hasher.finalize()))
    })
    .await
    .map_err(|e| format!("执行文件校验任务失败: {}", e))?
}

async fn verify_sha256(path: PathBuf, expected: &str, label: &str) -> Result<(), String> {
    let actual = sha256_file(path).await?;
    if actual.eq_ignore_ascii_case(expected) {
        Ok(())
    } else {
        Err(format!(
            "{} 校验失败，预期 {}，实际 {}",
            label, expected, actual
        ))
    }
}

async fn download_part(
    client: &reqwest::Client,
    url: &str,
    target_path: &Path,
) -> Result<(), String> {
    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("下载分卷失败: {}", e))?
        .error_for_status()
        .map_err(|e| format!("下载分卷响应异常: {}", e))?;

    let mut file = File::create(target_path)
        .await
        .map_err(|e| format!("写入分卷失败: {}", e))?;
    let mut response = response;

    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|e| format!("读取分卷下载流失败: {}", e))?
    {
        file.write_all(&chunk)
            .await
            .map_err(|e| format!("写入分卷失败: {}", e))?;
    }

    file.flush()
        .await
        .map_err(|e| format!("写入分卷失败: {}", e))?;
    Ok(())
}

async fn download_parts(
    app: &AppHandle,
    manifest: &RuntimeManifest,
    download_dir: &Path,
) -> Result<Vec<PathBuf>, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .map_err(|e| format!("创建下载客户端失败: {}", e))?;

    let base_url = normalize_base_url(manifest);
    let total_parts = manifest.parts.len();
    let total_bytes = manifest
        .parts
        .iter()
        .map(|part| part.size.unwrap_or(0))
        .sum::<u64>()
        .max(1);
    let mut downloaded_bytes = 0u64;
    let mut downloaded_paths = Vec::with_capacity(total_parts);

    for (index, part) in manifest.parts.iter().enumerate() {
        let url = format!("{}/{}", base_url, part.name);
        let target_path = download_dir.join(&part.name);
        let message = format!("正在下载分卷 {}/{}: {}", index + 1, total_parts, part.name);
        let progress = 10.0 + (downloaded_bytes as f64 / total_bytes as f64) * 60.0;
        emit_progress(
            app,
            "download",
            message,
            progress,
            Some(part.name.clone()),
            Some(index + 1),
            Some(total_parts),
        );

        download_part(&client, &url, &target_path).await?;

        if let Some(expected_size) = part.size {
            let actual_size = fs::metadata(&target_path)
                .map_err(|e| format!("读取分卷大小失败: {}", e))?
                .len();
            if actual_size != expected_size {
                return Err(format!(
                    "分卷 {} 大小不匹配，预期 {} 字节，实际 {} 字节",
                    part.name, expected_size, actual_size
                ));
            }
            downloaded_bytes += expected_size;
        }

        if let Some(expected_sha) = &part.sha256 {
            verify_sha256(
                target_path.clone(),
                expected_sha,
                &format!("分卷 {}", part.name),
            )
            .await?;
        }

        downloaded_paths.push(target_path);
    }

    Ok(downloaded_paths)
}

async fn combine_parts(output_path: PathBuf, part_paths: Vec<PathBuf>) -> Result<(), String> {
    tokio::task::spawn_blocking(move || -> Result<(), String> {
        let mut output =
            fs::File::create(&output_path).map_err(|e| format!("创建合并文件失败: {}", e))?;

        for part_path in part_paths {
            let mut input =
                fs::File::open(&part_path).map_err(|e| format!("读取分卷失败: {}", e))?;
            io::copy(&mut input, &mut output).map_err(|e| format!("合并分卷失败: {}", e))?;
        }

        output
            .flush()
            .map_err(|e| format!("写入合并文件失败: {}", e))
    })
    .await
    .map_err(|e| format!("执行分卷合并任务失败: {}", e))?
}

async fn extract_runtime_archive(zip_path: PathBuf, extract_root: PathBuf) -> Result<(), String> {
    tokio::task::spawn_blocking(move || -> Result<(), String> {
        let file = fs::File::open(&zip_path).map_err(|e| format!("打开运行时压缩包失败: {}", e))?;
        let mut archive =
            ZipArchive::new(file).map_err(|e| format!("解析运行时压缩包失败: {}", e))?;

        for index in 0..archive.len() {
            let mut entry = archive
                .by_index(index)
                .map_err(|e| format!("读取压缩条目失败: {}", e))?;
            let Some(relative_path) = entry.enclosed_name().map(|path| path.to_path_buf()) else {
                continue;
            };
            let output_path = extract_root.join(relative_path);

            if entry.is_dir() {
                fs::create_dir_all(&output_path).map_err(|e| format!("创建目录失败: {}", e))?;
                continue;
            }

            if let Some(parent) = output_path.parent() {
                fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {}", e))?;
            }

            let mut output =
                fs::File::create(&output_path).map_err(|e| format!("写入运行时文件失败: {}", e))?;
            io::copy(&mut entry, &mut output).map_err(|e| format!("写入运行时文件失败: {}", e))?;
            output
                .flush()
                .map_err(|e| format!("写入运行时文件失败: {}", e))?;
        }

        Ok(())
    })
    .await
    .map_err(|e| format!("执行运行时解压任务失败: {}", e))?
}

fn remove_dir_if_exists(path: &Path) -> Result<(), String> {
    const REMOVE_RETRY_TIMES: usize = RUNTIME_FS_RETRY_TIMES;
    const REMOVE_RETRY_DELAY_MS: u64 = RUNTIME_FS_RETRY_DELAY_MS;

    for attempt in 0..REMOVE_RETRY_TIMES {
        if !path.exists() {
            return Ok(());
        }

        match fs::remove_dir_all(path) {
            Ok(_) => return Ok(()),
            Err(error) => {
                if attempt + 1 == REMOVE_RETRY_TIMES {
                    return Err(format!("删除目录失败: {}", error));
                }
                std::thread::sleep(std::time::Duration::from_millis(REMOVE_RETRY_DELAY_MS));
            }
        }
    }

    Ok(())
}

fn rename_dir_with_retry(from: &Path, to: &Path) -> Result<(), String> {
    for attempt in 0..RUNTIME_FS_RETRY_TIMES {
        match fs::rename(from, to) {
            Ok(_) => return Ok(()),
            Err(error) => {
                if attempt + 1 == RUNTIME_FS_RETRY_TIMES {
                    return Err(format!("安装运行时依赖失败: {}", error));
                }
                std::thread::sleep(std::time::Duration::from_millis(
                    RUNTIME_FS_RETRY_DELAY_MS,
                ));
            }
        }
    }

    Ok(())
}

fn remove_file_if_exists(path: &Path) -> Result<(), String> {
    if path.exists() {
        fs::remove_file(path).map_err(|e| format!("删除文件失败: {}", e))?;
    }
    Ok(())
}

async fn install_runtime_assets(app: &AppHandle, manifest: &RuntimeManifest) -> Result<(), String> {
    let runtime_root = runtime_root_dir(app)?;
    let runtime_bin = runtime_bin_dir(app)?;
    let work_root = runtime_root.join(".setup");
    let download_dir = work_root.join("parts");
    let extract_dir = work_root.join("extract");
    let archive_path = work_root.join(&manifest.archive_name);

    fs::create_dir_all(&download_dir).map_err(|e| format!("创建下载目录失败: {}", e))?;
    remove_dir_if_exists(&extract_dir)?;
    fs::create_dir_all(&extract_dir).map_err(|e| format!("创建解压目录失败: {}", e))?;

    let part_paths = download_parts(app, manifest, &download_dir).await?;

    emit_progress(app, "combine", "正在合并分卷...", 74.0, None, None, None);
    remove_file_if_exists(&archive_path)?;
    combine_parts(archive_path.clone(), part_paths).await?;

    if let Some(expected_size) = manifest.archive_size {
        let actual_size = fs::metadata(&archive_path)
            .map_err(|e| format!("读取合并包大小失败: {}", e))?
            .len();
        if actual_size != expected_size {
            return Err(format!(
                "运行时压缩包大小不匹配，预期 {} 字节，实际 {} 字节",
                expected_size, actual_size
            ));
        }
    }

    if let Some(expected_sha) = &manifest.archive_sha256 {
        emit_progress(
            app,
            "verify",
            "正在校验运行时压缩包...",
            82.0,
            None,
            None,
            None,
        );
        verify_sha256(archive_path.clone(), expected_sha, "运行时压缩包").await?;
    }

    emit_progress(
        app,
        "extract",
        "正在解压运行时依赖...",
        90.0,
        None,
        None,
        None,
    );
    extract_runtime_archive(archive_path.clone(), extract_dir.clone()).await?;

    let staged_bin = extract_dir.join("bin");
    if !staged_bin.exists() {
        return Err("运行时压缩包缺少 bin 目录，请重新生成发布分卷".to_string());
    }

    remove_dir_if_exists(&runtime_bin)?;
    if let Some(parent) = runtime_bin.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("创建运行时目录失败: {}", e))?;
    }
    rename_dir_with_retry(&staged_bin, &runtime_bin)?;

    let required_files = required_files_from_manifest(manifest);
    if !has_required_files(&runtime_bin, &required_files) {
        return Err("运行时依赖解压完成，但关键工具文件不完整".to_string());
    }

    write_install_state(app, manifest)?;

    let _ = remove_file_if_exists(&archive_path);
    let _ = remove_dir_if_exists(&download_dir);
    let _ = remove_dir_if_exists(&extract_dir);
    let _ = remove_dir_if_exists(&work_root);
    Ok(())
}

#[tauri::command]
pub async fn prepare_runtime_assets(app: AppHandle) -> Result<RuntimeAssetsResult, String> {
    let _guard = prepare_lock().lock().await;

    #[cfg(dev)]
    {
        let runtime_dir = get_bin_root_dir(&app).display().to_string();
        emit_progress(
            &app,
            "ready",
            "开发环境已使用本地 bin 目录",
            100.0,
            None,
            None,
            None,
        );
        return Ok(RuntimeAssetsResult {
            source: "dev".to_string(),
            version: "dev".to_string(),
            runtime_dir,
        });
    }

    #[cfg(not(dev))]
    {
        crate::commands::system::force_stop_runtime_processes(&app);
        emit_progress(
            &app,
            "check",
            "正在检查运行时依赖...",
            2.0,
            None,
            None,
            None,
        );
        let local_runtime_state = local_runtime_state_if_ready(&app);

        if local_runtime_state.is_some() {
            emit_progress(
                &app,
                "manifest",
                "已检测到本地运行时缓存，正在检查是否需要更新...",
                6.0,
                None,
                None,
                None,
            );
        }

        let manifest = match fetch_manifest(&app).await {
            Ok(manifest) => manifest,
            Err(error) => {
                if let Some(state) = local_runtime_state.as_ref() {
                    let runtime_dir = runtime_bin_dir(&app)?.display().to_string();
                    emit_progress(
                        &app,
                        "ready",
                        "无法连接运行时清单，已使用本地缓存离线启动",
                        100.0,
                        None,
                        None,
                        None,
                    );
                    return Ok(RuntimeAssetsResult {
                        source: "runtime-cache-offline".to_string(),
                        version: state.version.clone(),
                        runtime_dir,
                    });
                }

                return Err(error);
            }
        };

        if manifest.parts.is_empty() {
            return Err("运行时清单未包含任何分卷信息".to_string());
        }

        if is_runtime_ready(&app, &manifest) {
            let runtime_dir = runtime_bin_dir(&app)?.display().to_string();
            emit_progress(&app, "ready", "运行时依赖已就绪", 100.0, None, None, None);
            return Ok(RuntimeAssetsResult {
                source: "runtime-cache".to_string(),
                version: manifest.version.clone(),
                runtime_dir,
            });
        }

        if local_runtime_state.is_some() {
            emit_progress(
                &app,
                "download",
                "本地运行时缓存已过期或损坏，正在下载新版依赖...",
                10.0,
                None,
                None,
                None,
            );
        }

        install_runtime_assets(&app, &manifest).await?;
        let runtime_dir = runtime_bin_dir(&app)?.display().to_string();
        emit_progress(&app, "ready", "运行时依赖安装完成", 100.0, None, None, None);

        return Ok(RuntimeAssetsResult {
            source: "download".to_string(),
            version: manifest.version,
            runtime_dir,
        });
    }
}

#[tauri::command]
pub async fn warmup_platform_tools(paths: State<'_, AppPaths>) -> Result<(), String> {
    let _ = adb_run_async(&paths.adb, &["devices"]).await;
    Ok(())
}

#[tauri::command]
pub fn get_tool_runtime_path(paths: State<'_, AppPaths>) -> Result<String, String> {
    let bin_dir = paths
        .adb
        .parent()
        .and_then(|platform_tools_dir| platform_tools_dir.parent())
        .ok_or_else(|| "无法解析工具依赖目录".to_string())?;

    Ok(bin_dir.display().to_string())
}
