use super::*;

pub(super) fn get_patch_mode_package_title(patch_mode: &str) -> String {
    format!("{}_一键刷入包", get_patch_mode_label(patch_mode))
}

pub(super) fn get_patch_mode_manager_name(patch_mode: &str) -> &'static str {
    match normalize_patch_mode(patch_mode).as_str() {
        PATCH_MODE_MAGISK_ALPHA => "magisk_Alpha",
        PATCH_MODE_APATCH => "APatch",
        PATCH_MODE_FOLKPATCH => "FolkPatch",
        PATCH_MODE_KERNELSU => "KernelSU",
        PATCH_MODE_KERNELSU_NEXT => "KernelSU_Next",
        PATCH_MODE_SUKISU_ULTRA => "SukiSU_Ultra",
        _ => "Magisk",
    }
}

pub(super) fn get_patch_mode_manager_apk_path(
    patch_mode: &str,
    magisk_apk_path: &Path,
    apatch_apk_path: &Path,
    kernel_su_apk_path: Option<&Path>,
) -> Result<PathBuf, String> {
    let normalized_mode = normalize_patch_mode(patch_mode);
    let candidate = match normalized_mode.as_str() {
        PATCH_MODE_APATCH | PATCH_MODE_FOLKPATCH => apatch_apk_path,
        PATCH_MODE_KERNELSU | PATCH_MODE_KERNELSU_NEXT | PATCH_MODE_SUKISU_ULTRA => {
            kernel_su_apk_path.ok_or_else(|| {
                format!(
                    "{} 管理器 APK 路径为空",
                    get_patch_mode_manager_name(&normalized_mode)
                )
            })?
        }
        _ => magisk_apk_path,
    };

    if !candidate.exists() || !candidate.is_file() {
        return Err(format!(
            "未找到 {} 管理器 APK: {}",
            get_patch_mode_manager_name(&normalized_mode),
            candidate.display()
        ));
    }

    Ok(normalize_local_path(candidate))
}

pub(super) fn get_package_flash_partition(target_partition: &str, slot_suffix: &str) -> String {
    let normalized_partition = normalized_patch_partition_name(target_partition);
    let normalized_slot = normalize_slot_suffix(slot_suffix).unwrap_or_default();
    if normalized_slot.is_empty() {
        normalized_partition
    } else {
        format!("{}{}", normalized_partition, normalized_slot)
    }
}

pub(super) fn get_package_root_dir_name(
    patch_mode: &str,
    package_device_info: &PatchPackageDeviceInfo,
) -> String {
    let mode = sanitize_file_name_component(get_patch_mode_manager_name(patch_mode));
    let model = if package_device_info.model.trim().is_empty() {
        "Null".to_string()
    } else {
        sanitize_file_name_component(&package_device_info.model)
    };
    let codename = if package_device_info.codename.trim().is_empty() {
        "Null".to_string()
    } else {
        sanitize_file_name_component(&package_device_info.codename)
    };
    let build_version = if package_device_info.build_version.trim().is_empty() {
        "Null".to_string()
    } else {
        sanitize_file_name_component(&package_device_info.build_version)
    };
    format!("{}_{}_{}_{}", mode, model, codename, build_version)
}

