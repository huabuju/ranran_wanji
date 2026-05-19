use super::*;

pub(super) fn apk_entry_exists(zip: &mut ZipArchive<fs::File>, path: &str) -> bool {
    zip.by_name(path).is_ok()
}

pub(super) fn extract_zip_entry(
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

pub(super) fn collect_relative_files(
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

pub(super) async fn verify_remote_tmp_capability(
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

pub(super) async fn verify_remote_apatch_tmp_capability(
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

pub(super) async fn verify_remote_boot_inputs(
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

pub(super) async fn verify_remote_apatch_inputs(
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

pub(super) async fn verify_remote_folkpatch_tmp_capability(
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

pub(super) async fn verify_remote_folkpatch_inputs(
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

pub(super) async fn verify_remote_kernelsu_tmp_capability(
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

pub(super) async fn verify_remote_resukisu_tmp_capability(
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
                "cd {dir} && [ -s ./ksud ] && ls -l ./ksud && echo TMP_BIN_OK || (echo TMP_BIN_MISSING && exit 1)",
                dir = remote_work_dir
            ),
        ],
        "CHK",
        "ReSukiSU 修补所需文件不完整或不可执行",
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
        "ReSukiSU 所需的 ksud 无法在手机端执行，请检查 ABI 匹配或执行权限",
    )
    .await?;

    Ok(())
}

pub(super) async fn verify_remote_kernelsu_inputs(
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

pub(super) async fn verify_patch_output(
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

pub(super) fn extract_magisk_kit_from_apk(
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

pub(super) fn extract_apatch_kit_from_apk(
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

pub(super) fn extract_folkpatch_kit_from_apk(
    apk_path: &Path,
    output_root: &Path,
    device_abi: &str,
) -> Result<ExtractedPatchKit, String> {
    if !apk_path.exists() {
        return Err(format!("FolkPatch APK 不存在: {}", apk_path.display()));
    }
    if !apk_path.is_file() {
        return Err(format!(
            "FolkPatch APK 路径不是文件: {}",
            apk_path.display()
        ));
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

pub(super) fn extract_resukisu_kit_from_apk(
    apk_path: &Path,
    output_root: &Path,
    device_abi: &str,
) -> Result<ExtractedPatchKit, String> {
    if !apk_path.exists() {
        return Err(format!("ReSukiSU APK 不存在: {}", apk_path.display()));
    }
    if !apk_path.is_file() {
        return Err(format!("ReSukiSU APK 路径不是文件: {}", apk_path.display()));
    }

    let file = fs::File::open(apk_path).map_err(|e| format!("打开 ReSukiSU APK 失败: {}", e))?;
    let mut zip = ZipArchive::new(file).map_err(|e| format!("解析 ReSukiSU APK 失败: {}", e))?;

    let lib_folder = normalize_lib_folder(device_abi)?;
    let required_libs = [(format!("lib/{}/libksud.so", lib_folder), "ksud")];

    for (entry_name, _) in &required_libs {
        if !apk_entry_exists(&mut zip, entry_name) {
            return Err(format!("ReSukiSU APK 缺少 ABI 对应二进制: {}", entry_name));
        }
    }

    let extract_dir = output_root.join("resukisu-kit");
    recreate_local_dir(&extract_dir).map_err(|e| format!("创建 ReSukiSU 解包目录失败: {}", e))?;

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

pub(super) fn extract_kernelsu_kit_from_apk(
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
