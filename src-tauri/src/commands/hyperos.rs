use super::{
    rom_client::{build_http_client, fetch_text},
    rom_region::{
        get_region_label, infer_region_from_filename, infer_region_from_text,
        infer_region_from_version,
    },
};
use serde::Serialize;
use serde_json::Value;
use std::collections::HashSet;

const HYPEROS_FANS_SOURCE_NAME: &str = "HyperOS.fans";
const HYPEROS_FANS_DEVICES_URL: &str = "https://data.hyperos.fans/devices.json";
const HYPEROS_FANS_DEVICE_PAGE_PREFIX: &str = "https://hyperos.fans/zh/devices/";
const HYPEROS_FANS_DEVICE_JSON_PREFIX: &str = "https://data.hyperos.fans/devices/";
const HYPEROS_FANS_DOWNLOAD_PREFIX: &str =
    "https://bkt-sgp-miui-ota-update-alisgp.oss-ap-southeast-1.aliyuncs.com";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HyperOsFansCatalogItem {
    pub codename: String,
    pub name: String,
    pub brand: String,
    pub rom_count: Option<usize>,
    pub source_ref: String,
    pub page_url: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HyperOsFansRomEntry {
    pub codename: String,
    pub name: String,
    pub version: String,
    pub android: String,
    pub region: String,
    pub region_label: String,
    pub flash_type: String,
    pub date: String,
    pub size: String,
    pub branch: String,
    pub filename: String,
    pub url: Vec<String>,
    pub source_url: String,
}

trait StringFallback {
    fn or_else_if_empty(self, fallback: impl FnOnce() -> String) -> String;
}

impl StringFallback for String {
    fn or_else_if_empty(self, fallback: impl FnOnce() -> String) -> String {
        if self.is_empty() {
            fallback()
        } else {
            self
        }
    }
}

fn get_string(value: Option<&Value>) -> String {
    value
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim()
        .to_string()
}

fn get_locale_string(value: Option<&Value>) -> String {
    let Some(raw_value) = value else {
        return String::new();
    };

    get_string(raw_value.get("zh"))
        .or_else_if_empty(|| get_string(raw_value.get("en")))
        .or_else_if_empty(|| get_string(Some(raw_value)))
}

fn infer_brand_key(group: &Value, device_name: &str) -> String {
    let explicit_brand = get_string(group.get("brand")).to_ascii_lowercase();
    if !explicit_brand.is_empty() {
        return explicit_brand;
    }

    let lower_name = device_name.trim().to_ascii_lowercase();
    if lower_name.starts_with("xiaomi") || lower_name.starts_with("mi ") || lower_name == "mi" {
        return "xiaomi".to_string();
    }
    if lower_name.starts_with("redmi") {
        return "redmi".to_string();
    }
    if lower_name.starts_with("poco") {
        return "poco".to_string();
    }

    "other".to_string()
}

fn is_visible_flag(value: Option<&Value>) -> bool {
    match value {
        Some(Value::String(text)) => {
            let normalized = text.trim().to_ascii_lowercase();
            !matches!(normalized.as_str(), "" | "0" | "false" | "no")
        }
        Some(Value::Number(number)) => number.as_i64().unwrap_or(0) != 0,
        Some(Value::Bool(flag)) => *flag,
        Some(_) => true,
        None => true,
    }
}

fn collect_object_or_array_items<'a>(value: Option<&'a Value>) -> Vec<&'a Value> {
    match value {
        Some(Value::Array(items)) => items.iter().collect(),
        Some(Value::Object(items)) => items.values().collect(),
        _ => Vec::new(),
    }
}

fn build_device_page_url(codename: &str) -> String {
    format!("{}{}", HYPEROS_FANS_DEVICE_PAGE_PREFIX, codename)
}

fn build_device_json_url(codename: &str) -> String {
    format!("{}{}.json", HYPEROS_FANS_DEVICE_JSON_PREFIX, codename)
}

fn build_download_url(version: &str, filename: &str) -> String {
    format!(
        "{}/{}/{}",
        HYPEROS_FANS_DOWNLOAD_PREFIX,
        version.trim(),
        filename.trim()
    )
}

