use super::*;

pub(super) fn build_auto_source_response(
    device_codename: &str,
    device_version: &str,
    candidate: AutoRomCandidate,
    boot_path: String,
) -> BootPatchAutoSourceResponse {
    BootPatchAutoSourceResponse {
        device_codename: device_codename.to_string(),
        device_version: device_version.to_string(),
        boot_path,
        source_key: candidate.source_key.to_string(),
        source_label: candidate.source_label.to_string(),
        rom_name: candidate.rom_name,
        rom_version: candidate.rom_version,
        rom_page_url: candidate.rom_page_url,
    }
}

pub(super) async fn read_boot_patch_auto_source_props(
    window: &Window,
    adb_path: &PathBuf,
    serial: Option<&str>,
) -> Result<(String, String), String> {
    emit_log(
        window,
        "开始读取当前设备属性，用于自动匹配在线 ROM",
        "info",
        "PREP",
    );

    let props = dump_props_async_with_serial(adb_path, serial).await;
    if props.is_empty() {
        return Err("读取设备 getprop 失败，无法自动匹配在线 ROM".to_string());
    }

    let primary_codename = normalize_device_prop_value(props.get("ro.product.device"));
    let fallback_codename = normalize_device_prop_value(props.get("ro.product.odm.device"));
    let device_codename = if primary_codename.is_empty() {
        fallback_codename
    } else {
        primary_codename
    };

    let primary_version =
        normalize_device_prop_value(props.get("ro.vivo.build.version.incremental"));
    let fallback_version = normalize_device_prop_value(props.get("ro.mi.os.version.incremental"));
    let device_version = if primary_version.is_empty() {
        fallback_version
    } else {
        primary_version
    };

    if device_codename.is_empty() {
        return Err("未读取到 ro.product.device / ro.product.odm.device".to_string());
    }
    if device_version.is_empty() {
        return Err(
            "未读取到 ro.vivo.build.version.incremental / ro.mi.os.version.incremental".to_string(),
        );
    }

    emit_log(
        window,
        format!("设备机型代号: {}", device_codename),
        "info",
        "PREP",
    );
    emit_log(
        window,
        format!("设备完整版本号: {}", device_version),
        "info",
        "PREP",
    );

    if let Some(overrides) = get_boot_patch_test_overrides() {
        emit_log(
            window,
            format!(
                "已启用测试覆盖：{} {}",
                overrides.codename, overrides.build_version
            ),
            "warning",
            "PREP",
        );
        return Ok((overrides.codename, overrides.build_version));
    }

    Ok((device_codename, device_version))
}

pub(super) fn get_local_magiskboot_path(window: &Window) -> PathBuf {
    let executable_name = if cfg!(target_os = "windows") {
        "magiskboot.exe"
    } else {
        "magiskboot"
    };
    get_bin_root_dir(&window.app_handle())
        .join("platform-tools")
        .join(executable_name)
}

pub(super) fn get_local_7z_path(window: &Window) -> PathBuf {
    let executable_name = if cfg!(target_os = "windows") {
        "7z.exe"
    } else {
        "7z"
    };
    get_bin_root_dir(&window.app_handle())
        .join("7-Zip-core")
        .join(executable_name)
}

pub(super) fn run_local_magiskboot_command(
    magiskboot_path: &Path,
    current_dir: &Path,
    args: &[String],
    error_prefix: &str,
) -> Result<String, String> {
    let mut command = create_hidden_command(&magiskboot_path.to_path_buf());
    let output = command
        .current_dir(current_dir)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .args(args)
        .output()
        .map_err(|e| format!("{}: {}", error_prefix, e))?;

    let combined = join_process_output(&output.stdout, &output.stderr);
    if output.status.success() {
        Ok(combined)
    } else if combined.trim().is_empty() {
        Err(error_prefix.to_string())
    } else {
        Err(format!("{}: {}", error_prefix, combined))
    }
}

pub(super) fn run_local_7z_command(
    seven_zip_path: &Path,
    current_dir: &Path,
    args: &[String],
    error_prefix: &str,
) -> Result<String, String> {
    let mut command = create_hidden_command(&seven_zip_path.to_path_buf());
    let output = command
        .current_dir(current_dir)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .args(args)
        .output()
        .map_err(|e| format!("{}: {}", error_prefix, e))?;

    let combined = join_process_output(&output.stdout, &output.stderr);
    if output.status.success() {
        Ok(combined)
    } else if combined.trim().is_empty() {
        Err(error_prefix.to_string())
    } else {
        Err(format!("{}: {}", error_prefix, combined))
    }
}

