use crate::adb::core::{create_hidden_async_command, get_link_dumper_path};
use crate::utils::process::{
    output_tracked_async_command, spawn_tracked_async_command, wait_tracked_async_child,
    PROCESS_KIND_LINK_DUMPER,
};
use serde::Serialize;
use std::fs;
use std::process::Stdio;
use tauri::{AppHandle, Emitter, Manager, Window};
use tokio::io::{AsyncBufReadExt, BufReader};

#[derive(Debug, Serialize, Clone)]
pub struct PayloadLog {
    pub content: String,
    pub log_type: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct PayloadResponse {
    pub partitions: Vec<serde_json::Value>,
    pub metadata: Option<serde_json::Value>,
}

#[tauri::command]
pub async fn list_payload_partitions(
    app: tauri::AppHandle,
    payload_path: String,
) -> Result<PayloadResponse, String> {
    let dumper_path = get_link_dumper_path(&app);
    if !dumper_path.exists() {
        return Err(format!("未找到 link-dumper 工具: {:?}", dumper_path));
    }

    let output_dir = app
        .path()
        .app_local_data_dir()
        .map_err(|e| format!("获取应用数据目录失败: {}", e))?
        .join("link-dumper-temp");

    fs::create_dir_all(&output_dir).map_err(|e| format!("创建输出目录失败: {}", e))?;

    let mut info_command = create_hidden_async_command(&dumper_path);
    let _ = output_tracked_async_command(
        info_command
            .arg("--info")
            .arg("--out")
            .arg(&output_dir)
            .arg(&payload_path),
        PROCESS_KIND_LINK_DUMPER,
    )
    .await
    .map_err(|e| format!("获取信息失败: {}", e))?;

    let partitions_file = output_dir.join("partitions_info.json");
    let partitions: Vec<serde_json::Value> = if partitions_file.exists() {
        let content = fs::read_to_string(partitions_file).map_err(|e| e.to_string())?;
        serde_json::from_str(&content).map_err(|e| e.to_string())?
    } else {
        let _ = fs::remove_dir_all(&output_dir);
        return Err("获取列表失败：在输出目录中未找到 partitions_info.json 文件。".to_string());
    };

    let mut metadata: Option<serde_json::Value> = None;
    let metadata_file = output_dir.join("metadata.json");
    if metadata_file.exists() {
        if let Ok(content) = fs::read_to_string(metadata_file) {
            if let Ok(json) = serde_json::from_str(&content) {
                metadata = Some(json);
            }
        }
    }

    let _ = fs::remove_dir_all(output_dir);

    Ok(PayloadResponse {
        partitions,
        metadata,
    })
}

#[tauri::command]
pub async fn extract_payload_partitions(
    app: AppHandle,
    window: Window,
    payload_path: String,
    partitions: Vec<String>,
    output_dir: String,
) -> Result<(), String> {
    let dumper_path = get_link_dumper_path(&app);
    if !dumper_path.exists() {
        return Err(format!("未找到 link-dumper 工具: {:?}", dumper_path));
    }

    let partitions_arg = partitions.join(",");
    let mut command = create_hidden_async_command(&dumper_path);
    let mut child = spawn_tracked_async_command(
        command
            .arg("--partitions")
            .arg(&partitions_arg)
            .arg("--out")
            .arg(&output_dir)
            .arg(&payload_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped()),
        PROCESS_KIND_LINK_DUMPER,
    )
    .map_err(|e| format!("启动提取进程失败: {}", e))?;

    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();

    let window_clone = window.clone();
    let stdout_handle = tokio::spawn(async move {
        let mut reader = BufReader::new(stdout).lines();
        while let Ok(Some(line)) = reader.next_line().await {
            let _ = window_clone.emit(
                "payload-log",
                PayloadLog {
                    content: line,
                    log_type: "info".to_string(),
                },
            );
        }
    });

    let window_clone_err = window.clone();
    let stderr_handle = tokio::spawn(async move {
        let mut reader = BufReader::new(stderr).lines();
        while let Ok(Some(line)) = reader.next_line().await {
            let _ = window_clone_err.emit(
                "payload-log",
                PayloadLog {
                    content: line,
                    log_type: "error".to_string(),
                },
            );
        }
    });

    let status = wait_tracked_async_child(&mut child, PROCESS_KIND_LINK_DUMPER)
        .await
        .map_err(|e| format!("等待进程结束失败: {}", e))?;

    let _ = stdout_handle.await;
    let _ = stderr_handle.await;

    if status.success() {
        let _ = window.emit(
            "payload-log",
            PayloadLog {
                content: "提取任务全部完成。".to_string(),
                log_type: "success".to_string(),
            },
        );
        Ok(())
    } else {
        Err(format!("进程退出码异常: {:?}", status.code()))
    }
}
