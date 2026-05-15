use crate::adb::core::{adb_run_async_with_serial, create_hidden_async_command, AppPaths};
use crate::utils::process::{output_tracked_async_command, PROCESS_KIND_ADB_CLIENT};
use serde::{Deserialize, Serialize};
use tauri::State;

// ==============================
// 数据模型
// ==============================

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FileEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub is_symlink: bool,
    pub size: u64,
    pub modified: String,
    pub permissions: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StorageInfo {
    pub path: String,
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub available_bytes: u64,
    pub use_percent: u32,
}

// ==============================
// 剥离 ANSI 转义序列（ESC[ ... m / ESC[ ... 其他结束符）
// 解决 Android ls 输出颜色码导致乱码的问题
// ==============================
fn strip_ansi(s: &str) -> String {
    let bytes = s.as_bytes();
    let len = bytes.len();
    let mut out = String::with_capacity(len);
    let mut i = 0;
    while i < len {
        // ESC 字符 = 0x1b
        if bytes[i] == 0x1b {
            // 跳过 ESC[ ... 直到遇到字母（结束符）
            i += 1;
            if i < len && bytes[i] == b'[' {
                i += 1;
                // 跳过参数字节（0x30-0x3F）和中间字节（0x20-0x2F），直到最终字节（0x40-0x7E）
                while i < len && !(0x40..=0x7E).contains(&bytes[i]) {
                    i += 1;
                }
                i += 1; // 跳过最终字节
            }
            continue;
        }
        // 跳过其他控制字符，但必须保留 \n（换行，0x0A）和 \t（制表符，0x09）
        // \r（0x0D）直接丢弃（Windows 换行残留）
        if bytes[i] < 0x20 && bytes[i] != b'\t' && bytes[i] != b'\n' {
            i += 1;
            continue;
        }
        // 安全地推送 UTF-8 字符
        if bytes[i] < 0x80 {
            out.push(bytes[i] as char);
            i += 1;
        } else {
            // 多字节 UTF-8 字符：找到完整字符边界后推送
            let start = i;
            i += 1;
            while i < len && (bytes[i] & 0xC0) == 0x80 {
                i += 1;
            }
            if let Ok(ch) = std::str::from_utf8(&bytes[start..i]) {
                out.push_str(ch);
            }
        }
    }
    out
}

// ==============================
// Command: 列出指定目录内容
// ==============================
#[tauri::command]
pub async fn adb_list_dir(
    paths: State<'_, AppPaths>,
    serial: Option<String>,
    path: String,
) -> Result<Vec<FileEntry>, String> {
    let serial_str = serial.unwrap_or_default();
    let sr: Option<&str> = if serial_str.is_empty() {
        None
    } else {
        Some(serial_str.as_str())
    };

    // 规范化路径：末尾加 / 确保软链接目录（如 /sdcard -> /storage/emulated/0）被正确展开
    let normalized_path = if path.ends_with('/') {
        path.clone()
    } else {
        format!("{}/", path)
    };

    // 直接用 adb shell ls -la <path/>，末尾加 / 强制展开软链接目录
    // 不用 --color=never（busybox 不支持），改为 strip_ansi 后处理剥离颜色码
    let raw_output =
        adb_run_async_with_serial(&paths.adb, sr, &["shell", "ls", "-la", &normalized_path])
            .await
            .unwrap_or_default();

    // 剥离 ANSI 转义序列，彻底消除乱码
    let output = strip_ansi(&raw_output);

    // 错误检测
    if output.contains("No such file or directory") {
        return Err(format!("路径不存在: {}", path));
    }
    if output.contains("Permission denied") && output.trim().lines().count() <= 2 {
        return Err(format!("权限不足，无法访问: {}", path));
    }

    let mut entries: Vec<FileEntry> = Vec::new();

    for line in output.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with("total") || line.starts_with("Total") {
            continue;
        }
        // 必须以权限字符开头（d/-/l/c/b/p/s）
        let first_char = line.chars().next().unwrap_or(' ');
        if !matches!(first_char, 'd' | '-' | 'l' | 'c' | 'b' | 'p' | 's') {
            continue;
        }

        if let Some(entry) = parse_ls_line(line, &path) {
            if entry.name == "." || entry.name == ".." {
                continue;
            }
            entries.push(entry);
        }
    }

    // 文件夹排前，同类按名称字典序
    entries.sort_by(|a, b| {
        if a.is_dir != b.is_dir {
            return if a.is_dir {
                std::cmp::Ordering::Less
            } else {
                std::cmp::Ordering::Greater
            };
        }
        a.name.to_lowercase().cmp(&b.name.to_lowercase())
    });

    Ok(entries)
}

