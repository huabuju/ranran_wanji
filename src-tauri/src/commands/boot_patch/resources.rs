use super::*;

pub(super) fn normalize_patch_mode(value: &str) -> String {
    match value.trim().to_ascii_lowercase().as_str() {
        PATCH_MODE_KERNELSU => PATCH_MODE_KERNELSU.to_string(),
        PATCH_MODE_KERNELSU_NEXT => PATCH_MODE_KERNELSU_NEXT.to_string(),
        PATCH_MODE_SUKISU_ULTRA => PATCH_MODE_SUKISU_ULTRA.to_string(),
        PATCH_MODE_MAGISK_ALPHA => PATCH_MODE_MAGISK_ALPHA.to_string(),
        PATCH_MODE_APATCH => PATCH_MODE_APATCH.to_string(),
        PATCH_MODE_FOLKPATCH => PATCH_MODE_FOLKPATCH.to_string(),
        _ => PATCH_MODE_MAGISK.to_string(),
    }
}

pub(super) fn is_kernelsu_patch_mode(value: &str) -> bool {
    matches!(
        normalize_patch_mode(value).as_str(),
        PATCH_MODE_KERNELSU | PATCH_MODE_KERNELSU_NEXT | PATCH_MODE_SUKISU_ULTRA
    )
}

pub(super) fn is_apatch_patch_mode(value: &str) -> bool {
    matches!(
        normalize_patch_mode(value).as_str(),
        PATCH_MODE_APATCH | PATCH_MODE_FOLKPATCH
    )
}

pub(super) fn is_folkpatch_patch_mode(value: &str) -> bool {
    normalize_patch_mode(value) == PATCH_MODE_FOLKPATCH
}

pub(super) fn get_patch_mode_label(value: &str) -> &'static str {
    match normalize_patch_mode(value).as_str() {
        PATCH_MODE_MAGISK_ALPHA => "Magisk_Alpha",
        PATCH_MODE_APATCH => "APatch",
        PATCH_MODE_FOLKPATCH => "FolkPatch",
        PATCH_MODE_KERNELSU => "KernelSU",
        PATCH_MODE_KERNELSU_NEXT => "KernelSU_Next",
        PATCH_MODE_SUKISU_ULTRA => "SukiSU_Ultra",
        _ => "Magisk",
    }
}

pub(super) fn get_magisk_patch_dir_name(value: &str) -> &'static str {
    match normalize_patch_mode(value).as_str() {
        PATCH_MODE_MAGISK_ALPHA => "magisk_Alpha",
        _ => "magisk",
    }
}

pub(super) fn get_kernel_patch_dir_name(value: &str) -> &'static str {
    match normalize_patch_mode(value).as_str() {
        PATCH_MODE_KERNELSU_NEXT => "KernelSU_Next",
        PATCH_MODE_SUKISU_ULTRA => "SukiSU_Ultra",
        _ => "KernelSU",
    }
}

pub(super) fn get_apatch_patch_dir_name(value: &str) -> &'static str {
    match normalize_patch_mode(value).as_str() {
        PATCH_MODE_FOLKPATCH => "FolkPatch",
        _ => "APatch",
    }
}

pub(super) fn get_patch_output_prefix(value: &str) -> &'static str {
    match normalize_patch_mode(value).as_str() {
        PATCH_MODE_MAGISK_ALPHA => "magisk_alpha_patched",
        PATCH_MODE_APATCH => "apatch_patched",
        PATCH_MODE_FOLKPATCH => "folkpatch_patched",
        PATCH_MODE_KERNELSU => "kernelsu_patched",
        PATCH_MODE_KERNELSU_NEXT => "kernelsu_next_patched",
        PATCH_MODE_SUKISU_ULTRA => "sukisu_ultra_patched",
        _ => "magisk_patched",
    }
}

pub(super) fn get_resource_base_dir(_window: &Window) -> PathBuf {
    get_bin_root_dir(&_window.app_handle())
}

pub(super) fn get_boot_patch_resource_root_dir(window: &Window) -> PathBuf {
    get_resource_base_dir(window).join(BOOT_PATCH_RESOURCE_DIR_NAME)
}

pub(super) fn resolve_boot_patch_resource_dir(window: &Window, directory_name: &str) -> PathBuf {
    let grouped_dir = get_boot_patch_resource_root_dir(window).join(directory_name);
    if grouped_dir.exists() {
        grouped_dir
    } else {
        get_resource_base_dir(window).join(directory_name)
    }
}

pub(super) fn get_magisk_patch_versions_dir(window: &Window, patch_mode: &str) -> PathBuf {
    resolve_boot_patch_resource_dir(window, get_magisk_patch_dir_name(patch_mode))
}