pub(super) fn parse_prop_assignments(content: &str, props: &mut Vec<(String, String)>) {
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        if let Some((key, value)) = trimmed.split_once('=') {
            let normalized_key = key.trim();
            let normalized_value = value.trim();
            if !normalized_key.is_empty() && !normalized_value.is_empty() {
                props.push((normalized_key.to_string(), normalized_value.to_string()));
            }
        }
    }
}

pub(super) fn parse_prop_tokens(content: &str, props: &mut Vec<(String, String)>) {
    for token in content.split_whitespace() {
        if let Some((key, value)) = token.split_once('=') {
            let normalized_key = key.trim().trim_matches('"').trim_matches('\'');
            let normalized_value = value.trim().trim_matches('"').trim_matches('\'');
            if normalized_key.is_empty() || normalized_value.is_empty() {
                continue;
            }

            if normalized_key.starts_with("ro.") || normalized_key.starts_with("androidboot.") {
                props.push((
                    normalized_key.to_string(),
                    normalized_value.trim_matches(',').to_string(),
                ));
            }
        }
    }
}

pub(super) fn parse_header_value(content: &str, key: &str) -> Option<String> {
    for line in content.lines() {
        let trimmed = line.trim();
        let lower = trimmed.to_ascii_lowercase();
        if !lower.starts_with(key) {
            continue;
        }

        let mut tail = trimmed[key.len()..].trim();
        if tail.starts_with('[') && tail.ends_with(']') && tail.len() >= 2 {
            tail = &tail[1..tail.len() - 1];
        } else {
            tail = tail.trim_start_matches(|ch: char| ch == ':' || ch == '=' || ch.is_whitespace());
            if tail.starts_with('[') && tail.ends_with(']') && tail.len() >= 2 {
                tail = &tail[1..tail.len() - 1];
            }
        }

        let normalized = tail.trim();
        if !normalized.is_empty() {
            return Some(normalized.to_string());
        }
    }

    None
}

pub(super) fn extract_boot_property_value_from_binary(content: &str, key: &str) -> Option<String> {
    let markers = [format!("{key}\0"), format!("{key}=")];
    for marker in markers {
        let Some(start_index) = content.find(&marker) else {
            continue;
        };

        let mut tail = &content[start_index + marker.len()..];
        tail = tail.trim_start_matches(|ch: char| ch == '\0' || ch.is_whitespace());
        let end_index = tail
            .find(|ch: char| ch == '\0' || ch == '\r' || ch == '\n')
            .unwrap_or(tail.len());
        let candidate = tail[..end_index].trim();
        if !candidate.is_empty() {
            return Some(candidate.to_string());
        }
    }

    None
}

pub(super) fn append_props_from_boot_image_binary(
    path: &Path,
    props: &mut Vec<(String, String)>,
) -> Result<(), String> {
    let bytes = fs::read(path)
        .map_err(|e| format!("读取 boot 镜像二进制失败 {}: {}", path.display(), e))?;
    let content = String::from_utf8_lossy(&bytes);

    let binary_prop_mappings = [
        (
            "ro.product.bootimage.marketname",
            "ro.product.bootimage.marketname",
        ),
        ("ro.product.bootimage.model", "ro.product.bootimage.model"),
        ("ro.product.bootimage.name", "ro.product.bootimage.name"),
        ("ro.product.bootimage.device", "ro.product.bootimage.device"),
        ("ro.product.bootimage.brand", "ro.product.bootimage.brand"),
        (
            "ro.product.bootimage.manufacturer",
            "ro.product.bootimage.manufacturer",
        ),
        (
            "ro.bootimage.build.fingerprint",
            "ro.bootimage.build.fingerprint",
        ),
        (
            "ro.bootimage.build.version.incremental",
            "ro.bootimage.build.version.incremental",
        ),
        (
            "ro.bootimage.build.version.release",
            "ro.bootimage.build.version.release",
        ),
        ("ro.product.marketname", "ro.product.marketname"),
        ("ro.product.model", "ro.product.model"),
        ("ro.product.name", "ro.product.name"),
        ("ro.product.device", "ro.product.device"),
        ("ro.product.brand", "ro.product.brand"),
        ("ro.product.manufacturer", "ro.product.manufacturer"),
        ("ro.build.fingerprint", "ro.build.fingerprint"),
        (
            "ro.build.version.incremental",
            "ro.build.version.incremental",
        ),
        ("ro.build.version.release", "ro.build.version.release"),
        ("androidboot.product.device", "androidboot.product.device"),
        (
            "com.android.build.boot.fingerprint",
            "ro.bootimage.build.fingerprint",
        ),
        (
            "com.android.build.init_boot.fingerprint",
            "ro.bootimage.build.fingerprint",
        ),
        ("com.android.build.boot.os_version", "__header_os_version"),
        (
            "com.android.build.init_boot.os_version",
            "__header_os_version",
        ),
    ];

    for (binary_key, normalized_key) in binary_prop_mappings {
        if let Some(value) = extract_boot_property_value_from_binary(&content, binary_key) {
            props.push((normalized_key.to_string(), value));
        }
    }

    Ok(())
}

