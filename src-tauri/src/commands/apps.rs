use crate::adb::core::{
    adb_pull_to_local_file, adb_run_async_with_serial, create_hidden_async_command, get_tool_path,
    AppPaths,
};
use crate::utils::process::{output_tracked_async_command, PROCESS_KIND_ADB_CLIENT};
use crate::models::device_info::{AppLabelEntry, PackageInfo};
use std::path::PathBuf;
use tauri::{AppHandle, State};

// ==============================
// 获取已安装包列表 (区分用户/系统)
// ==============================
#[tauri::command]
pub async fn get_packages(
    paths: State<'_, AppPaths>,
    serial: Option<String>,
    only_third_party: bool,
) -> Result<Vec<String>, String> {
    let serial_str = serial.unwrap_or_default();
    let sr: Option<&str> = if serial_str.is_empty() {
        None
    } else {
        Some(serial_str.as_str())
    };

    let mut args = vec!["shell", "pm", "list", "packages"];
    if only_third_party {
        args.push("-3");
    } else {
        args.push("-s");
    }

    let output = adb_run_async_with_serial(&paths.adb, sr, &args)
        .await
        .unwrap_or_default();
    let packages: Vec<String> = output
        .lines()
        .filter_map(|l| {
            let s = l.trim();
            if s.starts_with("package:") {
                Some(s.replace("package:", "").trim().to_string())
            } else {
                None
            }
        })
        .collect();

    Ok(packages)
}

// ==============================
// 批量获取所有已禁用的包名列表
// ==============================
#[tauri::command]
pub async fn get_disabled_packages(
    paths: State<'_, AppPaths>,
    serial: Option<String>,
) -> Result<Vec<String>, String> {
    let serial_str = serial.unwrap_or_default();
    let sr: Option<&str> = if serial_str.is_empty() {
        None
    } else {
        Some(serial_str.as_str())
    };

    let output =
        adb_run_async_with_serial(&paths.adb, sr, &["shell", "pm", "list", "packages", "-d"])
            .await
            .unwrap_or_default();
    let packages: Vec<String> = output
        .lines()
        .filter_map(|l| {
            let s = l.trim();
            if s.starts_with("package:") {
                Some(s.replace("package:", "").trim().to_string())
            } else {
                None
            }
        })
        .collect();
    Ok(packages)
}

// ==============================
// 获取单个包的详细信息
// ==============================
#[tauri::command]
pub async fn get_package_detail(
    paths: State<'_, AppPaths>,
    serial: Option<String>,
    package_name: String,
) -> Result<PackageInfo, String> {
    let serial_str = serial.unwrap_or_default();
    let sr: Option<&str> = if serial_str.is_empty() {
        None
    } else {
        Some(serial_str.as_str())
    };

    let output = adb_run_async_with_serial(
        &paths.adb,
        sr,
        &["shell", "dumpsys", "package", &package_name],
    )
    .await
    .unwrap_or_default();

    let mut is_enabled = true;
    for line in output.lines() {
        if line.trim().contains("enabled=false") {
            is_enabled = false;
        }
    }

    Ok(PackageInfo {
        package_name,
        is_system: false,
        is_enabled,
    })
}

// ==============================
// 获取前台应用包名/Activity
// ==============================
#[tauri::command]
pub async fn get_foreground_package(
    paths: State<'_, AppPaths>,
    serial: Option<String>,
) -> Result<(String, String), String> {
    let serial_str = serial.unwrap_or_default();
    let sr: Option<&str> = if serial_str.is_empty() {
        None
    } else {
        Some(serial_str.as_str())
    };

    let output =
        adb_run_async_with_serial(&paths.adb, sr, &["shell", "dumpsys", "activity", "top"])
            .await
            .unwrap_or_default();

    let mut pkg = String::new();
    let mut act = String::new();

    for line in output.lines().take(100) {
        let s = line.trim();
        if s.starts_with("ACTIVITY") && s.contains("/") {
            let parts: Vec<&str> = s.split_whitespace().collect();
            for part in parts {
                if part.contains("/") && part.contains(".") {
                    let cleaned = part.trim_end_matches('}');
                    let sub_parts: Vec<&str> = cleaned.split('/').collect();
                    if sub_parts.len() == 2 {
                        pkg = sub_parts[0].to_string();
                        act = cleaned.to_string();
                        break;
                    }
                }
            }
        }
        if !pkg.is_empty() {
            break;
        }
    }

    if pkg.is_empty() {
        return Err("无法获取前台应用信息".to_string());
    }

    Ok((pkg, act))
}