// ==============================
// 解析 `ls -la` 单行
// 支持两种格式：
//   标准: perms links owner group size date time name
//   简化: perms owner group size date time name
// ==============================
fn parse_ls_line(line: &str, parent_path: &str) -> Option<FileEntry> {
    // 先分割出权限字段（第一个空格前）
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 6 {
        return None;
    }

    let permissions = parts[0];
    let first_char = permissions.chars().next().unwrap_or('-');
    let is_dir = first_char == 'd';
    let is_symlink = first_char == 'l';

    // 判断是否为 8 字段格式（有 links 列）：第2列能否被解析为数字
    let has_links_col = parts[1].parse::<u64>().is_ok();

    // 根据格式确定 size 字段的列索引
    // 标准：0=perms 1=links 2=owner 3=group 4=size 5=date 6=time 7+=name
    // 简化：0=perms 1=owner 2=group 3=size 4=date 5=time 6+=name
    let (size_col, date_col, time_col, _name_col) = if has_links_col && parts.len() >= 8 {
        (4, 5, 6, 7)
    } else if !has_links_col && parts.len() >= 7 {
        (3, 4, 5, 6)
    } else {
        return None;
    };

    let size: u64 = parts
        .get(size_col)
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let date = parts.get(date_col).unwrap_or(&"");
    let time_part = parts.get(time_col).unwrap_or(&"");
    let modified = format!("{} {}", date, time_part);

    // 从原始行提取文件名：在 time 字段之后的所有内容
    // 用更健壮的方式：在 line 中找到 time_part 之后跳过
    let name_raw = extract_name_from_line(line, time_part)?;

    // 软链接处理 "name -> target"，只取 name
    let name = if is_symlink {
        name_raw
            .splitn(2, " -> ")
            .next()
            .unwrap_or(name_raw)
            .trim()
            .to_string()
    } else {
        name_raw.trim().to_string()
    };

    if name.is_empty() || name == "." || name == ".." {
        return None;
    }

    let normalized_parent = parent_path.trim_end_matches('/');
    let full_path = format!("{}/{}", normalized_parent, name);

    Some(FileEntry {
        name,
        path: full_path,
        is_dir,
        is_symlink,
        size: if is_dir { 0 } else { size },
        modified,
        permissions: permissions.to_string(),
    })
}

/// 从 ls -la 行中提取 time 字段之后的文件名原始字符串
fn extract_name_from_line<'a>(line: &'a str, time_part: &str) -> Option<&'a str> {
    // 在行中找到 time_part 的最后一次出现后跳过它和后面的空白
    if time_part.is_empty() {
        return line.split_whitespace().last();
    }
    // 扫描找到 time 字段后的位置
    let bytes = line.as_bytes();
    let tb = time_part.as_bytes();
    let tlen = tb.len();
    let len = bytes.len();
    let mut last_pos: Option<usize> = None;
    let mut i = 0;
    while i + tlen <= len {
        if &bytes[i..i + tlen] == tb {
            // 确认前面是空格（字段分隔）
            if i == 0 || bytes[i - 1] == b' ' {
                last_pos = Some(i + tlen);
            }
        }
        i += 1;
    }
    if let Some(pos) = last_pos {
        let mut j = pos;
        while j < len && bytes[j] == b' ' {
            j += 1;
        }
        if j < len {
            return Some(&line[j..]);
        }
    }
    // 退化
    line.split_whitespace().last()
}

