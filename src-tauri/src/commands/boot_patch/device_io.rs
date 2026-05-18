use super::*;

pub(super) async fn run_adb_command(
    adb_path: &PathBuf,
    serial: Option<&str>,
    args: &[String],
) -> Result<(bool, String), String> {
    let mut command = create_hidden_async_command(adb_path);

    if let Some(serial) = serial.filter(|value| !value.is_empty()) {
        command.arg("-s").arg(serial);
    }

    for arg in args {
        command.arg(arg);
    }

    let output = output_tracked_async_command(&mut command, PROCESS_KIND_ADB_CLIENT)
        .await
        .map_err(|e| format!("执行 adb 命令失败: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

    let combined = if stdout.is_empty() {
        stderr
    } else if stderr.is_empty() {
        stdout
    } else {
        format!("{}\n{}", stdout, stderr)
    };

    Ok((output.status.success(), combined))
}

pub(super) async fn run_fastboot_command(
    fastboot_path: &PathBuf,
    serial: Option<&str>,
    args: &[String],
) -> Result<(bool, String), String> {
    let _guard = acquire_fastboot_command_guard().await;
    let mut command = create_hidden_async_command(fastboot_path);

    if let Some(serial) = serial.filter(|value| !value.is_empty()) {
        command.arg("-s").arg(serial);
    }

    for arg in args {
        command.arg(arg);
    }

    let output = output_tracked_async_command(&mut command, PROCESS_KIND_FASTBOOT)
        .await
        .map_err(|e| format!("执行 fastboot 命令失败: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

    let combined = if stdout.is_empty() {
        stderr
    } else if stderr.is_empty() {
        stdout
    } else {
        format!("{}\n{}", stdout, stderr)
    };

    Ok((output.status.success(), combined))
}

pub(super) async fn run_checked_adb(
    window: &Window,
    adb_path: &PathBuf,
    serial: Option<&str>,
    args: &[String],
    tag: &str,
    error_prefix: &str,
) -> Result<String, String> {
    let (success, output) = run_adb_command(adb_path, serial, args).await?;

    if !output.trim().is_empty() {
        emit_log(
            window,
            output.clone(),
            if success { "info" } else { "error" },
            tag,
        );
    }

    if success {
        Ok(output)
    } else if output.trim().is_empty() {
        Err(error_prefix.to_string())
    } else {
        Err(format!("{}: {}", error_prefix, output))
    }
}

pub(super) async fn run_quiet_adb(
    adb_path: &PathBuf,
    serial: Option<&str>,
    args: &[String],
    error_prefix: &str,
) -> Result<(), String> {
    let (success, output) = run_adb_command(adb_path, serial, args).await?;

    if success {
        Ok(())
    } else if output.trim().is_empty() {
        Err(error_prefix.to_string())
    } else {
        Err(format!("{}: {}", error_prefix, output))
    }
}

pub(super) async fn run_checked_fastboot(
    window: &Window,
    fastboot_path: &PathBuf,
    serial: Option<&str>,
    args: &[String],
    tag: &str,
    error_prefix: &str,
) -> Result<String, String> {
    let (success, output) = run_fastboot_command(fastboot_path, serial, args).await?;

    if !output.trim().is_empty() {
        emit_log(
            window,
            output.clone(),
            if success { "info" } else { "error" },
            tag,
        );
    }

    if success {
        Ok(output)
    } else if output.trim().is_empty() {
        Err(error_prefix.to_string())
    } else {
        Err(format!("{}: {}", error_prefix, output))
    }
}

pub(super) fn parse_kernel_major_minor(value: &str) -> Option<String> {
    let bytes = value.as_bytes();
    let len = bytes.len();
    let mut index = 0;

    while index < len {
        if !bytes[index].is_ascii_digit() {
            index += 1;
            continue;
        }

        let start = index;
        while index < len && bytes[index].is_ascii_digit() {
            index += 1;
        }

        if index >= len || bytes[index] != b'.' {
            continue;
        }

        index += 1;
        let minor_start = index;
        while index < len && bytes[index].is_ascii_digit() {
            index += 1;
        }

        if minor_start == index {
            continue;
        }

        return Some(value[start..index].to_string());
    }

    None
}

pub(super) fn parse_android_tag_from_text(value: &str) -> Option<String> {
    let lower = value.to_ascii_lowercase();
    let start = lower.find("android")?;
    let suffix = &lower[start + "android".len()..];
    let digits: String = suffix
        .chars()
        .take_while(|ch| ch.is_ascii_digit())
        .collect();

    if digits.is_empty() {
        None
    } else {
        Some(format!("android{}", digits))
    }
}

pub(super) fn parse_android_tag_from_release(value: &str) -> Option<String> {
    let digits: String = value
        .trim()
        .chars()
        .skip_while(|ch| !ch.is_ascii_digit())
        .take_while(|ch| ch.is_ascii_digit())
        .collect();

    if digits.is_empty() {
        None
    } else {
        Some(format!("android{}", digits))
    }
}

pub(super) fn build_device_kmi(kernel_release: &str, android_release: &str) -> String {
    let kernel_version = parse_kernel_major_minor(kernel_release).unwrap_or_default();
    let android_tag = parse_android_tag_from_text(kernel_release)
        .or_else(|| parse_android_tag_from_release(android_release))
        .unwrap_or_default();

    if kernel_version.is_empty() || android_tag.is_empty() {
        String::new()
    } else {
        format!("{}-{}", android_tag, kernel_version)
    }
}

pub(super) async fn detect_device_kmi(
    adb_path: &PathBuf,
    serial: Option<&str>,
) -> Result<String, String> {
    let kernel_release = match run_adb_command(
        adb_path,
        serial,
        &["shell".to_string(), "uname".to_string(), "-r".to_string()],
    )
    .await
    {
        Ok((true, output)) => output,
        Ok((false, _)) | Err(_) => String::new(),
    };

    let android_release = match run_adb_command(
        adb_path,
        serial,
        &[
            "shell".to_string(),
            "getprop".to_string(),
            "ro.build.version.release".to_string(),
        ],
    )
    .await
    {
        Ok((true, output)) => output,
        Ok((false, _)) | Err(_) => String::new(),
    };

    Ok(build_device_kmi(&kernel_release, &android_release))
}

pub(super) async fn install_root_manager_apk(
    window: &Window,
    adb_path: &PathBuf,
    serial: Option<&str>,
    apk_path: &Path,
    app_label: &str,
) -> (bool, Option<String>) {
    emit_log(
        window,
        format!("安装对应 {} APK: {}", app_label, apk_path.display()),
        "info",
        "ROOT",
    );

    let install_args = vec![
        "install".to_string(),
        "-r".to_string(),
        "-d".to_string(),
        apk_path.to_string_lossy().to_string(),
    ];

    match run_adb_command(adb_path, serial, &install_args).await {
        Ok((true, output)) => {
            if !output.trim().is_empty() {
                emit_log(window, output, "info", "ROOT");
            }
            emit_log(
                window,
                format!("{} APK 安装完成", app_label),
                "success",
                "ROOT",
            );
            (true, None)
        }
        Ok((false, output)) => {
            let message = if output.trim().is_empty() {
                format!("安装 {} APK 失败", app_label)
            } else {
                format!("安装 {} APK 失败: {}", app_label, output)
            };
            emit_log(window, message.clone(), "warning", "ROOT");
            emit_log(
                window,
                format!(
                    "将跳过自动安装并继续后续刷入流程；如有需要，可在设备开机后手动安装 APK: {}",
                    apk_path.display()
                ),
                "warning",
                "ROOT",
            );
            (false, Some(message))
        }
        Err(error) => {
            let message = format!("安装 {} APK 失败: {}", app_label, error);
            emit_log(window, message.clone(), "warning", "ROOT");
            emit_log(
                window,
                format!(
                    "将跳过自动安装并继续后续刷入流程；如有需要，可在设备开机后手动安装 APK: {}",
                    apk_path.display()
                ),
                "warning",
                "ROOT",
            );
            (false, Some(message))
        }
    }
}

pub(super) async fn detect_target_slot_suffix_best_effort(
    window: &Window,
    adb_path: &PathBuf,
    serial: Option<&str>,
    required: bool,
) -> Result<String, String> {
    match detect_current_slot_suffix(adb_path, serial).await {
        Ok(slot_suffix) => {
            let slot_suffix = slot_suffix.unwrap_or_default();
            if slot_suffix.is_empty() {
                emit_log(
                    window,
                    "未读取到 ro.boot.slot_suffix，将按无插槽设备处理，后续刷入不会附带 _a / _b",
                    "info",
                    "PREP",
                );
            } else {
                emit_log(
                    window,
                    format!("当前目标插槽: {}", slot_suffix),
                    "info",
                    "PREP",
                );
            }
            Ok(slot_suffix)
        }
        Err(error) if !required => {
            emit_log(
                window,
                format!(
                    "未能读取 ro.boot.slot_suffix，KernelSU 类仅修补模式将继续执行: {}",
                    error
                ),
                "warning",
                "PREP",
            );
            Ok(String::new())
        }
        Err(error) => Err(error),
    }
}

pub(super) async fn detect_current_device_mode(
    adb_path: &PathBuf,
    fastboot_path: &PathBuf,
    serial: Option<&str>,
) -> Result<String, String> {
    detect_device_state(adb_path, fastboot_path, serial).await
}

pub(super) async fn wait_for_expected_mode(
    adb_path: &PathBuf,
    fastboot_path: &PathBuf,
    serial: Option<&str>,
    expected_modes: &[&str],
    attempts: usize,
    interval_ms: u64,
) -> Result<String, String> {
    for _ in 0..attempts {
        let current_mode = detect_current_device_mode(adb_path, fastboot_path, serial).await?;
        if expected_modes.iter().any(|mode| *mode == current_mode) {
            return Ok(current_mode);
        }

        sleep(Duration::from_millis(interval_ms)).await;
    }

    Err(format!("等待设备进入 {} 超时", expected_modes.join(" / ")))
}

pub(super) async fn resolve_fastboot_serial(
    fastboot_path: &PathBuf,
    preferred_serial: Option<&str>,
) -> Result<Option<String>, String> {
    let (_, output) = run_fastboot_command(fastboot_path, None, &["devices".to_string()]).await?;
    let serials: Vec<String> = output
        .lines()
        .filter_map(|line| line.split_whitespace().next())
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .collect();

    if let Some(preferred) = preferred_serial.filter(|value| !value.is_empty()) {
        if serials.iter().any(|value| value == preferred) {
            return Ok(Some(preferred.to_string()));
        }
    }

    if serials.len() == 1 {
        return Ok(Some(serials[0].clone()));
    }

    if preferred_serial.is_none() && !serials.is_empty() {
        return Ok(None);
    }

    Err("未找到可用的 Fastboot 设备序列号，或当前连接了多台设备".to_string())
}

pub(super) fn fastboot_output_succeeded(output: &str) -> bool {
    let upper = output.to_ascii_uppercase();
    upper.contains("OKAY") || upper.contains("FINISHED")
}

pub(super) fn is_supported_root_partition(target_partition: &str) -> bool {
    let (base_partition, _) = split_partition_and_slot_suffix(target_partition);
    matches!(base_partition.as_str(), "boot" | "init_boot")
}

pub(super) async fn flash_patched_image(
    window: &Window,
    fastboot_path: &PathBuf,
    serial: Option<&str>,
    target_partition: &str,
    target_slot_suffix: Option<&str>,
    patched_image_path: &Path,
) -> Result<String, String> {
    let flash_partition = build_partition_with_slot_suffix(target_partition, target_slot_suffix);
    let args = vec![
        "flash".to_string(),
        flash_partition.clone(),
        patched_image_path.to_string_lossy().to_string(),
    ];

    let (success, output) = run_fastboot_command(fastboot_path, serial, &args).await?;
    if !output.trim().is_empty() {
        emit_log(
            window,
            output.clone(),
            if success { "info" } else { "error" },
            "FLASH",
        );
    }
    let target_partition = flash_partition.as_str();

    if success && fastboot_output_succeeded(&output) {
        Ok(flash_partition)
    } else if output.trim().is_empty() {
        Err(format!("刷入 {} 失败", target_partition))
    } else {
        Err(output)
    }
}

pub(super) async fn reboot_system_from_fastboot_best_effort(
    window: &Window,
    fastboot_path: &PathBuf,
    serial: Option<&str>,
    reason: &str,
) {
    emit_log(
        window,
        format!("准备重启开机: {}", reason),
        "warning",
        "ROOT",
    );

    match run_fastboot_command(fastboot_path, serial, &["reboot".to_string()]).await {
        Ok((true, output)) => {
            if !output.trim().is_empty() {
                emit_log(window, output, "info", "ROOT");
            }
            emit_log(window, "已发送开机指令", "success", "ROOT");
        }
        Ok((false, output)) => {
            let message = if output.trim().is_empty() {
                "开机指令执行失败".to_string()
            } else {
                format!("开机指令执行失败: {}", output)
            };
            emit_log(window, message, "warning", "ROOT");
        }
        Err(error) => emit_log(
            window,
            format!("开机指令执行异常: {}", error),
            "warning",
            "ROOT",
        ),
    }
}

pub(super) async fn ensure_fastboot_mode(
    window: &Window,
    adb_path: &PathBuf,
    fastboot_path: &PathBuf,
    serial: Option<&str>,
) -> Result<Option<String>, String> {
    let current_mode = wait_for_expected_mode(
        adb_path,
        fastboot_path,
        serial,
        &["fastboot", "fastbootd"],
        DEVICE_MODE_WAIT_ATTEMPTS,
        DEVICE_MODE_WAIT_INTERVAL_MS,
    )
    .await?;

    let mut fastboot_serial = resolve_fastboot_serial(fastboot_path, serial).await?;

    if current_mode == "fastboot" {
        emit_log(window, "设备已进入 Fastboot 模式", "success", "ROOT");
        return Ok(fastboot_serial);
    }

    emit_log(
        window,
        "检测到设备当前位于 FastbootD，先切回 Fastboot 再执行首次刷入",
        "warning",
        "ROOT",
    );
    run_checked_fastboot(
        window,
        fastboot_path,
        fastboot_serial.as_deref(),
        &["reboot".to_string(), "bootloader".to_string()],
        "ROOT",
        "从 FastbootD 切回 Fastboot 失败",
    )
    .await?;

    wait_for_expected_mode(
        adb_path,
        fastboot_path,
        serial,
        &["fastboot"],
        DEVICE_MODE_WAIT_ATTEMPTS,
        DEVICE_MODE_WAIT_INTERVAL_MS,
    )
    .await?;
    fastboot_serial = resolve_fastboot_serial(fastboot_path, serial).await?;
    emit_log(
        window,
        "已切回 Fastboot 模式，准备首次刷入",
        "success",
        "ROOT",
    );

    Ok(fastboot_serial)
}

pub(super) async fn read_device_abi(
    window: &Window,
    adb_path: &PathBuf,
    serial: Option<&str>,
) -> Result<String, String> {
    emit_log(window, "读取设备 ABI 信息", "info", "CHK");
    let output = run_checked_adb(
        window,
        adb_path,
        serial,
        &[
            "shell".to_string(),
            "getprop".to_string(),
            "ro.product.cpu.abi".to_string(),
        ],
        "CHK",
        "读取设备 ABI 失败",
    )
    .await?;

    let abi = output.lines().last().unwrap_or("").trim().to_string();
    if abi.is_empty() {
        return Err("设备 ABI 为空，无法选择对应的 Magisk 二进制".to_string());
    }
    Ok(abi)
}

pub(super) fn normalize_lib_folder(abi: &str) -> Result<&'static str, String> {
    let abi = abi.trim();
    if abi.starts_with("arm64-v8a") {
        Ok("arm64-v8a")
    } else if abi.starts_with("armeabi-v7a") || abi.starts_with("armeabi") {
        Ok("armeabi-v7a")
    } else if abi.starts_with("x86_64") {
        Ok("x86_64")
    } else if abi.starts_with("x86") {
        Ok("x86")
    } else {
        Err(format!("暂不支持该设备 ABI: {}", abi))
    }
}

pub(super) async fn stream_patch_command(
    window: &Window,
    adb_path: &PathBuf,
    serial: Option<&str>,
    shell_command: &str,
) -> Result<(), String> {
    let mut command = create_hidden_async_command(adb_path);

    if let Some(serial) = serial.filter(|value| !value.is_empty()) {
        command.arg("-s").arg(serial);
    }

    let mut child = spawn_tracked_async_command(
        command
            .arg("shell")
            .arg(shell_command)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped()),
        PROCESS_KIND_ADB_CLIENT,
    )
    .map_err(|e| format!("启动手机端修补进程失败: {}", e))?;

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| "无法读取修补标准输出".to_string())?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| "无法读取修补错误输出".to_string())?;

    let stdout_task = tokio::spawn(async move {
        let mut reader = stdout;
        let mut buffer = Vec::new();
        let _ = reader.read_to_end(&mut buffer).await;
        String::from_utf8_lossy(&buffer).trim().to_string()
    });

    let stderr_task = tokio::spawn(async move {
        let mut reader = stderr;
        let mut buffer = Vec::new();
        let _ = reader.read_to_end(&mut buffer).await;
        String::from_utf8_lossy(&buffer).trim().to_string()
    });

    let status = wait_tracked_async_child(&mut child, PROCESS_KIND_ADB_CLIENT)
        .await
        .map_err(|e| format!("等待修补进程结束失败: {}", e))?;

    let stdout_output = stdout_task.await.unwrap_or_default();
    let stderr_output = stderr_task.await.unwrap_or_default();

    if !stdout_output.is_empty() {
        emit_log(window, stdout_output, "info", "PATCH");
    }

    if !stderr_output.is_empty() {
        emit_log(window, stderr_output, "error", "PATCH");
    }

    if status.success() {
        Ok(())
    } else {
        Err(format!(
            "boot_patch.sh 执行失败，退出码: {}",
            status
                .code()
                .map(|code| code.to_string())
                .unwrap_or_else(|| "未知".to_string())
        ))
    }
}