// ==============================
// 应用管理操作 (卸载, 清除, 冻结, 解冻, 停止)
// ==============================
#[tauri::command]
pub async fn manage_package(
    paths: State<'_, AppPaths>,
    serial: Option<String>,
    package_name: String,
    action: String,
) -> Result<String, String> {
    let serial_str = serial.unwrap_or_default();
    let sr: Option<&str> = if serial_str.is_empty() {
        None
    } else {
        Some(serial_str.as_str())
    };

    match action.as_str() {
        "freeze" => try_freeze(&paths.adb, sr, &package_name).await,
        "unfreeze" => try_unfreeze(&paths.adb, sr, &package_name).await,
        "block_network" => try_block_network(&paths.adb, sr, &package_name).await,
        "unblock_network" => try_unblock_network(&paths.adb, sr, &package_name).await,
        _ => {
            let args = match action.as_str() {
                "uninstall" => vec!["uninstall".to_string(), package_name],
                "uninstall_keep" => vec![
                    "shell".to_string(),
                    "pm".to_string(),
                    "uninstall".to_string(),
                    "-k".to_string(),
                    "--user".to_string(),
                    "0".to_string(),
                    package_name,
                ],
                "clear" => vec![
                    "shell".to_string(),
                    "pm".to_string(),
                    "clear".to_string(),
                    package_name,
                ],
                "stop" => vec![
                    "shell".to_string(),
                    "am".to_string(),
                    "force-stop".to_string(),
                    package_name,
                ],
                _ => return Err("不支持的操作".to_string()),
            };
            let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
            adb_run_async_with_serial(&paths.adb, sr, &args_ref).await
        }
    }
}

// ==============================
// 判断 pm 命令输出是否代表操作失败
// ==============================
fn is_pm_output_failed(output: &str) -> bool {
    let lower = output.to_lowercase();
    lower.contains("exception")
        || lower.contains("error")
        || lower.contains("failed")
        || lower.contains("unknown")
        || lower.trim().is_empty()
}

// ==============================
// 冻结 - 多方案降级（Fallback）
// ==============================
async fn try_freeze(
    adb: &std::path::PathBuf,
    serial: Option<&str>,
    package_name: &str,
) -> Result<String, String> {
    let strategies: &[(&[&str], &str)] = &[
        (
            &["shell", "pm", "disable-user", "--user", "0"],
            "pm disable-user --user 0",
        ),
        (&["shell", "pm", "disable"], "pm disable"),
        (&["shell", "pm", "hide"], "pm hide"),
    ];

    let mut last_err = String::new();

    for (cmd_base, label) in strategies {
        let mut args: Vec<&str> = cmd_base.to_vec();
        args.push(package_name);
        match adb_run_async_with_serial(adb, serial, &args).await {
            Ok(output) if !is_pm_output_failed(&output) => {
                return Ok(format!("冻结成功（方案: {}）: {}", label, output));
            }
            Ok(output) => {
                last_err = format!("[{}] 输出异常: {}", label, output);
            }
            Err(e) => {
                last_err = format!("[{}] 执行失败: {}", label, e);
            }
        }
    }

    Err(format!("所有冻结方案均失败，最后错误: {}", last_err))
}

// ==============================
// 解冻 - 多方案降级（Fallback）
// ==============================
async fn try_unfreeze(
    adb: &std::path::PathBuf,
    serial: Option<&str>,
    package_name: &str,
) -> Result<String, String> {
    let strategies: &[(&[&str], &str)] = &[
        (
            &["shell", "pm", "enable", "--user", "0"],
            "pm enable --user 0",
        ),
        (&["shell", "pm", "enable"], "pm enable"),
        (&["shell", "pm", "unhide"], "pm unhide"),
    ];

    let mut last_err = String::new();

    for (cmd_base, label) in strategies {
        let mut args: Vec<&str> = cmd_base.to_vec();
        args.push(package_name);
        match adb_run_async_with_serial(adb, serial, &args).await {
            Ok(output) if !is_pm_output_failed(&output) => {
                return Ok(format!("解冻成功（方案: {}）: {}", label, output));
            }
            Ok(output) => {
                last_err = format!("[{}] 输出异常: {}", label, output);
            }
            Err(e) => {
                last_err = format!("[{}] 执行失败: {}", label, e);
            }
        }
    }

    Err(format!("所有解冻方案均失败，最后错误: {}", last_err))
}