pub(super) fn append_props_from_text_file(
    path: &Path,
    props: &mut Vec<(String, String)>,
) -> Result<(), String> {
    let bytes =
        fs::read(path).map_err(|e| format!("读取镜像文本文件失败 {}: {}", path.display(), e))?;
    let content = String::from_utf8_lossy(&bytes);

    parse_prop_assignments(&content, props);
    parse_prop_tokens(&content, props);

    let file_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    if file_name == "header" {
        if let Some(os_version) = parse_header_value(&content, "os_version") {
            props.push(("__header_os_version".to_string(), os_version));
        }
    }

    Ok(())
}

pub(super) fn is_boot_metadata_text_file(path: &Path) -> bool {
    let file_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();

    matches!(
        file_name.as_str(),
        "default.prop"
            | "prop.default"
            | "build.prop"
            | "vendor_build.prop"
            | "odm_build.prop"
            | "product_build.prop"
            | "system_build.prop"
            | "system_ext_build.prop"
            | "vendor_bootconfig"
            | "bootconfig"
            | "cmdline"
            | "header"
    ) || file_name.ends_with(".prop")
}

pub(super) fn find_first_prop_value(props: &[(String, String)], keys: &[&str]) -> String {
    for key in keys {
        if let Some((_, value)) = props.iter().find(|(candidate_key, candidate_value)| {
            candidate_key.eq_ignore_ascii_case(key) && !candidate_value.trim().is_empty()
        }) {
            return value.trim().to_string();
        }
    }

    String::new()
}

pub(super) fn fill_empty_package_info_field(target: &mut String, candidate: String) {
    if target.trim().is_empty() && !candidate.trim().is_empty() {
        *target = candidate;
    }
}

pub(super) fn parse_package_info_from_fingerprint(fingerprint: &str) -> PatchPackageDeviceInfo {
    let mut info = PatchPackageDeviceInfo::default();
    let trimmed = fingerprint.trim();
    if trimmed.is_empty() {
        return info;
    }

    let (product_part, version_part) = match trimmed.split_once(':') {
        Some(parts) => parts,
        None => return info,
    };

    let product_segments = product_part.split('/').collect::<Vec<_>>();
    if let Some(brand) = product_segments
        .first()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
    {
        info.oem_name = brand.to_string();
    }
    if let Some(codename) = product_segments
        .get(2)
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
    {
        info.codename = codename.to_string();
    } else if let Some(product_name) = product_segments
        .get(1)
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
    {
        info.codename = product_name.to_string();
    }

    let version_segments = version_part.split('/').collect::<Vec<_>>();
    if let Some(android_version) = version_segments
        .first()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
    {
        info.android_version = android_version.to_string();
    }
    if let Some(build_version) = version_segments
        .get(2)
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
    {
        info.build_version = build_version.to_string();
    }

    info
}