// ==============================
// Command: 从手机拉取文件到电脑 (pull)
// ==============================
#[tauri::command]
pub async fn adb_pull_file(
    paths: State<'_, AppPaths>,
    serial: Option<String>,
    remote_path: String,
    local_path: String,
) -> Result<String, String> {
    let serial_str = serial.unwrap_or_default();
    let mut args: Vec<String> = Vec::new();

    if !serial_str.is_empty() {
        args.push("-s".to_string());
        args.push(serial_str.clone());
    }

    args.push("pull".to_string());
    args.push(remote_path.clone());
    args.push(local_path.clone());

    let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();

    let mut command = create_hidden_async_command(&paths.adb);
    let output = output_tracked_async_command(command.args(&args_ref), PROCESS_KIND_ADB_CLIENT)
        .await
        .map_err(|e| format!("执行 adb pull 失败: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

    if !output.status.success() && stderr.contains("error") {
        return Err(format!("pull 失败: {}", stderr));
    }

    let result = if stdout.is_empty() { stderr } else { stdout };
    Ok(result)
}

// ==============================
// Command: 从电脑推送文件到手机 (push)
// ==============================
#[tauri::command]
pub async fn adb_push_file(
    paths: State<'_, AppPaths>,
    serial: Option<String>,
    local_path: String,
    remote_path: String,
) -> Result<String, String> {
    let serial_str = serial.unwrap_or_default();
    let mut args: Vec<String> = Vec::new();

    if !serial_str.is_empty() {
        args.push("-s".to_string());
        args.push(serial_str.clone());
    }

    args.push("push".to_string());
    args.push(local_path.clone());
    args.push(remote_path.clone());

    let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();

    let mut command = create_hidden_async_command(&paths.adb);
    let output = output_tracked_async_command(command.args(&args_ref), PROCESS_KIND_ADB_CLIENT)
        .await
        .map_err(|e| format!("执行 adb push 失败: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

    if !output.status.success() && (stderr.contains("error") || stderr.contains("failed")) {
        return Err(format!("push 失败: {}", stderr));
    }

    let result = if stdout.is_empty() { stderr } else { stdout };
    Ok(result)
}

// ==============================
// Command: 删除手机文件/目录
// ==============================
#[tauri::command]
pub async fn adb_delete_item(
    paths: State<'_, AppPaths>,
    serial: Option<String>,
    remote_path: String,
) -> Result<String, String> {
    let serial_str = serial.unwrap_or_default();
    let sr: Option<&str> = if serial_str.is_empty() {
        None
    } else {
        Some(serial_str.as_str())
    };

    // 先判断是否为目录
    let stat_out =
        adb_run_async_with_serial(&paths.adb, sr, &["shell", "stat", "-c", "%F", &remote_path])
            .await
            .unwrap_or_default();

    let rm_args = if stat_out.contains("directory") {
        vec!["shell", "rm", "-rf", &remote_path]
    } else {
        vec!["shell", "rm", "-f", &remote_path]
    };

    let result = adb_run_async_with_serial(&paths.adb, sr, &rm_args).await?;

    // 如果返回非空且含错误信息则返回错误
    if result.contains("No such file") || result.contains("cannot remove") {
        return Err(format!("删除失败: {}", result));
    }

    Ok(format!("已删除: {}", remote_path))
}

// ==============================
// Command: 在手机上新建文件夹
// ==============================
#[tauri::command]
pub async fn adb_mkdir(
    paths: State<'_, AppPaths>,
    serial: Option<String>,
    remote_path: String,
) -> Result<String, String> {
    let serial_str = serial.unwrap_or_default();
    let sr: Option<&str> = if serial_str.is_empty() {
        None
    } else {
        Some(serial_str.as_str())
    };

    let result =
        adb_run_async_with_serial(&paths.adb, sr, &["shell", "mkdir", "-p", &remote_path]).await?;

    if result.contains("Permission denied") || result.contains("cannot") {
        return Err(format!("新建文件夹失败: {}", result));
    }

    Ok(format!("已创建: {}", remote_path))
}

// ==============================
// Command: 重命名/移动手机文件
// ==============================
#[tauri::command]
pub async fn adb_rename_item(
    paths: State<'_, AppPaths>,
    serial: Option<String>,
    old_path: String,
    new_path: String,
) -> Result<String, String> {
    let serial_str = serial.unwrap_or_default();
    let sr: Option<&str> = if serial_str.is_empty() {
        None
    } else {
        Some(serial_str.as_str())
    };

    let result =
        adb_run_async_with_serial(&paths.adb, sr, &["shell", "mv", &old_path, &new_path]).await?;

    if result.contains("No such file")
        || result.contains("cannot")
        || result.contains("Permission denied")
    {
        return Err(format!("重命名失败: {}", result));
    }

    Ok(format!("已重命名: {} → {}", old_path, new_path))
}

// ==============================
// Command: 获取存储信息
// ==============================
#[tauri::command]
pub async fn adb_get_storage_info(
    paths: State<'_, AppPaths>,
    serial: Option<String>,
    path: String,
) -> Result<StorageInfo, String> {
    let serial_str = serial.unwrap_or_default();
    let sr: Option<&str> = if serial_str.is_empty() {
        None
    } else {
        Some(serial_str.as_str())
    };

    let output = adb_run_async_with_serial(&paths.adb, sr, &["shell", "df", "-k", &path])
        .await
        .unwrap_or_default();

    // df -k 输出：Filesystem   1K-blocks   Used Available Use% Mounted on
    for line in output.lines().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 5 {
            let total_kb: u64 = parts[1].parse().unwrap_or(0);
            let used_kb: u64 = parts[2].parse().unwrap_or(0);
            let avail_kb: u64 = parts[3].parse().unwrap_or(0);
            let use_pct: u32 = parts[4].trim_end_matches('%').parse().unwrap_or(0);
            return Ok(StorageInfo {
                path: path.clone(),
                total_bytes: total_kb * 1024,
                used_bytes: used_kb * 1024,
                available_bytes: avail_kb * 1024,
                use_percent: use_pct,
            });
        }
    }

    Err(format!("无法获取 {} 的存储信息", path))
}