pub(super) fn build_flash_package_readme_resolved(
    patch_mode: &str,
    package_device_info: &PatchPackageDeviceInfo,
    target_partition: &str,
    target_slot_suffix: &str,
) -> String {
    let manager_name = get_patch_mode_manager_name(patch_mode);
    let flash_partition = get_package_flash_partition(target_partition, target_slot_suffix);
    let model_value = if package_device_info.model.trim().is_empty() {
        "Null".to_string()
    } else {
        package_device_info.model.clone()
    };
    let codename_value = if package_device_info.codename.trim().is_empty() {
        "Null".to_string()
    } else {
        package_device_info.codename.clone()
    };
    let version_value = if package_device_info.build_version.trim().is_empty() {
        "Null".to_string()
    } else {
        package_device_info.build_version.clone()
    };
    let android_value = if package_device_info.android_version.trim().is_empty() {
        "Null".to_string()
    } else {
        package_device_info.android_version.clone()
    };
    let oem_value = if package_device_info.oem_name.trim().is_empty() {
        "Null".to_string()
    } else {
        package_device_info.oem_name.clone()
    };

    format!(
        "资料包适用信息：\r\n\
机型：{model_value}\r\n\
代号：{codename_value}\r\n\
完整版本：{version_value}\r\n\
安卓版本：{android_value}\r\n\
厂商：{oem_value}\r\n\
目标分区：{flash_partition}\r\n\
Root 方案：{manager_name}\r\n\
\r\n\
注意事项：\r\n\
1. 本说明中的机型、代号、完整版本、安卓版本与厂商信息均优先来自本次被修补的 boot / init_boot 镜像；镜像内未解析到的字段会显示为 Null。\r\n\
2. 刷入前请先确认 BootLoader 已解锁，并提前备份重要数据。\r\n\
3. 刷入前请先安装本资料包内对应的 {manager_name} 管理器，并卸载设备中已安装的旧版同类管理器，避免安装或校验冲突。\r\n\
4. 若设备为 A/B 或 VAB 分区结构，脚本会优先检测当前活动槽位，并自动拼接目标分区后缀。\r\n\
5. 若自动识别失败，可手动编辑脚本中的分区变量后再执行。\r\n\
\r\n\
使用步骤：\r\n\
1. 保持设备通过 USB 连接电脑。\r\n\
2. 如果设备当前处于系统内，双击资料包中的 bat 脚本，按提示重启到 bootloader。\r\n\
3. 如果设备已经处于 fastboot / fastbootd，也可以直接运行该脚本继续操作。\r\n\
4. 按脚本提示选择要刷入修补镜像还是原版镜像；输入 1 刷入修补镜像，输入其它任意内容则刷回原版镜像。\r\n\
5. 刷入完成后，脚本会自动重启设备。\r\n"
    )
}

