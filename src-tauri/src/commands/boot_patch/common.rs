use super::*;

pub(super) fn emit_log(window: &Window, content: impl Into<String>, log_type: &str, tag: &str) {
    let _ = window.emit(
        "boot-patch-log",
        BootPatchLog {
            content: content.into(),
            log_type: log_type.to_string(),
            tag: tag.to_string(),
        },
    );
}

pub(super) fn chrono_like_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    duration.as_secs().to_string()
}

pub(super) fn get_boot_patch_test_overrides() -> Option<BootPatchTestOverrides> {
    // 默认返回 None，使用真实设备属性。
    // 如需固定机型和版本号做测试，请临时改为 Some(...)。
    None

    // Some(BootPatchTestOverrides {
    //     codename: "shennong".to_string(),
    //     build_version: "OS3.0.304.0.WNBCNXM".to_string(),
    // })
}

pub(super) fn is_http_url(value: &str) -> bool {
    let normalized = value.trim().to_ascii_lowercase();
    normalized.starts_with("http://") || normalized.starts_with("https://")
}

pub(super) fn is_payload_source(value: &str) -> bool {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return false;
    }

    if is_http_url(trimmed) {
        return true;
    }

    Path::new(trimmed)
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| {
            let ext = ext.to_ascii_lowercase();
            ext == "bin" || ext == "zip"
        })
        .unwrap_or(false)
}

pub(super) fn normalize_match_text(value: &str) -> String {
    value
        .trim()
        .chars()
        .filter(|ch| !ch.is_whitespace() && *ch != '-' && *ch != '_')
        .flat_map(|ch| ch.to_uppercase())
        .collect()
}

pub(super) fn normalize_device_prop_value(value: Option<&String>) -> String {
    match value.map(|item| item.trim()) {
        Some("") | Some("--") | None => String::new(),
        Some(text) => text.to_string(),
    }
}

pub(super) fn version_matches(candidate_version: &str, device_version: &str) -> bool {
    let left = normalize_match_text(candidate_version);
    let right = normalize_match_text(device_version);
    !left.is_empty() && left == right
}

pub(super) fn codename_matches(candidate_codename: &str, device_codename: &str) -> bool {
    let left = normalize_match_text(candidate_codename);
    let right = normalize_match_text(device_codename);
    !left.is_empty() && left == right
}

pub(super) fn is_recovery_rom_flash_type(flash_type: &str) -> bool {
    let normalized = flash_type.trim().to_ascii_lowercase();
    matches!(normalized.as_str(), "" | "card" | "recovery")
}

pub(super) fn url_filename_hint(url: &str, filename: &str) -> String {
    let hinted = filename.trim();
    if !hinted.is_empty() {
        return hinted.to_ascii_lowercase();
    }

    let trimmed = url.trim();
    let without_query = trimmed
        .split(|ch| ch == '?' || ch == '#')
        .next()
        .unwrap_or(trimmed);
    without_query
        .rsplit('/')
        .next()
        .unwrap_or(without_query)
        .trim()
        .to_ascii_lowercase()
}

pub(super) fn is_supported_online_boot_url(url: &str, filename: &str) -> bool {
    if !is_http_url(url) {
        return false;
    }

    let lower_url = url.trim().to_ascii_lowercase();
    let hint = url_filename_hint(url, filename);
    if hint.ends_with(".tgz")
        || hint.ends_with(".tar")
        || hint.ends_with(".tar.gz")
        || lower_url.contains("firmware")
        || hint.contains("firmware")
        || hint.contains("fw_")
    {
        return false;
    }

    hint.ends_with(".zip") || hint.ends_with(".bin") || !lower_url.contains(".tgz")
}

pub(super) fn pick_supported_online_boot_url(urls: &[String], filename: &str) -> Option<String> {
    urls.iter()
        .find(|url| {
            let hint = url_filename_hint(url, filename);
            (hint.ends_with(".zip") || hint.ends_with(".bin"))
                && is_supported_online_boot_url(url, filename)
        })
        .or_else(|| {
            urls.iter()
                .find(|url| is_supported_online_boot_url(url, filename))
        })
        .map(|url| url.trim().to_string())
}

pub(super) fn join_process_output(stdout: &[u8], stderr: &[u8]) -> String {
    let stdout_text = String::from_utf8_lossy(stdout).trim().to_string();
    let stderr_text = String::from_utf8_lossy(stderr).trim().to_string();

    if stdout_text.is_empty() {
        stderr_text
    } else if stderr_text.is_empty() {
        stdout_text
    } else {
        format!("{}\n{}", stdout_text, stderr_text)
    }
}

pub(super) fn normalize_local_path(path: &Path) -> PathBuf {
    fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

pub(super) fn sanitize_file_name_component(value: &str) -> String {
    let sanitized = value
        .trim()
        .chars()
        .map(|ch| match ch {
            '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*' => '_',
            c if c.is_control() => '_',
            c => c,
        })
        .collect::<String>()
        .trim()
        .trim_matches('.')
        .to_string();

    if sanitized.is_empty() {
        "Null".to_string()
    } else {
        sanitized
    }
}

pub(super) fn path_file_name_string(path: &Path) -> String {
    path.file_name()
        .and_then(|value| value.to_str())
        .map(|value| value.to_string())
        .unwrap_or_else(|| "Null".to_string())
}

pub(super) fn to_unix_path(path: &Path) -> String {
    path.components()
        .map(|part| part.as_os_str().to_string_lossy().to_string())
        .collect::<Vec<_>>()
        .join("/")
}

pub(super) fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}

pub(super) fn recreate_local_dir(local_dir: &Path) -> Result<(), String> {
    if local_dir.exists() {
        fs::remove_dir_all(local_dir)
            .map_err(|e| format!("清理本地目录失败 {}: {}", local_dir.display(), e))?;
    }

    fs::create_dir_all(local_dir)
        .map_err(|e| format!("创建本地目录失败 {}: {}", local_dir.display(), e))?;
    Ok(())
}

pub(super) fn build_remote_chmod_command(remote_work_dir: &str) -> String {
    format!(
        "dir={dir}; chmod 755 -R \"$dir\"",
        dir = shell_quote(remote_work_dir)
    )
}
