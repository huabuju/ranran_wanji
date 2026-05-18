use super::*;

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
pub(super) struct ExtractedPatchKit {
    pub(super) local_dir: PathBuf,
    pub(super) files: Vec<PathBuf>,
}

#[derive(Debug, Clone)]
pub(super) struct AutoRomCandidate {
    pub(super) source_key: &'static str,
    pub(super) source_label: &'static str,
    pub(super) rom_name: String,
    pub(super) rom_version: String,
    pub(super) rom_page_url: String,
    pub(super) filename: String,
    pub(super) urls: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub(super) struct PatchPackageDeviceInfo {
    pub(super) codename: String,
    pub(super) build_version: String,
    pub(super) model: String,
    pub(super) oem_name: String,
    pub(super) android_version: String,
}

#[derive(Debug, Clone, Default)]
pub(super) struct BootPatchTestOverrides {
    pub(super) codename: String,
    pub(super) build_version: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct CodenameModelMapFile {
    pub(super) entries: Vec<CodenameModelMapEntry>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct CodenameModelMapEntry {
    pub(super) codename: String,
    pub(super) name: String,
}
