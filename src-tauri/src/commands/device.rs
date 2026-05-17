use crate::adb::core::{
    adb_run_async, adb_run_async_with_serial, adb_run_async_with_serial_combined,
    detect_device_state, dump_fastboot_vars_async_with_serial, dump_props_async_with_serial,
    fastboot_run_async_with_serial, list_fastboot_devices, prop, prop_or,
    scan_connected_devices_with_report, AppPaths, DetectedDevice,
    DeviceScanReport as CoreDeviceScanReport,
};
use crate::models::device_info::{
    DeviceBasicInfo, DeviceEntry, DeviceScanReport, DeviceScanSnapshot, MdnsDevice, PartitionEntry,
    PropEntry, ResourceInfo,
};
use crate::utils::parse::{parse_battery_field, parse_df, parse_meminfo, parse_uptime, round2};
use tauri::State;

fn map_detected_device(device: DetectedDevice) -> DeviceEntry {
    DeviceEntry {
        serial: device.serial,
        state: device.state,
        source: device.source,
    }
}

fn map_scan_report(report: CoreDeviceScanReport) -> DeviceScanReport {
    DeviceScanReport {
        raw_output: report.raw_output,
        devices: report.devices.into_iter().map(map_detected_device).collect(),
    }
}
// ==============================
// Command: 获取已连接设备列表（无需 serial，用于枚举所有设备）
// ==============================
#[tauri::command]
pub async fn get_connected_devices(
    paths: State<'_, AppPaths>,
) -> Result<DeviceScanSnapshot, String> {
    let snapshot = scan_connected_devices_with_report(&paths.adb, &paths.fastboot).await;
    Ok(DeviceScanSnapshot {
        devices: snapshot.devices.into_iter().map(map_detected_device).collect(),
        adb_report: map_scan_report(snapshot.adb_report),
        fastboot_report: map_scan_report(snapshot.fastboot_report),
        duration_ms: snapshot.duration_ms,
    })
}

// ==============================
// Command: 获取当前设备连接状态
// ==============================
#[tauri::command]
pub async fn get_device_state(
    paths: State<'_, AppPaths>,
    serial: Option<String>,
) -> Result<String, String> {
    detect_device_state(&paths.adb, &paths.fastboot, serial.as_deref()).await
}