// ==============================
// 禁网 - 基于 cmd connectivity (Android 11+)
// ==============================
async fn try_block_network(
    adb: &std::path::PathBuf,
    serial: Option<&str>,
    package_name: &str,
) -> Result<String, String> {
    // 确保开启了应用级防火墙过滤链 (chain3)
    let _ = adb_run_async_with_serial(
        adb,
        serial,
        &["shell", "cmd", "connectivity", "set-chain3-enabled", "true"],
    )
    .await;

    // 执行禁网命令
    match adb_run_async_with_serial(
        adb,
        serial,
        &[
            "shell",
            "cmd",
            "connectivity",
            "set-package-networking-enabled",
            "false",
            package_name,
        ],
    )
    .await
    {
        Ok(output)
            if !output.to_lowercase().contains("error")
                && !output.to_lowercase().contains("exception")
                && !output.to_lowercase().contains("unknown command") =>
        {
            Ok(format!("禁网成功: {}", output.trim()))
        }
        Ok(output) => Err(format!("禁网可能失败 (该功能需要 Android 11+): {}", output)),
        Err(e) => Err(format!("禁网执行失败: {}", e)),
    }
}

// ==============================
// 联网 - 基于 cmd connectivity (Android 11+)
// ==============================
async fn try_unblock_network(
    adb: &std::path::PathBuf,
    serial: Option<&str>,
    package_name: &str,
) -> Result<String, String> {
    match adb_run_async_with_serial(
        adb,
        serial,
        &[
            "shell",
            "cmd",
            "connectivity",
            "set-package-networking-enabled",
            "true",
            package_name,
        ],
    )
    .await
    {
        Ok(output)
            if !output.to_lowercase().contains("error")
                && !output.to_lowercase().contains("exception")
                && !output.to_lowercase().contains("unknown command") =>
        {
            Ok(format!("联网成功: {}", output.trim()))
        }
        Ok(output) => Err(format!("联网可能失败 (该功能需要 Android 11+): {}", output)),
        Err(e) => Err(format!("联网执行失败: {}", e)),
    }
}

// ==============================
// 安装 APK
// ==============================
#[tauri::command]
pub async fn install_package(
    paths: State<'_, AppPaths>,
    serial: Option<String>,
    apk_path: String,
    reinstall: bool,
    downgrade: bool,
) -> Result<String, String> {
    let serial_str = serial.unwrap_or_default();
    let sr: Option<&str> = if serial_str.is_empty() {
        None
    } else {
        Some(serial_str.as_str())
    };

    let mut args = vec!["install".to_string()];
    if reinstall {
        args.push("-r".to_string());
    }
    if downgrade {
        args.push("-d".to_string());
    }
    args.push(apk_path);

    let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    adb_run_async_with_serial(&paths.adb, sr, &args_ref).await
}