fn resolve_region_fields(branch_label: &str, version: &str, filename: &str) -> (String, String) {
    let region = infer_region_from_text(branch_label)
        .or_else_if_empty(|| infer_region_from_filename(filename))
        .or_else_if_empty(|| infer_region_from_version(version));
    let region_label = get_region_label(&region);
    (region, region_label)
}

fn parse_catalog_items(json_text: &str) -> Result<Vec<HyperOsFansCatalogItem>, String> {
    let root: Value = serde_json::from_str(json_text)
        .map_err(|error| format!("解析 HyperOS.fans 机型目录 JSON 失败: {}", error))?;
    let groups = root
        .as_object()
        .ok_or_else(|| "HyperOS.fans 机型目录格式无效".to_string())?;

    let mut items = Vec::new();

    for group in groups.values() {
        for device in collect_object_or_array_items(group.get("devices")) {
            let codename = get_string(device.get("code"));
            if codename.is_empty() {
                continue;
            }

            let name = get_locale_string(device.get("name")).or_else_if_empty(|| codename.clone());
            let brand = infer_brand_key(group, &name);

            items.push(HyperOsFansCatalogItem {
                codename: codename.clone(),
                name,
                brand,
                rom_count: None,
                source_ref: build_device_json_url(&codename),
                page_url: build_device_page_url(&codename),
            });
        }
    }

    items.sort_by(|left, right| {
        left.name
            .cmp(&right.name)
            .then_with(|| left.codename.cmp(&right.codename))
    });

    Ok(items)
}

fn push_rom_entry(
    entries: &mut Vec<HyperOsFansRomEntry>,
    seen_keys: &mut HashSet<String>,
    codename: &str,
    device_name: &str,
    branch_label: &str,
    page_url: &str,
    version: &str,
    android: &str,
    date: &str,
    flash_type: &str,
    filename: &str,
) {
    let normalized_filename = filename.trim();
    if normalized_filename.is_empty() {
        return;
    }

    let entry_key = format!(
        "{}|{}|{}|{}",
        codename,
        version.trim(),
        flash_type.trim(),
        normalized_filename
    );
    if !seen_keys.insert(entry_key) {
        return;
    }

    let (region, region_label) =
        resolve_region_fields(branch_label, version.trim(), normalized_filename);

    entries.push(HyperOsFansRomEntry {
        codename: codename.to_string(),
        name: device_name.to_string(),
        version: version.trim().to_string(),
        android: android.trim().to_string(),
        region,
        region_label,
        flash_type: flash_type.trim().to_string(),
        date: date.trim().to_string(),
        size: String::new(),
        branch: branch_label.trim().to_string(),
        filename: normalized_filename.to_string(),
        url: vec![build_download_url(version, normalized_filename)],
        source_url: page_url.to_string(),
    });
}

fn parse_model_rom_entries(
    codename: &str,
    json_text: &str,
) -> Result<Vec<HyperOsFansRomEntry>, String> {
    let root: Value = serde_json::from_str(json_text)
        .map_err(|error| format!("解析 HyperOS.fans 机型 ROM JSON 失败: {}", error))?;

    let normalized_codename =
        get_string(root.get("device")).or_else_if_empty(|| codename.trim().to_string());
    let device_name =
        get_locale_string(root.get("name")).or_else_if_empty(|| normalized_codename.clone());
    let page_url = build_device_page_url(&normalized_codename);

    let mut entries = Vec::new();
    let mut seen_keys = HashSet::new();

    for branch in collect_object_or_array_items(root.get("branches")) {
        if !is_visible_flag(branch.get("show")) {
            continue;
        }

        let branch_label = get_locale_string(branch.get("name"));

        for rom in collect_object_or_array_items(branch.get("roms")) {
            let version = get_string(rom.get("os"));
            if version.is_empty() {
                continue;
            }

            let android = get_string(rom.get("android"));
            let date = get_string(rom.get("release"));
            let recovery = get_string(rom.get("recovery"));
            let fastboot = get_string(rom.get("fastboot"));

            push_rom_entry(
                &mut entries,
                &mut seen_keys,
                &normalized_codename,
                &device_name,
                &branch_label,
                &page_url,
                &version,
                &android,
                &date,
                "card",
                &recovery,
            );
            push_rom_entry(
                &mut entries,
                &mut seen_keys,
                &normalized_codename,
                &device_name,
                &branch_label,
                &page_url,
                &version,
                &android,
                &date,
                "fastboot",
                &fastboot,
            );
        }
    }

    entries.sort_by(|left, right| {
        right
            .date
            .cmp(&left.date)
            .then_with(|| right.version.cmp(&left.version))
            .then_with(|| left.flash_type.cmp(&right.flash_type))
    });

    Ok(entries)
}

