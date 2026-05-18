use super::*;

pub(super) async fn inspect_payload_partitions(
    window: &Window,
    payload_source: &str,
    info_dir: &Path,
) -> Result<Vec<Value>, String> {
    let app_handle = window.app_handle();
    let dumper_path = get_link_dumper_path(&app_handle);
    if !dumper_path.exists() {
        return Err(format!("未找到 link-dumper 工具: {:?}", dumper_path));
    }

    fs::create_dir_all(info_dir).map_err(|e| format!("创建 payload 信息目录失败: {}", e))?;
    emit_log(
        window,
        format!("正在解析 payload 分区信息: {}", payload_source),
        "info",
        "PREP",
    );

    let mut info_command = create_hidden_async_command(&dumper_path);
    let output = output_tracked_async_command(
        info_command
            .arg("--info")
            .arg("--out")
            .arg(info_dir)
            .arg(payload_source),
        PROCESS_KIND_LINK_DUMPER,
    )
    .await
    .map_err(|e| format!("解析 payload 信息失败: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if !stdout.is_empty() {
        emit_log(window, stdout, "info", "PREP");
    }

    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    if !stderr.is_empty() {
        emit_log(
            window,
            stderr.clone(),
            if output.status.success() {
                "info"
            } else {
                "error"
            },
            "PREP",
        );
    }

    if !output.status.success() {
        return Err(if stderr.is_empty() {
            format!(
                "解析 payload 信息失败，退出码: {}",
                output
                    .status
                    .code()
                    .map(|code| code.to_string())
                    .unwrap_or_else(|| "未知".to_string())
            )
        } else {
            format!("解析 payload 信息失败: {}", stderr)
        });
    }

    let partitions_file = info_dir.join("partitions_info.json");
    if !partitions_file.exists() {
        return Err("payload 解析完成，但未找到 partitions_info.json".to_string());
    }

    let content =
        fs::read_to_string(&partitions_file).map_err(|e| format!("读取分区信息失败: {}", e))?;
    let partitions: Vec<Value> =
        serde_json::from_str(&content).map_err(|e| format!("解析分区信息失败: {}", e))?;
    Ok(partitions)
}

pub(super) async fn extract_payload_partition_for_patch(
    window: &Window,
    payload_source: &str,
    partition_name: &str,
    extract_dir: &Path,
) -> Result<PathBuf, String> {
    let app_handle = window.app_handle();
    let dumper_path = get_link_dumper_path(&app_handle);
    if !dumper_path.exists() {
        return Err(format!("未找到 link-dumper 工具: {:?}", dumper_path));
    }

    fs::create_dir_all(extract_dir).map_err(|e| format!("创建 payload 提取目录失败: {}", e))?;

    emit_log(
        window,
        format!("已识别可修补分区: {}", partition_name),
        "success",
        "PREP",
    );
    emit_log(
        window,
        format!("开始自动提取 {} 分区", partition_name),
        "info",
        "PREP",
    );

    let mut command = create_hidden_async_command(&dumper_path);
    let mut child = spawn_tracked_async_command(
        command
            .arg("--partitions")
            .arg(partition_name)
            .arg("--out")
            .arg(extract_dir)
            .arg(payload_source)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped()),
        PROCESS_KIND_LINK_DUMPER,
    )
    .map_err(|e| format!("启动 payload 提取进程失败: {}", e))?;

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| "无法读取 payload 提取标准输出".to_string())?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| "无法读取 payload 提取错误输出".to_string())?;

    let window_stdout = window.clone();
    let stdout_handle = tokio::spawn(async move {
        let mut reader = BufReader::new(stdout).lines();
        while let Ok(Some(line)) = reader.next_line().await {
            emit_log(&window_stdout, line, "info", "PREP");
        }
    });

    let window_stderr = window.clone();
    let stderr_handle = tokio::spawn(async move {
        let mut reader = BufReader::new(stderr).lines();
        while let Ok(Some(line)) = reader.next_line().await {
            emit_log(&window_stderr, line, "error", "PREP");
        }
    });

    let status = wait_tracked_async_child(&mut child, PROCESS_KIND_LINK_DUMPER)
        .await
        .map_err(|e| format!("等待 payload 提取进程结束失败: {}", e))?;

    let _ = stdout_handle.await;
    let _ = stderr_handle.await;

    if !status.success() {
        return Err(format!(
            "payload 分区提取失败，退出码: {}",
            status
                .code()
                .map(|code| code.to_string())
                .unwrap_or_else(|| "未知".to_string())
        ));
    }

    let partition_basename = payload_partition_basename(partition_name);
    let expected_file_name = if partition_basename.to_ascii_lowercase().ends_with(".img") {
        partition_basename.to_string()
    } else {
        format!("{}.img", partition_basename)
    };
    let extracted_path = find_partition_image_recursive(extract_dir, &expected_file_name)?
        .ok_or_else(|| format!("提取完成，但未找到输出文件: {}", expected_file_name))?;

    emit_log(
        window,
        format!("自动提取完成: {}", extracted_path.display()),
        "success",
        "PREP",
    );

    Ok(extracted_path)
}