pub(super) fn build_patch_package_device_info_from_props(
    props: &[(String, String)],
) -> PatchPackageDeviceInfo {
    let mut info = PatchPackageDeviceInfo {
        codename: find_first_prop_value(
            props,
            &[
                "ro.product.bootimage.device",
                "ro.product.bootimage.name",
                "ro.product.device",
                "ro.product.system.device",
                "ro.product.vendor.device",
                "ro.product.odm.device",
                "ro.build.product",
                "androidboot.product.device",
            ],
        ),
        build_version: find_first_prop_value(
            props,
            &[
                "ro.bootimage.build.version.incremental",
                "ro.vivo.build.version.incremental",
                "ro.mi.os.version.incremental",
                "ro.system.build.version.incremental",
                "ro.vendor.build.version.incremental",
                "ro.build.version.incremental",
                "ro.build.display.id",
                "ro.system.build.display.id",
            ],
        ),
        model: find_first_prop_value(
            props,
            &[
                "ro.product.bootimage.marketname",
                "ro.product.bootimage.model",
                "ro.product.marketname",
                "ro.product.model",
                "ro.product.system.model",
                "ro.product.vendor.model",
                "ro.product.odm.model",
                "ro.product.name",
            ],
        ),
        oem_name: find_first_prop_value(
            props,
            &[
                "ro.product.bootimage.brand",
                "ro.product.bootimage.manufacturer",
                "ro.product.brand",
                "ro.product.manufacturer",
                "ro.product.vendor.brand",
                "ro.vivo.oem.name",
            ],
        ),
        android_version: find_first_prop_value(
            props,
            &[
                "ro.bootimage.build.version.release",
                "ro.build.version.release",
                "ro.system.build.version.release",
                "ro.vendor.build.version.release",
                "__header_os_version",
            ],
        ),
    };

    let fingerprint = find_first_prop_value(
        props,
        &[
            "ro.bootimage.build.fingerprint",
            "ro.build.fingerprint",
            "ro.system.build.fingerprint",
            "ro.vendor.build.fingerprint",
        ],
    );
    let fingerprint_info = parse_package_info_from_fingerprint(&fingerprint);
    fill_empty_package_info_field(&mut info.codename, fingerprint_info.codename);
    fill_empty_package_info_field(&mut info.build_version, fingerprint_info.build_version);
    fill_empty_package_info_field(&mut info.oem_name, fingerprint_info.oem_name);
    fill_empty_package_info_field(&mut info.android_version, fingerprint_info.android_version);

    info
}

pub(super) fn get_codename_model_map_path(window: &Window) -> PathBuf {
    get_bin_root_dir(&window.app_handle())
        .join("rom-data")
        .join("codename-model-map.json")
}

pub(super) fn infer_model_name_from_codename(window: &Window, codename: &str) -> String {
    let trimmed_codename = codename.trim();
    if trimmed_codename.is_empty() {
        return String::new();
    }

    let map_path = get_codename_model_map_path(window);
    let Ok(content) = fs::read_to_string(&map_path) else {
        return String::new();
    };
    let Ok(model_map) = serde_json::from_str::<CodenameModelMapFile>(&content) else {
        return String::new();
    };

    model_map
        .entries
        .into_iter()
        .find(|entry| entry.codename.trim().eq_ignore_ascii_case(trimmed_codename))
        .map(|entry| entry.name.trim().to_string())
        .unwrap_or_default()
}

pub(super) fn merge_patch_package_device_info(
    target: &mut PatchPackageDeviceInfo,
    candidate: PatchPackageDeviceInfo,
) {
    fill_empty_package_info_field(&mut target.codename, candidate.codename);
    fill_empty_package_info_field(&mut target.build_version, candidate.build_version);
    fill_empty_package_info_field(&mut target.model, candidate.model);
    fill_empty_package_info_field(&mut target.oem_name, candidate.oem_name);
    fill_empty_package_info_field(&mut target.android_version, candidate.android_version);
}

pub(super) fn has_primary_patch_package_device_info(info: &PatchPackageDeviceInfo) -> bool {
    !info.codename.trim().is_empty()
        || !info.build_version.trim().is_empty()
        || !info.oem_name.trim().is_empty()
        || !info.android_version.trim().is_empty()
}

pub(super) fn extract_patch_package_device_info_from_binary_image(
    boot_image_path: &Path,
) -> Result<PatchPackageDeviceInfo, String> {
    if !boot_image_path.exists() || !boot_image_path.is_file() {
        return Err(format!("待修补镜像不存在: {}", boot_image_path.display()));
    }

    let mut props: Vec<(String, String)> = Vec::new();
    append_props_from_boot_image_binary(boot_image_path, &mut props)?;
    Ok(build_patch_package_device_info_from_props(&props))
}

