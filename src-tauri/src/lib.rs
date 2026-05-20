pub mod adb;
pub mod commands;
pub mod models;
pub mod utils;

use commands::downloader::{
    cancel_download, get_download_tasks, open_download_folder, remove_download_task,
    retry_download, start_download, DownloadManager,
};
use std::sync::Arc;

// 从拆分出的 commands 模块导入给 Builder::generate_handler 使用
use adb::core::{get_adb_path, get_fastboot_path, get_scrcpy_path, AppPaths};
use commands::apps::{
    extract_apk, get_app_labels, get_disabled_packages, get_foreground_package, get_package_detail,
    get_packages, install_package, manage_package,
};
use commands::boot_patch::{
    boot_patch_one_key_root, generate_apatch_super_key, get_boot_patch_tool_options,
    get_kernelsu_runtime, prepare_boot_patch_auto_source,
    get_kernelsu_versions, patch_boot_image,
};
use commands::device::{
    adb_pair, adb_send_keyevent, check_file_exists, delete_sys_prop, device_reboot, get_app_status,
    get_connected_devices, get_device_info, get_device_resources, get_device_state,
    get_partition_info, get_sys_props, run_adb, run_adb_shell, run_fastboot, scan_adb_devices,
    set_sys_prop,
};
use commands::file_manager::{
    adb_delete_item, adb_get_storage_info, adb_list_dir, adb_mkdir, adb_pull_file, adb_push_file,
    adb_rename_item,
};
use commands::github_apk::fetch_github_apk_assets;
use commands::hyperos::{fetch_hyperos_catalog, fetch_hyperos_model_roms};
use commands::miuier::{fetch_miuier_catalog, fetch_miuier_model_roms};
use commands::rom_data::generate_codename_model_map;
use commands::runtime_assets::{get_tool_runtime_path, prepare_runtime_assets, warmup_platform_tools};
use commands::system::{
    clear_tool_cache, get_online_update_json, open_device_manager, open_driver_folder,
    open_folder_path, open_platform_tools_cmd, open_tool_dependency_folder, restart_app,
    start_scrcpy, stop_scrcpy, ExitCleanupState,
};
use commands::xfu::{fetch_xfu_catalog, fetch_xfu_model_roms};
use commands::xiaomirom::{
    fetch_xiaomirom_catalog, fetch_xiaomirom_model_roms, resolve_xiaomirom_download_urls,
};
use tauri::{Emitter, Manager};
use utils::process::init_process_tracker;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init());

    builder
        .setup(|app| {
            let app_handle = app.handle().clone();
            let adb = get_adb_path(&app_handle);
            let fastboot = get_fastboot_path(&app_handle);
            let scrcpy = get_scrcpy_path(&app_handle);
            init_process_tracker(&adb);

            // 启动时先显示主窗口，ADB 预热放到后台异步执行，避免启动页等待过久
            let handle_for_init = app_handle.clone();
            tauri::async_runtime::spawn(async move {
                if let Some(main) = handle_for_init.get_webview_window("main") {
                    if let Some(splash) = handle_for_init.get_webview_window("splashscreen") {
                        let _ = main.show();
                        let _ = splash.close();
                    }
                }
            });

            app.manage(AppPaths {
                adb,
                fastboot,
                scrcpy,
            });
            app.manage(ExitCleanupState::default());
            app.manage(Arc::new(DownloadManager::new()));

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_connected_devices,
            get_device_info,
            get_device_resources,
            get_partition_info,
            adb_send_keyevent,
            get_app_status,
            run_adb_shell,
            run_adb,
            device_reboot,
            get_device_state,
            run_fastboot,
            check_file_exists,
            open_platform_tools_cmd,
            open_device_manager,
            start_scrcpy,
            stop_scrcpy,
            open_driver_folder,
            open_folder_path,
            open_tool_dependency_folder,
            clear_tool_cache,
            restart_app,
            prepare_runtime_assets,
            warmup_platform_tools,
            get_tool_runtime_path,
            get_packages,
            get_disabled_packages,
            get_package_detail,
            get_foreground_package,
            manage_package,
            install_package,
            get_app_labels,
            extract_apk,
            start_download,
            retry_download,
            cancel_download,
            get_download_tasks,
            remove_download_task,
            open_download_folder,
            fetch_github_apk_assets,
            get_online_update_json,
            commands::payload::list_payload_partitions,
            commands::payload::extract_payload_partitions,
            adb_list_dir,
            adb_pull_file,
            adb_push_file,
            adb_delete_item,
            adb_mkdir,
            adb_rename_item,
            adb_get_storage_info,
            get_sys_props,
            set_sys_prop,
            delete_sys_prop,
            scan_adb_devices,
            adb_pair,
            patch_boot_image,
            generate_apatch_super_key,
            get_boot_patch_tool_options,
            get_kernelsu_runtime,
            get_kernelsu_versions,
            prepare_boot_patch_auto_source,
            boot_patch_one_key_root,
            fetch_hyperos_catalog,
            fetch_hyperos_model_roms,
            fetch_miuier_catalog,
            fetch_miuier_model_roms,
            generate_codename_model_map,
            fetch_xfu_catalog,
            fetch_xfu_model_roms,
            fetch_xiaomirom_catalog,
            fetch_xiaomirom_model_roms,
            resolve_xiaomirom_download_urls,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app_handle, event| match event {
            tauri::RunEvent::WindowEvent {
                label,
                event: tauri::WindowEvent::CloseRequested { api, .. },
                ..
            } if label == "main" => {
                api.prevent_close();
                let _ = app_handle.emit("app-closing", ());

                let app_handle = app_handle.clone();
                tauri::async_runtime::spawn(async move {
                    tokio::time::sleep(std::time::Duration::from_millis(120)).await;
                    app_handle.exit(0);
                });
            }
            tauri::RunEvent::ExitRequested { .. } => {
                let exit_cleanup_state = app_handle.state::<ExitCleanupState>();
                if exit_cleanup_state
                    .skip_cleanup_on_exit
                    .load(std::sync::atomic::Ordering::SeqCst)
                {
                    return;
                }

                let paths = app_handle.state::<AppPaths>();
                commands::system::cleanup_processes(&paths);

                // 解除文件锁定后，同步删除未完成的下载文件
                let manager = app_handle.state::<Arc<commands::downloader::DownloadManager>>();
                commands::downloader::cleanup_on_exit(&manager);
            }
            _ => {}
        });
}