#[tauri::command]
pub async fn fetch_hyperos_catalog() -> Result<Vec<HyperOsFansCatalogItem>, String> {
    let client = build_http_client(HYPEROS_FANS_SOURCE_NAME)?;
    let json_text = fetch_text(&client, HYPEROS_FANS_DEVICES_URL, HYPEROS_FANS_SOURCE_NAME).await?;
    parse_catalog_items(&json_text)
}

#[tauri::command]
pub async fn fetch_hyperos_model_roms(
    codename: String,
) -> Result<Vec<HyperOsFansRomEntry>, String> {
    let trimmed_codename = codename.trim();
    if trimmed_codename.is_empty() {
        return Err("codename 不能为空".to_string());
    }

    let client = build_http_client(HYPEROS_FANS_SOURCE_NAME)?;
    let json_url = build_device_json_url(trimmed_codename);
    let json_text = fetch_text(&client, &json_url, HYPEROS_FANS_SOURCE_NAME).await?;
    let entries = parse_model_rom_entries(trimmed_codename, &json_text)?;

    if entries.is_empty() {
        return Err(format!(
            "未找到 {} 的 HyperOS.fans ROM 列表",
            trimmed_codename
        ));
    }

    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::{parse_catalog_items, parse_model_rom_entries};

    #[test]
    fn should_parse_catalog_items() {
        let json = r#"
        {
          "mi": {
            "brand": "Xiaomi",
            "devices": [
              {
                "code": "houji",
                "name": { "zh": "小米 14", "en": "Xiaomi 14" }
              }
            ]
          },
          "redmi": {
            "brand": "Redmi",
            "devices": [
              {
                "code": "duchamp",
                "name": { "zh": "Redmi K70E", "en": "Redmi K70E" }
              }
            ]
          }
        }
        "#;

        let items = parse_catalog_items(json).expect("catalog should parse");
        assert_eq!(items.len(), 2);
        assert!(items
            .iter()
            .any(|item| item.codename == "houji" && item.brand == "xiaomi"));
        assert!(items
            .iter()
            .any(|item| item.codename == "duchamp" && item.brand == "redmi"));
    }

    #[test]
    fn should_parse_model_rom_entries() {
        let json = r#"
        {
          "device": "houji",
          "name": { "zh": "小米 14", "en": "Xiaomi 14" },
          "branches": [
            {
              "name": { "zh": "小米澎湃OS 正式版", "en": "Stable" },
              "show": "1",
              "roms": {
                "OS3.0.300.6.WNCCNXM": {
                  "os": "OS3.0.300.6.WNCCNXM",
                  "android": "16.0",
                  "release": "2026-03-06",
                  "recovery": "houji-ota_full-OS3.0.300.6.WNCCNXM-user-16.0-acbd9ef007.zip",
                  "fastboot": "houji_images_OS3.0.300.6.WNCCNXM_20260306.0000.00_16.0_cn.tgz"
                }
              }
            }
          ]
        }
        "#;

        let items = parse_model_rom_entries("houji", json).expect("rom list should parse");
        assert_eq!(items.len(), 2);
        assert!(items
            .iter()
            .any(|item| item.flash_type == "card" && item.filename.ends_with(".zip")));
        assert!(items
            .iter()
            .any(|item| item.flash_type == "fastboot" && item.filename.ends_with(".tgz")));
        assert!(items
            .iter()
            .any(|item| item.region == "china" && item.region_label == "中国版"));
    }
}