pub(super) fn build_flash_package_bat(
    patch_mode: &str,
    package_title: &str,
    package_device_info: &PatchPackageDeviceInfo,
    target_partition: &str,
    patched_image_name: &str,
    original_image_name: &str,
) -> String {
    let manager_name = get_patch_mode_manager_name(patch_mode);
    let title_suffix = if package_device_info.model.trim().is_empty() {
        package_title.to_string()
    } else {
        format!("{}_{}", package_device_info.model, package_title)
    };
    let title_value = sanitize_file_name_component(&title_suffix);
    format!("
@echo off\r\n\
cd /d %~dp0\r\n\
setlocal EnableExtensions\r\n\
set \"fastboot=platform-tools\\fastboot.exe\"\r\n\
set \"adb=platform-tools\\adb.exe\"\r\n\
set \"target_partition={target_partition}\"\r\n\
set \"patched_image=firmware-update\\{patched_image_name}\"\r\n\
set \"stock_image=firmware-update\\{original_image_name}\"\r\n\
title {title_value}\r\n\
echo.\r\n\
\r\n\
:check_adb\r\n\
cls\r\n\
set \"adb_state=\"\r\n\
for /f \"delims=\" %%t in ('%adb% %* get-state 2^>nul^') do set \"adb_state=%%t\"\r\n\
if /i \"%adb_state%\"==\"device\" goto read_adb_props\r\n\
goto check_fastboot\r\n\
\r\n\
:read_adb_props\r\n\
set version=\r\n\
set model=\r\n\
for /f \"delims=\" %%t in ('%adb% shell getprop ro.build.version.release 2^>nul^') do set \"version=%%t\"\r\n\
for /f \"delims=\" %%t in ('%adb% shell getprop ro.product.model 2^>nul^') do set \"model=%%t\"\r\n\
cls\r\n\
echo.\r\n\
if defined model goto adb_connected\r\n\
goto check_fastboot\r\n\
\r\n\
:adb_connected\r\n\
echo. -- [%time:~0,8%] 设备已在 adb 模式下连接\r\n\
echo. -- [%time:~0,8%] 机型:%model%\r\n\
echo. -- [%time:~0,8%] 安卓版本:%version%\r\n\
echo.\r\n\
%adb% %* shell getprop ro.boot.flash.locked 2>&1 | findstr /r /c:\"^0\" >nul 2>nul  2>&1 && %adb% %* shell getprop ro.boot.verifiedbootstate 2>&1 | findstr /r /c:\"^orange\" >nul 2>nul && echo  -- [%time:~0,8%] 解锁状态:设备已解锁 || echo  -- [%time:~0,8%] 解锁状态:设备未解锁\r\n\
%adb% %* shell getprop ro.boot.slot_suffix 2>&1 | findstr /r /c:\"^_\" >nul 2>nul && echo  -- [%time:~0,8%] 设备分区:ab/Vab || echo  -- [%time:~0,8%] 设备分区:Aonly\r\n\
echo.\r\n\
echo. -- [%time:~0,8%] 按任意键重启到 bootloader 继续刷入\r\n\
echo.\r\n\
pause >nul\r\n\
cls\r\n\
echo.\r\n\
echo. -- [%time:~0,8%] 正在重启到 bootloader\r\n\
%adb% reboot bootloader >nul 2>nul\r\n\
goto check_fastboot\r\n\
\r\n\
:check_fastboot\r\n\
set \"dev=\"\r\n\
for /f \"delims=\" %%t in ('%fastboot% %* devices 2^>nul^') do set \"dev=%%t\"\r\n\
if not defined dev goto no_fastboot_device\r\n\
echo. -- [%time:~0,8%] 设备已在 fastboot 模式下连接\r\n\
echo. -- [%time:~0,8%] 设备信息:%dev%\r\n\
echo.\r\n\
%fastboot% %* getvar unlocked 2>&1 | findstr /r /c:\"^unlocked: yes\" >nul 2>nul && echo  -- [%time:~0,8%] 解锁状态:设备已解锁 || echo  -- [%time:~0,8%] 解锁状态:设备未解锁\r\n\
%fastboot% %* getvar current-slot 2>&1 | findstr /r /c:\"^current\" >nul 2>nul && echo  -- [%time:~0,8%] 设备分区:ab/Vab || echo  -- [%time:~0,8%] 设备分区:Aonly\r\n\
goto confirm_flash\r\n\
\r\n\
:no_fastboot_device\r\n\
cls\r\n\
echo.\r\n\
echo. -- [%time:~0,8%] 当前未检测到 ADB / Fastboot 设备\r\n\
echo. -- [%time:~0,8%] 请确认 USB 连接、驱动、调试授权或手动进入 BootLoader\r\n\
echo. -- [%time:~0,8%] 未检测到已连接设备，即将自动刷新\r\n\
echo.\r\n\
timeout /t 2 /nobreak >nul\r\n\
goto check_adb\r\n\
\r\n\
:confirm_flash\r\n\
echo.\r\n\
echo. -- [%time:~0,8%] 按任意键开始刷入 {manager_name} 修补镜像\r\n\
echo.\r\n\
pause >nul\r\n\
cls\r\n\
echo.\r\n\
set \"slot=\"\r\n\
%fastboot% %* getvar unlocked 2>&1 | findstr /r /c:\"^unlocked: yes\" >nul 2>nul && echo device:unlocked || echo device:locked\r\n\
%fastboot% %* getvar current-slot 2>&1 | findstr /r /c:\"^current-slot: a\" >nul 2>nul && set \"slot=_a\"\r\n\
if not defined slot %fastboot% %* getvar current-slot 2>&1 | findstr /r /c:\"^current-slot: b\" >nul 2>nul && set \"slot=_b\"\r\n\
if defined slot (\r\n\
echo.slot:%slot%\r\n\
) else (\r\n\
echo.slot:A-only\r\n\
)\r\n\
set /p flash_choice=Need {manager_name} Root? 输入 1 刷入修补镜像，其它值刷入原版镜像:\r\n\
if /i \"%flash_choice%\"==\"1\" (\r\n\
  set \"flash_image=%patched_image%\"\r\n\
) else (\r\n\
  set \"flash_image=%stock_image%\"\r\n\
)\r\n\
\r\n\
set \"flash_partition=%target_partition%\"\r\n\
if defined slot (\r\n\
  if /i \"%target_partition%\"==\"boot\" set \"flash_partition=boot%slot%\"\r\n\
  if /i \"%target_partition%\"==\"init_boot\" set \"flash_partition=init_boot%slot%\"\r\n\
)\r\n\
\r\n\
echo. -- [%time:~0,8%] 即将刷入分区: %flash_partition%\r\n\
echo. -- [%time:~0,8%] 先尝试在 Fastboot 模式下刷入\r\n\
goto flash_fastboot\r\n\
\r\n\
:flash_fastboot\r\n\
%fastboot% %* flash %flash_partition% %flash_image%\r\n\
if not errorlevel 1 goto flash_success\r\n\
echo.\r\n\
echo. -- [%time:~0,8%] Fastboot 模式刷入失败，准备重启到 FastbootD 重试\r\n\
%fastboot% %* reboot fastboot\r\n\
if errorlevel 1 (\r\n\
  echo.\r\n\
  echo. -- [%time:~0,8%] 重启到 FastbootD 失败，请检查上方日志\r\n\
  pause\r\n\
  exit /b 1\r\n\
)\r\n\
\r\n\
\r\n\
:wait_fastbootd\r\n\
%fastboot% %* getvar is-userspace 2>&1 | findstr /i /c:\"is-userspace: yes\" >nul 2>nul && goto flash_fastbootd\r\n\
timeout /t 1 /nobreak >nul\r\n\
goto wait_fastbootd\r\n\
\r\n\
:flash_fastbootd\r\n\
echo.\r\n\
echo. -- [%time:~0,8%] 已进入 FastbootD，开始再次刷入\r\n\
%fastboot% %* flash %flash_partition% %flash_image%\r\n\
if errorlevel 1 (\r\n\
  echo.\r\n\
  echo. -- [%time:~0,8%] FastbootD 模式刷入仍然失败，请检查上方日志\r\n\
  pause\r\n\
  exit /b 1\r\n\
)\r\n\
\r\n\
:flash_success\r\n\
echo.\r\n\
echo. -- [%time:~0,8%] 刷入完成，准备重启设备\r\n\
%fastboot% %* reboot\r\n\
pause\r\n",
    )
}