pub(super) fn extract_patch_package_device_info_from_unpacked_image(
    window: &Window,
    boot_image_path: &Path,
) -> Result<PatchPackageDeviceInfo, String> {
    let magiskboot_path = get_local_magiskboot_path(window);
    let seven_zip_path = get_local_7z_path(window);
    if !magiskboot_path.exists() || !magiskboot_path.is_file() {
        return Err(format!(
            "未找到本地 magiskboot: {}",
            magiskboot_path.display()
        ));
    }
    if !seven_zip_path.exists() || !seven_zip_path.is_file() {
        return Err(format!("7z not found: {}", seven_zip_path.display()));
    }
    if !boot_image_path.exists() || !boot_image_path.is_file() {
        return Err(format!("待修补镜像不存在: {}", boot_image_path.display()));
    }

    let temp_root = std::env::temp_dir().join(format!(
        "ranran-boot-package-meta-{}-{}",
        chrono_like_timestamp(),
        std::process::id()
    ));
    recreate_local_dir(&temp_root)?;

    let result = (|| -> Result<PatchPackageDeviceInfo, String> {
        let boot_image = normalize_local_path(boot_image_path);
        run_local_magiskboot_command(
            &magiskboot_path,
            &temp_root,
            &[
                "unpack".to_string(),
                "-h".to_string(),
                boot_image.to_string_lossy().to_string(),
            ],
            "解析 boot/init_boot 镜像失败",
        )?;

        let mut props: Vec<(String, String)> = Vec::new();
        let mut unpacked_files = Vec::new();
        collect_relative_files(&temp_root, &temp_root, &mut unpacked_files)?;
        for relative in unpacked_files {
            let file_path = temp_root.join(relative);
            if file_path.is_file() && is_boot_metadata_text_file(&file_path) {
                append_props_from_text_file(&file_path, &mut props)?;
            }
        }

        let ramdisk_cpio_path = temp_root.join("ramdisk.cpio");
        if ramdisk_cpio_path.exists() && ramdisk_cpio_path.is_file() {
            let ramdisk_root = temp_root.join("ramdisk");
            recreate_local_dir(&ramdisk_root).map_err(|e| {
                format!(
                    "创建 ramdisk 解包目录失败 {}: {}",
                    ramdisk_root.display(),
                    e
                )
            })?;
            run_local_7z_command(
                &seven_zip_path,
                &temp_root,
                &[
                    "x".to_string(),
                    ramdisk_cpio_path.to_string_lossy().to_string(),
                    format!("-o{}", ramdisk_root.to_string_lossy()),
                    "-y".to_string(),
                ],
                "提取 ramdisk.cpio 内容失败",
            )?;

            let mut ramdisk_files = Vec::new();
            collect_relative_files(&ramdisk_root, &ramdisk_root, &mut ramdisk_files)?;
            for relative in ramdisk_files {
                let file_path = ramdisk_root.join(relative);
                if file_path.is_file() && is_boot_metadata_text_file(&file_path) {
                    append_props_from_text_file(&file_path, &mut props)?;
                }
            }
        }

        Ok(build_patch_package_device_info_from_props(&props))
    })();

    let _ = fs::remove_dir_all(&temp_root);
    result
}

pub(super) fn extract_patch_package_device_info_from_image(
    window: &Window,
    boot_image_path: &Path,
) -> Result<PatchPackageDeviceInfo, String> {
    let mut info = extract_patch_package_device_info_from_binary_image(boot_image_path)?;
    if has_primary_patch_package_device_info(&info) {
        return Ok(info);
    }

    if let Ok(unpacked_info) =
        extract_patch_package_device_info_from_unpacked_image(window, boot_image_path)
    {
        merge_patch_package_device_info(&mut info, unpacked_info);
    }

    Ok(info)
}

