use crate::adb::core::{
    acquire_fastboot_command_guard, create_hidden_async_command, create_hidden_command,
    detect_device_state,
    dump_props_async_with_serial, get_bin_root_dir, get_link_dumper_path, AppPaths,
};
use super::hyperos::{fetch_hyperos_catalog, fetch_hyperos_model_roms, HyperOsFansRomEntry};
use super::miuier::{fetch_miuier_catalog, fetch_miuier_model_roms, MiuierRomEntry};
use super::xfu::{fetch_xfu_catalog, fetch_xfu_model_roms, XfuRomEntry};
use super::xiaomirom::{
    fetch_xiaomirom_catalog, fetch_xiaomirom_model_roms, resolve_xiaomirom_download_urls,
    XiaomiRomEntry,
};
use crate::utils::process::{
    output_tracked_async_command, spawn_tracked_async_command, wait_tracked_async_child,
    PROCESS_KIND_ADB_CLIENT, PROCESS_KIND_FASTBOOT, PROCESS_KIND_LINK_DUMPER,
};
use getrandom::fill as fill_random_bytes;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashSet;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tauri::{Emitter, Manager, State, Window};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, BufReader};
use tokio::time::{sleep, Duration};
#[cfg(target_os = "windows")]
use windows::Win32::Globalization::{GetACP, WideCharToMultiByte};
use zip::{ZipArchive, ZipWriter};

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BootPatchRequest {
    pub patch_mode: String,
    pub boot_path: String,
    pub magisk_apk_path: String,
    pub apatch_apk_path: String,
    pub apatch_super_key: String,
    pub kernel_su_path: String,
    pub output_dir: String,
    pub kernel_su_kmi: String,
    pub keep_verity: bool,
    pub keep_force_encrypt: bool,
    pub patch_vbmeta_flag: bool,
    pub recovery_mode: bool,
    pub kernel_su_allow_shell: bool,
    pub kernel_su_enable_adbd: bool,
    pub cleanup_remote: bool,
}