pub(super) fn get_kernelsu_versions_dir(window: &Window) -> PathBuf {
    resolve_boot_patch_resource_dir(window, get_kernel_patch_dir_name(PATCH_MODE_KERNELSU))
}

pub(super) fn get_kernel_patch_versions_dir(window: &Window, patch_mode: &str) -> PathBuf {
    resolve_boot_patch_resource_dir(window, get_kernel_patch_dir_name(patch_mode))
}

pub(super) fn get_apatch_patch_versions_dir(window: &Window, patch_mode: &str) -> PathBuf {
    resolve_boot_patch_resource_dir(window, get_apatch_patch_dir_name(patch_mode))
}

pub(super) fn list_apk_file_options(dir: &Path) -> Result<Vec<BootPatchToolOption>, String> {
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut options = Vec::new();
    let entries =
        fs::read_dir(dir).map_err(|e| format!("读取版本目录失败 {}: {}", dir.display(), e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("读取版本目录项失败: {}", e))?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let extension = path
            .extension()
            .and_then(|value| value.to_str())
            .map(|value| value.to_ascii_lowercase())
            .unwrap_or_default();
        if extension != "apk" {
            continue;
        }

        let label = get_apk_display_name(&path);
        if label.is_empty() {
            continue;
        }

        options.push(BootPatchToolOption {
            label,
            value: normalize_local_path(&path).to_string_lossy().to_string(),
        });
    }

    options.sort_by(|left, right| left.label.cmp(&right.label));
    Ok(options)
}

pub(super) fn get_apk_display_name(path: &Path) -> String {
    path.file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .trim()
        .to_string()
}

pub(super) fn normalize_kernelsu_version_name(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    let stem = Path::new(trimmed)
        .file_stem()
        .and_then(|item| item.to_str())
        .unwrap_or(trimmed)
        .trim();

    if stem.starts_with("Kernel_") || stem.starts_with("kernel_") {
        return stem.to_string();
    }

    let release_suffix = "-release";
    if let Some(body) = stem.strip_suffix(release_suffix) {
        return normalize_kernelsu_version_name(body);
    }
    if let Some(body) = stem.strip_suffix(&release_suffix.to_ascii_uppercase()) {
        return normalize_kernelsu_version_name(body);
    }

    let prefix = "KernelSU_v";
    if let Some(body) = stem.strip_prefix(prefix) {
        let body = body.trim_matches('_');
        if !body.is_empty() {
            return format!("Kernel_{}", body);
        }
    }
    if let Some(body) = stem.strip_prefix("kernelsu_v") {
        let body = body.trim_matches('_');
        if !body.is_empty() {
            return format!("Kernel_{}", body);
        }
    }

    stem.to_string()
}

pub(super) fn resolve_kernelsu_apk_path(requested_path: &str) -> Result<PathBuf, String> {
    let trimmed = requested_path.trim();
    if trimmed.is_empty() {
        return Err("KernelSU 类 APK 路径不能为空".to_string());
    }

    let candidate = PathBuf::from(trimmed);
    if !candidate.exists() {
        return Err(format!("未找到 KernelSU 类 APK: {}", candidate.display()));
    }
    if !candidate.is_file() {
        return Err(format!(
            "KernelSU 类 APK 路径不是文件: {}",
            candidate.display()
        ));
    }

    Ok(normalize_local_path(&candidate))
}

pub(super) fn build_kernelsu_version_item(
    directory_name: &str,
    directory_path: &Path,
    apk_path: &Path,
) -> KernelSuVersionItem {
    let normalized_directory_name = normalize_kernelsu_version_name(directory_name);
    let normalized_apk_path = normalize_local_path(apk_path);

    KernelSuVersionItem {
        label: get_apk_display_name(&normalized_apk_path),
        value: normalized_apk_path.to_string_lossy().replace('\\', "/"),
        directory_name: normalized_directory_name,
        directory_path: normalize_local_path(directory_path)
            .to_string_lossy()
            .replace('\\', "/"),
        apk_path: normalized_apk_path.to_string_lossy().replace('\\', "/"),
    }
}