pub(super) async fn read_patch_package_device_info(
    window: &Window,
    _adb_path: &PathBuf,
    _serial: Option<&str>,
    boot_image_path: &Path,
) -> PatchPackageDeviceInfo {
    let test_overrides = get_boot_patch_test_overrides();
    let mut info = extract_patch_package_device_info_from_image(window, boot_image_path)
        .unwrap_or_else(|error| {
            emit_log(
                window,
                format!("未能从本地镜像提取资料包信息: {}", error),
                "warning",
                "PACK",
            );
            PatchPackageDeviceInfo::default()
        });

    if let Some(overrides) = test_overrides {
        info.codename = overrides.codename;
        info.build_version = overrides.build_version;
    }

    if info.model.trim().is_empty() {
        let inferred_model = infer_model_name_from_codename(window, &info.codename);
        if !inferred_model.is_empty() {
            info.model = inferred_model;
        }
    }

    let model_preview = if info.model.trim().is_empty() {
        "Null"
    } else {
        info.model.as_str()
    };
    let codename_preview = if info.codename.trim().is_empty() {
        "Null"
    } else {
        info.codename.as_str()
    };
    let version_preview = if info.build_version.trim().is_empty() {
        "Null"
    } else {
        info.build_version.as_str()
    };

    emit_log(
        window,
        format!(
            "镜像解析预览: 机型={} | 代号={} | 版本={}",
            model_preview, codename_preview, version_preview
        ),
        "info",
        "PACK",
    );

    if info.model.trim().is_empty()
        && info.codename.trim().is_empty()
        && info.build_version.trim().is_empty()
    {
        emit_log(
            window,
            format!(
                "镜像 {} 未解析到机型/代号/版本信息，资料包文件名将使用 Null 占位",
                boot_image_path.display()
            ),
            "warning",
            "PACK",
        );
    } else {
        emit_log(
            window,
            format!(
                "资料包命名信息已改为从镜像读取: 机型={}，代号={}，版本={}",
                if info.model.trim().is_empty() {
                    "Null"
                } else {
                    info.model.as_str()
                },
                if info.codename.trim().is_empty() {
                    "Null"
                } else {
                    info.codename.as_str()
                },
                if info.build_version.trim().is_empty() {
                    "Null"
                } else {
                    info.build_version.as_str()
                },
            ),
            "info",
            "PACK",
        );
    }

    info
}

pub(super) fn build_direct_candidate(
    source_key: &'static str,
    source_label: &'static str,
    rom_name: &str,
    rom_version: &str,
    rom_page_url: &str,
    filename: &str,
    urls: &[String],
) -> Option<AutoRomCandidate> {
    let supported_urls = urls
        .iter()
        .map(|url| url.trim().to_string())
        .filter(|url| is_supported_online_boot_url(url, filename))
        .collect::<Vec<_>>();

    if supported_urls.is_empty() {
        return None;
    }

    Some(AutoRomCandidate {
        source_key,
        source_label,
        rom_name: rom_name.trim().to_string(),
        rom_version: rom_version.trim().to_string(),
        rom_page_url: rom_page_url.trim().to_string(),
        filename: filename.trim().to_string(),
        urls: supported_urls,
    })
}

pub(super) fn select_hyperos_candidate(
    entries: Vec<HyperOsFansRomEntry>,
    device_version: &str,
) -> Option<AutoRomCandidate> {
    entries.into_iter().find_map(|entry| {
        if !version_matches(&entry.version, device_version)
            || !is_recovery_rom_flash_type(&entry.flash_type)
        {
            return None;
        }

        build_direct_candidate(
            "hyperos_fans",
            "HyperOS.fans",
            &entry.name,
            &entry.version,
            &entry.source_url,
            &entry.filename,
            &entry.url,
        )
    })
}

pub(super) fn select_miuier_candidate(
    entries: Vec<MiuierRomEntry>,
    device_version: &str,
) -> Option<AutoRomCandidate> {
    entries.into_iter().find_map(|entry| {
        if !version_matches(&entry.version, device_version)
            || !is_recovery_rom_flash_type(&entry.flash_type)
        {
            return None;
        }

        build_direct_candidate(
            "miuier",
            "MIUIER",
            &entry.name,
            &entry.version,
            &entry.source_url,
            &entry.filename,
            &entry.url,
        )
    })
}

pub(super) fn select_xfu_candidate(
    entries: Vec<XfuRomEntry>,
    device_version: &str,
) -> Option<AutoRomCandidate> {
    entries.into_iter().find_map(|entry| {
        if !version_matches(&entry.version, device_version)
            || !is_recovery_rom_flash_type(&entry.flash_type)
        {
            return None;
        }

        build_direct_candidate(
            "xfu",
            "XMFirmware",
            &entry.name,
            &entry.version,
            &entry.source_url,
            &entry.filename,
            &entry.url,
        )
    })
}