// ==============================
// Command: 获取设备基本信息
// ==============================
#[tauri::command]
pub async fn get_device_info(
    paths: State<'_, AppPaths>,
    serial: Option<String>,
) -> Result<DeviceBasicInfo, String> {
    let adb = &paths.adb;
    let serial_str = serial.clone().unwrap_or_default();
    let sr: Option<&str> = if serial_str.is_empty() {
        None
    } else {
        Some(serial_str.as_str())
    };

    let (props, wm_size, wm_density, uptime_raw, kernel) = tokio::join!(
        dump_props_async_with_serial(adb, sr),
        async {
            adb_run_async_with_serial(adb, sr, &["shell", "wm", "size"])
                .await
                .unwrap_or_default()
        },
        async {
            adb_run_async_with_serial(adb, sr, &["shell", "wm", "density"])
                .await
                .unwrap_or_default()
        },
        async {
            adb_run_async_with_serial(adb, sr, &["shell", "cat", "/proc/uptime"])
                .await
                .unwrap_or_default()
        },
        async {
            adb_run_async_with_serial(adb, sr, &["shell", "uname", "-r"])
                .await
                .unwrap_or_default()
        },
    );

    if props.is_empty() {
        // 检测 fastboot 设备
        let fb_devices = list_fastboot_devices(&paths.fastboot).await;
        let fb_match = if let Some(s) = sr {
            fb_devices.iter().any(|device| device.serial == s)
        } else {
            !fb_devices.is_empty()
        };

        if !fb_match {
            return Err("没有已连接的设备或没有授权".to_string());
        }

        let fb_props = dump_fastboot_vars_async_with_serial(&paths.fastboot, sr).await;
        if fb_props.is_empty() {
            return Err("获取 Fastboot 信息失败".to_string());
        }

        let is_fastbootd = prop(&fb_props, "is-userspace").to_lowercase() == "yes";
        let state_str = if is_fastbootd {
            "Device (FastbootD)".to_string()
        } else {
            "Device (Fastboot)".to_string()
        };
        let hw_platform = prop_or(&fb_props, &["hardware", "product"]);

        let mut unlocked_raw = prop(&fb_props, "unlocked").to_lowercase();
        if unlocked_raw == "--" || unlocked_raw.is_empty() {
            let secure = prop(&fb_props, "secure").to_lowercase();
            if secure == "no" {
                unlocked_raw = "yes".to_string();
            } else if secure == "yes" {
                unlocked_raw = "no".to_string();
            }
        }

        let unlock_state = match unlocked_raw.as_str() {
            "yes" => "unlocked".to_string(),
            "no" => "locked".to_string(),
            other if !other.is_empty() && other != "--" => other.to_string(),
            _ => "unknown".to_string(),
        };

        let slot = prop(&fb_props, "current-slot").to_uppercase();
        let ab_slot = if slot.is_empty() || slot == "--" {
            "--".to_string()
        } else {
            format!("{} (当前)", slot)
        };

        return Ok(DeviceBasicInfo {
            device_name: prop(&fb_props, "product").to_string(),
            device_codename: prop(&fb_props, "product").to_string(),
            serial: prop(&fb_props, "serialno").to_string(),
            state: state_str,
            brand: "--".to_string(),
            android_version: "--".to_string(),
            os_version: "未知 (Fastboot)".to_string(),
            cpu_codename: "--".to_string(),
            cpu_arch: "--".to_string(),
            hardware_platform: hw_platform,
            board_id: prop(&fb_props, "serialno").to_string(),
            resolution: "--".to_string(),
            display_density: "--".to_string(),
            unlock_state,
            ab_slot,
            vndk_version: "--".to_string(),
            uptime: "--".to_string(),
            build_date: "--".to_string(),
            build_version: prop(&fb_props, "version-bootloader").to_string(),
            fingerprint: "--".to_string(),
            kernel_version: "--".to_string(),
            manufacturer: "--".to_string(),
            product_model: "--".to_string(),
            product_name: prop(&fb_props, "product").to_string(),
            security_patch: "--".to_string(),
            vendor_security_patch: "--".to_string(),
            build_incremental: "--".to_string(),
            build_type: "--".to_string(),
            build_tags: "--".to_string(),
            baseband_version: "--".to_string(),
            soc_manufacturer: "--".to_string(),
            soc_model: "--".to_string(),
            cpu_abilist: "--".to_string(),
        });
    }

    let android_ver = prop(&props, "ro.build.version.release").to_string();
    let sdk = prop(&props, "ro.build.version.sdk").to_string();
    let os_version = format!("Android {}({})", android_ver, sdk);
    let hw_platform = prop_or(&props, &["ro.hardware", "ro.board.platform"]);

    let vb_state = prop(&props, "ro.boot.vbmeta.device_state").to_lowercase();
    let flash_locked = prop(&props, "ro.boot.flash.locked");
    let verified_boot = prop(&props, "ro.boot.verifiedbootstate").to_lowercase();

    let unlock_state = if !vb_state.is_empty() && vb_state != "--" {
        if vb_state == "unlocked" {
            "unlocked".to_string()
        } else {
            "locked".to_string()
        }
    } else if !flash_locked.is_empty() && flash_locked != "--" {
        if flash_locked == "0" {
            "unlocked".to_string()
        } else {
            "locked".to_string()
        }
    } else if !verified_boot.is_empty() && verified_boot != "--" {
        match verified_boot.as_str() {
            "orange" | "yellow" => "unlocked".to_string(),
            "green" => "locked".to_string(),
            _ => "unknown".to_string(),
        }
    } else {
        "unknown".to_string()
    };

    let slot = prop(&props, "ro.boot.slot_suffix")
        .trim_start_matches('_')
        .to_uppercase();
    let ab_slot = if slot.is_empty() {
        "--".to_string()
    } else {
        format!("{} (当前)", slot)
    };

    let resolution = wm_size
        .lines()
        .find(|l| l.contains("Physical size") || l.contains("Override size"))
        .and_then(|l| l.split(':').nth(1))
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "--".to_string());

    let display_density = wm_density
        .lines()
        .find(|l| l.contains("Physical density") || l.contains("Override density"))
        .and_then(|l| l.split(':').nth(1))
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "--".to_string());

    let is_wireless =
        serial_str.contains('.') || serial_str.contains(':') || serial_str.starts_with("adb-");
    let state_display = if is_wireless {
        "Device (Wireless)".to_string()
    } else {
        "Device (Usb)".to_string()
    };

    Ok(DeviceBasicInfo {
        device_name: prop_or(&props, &["ro.product.marketname", "ro.product.model"]),
        device_codename: prop(&props, "ro.product.device").to_string(),
        serial: prop_or(&props, &["ro.serialno", "ro.boot.serialno"]),
        state: state_display,
        brand: prop(&props, "ro.product.brand").to_string(),
        android_version: android_ver,
        os_version,
        cpu_codename: prop(&props, "ro.board.platform").to_string(),
        cpu_arch: prop_or(&props, &["ro.product.cpu.abi"]),
        hardware_platform: hw_platform,
        board_id: prop_or(&props, &["ro.serialno", "ro.boot.serialno"]),
        resolution,
        display_density,
        unlock_state,
        ab_slot,
        vndk_version: prop(&props, "ro.vndk.version").to_string(),
        uptime: parse_uptime(&uptime_raw),
        build_date: prop(&props, "ro.build.date").to_string(),
        build_version: prop(&props, "ro.build.display.id").to_string(),
        fingerprint: prop(&props, "ro.build.fingerprint").to_string(),
        kernel_version: kernel.trim().to_string(),
        manufacturer: prop(&props, "ro.product.manufacturer").to_string(),
        product_model: prop(&props, "ro.product.model").to_string(),
        product_name: prop(&props, "ro.product.name").to_string(),
        security_patch: prop(&props, "ro.build.version.security_patch").to_string(),
        vendor_security_patch: prop(&props, "ro.vendor.build.security_patch").to_string(),
        build_incremental: prop(&props, "ro.build.version.incremental").to_string(),
        build_type: prop(&props, "ro.build.type").to_string(),
        build_tags: prop(&props, "ro.build.tags").to_string(),
        baseband_version: prop(&props, "gsm.version.baseband").to_string(),
        soc_manufacturer: prop(&props, "ro.soc.manufacturer").to_string(),
        soc_model: prop(&props, "ro.soc.model").to_string(),
        cpu_abilist: prop(&props, "ro.product.cpu.abilist").to_string(),
    })
}