#[derive(Debug, Serialize, Clone)]
pub struct BootPatchLog {
    pub content: String,
    pub log_type: String,
    pub tag: String,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BootPatchResponse {
    pub patch_mode: String,
    pub output_path: String,
    pub output_file_name: String,
    pub package_zip_path: String,
    pub package_zip_file_name: String,
    pub remote_work_dir: String,
    pub target_partition: String,
    pub target_slot_suffix: String,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OneKeyRootRequest {
    pub patch_mode: String,
    pub patched_image_path: String,
    pub magisk_apk_path: String,
    pub apatch_apk_path: String,
    pub kernel_su_apk_path: String,
    pub target_partition: String,
    pub target_slot_suffix: String,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OneKeyRootResponse {
    pub patch_mode: String,
    pub flashed_partition: String,
    pub flashed_mode: String,
    pub installed_magisk_path: String,
    pub installed_apatch_path: String,
    pub installed_kernel_su_path: String,
    pub patched_image_path: String,
    pub target_slot_suffix: String,
    pub magisk_install_succeeded: bool,
    pub magisk_install_error: Option<String>,
    pub apatch_install_succeeded: bool,
    pub apatch_install_error: Option<String>,
    pub kernel_su_install_succeeded: bool,
    pub kernel_su_install_error: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct KernelSuRuntimeRequest {
    pub kernel_su_path: String,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct KernelSuRuntimeResponse {
    pub supported_kmis: Vec<String>,
    pub detected_kmi: String,
    pub default_kmi: String,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct KernelSuVersionItem {
    pub label: String,
    pub value: String,
    pub directory_name: String,
    pub directory_path: String,
    pub apk_path: String,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BootPatchToolOption {
    pub label: String,
    pub value: String,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BootPatchToolOptionsResponse {
    pub magisk_apk_options: Vec<BootPatchToolOption>,
    pub magisk_alpha_apk_options: Vec<BootPatchToolOption>,
    pub apatch_apk_options: Vec<BootPatchToolOption>,
    pub folk_patch_apk_options: Vec<BootPatchToolOption>,
    pub kernel_su_options: Vec<KernelSuVersionItem>,
    pub kernel_su_next_options: Vec<KernelSuVersionItem>,
    pub suki_su_ultra_options: Vec<KernelSuVersionItem>,
    pub magisk_apk_dir: String,
    pub magisk_alpha_apk_dir: String,
    pub apatch_apk_dir: String,
    pub folk_patch_apk_dir: String,
    pub kernel_su_dir: String,
    pub kernel_su_next_dir: String,
    pub suki_su_ultra_dir: String,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BootPatchAutoSourceResponse {
    pub device_codename: String,
    pub device_version: String,
    pub boot_path: String,
    pub source_key: String,
    pub source_label: String,
    pub rom_name: String,
    pub rom_version: String,
    pub rom_page_url: String,
}

#[derive(Debug, Clone)]
struct ExtractedPatchKit {
    local_dir: PathBuf,
    files: Vec<PathBuf>,
}

#[derive(Debug, Clone)]
struct AutoRomCandidate {
    source_key: &'static str,
    source_label: &'static str,
    rom_name: String,
    rom_version: String,
    rom_page_url: String,
    filename: String,
    urls: Vec<String>,
}

#[derive(Debug, Clone, Default)]
struct PatchPackageDeviceInfo {
    codename: String,
    build_version: String,
    model: String,
    oem_name: String,
    android_version: String,
}

#[derive(Debug, Clone, Default)]
struct BootPatchTestOverrides {
    codename: String,
    build_version: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CodenameModelMapFile {
    entries: Vec<CodenameModelMapEntry>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CodenameModelMapEntry {
    codename: String,
    name: String,
}

const REMOTE_BOOT_PATCH_DIR: &str = "/data/local/tmp/ranran_boot_patch_workspace";
const PAYLOAD_TEMP_DIR_NAME: &str = "ranran-boot-patch-payload";
const MAGISK_KIT_DIR_NAME: &str = "magisk-kit";
const APATCH_KIT_DIR_NAME: &str = "apatch-kit";
const BOOT_PATCH_RESOURCE_DIR_NAME: &str = "boot-patch";
const PATCH_MODE_MAGISK: &str = "magisk";
const PATCH_MODE_MAGISK_ALPHA: &str = "magisk_alpha";
const PATCH_MODE_APATCH: &str = "apatch";
const PATCH_MODE_FOLKPATCH: &str = "folkpatch";
const PATCH_MODE_KERNELSU: &str = "kernelsu";
const PATCH_MODE_KERNELSU_NEXT: &str = "kernelsu_next";
const PATCH_MODE_SUKISU_ULTRA: &str = "sukisu_ultra";
const APATCH_SUPER_KEY_LENGTH: usize = 24;
const APATCH_SUPER_KEY_MIN_LENGTH: usize = 8;
const APATCH_SUPER_KEY_MAX_LENGTH: usize = 63;
const FLASH_PACKAGE_README_FILE_NAME: &str = "线刷必看.txt";
const FLASH_PACKAGE_BAT_FILE_NAME: &str = "双击我获取root.bat";
const APATCH_SUPER_KEY_UPPERCASE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const APATCH_SUPER_KEY_LOWERCASE: &[u8] = b"abcdefghijklmnopqrstuvwxyz";
const APATCH_SUPER_KEY_DIGITS: &[u8] = b"0123456789";
const APATCH_SUPER_KEY_CHARSET: &[u8] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
// 1200 * 100ms = 120000ms = 120秒 我们要最多等待120秒且100ms 轮询一次
const DEVICE_MODE_WAIT_ATTEMPTS: usize = 1200;
const DEVICE_MODE_WAIT_INTERVAL_MS: u64 = 100;

fn emit_log(window: &Window, content: impl Into<String>, log_type: &str, tag: &str) {
    let _ = window.emit(
        "boot-patch-log",
        BootPatchLog {
            content: content.into(),
            log_type: log_type.to_string(),
            tag: tag.to_string(),
        },
    );
}

fn chrono_like_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    duration.as_secs().to_string()
}

fn get_boot_patch_test_overrides() -> Option<BootPatchTestOverrides> {
    // 默认返回 None，使用真实设备属性。
    // 如需固定机型和版本号做测试，请临时改为 Some(...)。
    None

    // Some(BootPatchTestOverrides {
    //     codename: "shennong".to_string(),
    //     build_version: "OS3.0.304.0.WNBCNXM".to_string(),
    // })
}

fn is_http_url(value: &str) -> bool {
    let normalized = value.trim().to_ascii_lowercase();
    normalized.starts_with("http://") || normalized.starts_with("https://")
}

fn is_payload_source(value: &str) -> bool {
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

fn normalize_match_text(value: &str) -> String {
    value
        .trim()
        .chars()
        .filter(|ch| !ch.is_whitespace() && *ch != '-' && *ch != '_')
        .flat_map(|ch| ch.to_uppercase())
        .collect()
}

fn normalize_device_prop_value(value: Option<&String>) -> String {
    match value.map(|item| item.trim()) {
        Some("") | Some("--") | None => String::new(),
        Some(text) => text.to_string(),
    }
}

fn version_matches(candidate_version: &str, device_version: &str) -> bool {
    let left = normalize_match_text(candidate_version);
    let right = normalize_match_text(device_version);
    !left.is_empty() && left == right
}

fn codename_matches(candidate_codename: &str, device_codename: &str) -> bool {
    let left = normalize_match_text(candidate_codename);
    let right = normalize_match_text(device_codename);
    !left.is_empty() && left == right
}

fn is_recovery_rom_flash_type(flash_type: &str) -> bool {
    let normalized = flash_type.trim().to_ascii_lowercase();
    matches!(normalized.as_str(), "" | "card" | "recovery")
}

fn url_filename_hint(url: &str, filename: &str) -> String {
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

fn is_supported_online_boot_url(url: &str, filename: &str) -> bool {
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

fn pick_supported_online_boot_url(urls: &[String], filename: &str) -> Option<String> {
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

fn build_auto_source_response(
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

async fn read_boot_patch_auto_source_props(
    window: &Window,
    adb_path: &PathBuf,
    serial: Option<&str>,
) -> Result<(String, String), String> {
    emit_log(window, "开始读取当前设备属性，用于自动匹配在线 ROM", "info", "PREP");

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
    let fallback_version =
        normalize_device_prop_value(props.get("ro.mi.os.version.incremental"));
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
            "未读取到 ro.vivo.build.version.incremental / ro.mi.os.version.incremental"
                .to_string(),
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

fn get_local_magiskboot_path(window: &Window) -> PathBuf {
    let executable_name = if cfg!(target_os = "windows") {
        "magiskboot.exe"
    } else {
        "magiskboot"
    };
    get_bin_root_dir(&window.app_handle())
        .join("platform-tools")
        .join(executable_name)
}

fn get_local_7z_path(window: &Window) -> PathBuf {
    let executable_name = if cfg!(target_os = "windows") {
        "7z.exe"
    } else {
        "7z"
    };
    get_bin_root_dir(&window.app_handle())
        .join("7-Zip-core")
        .join(executable_name)
}

fn join_process_output(stdout: &[u8], stderr: &[u8]) -> String {
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

fn run_local_magiskboot_command(
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

fn run_local_7z_command(
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

fn parse_prop_assignments(content: &str, props: &mut Vec<(String, String)>) {
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

fn parse_prop_tokens(content: &str, props: &mut Vec<(String, String)>) {
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

fn parse_header_value(content: &str, key: &str) -> Option<String> {
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

fn extract_boot_property_value_from_binary(content: &str, key: &str) -> Option<String> {
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

fn append_props_from_boot_image_binary(path: &Path, props: &mut Vec<(String, String)>) -> Result<(), String> {
    let bytes = fs::read(path).map_err(|e| format!("读取 boot 镜像二进制失败 {}: {}", path.display(), e))?;
    let content = String::from_utf8_lossy(&bytes);

    let binary_prop_mappings = [
        ("ro.product.bootimage.marketname", "ro.product.bootimage.marketname"),
        ("ro.product.bootimage.model", "ro.product.bootimage.model"),
        ("ro.product.bootimage.name", "ro.product.bootimage.name"),
        ("ro.product.bootimage.device", "ro.product.bootimage.device"),
        ("ro.product.bootimage.brand", "ro.product.bootimage.brand"),
        ("ro.product.bootimage.manufacturer", "ro.product.bootimage.manufacturer"),
        ("ro.bootimage.build.fingerprint", "ro.bootimage.build.fingerprint"),
        ("ro.bootimage.build.version.incremental", "ro.bootimage.build.version.incremental"),
        ("ro.bootimage.build.version.release", "ro.bootimage.build.version.release"),
        ("ro.product.marketname", "ro.product.marketname"),
        ("ro.product.model", "ro.product.model"),
        ("ro.product.name", "ro.product.name"),
        ("ro.product.device", "ro.product.device"),
        ("ro.product.brand", "ro.product.brand"),
        ("ro.product.manufacturer", "ro.product.manufacturer"),
        ("ro.build.fingerprint", "ro.build.fingerprint"),
        ("ro.build.version.incremental", "ro.build.version.incremental"),
        ("ro.build.version.release", "ro.build.version.release"),
        ("androidboot.product.device", "androidboot.product.device"),
        ("com.android.build.boot.fingerprint", "ro.bootimage.build.fingerprint"),
        ("com.android.build.init_boot.fingerprint", "ro.bootimage.build.fingerprint"),
        ("com.android.build.boot.os_version", "__header_os_version"),
        ("com.android.build.init_boot.os_version", "__header_os_version"),
    ];

    for (binary_key, normalized_key) in binary_prop_mappings {
        if let Some(value) = extract_boot_property_value_from_binary(&content, binary_key) {
            props.push((normalized_key.to_string(), value));
        }
    }

    Ok(())
}

fn append_props_from_text_file(path: &Path, props: &mut Vec<(String, String)>) -> Result<(), String> {
    let bytes = fs::read(path).map_err(|e| format!("读取镜像文本文件失败 {}: {}", path.display(), e))?;
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

fn is_boot_metadata_text_file(path: &Path) -> bool {
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

fn find_first_prop_value(props: &[(String, String)], keys: &[&str]) -> String {
    for key in keys {
        if let Some((_, value)) = props.iter().find(|(candidate_key, candidate_value)| {
            candidate_key.eq_ignore_ascii_case(key) && !candidate_value.trim().is_empty()
        }) {
            return value.trim().to_string();
        }
    }

    String::new()
}

fn fill_empty_package_info_field(target: &mut String, candidate: String) {
    if target.trim().is_empty() && !candidate.trim().is_empty() {
        *target = candidate;
    }
}

fn parse_package_info_from_fingerprint(fingerprint: &str) -> PatchPackageDeviceInfo {
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
    if let Some(brand) = product_segments.first().map(|value| value.trim()).filter(|value| !value.is_empty()) {
        info.oem_name = brand.to_string();
    }
    if let Some(codename) = product_segments.get(2).map(|value| value.trim()).filter(|value| !value.is_empty()) {
        info.codename = codename.to_string();
    } else if let Some(product_name) = product_segments.get(1).map(|value| value.trim()).filter(|value| !value.is_empty()) {
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

fn build_patch_package_device_info_from_props(props: &[(String, String)]) -> PatchPackageDeviceInfo {
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

fn get_codename_model_map_path(window: &Window) -> PathBuf {
    get_bin_root_dir(&window.app_handle())
        .join("rom-data")
        .join("codename-model-map.json")
}

fn infer_model_name_from_codename(window: &Window, codename: &str) -> String {
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

fn merge_patch_package_device_info(target: &mut PatchPackageDeviceInfo, candidate: PatchPackageDeviceInfo) {
    fill_empty_package_info_field(&mut target.codename, candidate.codename);
    fill_empty_package_info_field(&mut target.build_version, candidate.build_version);
    fill_empty_package_info_field(&mut target.model, candidate.model);
    fill_empty_package_info_field(&mut target.oem_name, candidate.oem_name);
    fill_empty_package_info_field(&mut target.android_version, candidate.android_version);
}

fn has_primary_patch_package_device_info(info: &PatchPackageDeviceInfo) -> bool {
    !info.codename.trim().is_empty()
        || !info.build_version.trim().is_empty()
        || !info.oem_name.trim().is_empty()
        || !info.android_version.trim().is_empty()
}

fn extract_patch_package_device_info_from_binary_image(
    boot_image_path: &Path,
) -> Result<PatchPackageDeviceInfo, String> {
    if !boot_image_path.exists() || !boot_image_path.is_file() {
        return Err(format!("待修补镜像不存在: {}", boot_image_path.display()));
    }

    let mut props: Vec<(String, String)> = Vec::new();
    append_props_from_boot_image_binary(boot_image_path, &mut props)?;
    Ok(build_patch_package_device_info_from_props(&props))
}

fn extract_patch_package_device_info_from_unpacked_image(
    window: &Window,
    boot_image_path: &Path,
) -> Result<PatchPackageDeviceInfo, String> {
    let magiskboot_path = get_local_magiskboot_path(window);
    let seven_zip_path = get_local_7z_path(window);
    if !magiskboot_path.exists() || !magiskboot_path.is_file() {
        return Err(format!("未找到本地 magiskboot: {}", magiskboot_path.display()));
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
            recreate_local_dir(&ramdisk_root)
                .map_err(|e| format!("创建 ramdisk 解包目录失败 {}: {}", ramdisk_root.display(), e))?;
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

fn extract_patch_package_device_info_from_image(
    window: &Window,
    boot_image_path: &Path,
) -> Result<PatchPackageDeviceInfo, String> {
    let mut info = extract_patch_package_device_info_from_binary_image(boot_image_path)?;
    if has_primary_patch_package_device_info(&info) {
        return Ok(info);
    }

    if let Ok(unpacked_info) = extract_patch_package_device_info_from_unpacked_image(window, boot_image_path) {
        merge_patch_package_device_info(&mut info, unpacked_info);
    }

    Ok(info)
}

async fn read_patch_package_device_info(
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

    let model_preview = if info.model.trim().is_empty() { "Null" } else { info.model.as_str() };
    let codename_preview = if info.codename.trim().is_empty() { "Null" } else { info.codename.as_str() };
    let version_preview = if info.build_version.trim().is_empty() { "Null" } else { info.build_version.as_str() };

    emit_log(
        window,
        format!(
            "镜像解析预览: 机型={} | 代号={} | 版本={}",
            model_preview, codename_preview, version_preview
        ),
        "info",
        "PACK",
    );

    if info.model.trim().is_empty() && info.codename.trim().is_empty() && info.build_version.trim().is_empty() {
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
                if info.model.trim().is_empty() { "Null" } else { info.model.as_str() },
                if info.codename.trim().is_empty() { "Null" } else { info.codename.as_str() },
                if info.build_version.trim().is_empty() { "Null" } else { info.build_version.as_str() },
            ),
            "info",
            "PACK",
        );
    }

    info
}

fn build_direct_candidate(
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

fn select_hyperos_candidate(
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

fn select_miuier_candidate(
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

fn select_xfu_candidate(
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

async fn resolve_xiaomirom_candidate_url(
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
                format!("XiaomiROM 解析版本 {} 官方链接失败: {}", entry.version, error),
                "warning",
                "PREP",
            );
            None
        }
    }
}

async fn find_auto_source_from_xiaomirom(
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

async fn find_auto_source_from_hyperos(
    window: &Window,
    device_codename: &str,
    device_version: &str,
) -> Result<Option<AutoRomCandidate>, String> {
    emit_log(window, "尝试从 HyperOS.fans 自动匹配在线 ROM", "info", "PREP");
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
            format!("HyperOS.fans 未找到版本 {} 对应的可用恢复包", device_version),
            "info",
            "PREP",
        );
    }
    Ok(candidate)
}

async fn find_auto_source_from_miuier(
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

async fn find_auto_source_from_xfu(
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
        emit_log(window, "XiaomiFirmwareUpdater 未找到对应机型代号", "info", "PREP");
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

async fn resolve_boot_patch_auto_source(
    window: &Window,
    adb_path: &PathBuf,
    serial: Option<&str>,
) -> Result<BootPatchAutoSourceResponse, String> {
    let (device_codename, device_version) =
        read_boot_patch_auto_source_props(window, adb_path, serial).await?;

    let candidate = match find_auto_source_from_xiaomirom(window, &device_codename, &device_version)
        .await
    {
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
                format!("XiaomiFirmwareUpdater 匹配失败，继续尝试下一个站点: {}", error),
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
    emit_log(window, format!("自动填入官方链接: {}", boot_path), "success", "PREP");

    Ok(build_auto_source_response(
        &device_codename,
        &device_version,
        candidate,
        boot_path,
    ))
}

fn normalize_patch_mode(value: &str) -> String {
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

fn is_kernelsu_patch_mode(value: &str) -> bool {
    matches!(
        normalize_patch_mode(value).as_str(),
        PATCH_MODE_KERNELSU | PATCH_MODE_KERNELSU_NEXT | PATCH_MODE_SUKISU_ULTRA
    )
}

fn is_apatch_patch_mode(value: &str) -> bool {
    matches!(
        normalize_patch_mode(value).as_str(),
        PATCH_MODE_APATCH | PATCH_MODE_FOLKPATCH
    )
}

fn is_folkpatch_patch_mode(value: &str) -> bool {
    normalize_patch_mode(value) == PATCH_MODE_FOLKPATCH
}

fn get_patch_mode_label(value: &str) -> &'static str {
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

fn get_magisk_patch_dir_name(value: &str) -> &'static str {
    match normalize_patch_mode(value).as_str() {
        PATCH_MODE_MAGISK_ALPHA => "magisk_Alpha",
        _ => "magisk",
    }
}

fn get_kernel_patch_dir_name(value: &str) -> &'static str {
    match normalize_patch_mode(value).as_str() {
        PATCH_MODE_KERNELSU_NEXT => "KernelSU_Next",
        PATCH_MODE_SUKISU_ULTRA => "SukiSU_Ultra",
        _ => "KernelSU",
    }
}

fn get_apatch_patch_dir_name(value: &str) -> &'static str {
    match normalize_patch_mode(value).as_str() {
        PATCH_MODE_FOLKPATCH => "FolkPatch",
        _ => "APatch",
    }
}

fn get_patch_output_prefix(value: &str) -> &'static str {
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

fn get_resource_base_dir(_window: &Window) -> PathBuf {
    get_bin_root_dir(&_window.app_handle())
}

fn get_boot_patch_resource_root_dir(window: &Window) -> PathBuf {
    get_resource_base_dir(window).join(BOOT_PATCH_RESOURCE_DIR_NAME)
}

fn resolve_boot_patch_resource_dir(window: &Window, directory_name: &str) -> PathBuf {
    let grouped_dir = get_boot_patch_resource_root_dir(window).join(directory_name);
    if grouped_dir.exists() {
        grouped_dir
    } else {
        get_resource_base_dir(window).join(directory_name)
    }
}

fn get_magisk_patch_versions_dir(window: &Window, patch_mode: &str) -> PathBuf {
    resolve_boot_patch_resource_dir(window, get_magisk_patch_dir_name(patch_mode))
}

fn get_kernelsu_versions_dir(window: &Window) -> PathBuf {
    resolve_boot_patch_resource_dir(window, get_kernel_patch_dir_name(PATCH_MODE_KERNELSU))
}

fn get_kernel_patch_versions_dir(window: &Window, patch_mode: &str) -> PathBuf {
    resolve_boot_patch_resource_dir(window, get_kernel_patch_dir_name(patch_mode))
}

fn get_apatch_patch_versions_dir(window: &Window, patch_mode: &str) -> PathBuf {
    resolve_boot_patch_resource_dir(window, get_apatch_patch_dir_name(patch_mode))
}

fn list_apk_file_options(dir: &Path) -> Result<Vec<BootPatchToolOption>, String> {
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut options = Vec::new();
    let entries = fs::read_dir(dir)
        .map_err(|e| format!("读取版本目录失败 {}: {}", dir.display(), e))?;

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

fn get_apk_display_name(path: &Path) -> String {
    path.file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .trim()
        .to_string()
}

fn normalize_kernelsu_version_name(value: &str) -> String {
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

fn normalize_local_path(path: &Path) -> PathBuf {
    fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

fn sanitize_file_name_component(value: &str) -> String {
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

fn path_file_name_string(path: &Path) -> String {
    path.file_name()
        .and_then(|value| value.to_str())
        .map(|value| value.to_string())
        .unwrap_or_else(|| "Null".to_string())
}

fn to_unix_path(path: &Path) -> String {
    path.components()
        .map(|part| part.as_os_str().to_string_lossy().to_string())
        .collect::<Vec<_>>()
        .join("/")
}

fn resolve_kernelsu_apk_path(requested_path: &str) -> Result<PathBuf, String> {
    let trimmed = requested_path.trim();
    if trimmed.is_empty() {
        return Err("KernelSU 类 APK 路径不能为空".to_string());
    }

    let candidate = PathBuf::from(trimmed);
    if !candidate.exists() {
        return Err(format!("未找到 KernelSU 类 APK: {}", candidate.display()));
    }
    if !candidate.is_file() {
        return Err(format!("KernelSU 类 APK 路径不是文件: {}", candidate.display()));
    }

    Ok(normalize_local_path(&candidate))
}

fn build_kernelsu_version_item(
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

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}

fn payload_partition_basename(partition_name: &str) -> &str {
    partition_name.rsplit('/').next().unwrap_or(partition_name)
}

fn normalized_patch_partition_name(partition_name: &str) -> String {
    payload_partition_basename(partition_name)
        .strip_suffix(".img")
        .unwrap_or(payload_partition_basename(partition_name))
        .to_ascii_lowercase()
}

fn normalize_slot_suffix(value: &str) -> Option<String> {
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

fn extract_slot_suffix_from_text(value: &str) -> Option<String> {
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

fn split_partition_and_slot_suffix(partition_name: &str) -> (String, Option<String>) {
    let normalized = normalized_patch_partition_name(partition_name);
    if let Some(base) = normalized.strip_suffix("_a") {
        return (base.to_string(), Some("_a".to_string()));
    }
    if let Some(base) = normalized.strip_suffix("_b") {
        return (base.to_string(), Some("_b".to_string()));
    }

    (normalized, None)
}

fn build_partition_with_slot_suffix(target_partition: &str, slot_suffix: Option<&str>) -> String {
    let (base_partition, embedded_slot_suffix) = split_partition_and_slot_suffix(target_partition);
    let slot_suffix = slot_suffix
        .and_then(normalize_slot_suffix)
        .or(embedded_slot_suffix);

    match slot_suffix {
        Some(slot_suffix) => format!("{}{}", base_partition, slot_suffix),
        None => base_partition,
    }
}

fn infer_target_partition_from_boot_path(boot_path: &Path) -> String {
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

async fn detect_current_slot_suffix(
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

fn pick_patch_partition(partitions: &[Value]) -> Option<String> {
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

fn pick_boot_partition(partitions: &[Value]) -> Option<String> {
    partitions
        .iter()
        .filter_map(|item| item.get("partition_name").and_then(|value| value.as_str()))
        .find(|name| normalized_patch_partition_name(name) == "boot")
        .map(|name| name.to_string())
}

fn ensure_apatch_boot_partition(target_partition: &str) -> Result<(), String> {
    if normalized_patch_partition_name(target_partition) != "boot" {
        return Err(format!(
            "APatch 官方仅支持 boot 分区，当前分区为: {}",
            target_partition
        ));
    }

    Ok(())
}

fn find_partition_image_recursive(
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

fn recreate_local_dir(local_dir: &Path) -> Result<(), String> {
    if local_dir.exists() {
        fs::remove_dir_all(local_dir)
            .map_err(|e| format!("清理本地目录失败 {}: {}", local_dir.display(), e))?;
    }

    fs::create_dir_all(local_dir)
        .map_err(|e| format!("创建本地目录失败 {}: {}", local_dir.display(), e))?;
    Ok(())
}

fn build_remote_chmod_command(remote_work_dir: &str) -> String {
    format!(
        "dir={dir}; chmod 755 -R \"$dir\"",
        dir = shell_quote(remote_work_dir)
    )
}

fn get_patch_mode_package_title(patch_mode: &str) -> String {
    format!("{}_一键刷入包", get_patch_mode_label(patch_mode))
}

fn get_patch_mode_manager_name(patch_mode: &str) -> &'static str {
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

fn get_patch_mode_manager_apk_path(
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
                format!("{} 管理器 APK 路径为空", get_patch_mode_manager_name(&normalized_mode))
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

fn get_package_flash_partition(target_partition: &str, slot_suffix: &str) -> String {
    let normalized_partition = normalized_patch_partition_name(target_partition);
    let normalized_slot = normalize_slot_suffix(slot_suffix).unwrap_or_default();
    if normalized_slot.is_empty() {
        normalized_partition
    } else {
        format!("{}{}", normalized_partition, normalized_slot)
    }
}

fn get_package_root_dir_name(
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


fn build_flash_package_readme_resolved(
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

fn build_flash_package_bat(
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

fn write_text_file(path: &Path, content: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("创建目录失败 {}: {}", parent.display(), e))?;
    }
    fs::write(path, content).map_err(|e| format!("写入文件失败 {}: {}", path.display(), e))
}

#[cfg(target_os = "windows")]
fn encode_text_to_ansi_bytes(content: &str) -> Result<Vec<u8>, String> {
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
    let written = unsafe { WideCharToMultiByte(code_page, 0, &wide, Some(&mut buffer), None, None) };

    if written <= 0 {
        return Err("写入 ANSI 编码缓冲区失败".to_string());
    }

    Ok(buffer)
}

#[cfg(not(target_os = "windows"))]
fn encode_text_to_ansi_bytes(content: &str) -> Result<Vec<u8>, String> {
    Ok(content.as_bytes().to_vec())
}

fn write_ansi_text_file(path: &Path, content: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("创建目录失败 {}: {}", parent.display(), e))?;
    }

    let data = encode_text_to_ansi_bytes(content)?;
    fs::write(path, data).map_err(|e| format!("写入 ANSI 文件失败 {}: {}", path.display(), e))
}

fn copy_dir_recursive(source: &Path, target: &Path) -> Result<(), String> {
    if !source.exists() {
        return Err(format!("目录不存在: {}", source.display()));
    }

    fs::create_dir_all(target).map_err(|e| format!("创建目录失败 {}: {}", target.display(), e))?;
    let entries = fs::read_dir(source).map_err(|e| format!("读取目录失败 {}: {}", source.display(), e))?;

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
fn add_file_to_zip(
    zip: &mut ZipWriter<fs::File>,
    base_dir: &Path,
    file_path: &Path,
    options: zip::write::SimpleFileOptions,
) -> Result<(), String> {
    let relative_path = file_path.strip_prefix(base_dir).map_err(|e| {
        format!(
            "计算 ZIP 相对路径失败 {}: {}",
            file_path.display(),
            e
        )
    })?;
    let zip_entry_name = to_unix_path(relative_path);
    zip.start_file(&zip_entry_name, options)
        .map_err(|e| format!("写入 ZIP 条目失败 {}: {}", zip_entry_name, e))?;
    let data = fs::read(file_path).map_err(|e| format!("读取文件失败 {}: {}", file_path.display(), e))?;
    zip.write_all(&data)
        .map_err(|e| format!("写入 ZIP 数据失败 {}: {}", file_path.display(), e))?;
    Ok(())
}

#[allow(dead_code)]
fn add_dir_to_zip(
    zip: &mut ZipWriter<fs::File>,
    base_dir: &Path,
    current_dir: &Path,
    options: zip::write::SimpleFileOptions,
) -> Result<(), String> {
    let entries =
        fs::read_dir(current_dir).map_err(|e| format!("读取目录失败 {}: {}", current_dir.display(), e))?;

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

fn build_flash_package_zip(
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
    write_ansi_text_file(
        &package_dir.join(FLASH_PACKAGE_BAT_FILE_NAME),
        &bat_content,
    )?;

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

fn apk_entry_exists(zip: &mut ZipArchive<fs::File>, path: &str) -> bool {
    zip.by_name(path).is_ok()
}

fn extract_zip_entry(
    zip: &mut ZipArchive<fs::File>,
    entry_name: &str,
    dest_path: &Path,
) -> Result<(), String> {
    let mut file = zip
        .by_name(entry_name)
        .map_err(|e| format!("APK 中未找到 {}: {}", entry_name, e))?;

    if let Some(parent) = dest_path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {}", e))?;
    }

    let mut output = fs::File::create(dest_path).map_err(|e| format!("创建文件失败: {}", e))?;
    io::copy(&mut file, &mut output).map_err(|e| format!("写出文件失败: {}", e))?;
    Ok(())
}

fn collect_relative_files(
    base: &Path,
    current: &Path,
    files: &mut Vec<PathBuf>,
) -> Result<(), String> {
    let entries = fs::read_dir(current).map_err(|e| format!("读取目录失败: {}", e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("读取目录项失败: {}", e))?;
        let path = entry.path();

        if path.is_dir() {
            collect_relative_files(base, &path, files)?;
        } else if path.is_file() {
            let relative = path
                .strip_prefix(base)
                .map_err(|e| format!("处理相对路径失败: {}", e))?;
            files.push(relative.to_path_buf());
        }
    }

    Ok(())
}

async fn run_adb_command(
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

async fn run_fastboot_command(
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

async fn run_checked_adb(
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

async fn run_quiet_adb(
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

async fn run_checked_fastboot(
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

fn parse_kernel_major_minor(value: &str) -> Option<String> {
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

fn parse_android_tag_from_text(value: &str) -> Option<String> {
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

fn parse_android_tag_from_release(value: &str) -> Option<String> {
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

fn build_device_kmi(kernel_release: &str, android_release: &str) -> String {
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

async fn detect_device_kmi(adb_path: &PathBuf, serial: Option<&str>) -> Result<String, String> {
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

fn list_kernel_patch_versions_in_dir(versions_dir: &Path) -> Result<Vec<KernelSuVersionItem>, String> {
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
            versions.push(build_kernelsu_version_item(&version_name, &versions_dir, &path));
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

fn list_kernelsu_versions(window: &Window) -> Result<Vec<KernelSuVersionItem>, String> {
    list_kernel_patch_versions_in_dir(&get_kernelsu_versions_dir(window))
}

fn list_kernel_patch_versions(
    window: &Window,
    patch_mode: &str,
) -> Result<Vec<KernelSuVersionItem>, String> {
    let versions_dir = get_kernel_patch_versions_dir(window, patch_mode);
    if !versions_dir.exists() {
        return Ok(Vec::new());
    }

    let mut versions = Vec::new();
    let mut seen_apk_paths = HashSet::new();
    let entries = fs::read_dir(&versions_dir)
        .map_err(|e| format!("读取 {} 版本目录失败: {}", get_patch_mode_label(patch_mode), e))?;

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
            versions.push(build_kernelsu_version_item(&version_name, &versions_dir, &path));
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

async fn install_root_manager_apk(
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

async fn detect_target_slot_suffix_best_effort(
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

async fn detect_current_device_mode(
    adb_path: &PathBuf,
    fastboot_path: &PathBuf,
    serial: Option<&str>,
) -> Result<String, String> {
    detect_device_state(adb_path, fastboot_path, serial).await
}

async fn wait_for_expected_mode(
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

async fn resolve_fastboot_serial(
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

fn fastboot_output_succeeded(output: &str) -> bool {
    let upper = output.to_ascii_uppercase();
    upper.contains("OKAY") || upper.contains("FINISHED")
}

fn is_supported_root_partition(target_partition: &str) -> bool {
    let (base_partition, _) = split_partition_and_slot_suffix(target_partition);
    matches!(base_partition.as_str(), "boot" | "init_boot")
}

async fn flash_patched_image(
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

async fn reboot_system_from_fastboot_best_effort(
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

async fn ensure_fastboot_mode(
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

async fn read_device_abi(
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

fn normalize_lib_folder(abi: &str) -> Result<&'static str, String> {
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

async fn verify_remote_tmp_capability(
    window: &Window,
    adb_path: &PathBuf,
    serial: Option<&str>,
    remote_work_dir: &str,
) -> Result<(), String> {
    run_checked_adb(
        window,
        adb_path,
        serial,
        &[
            "shell".to_string(),
            format!(
                "echo ranran_boot_patch > {dir}/.rw_test && [ -f {dir}/.rw_test ] && echo TMP_WRITE_OK || (echo TMP_WRITE_FAILED && exit 1)",
                dir = remote_work_dir
            ),
        ],
        "CHK",
        "检测 /data/local/tmp 可写性失败",
    )
    .await?;

    run_checked_adb(
        window,
        adb_path,
        serial,
        &[
            "shell".to_string(),
            format!(
                "cd {dir} && [ -s ./busybox ] && [ -s ./magiskboot ] && [ -s ./magiskinit ] && [ -s ./magisk ] && ls -l ./busybox ./magiskboot ./magiskinit ./magisk && echo TMP_BIN_OK || (echo TMP_BIN_MISSING && exit 1)",
                dir = remote_work_dir
            ),
        ],
        "CHK",
        "/data/local/tmp 目录下的 busybox 无法执行，可能是分区 noexec 或设备策略限制",
    )
    .await?;

    run_checked_adb(
        window,
        adb_path,
        serial,
        &[
            "shell".to_string(),
            format!(
                "cd {dir} && ./magiskboot cleanup >/dev/null 2>&1 && echo MAGISKBOOT_EXEC_OK || (echo MAGISKBOOT_EXEC_FAILED && exit 1)",
                dir = remote_work_dir
            ),
        ],
        "CHK",
        "magiskboot 无法在手机端执行，请检查 ABI 匹配或执行权限",
    )
    .await?;

    Ok(())
}

async fn verify_remote_apatch_tmp_capability(
    window: &Window,
    adb_path: &PathBuf,
    serial: Option<&str>,
    remote_work_dir: &str,
) -> Result<(), String> {
    run_checked_adb(
        window,
        adb_path,
        serial,
        &[
            "shell".to_string(),
            format!(
                "echo ranran_boot_patch > {dir}/.rw_test && [ -f {dir}/.rw_test ] && echo TMP_WRITE_OK || (echo TMP_WRITE_FAILED && exit 1)",
                dir = remote_work_dir
            ),
        ],
        "CHK",
        "检测 /data/local/tmp 可写性失败",
    )
    .await?;

    run_checked_adb(
        window,
        adb_path,
        serial,
        &[
            "shell".to_string(),
            format!(
                "cd {dir} && [ -s ./boot_patch.sh ] && [ -s ./util_functions.sh ] && [ -s ./kpimg ] && [ -s ./kptools ] && [ -s ./magiskboot ] && ls -l ./boot_patch.sh ./util_functions.sh ./kpimg ./kptools ./magiskboot && echo TMP_BIN_OK || (echo TMP_BIN_MISSING && exit 1)",
                dir = remote_work_dir
            ),
        ],
        "CHK",
        "/data/local/tmp 目录下的 APatch 修补文件不完整或不可执行",
    )
    .await?;

    run_checked_adb(
        window,
        adb_path,
        serial,
        &[
            "shell".to_string(),
            format!(
                "cd {dir} && ./magiskboot cleanup >/dev/null 2>&1 && echo MAGISKBOOT_EXEC_OK || (echo MAGISKBOOT_EXEC_FAILED && exit 1)",
                dir = remote_work_dir
            ),
        ],
        "CHK",
        "APatch 所需的 magiskboot 无法在手机端执行，请检查 ABI 匹配或执行权限",
    )
    .await?;

    run_checked_adb(
        window,
        adb_path,
        serial,
        &[
            "shell".to_string(),
            format!(
                "cd {dir} && ./kptools -h >/dev/null 2>&1 && echo KPTOOLS_EXEC_OK || (echo KPTOOLS_EXEC_FAILED && exit 1)",
                dir = remote_work_dir
            ),
        ],
        "CHK",
        "APatch 所需的 kptools 无法在手机端执行，请检查 ABI 匹配或执行权限",
    )
    .await?;

    Ok(())
}

async fn verify_remote_boot_inputs(
    window: &Window,
    adb_path: &PathBuf,
    serial: Option<&str>,
    remote_work_dir: &str,
) -> Result<(), String> {
    run_checked_adb(
        window,
        adb_path,
        serial,
        &[
            "shell".to_string(),
            format!(
                "cd {dir} && [ -s ./boot.img ] && [ -s ./boot_patch.sh ] && [ -s ./util_functions.sh ] && [ -s ./magiskboot ] && [ -s ./magiskinit ] && [ -s ./magisk ] && [ -s ./stub.apk ] && [ -s ./init-ld ] && echo PATCH_INPUTS_OK || (echo PATCH_INPUTS_MISSING && ls -la && exit 1)",
                dir = remote_work_dir
            ),
        ],
        "CHK",
        "手机端 patch 输入文件不完整或大小异常",
    )
    .await?;

    Ok(())
}

async fn verify_remote_apatch_inputs(
    window: &Window,
    adb_path: &PathBuf,
    serial: Option<&str>,
    remote_work_dir: &str,
) -> Result<(), String> {
    run_checked_adb(
        window,
        adb_path,
        serial,
        &[
            "shell".to_string(),
            format!(
                "cd {dir} && [ -s ./boot.img ] && [ -s ./boot_patch.sh ] && [ -s ./util_functions.sh ] && [ -s ./magiskboot ] && [ -s ./kptools ] && [ -s ./kpimg ] && echo PATCH_INPUTS_OK || (echo PATCH_INPUTS_MISSING && ls -la && exit 1)",
                dir = remote_work_dir
            ),
        ],
        "CHK",
        "手机端 APatch patch 输入文件不完整或大小异常",
    )
    .await?;

    Ok(())
}

async fn verify_remote_folkpatch_tmp_capability(
    window: &Window,
    adb_path: &PathBuf,
    serial: Option<&str>,
    remote_work_dir: &str,
) -> Result<(), String> {
    run_checked_adb(
        window,
        adb_path,
        serial,
        &[
            "shell".to_string(),
            format!(
                "echo ranran_boot_patch > {dir}/.rw_test && [ -f {dir}/.rw_test ] && echo TMP_WRITE_OK || (echo TMP_WRITE_FAILED && exit 1)",
                dir = remote_work_dir
            ),
        ],
        "CHK",
        "检测 /data/local/tmp 可写性失败",
    )
    .await?;

    run_checked_adb(
        window,
        adb_path,
        serial,
        &[
            "shell".to_string(),
            format!(
                "cd {dir} && [ -s ./boot_patch.sh ] && [ -s ./util_functions.sh ] && [ -s ./kpimg ] && [ -s ./kptools ] && ls -l ./boot_patch.sh ./util_functions.sh ./kpimg ./kptools && echo TMP_BIN_OK || (echo TMP_BIN_MISSING && exit 1)",
                dir = remote_work_dir
            ),
        ],
        "CHK",
        "/data/local/tmp 目录下的 FolkPatch 修补文件不完整或不可执行",
    )
    .await?;

    run_checked_adb(
        window,
        adb_path,
        serial,
        &[
            "shell".to_string(),
            format!(
                "cd {dir} && ./kptools -h >/dev/null 2>&1 && echo KPTOOLS_EXEC_OK || (echo KPTOOLS_EXEC_FAILED && exit 1)",
                dir = remote_work_dir
            ),
        ],
        "CHK",
        "FolkPatch 所需的 kptools 无法在手机端执行，请检查 ABI 匹配或执行权限",
    )
    .await?;

    Ok(())
}

async fn verify_remote_folkpatch_inputs(
    window: &Window,
    adb_path: &PathBuf,
    serial: Option<&str>,
    remote_work_dir: &str,
) -> Result<(), String> {
    run_checked_adb(
        window,
        adb_path,
        serial,
        &[
            "shell".to_string(),
            format!(
                "cd {dir} && [ -s ./boot.img ] && [ -s ./boot_patch.sh ] && [ -s ./util_functions.sh ] && [ -s ./kptools ] && [ -s ./kpimg ] && echo PATCH_INPUTS_OK || (echo PATCH_INPUTS_MISSING && ls -la && exit 1)",
                dir = remote_work_dir
            ),
        ],
        "CHK",
        "手机端 FolkPatch patch 输入文件不完整或大小异常",
    )
    .await?;

    Ok(())
}

async fn verify_remote_kernelsu_tmp_capability(
    window: &Window,
    adb_path: &PathBuf,
    serial: Option<&str>,
    remote_work_dir: &str,
) -> Result<(), String> {
    run_checked_adb(
        window,
        adb_path,
        serial,
        &[
            "shell".to_string(),
            format!(
                "echo ranran_boot_patch > {dir}/.rw_test && [ -f {dir}/.rw_test ] && echo TMP_WRITE_OK || (echo TMP_WRITE_FAILED && exit 1)",
                dir = remote_work_dir
            ),
        ],
        "CHK",
        "检测 /data/local/tmp 可写性失败",
    )
    .await?;

    run_checked_adb(
        window,
        adb_path,
        serial,
        &[
            "shell".to_string(),
            format!(
                "cd {dir} && [ -s ./ksud ] && [ -s ./magiskboot ] && ls -l ./ksud ./magiskboot && echo TMP_BIN_OK || (echo TMP_BIN_MISSING && exit 1)",
                dir = remote_work_dir
            ),
        ],
        "CHK",
        "KernelSU 修补所需文件不完整或不可执行",
    )
    .await?;

    run_checked_adb(
        window,
        adb_path,
        serial,
        &[
            "shell".to_string(),
            format!(
                "cd {dir} && ./magiskboot cleanup >/dev/null 2>&1 && echo MAGISKBOOT_EXEC_OK || (echo MAGISKBOOT_EXEC_FAILED && exit 1)",
                dir = remote_work_dir
            ),
        ],
        "CHK",
        "KernelSU 所需的 magiskboot 无法在手机端执行，请检查 ABI 匹配或执行权限",
    )
    .await?;

    run_checked_adb(
        window,
        adb_path,
        serial,
        &[
            "shell".to_string(),
            format!(
                "cd {dir} && ./ksud -V >/dev/null 2>&1 && echo KSUD_EXEC_OK || (echo KSUD_EXEC_FAILED && exit 1)",
                dir = remote_work_dir
            ),
        ],
        "CHK",
        "KernelSU 所需的 ksud 无法在手机端执行，请检查 ABI 匹配或执行权限",
    )
    .await?;

    Ok(())
}

async fn verify_remote_kernelsu_inputs(
    window: &Window,
    adb_path: &PathBuf,
    serial: Option<&str>,
    remote_work_dir: &str,
) -> Result<(), String> {
    run_checked_adb(
        window,
        adb_path,
        serial,
        &[
            "shell".to_string(),
            format!(
                "cd {dir} && [ -s ./boot.img ] && [ -s ./ksud ] && [ -s ./magiskboot ] && echo PATCH_INPUTS_OK || (echo PATCH_INPUTS_MISSING && ls -la && exit 1)",
                dir = remote_work_dir
            ),
        ],
        "CHK",
        "手机端 KernelSU patch 输入文件不完整或大小异常",
    )
    .await?;

    Ok(())
}

async fn verify_patch_output(
    window: &Window,
    adb_path: &PathBuf,
    serial: Option<&str>,
    remote_output_path: &str,
) -> Result<(), String> {
    run_checked_adb(
        window,
        adb_path,
        serial,
        &[
            "shell".to_string(),
            format!(
                "[ -s {remote_output} ] && ls -l {remote_output} && echo PATCH_OUTPUT_OK || (echo PATCH_OUTPUT_INVALID && exit 1)",
                remote_output = remote_output_path
            ),
        ],
        "PATCH",
        "new-boot.img 未生成或文件大小为 0",
    )
    .await?;

    Ok(())
}

fn extract_magisk_kit_from_apk(
    _window: &Window,
    apk_path: &Path,
    output_root: &Path,
    device_abi: &str,
) -> Result<ExtractedPatchKit, String> {
    if !apk_path.exists() {
        return Err(format!("Magisk APK 不存在: {}", apk_path.display()));
    }
    if !apk_path.is_file() {
        return Err(format!("Magisk APK 路径不是文件: {}", apk_path.display()));
    }

    let file = fs::File::open(apk_path).map_err(|e| format!("打开 Magisk APK 失败: {}", e))?;
    let mut zip = ZipArchive::new(file).map_err(|e| format!("解析 Magisk APK 失败: {}", e))?;

    let lib_folder = normalize_lib_folder(device_abi)?;
    let required_assets = [
        ("assets/boot_patch.sh", "boot_patch.sh"),
        ("assets/util_functions.sh", "util_functions.sh"),
        ("assets/stub.apk", "stub.apk"),
    ];
    let required_libs = [
        (format!("lib/{}/libmagisk.so", lib_folder), "magisk"),
        (format!("lib/{}/libmagiskboot.so", lib_folder), "magiskboot"),
        (format!("lib/{}/libmagiskinit.so", lib_folder), "magiskinit"),
        (format!("lib/{}/libinit-ld.so", lib_folder), "init-ld"),
        (format!("lib/{}/libbusybox.so", lib_folder), "busybox"),
    ];

    for (entry_name, _) in &required_assets {
        if !apk_entry_exists(&mut zip, entry_name) {
            return Err(format!("Magisk APK 缺少资源文件: {}", entry_name));
        }
    }

    for (entry_name, _) in &required_libs {
        if !apk_entry_exists(&mut zip, entry_name) {
            return Err(format!("Magisk APK 缺少 ABI 对应二进制: {}", entry_name));
        }
    }

    let extract_dir = output_root.join(MAGISK_KIT_DIR_NAME);
    recreate_local_dir(&extract_dir).map_err(|e| format!("创建 Magisk 解包目录失败: {}", e))?;

    for (entry_name, file_name) in &required_assets {
        extract_zip_entry(&mut zip, entry_name, &extract_dir.join(file_name))?;
    }

    for (entry_name, file_name) in &required_libs {
        extract_zip_entry(&mut zip, entry_name, &extract_dir.join(file_name))?;
    }

    let chromeos_dir = extract_dir.join("chromeos");
    let chromeos_entries = [
        "assets/chromeos/futility",
        "assets/chromeos/kernel.keyblock",
        "assets/chromeos/kernel_data_key.vbprivk",
    ];
    if chromeos_entries
        .iter()
        .all(|entry_name| apk_entry_exists(&mut zip, entry_name))
    {
        fs::create_dir_all(&chromeos_dir).map_err(|e| format!("创建 chromeos 目录失败: {}", e))?;
        extract_zip_entry(
            &mut zip,
            "assets/chromeos/futility",
            &chromeos_dir.join("futility"),
        )?;
        extract_zip_entry(
            &mut zip,
            "assets/chromeos/kernel.keyblock",
            &chromeos_dir.join("kernel.keyblock"),
        )?;
        extract_zip_entry(
            &mut zip,
            "assets/chromeos/kernel_data_key.vbprivk",
            &chromeos_dir.join("kernel_data_key.vbprivk"),
        )?;
    }

    let mut files = Vec::new();
    collect_relative_files(&extract_dir, &extract_dir, &mut files)?;

    Ok(ExtractedPatchKit {
        local_dir: extract_dir,
        files,
    })
}

fn extract_apatch_kit_from_apk(
    apk_path: &Path,
    output_root: &Path,
    device_abi: &str,
) -> Result<ExtractedPatchKit, String> {
    if !apk_path.exists() {
        return Err(format!("APatch APK 不存在: {}", apk_path.display()));
    }
    if !apk_path.is_file() {
        return Err(format!("APatch APK 路径不是文件: {}", apk_path.display()));
    }

    let file = fs::File::open(apk_path).map_err(|e| format!("打开 APatch APK 失败: {}", e))?;
    let mut zip = ZipArchive::new(file).map_err(|e| format!("解析 APatch APK 失败: {}", e))?;

    let lib_folder = normalize_lib_folder(device_abi)?;
    let required_assets = [
        ("assets/boot_patch.sh", "boot_patch.sh"),
        ("assets/util_functions.sh", "util_functions.sh"),
        ("assets/kpimg", "kpimg"),
    ];
    let required_libs = [
        (format!("lib/{}/libkptools.so", lib_folder), "kptools"),
        (format!("lib/{}/libmagiskboot.so", lib_folder), "magiskboot"),
    ];

    for (entry_name, _) in &required_assets {
        if !apk_entry_exists(&mut zip, entry_name) {
            return Err(format!("APatch APK 缺少资源文件: {}", entry_name));
        }
    }

    for (entry_name, _) in &required_libs {
        if !apk_entry_exists(&mut zip, entry_name) {
            return Err(format!("APatch APK 缺少 ABI 对应二进制: {}", entry_name));
        }
    }

    let extract_dir = output_root.join(APATCH_KIT_DIR_NAME);
    recreate_local_dir(&extract_dir).map_err(|e| format!("创建 APatch 解包目录失败: {}", e))?;

    for (entry_name, file_name) in &required_assets {
        extract_zip_entry(&mut zip, entry_name, &extract_dir.join(file_name))?;
    }

    for (entry_name, file_name) in &required_libs {
        extract_zip_entry(&mut zip, entry_name, &extract_dir.join(file_name))?;
    }

    let mut files = Vec::new();
    collect_relative_files(&extract_dir, &extract_dir, &mut files)?;

    Ok(ExtractedPatchKit {
        local_dir: extract_dir,
        files,
    })
}

fn extract_folkpatch_kit_from_apk(
    apk_path: &Path,
    output_root: &Path,
    device_abi: &str,
) -> Result<ExtractedPatchKit, String> {
    if !apk_path.exists() {
        return Err(format!("FolkPatch APK 不存在: {}", apk_path.display()));
    }
    if !apk_path.is_file() {
        return Err(format!("FolkPatch APK 路径不是文件: {}", apk_path.display()));
    }

    let file = fs::File::open(apk_path).map_err(|e| format!("打开 FolkPatch APK 失败: {}", e))?;
    let mut zip = ZipArchive::new(file).map_err(|e| format!("解析 FolkPatch APK 失败: {}", e))?;

    let lib_folder = normalize_lib_folder(device_abi)?;
    let required_assets = [
        ("assets/boot_patch.sh", "boot_patch.sh"),
        ("assets/util_functions.sh", "util_functions.sh"),
        ("assets/kpimg", "kpimg"),
    ];
    let required_libs = [(format!("lib/{}/libkptools.so", lib_folder), "kptools")];

    for (entry_name, _) in &required_assets {
        if !apk_entry_exists(&mut zip, entry_name) {
            return Err(format!("FolkPatch APK 缺少资源文件: {}", entry_name));
        }
    }

    for (entry_name, _) in &required_libs {
        if !apk_entry_exists(&mut zip, entry_name) {
            return Err(format!("FolkPatch APK 缺少 ABI 对应二进制: {}", entry_name));
        }
    }

    let extract_dir = output_root.join("folkpatch-kit");
    recreate_local_dir(&extract_dir).map_err(|e| format!("创建 FolkPatch 解包目录失败: {}", e))?;

    for (entry_name, file_name) in &required_assets {
        extract_zip_entry(&mut zip, entry_name, &extract_dir.join(file_name))?;
    }

    for (entry_name, file_name) in &required_libs {
        extract_zip_entry(&mut zip, entry_name, &extract_dir.join(file_name))?;
    }

    let mut files = Vec::new();
    collect_relative_files(&extract_dir, &extract_dir, &mut files)?;

    Ok(ExtractedPatchKit {
        local_dir: extract_dir,
        files,
    })
}

fn extract_kernelsu_kit_from_apk(
    apk_path: &Path,
    output_root: &Path,
    device_abi: &str,
) -> Result<ExtractedPatchKit, String> {
    if !apk_path.exists() {
        return Err(format!("KernelSU APK 不存在: {}", apk_path.display()));
    }
    if !apk_path.is_file() {
        return Err(format!("KernelSU APK 路径不是文件: {}", apk_path.display()));
    }

    let file = fs::File::open(apk_path).map_err(|e| format!("打开 KernelSU APK 失败: {}", e))?;
    let mut zip = ZipArchive::new(file).map_err(|e| format!("解析 KernelSU APK 失败: {}", e))?;

    let lib_folder = normalize_lib_folder(device_abi)?;
    let required_libs = [
        (format!("lib/{}/libksud.so", lib_folder), "ksud"),
        (format!("lib/{}/libmagiskboot.so", lib_folder), "magiskboot"),
    ];

    for (entry_name, _) in &required_libs {
        if !apk_entry_exists(&mut zip, entry_name) {
            return Err(format!("KernelSU APK 缺少 ABI 对应二进制: {}", entry_name));
        }
    }

    let extract_dir = output_root.join("kernelsu-kit");
    recreate_local_dir(&extract_dir).map_err(|e| format!("创建 KernelSU 解包目录失败: {}", e))?;

    for (entry_name, file_name) in &required_libs {
        extract_zip_entry(&mut zip, entry_name, &extract_dir.join(file_name))?;
    }

    let mut files = Vec::new();
    collect_relative_files(&extract_dir, &extract_dir, &mut files)?;

    Ok(ExtractedPatchKit {
        local_dir: extract_dir,
        files,
    })
}

async fn inspect_payload_partitions(
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

async fn extract_payload_partition_for_patch(
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

async fn stream_patch_command(
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

async fn cleanup_remote_dir(
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

fn cleanup_local_dir(window: &Window, local_dir: &Path) {
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

fn parse_kernelsu_supported_kmis_output(output: &str) -> Vec<String> {
    let mut kmis: Vec<String> = output
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with("warning:"))
        .map(ToString::to_string)
        .collect();
    kmis.sort();
    kmis.dedup();
    kmis
}

fn parse_kernelsu_current_kmi_output(output: &str) -> String {
    output
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty() && !line.starts_with("warning:") && !line.starts_with("Error:"))
        .unwrap_or_default()
        .to_string()
}

async fn query_kernelsu_supported_kmis_via_device(
    window: &Window,
    adb_path: &PathBuf,
    serial: Option<&str>,
    kernel_su_apk_path: &Path,
) -> Result<(Vec<String>, String), String> {
    let device_abi = read_device_abi(window, adb_path, serial).await?;
    let runtime_suffix = chrono_like_timestamp();
    let local_data_root = window
        .app_handle()
        .path()
        .app_local_data_dir()
        .map_err(|e| format!("获取应用数据目录失败: {}", e))?
        .join("boot-patch-temp")
        .join(format!("kernelsu-runtime-{}", runtime_suffix));
    fs::create_dir_all(&local_data_root).map_err(|e| format!("创建 KernelSU 运行时目录失败: {}", e))?;

    let extract_result = extract_kernelsu_kit_from_apk(kernel_su_apk_path, &local_data_root, &device_abi);
    let extracted = match extract_result {
        Ok(extracted) => extracted,
        Err(error) => {
            let _ = fs::remove_dir_all(&local_data_root);
            return Err(error);
        }
    };

    let remote_work_dir = format!("{}/kernelsu-runtime-{}", REMOTE_BOOT_PATCH_DIR, runtime_suffix);
    let result = async {
        recreate_remote_dir(window, adb_path, serial, &remote_work_dir).await?;

        for relative in &extracted.files {
            let local_path = extracted.local_dir.join(relative);
            let remote_path = format!("{}/{}", remote_work_dir, to_unix_path(relative));
            run_quiet_adb(
                adb_path,
                serial,
                &[
                    "push".to_string(),
                    local_path.to_string_lossy().to_string(),
                    remote_path,
                ],
                "推送 KernelSU 运行时文件失败",
            )
            .await?;
        }

        run_checked_adb(
            window,
            adb_path,
            serial,
            &[
                "shell".to_string(),
                format!("cd {dir} && chmod 755 ./ksud ./magiskboot", dir = remote_work_dir),
            ],
            "CHK",
            "设置 KernelSU 运行时文件权限失败",
        )
        .await?;

        verify_remote_kernelsu_tmp_capability(window, adb_path, serial, &remote_work_dir).await?;

        let output = run_checked_adb(
            window,
            adb_path,
            serial,
            &[
                "shell".to_string(),
                format!("cd {dir} && ./ksud boot-info supported-kmis 2>&1", dir = remote_work_dir),
            ],
            "CHK",
            "读取 KernelSU 支持的 KMI 列表失败",
        )
        .await?;

        let current_kmi = query_kernelsu_current_kmi_via_device(
            window,
            adb_path,
            serial,
            &remote_work_dir,
        )
        .await
        .unwrap_or_default();

        Ok((parse_kernelsu_supported_kmis_output(&output), current_kmi))
    }
    .await;

    cleanup_remote_dir(window, adb_path, serial, &remote_work_dir).await;
    cleanup_local_dir(window, &extracted.local_dir);
    let _ = fs::remove_dir_all(&local_data_root);

    result
}

async fn query_kernelsu_current_kmi_via_device(
    window: &Window,
    adb_path: &PathBuf,
    serial: Option<&str>,
    remote_work_dir: &str,
) -> Result<String, String> {
    let output = run_checked_adb(
        window,
        adb_path,
        serial,
        &[
            "shell".to_string(),
            format!("cd {dir} && ./ksud boot-info current-kmi 2>&1", dir = remote_work_dir),
        ],
        "CHK",
        "读取 KernelSU 当前 KMI 失败",
    )
    .await?;

    Ok(parse_kernelsu_current_kmi_output(&output))
}

async fn recreate_remote_dir(
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

fn random_index(max: usize) -> Result<usize, String> {
    if max == 0 {
        return Err("随机字符集不能为空".to_string());
    }

    let mut bytes = [0u8; 4];
    fill_random_bytes(&mut bytes).map_err(|e| format!("生成 APatch SuperKey 失败: {}", e))?;
    Ok((u32::from_le_bytes(bytes) as usize) % max)
}

fn sample_random_char(charset: &[u8]) -> Result<char, String> {
    let index = random_index(charset.len())?;
    Ok(charset[index] as char)
}

fn shuffle_chars(chars: &mut [char]) -> Result<(), String> {
    if chars.len() <= 1 {
        return Ok(());
    }

    for index in (1..chars.len()).rev() {
        let swap_index = random_index(index + 1)?;
        chars.swap(index, swap_index);
    }

    Ok(())
}

fn generate_apatch_super_key_value(length: usize) -> Result<String, String> {
    let safe_length = length.clamp(APATCH_SUPER_KEY_MIN_LENGTH, APATCH_SUPER_KEY_MAX_LENGTH);
    let mut characters = vec![
        sample_random_char(APATCH_SUPER_KEY_UPPERCASE)?,
        sample_random_char(APATCH_SUPER_KEY_LOWERCASE)?,
        sample_random_char(APATCH_SUPER_KEY_DIGITS)?,
    ];

    while characters.len() < safe_length {
        characters.push(sample_random_char(APATCH_SUPER_KEY_CHARSET)?);
    }

    shuffle_chars(&mut characters)?;
    let generated: String = characters.into_iter().collect();
    validate_apatch_super_key(&generated)
}

fn validate_apatch_super_key(value: &str) -> Result<String, String> {
    let normalized = value.trim();
    if normalized.is_empty() {
        return Err("APatch SuperKey 不能为空".to_string());
    }

    let length = normalized.chars().count();
    if !(APATCH_SUPER_KEY_MIN_LENGTH..=APATCH_SUPER_KEY_MAX_LENGTH).contains(&length) {
        return Err("APatch SuperKey 长度需为 8-63 位".to_string());
    }

    if !normalized.chars().all(|ch| ch.is_ascii_alphanumeric()) {
        return Err("APatch SuperKey 只能包含字母和数字".to_string());
    }

    Ok(normalized.to_string())
}

#[tauri::command]
pub fn generate_apatch_super_key() -> Result<String, String> {
    generate_apatch_super_key_value(APATCH_SUPER_KEY_LENGTH)
}

fn build_patched_output_name(
    patch_mode: &str,
    boot_path: &Path,
    apatch_super_key: Option<&str>,
) -> String {
    let prefix = get_patch_output_prefix(patch_mode);
    let boot_stem = boot_path
        .file_stem()
        .map(|name| name.to_string_lossy().to_string())
        .filter(|name| !name.trim().is_empty())
        .unwrap_or_else(|| "boot".to_string());
    let output_suffix = chrono_like_timestamp();
    if is_apatch_patch_mode(patch_mode) {
        if let Some(super_key) = apatch_super_key.filter(|value| !value.trim().is_empty()) {
            return format!(
                "{}_{}_superkey_{}_{}.img",
                prefix, boot_stem, super_key, output_suffix
            );
        }
    }

    format!("{}_{}_{}.img", prefix, boot_stem, output_suffix)
}

#[tauri::command]
pub async fn patch_boot_image(
    window: Window,
    paths: State<'_, AppPaths>,
    serial: Option<String>,
    request: BootPatchRequest,
) -> Result<BootPatchResponse, String> {
    let serial_ref = serial.as_deref().filter(|value| !value.is_empty());
    let adb_path = &paths.adb;
    let patch_mode = normalize_patch_mode(&request.patch_mode);
    let remote_work_dir = REMOTE_BOOT_PATCH_DIR.to_string();
    let mut local_cleanup_dirs: Vec<PathBuf> = Vec::new();

    let result = async {
        let magisk_apk_path = PathBuf::from(&request.magisk_apk_path);
        let apatch_apk_path = PathBuf::from(&request.apatch_apk_path);
        let output_dir = PathBuf::from(&request.output_dir);
        let patch_label = get_patch_mode_label(&patch_mode);
        if is_kernelsu_patch_mode(&patch_mode) {
            let kernel_su_apk_path = resolve_kernelsu_apk_path(&request.kernel_su_path)?;
            let requested_boot_path = request.boot_path.trim();
            if requested_boot_path.is_empty() {
                return Err("Boot 镜像路径不能为空".to_string());
            }

            let (boot_path, target_partition) = if is_payload_source(requested_boot_path) {
                emit_log(
                    &window,
                    format!("已进入 Payload 模式，输入来源: {}", requested_boot_path),
                    "info",
                    "PREP",
                );
                let payload_temp_dir = std::env::temp_dir().join(PAYLOAD_TEMP_DIR_NAME);
                let info_dir = payload_temp_dir.join("info");
                let extract_dir = payload_temp_dir.join("extract");
                recreate_local_dir(&payload_temp_dir)
                    .map_err(|e| format!("重建 Payload 临时目录失败: {}", e))?;
                emit_log(
                    &window,
                    format!("Payload 临时工作目录: {}", payload_temp_dir.display()),
                    "info",
                    "PREP",
                );
                emit_log(
                    &window,
                    format!("Payload 信息输出目录: {}", info_dir.display()),
                    "info",
                    "PREP",
                );
                emit_log(
                    &window,
                    format!("Payload 镜像提取目录: {}", extract_dir.display()),
                    "info",
                    "PREP",
                );
                let partitions = inspect_payload_partitions(&window, requested_boot_path, &info_dir).await?;
                let selected_partition = pick_patch_partition(&partitions)
                    .ok_or_else(|| "未在 Payload 中找到可修补的 init_boot / boot 分区".to_string())?;
                let target_partition = normalized_patch_partition_name(&selected_partition);
                let extracted_boot_path = extract_payload_partition_for_patch(
                    &window,
                    requested_boot_path,
                    &selected_partition,
                    &extract_dir,
                )
                .await?;
                emit_log(
                    &window,
                    format!("Payload 提取完成，待修补镜像路径: {}", extracted_boot_path.display()),
                    "success",
                    "PREP",
                );
                local_cleanup_dirs.push(payload_temp_dir);
                (extracted_boot_path, target_partition)
            } else {
                emit_log(
                    &window,
                    format!("已进入本地镜像模式，直接使用原始文件: {}", requested_boot_path),
                    "info",
                    "PREP",
                );
                let local_boot_path = PathBuf::from(requested_boot_path);
                let target_partition = infer_target_partition_from_boot_path(&local_boot_path);
                (local_boot_path, target_partition)
            };

            if !boot_path.exists() {
                return Err(format!("Boot 文件不存在: {}", boot_path.display()));
            }
            if !boot_path.is_file() {
                return Err(format!("Boot 路径不是文件: {}", boot_path.display()));
            }
            let package_device_info =
                read_patch_package_device_info(&window, adb_path, serial_ref, &boot_path).await;
            fs::create_dir_all(&output_dir).map_err(|e| format!("创建输出目录失败: {}", e))?;
            emit_log(
                &window,
                format!("最终修补输出目录: {}", output_dir.display()),
                "info",
                "PREP",
            );

            let target_slot_suffix = detect_target_slot_suffix_best_effort(
                &window,
                adb_path,
                serial_ref,
                false,
            )
            .await?;
            let device_abi = read_device_abi(&window, adb_path, serial_ref).await?;
            let local_data_root = window
                .app_handle()
                .path()
                .app_local_data_dir()
                .map_err(|e| format!("获取应用数据目录失败: {}", e))?
                .join("boot-patch-temp");
            fs::create_dir_all(&local_data_root).map_err(|e| format!("创建本地缓存目录失败: {}", e))?;

            let extracted =
                extract_kernelsu_kit_from_apk(&kernel_su_apk_path, &local_data_root, &device_abi)?;
            local_cleanup_dirs.push(extracted.local_dir.clone());

            emit_log(
                &window,
                format!("创建手机临时目录: {}", remote_work_dir),
                "info",
                "PUSH",
            );
            recreate_remote_dir(&window, adb_path, serial_ref, &remote_work_dir).await?;

            emit_log(
                &window,
                &format!("开始推送 {} 修补资源到手机", patch_label),
                "info",
                "PUSH",
            );
            for relative in &extracted.files {
                let local_path = extracted.local_dir.join(relative);
                let remote_path = format!("{}/{}", remote_work_dir, to_unix_path(relative));
                run_quiet_adb(
                    adb_path,
                    serial_ref,
                    &[
                        "push".to_string(),
                        local_path.to_string_lossy().to_string(),
                        remote_path,
                    ],
                    &format!("推送 {} 修补文件失败", patch_label),
                )
                .await?;
            }
            emit_log(
                &window,
                &format!("{} 修补资源推送完成", patch_label),
                "success",
                "PUSH",
            );

            let remote_boot_path = format!("{}/boot.img", remote_work_dir);
            emit_log(
                &window,
                format!("推送 Boot 文件到手机: {}", boot_path.display()),
                "info",
                "PUSH",
            );
            run_quiet_adb(
                adb_path,
                serial_ref,
                &[
                    "push".to_string(),
                    boot_path.to_string_lossy().to_string(),
                    remote_boot_path.clone(),
                ],
                "推送 Boot 文件失败",
            )
            .await?;
            emit_log(&window, "Boot 文件推送完成", "success", "PUSH");

            run_checked_adb(
                &window,
                adb_path,
                serial_ref,
                &[
                    "shell".to_string(),
                    build_remote_chmod_command(&remote_work_dir),
                ],
                "PUSH",
                "设置手机端文件权限失败",
            )
            .await?;

            verify_remote_kernelsu_tmp_capability(&window, adb_path, serial_ref, &remote_work_dir)
                .await?;
            verify_remote_kernelsu_inputs(&window, adb_path, serial_ref, &remote_work_dir).await?;

            let output_file_name = build_patched_output_name(&patch_mode, &boot_path, None);
            let remote_output_path = format!("{}/{}", remote_work_dir, output_file_name);
            let mut patch_command = format!(
                "cd {dir} && ./ksud boot-patch -b ./boot.img -o . --out-name {out_name} --magiskboot ./magiskboot --partition {partition}",
                dir = remote_work_dir,
                out_name = shell_quote(&output_file_name),
                partition = shell_quote(&target_partition),
            );

            if !request.kernel_su_kmi.trim().is_empty() {
                patch_command.push_str(" --kmi ");
                patch_command.push_str(&shell_quote(request.kernel_su_kmi.trim()));
            }
            if request.kernel_su_allow_shell {
                patch_command.push_str(" --allow-shell");
            }
            if request.kernel_su_enable_adbd {
                patch_command.push_str(" --enable-adbd");
            }
            patch_command.push_str(" 2>&1");

            emit_log(
                &window,
                &format!("开始执行手机端 {} ksud boot-patch", patch_label),
                "info",
                "PATCH",
            );
            stream_patch_command(&window, adb_path, serial_ref, &patch_command).await?;
            verify_patch_output(&window, adb_path, serial_ref, &remote_output_path).await?;

            let local_output_path = output_dir.join(&output_file_name);
            emit_log(
                &window,
                format!("拉回 patched 镜像到电脑: {}", local_output_path.display()),
                "info",
                "PULL",
            );
            run_checked_adb(
                &window,
                adb_path,
                serial_ref,
                &[
                    "pull".to_string(),
                    remote_output_path,
                    local_output_path.to_string_lossy().to_string(),
                ],
                "PULL",
                &format!("拉回 {} patched 镜像失败", patch_label),
            )
            .await?;

            if !local_output_path.exists() {
                return Err(format!("{} patched 镜像拉回完成，但本地未找到输出文件", patch_label));
            }

            emit_log(
                &window,
                format!("Boot 修补完成: {}", local_output_path.display()),
                "success",
                "DONE",
            );

            emit_log(&window, "开始整理刷机资料包", "info", "PACK");
            let manager_apk_path = get_patch_mode_manager_apk_path(
                &patch_mode,
                &magisk_apk_path,
                &apatch_apk_path,
                Some(&kernel_su_apk_path),
            )?;
            let (package_zip_path, package_zip_file_name) = build_flash_package_zip(
                &window,
                &output_dir,
                &patch_mode,
                &manager_apk_path,
                &boot_path,
                &local_output_path,
                &target_partition,
                &target_slot_suffix,
                &package_device_info,
            )?;
            emit_log(
                &window,
                format!("刷机资料包已生成: {}", package_zip_path.display()),
                "success",
                "PACK",
            );

            return Ok(BootPatchResponse {
                patch_mode: patch_mode.clone(),
                output_path: local_output_path.to_string_lossy().to_string(),
                output_file_name,
                package_zip_path: package_zip_path.to_string_lossy().to_string(),
                package_zip_file_name,
                remote_work_dir: remote_work_dir.clone(),
                target_partition,
                target_slot_suffix,
            });
        }
        emit_log(
            &window,
            format!(
                "当前修补方案: {}",
                patch_label
            ),
            "info",
            "PREP",
        );
        let requested_boot_path = request.boot_path.trim();
        if requested_boot_path.is_empty() {
            return Err("Boot 镜像路径不能为空".to_string());
        }

        let is_apatch_mode = is_apatch_patch_mode(&patch_mode);
        let (boot_path, target_partition) = if is_payload_source(requested_boot_path) {
            emit_log(
                &window,
                format!("已进入 Payload 模式，输入来源: {}", requested_boot_path),
                "info",
                "PREP",
            );
            let payload_temp_dir = std::env::temp_dir().join(PAYLOAD_TEMP_DIR_NAME);
            let info_dir = payload_temp_dir.join("info");
            let extract_dir = payload_temp_dir.join("extract");
            recreate_local_dir(&payload_temp_dir)
                .map_err(|e| format!("重建 Payload 临时目录失败: {}", e))?;
            emit_log(
                &window,
                format!("Payload 临时工作目录: {}", payload_temp_dir.display()),
                "info",
                "PREP",
            );
            emit_log(
                &window,
                format!("Payload 信息输出目录: {}", info_dir.display()),
                "info",
                "PREP",
            );
            emit_log(
                &window,
                format!("Payload 镜像提取目录: {}", extract_dir.display()),
                "info",
                "PREP",
            );
            let partitions = inspect_payload_partitions(&window, requested_boot_path, &info_dir).await?;
            let selected_partition = pick_patch_partition(&partitions)
                .ok_or_else(|| "未在 payload 中找到可修补的 init_boot / boot 分区".to_string())?;
            let selected_partition = if is_apatch_mode {
                pick_boot_partition(&partitions).ok_or_else(|| {
                    format!("{} 官方仅支持 boot 分区，当前 payload 未找到可修补的 boot 分区", patch_label)
                })?
            } else {
                selected_partition
            };
            let target_partition = normalized_patch_partition_name(&selected_partition);
            let extracted_boot_path = extract_payload_partition_for_patch(
                &window,
                requested_boot_path,
                &selected_partition,
                &extract_dir,
            )
            .await?;
            emit_log(
                &window,
                format!("Payload 提取完成，待修补镜像路径: {}", extracted_boot_path.display()),
                "success",
                "PREP",
            );
            local_cleanup_dirs.push(payload_temp_dir);
            (extracted_boot_path, target_partition)
        } else {
            emit_log(
                &window,
                format!("已进入本地镜像模式，直接使用原文件: {}", requested_boot_path),
                "info",
                "PREP",
            );
            let local_boot_path = PathBuf::from(requested_boot_path);
            let target_partition = infer_target_partition_from_boot_path(&local_boot_path);
            if is_apatch_mode {
                ensure_apatch_boot_partition(&target_partition)?;
            }
            (local_boot_path, target_partition)
        };

        if !boot_path.exists() {
            return Err(format!("Boot 文件不存在: {}", boot_path.display()));
        }
        if !boot_path.is_file() {
            return Err(format!("Boot 路径不是文件: {}", boot_path.display()));
        }
        let package_device_info =
            read_patch_package_device_info(&window, adb_path, serial_ref, &boot_path).await;
        let apatch_super_key = if is_apatch_mode {
            Some(validate_apatch_super_key(&request.apatch_super_key)?)
        } else {
            None
        };
        if is_apatch_mode {
            ensure_apatch_boot_partition(&target_partition)?;
        }
        if is_apatch_mode {
            if !apatch_apk_path.exists() {
                return Err(format!("APatch APK 不存在: {}", apatch_apk_path.display()));
            }
            if !apatch_apk_path.is_file() {
                return Err(format!("APatch APK 路径不是文件: {}", apatch_apk_path.display()));
            }
        } else {
            if !magisk_apk_path.exists() {
                return Err(format!("{} APK 不存在: {}", patch_label, magisk_apk_path.display()));
            }
            if !magisk_apk_path.is_file() {
                return Err(format!("{} APK 路径不是文件: {}", patch_label, magisk_apk_path.display()));
            }
        }

        fs::create_dir_all(&output_dir).map_err(|e| format!("创建输出目录失败: {}", e))?;
        emit_log(
            &window,
            format!("最终修补输出目录: {}", output_dir.display()),
            "info",
            "PREP",
        );

        let target_slot_suffix = detect_current_slot_suffix(adb_path, serial_ref).await?.unwrap_or_default();
        if target_slot_suffix.is_empty() {
            emit_log(
                &window,
                "未读取到 ro.boot.slot_suffix，按无插槽设备处理，后续刷入将不带 _a / _b",
                "info",
                "PREP",
            );
        } else {
            emit_log(
                &window,
                format!("当前要刷入的目标插槽: {}", target_slot_suffix),
                "info",
                "PREP",
            );
        }

        let device_abi = read_device_abi(&window, adb_path, serial_ref).await?;
        let local_data_root = window
            .app_handle()
            .path()
            .app_local_data_dir()
            .map_err(|e| format!("获取应用数据目录失败: {}", e))?
            .join("boot-patch-temp");
        fs::create_dir_all(&local_data_root).map_err(|e| format!("创建本地缓存目录失败: {}", e))?;

        let extracted = if is_apatch_mode {
            if is_folkpatch_patch_mode(&patch_mode) {
                extract_folkpatch_kit_from_apk(&apatch_apk_path, &local_data_root, &device_abi)?
            } else {
                extract_apatch_kit_from_apk(&apatch_apk_path, &local_data_root, &device_abi)?
            }
        } else {
            extract_magisk_kit_from_apk(&window, &magisk_apk_path, &local_data_root, &device_abi)?
        };
        local_cleanup_dirs.push(extracted.local_dir.clone());

        emit_log(
            &window,
            format!("创建手机临时目录: {}", remote_work_dir),
            "info",
            "PUSH",
        );
        recreate_remote_dir(&window, adb_path, serial_ref, &remote_work_dir).await?;
        run_checked_adb(
            &window,
            adb_path,
            serial_ref,
            &[
                "shell".to_string(),
                "mkdir".to_string(),
                "-p".to_string(),
                remote_work_dir.clone(),
            ],
            "PUSH",
            "创建手机临时目录失败",
        )
        .await?;

        emit_log(&window, "开始推送修补资源到手机", "info", "PUSH");
        for relative in &extracted.files {
            if let Some(parent) = relative.parent().filter(|parent| !parent.as_os_str().is_empty()) {
                let remote_dir = format!("{}/{}", remote_work_dir, to_unix_path(parent));
                run_checked_adb(
                    &window,
                    adb_path,
                    serial_ref,
                    &[
                        "shell".to_string(),
                        "mkdir".to_string(),
                        "-p".to_string(),
                        remote_dir,
                    ],
                    "PUSH",
                    "创建手机子目录失败",
                )
                .await?;
            }

            let local_path = extracted.local_dir.join(relative);
            let remote_path = format!("{}/{}", remote_work_dir, to_unix_path(relative));
            run_quiet_adb(
                adb_path,
                serial_ref,
                &[
                    "push".to_string(),
                    local_path.to_string_lossy().to_string(),
                    remote_path,
                ],
                "推送修补文件失败",
            )
            .await?;
        }
        emit_log(&window, "修补资源推送完成", "success", "PUSH");

        let remote_boot_path = format!("{}/boot.img", remote_work_dir);
        emit_log(
            &window,
            format!("推送 Boot 文件到手机: {}", boot_path.display()),
            "info",
            "PUSH",
        );
        run_quiet_adb(
            adb_path,
            serial_ref,
            &[
                "push".to_string(),
                boot_path.to_string_lossy().to_string(),
                remote_boot_path.clone(),
            ],
            "推送 Boot 文件失败",
        )
        .await?;
        emit_log(&window, "Boot 文件推送完成", "success", "PUSH");

        emit_log(&window, "准备设置手机端文件权限", "info", "PUSH");
        run_checked_adb(
            &window,
            adb_path,
            serial_ref,
            &[
                "shell".to_string(),
                build_remote_chmod_command(&remote_work_dir),
            ],
            "PUSH",
            "设置手机端文件权限失败",
        )
        .await?;

        if is_apatch_mode {
            if is_folkpatch_patch_mode(&patch_mode) {
                verify_remote_folkpatch_tmp_capability(
                    &window,
                    adb_path,
                    serial_ref,
                    &remote_work_dir,
                )
                .await?;
                verify_remote_folkpatch_inputs(&window, adb_path, serial_ref, &remote_work_dir)
                    .await?;
            } else {
                verify_remote_apatch_tmp_capability(
                    &window,
                    adb_path,
                    serial_ref,
                    &remote_work_dir,
                )
                .await?;
                verify_remote_apatch_inputs(&window, adb_path, serial_ref, &remote_work_dir)
                    .await?;
            }
        } else {
            verify_remote_tmp_capability(&window, adb_path, serial_ref, &remote_work_dir).await?;
            verify_remote_boot_inputs(&window, adb_path, serial_ref, &remote_work_dir).await?;
        }

        emit_log(
            &window,
            if is_apatch_mode {
                format!("开始执行手机端 {} boot_patch.sh", patch_label)
            } else {
                format!("开始执行手机端 {} boot_patch.sh", patch_label)
            },
            "info",
            "PATCH",
        );
        let patch_command = if is_apatch_mode {
            format!(
                "cd {dir} && sh ./boot_patch.sh {super_key} ./boot.img 2>&1",
                dir = remote_work_dir,
                super_key = shell_quote(apatch_super_key.as_deref().unwrap_or_default()),
            )
        } else {
            format!(
                "cd {dir} && export KEEPVERITY={keep_verity} && export KEEPFORCEENCRYPT={keep_force_encrypt} && export PATCHVBMETAFLAG={patch_vbmeta_flag} && export RECOVERYMODE={recovery_mode} && export LEGACYSAR=false && sh ./boot_patch.sh ./boot.img 2>&1",
                dir = remote_work_dir,
                keep_verity = if request.keep_verity { "true" } else { "false" },
                keep_force_encrypt = if request.keep_force_encrypt { "true" } else { "false" },
                patch_vbmeta_flag = if request.patch_vbmeta_flag { "true" } else { "false" },
                recovery_mode = if request.recovery_mode { "true" } else { "false" },
            )
        };
        stream_patch_command(&window, adb_path, serial_ref, &patch_command).await?;

        let remote_output_path = format!("{}/new-boot.img", remote_work_dir);
        verify_patch_output(&window, adb_path, serial_ref, &remote_output_path).await?;

        let output_file_name =
            build_patched_output_name(&patch_mode, &boot_path, apatch_super_key.as_deref());
        let local_output_path = output_dir.join(&output_file_name);

        emit_log(
            &window,
            format!("拉回 patched 镜像到电脑: {}", local_output_path.display()),
            "info",
            "PULL",
        );
        run_checked_adb(
            &window,
            adb_path,
            serial_ref,
            &[
                "pull".to_string(),
                remote_output_path,
                local_output_path.to_string_lossy().to_string(),
            ],
            "PULL",
            "拉回 patched 镜像失败",
        )
        .await?;

        if !local_output_path.exists() {
            return Err("patched 镜像拉回完成，但本地未找到输出文件".to_string());
        }

        emit_log(
            &window,
            format!("Boot 修补完成: {}", local_output_path.display()),
            "success",
            "DONE",
        );

        emit_log(&window, "开始整理刷机资料包", "info", "PACK");
        let manager_apk_path =
            get_patch_mode_manager_apk_path(&patch_mode, &magisk_apk_path, &apatch_apk_path, None)?;
        let (package_zip_path, package_zip_file_name) = build_flash_package_zip(
            &window,
            &output_dir,
            &patch_mode,
            &manager_apk_path,
            &boot_path,
            &local_output_path,
            &target_partition,
            &target_slot_suffix,
            &package_device_info,
        )?;
        emit_log(
            &window,
            format!("刷机资料包已生成: {}", package_zip_path.display()),
            "success",
            "PACK",
        );

        Ok(BootPatchResponse {
            patch_mode: patch_mode.clone(),
            output_path: local_output_path.to_string_lossy().to_string(),
            output_file_name,
            package_zip_path: package_zip_path.to_string_lossy().to_string(),
            package_zip_file_name,
            remote_work_dir: remote_work_dir.clone(),
            target_partition,
            target_slot_suffix,
        })
    }
    .await;

    if let Err(error) = &result {
        emit_log(&window, format!("修补流程失败: {}", error), "error", "DONE");
    }

    if request.cleanup_remote {
        cleanup_remote_dir(&window, adb_path, serial_ref, &remote_work_dir).await;
    }

    for local_dir in local_cleanup_dirs {
        cleanup_local_dir(&window, &local_dir);
    }

    result
}

#[tauri::command]
pub async fn get_kernelsu_runtime(
    window: Window,
    paths: State<'_, AppPaths>,
    serial: Option<String>,
    request: KernelSuRuntimeRequest,
) -> Result<KernelSuRuntimeResponse, String> {
    let serial_ref = serial.as_deref().filter(|value| !value.is_empty());
    let requested_path = request.kernel_su_path.trim();
    if requested_path.is_empty() {
        return Ok(KernelSuRuntimeResponse {
            supported_kmis: Vec::new(),
            detected_kmi: String::new(),
            default_kmi: String::new(),
        });
    }

    let kernel_su_apk_path = resolve_kernelsu_apk_path(requested_path)?;
    let current_mode = if serial_ref.is_some() {
        detect_current_device_mode(&paths.adb, &paths.fastboot, serial_ref).await?
    } else {
        String::new()
    };
    let (supported_kmis, detected_kmi) = if current_mode == "device" {
        let (supported_kmis, ksud_current_kmi) =
            query_kernelsu_supported_kmis_via_device(&window, &paths.adb, serial_ref, &kernel_su_apk_path)
                .await?;
        let fallback_detected_kmi = detect_device_kmi(&paths.adb, serial_ref).await?;
        let detected_kmi = if ksud_current_kmi.trim().is_empty() {
            fallback_detected_kmi
        } else {
            ksud_current_kmi
        };
        (supported_kmis, detected_kmi)
    } else {
        (Vec::new(), String::new())
    };
    let default_kmi = if detected_kmi.is_empty() {
        String::new()
    } else if supported_kmis.iter().any(|item| item == &detected_kmi) {
        detected_kmi.clone()
    } else {
        detected_kmi.clone()
    };

    Ok(KernelSuRuntimeResponse {
        supported_kmis,
        detected_kmi,
        default_kmi,
    })
}

#[tauri::command]
pub fn get_kernelsu_versions(window: Window) -> Result<Vec<KernelSuVersionItem>, String> {
    list_kernelsu_versions(&window)
}

#[tauri::command]
pub fn get_boot_patch_tool_options(window: Window) -> Result<BootPatchToolOptionsResponse, String> {
    let magisk_dir = get_magisk_patch_versions_dir(&window, PATCH_MODE_MAGISK);
    let magisk_alpha_dir = get_magisk_patch_versions_dir(&window, PATCH_MODE_MAGISK_ALPHA);
    let apatch_dir = get_apatch_patch_versions_dir(&window, PATCH_MODE_APATCH);
    let folk_patch_dir = get_apatch_patch_versions_dir(&window, PATCH_MODE_FOLKPATCH);
    let kernelsu_dir = get_kernel_patch_versions_dir(&window, PATCH_MODE_KERNELSU);
    let kernelsu_next_dir = get_kernel_patch_versions_dir(&window, PATCH_MODE_KERNELSU_NEXT);
    let sukisu_ultra_dir = get_kernel_patch_versions_dir(&window, PATCH_MODE_SUKISU_ULTRA);
    let magisk_apk_options = list_apk_file_options(&magisk_dir)?;
    let magisk_alpha_apk_options = list_apk_file_options(&magisk_alpha_dir)?;
    let apatch_apk_options = list_apk_file_options(&apatch_dir)?;
    let folk_patch_apk_options = list_apk_file_options(&folk_patch_dir)?;
    let kernel_su_options = list_kernelsu_versions(&window)?;
    let kernel_su_next_options = list_kernel_patch_versions(&window, PATCH_MODE_KERNELSU_NEXT)?;
    let suki_su_ultra_options = list_kernel_patch_versions(&window, PATCH_MODE_SUKISU_ULTRA)?;

    Ok(BootPatchToolOptionsResponse {
        magisk_apk_options,
        magisk_alpha_apk_options,
        apatch_apk_options,
        folk_patch_apk_options,
        kernel_su_options,
        kernel_su_next_options,
        suki_su_ultra_options,
        magisk_apk_dir: magisk_dir.to_string_lossy().to_string(),
        magisk_alpha_apk_dir: magisk_alpha_dir.to_string_lossy().to_string(),
        apatch_apk_dir: apatch_dir.to_string_lossy().to_string(),
        folk_patch_apk_dir: folk_patch_dir.to_string_lossy().to_string(),
        kernel_su_dir: kernelsu_dir.to_string_lossy().to_string(),
        kernel_su_next_dir: kernelsu_next_dir.to_string_lossy().to_string(),
        suki_su_ultra_dir: sukisu_ultra_dir.to_string_lossy().to_string(),
    })
}

#[tauri::command]
pub async fn prepare_boot_patch_auto_source(
    window: Window,
    paths: State<'_, AppPaths>,
    serial: Option<String>,
) -> Result<BootPatchAutoSourceResponse, String> {
    let serial_ref = serial.as_deref().filter(|value| !value.is_empty());
    resolve_boot_patch_auto_source(&window, &paths.adb, serial_ref).await
}

#[tauri::command]
pub async fn boot_patch_one_key_root(
    window: Window,
    paths: State<'_, AppPaths>,
    serial: Option<String>,
    request: OneKeyRootRequest,
) -> Result<OneKeyRootResponse, String> {
    let adb_serial = serial.as_deref().filter(|value| !value.is_empty());
    let patch_mode = normalize_patch_mode(&request.patch_mode);
    let patched_image_path = PathBuf::from(request.patched_image_path.trim());
    let magisk_apk_path = PathBuf::from(request.magisk_apk_path.trim());
    let apatch_apk_path = PathBuf::from(request.apatch_apk_path.trim());
    let kernel_su_apk_path = PathBuf::from(request.kernel_su_apk_path.trim());
    let target_partition = normalized_patch_partition_name(&request.target_partition);
    let requested_slot_suffix = normalize_slot_suffix(&request.target_slot_suffix);
    let adb_path = &paths.adb;
    let fastboot_path = &paths.fastboot;
    if is_kernelsu_patch_mode(&patch_mode) {
        let patch_label = get_patch_mode_label(&patch_mode);
        if request.patched_image_path.trim().is_empty() {
            return Err("修补产物路径不能为空".to_string());
        }
        if target_partition.is_empty() {
            return Err("刷入分区不能为空".to_string());
        }
        if !is_supported_root_partition(&target_partition) {
            return Err(format!(
                "一键 Root 仅支持刷入 boot / init_boot，当前分区为: {}",
                target_partition
            ));
        }
        if !patched_image_path.exists() || !patched_image_path.is_file() {
            return Err(format!(
                "未找到修补后的镜像文件: {}",
                patched_image_path.display()
            ));
        }

        emit_log(&window, "开始执行一键 Root 流程", "info", "ROOT");
        emit_log(&window, &format!("当前方案: {}", patch_label), "info", "ROOT");
        emit_log(
            &window,
            format!("目标刷入分区: {}", target_partition),
            "info",
            "ROOT",
        );
        emit_log(
            &window,
            format!("修补镜像路径: {}", patched_image_path.display()),
            "info",
            "ROOT",
        );

        if let Some(slot_suffix) = requested_slot_suffix.as_deref() {
            emit_log(
                &window,
                format!("目标刷入插槽: {}", slot_suffix),
                "info",
                "ROOT",
            );
        }

        if request.kernel_su_apk_path.trim().is_empty() {
            return Err(format!("{} APK 路径不能为空", patch_label));
        }
        if !kernel_su_apk_path.exists() || !kernel_su_apk_path.is_file() {
            return Err(format!("未找到对应 {} APK: {}", patch_label, kernel_su_apk_path.display()));
        }

        let current_mode = detect_current_device_mode(adb_path, fastboot_path, adb_serial).await?;
        if current_mode != "device" {
            return Err(format!(
                "一键 Root 需要设备先处于 ADB 已连接状态，当前模式为: {}",
                current_mode
            ));
        }

        let (kernel_su_install_succeeded, kernel_su_install_error) = install_root_manager_apk(
            &window,
            adb_path,
            adb_serial,
            &kernel_su_apk_path,
            patch_label,
        )
        .await;
        emit_log(&window, "准备重启到 Fastboot", "info", "ROOT");

        run_checked_adb(
            &window,
            adb_path,
            adb_serial,
            &["reboot".to_string(), "bootloader".to_string()],
            "ROOT",
            "重启到 Fastboot 失败",
        )
        .await?;

        let mut fastboot_serial =
            ensure_fastboot_mode(&window, adb_path, fastboot_path, adb_serial).await?;
        let flash_partition =
            build_partition_with_slot_suffix(&target_partition, requested_slot_suffix.as_deref());

        emit_log(
            &window,
            format!("开始在 Fastboot 模式刷入 {}", target_partition),
            "info",
            "ROOT",
        );
        let flashed_mode = match flash_patched_image(
            &window,
            fastboot_path,
            fastboot_serial.as_deref(),
            &target_partition,
            requested_slot_suffix.as_deref(),
            &patched_image_path,
        )
        .await
        {
            Ok(_) => "fastboot".to_string(),
            Err(_fastboot_error) => {
                emit_log(
                    &window,
                    format!(
                        "Fastboot 模式刷入 {} 失败，准备切换到 FastbootD 重试",
                        target_partition
                    ),
                    "warning",
                    "ROOT",
                );

                if let Err(error) = run_checked_fastboot(
                    &window,
                    fastboot_path,
                    fastboot_serial.as_deref(),
                    &["reboot".to_string(), "fastboot".to_string()],
                    "ROOT",
                    "从 Fastboot 重启到 FastbootD 失败",
                )
                .await
                {
                    reboot_system_from_fastboot_best_effort(
                        &window,
                        fastboot_path,
                        fastboot_serial.as_deref(),
                        "Fastboot 刷入失败，且无法切换到 FastbootD",
                    )
                    .await;
                    return Err(error);
                }

                if let Err(error) = wait_for_expected_mode(
                    adb_path,
                    fastboot_path,
                    adb_serial,
                    &["fastbootd"],
                    DEVICE_MODE_WAIT_ATTEMPTS,
                    DEVICE_MODE_WAIT_INTERVAL_MS,
                )
                .await
                {
                    reboot_system_from_fastboot_best_effort(
                        &window,
                        fastboot_path,
                        fastboot_serial.as_deref(),
                        "等待设备进入 FastbootD 超时",
                    )
                    .await;
                    return Err(error);
                }

                fastboot_serial = resolve_fastboot_serial(fastboot_path, adb_serial).await?;
                emit_log(
                    &window,
                    format!("开始在 FastbootD 模式刷入 {}", target_partition),
                    "info",
                    "ROOT",
                );

                match flash_patched_image(
                    &window,
                    fastboot_path,
                    fastboot_serial.as_deref(),
                    &target_partition,
                    requested_slot_suffix.as_deref(),
                    &patched_image_path,
                )
                .await
                {
                    Ok(_) => "fastbootd".to_string(),
                    Err(fastbootd_error) => {
                        emit_log(
                            &window,
                            format!(
                                "FastbootD 模式刷入 {} 仍然失败，请查看上一条 FLASH 日志",
                                target_partition
                            ),
                            "error",
                            "ROOT",
                        );
                        reboot_system_from_fastboot_best_effort(
                            &window,
                            fastboot_path,
                            fastboot_serial.as_deref(),
                            "FastbootD 刷入失败，按要求直接开机",
                        )
                        .await;
                        return Err(format!(
                            "刷入 {} 失败: {}",
                            target_partition, fastbootd_error
                        ));
                    }
                }
            }
        };

        emit_log(
            &window,
            format!("{} 分区刷入完成，准备重启系统", target_partition),
            "success",
            "ROOT",
        );
        run_checked_fastboot(
            &window,
            fastboot_path,
            fastboot_serial.as_deref(),
            &["reboot".to_string()],
            "ROOT",
            "重启回系统失败",
        )
        .await?;

        emit_log(
            &window,
            format!(
                "一键 Root 完成，已在 {} 模式刷入 {}",
                flashed_mode, target_partition
            ),
            "success",
            "DONE",
        );

        return Ok(OneKeyRootResponse {
            patch_mode: patch_mode.clone(),
            flashed_partition: flash_partition,
            flashed_mode,
            installed_magisk_path: String::new(),
            installed_apatch_path: String::new(),
            installed_kernel_su_path: kernel_su_apk_path.to_string_lossy().to_string(),
            patched_image_path: patched_image_path.to_string_lossy().to_string(),
            target_slot_suffix: requested_slot_suffix.unwrap_or_default(),
            magisk_install_succeeded: true,
            magisk_install_error: None,
            apatch_install_succeeded: true,
            apatch_install_error: None,
            kernel_su_install_succeeded,
            kernel_su_install_error,
        });
    }

    if is_apatch_patch_mode(&patch_mode) {
        let patch_label = get_patch_mode_label(&patch_mode);
        if request.patched_image_path.trim().is_empty() {
            return Err("修补产物路径不能为空".to_string());
        }
        if request.apatch_apk_path.trim().is_empty() {
            return Err(format!("{} APK 路径不能为空", patch_label));
        }
        if target_partition.is_empty() {
            return Err("刷入分区不能为空".to_string());
        }
        ensure_apatch_boot_partition(&target_partition)?;
        if !is_supported_root_partition(&target_partition) {
            return Err(format!(
                "一键 Root 仅支持刷入 boot / init_boot，当前分区为: {}",
                target_partition
            ));
        }
        if !patched_image_path.exists() || !patched_image_path.is_file() {
            return Err(format!(
                "未找到修补后的镜像文件: {}",
                patched_image_path.display()
            ));
        }
        if !apatch_apk_path.exists() || !apatch_apk_path.is_file() {
            return Err(format!("未找到 {} APK: {}", patch_label, apatch_apk_path.display()));
        }

        emit_log(&window, "开始执行一键 Root 流程", "info", "ROOT");
        emit_log(&window, &format!("当前方案: {}", patch_label), "info", "ROOT");
        emit_log(
            &window,
            format!("目标刷入分区: {}", target_partition),
            "info",
            "ROOT",
        );
        emit_log(
            &window,
            format!("修补镜像路径: {}", patched_image_path.display()),
            "info",
            "ROOT",
        );

        if let Some(slot_suffix) = requested_slot_suffix.as_deref() {
            emit_log(
                &window,
                format!("目标刷入插槽: {}", slot_suffix),
                "info",
                "ROOT",
            );
        }

        let current_mode = detect_current_device_mode(adb_path, fastboot_path, adb_serial).await?;
        if current_mode != "device" {
            return Err(format!(
                "一键 Root 需要设备先处于 ADB 已连接状态，当前模式为: {}",
                current_mode
            ));
        }

        let (apatch_install_succeeded, apatch_install_error) =
            install_root_manager_apk(&window, adb_path, adb_serial, &apatch_apk_path, patch_label)
                .await;

        emit_log(&window, "准备重启到 Fastboot", "info", "ROOT");
        run_checked_adb(
            &window,
            adb_path,
            adb_serial,
            &["reboot".to_string(), "bootloader".to_string()],
            "ROOT",
            "重启到 Fastboot 失败",
        )
        .await?;

        let mut fastboot_serial =
            ensure_fastboot_mode(&window, adb_path, fastboot_path, adb_serial).await?;
        let flash_partition =
            build_partition_with_slot_suffix(&target_partition, requested_slot_suffix.as_deref());

        emit_log(
            &window,
            format!("开始在 Fastboot 模式刷入 {}", target_partition),
            "info",
            "ROOT",
        );
        let flashed_mode = match flash_patched_image(
            &window,
            fastboot_path,
            fastboot_serial.as_deref(),
            &target_partition,
            requested_slot_suffix.as_deref(),
            &patched_image_path,
        )
        .await
        {
            Ok(_) => "fastboot".to_string(),
            Err(_fastboot_error) => {
                emit_log(
                    &window,
                    format!(
                        "Fastboot 模式刷入 {} 失败，准备切换到 FastbootD 重试",
                        target_partition
                    ),
                    "warning",
                    "ROOT",
                );

                if let Err(error) = run_checked_fastboot(
                    &window,
                    fastboot_path,
                    fastboot_serial.as_deref(),
                    &["reboot".to_string(), "fastboot".to_string()],
                    "ROOT",
                    "从 Fastboot 重启到 FastbootD 失败",
                )
                .await
                {
                    reboot_system_from_fastboot_best_effort(
                        &window,
                        fastboot_path,
                        fastboot_serial.as_deref(),
                        "Fastboot 刷入失败，且无法切换到 FastbootD",
                    )
                    .await;
                    return Err(error);
                }

                if let Err(error) = wait_for_expected_mode(
                    adb_path,
                    fastboot_path,
                    adb_serial,
                    &["fastbootd"],
                    DEVICE_MODE_WAIT_ATTEMPTS,
                    DEVICE_MODE_WAIT_INTERVAL_MS,
                )
                .await
                {
                    reboot_system_from_fastboot_best_effort(
                        &window,
                        fastboot_path,
                        fastboot_serial.as_deref(),
                        "等待设备进入 FastbootD 超时",
                    )
                    .await;
                    return Err(error);
                }

                fastboot_serial = resolve_fastboot_serial(fastboot_path, adb_serial).await?;
                emit_log(
                    &window,
                    format!("开始在 FastbootD 模式刷入 {}", target_partition),
                    "info",
                    "ROOT",
                );

                match flash_patched_image(
                    &window,
                    fastboot_path,
                    fastboot_serial.as_deref(),
                    &target_partition,
                    requested_slot_suffix.as_deref(),
                    &patched_image_path,
                )
                .await
                {
                    Ok(_) => "fastbootd".to_string(),
                    Err(fastbootd_error) => {
                        emit_log(
                            &window,
                            format!(
                                "FastbootD 模式刷入 {} 仍然失败，请查看上一条 FLASH 日志",
                                target_partition
                            ),
                            "error",
                            "ROOT",
                        );
                        reboot_system_from_fastboot_best_effort(
                            &window,
                            fastboot_path,
                            fastboot_serial.as_deref(),
                            "FastbootD 刷入失败，按要求直接开机",
                        )
                        .await;
                        return Err(format!(
                            "刷入 {} 失败: {}",
                            target_partition, fastbootd_error
                        ));
                    }
                }
            }
        };

        emit_log(
            &window,
            format!("{} 分区刷入完成，准备重启系统", target_partition),
            "success",
            "ROOT",
        );
        run_checked_fastboot(
            &window,
            fastboot_path,
            fastboot_serial.as_deref(),
            &["reboot".to_string()],
            "ROOT",
            "重启回系统失败",
        )
        .await?;

        emit_log(
            &window,
            format!(
                "一键 Root 完成，已在 {} 模式刷入 {}",
                flashed_mode, target_partition
            ),
            "success",
            "DONE",
        );

        return Ok(OneKeyRootResponse {
            patch_mode: patch_mode.clone(),
            flashed_partition: flash_partition,
            flashed_mode,
            installed_magisk_path: String::new(),
            installed_apatch_path: apatch_apk_path.to_string_lossy().to_string(),
            installed_kernel_su_path: String::new(),
            patched_image_path: patched_image_path.to_string_lossy().to_string(),
            target_slot_suffix: requested_slot_suffix.unwrap_or_default(),
            magisk_install_succeeded: true,
            magisk_install_error: None,
            apatch_install_succeeded,
            apatch_install_error,
            kernel_su_install_succeeded: true,
            kernel_su_install_error: None,
        });
    }

    let patch_label = get_patch_mode_label(&patch_mode);
    if request.patched_image_path.trim().is_empty() {
        return Err("修补产物路径不能为空".to_string());
    }
    if request.magisk_apk_path.trim().is_empty() {
        return Err(format!("{} APK 路径不能为空", patch_label));
    }
    if target_partition.is_empty() {
        return Err("刷入分区不能为空".to_string());
    }
    if !is_supported_root_partition(&target_partition) {
        return Err(format!(
            "一键 Root 仅支持刷入 boot / init_boot，当前分区为: {}",
            target_partition
        ));
    }
    if !patched_image_path.exists() || !patched_image_path.is_file() {
        return Err(format!(
            "未找到修补后的镜像文件: {}",
            patched_image_path.display()
        ));
    }
    if !magisk_apk_path.exists() || !magisk_apk_path.is_file() {
        return Err(format!("未找到 {} APK: {}", patch_label, magisk_apk_path.display()));
    }

    emit_log(&window, "开始执行一键 Root 流程", "info", "ROOT");
    emit_log(&window, &format!("当前方案: {}", patch_label), "info", "ROOT");
    emit_log(
        &window,
        format!("目标刷入分区: {}", target_partition),
        "info",
        "ROOT",
    );
    emit_log(
        &window,
        format!("修补镜像路径: {}", patched_image_path.display()),
        "info",
        "ROOT",
    );

    if let Some(slot_suffix) = requested_slot_suffix.as_deref() {
        emit_log(
            &window,
            format!("目标刷入插槽: {}", slot_suffix),
            "info",
            "ROOT",
        );
    }

    let current_mode = detect_current_device_mode(adb_path, fastboot_path, adb_serial).await?;
    if current_mode != "device" {
        return Err(format!(
            "一键 Root 需要设备先处于 ADB 已连接状态，当前模式为: {}",
            current_mode
        ));
    }

    let (magisk_install_succeeded, magisk_install_error) =
        install_root_manager_apk(&window, adb_path, adb_serial, &magisk_apk_path, patch_label).await;

    emit_log(&window, "准备重启到 Fastboot", "info", "ROOT");
    run_checked_adb(
        &window,
        adb_path,
        adb_serial,
        &["reboot".to_string(), "bootloader".to_string()],
        "ROOT",
        "重启到 Fastboot 失败",
    )
    .await?;

    let mut fastboot_serial =
        ensure_fastboot_mode(&window, adb_path, fastboot_path, adb_serial).await?;
    let flash_partition =
        build_partition_with_slot_suffix(&target_partition, requested_slot_suffix.as_deref());

    emit_log(
        &window,
        format!("开始在 Fastboot 模式刷入 {}", target_partition),
        "info",
        "ROOT",
    );
    let flashed_mode = match flash_patched_image(
        &window,
        fastboot_path,
        fastboot_serial.as_deref(),
        &target_partition,
        requested_slot_suffix.as_deref(),
        &patched_image_path,
    )
    .await
    {
        Ok(_) => "fastboot".to_string(),
        Err(_fastboot_error) => {
            emit_log(
                &window,
                format!(
                    "Fastboot 模式刷入 {} 失败，准备切换到 FastbootD 重试",
                    target_partition
                ),
                "warning",
                "ROOT",
            );

            if let Err(error) = run_checked_fastboot(
                &window,
                fastboot_path,
                fastboot_serial.as_deref(),
                &["reboot".to_string(), "fastboot".to_string()],
                "ROOT",
                "从 Fastboot 重启到 FastbootD 失败",
            )
            .await
            {
                reboot_system_from_fastboot_best_effort(
                    &window,
                    fastboot_path,
                    fastboot_serial.as_deref(),
                    "Fastboot 刷入失败，且无法切换到 FastbootD",
                )
                .await;
                return Err(error);
            }

            if let Err(error) = wait_for_expected_mode(
                adb_path,
                fastboot_path,
                adb_serial,
                &["fastbootd"],
                DEVICE_MODE_WAIT_ATTEMPTS,
                DEVICE_MODE_WAIT_INTERVAL_MS,
            )
            .await
            {
                reboot_system_from_fastboot_best_effort(
                    &window,
                    fastboot_path,
                    fastboot_serial.as_deref(),
                    "等待设备进入 FastbootD 超时",
                )
                .await;
                return Err(error);
            }

            fastboot_serial = resolve_fastboot_serial(fastboot_path, adb_serial).await?;
            emit_log(
                &window,
                format!("开始在 FastbootD 模式刷入 {}", target_partition),
                "info",
                "ROOT",
            );

            match flash_patched_image(
                &window,
                fastboot_path,
                fastboot_serial.as_deref(),
                &target_partition,
                requested_slot_suffix.as_deref(),
                &patched_image_path,
            )
            .await
            {
                Ok(_) => "fastbootd".to_string(),
                Err(fastbootd_error) => {
                    emit_log(
                        &window,
                        format!(
                            "FastbootD 模式刷入 {} 仍然失败，请查看上一条 FLASH 日志",
                            target_partition
                        ),
                        "error",
                        "ROOT",
                    );
                    reboot_system_from_fastboot_best_effort(
                        &window,
                        fastboot_path,
                        fastboot_serial.as_deref(),
                        "FastbootD 刷入失败，按要求直接开机",
                    )
                    .await;
                    return Err(format!(
                        "刷入 {} 失败: {}",
                        target_partition, fastbootd_error
                    ));
                }
            }
        }
    };

    emit_log(
        &window,
        format!("{} 分区刷入完成，准备重启系统", target_partition),
        "success",
        "ROOT",
    );
    run_checked_fastboot(
        &window,
        fastboot_path,
        fastboot_serial.as_deref(),
        &["reboot".to_string()],
        "ROOT",
        "重启回系统失败",
    )
    .await?;

    emit_log(
        &window,
        format!(
            "一键 Root 完成，已在 {} 模式刷入 {}",
            flashed_mode, target_partition
        ),
        "success",
        "DONE",
    );

    Ok(OneKeyRootResponse {
        patch_mode: patch_mode.clone(),
        flashed_partition: flash_partition,
        flashed_mode,
        installed_magisk_path: magisk_apk_path.to_string_lossy().to_string(),
        installed_apatch_path: String::new(),
        installed_kernel_su_path: String::new(),
        patched_image_path: patched_image_path.to_string_lossy().to_string(),
        target_slot_suffix: requested_slot_suffix.unwrap_or_default(),
        magisk_install_succeeded,
        magisk_install_error,
        apatch_install_succeeded: true,
        apatch_install_error: None,
        kernel_su_install_succeeded: true,
        kernel_su_install_error: None,
    })
}