pub(super) fn write_text_file(path: &Path, content: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("创建目录失败 {}: {}", parent.display(), e))?;
    }
    fs::write(path, content).map_err(|e| format!("写入文件失败 {}: {}", path.display(), e))
}

#[cfg(target_os = "windows")]
pub(super) fn encode_text_to_ansi_bytes(content: &str) -> Result<Vec<u8>, String> {
    let wide: Vec<u16> = content.encode_utf16().collect();
    if wide.is_empty() {
        return Ok(Vec::new());
    }

    let code_page = unsafe { GetACP() };
    let required = unsafe { WideCharToMultiByte(code_page, 0, &wide, None, None, None) };

    if required <= 0 {
        return Err("将文本转换为 ANSI 编码失败".to_string());
    }

    let mut buffer = vec![0u8; required as usize];
    let written =
        unsafe { WideCharToMultiByte(code_page, 0, &wide, Some(&mut buffer), None, None) };

    if written <= 0 {
        return Err("写入 ANSI 编码缓冲区失败".to_string());
    }

    Ok(buffer)
}

#[cfg(not(target_os = "windows"))]
pub(super) fn encode_text_to_ansi_bytes(content: &str) -> Result<Vec<u8>, String> {
    Ok(content.as_bytes().to_vec())
}

pub(super) fn write_ansi_text_file(path: &Path, content: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("创建目录失败 {}: {}", parent.display(), e))?;
    }

    let data = encode_text_to_ansi_bytes(content)?;
    fs::write(path, data).map_err(|e| format!("写入 ANSI 文件失败 {}: {}", path.display(), e))
}

pub(super) fn copy_dir_recursive(source: &Path, target: &Path) -> Result<(), String> {
    if !source.exists() {
        return Err(format!("目录不存在: {}", source.display()));
    }

    fs::create_dir_all(target).map_err(|e| format!("创建目录失败 {}: {}", target.display(), e))?;
    let entries =
        fs::read_dir(source).map_err(|e| format!("读取目录失败 {}: {}", source.display(), e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("读取目录项失败: {}", e))?;
        let source_path = entry.path();
        let target_path = target.join(entry.file_name());
        if source_path.is_dir() {
            copy_dir_recursive(&source_path, &target_path)?;
        } else {
            fs::copy(&source_path, &target_path).map_err(|e| {
                format!(
                    "复制文件失败 {} -> {}: {}",
                    source_path.display(),
                    target_path.display(),
                    e
                )
            })?;
        }
    }

    Ok(())
}

#[allow(dead_code)]
pub(super) fn add_file_to_zip(
    zip: &mut ZipWriter<fs::File>,
    base_dir: &Path,
    file_path: &Path,
    options: zip::write::SimpleFileOptions,
) -> Result<(), String> {
    let relative_path = file_path
        .strip_prefix(base_dir)
        .map_err(|e| format!("计算 ZIP 相对路径失败 {}: {}", file_path.display(), e))?;
    let zip_entry_name = to_unix_path(relative_path);
    zip.start_file(&zip_entry_name, options)
        .map_err(|e| format!("写入 ZIP 条目失败 {}: {}", zip_entry_name, e))?;
    let data =
        fs::read(file_path).map_err(|e| format!("读取文件失败 {}: {}", file_path.display(), e))?;
    zip.write_all(&data)
        .map_err(|e| format!("写入 ZIP 数据失败 {}: {}", file_path.display(), e))?;
    Ok(())
}