// ==============================
// Command: 获取资源监控数据
// ==============================
#[tauri::command]
pub async fn get_device_resources(
    paths: State<'_, AppPaths>,
    serial: Option<String>,
) -> Result<ResourceInfo, String> {
    let adb = &paths.adb;
    let serial_str = serial.clone().unwrap_or_default();
    let sr: Option<&str> = if serial_str.is_empty() {
        None
    } else {
        Some(serial_str.as_str())
    };

    let (df_out, meminfo, bat_out) = tokio::join!(
        async {
            adb_run_async_with_serial(adb, sr, &["shell", "df", "/data"])
                .await
                .unwrap_or_default()
        },
        async {
            adb_run_async_with_serial(adb, sr, &["shell", "cat", "/proc/meminfo"])
                .await
                .unwrap_or_default()
        },
        async {
            adb_run_async_with_serial(adb, sr, &["shell", "dumpsys", "battery"])
                .await
                .unwrap_or_default()
        }
    );

    let (stor_total, stor_used) = parse_df(&df_out);
    let (mem_total, mem_avail) = parse_meminfo(&meminfo);
    let mem_used = mem_total.saturating_sub(mem_avail);

    let bat_level = parse_battery_field(&bat_out, "level").unwrap_or(0) as u32;
    let bat_temp = parse_battery_field(&bat_out, "temperature").unwrap_or(0) as f64 / 10.0;

    let st = stor_total as f64 / 1024.0 / 1024.0;
    let su = stor_used as f64 / 1024.0 / 1024.0;
    let mt = mem_total as f64 / 1024.0 / 1024.0;
    let mu = mem_used as f64 / 1024.0 / 1024.0;

    Ok(ResourceInfo {
        storage_used_gb: round2(su),
        storage_total_gb: round2(st),
        storage_percent: if st > 0.0 {
            ((su / st) * 100.0).round() as u32
        } else {
            0
        },
        memory_used_gb: round2(mu),
        memory_total_gb: round2(mt),
        memory_percent: if mt > 0.0 {
            ((mu / mt) * 100.0).round() as u32
        } else {
            0
        },
        battery_level: bat_level,
        battery_temp: bat_temp,
    })
}

