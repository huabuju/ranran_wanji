use super::{
    rom_client::{build_http_client, fetch_text},
    rom_region::{
        get_region_label, infer_region_from_filename, infer_region_from_text,
        infer_region_from_version,
    },
};
use serde::Serialize;
use serde_json::Value;
use std::collections::{HashMap, HashSet};

const MIUIER_SOURCE_NAME: &str = "MIUIER";
const MIUIER_DEVICES_PAYLOAD_URL: &str = "https://roms.miuier.com/en-us/devices/_payload.json";
const MIUIER_DEVICE_PAGE_PREFIX: &str = "https://roms.miuier.com/en-us/devices/";
const MIUIER_DOWNLOAD_PREFIX: &str =
    "https://bkt-sgp-miui-ota-update-alisgp.oss-ap-southeast-1.aliyuncs.com";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MiuierCatalogItem {
    pub codename: String,
    pub name: String,
    pub brand: String,
    pub rom_count: Option<usize>,
    pub source_ref: String,
    pub page_url: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MiuierRomEntry {
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
    fn if_empty_then(self, fallback: impl FnOnce() -> String) -> String;
}

impl StringFallback for String {
    fn if_empty_then(self, fallback: impl FnOnce() -> String) -> String {
        if self.is_empty() {
            fallback()
        } else {
            self
        }
    }
}

struct PayloadStore {
    items: Vec<Value>,
}

impl PayloadStore {
    fn from_text(json_text: &str) -> Result<Self, String> {
        let items: Vec<Value> = serde_json::from_str(json_text)
            .map_err(|error| format!("解析 MIUIER payload JSON 失败: {}", error))?;
        if items.is_empty() {
            return Err("MIUIER payload is empty".to_string());
        }

        Ok(Self { items })
    }

    fn resolve<'a>(&'a self, value: &'a Value) -> Option<&'a Value> {
        match value {
            Value::Number(number) => {
                let index = number.as_i64()?;
                if index < 0 {
                    return None;
                }
                self.items
                    .get(index as usize)
                    .and_then(|item| self.resolve(item))
            }
            Value::Array(items) => {
                if let Some(marker) = items.first().and_then(Value::as_str) {
                    if matches!(marker, "ShallowReactive" | "Reactive") {
                        return items.get(1).and_then(|item| self.resolve(item));
                    }
                }
                Some(value)
            }
            _ => Some(value),
        }
    }

    fn data_object(&self) -> Result<&Value, String> {
        let meta = self
            .items
            .first()
            .and_then(Value::as_object)
            .ok_or_else(|| "MIUIER payload is missing root metadata".to_string())?;
        let mut node = meta
            .get("data")
            .and_then(|value| self.resolve(value))
            .ok_or_else(|| "MIUIER payload is missing data node".to_string())?;

        if let Some(object) = node.as_object() {
            if !(object.contains_key("brands")
                || object.contains_key("branches")
                || object.contains_key("codename"))
            {
                if let Some(inner) = object.values().next().and_then(|value| self.resolve(value)) {
                    node = inner;
                }
            }
        }

        if node.is_object() {
            Ok(node)
        } else {
            Err("MIUIER payload data node is invalid".to_string())
        }
    }

    fn get_field<'a>(&'a self, object_value: &'a Value, key: &str) -> Option<&'a Value> {
        let object = self.resolve(object_value)?.as_object()?;
        let raw_value = object.get(key)?;
        self.resolve(raw_value)
    }

    fn get_string(&self, value: Option<&Value>) -> String {
        match value.and_then(|item| self.resolve(item)) {
            Some(Value::String(text)) => text.trim().to_string(),
            Some(Value::Number(number)) => number.to_string(),
            Some(Value::Bool(flag)) => flag.to_string(),
            _ => String::new(),
        }
    }

    fn get_field_string(&self, object_value: &Value, key: &str) -> String {
        self.get_string(self.get_field(object_value, key))
    }

    fn get_locale_string(&self, object_value: &Value) -> String {
        self.get_field_string(object_value, "en-us")
            .if_empty_then(|| self.get_field_string(object_value, "zh-cn"))
            .if_empty_then(|| self.get_string(Some(object_value)))
    }

    fn get_array_items<'a>(&'a self, value: Option<&'a Value>) -> Vec<&'a Value> {
        let Some(Value::Array(items)) = value.and_then(|item| self.resolve(item)) else {
            return Vec::new();
        };

        items.iter().filter_map(|item| self.resolve(item)).collect()
    }

    fn get_field_array_items<'a>(&'a self, object_value: &'a Value, key: &str) -> Vec<&'a Value> {
        self.get_array_items(self.get_field(object_value, key))
    }
}

fn normalize_brand_key(value: &str) -> String {
    match value.trim().to_ascii_lowercase().as_str() {
        "xiaomi" => "xiaomi".to_string(),
        "redmi" => "redmi".to_string(),
        "poco" => "poco".to_string(),
        _ => "other".to_string(),
    }
}

fn score_catalog_name(name: &str) -> i32 {
    let normalized = name.trim().to_ascii_lowercase();
    if normalized.starts_with("xiaomi ") {
        return 5;
    }
    if normalized.starts_with("redmi ") {
        return 4;
    }
    if normalized.starts_with("poco ") {
        return 3;
    }
    if normalized.starts_with("mi ") || normalized.starts_with("mi") {
        return 2;
    }
    1
}

fn build_device_page_url(codename: &str) -> String {
    format!("{}{}", MIUIER_DEVICE_PAGE_PREFIX, codename.trim())
}

fn build_device_payload_url(codename: &str) -> String {
    format!(
        "{}{}/_payload.json",
        MIUIER_DEVICE_PAGE_PREFIX,
        codename.trim()
    )
}

fn build_download_url(version: &str, filename: &str) -> String {
    format!(
        "{}/{}/{}",
        MIUIER_DOWNLOAD_PREFIX,
        version.trim(),
        filename.trim()
    )
}

fn resolve_region_fields(branch_label: &str, version: &str, filename: &str) -> (String, String) {
    let region = infer_region_from_text(branch_label)
        .if_empty_then(|| infer_region_from_filename(filename))
        .if_empty_then(|| infer_region_from_version(version));
    let region_label = get_region_label(&region);
    (region, region_label)
}

fn is_visible_flag(value: Option<&Value>, store: &PayloadStore) -> bool {
    match value.and_then(|item| store.resolve(item)) {
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

fn parse_catalog_items(json_text: &str) -> Result<Vec<MiuierCatalogItem>, String> {
    let store = PayloadStore::from_text(json_text)?;
    let data = store.data_object()?;
    let mut catalog_map = HashMap::<String, MiuierCatalogItem>::new();

    for brand_item in store.get_field_array_items(data, "brands") {
        let brand = normalize_brand_key(&store.get_field_string(brand_item, "brand"));

        for series_item in store.get_field_array_items(brand_item, "series") {
            for device_item in store.get_field_array_items(series_item, "devices") {
                let codename = store.get_field_string(device_item, "code");
                if codename.is_empty() {
                    continue;
                }

                let name = store
                    .get_locale_string(device_item)
                    .if_empty_then(|| codename.clone());
                let next_item = MiuierCatalogItem {
                    codename: codename.clone(),
                    name: name.clone(),
                    brand: brand.clone(),
                    rom_count: None,
                    source_ref: build_device_payload_url(&codename),
                    page_url: build_device_page_url(&codename),
                };

                match catalog_map.get_mut(&codename) {
                    Some(current) => {
                        let current_score = score_catalog_name(&current.name);
                        let next_score = score_catalog_name(&name);
                        if next_score > current_score
                            || (next_score == current_score && name.len() < current.name.len())
                        {
                            *current = next_item;
                        }
                    }
                    None => {
                        catalog_map.insert(codename, next_item);
                    }
                }
            }
        }
    }

    let mut items = catalog_map.into_values().collect::<Vec<_>>();
    items.sort_by(|left, right| {
        left.name
            .cmp(&right.name)
            .then_with(|| left.codename.cmp(&right.codename))
    });
    Ok(items)
}

fn push_rom_entry(
    entries: &mut Vec<MiuierRomEntry>,
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
    let normalized_version = version.trim();
    let normalized_filename = filename.trim();
    if normalized_version.is_empty() || normalized_filename.is_empty() {
        return;
    }

    let entry_key = format!(
        "{}|{}|{}|{}",
        codename,
        normalized_version,
        flash_type.trim(),
        normalized_filename
    );
    if !seen_keys.insert(entry_key) {
        return;
    }

    let (region, region_label) =
        resolve_region_fields(branch_label, normalized_version, normalized_filename);

    entries.push(MiuierRomEntry {
        codename: codename.to_string(),
        name: device_name.to_string(),
        version: normalized_version.to_string(),
        android: android.trim().to_string(),
        region,
        region_label,
        flash_type: flash_type.trim().to_string(),
        date: date.trim().to_string(),
        size: String::new(),
        branch: branch_label.trim().to_string(),
        filename: normalized_filename.to_string(),
        url: vec![build_download_url(normalized_version, normalized_filename)],
        source_url: page_url.to_string(),
    });
}

fn parse_model_rom_entries(codename: &str, json_text: &str) -> Result<Vec<MiuierRomEntry>, String> {
    let store = PayloadStore::from_text(json_text)?;
    let data = store.data_object()?;
    let normalized_codename = store
        .get_field_string(data, "codename")
        .if_empty_then(|| codename.trim().to_string());
    if normalized_codename.is_empty() {
        return Err("MIUIER payload is missing codename".to_string());
    }

    let device_name = store
        .get_locale_string(data)
        .if_empty_then(|| normalized_codename.clone());
    let page_url = build_device_page_url(&normalized_codename);
    let mut entries = Vec::new();
    let mut seen_keys = HashSet::new();

    for branch_item in store.get_field_array_items(data, "branches") {
        if !is_visible_flag(store.get_field(branch_item, "show"), &store) {
            continue;
        }

        let branch_label = store
            .get_locale_string(branch_item)
            .if_empty_then(|| store.get_field_string(branch_item, "branch"));

        for link_item in store.get_field_array_items(branch_item, "links") {
            let version = store.get_field_string(link_item, "miui");
            let android = store.get_field_string(link_item, "android");
            let date = store.get_field_string(link_item, "release");
            let recovery = store.get_field_string(link_item, "recovery");
            let fastboot = store.get_field_string(link_item, "fastboot");

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
pub async fn fetch_miuier_catalog() -> Result<Vec<MiuierCatalogItem>, String> {
    let client = build_http_client(MIUIER_SOURCE_NAME)?;
    let json_text = fetch_text(&client, MIUIER_DEVICES_PAYLOAD_URL, MIUIER_SOURCE_NAME).await?;
    let items = parse_catalog_items(&json_text)?;

    if items.is_empty() {
        return Err("failed to parse any device from MIUIER catalog payload".to_string());
    }

    Ok(items)
}

#[tauri::command]
pub async fn fetch_miuier_model_roms(codename: String) -> Result<Vec<MiuierRomEntry>, String> {
    let trimmed_codename = codename.trim();
    if trimmed_codename.is_empty() {
        return Err("codename cannot be empty".to_string());
    }

    let client = build_http_client(MIUIER_SOURCE_NAME)?;
    let payload_url = build_device_payload_url(trimmed_codename);
    let json_text = fetch_text(&client, &payload_url, MIUIER_SOURCE_NAME).await?;
    let entries = parse_model_rom_entries(trimmed_codename, &json_text)?;

    if entries.is_empty() {
        return Err(format!(
            "failed to find any MIUIER rom entries for {}",
            trimmed_codename
        ));
    }

    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::{parse_catalog_items, parse_model_rom_entries};

    #[test]
    fn should_parse_catalog_items_from_payload() {
        let json = r#"
        [
          { "data": 1 },
          ["ShallowReactive", 2],
          { "$key": 3 },
          { "brands": 4 },
          [
            {
              "brand": "Xiaomi",
              "series": [
                {
                  "devices": [
                    { "code": "fuxi", "en-us": "Xiaomi 13" },
                    { "code": "alioth", "en-us": "Mi 11X" }
                  ]
                }
              ]
            },
            {
              "brand": "Redmi",
              "series": [
                {
                  "devices": [
                    { "code": "alioth", "en-us": "Redmi K40" }
                  ]
                }
              ]
            }
          ]
        ]
        "#;

        let items = parse_catalog_items(json).expect("catalog should parse");
        assert_eq!(items.len(), 2);
        assert!(items
            .iter()
            .any(|item| item.codename == "fuxi" && item.brand == "xiaomi"));
        assert!(items
            .iter()
            .any(|item| item.codename == "alioth" && item.name == "Redmi K40"));
    }

    #[test]
    fn should_parse_model_rom_entries_from_payload() {
        let json = r#"
        [
          { "data": 1 },
          ["ShallowReactive", 2],
          { "$key": 3 },
          {
            "codename": "fuxi",
            "en-us": "Xiaomi 13",
            "branches": [
              {
                "en-us": "China Mainland Stable",
                "show": "1",
                "links": [
                  {
                    "miui": "V14.0.5.0.UMCCNXM",
                    "android": "14.0",
                    "recovery": "miui_FUXI_V14.0.5.0.UMCCNXM_bb63991392_14.0.zip",
                    "fastboot": "fuxi_images_V14.0.5.0.UMCCNXM_20231103.0000.00_14.0_cn_b6df746cf6.tgz",
                    "release": "2023-11-07"
                  },
                  {
                    "miui": "V14.0.4.0.UMCCNXM",
                    "android": "14.0",
                    "recovery": "miui_FUXI_V14.0.4.0.UMCCNXM_f88642927a_14.0.zip",
                    "release": "2023-10-19"
                  }
                ]
              },
              {
                "en-us": "Hidden",
                "show": "0",
                "links": [
                  {
                    "miui": "V0",
                    "android": "14.0",
                    "recovery": "hidden.zip",
                    "release": "2023-01-01"
                  }
                ]
              }
            ]
          }
        ]
        "#;

        let items = parse_model_rom_entries("fuxi", json).expect("rom list should parse");
        assert_eq!(items.len(), 3);
        assert!(items
            .iter()
            .any(|item| item.flash_type == "card" && item.date == "2023-11-07"));
        assert!(items
            .iter()
            .any(|item| item.flash_type == "fastboot" && item.filename.ends_with(".tgz")));
        assert!(!items.iter().any(|item| item.filename == "hidden.zip"));
        assert!(items
            .iter()
            .any(|item| item.region == "china" && item.region_label == "中国版"));
    }
}
