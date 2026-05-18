use super::hyperos::{fetch_hyperos_catalog, fetch_hyperos_model_roms, HyperOsFansRomEntry};
use super::miuier::{fetch_miuier_catalog, fetch_miuier_model_roms, MiuierRomEntry};
use super::xfu::{fetch_xfu_catalog, fetch_xfu_model_roms, XfuRomEntry};
use super::xiaomirom::{
    fetch_xiaomirom_catalog, fetch_xiaomirom_model_roms, resolve_xiaomirom_download_urls,
    XiaomiRomEntry,
};
use crate::adb::core::{
    acquire_fastboot_command_guard, create_hidden_async_command, create_hidden_command,
    detect_device_state, dump_props_async_with_serial, get_bin_root_dir, get_link_dumper_path,
    AppPaths,
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

mod apatch_key;
mod auto_source;
mod common;
mod constants;
mod device_io;
mod flash_package;
mod partition;
mod patch_kits;
mod payload;
mod resources;
mod types;
mod workflow;

pub use types::{
    BootPatchAutoSourceResponse, BootPatchLog, BootPatchRequest, BootPatchResponse,
    BootPatchToolOption, BootPatchToolOptionsResponse, KernelSuRuntimeRequest,
    KernelSuRuntimeResponse, KernelSuVersionItem, OneKeyRootRequest, OneKeyRootResponse,
};

use apatch_key::*;
use auto_source::*;
use common::*;
use constants::*;
use device_io::*;
use flash_package::*;
use partition::*;
use patch_kits::*;
use payload::*;
use resources::*;
use types::{
    AutoRomCandidate, BootPatchTestOverrides, CodenameModelMapFile, ExtractedPatchKit,
    PatchPackageDeviceInfo,
};

#[tauri::command]
pub fn generate_apatch_super_key() -> Result<String, String> {
    apatch_key::generate_apatch_super_key_impl()
}

#[tauri::command]
pub async fn patch_boot_image(
    window: Window,
    paths: State<'_, AppPaths>,
    serial: Option<String>,
    request: BootPatchRequest,
) -> Result<BootPatchResponse, String> {
    workflow::patch_boot_image_impl(window, paths, serial, request).await
}

#[tauri::command]
pub async fn get_kernelsu_runtime(
    window: Window,
    paths: State<'_, AppPaths>,
    serial: Option<String>,
    request: KernelSuRuntimeRequest,
) -> Result<KernelSuRuntimeResponse, String> {
    workflow::get_kernelsu_runtime_impl(window, paths, serial, request).await
}

#[tauri::command]
pub fn get_kernelsu_versions(window: Window) -> Result<Vec<KernelSuVersionItem>, String> {
    workflow::get_kernelsu_versions_impl(window)
}

#[tauri::command]
pub fn get_boot_patch_tool_options(window: Window) -> Result<BootPatchToolOptionsResponse, String> {
    workflow::get_boot_patch_tool_options_impl(window)
}

#[tauri::command]
pub async fn prepare_boot_patch_auto_source(
    window: Window,
    paths: State<'_, AppPaths>,
    serial: Option<String>,
) -> Result<BootPatchAutoSourceResponse, String> {
    workflow::prepare_boot_patch_auto_source_impl(window, paths, serial).await
}

#[tauri::command]
pub async fn boot_patch_one_key_root(
    window: Window,
    paths: State<'_, AppPaths>,
    serial: Option<String>,
    request: OneKeyRootRequest,
) -> Result<OneKeyRootResponse, String> {
    workflow::boot_patch_one_key_root_impl(window, paths, serial, request).await
}