// ==============================
// 批量获取应用真实显示名 (Label) 和 UID
// ==============================
#[tauri::command]
pub async fn get_app_labels(
    app: AppHandle,
    paths: State<'_, AppPaths>,
    serial: Option<String>,
    only_third_party: bool,
) -> Result<Vec<AppLabelEntry>, String> {
    let serial_str = serial.clone().unwrap_or_default();
    let sr: Option<&str> = if serial_str.is_empty() {
        None
    } else {
        Some(serial_str.as_str())
    };

    let dex_path = get_tool_path(&app, "AppLabelHelper.dex");

    if dex_path.exists() {
        let dex_str = dex_path.to_string_lossy().into_owned();
        let push_args = [
            "push",
            dex_str.as_str(),
            "/data/local/tmp/AppLabelHelper.dex",
        ];
        let _ = adb_run_async_with_serial(&paths.adb, sr, &push_args).await;
    }

    let filter_arg = if only_third_party { "-3" } else { "-s" };

    // 手动构建命令，支持 serial 注入
    let mut cmd = create_hidden_async_command(&paths.adb);
    if let Some(s) = sr {
        cmd.args(["-s", s]);
    }
    cmd.args([
        "shell",
        "app_process",
        "-Djava.class.path=/data/local/tmp/AppLabelHelper.dex",
        "/data/local/tmp",
        "Main",
        filter_arg,
    ]);

    let output = output_tracked_async_command(&mut cmd, PROCESS_KIND_ADB_CLIENT)
        .await
        .map_err(|e| format!("执行 app_process 失败: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if stdout.trim().is_empty()
        && (stderr.contains("Error") || stderr.contains("Exception") || stderr.contains("Failed"))
    {
        return Ok(vec![]);
    }

    let results: Vec<AppLabelEntry> = serde_json::from_str(stdout.trim())
        .map_err(|e| format!("解析应用列表 JSON 失败: {}, 原始输出: {}", e, stdout))?;

    Ok(results)
}

// ==============================
// 提取 APK 到本地（自动处理 Split APK → .apks）
// ==============================
#[tauri::command]
pub async fn extract_apk(
    paths: State<'_, AppPaths>,
    serial: Option<String>,
    package_name: String,
    save_path: String,
) -> Result<String, String> {
    let serial_str = serial.clone().unwrap_or_default();
    let sr: Option<&str> = if serial_str.is_empty() {
        None
    } else {
        Some(serial_str.as_str())
    };

    let pm_output =
        adb_run_async_with_serial(&paths.adb, sr, &["shell", "pm", "path", &package_name])
            .await
            .map_err(|e| format!("获取 APK 路径失败: {}", e))?;

    let device_paths: Vec<String> = pm_output
        .lines()
        .filter_map(|line| {
            let s = line.trim();
            if s.starts_with("package:") {
                Some(s.replace("package:", "").trim().to_string())
            } else {
                None
            }
        })
        .collect();

    if device_paths.is_empty() {
        return Err(format!("未找到包 {} 的 APK 路径", package_name));
    }

    let actual_save_path = if device_paths.len() == 1 {
        if save_path.ends_with(".apks") {
            save_path.trim_end_matches(".apks").to_string() + ".apk"
        } else {
            save_path.clone()
        }
    } else {
        if save_path.ends_with(".apk") && !save_path.ends_with(".apks") {
            save_path.trim_end_matches(".apk").to_string() + ".apks"
        } else {
            save_path.clone()
        }
    };

    if device_paths.len() == 1 {
        adb_pull_to_local_file(
            &paths.adb,
            sr,
            &device_paths[0],
            &PathBuf::from(&actual_save_path),
        )
        .await
        .map_err(|e| format!("提取 APK 失败: {}", e))?;
        return Ok(actual_save_path);
    }

    let tmp_dir = std::env::temp_dir().join(format!("apk_extract_{}", &package_name));
    std::fs::create_dir_all(&tmp_dir).map_err(|e| format!("创建临时目录失败: {}", e))?;

    let mut pulled_files: Vec<(String, std::path::PathBuf)> = Vec::new();

    for device_path in &device_paths {
        let file_name = device_path
            .split('/')
            .last()
            .unwrap_or("split.apk")
            .to_string();
        let local_path = tmp_dir.join(&file_name);

        adb_pull_to_local_file(&paths.adb, sr, device_path, &local_path)
            .await
            .map_err(|e| format!("拉取 {} 失败: {}", file_name, e))?;

        pulled_files.push((file_name, local_path));
    }

    {
        let apks_file = std::fs::File::create(&actual_save_path)
            .map_err(|e| format!("创建 .apks 文件失败: {}", e))?;
        let mut zip = zip::ZipWriter::new(apks_file);
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);

        for (file_name, local_path) in &pulled_files {
            let data =
                std::fs::read(local_path).map_err(|e| format!("读取 {} 失败: {}", file_name, e))?;
            zip.start_file(file_name, options)
                .map_err(|e| format!("写入 ZIP 条目失败: {}", e))?;
            use std::io::Write;
            zip.write_all(&data)
                .map_err(|e| format!("写入 ZIP 数据失败: {}", e))?;
        }

        zip.finish()
            .map_err(|e| format!("ZIP 文件收尾失败: {}", e))?;
    }

    let _ = std::fs::remove_dir_all(&tmp_dir);

    Ok(actual_save_path)
}