// ==============================
// Command: 获取分区信息
// ==============================
#[tauri::command]
pub async fn get_partition_info(
    paths: State<'_, AppPaths>,
    serial: Option<String>,
) -> Result<Vec<PartitionEntry>, String> {
    let serial_str = serial.clone().unwrap_or_default();
    let sr: Option<&str> = if serial_str.is_empty() {
        None
    } else {
        Some(serial_str.as_str())
    };
    let state = detect_device_state(&paths.adb, &paths.fastboot, sr).await?;

    if state == "device" {
        let output = adb_run_async_with_serial(
            &paths.adb,
            sr,
            &["shell", "ls", "-la", "/dev/block/by-name/"],
        )
        .await
        .map_err(|e| e)?;

        let mut partitions: Vec<PartitionEntry> = output
            .lines()
            .filter(|l| l.contains(" -> "))
            .filter_map(|line| {
                let arrow = line.find(" -> ")?;
                let target = line[arrow + 4..].trim().to_string();
                let name = line[..arrow].split_whitespace().last()?.to_string();
                if name.is_empty() {
                    return None;
                }
                Some(PartitionEntry {
                    name,
                    block_device: target,
                })
            })
            .collect();

        partitions.sort_by(|a, b| a.name.cmp(&b.name));
        return Ok(partitions);
    }

    if state == "fastboot" || state == "fastbootd" {
        let output = fastboot_run_async_with_serial(&paths.fastboot, sr, &["getvar", "all"])
            .await
            .unwrap_or_default();

        let mut partitions: Vec<PartitionEntry> = Vec::new();
        let mut seen = std::collections::HashSet::<String>::new();

        for line in output.lines() {
            let line = line.trim();
            let clean_line = if let Some(stripped) = line.strip_prefix("(bootloader) ") {
                stripped.trim()
            } else {
                line
            };

            if clean_line.starts_with("partition-size:")
                || clean_line.starts_with("partition-type:")
            {
                if let Some(first_colon) = clean_line.find(':') {
                    let rest = &clean_line[first_colon + 1..].trim();
                    let name = if let Some(second_colon) = rest.find(':') {
                        rest[..second_colon].trim().to_string()
                    } else {
                        rest.split_whitespace().next().unwrap_or("").to_string()
                    };

                    if !name.is_empty() && name != "all" && !seen.contains(&name) {
                        partitions.push(PartitionEntry {
                            name: name.clone(),
                            block_device: "fastboot".to_string(),
                        });
                        seen.insert(name);
                    }
                }
            }
        }

        partitions.sort_by(|a, b| a.name.cmp(&b.name));
        return Ok(partitions);
    }

    Err("未检测到有效连接的设备 (ADB 或 Fastboot)".to_string())
}

