use crate::adb::core::get_bin_root_dir;
use super::hyperos::{fetch_hyperos_catalog, HyperOsFansCatalogItem};
use super::miuier::{fetch_miuier_catalog, MiuierCatalogItem};
use super::xfu::{fetch_xfu_catalog, XfuCatalogItem};
use super::xiaomirom::{fetch_xiaomirom_catalog, XiaomiRomCatalogItem};
use serde::Serialize;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tauri::{Manager, Window};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct CodenameModelMapEntry {
    codename: String,
    name: String,
    brand: String,
    sources: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct CodenameModelMapFile {
    generated_at: String,
    total: usize,
    entries: Vec<CodenameModelMapEntry>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateCodenameModelMapResponse {
    pub output_path: String,
    pub total: usize,
}

fn chrono_like_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    duration.as_secs().to_string()
}

fn normalize_text(value: &str) -> String {
    value.trim().to_string()
}

fn normalize_model_name(value: &str) -> String {
    let mut normalized = value.trim().to_string();
    if let Some((head, _)) = normalized.split_once('(') {
        normalized = head.trim().to_string();
    }
    if let Some((head, _)) = normalized.split_once(" / ") {
        normalized = head.trim().to_string();
    }
    if let Some((head, _)) = normalized.split_once('/') {
        normalized = head.trim().to_string();
    }
    normalized.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn normalize_codename_key(value: &str) -> String {
    value.trim().to_ascii_lowercase()
}

fn infer_brand_from_name(name: &str) -> String {
    let lower = name.trim().to_ascii_lowercase();
    if lower.starts_with("xiaomi") || lower.starts_with("mi ") || lower == "mi" {
        return "xiaomi".to_string();
    }
    if lower.starts_with("redmi") {
        return "redmi".to_string();
    }
    if lower.starts_with("poco") {
        return "poco".to_string();
    }
    "other".to_string()
}

fn score_model_name(name: &str) -> i32 {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return -100;
    }

    let mut score = 0i32;
    let lower = trimmed.to_ascii_lowercase();
    if lower.starts_with("xiaomi") || lower.starts_with("redmi") || lower.starts_with("poco") {
        score += 18;
    }
    if trimmed.contains(' ') {
        score += 8;
    }
    if trimmed.chars().any(|ch| ch.is_ascii_digit()) {
        score += 8;
    }
    if (8..=24).contains(&trimmed.len()) {
        score += 12;
    } else if trimmed.len() > 32 {
        score -= 12;
    }
    if trimmed.contains('/') {
        score -= 24;
    }
    if trimmed.contains('(') || trimmed.contains(')') {
        score -= 16;
    }
    if lower.contains("satellite") {
        score -= 12;
    }
    if lower.contains("special edition") || lower.contains("edition") {
        score -= 8;
    }
    if trimmed.eq_ignore_ascii_case("xiaomi")
        || trimmed.eq_ignore_ascii_case("redmi")
        || trimmed.eq_ignore_ascii_case("poco")
    {
        score -= 20;
    }
    score
}

fn get_rom_data_dir(window: &Window) -> PathBuf {
    get_bin_root_dir(&window.app_handle()).join("rom-data")
}

fn get_codename_model_map_path(window: &Window) -> PathBuf {
    get_rom_data_dir(window).join("codename-model-map.json")
}

fn merge_catalog_entry(
    merged: &mut HashMap<String, CodenameModelMapEntry>,
    source_label: &str,
    codename: &str,
    name: &str,
    brand: &str,
) {
    let normalized_codename = normalize_text(codename);
    if normalized_codename.is_empty() {
        return;
    }

    let normalized_name = normalize_model_name(name);
    let normalized_brand = {
        let trimmed = normalize_text(brand);
        if trimmed.is_empty() {
            infer_brand_from_name(&normalized_name)
        } else {
            trimmed
        }
    };

    let key = normalize_codename_key(&normalized_codename);
    let entry = merged.entry(key).or_insert_with(|| CodenameModelMapEntry {
        codename: normalized_codename.clone(),
        name: normalized_name.clone(),
        brand: normalized_brand.clone(),
        sources: Vec::new(),
    });

    if score_model_name(&normalized_name) > score_model_name(&entry.name) {
        entry.name = normalized_name.clone();
    }
    if entry.brand.trim().is_empty() || entry.brand == "other" {
        entry.brand = normalized_brand;
    }
    if !entry
        .sources
        .iter()
        .any(|item| item.eq_ignore_ascii_case(source_label))
    {
        entry.sources.push(source_label.to_string());
    }
}

fn merge_xiaomirom_catalog(
    merged: &mut HashMap<String, CodenameModelMapEntry>,
    items: Vec<XiaomiRomCatalogItem>,
) {
    for item in items {
        merge_catalog_entry(merged, "xiaomirom", &item.codename, &item.name, "");
    }
}

fn merge_hyperos_catalog(
    merged: &mut HashMap<String, CodenameModelMapEntry>,
    items: Vec<HyperOsFansCatalogItem>,
) {
    for item in items {
        merge_catalog_entry(merged, "hyperos_fans", &item.codename, &item.name, &item.brand);
    }
}

fn merge_miuier_catalog(
    merged: &mut HashMap<String, CodenameModelMapEntry>,
    items: Vec<MiuierCatalogItem>,
) {
    for item in items {
        merge_catalog_entry(merged, "miuier", &item.codename, &item.name, &item.brand);
    }
}

fn merge_xfu_catalog(
    merged: &mut HashMap<String, CodenameModelMapEntry>,
    items: Vec<XfuCatalogItem>,
) {
    for item in items {
        merge_catalog_entry(merged, "xfu", &item.codename, &item.name, &item.brand);
    }
}

async fn build_codename_model_map_file() -> Result<CodenameModelMapFile, String> {
    let mut merged = HashMap::<String, CodenameModelMapEntry>::new();

    let xiaomirom_catalog = fetch_xiaomirom_catalog().await?;
    merge_xiaomirom_catalog(&mut merged, xiaomirom_catalog);

    let hyperos_catalog = fetch_hyperos_catalog().await?;
    merge_hyperos_catalog(&mut merged, hyperos_catalog);

    let miuier_catalog = fetch_miuier_catalog().await?;
    merge_miuier_catalog(&mut merged, miuier_catalog);

    let xfu_catalog = fetch_xfu_catalog().await?;
    merge_xfu_catalog(&mut merged, xfu_catalog);

    let mut entries = merged.into_values().collect::<Vec<_>>();
    entries.sort_by(|left, right| {
        left.codename
            .cmp(&right.codename)
            .then_with(|| left.name.cmp(&right.name))
    });
    for entry in &mut entries {
        entry.sources.sort();
    }

    Ok(CodenameModelMapFile {
        generated_at: chrono_like_timestamp(),
        total: entries.len(),
        entries,
    })
}

pub async fn generate_codename_model_map_to_path(
    output_path: &Path,
) -> Result<GenerateCodenameModelMapResponse, String> {
    let output = build_codename_model_map_file().await?;
    let Some(output_dir) = output_path.parent() else {
        return Err(format!("机型映射表输出路径无效: {}", output_path.display()));
    };

    fs::create_dir_all(&output_dir)
        .map_err(|e| format!("创建 ROM 数据目录失败 {}: {}", output_dir.display(), e))?;

    let json = serde_json::to_string_pretty(&output)
        .map_err(|e| format!("序列化机型映射表失败: {}", e))?;
    fs::write(output_path, format!("{json}\n"))
        .map_err(|e| format!("写入机型映射表失败 {}: {}", output_path.display(), e))?;

    Ok(GenerateCodenameModelMapResponse {
        output_path: output_path.to_string_lossy().to_string(),
        total: output.total,
    })
}

#[tauri::command]
pub async fn generate_codename_model_map(window: Window) -> Result<GenerateCodenameModelMapResponse, String> {
    let output_path = get_codename_model_map_path(&window);
    generate_codename_model_map_to_path(&output_path).await
}
