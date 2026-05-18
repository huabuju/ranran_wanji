use super::*;

pub(super) fn payload_partition_basename(partition_name: &str) -> &str {
    partition_name.rsplit('/').next().unwrap_or(partition_name)
}

pub(super) fn normalized_patch_partition_name(partition_name: &str) -> String {
    payload_partition_basename(partition_name)
        .strip_suffix(".img")
        .unwrap_or(payload_partition_basename(partition_name))
        .to_ascii_lowercase()
}

pub(super) fn normalize_slot_suffix(value: &str) -> Option<String> {
    let normalized = value.trim().to_ascii_lowercase();
    if normalized.is_empty() || normalized == "--" {
        return None;
    }

    let slot = normalized.trim_start_matches('_');
    match slot {
        "a" | "b" => Some(format!("_{}", slot)),
        _ => None,
    }
}

pub(super) fn extract_slot_suffix_from_text(value: &str) -> Option<String> {
    for line in value.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if let Some((_, tail)) = trimmed.split_once(':') {
            if let Some(slot_suffix) = normalize_slot_suffix(tail) {
                return Some(slot_suffix);
            }
        }

        if let Some(slot_suffix) = normalize_slot_suffix(trimmed) {
            return Some(slot_suffix);
        }
    }

    None
}

pub(super) fn split_partition_and_slot_suffix(partition_name: &str) -> (String, Option<String>) {
    let normalized = normalized_patch_partition_name(partition_name);
    if let Some(base) = normalized.strip_suffix("_a") {
        return (base.to_string(), Some("_a".to_string()));
    }
    if let Some(base) = normalized.strip_suffix("_b") {
        return (base.to_string(), Some("_b".to_string()));
    }

    (normalized, None)
}

pub(super) fn build_partition_with_slot_suffix(
    target_partition: &str,
    slot_suffix: Option<&str>,
) -> String {
    let (base_partition, embedded_slot_suffix) = split_partition_and_slot_suffix(target_partition);
    let slot_suffix = slot_suffix
        .and_then(normalize_slot_suffix)
        .or(embedded_slot_suffix);

    match slot_suffix {
        Some(slot_suffix) => format!("{}{}", base_partition, slot_suffix),
        None => base_partition,
    }
}

pub(super) fn infer_target_partition_from_boot_path(boot_path: &Path) -> String {
    let file_name = boot_path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();

    if file_name.contains("init_boot") {
        return "init_boot".to_string();
    }

    "boot".to_string()
}

pub(super) async fn detect_current_slot_suffix(
    adb_path: &PathBuf,
    serial: Option<&str>,
) -> Result<Option<String>, String> {
    let (success, output) = run_adb_command(
        adb_path,
        serial,
        &[
            "shell".to_string(),
            "getprop".to_string(),
            "ro.boot.slot_suffix".to_string(),
        ],
    )
    .await?;

    if !success {
        return Ok(None);
    }

    Ok(extract_slot_suffix_from_text(&output))
}

pub(super) fn pick_patch_partition(partitions: &[Value]) -> Option<String> {
    partitions
        .iter()
        .filter_map(|item| item.get("partition_name").and_then(|value| value.as_str()))
        .find(|name| normalized_patch_partition_name(name) == "init_boot")
        .map(|name| name.to_string())
        .or_else(|| {
            partitions
                .iter()
                .filter_map(|item| item.get("partition_name").and_then(|value| value.as_str()))
                .find(|name| normalized_patch_partition_name(name) == "boot")
                .map(|name| name.to_string())
        })
}

pub(super) fn pick_boot_partition(partitions: &[Value]) -> Option<String> {
    partitions
        .iter()
        .filter_map(|item| item.get("partition_name").and_then(|value| value.as_str()))
        .find(|name| normalized_patch_partition_name(name) == "boot")
        .map(|name| name.to_string())
}

pub(super) fn ensure_apatch_boot_partition(target_partition: &str) -> Result<(), String> {
    if normalized_patch_partition_name(target_partition) != "boot" {
        return Err(format!(
            "APatch 官方仅支持 boot 分区，当前分区为: {}",
            target_partition
        ));
    }

    Ok(())
}

pub(super) fn find_partition_image_recursive(
    current_dir: &Path,
    expected_file_name: &str,
) -> Result<Option<PathBuf>, String> {
    let entries = fs::read_dir(current_dir).map_err(|e| format!("读取目录失败: {}", e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("读取目录项失败: {}", e))?;
        let path = entry.path();

        if path.is_dir() {
            if let Some(found) = find_partition_image_recursive(&path, expected_file_name)? {
                return Ok(Some(found));
            }
            continue;
        }

        let file_name = path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("");
        if file_name.eq_ignore_ascii_case(expected_file_name) {
            return Ok(Some(path));
        }
    }

    Ok(None)
}