// ==============================
// Command: 发送 ADB Keyevent
// ==============================
#[tauri::command]
pub async fn adb_send_keyevent(
    paths: State<'_, AppPaths>,
    serial: Option<String>,
    keycode: String,
) -> Result<(), String> {
    let serial_str = serial.unwrap_or_default();
    let sr: Option<&str> = if serial_str.is_empty() {
        None
    } else {
        Some(serial_str.as_str())
    };
    adb_run_async_with_serial(&paths.adb, sr, &["shell", "input", "keyevent", &keycode])
        .await
        .map(|_| ())
        .map_err(|e| format!("发送按键失败: {}", e))
}

// ==============================
// Command: 获取应用状态
// ==============================
#[tauri::command]
pub async fn get_app_status(
    paths: State<'_, AppPaths>,
    serial: Option<String>,
) -> Result<crate::models::device_info::AppStatus, String> {
    let adb = &paths.adb;
    let serial_str = serial.unwrap_or_default();
    let sr: Option<&str> = if serial_str.is_empty() {
        None
    } else {
        Some(serial_str.as_str())
    };

    let (sys_out, user_out) = tokio::join!(
        async {
            adb_run_async_with_serial(adb, sr, &["shell", "pm", "list", "packages", "-s"])
                .await
                .unwrap_or_default()
        },
        async {
            adb_run_async_with_serial(adb, sr, &["shell", "pm", "list", "packages", "-3"])
                .await
                .unwrap_or_default()
        },
    );

    let system_count = sys_out.lines().filter(|l| !l.trim().is_empty()).count() as u32;
    let user_count = user_out.lines().filter(|l| !l.trim().is_empty()).count() as u32;
    let total_count = system_count + user_count;

    Ok(crate::models::device_info::AppStatus {
        system_count,
        user_count,
        total_count,
    })
}

// ==============================
// Command: 执行 ADB 配对 (Pairing)
// ==============================
#[tauri::command]
pub async fn adb_pair(
    paths: State<'_, AppPaths>,
    addr: String,
    code: String,
) -> Result<String, String> {
    // adb pair <ip:port> <code>
    let args = vec!["pair".to_string(), addr, code];
    let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    adb_run_async_with_serial_combined(&paths.adb, None, &args_ref).await
}

// ==============================
// Command: 通用 ADB 命令执行 (非 Shell)
// ==============================
#[tauri::command]
pub async fn run_adb(
    paths: State<'_, AppPaths>,
    serial: Option<String>,
    args: Vec<String>,
) -> Result<String, String> {
    let serial_str = serial.unwrap_or_default();
    let sr: Option<&str> = if serial_str.is_empty() {
        None
    } else {
        Some(serial_str.as_str())
    };
    let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    adb_run_async_with_serial_combined(&paths.adb, sr, &args_ref).await
}

// ==============================
// Command: 通用 Fastboot 命令执行
// ==============================
#[tauri::command]
pub async fn run_fastboot(
    paths: State<'_, AppPaths>,
    serial: Option<String>,
    args: Vec<String>,
) -> Result<String, String> {
    let serial_str = serial.unwrap_or_default();
    let sr: Option<&str> = if serial_str.is_empty() {
        None
    } else {
        Some(serial_str.as_str())
    };
    let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    fastboot_run_async_with_serial(&paths.fastboot, sr, &args_ref).await
}

// ==============================
// Command: 检查宿主机文件是否存在
// ==============================
#[tauri::command]
pub fn check_file_exists(path: String) -> bool {
    std::path::Path::new(&path).exists()
}