#[allow(dead_code)]
pub(super) fn add_dir_to_zip(
    zip: &mut ZipWriter<fs::File>,
    base_dir: &Path,
    current_dir: &Path,
    options: zip::write::SimpleFileOptions,
) -> Result<(), String> {
    let entries = fs::read_dir(current_dir)
        .map_err(|e| format!("读取目录失败 {}: {}", current_dir.display(), e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("读取目录项失败: {}", e))?;
        let path = entry.path();
        if path.is_dir() {
            add_dir_to_zip(zip, base_dir, &path, options)?;
        } else {
            add_file_to_zip(zip, base_dir, &path, options)?;
        }
    }

    Ok(())
}

pub(super) fn build_flash_package_zip(
    window: &Window,
    output_dir: &Path,
    patch_mode: &str,
    manager_apk_path: &Path,
    original_boot_path: &Path,
    patched_image_path: &Path,
    target_partition: &str,
    target_slot_suffix: &str,
    package_device_info: &PatchPackageDeviceInfo,
) -> Result<(PathBuf, String), String> {
    let package_root_name = get_package_root_dir_name(patch_mode, package_device_info);
    let temp_root = output_dir.join(format!(
        "{}_package_{}",
        package_root_name,
        chrono_like_timestamp()
    ));
    let package_dir = temp_root.join(&package_root_name);
    let firmware_dir = package_dir.join("firmware-update");
    let platform_tools_dir = package_dir.join("platform-tools");

    recreate_local_dir(&temp_root)?;
    fs::create_dir_all(&firmware_dir)
        .map_err(|e| format!("创建 firmware-update 目录失败: {}", e))?;

    let manager_apk_name = path_file_name_string(manager_apk_path);
    let original_image_name = path_file_name_string(original_boot_path);
    let patched_image_name = path_file_name_string(patched_image_path);

    fs::copy(manager_apk_path, package_dir.join(&manager_apk_name))
        .map_err(|e| format!("复制管理器 APK 失败: {}", e))?;
    fs::copy(original_boot_path, firmware_dir.join(&original_image_name))
        .map_err(|e| format!("复制原版镜像失败: {}", e))?;
    fs::copy(patched_image_path, firmware_dir.join(&patched_image_name))
        .map_err(|e| format!("复制修补镜像失败: {}", e))?;

    let platform_tools_source = get_bin_root_dir(&window.app_handle()).join("platform-tools");
    copy_dir_recursive(&platform_tools_source, &platform_tools_dir)?;

    let package_title = get_patch_mode_package_title(patch_mode);
    let readme_content = build_flash_package_readme_resolved(
        patch_mode,
        package_device_info,
        target_partition,
        target_slot_suffix,
    );
    let bat_content = build_flash_package_bat(
        patch_mode,
        &package_title,
        package_device_info,
        target_partition,
        &patched_image_name,
        &original_image_name,
    );

    write_text_file(
        &package_dir.join(FLASH_PACKAGE_README_FILE_NAME),
        &readme_content,
    )?;
    write_ansi_text_file(&package_dir.join(FLASH_PACKAGE_BAT_FILE_NAME), &bat_content)?;

    let archive_file_name = format!("{}.7z", package_root_name);
    let archive_path = output_dir.join(&archive_file_name);
    if archive_path.exists() {
        fs::remove_file(&archive_path)
            .map_err(|e| format!("删除旧 7z 失败 {}: {}", archive_path.display(), e))?;
    }

    let seven_zip_path = get_local_7z_path(window);
    if !seven_zip_path.exists() || !seven_zip_path.is_file() {
        return Err(format!("7z not found: {}", seven_zip_path.display()));
    }

    run_local_7z_command(
        &seven_zip_path,
        &package_dir,
        &[
            "a".to_string(),
            "-t7z".to_string(),
            "-mx=9".to_string(),
            "-m0=LZMA2".to_string(),
            "-mfb=273".to_string(),
            "-md=256m".to_string(),
            "-ms=on".to_string(),
            archive_path.to_string_lossy().to_string(),
            "*".to_string(),
        ],
        "7z package build failed",
    )?;

    let _ = fs::remove_dir_all(&temp_root);

    Ok((archive_path, archive_file_name))
}