pub(super) async fn cleanup_remote_dir(
    window: &Window,
    adb_path: &PathBuf,
    serial: Option<&str>,
    remote_work_dir: &str,
) {
    let args = vec![
        "shell".to_string(),
        "rm".to_string(),
        "-rf".to_string(),
        remote_work_dir.to_string(),
    ];

    match run_adb_command(adb_path, serial, &args).await {
        Ok((true, output)) => {
            if !output.trim().is_empty() {
                emit_log(window, output, "info", "CLEAN");
            }
            emit_log(
                window,
                format!("已清理手机临时目录: {}", remote_work_dir),
                "success",
                "CLEAN",
            );
        }
        Ok((false, output)) => {
            let message = if output.trim().is_empty() {
                format!("清理手机临时目录失败: {}", remote_work_dir)
            } else {
                format!("清理手机临时目录失败: {}", output)
            };
            emit_log(window, message, "warning", "CLEAN");
        }
        Err(error) => emit_log(
            window,
            format!("清理手机临时目录失败: {}", error),
            "warning",
            "CLEAN",
        ),
    }
}

pub(super) fn cleanup_local_dir(window: &Window, local_dir: &Path) {
    if !local_dir.exists() {
        return;
    }

    match fs::remove_dir_all(local_dir) {
        Ok(_) => emit_log(
            window,
            format!("已清理本地临时目录: {}", local_dir.display()),
            "success",
            "CLEAN",
        ),
        Err(error) => emit_log(
            window,
            format!("清理本地临时目录失败: {}", error),
            "warning",
            "CLEAN",
        ),
    }
}

pub(super) async fn recreate_remote_dir(
    window: &Window,
    adb_path: &PathBuf,
    serial: Option<&str>,
    remote_work_dir: &str,
) -> Result<(), String> {
    emit_log(
        window,
        format!("清理并重建手机工作目录: {}", remote_work_dir),
        "info",
        "PUSH",
    );
    run_checked_adb(
        window,
        adb_path,
        serial,
        &[
            "shell".to_string(),
            format!(
                "rm -rf {dir} && mkdir -p {dir}",
                dir = shell_quote(remote_work_dir)
            ),
        ],
        "PUSH",
        "重建手机工作目录失败",
    )
    .await?;
    Ok(())
}