// ==============================
// Command: 通用 ADB Shell 命令执行
// ==============================
#[tauri::command]
pub async fn run_adb_shell(
    paths: State<'_, AppPaths>,
    serial: Option<String>,
    args: Vec<String>,
) -> Result<String, String> {
    let serial_str = serial.unwrap_or_default();
    let sr: Option<&str> = if serial_str.is_empty() {
        None
    } else {
        Some(serial_str.as_str())
    };
    let mut full_args = vec!["shell".to_string()];
    full_args.extend(args);
    let args_ref: Vec<&str> = full_args.iter().map(|s| s.as_str()).collect();
    adb_run_async_with_serial(&paths.adb, sr, &args_ref).await
}

// ==============================
// Command: 设备重启相关功能 (自动检测模式)
// ==============================
#[tauri::command]
pub async fn device_reboot(
    paths: State<'_, AppPaths>,
    serial: Option<String>,
    target: String,
) -> Result<String, String> {
    let serial_str = serial.clone().unwrap_or_default();
    let sr: Option<&str> = if serial_str.is_empty() {
        None
    } else {
        Some(serial_str.as_str())
    };
    let state = detect_device_state(&paths.adb, &paths.fastboot, sr).await?;

    if state == "device" {
        match target.as_str() {
            "system" => adb_run_async_with_serial(&paths.adb, sr, &["reboot"]).await,
            "bootloader" => {
                adb_run_async_with_serial(&paths.adb, sr, &["reboot", "bootloader"]).await
            }
            "recovery" => adb_run_async_with_serial(&paths.adb, sr, &["reboot", "recovery"]).await,
            "poweroff" => {
                adb_run_async_with_serial(&paths.adb, sr, &["shell", "reboot", "-p"]).await
            }
            _ => Err(format!("不支持的重启目标: {}", target)),
        }
    } else if state == "fastboot" || state == "fastbootd" {
        match target.as_str() {
            "system" => fastboot_run_async_with_serial(&paths.fastboot, sr, &["reboot"]).await,
            "bootloader" => {
                fastboot_run_async_with_serial(&paths.fastboot, sr, &["reboot", "bootloader"]).await
            }
            "recovery" => {
                fastboot_run_async_with_serial(&paths.fastboot, sr, &["reboot", "recovery"]).await
            }
            "poweroff" => {
                fastboot_run_async_with_serial(&paths.fastboot, sr, &["oem", "poweroff"]).await
            }
            _ => Err(format!("不支持的重启目标: {}", target)),
        }
    } else {
        Err("未检测到有效连接的设备 (ADB 或 Fastboot)".to_string())
    }
}

// ==============================
// Command: 获取系统属性或设置列表
// ==============================
#[tauri::command]
pub async fn get_sys_props(
    paths: State<'_, AppPaths>,
    serial: Option<String>,
    prop_type: String,
) -> Result<Vec<PropEntry>, String> {
    let serial_str = serial.unwrap_or_default();
    let sr: Option<&str> = if serial_str.is_empty() {
        None
    } else {
        Some(serial_str.as_str())
    };

    let args = if prop_type == "getprop" {
        vec!["shell", "getprop"]
    } else {
        vec!["shell", "settings", "list", &prop_type]
    };

    let output = adb_run_async_with_serial(&paths.adb, sr, &args).await?;
    let mut props = Vec::new();

    for line in output.lines() {
        if line.trim().is_empty() {
            continue;
        }

        if prop_type == "getprop" {
            // 格式: [ro.product.model]: [Redmi Note 12]
            if let (Some(start_key), Some(end_key)) = (line.find('['), line.find(']')) {
                let key = &line[start_key + 1..end_key];
                let remainder = &line[end_key + 1..];
                if let (Some(start_val), Some(end_val)) =
                    (remainder.find('['), remainder.rfind(']'))
                {
                    let value = &remainder[start_val + 1..end_val];
                    props.push(PropEntry {
                        key: key.to_string(),
                        value: value.to_string(),
                    });
                }
            }
        } else {
            // 格式: key=value
            if let Some(idx) = line.find('=') {
                let (key, value) = line.split_at(idx);
                props.push(PropEntry {
                    key: key.to_string(),
                    value: value[1..].to_string(),
                });
            }
        }
    }

    props.sort_by(|a, b| a.key.to_lowercase().cmp(&b.key.to_lowercase()));
    Ok(props)
}