pub(super) async fn resolve_xiaomirom_candidate_url(
    window: &Window,
    entry: &XiaomiRomEntry,
) -> Option<String> {
    emit_log(
        window,
        format!("XiaomiROM 命中版本 {}，继续解析官方下载直链", entry.version),
        "info",
        "PREP",
    );

    match resolve_xiaomirom_download_urls(entry.source_url.clone()).await {
        Ok(urls) => {
            let selected = pick_supported_online_boot_url(&urls, &entry.filename);
            if selected.is_none() {
                emit_log(
                    window,
                    format!(
                        "XiaomiROM 版本 {} 已解析到链接，但没有可用于 Boot 修补的恢复包",
                        entry.version
                    ),
                    "warning",
                    "PREP",
                );
            }
            selected
        }
        Err(error) => {
            emit_log(
                window,
                format!(
                    "XiaomiROM 解析版本 {} 官方链接失败: {}",
                    entry.version, error
                ),
                "warning",
                "PREP",
            );
            None
        }
    }
}

pub(super) async fn find_auto_source_from_xiaomirom(
    window: &Window,
    device_codename: &str,
    device_version: &str,
) -> Result<Option<AutoRomCandidate>, String> {
    emit_log(window, "尝试从 XiaomiROM 自动匹配在线 ROM", "info", "PREP");
    let catalog = fetch_xiaomirom_catalog().await?;
    let Some(model) = catalog
        .into_iter()
        .find(|item| codename_matches(&item.codename, device_codename))
    else {
        emit_log(window, "XiaomiROM 未找到对应机型代号", "info", "PREP");
        return Ok(None);
    };

    emit_log(
        window,
        format!("XiaomiROM 命中机型: {} ({})", model.name, model.codename),
        "info",
        "PREP",
    );

    let entries = fetch_xiaomirom_model_roms(model.series_url.clone()).await?;
    for entry in entries.into_iter().filter(|entry| {
        version_matches(&entry.version, device_version)
            && is_recovery_rom_flash_type(&entry.flash_type)
    }) {
        if let Some(url) = resolve_xiaomirom_candidate_url(window, &entry).await {
            return Ok(Some(AutoRomCandidate {
                source_key: "xiaomirom",
                source_label: "XiaomiROM",
                rom_name: entry.name.trim().to_string(),
                rom_version: entry.version.trim().to_string(),
                rom_page_url: entry.source_url.trim().to_string(),
                filename: entry.filename.trim().to_string(),
                urls: vec![url],
            }));
        }
    }

    emit_log(
        window,
        format!("XiaomiROM 未找到版本 {} 对应的可用恢复包", device_version),
        "info",
        "PREP",
    );
    Ok(None)
}

pub(super) async fn find_auto_source_from_hyperos(
    window: &Window,
    device_codename: &str,
    device_version: &str,
) -> Result<Option<AutoRomCandidate>, String> {
    emit_log(
        window,
        "尝试从 HyperOS.fans 自动匹配在线 ROM",
        "info",
        "PREP",
    );
    let catalog = fetch_hyperos_catalog().await?;
    let Some(model) = catalog
        .into_iter()
        .find(|item| codename_matches(&item.codename, device_codename))
    else {
        emit_log(window, "HyperOS.fans 未找到对应机型代号", "info", "PREP");
        return Ok(None);
    };

    emit_log(
        window,
        format!("HyperOS.fans 命中机型: {} ({})", model.name, model.codename),
        "info",
        "PREP",
    );

    let entries = fetch_hyperos_model_roms(model.codename.clone()).await?;
    let candidate = select_hyperos_candidate(entries, device_version);
    if candidate.is_none() {
        emit_log(
            window,
            format!(
                "HyperOS.fans 未找到版本 {} 对应的可用恢复包",
                device_version
            ),
            "info",
            "PREP",
        );
    }
    Ok(candidate)
}

pub(super) async fn find_auto_source_from_miuier(
    window: &Window,
    device_codename: &str,
    device_version: &str,
) -> Result<Option<AutoRomCandidate>, String> {
    emit_log(window, "尝试从 MIUIER 自动匹配在线 ROM", "info", "PREP");
    let catalog = fetch_miuier_catalog().await?;
    let Some(model) = catalog
        .into_iter()
        .find(|item| codename_matches(&item.codename, device_codename))
    else {
        emit_log(window, "MIUIER 未找到对应机型代号", "info", "PREP");
        return Ok(None);
    };

    emit_log(
        window,
        format!("MIUIER 命中机型: {} ({})", model.name, model.codename),
        "info",
        "PREP",
    );

    let entries = fetch_miuier_model_roms(model.codename.clone()).await?;
    let candidate = select_miuier_candidate(entries, device_version);
    if candidate.is_none() {
        emit_log(
            window,
            format!("MIUIER 未找到版本 {} 对应的可用恢复包", device_version),
            "info",
            "PREP",
        );
    }
    Ok(candidate)
}