pub(super) fn list_kernel_patch_versions_in_dir(
    versions_dir: &Path,
) -> Result<Vec<KernelSuVersionItem>, String> {
    if !versions_dir.exists() {
        return Ok(Vec::new());
    }

    let mut versions = Vec::new();
    let mut seen_apk_paths = HashSet::new();
    let entries =
        fs::read_dir(&versions_dir).map_err(|e| format!("读取 KernelSU 类版本目录失败: {}", e))?;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() {
            let file_name = entry.file_name().to_string_lossy().to_string();
            let lower_name = file_name.to_ascii_lowercase();
            if !lower_name.ends_with(".apk") {
                continue;
            }

            let normalized_apk_path = normalize_local_path(&path)
                .to_string_lossy()
                .replace('\\', "/");
            if !seen_apk_paths.insert(normalized_apk_path) {
                continue;
            }

            let version_name = normalize_kernelsu_version_name(&file_name);
            versions.push(build_kernelsu_version_item(
                &version_name,
                &versions_dir,
                &path,
            ));
            continue;
        }

        if !path.is_dir() {
            continue;
        }

        let dir_name = entry.file_name().to_string_lossy().to_string();
        let normalized_dir_name = normalize_kernelsu_version_name(&dir_name);
        if !normalized_dir_name.starts_with("Kernel_") {
            continue;
        }

        let mut apk_path = None;

        let child_entries = fs::read_dir(&path)
            .map_err(|e| format!("读取 KernelSU 类子目录失败 {}: {}", path.display(), e))?;

        for child in child_entries.flatten() {
            let child_path = child.path();
            if !child_path.is_file() {
                continue;
            }

            let child_name = child.file_name().to_string_lossy().to_string();
            let lower_name = child_name.to_ascii_lowercase();

            if apk_path.is_none() && lower_name.ends_with(".apk") {
                apk_path = Some(child_path);
            }
        }

        let Some(apk_path) = apk_path else {
            continue;
        };

        let normalized_apk_path = normalize_local_path(&apk_path)
            .to_string_lossy()
            .replace('\\', "/");
        if !seen_apk_paths.insert(normalized_apk_path) {
            continue;
        }

        versions.push(build_kernelsu_version_item(
            &normalized_dir_name,
            &path,
            &apk_path,
        ));
    }

    versions.sort_by(|left, right| right.directory_name.cmp(&left.directory_name));
    Ok(versions)
}

pub(super) fn list_kernelsu_versions(window: &Window) -> Result<Vec<KernelSuVersionItem>, String> {
    list_kernel_patch_versions_in_dir(&get_kernelsu_versions_dir(window))
}

pub(super) fn list_kernel_patch_versions(
    window: &Window,
    patch_mode: &str,
) -> Result<Vec<KernelSuVersionItem>, String> {
    let versions_dir = get_kernel_patch_versions_dir(window, patch_mode);
    if !versions_dir.exists() {
        return Ok(Vec::new());
    }

    let mut versions = Vec::new();
    let mut seen_apk_paths = HashSet::new();
    let entries = fs::read_dir(&versions_dir).map_err(|e| {
        format!(
            "读取 {} 版本目录失败: {}",
            get_patch_mode_label(patch_mode),
            e
        )
    })?;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() {
            let file_name = entry.file_name().to_string_lossy().to_string();
            let lower_name = file_name.to_ascii_lowercase();
            if !lower_name.ends_with(".apk") {
                continue;
            }

            let normalized_apk_path = normalize_local_path(&path)
                .to_string_lossy()
                .replace('\\', "/");
            if !seen_apk_paths.insert(normalized_apk_path) {
                continue;
            }

            let version_name = normalize_kernelsu_version_name(&file_name);
            versions.push(build_kernelsu_version_item(
                &version_name,
                &versions_dir,
                &path,
            ));
            continue;
        }

        if !path.is_dir() {
            continue;
        }

        let dir_name = entry.file_name().to_string_lossy().to_string();
        let normalized_dir_name = normalize_kernelsu_version_name(&dir_name);
        if !normalized_dir_name.starts_with("Kernel_") {
            continue;
        }

        let mut apk_path = None;
        let child_entries = fs::read_dir(&path).map_err(|e| {
            format!(
                "读取 {} 子目录失败 {}: {}",
                get_patch_mode_label(patch_mode),
                path.display(),
                e
            )
        })?;

        for child in child_entries.flatten() {
            let child_path = child.path();
            if !child_path.is_file() {
                continue;
            }

            let child_name = child.file_name().to_string_lossy().to_string();
            let lower_name = child_name.to_ascii_lowercase();
            if apk_path.is_none() && lower_name.ends_with(".apk") {
                apk_path = Some(child_path);
            }
        }

        let Some(apk_path) = apk_path else {
            continue;
        };

        let normalized_apk_path = normalize_local_path(&apk_path)
            .to_string_lossy()
            .replace('\\', "/");
        if !seen_apk_paths.insert(normalized_apk_path) {
            continue;
        }

        versions.push(build_kernelsu_version_item(
            &normalized_dir_name,
            &path,
            &apk_path,
        ));
    }

    versions.sort_by(|left, right| right.directory_name.cmp(&left.directory_name));
    Ok(versions)
}