// ==============================
// Command: 设置系统属性或设置
// ==============================
#[tauri::command]
pub async fn set_sys_prop(
    paths: State<'_, AppPaths>,
    serial: Option<String>,
    prop_type: String,
    key: String,
    value: String,
) -> Result<(), String> {
    let serial_str = serial.unwrap_or_default();
    let sr: Option<&str> = if serial_str.is_empty() {
        None
    } else {
        Some(serial_str.as_str())
    };

    let args = if prop_type == "getprop" {
        vec!["shell", "setprop", &key, &value]
    } else {
        vec!["shell", "settings", "put", &prop_type, &key, &value]
    };

    adb_run_async_with_serial(&paths.adb, sr, &args)
        .await
        .map(|_| ())
        .map_err(|e| e)
}

// ==============================
// Command: 删除系统设置 (getprop 不支持真删除，只能清空)
// ==============================
#[tauri::command]
pub async fn delete_sys_prop(
    paths: State<'_, AppPaths>,
    serial: Option<String>,
    prop_type: String,
    key: String,
) -> Result<(), String> {
    let serial_str = serial.unwrap_or_default();
    let sr: Option<&str> = if serial_str.is_empty() {
        None
    } else {
        Some(serial_str.as_str())
    };

    let args = if prop_type == "getprop" {
        vec!["shell", "setprop", &key, ""]
    } else {
        vec!["shell", "settings", "delete", &prop_type, &key]
    };

    adb_run_async_with_serial(&paths.adb, sr, &args)
        .await
        .map(|_| ())
        .map_err(|e| e)
}

// ==============================
// Command: 扫描局域网内的无线 ADB 设备 (mDNS)
// ==============================
#[tauri::command]
pub async fn scan_adb_devices(
    paths: State<'_, AppPaths>,
) -> Result<Vec<crate::models::device_info::MdnsDevice>, String> {
    let output = adb_run_async(&paths.adb, &["mdns", "services"])
        .await
        .unwrap_or_default();
    let mut parsed_devices = Vec::new();
    let mut parsed_seen = std::collections::HashSet::new();

    for raw_line in output.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.contains("List of discovered services") {
            continue;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 3 {
            continue;
        }

        let addr = parts[parts.len() - 1];
        let Some(pos) = addr.rfind(':') else {
            continue;
        };

        if !parsed_seen.insert(addr.to_string()) {
            continue;
        }

        let service_type = parts[parts.len() - 2].trim_end_matches('.').to_string();
        let instance_name = parts[..parts.len() - 2]
            .join(" ")
            .trim_end_matches('.')
            .to_string();
        let ip = addr[..pos].to_string();
        let port = addr[pos + 1..].to_string();

        parsed_devices.push(MdnsDevice {
            instance_name,
            service_type,
            ip,
            port,
        });
    }

    return Ok(parsed_devices);
    /*

            // 如果已经处理过这个地址，则跳过（去重）
            if seen.contains(addr) {
                continue;
            }
            seen.insert(addr.to_string());

            let service_type = parts[0].trim_end_matches('.').to_string();
            let full_instance = parts[1].trim_end_matches('.').to_string();
            let instance_name = full_instance.split('.').next().unwrap_or(&full_instance).to_string();

            if let Some(pos) = addr.rfind(':') {
                let ip = addr[..pos].to_string();
                let port = addr[pos+1..].to_string();

                devices.push(crate::models::device_info::MdnsDevice {
                    instance_name,
                    service_type,
                    ip,
                    port,
                });
            }
        }
    }

    Ok(devices)
    */
}