pub(super) async fn find_auto_source_from_xfu(
    window: &Window,
    device_codename: &str,
    device_version: &str,
) -> Result<Option<AutoRomCandidate>, String> {
    emit_log(
        window,
        "尝试从 XiaomiFirmwareUpdater 自动匹配在线 ROM",
        "info",
        "PREP",
    );
    let catalog = fetch_xfu_catalog().await?;
    let Some(model) = catalog
        .into_iter()
        .find(|item| codename_matches(&item.codename, device_codename))
    else {
        emit_log(
            window,
            "XiaomiFirmwareUpdater 未找到对应机型代号",
            "info",
            "PREP",
        );
        return Ok(None);
    };

    emit_log(
        window,
        format!(
            "XiaomiFirmwareUpdater 命中机型: {} ({})",
            model.name, model.codename
        ),
        "info",
        "PREP",
    );

    let entries = fetch_xfu_model_roms(model.codename.clone()).await?;
    let candidate = select_xfu_candidate(entries, device_version);
    if candidate.is_none() {
        emit_log(
            window,
            format!(
                "XiaomiFirmwareUpdater 未找到版本 {} 对应的可用恢复包",
                device_version
            ),
            "info",
            "PREP",
        );
    }
    Ok(candidate)
}

pub(super) async fn resolve_boot_patch_auto_source(
    window: &Window,
    adb_path: &PathBuf,
    serial: Option<&str>,
) -> Result<BootPatchAutoSourceResponse, String> {
    let (device_codename, device_version) =
        read_boot_patch_auto_source_props(window, adb_path, serial).await?;

    let candidate =
        match find_auto_source_from_xiaomirom(window, &device_codename, &device_version).await {
            Ok(Some(candidate)) => Some(candidate),
            Ok(None) => None,
            Err(error) => {
                emit_log(
                    window,
                    format!("XiaomiROM 匹配失败，继续尝试下一个站点: {}", error),
                    "warning",
                    "PREP",
                );
                None
            }
        };

    let candidate = if candidate.is_some() {
        candidate
    } else {
        match find_auto_source_from_hyperos(window, &device_codename, &device_version).await {
            Ok(Some(candidate)) => Some(candidate),
            Ok(None) => None,
            Err(error) => {
                emit_log(
                    window,
                    format!("HyperOS.fans 匹配失败，继续尝试下一个站点: {}", error),
                    "warning",
                    "PREP",
                );
                None
            }
        }
    };

    let candidate = if candidate.is_some() {
        candidate
    } else {
        match find_auto_source_from_miuier(window, &device_codename, &device_version).await {
            Ok(Some(candidate)) => Some(candidate),
            Ok(None) => None,
            Err(error) => {
                emit_log(
                    window,
                    format!("MIUIER 匹配失败，继续尝试下一个站点: {}", error),
                    "warning",
                    "PREP",
                );
                None
            }
        }
    };

    let candidate = if candidate.is_some() {
        candidate
    } else {
        match find_auto_source_from_xfu(window, &device_codename, &device_version).await {
            Ok(Some(candidate)) => Some(candidate),
            Ok(None) => None,
            Err(error) => {
                emit_log(
                    window,
                    format!(
                        "XiaomiFirmwareUpdater 匹配失败，继续尝试下一个站点: {}",
                        error
                    ),
                    "warning",
                    "PREP",
                );
                None
            }
        }
    };

    let Some(candidate) = candidate else {
        return Err(format!(
            "未从四个 ROM 站点匹配到机型 {}、版本 {} 对应的可用恢复包",
            device_codename, device_version
        ));
    };

    let boot_path = pick_supported_online_boot_url(&candidate.urls, &candidate.filename)
        .ok_or_else(|| "匹配到了 ROM 条目，但没有拿到可用于 Boot 修补的官方链接".to_string())?;

    emit_log(
        window,
        format!(
            "自动匹配成功: {} {}，来源 {}",
            candidate.rom_name, candidate.rom_version, candidate.source_label
        ),
        "success",
        "PREP",
    );
    emit_log(
        window,
        format!("自动填入官方链接: {}", boot_path),
        "success",
        "PREP",
    );

    Ok(build_auto_source_response(
        &device_codename,
        &device_version,
        candidate,
        boot_path,
    ))
}
