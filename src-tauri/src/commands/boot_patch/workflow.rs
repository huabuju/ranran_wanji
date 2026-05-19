use super::*;

pub(super) fn parse_kernelsu_supported_kmis_output(output: &str) -> Vec<String> {
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

pub(super) fn parse_kernelsu_current_kmi_output(output: &str) -> String {
    output
        .lines()
        .map(str::trim)
        .find(|line| {
            !line.is_empty() && !line.starts_with("warning:") && !line.starts_with("Error:")
        })
        .unwrap_or_default()
        .to_string()
}

pub(super) async fn query_kernelsu_supported_kmis_via_device(
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
    fs::create_dir_all(&local_data_root)
        .map_err(|e| format!("创建 KernelSU 运行时目录失败: {}", e))?;

    let extract_result =
        extract_kernelsu_kit_from_apk(kernel_su_apk_path, &local_data_root, &device_abi);
    let extracted = match extract_result {
        Ok(extracted) => extracted,
        Err(error) => {
            let _ = fs::remove_dir_all(&local_data_root);
            return Err(error);
        }
    };

    let remote_work_dir = format!(
        "{}/kernelsu-runtime-{}",
        REMOTE_BOOT_PATCH_DIR, runtime_suffix
    );
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
                format!(
                    "cd {dir} && chmod 755 ./ksud ./magiskboot",
                    dir = remote_work_dir
                ),
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
                format!(
                    "cd {dir} && ./ksud boot-info supported-kmis 2>&1",
                    dir = remote_work_dir
                ),
            ],
            "CHK",
            "读取 KernelSU 支持的 KMI 列表失败",
        )
        .await?;

        let current_kmi =
            query_kernelsu_current_kmi_via_device(window, adb_path, serial, &remote_work_dir)
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

pub(super) async fn query_kernelsu_current_kmi_via_device(
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
            format!(
                "cd {dir} && ./ksud boot-info current-kmi 2>&1",
                dir = remote_work_dir
            ),
        ],
        "CHK",
        "读取 KernelSU 当前 KMI 失败",
    )
    .await?;

    Ok(parse_kernelsu_current_kmi_output(&output))
}

pub(super) fn build_patched_output_name(
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

pub(super) async fn patch_boot_image_impl(
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

pub(super) async fn get_kernelsu_runtime_impl(
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
        let (supported_kmis, ksud_current_kmi) = query_kernelsu_supported_kmis_via_device(
            &window,
            &paths.adb,
            serial_ref,
            &kernel_su_apk_path,
        )
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

pub(super) fn get_kernelsu_versions_impl(
    window: Window,
) -> Result<Vec<KernelSuVersionItem>, String> {
    list_kernelsu_versions(&window)
}

pub(super) fn get_boot_patch_tool_options_impl(
    window: Window,
) -> Result<BootPatchToolOptionsResponse, String> {
    let magisk_dir = get_magisk_patch_versions_dir(&window, PATCH_MODE_MAGISK);
    let magisk_alpha_dir = get_magisk_patch_versions_dir(&window, PATCH_MODE_MAGISK_ALPHA);
    let apatch_dir = get_apatch_patch_versions_dir(&window, PATCH_MODE_APATCH);
    let folk_patch_dir = get_apatch_patch_versions_dir(&window, PATCH_MODE_FOLKPATCH);
    let kernelsu_dir = get_kernel_patch_versions_dir(&window, PATCH_MODE_KERNELSU);
    let kernelsu_next_dir = get_kernel_patch_versions_dir(&window, PATCH_MODE_KERNELSU_NEXT);
    let sukisu_ultra_dir = get_kernel_patch_versions_dir(&window, PATCH_MODE_SUKISU_ULTRA);
    let resukisu_dir = get_kernel_patch_versions_dir(&window, PATCH_MODE_RESUKISU);
    let magisk_apk_options = list_apk_file_options(&magisk_dir)?;
    let magisk_alpha_apk_options = list_apk_file_options(&magisk_alpha_dir)?;
    let apatch_apk_options = list_apk_file_options(&apatch_dir)?;
    let folk_patch_apk_options = list_apk_file_options(&folk_patch_dir)?;
    let kernel_su_options = list_kernelsu_versions(&window)?;
    let kernel_su_next_options = list_kernel_patch_versions(&window, PATCH_MODE_KERNELSU_NEXT)?;
    let suki_su_ultra_options = list_kernel_patch_versions(&window, PATCH_MODE_SUKISU_ULTRA)?;
    let re_suki_su_options = list_kernel_patch_versions(&window, PATCH_MODE_RESUKISU)?;

    Ok(BootPatchToolOptionsResponse {
        magisk_apk_options,
        magisk_alpha_apk_options,
        apatch_apk_options,
        folk_patch_apk_options,
        kernel_su_options,
        kernel_su_next_options,
        suki_su_ultra_options,
        re_suki_su_options,
        magisk_apk_dir: magisk_dir.to_string_lossy().to_string(),
        magisk_alpha_apk_dir: magisk_alpha_dir.to_string_lossy().to_string(),
        apatch_apk_dir: apatch_dir.to_string_lossy().to_string(),
        folk_patch_apk_dir: folk_patch_dir.to_string_lossy().to_string(),
        kernel_su_dir: kernelsu_dir.to_string_lossy().to_string(),
        kernel_su_next_dir: kernelsu_next_dir.to_string_lossy().to_string(),
        suki_su_ultra_dir: sukisu_ultra_dir.to_string_lossy().to_string(),
        re_suki_su_dir: resukisu_dir.to_string_lossy().to_string(),
    })
}

pub(super) async fn prepare_boot_patch_auto_source_impl(
    window: Window,
    paths: State<'_, AppPaths>,
    serial: Option<String>,
) -> Result<BootPatchAutoSourceResponse, String> {
    let serial_ref = serial.as_deref().filter(|value| !value.is_empty());
    resolve_boot_patch_auto_source(&window, &paths.adb, serial_ref).await
}

pub(super) async fn boot_patch_one_key_root_impl(
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
        emit_log(
            &window,
            &format!("当前方案: {}", patch_label),
            "info",
            "ROOT",
        );
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
            return Err(format!(
                "未找到对应 {} APK: {}",
                patch_label,
                kernel_su_apk_path.display()
            ));
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
            return Err(format!(
                "未找到 {} APK: {}",
                patch_label,
                apatch_apk_path.display()
            ));
        }

        emit_log(&window, "开始执行一键 Root 流程", "info", "ROOT");
        emit_log(
            &window,
            &format!("当前方案: {}", patch_label),
            "info",
            "ROOT",
        );
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
        return Err(format!(
            "未找到 {} APK: {}",
            patch_label,
            magisk_apk_path.display()
        ));
    }

    emit_log(&window, "开始执行一键 Root 流程", "info", "ROOT");
    emit_log(
        &window,
        &format!("当前方案: {}", patch_label),
        "info",
        "ROOT",
    );
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
        install_root_manager_apk(&window, adb_path, adb_serial, &magisk_apk_path, patch_label)
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
