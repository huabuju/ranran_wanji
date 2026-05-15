use super::{
    rom_client::{build_http_client, fetch_text},
    rom_region::{
        get_region_label, infer_region_from_filename, infer_region_from_text,
        infer_region_from_version,
    },
};
use serde::Serialize;
use std::collections::HashSet;

const XFU_SOURCE_NAME: &str = "XiaomiFirmwareUpdater";
const XFU_DEVICES_URL: &str =
    "https://raw.githubusercontent.com/XiaomiFirmwareUpdater/miui-updates-tracker/master/data/devices.yml";
const XFU_RSS_PREFIX: &str =
    "https://raw.githubusercontent.com/XiaomiFirmwareUpdater/miui-updates-tracker/master/rss/";
const XFU_PAGE_PREFIX: &str = "https://xmfirmwareupdater.com/miui/";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct XfuCatalogItem {
    pub codename: String,
    pub name: String,
    pub brand: String,
    pub rom_count: Option<usize>,
    pub source_ref: String,
    pub page_url: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct XfuRomEntry {
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

fn build_rss_url(codename: &str) -> String {
    format!("{}{}.xml", XFU_RSS_PREFIX, codename.trim())
}

fn build_page_url(codename: &str) -> String {
    format!("{}{}", XFU_PAGE_PREFIX, codename.trim())
}

fn infer_brand_key(name: &str) -> String {
    let lower_name = name.trim().to_ascii_lowercase();
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

fn normalize_catalog_name(value: &str) -> String {
    let mut normalized = value.trim().to_string();
    if normalized.is_empty() {
        return normalized;
    }

    let suffixes = [
        " China",
        " Global",
        " EEA",
        " India",
        " Indonesia",
        " Turkey",
        " Taiwan",
        " Russia",
        " Japan",
        " Singapore",
    ];

    loop {
        let mut changed = false;
        for suffix in suffixes {
            if normalized.ends_with(suffix) {
                normalized = normalized
                    .trim_end_matches(suffix)
                    .trim_end_matches('/')
                    .trim()
                    .to_string();
                changed = true;
                break;
            }
        }

        if !changed {
            break;
        }
    }

    normalized
}

fn should_keep_catalog_codename(codename: &str) -> bool {
    let trimmed = codename.trim();
    !trimmed.is_empty() && !trimmed.contains('_')
}

fn flush_catalog_item(
    items: &mut Vec<XfuCatalogItem>,
    seen: &mut HashSet<String>,
    codename: &str,
    raw_name: &str,
) {
    let trimmed_codename = codename.trim();
    if !should_keep_catalog_codename(trimmed_codename) || !seen.insert(trimmed_codename.to_string())
    {
        return;
    }

    let name = normalize_catalog_name(raw_name).if_empty_then(|| trimmed_codename.to_string());
    items.push(XfuCatalogItem {
        codename: trimmed_codename.to_string(),
        name: name.clone(),
        brand: infer_brand_key(&name),
        rom_count: None,
        source_ref: build_rss_url(trimmed_codename),
        page_url: build_page_url(trimmed_codename),
    });
}

fn parse_catalog_items(yaml_text: &str) -> Result<Vec<XfuCatalogItem>, String> {
    let mut items = Vec::new();
    let mut seen = HashSet::new();
    let mut current_codename = String::new();
    let mut current_name = String::new();

    for line in yaml_text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if !line.starts_with(' ') && trimmed.ends_with(':') {
            if !current_codename.is_empty() {
                flush_catalog_item(&mut items, &mut seen, &current_codename, &current_name);
            }

            current_codename = trimmed.trim_end_matches(':').trim().to_string();
            current_name.clear();
            continue;
        }

        if current_name.is_empty() && trimmed.starts_with("- ") {
            current_name = trimmed.trim_start_matches("- ").trim().to_string();
        }
    }

    if !current_codename.is_empty() {
        flush_catalog_item(&mut items, &mut seen, &current_codename, &current_name);
    }

    if items.is_empty() {
        return Err(
            "failed to parse any XiaomiFirmwareUpdater device from devices.yml".to_string(),
        );
    }

    items.sort_by(|left, right| {
        left.name
            .cmp(&right.name)
            .then_with(|| left.codename.cmp(&right.codename))
    });
    Ok(items)
}

fn extract_xml_blocks(xml_text: &str, tag: &str) -> Vec<String> {
    let start_tag = format!("<{}>", tag);
    let end_tag = format!("</{}>", tag);
    let mut result = Vec::new();
    let mut search_start = 0usize;

    while let Some(start_offset) = xml_text[search_start..].find(&start_tag) {
        let content_start = search_start + start_offset + start_tag.len();
        let Some(end_offset) = xml_text[content_start..].find(&end_tag) else {
            break;
        };
        let content_end = content_start + end_offset;
        result.push(xml_text[content_start..content_end].to_string());
        search_start = content_end + end_tag.len();
    }

    result
}

fn extract_xml_text(xml_text: &str, tag: &str) -> String {
    extract_xml_blocks(xml_text, tag)
        .into_iter()
        .next()
        .unwrap_or_default()
        .trim()
        .to_string()
}

fn decode_html_entities(value: &str) -> String {
    value
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&apos;", "'")
        .replace("&nbsp;", " ")
        .replace("&amp;", "&")
}

fn strip_html_tags(value: &str) -> String {
    let mut result = String::with_capacity(value.len());
    let mut inside_tag = false;

    for ch in value.chars() {
        match ch {
            '<' => inside_tag = true,
            '>' => inside_tag = false,
            _ if !inside_tag => result.push(ch),
            _ => {}
        }
    }

    result.trim().to_string()
}

fn normalize_description_html(value: &str) -> String {
    decode_html_entities(value)
        .replace("<p<b>", "<p><b>")
        .replace("<br />", "\n")
        .replace("<br/>", "\n")
        .replace("<br>", "\n")
        .replace("</p>", "\n")
        .replace("\r\n", "\n")
        .replace('\r', "\n")
}

fn extract_description_field(description_html: &str, label: &str) -> String {
    let marker = format!("<b>{}</b>", label);
    let Some(start) = description_html.find(&marker) else {
        return String::new();
    };

    let content = &description_html[start + marker.len()..];
    let line_end = content.find('\n').unwrap_or(content.len());
    strip_html_tags(content[..line_end].trim())
}

fn parse_branch_from_description(description_html: &str, flash_label: &str) -> String {
    let plain_text = strip_html_tags(description_html);
    let Some(start) = plain_text.find("New ") else {
        return String::new();
    };

    let content = &plain_text[start + 4..];
    let Some(end) = content.find(" update available!") else {
        return String::new();
    };

    content[..end]
        .trim()
        .trim_end_matches(flash_label)
        .trim()
        .to_string()
}

fn parse_rfc822_date(value: &str) -> String {
    let parts = value.split_whitespace().collect::<Vec<_>>();
    if parts.len() < 4 {
        return value.trim().to_string();
    }

    let day = parts.get(1).copied().unwrap_or_default();
    let month = match parts.get(2).copied().unwrap_or_default() {
        "Jan" => "01",
        "Feb" => "02",
        "Mar" => "03",
        "Apr" => "04",
        "May" => "05",
        "Jun" => "06",
        "Jul" => "07",
        "Aug" => "08",
        "Sep" => "09",
        "Oct" => "10",
        "Nov" => "11",
        "Dec" => "12",
        _ => "",
    };
    let year = parts.get(3).copied().unwrap_or_default();

    if year.len() == 4 && month.len() == 2 {
        format!("{}-{}-{:0>2}", year, month, day)
    } else {
        value.trim().to_string()
    }
}

fn parse_title_parts(title: &str) -> (String, String, String, String) {
    let trimmed = title.trim();
    let Some(content) = trimmed.strip_prefix("MIUI ") else {
        return (String::new(), String::new(), String::new(), String::new());
    };
    let Some((left, device_name)) = content.split_once(" update for ") else {
        return (String::new(), String::new(), String::new(), String::new());
    };

    if let Some(version) = left.strip_suffix(" Recovery") {
        return (
            version.trim().to_string(),
            "card".to_string(),
            "Recovery".to_string(),
            device_name.trim().to_string(),
        );
    }
    if let Some(version) = left.strip_suffix(" Fastboot") {
        return (
            version.trim().to_string(),
            "fastboot".to_string(),
            "Fastboot".to_string(),
            device_name.trim().to_string(),
        );
    }

    (
        left.trim().to_string(),
        String::new(),
        String::new(),
        device_name.trim().to_string(),
    )
}

fn extract_filename_from_url(url_text: &str) -> String {
    let trimmed = url_text.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    trimmed
        .rsplit('/')
        .next()
        .map(|value| value.trim().to_string())
        .unwrap_or_default()
}

fn resolve_region_fields(
    device_name: &str,
    title: &str,
    version: &str,
    filename: &str,
) -> (String, String) {
    let region = infer_region_from_text(device_name)
        .if_empty_then(|| infer_region_from_text(title))
        .if_empty_then(|| infer_region_from_filename(filename))
        .if_empty_then(|| infer_region_from_version(version));
    let region_label = get_region_label(&region);
    (region, region_label)
}

fn parse_model_rom_entries(codename: &str, xml_text: &str) -> Result<Vec<XfuRomEntry>, String> {
    let trimmed_codename = codename.trim();
    if trimmed_codename.is_empty() {
        return Err("codename cannot be empty".to_string());
    }

    let page_url = build_page_url(trimmed_codename);
    let mut entries = Vec::new();
    let mut seen = HashSet::new();

    for item_xml in extract_xml_blocks(xml_text, "item") {
        let title = extract_xml_text(&item_xml, "title");
        let link = extract_xml_text(&item_xml, "link");
        let description_raw = extract_xml_text(&item_xml, "description");
        let pub_date = extract_xml_text(&item_xml, "pubDate");
        let description_html = normalize_description_html(&description_raw);

        let (title_version, flash_type, flash_label, title_device_name) = parse_title_parts(&title);
        let device_name = extract_description_field(&description_html, "Device:")
            .if_empty_then(|| title_device_name.clone())
            .if_empty_then(|| trimmed_codename.to_string());
        let version_and_android = extract_description_field(&description_html, "Version:");
        let version_parts = version_and_android
            .split('|')
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .collect::<Vec<_>>();
        let version = version_parts
            .first()
            .cloned()
            .unwrap_or_default()
            .if_empty_then(|| title_version.clone());
        let android = version_parts.get(1).cloned().unwrap_or_default();
        let size = extract_description_field(&description_html, "Size:");
        let branch = parse_branch_from_description(&description_html, &flash_label);
        let filename = extract_filename_from_url(&link);
        let date = parse_rfc822_date(&pub_date);
        let (region, region_label) =
            resolve_region_fields(&device_name, &title, &version, &filename);

        if version.is_empty() || flash_type.is_empty() || link.is_empty() || filename.is_empty() {
            continue;
        }

        let entry_key = format!(
            "{}|{}|{}|{}|{}",
            trimmed_codename, device_name, version, flash_type, link
        );
        if !seen.insert(entry_key) {
            continue;
        }

        entries.push(XfuRomEntry {
            codename: trimmed_codename.to_string(),
            name: device_name,
            version,
            android,
            region,
            region_label,
            flash_type,
            date,
            size,
            branch,
            filename,
            url: vec![link],
            source_url: page_url.clone(),
        });
    }

    if entries.is_empty() {
        return Err(format!(
            "failed to parse any XiaomiFirmwareUpdater rom entries for {}",
            trimmed_codename
        ));
    }

    entries.sort_by(|left, right| {
        right
            .date
            .cmp(&left.date)
            .then_with(|| right.version.cmp(&left.version))
            .then_with(|| left.flash_type.cmp(&right.flash_type))
            .then_with(|| left.name.cmp(&right.name))
    });

    Ok(entries)
}

#[tauri::command]
pub async fn fetch_xfu_catalog() -> Result<Vec<XfuCatalogItem>, String> {
    let client = build_http_client(XFU_SOURCE_NAME)?;
    let yaml_text = fetch_text(&client, XFU_DEVICES_URL, XFU_SOURCE_NAME).await?;
    parse_catalog_items(&yaml_text)
}

#[tauri::command]
pub async fn fetch_xfu_model_roms(codename: String) -> Result<Vec<XfuRomEntry>, String> {
    let trimmed_codename = codename.trim();
    if trimmed_codename.is_empty() {
        return Err("codename cannot be empty".to_string());
    }

    let client = build_http_client(XFU_SOURCE_NAME)?;
    let rss_url = build_rss_url(trimmed_codename);
    let xml_text = fetch_text(&client, &rss_url, XFU_SOURCE_NAME).await?;
    parse_model_rom_entries(trimmed_codename, &xml_text)
}

#[cfg(test)]
mod tests {
    use super::{parse_catalog_items, parse_model_rom_entries};

    #[test]
    fn should_parse_catalog_items_from_devices_yml() {
        let yaml = r#"
fuxi:
- Xiaomi 13 China
- FUXI
fuxi_global:
- Xiaomi 13 Global
- FUXIGlobal
alioth:
- Redmi K40 China
- ALIOTH
"#;

        let items = parse_catalog_items(yaml).expect("catalog should parse");
        assert_eq!(items.len(), 2);
        assert!(items
            .iter()
            .any(|item| item.codename == "fuxi" && item.name == "Xiaomi 13"));
        assert!(items.iter().all(|item| !item.codename.contains('_')));
    }

    #[test]
    fn should_parse_rom_entries_from_rss() {
        let xml = r#"
<rss>
  <channel>
    <item>
      <title>MIUI OS3.0.3.0.WMCCNXM Recovery update for Xiaomi 13 China</title>
      <link>https://bigota.d.miui.com/OS3.0.3.0.WMCCNXM/fuxi-ota_full-OS3.0.3.0.WMCCNXM-user-16.0-72296ffab2.zip</link>
      <description>&lt;p&lt;b&gt;New Stable Recovery update available!&lt;/b&gt;&lt;/p&gt;
&lt;p&gt;&lt;b&gt;Device:&lt;/b&gt; Xiaomi 13 China&lt;/p&gt;
&lt;p&gt;&lt;b&gt;Codename:&lt;/b&gt; fuxi&lt;/p&gt;
&lt;p&gt;&lt;b&gt;Version:&lt;/b&gt; OS3.0.3.0.WMCCNXM | 16.0&lt;/p&gt;
&lt;p&gt;&lt;b&gt;Size:&lt;/b&gt; 7.0 GB&lt;/p&gt;
&lt;p&gt;&lt;b&gt;Download:&lt;/b&gt; &lt;a href='https://bigota.d.miui.com/OS3.0.3.0.WMCCNXM/fuxi-ota_full-OS3.0.3.0.WMCCNXM-user-16.0-72296ffab2.zip'&gt;Here&lt;/a&gt;&lt;/p&gt;</description>
      <pubDate>Fri, 30 Jan 2026 00:00:00 +0000</pubDate>
    </item>
    <item>
      <title>MIUI OS3.0.4.0.WMCMIXM Fastboot update for Xiaomi 13 Global</title>
      <link>https://bkt-sgp-miui-ota-update-alisgp.oss-ap-southeast-1.aliyuncs.com/OS3.0.4.0.WMCMIXM/fuxi_global_images_OS3.0.4.0.WMCMIXM_20260226.0000.00_16.0_global_31f8847656.tgz</link>
      <description>&lt;p&lt;b&gt;New Stable Fastboot update available!&lt;/b&gt;&lt;/p&gt;
&lt;p&gt;&lt;b&gt;Device:&lt;/b&gt; Xiaomi 13 Global&lt;/p&gt;
&lt;p&gt;&lt;b&gt;Codename:&lt;/b&gt; fuxi&lt;/p&gt;
&lt;p&gt;&lt;b&gt;Version:&lt;/b&gt; OS3.0.4.0.WMCMIXM | 16.0&lt;/p&gt;
&lt;p&gt;&lt;b&gt;Size:&lt;/b&gt; 8.1 GB&lt;/p&gt;
&lt;p&gt;&lt;b&gt;Download:&lt;/b&gt; &lt;a href='https://bkt-sgp-miui-ota-update-alisgp.oss-ap-southeast-1.aliyuncs.com/OS3.0.4.0.WMCMIXM/fuxi_global_images_OS3.0.4.0.WMCMIXM_20260226.0000.00_16.0_global_31f8847656.tgz'&gt;Here&lt;/a&gt;&lt;/p&gt;</description>
      <pubDate>Thu, 26 Feb 2026 00:00:00 +0000</pubDate>
    </item>
  </channel>
</rss>
"#;

        let items = parse_model_rom_entries("fuxi", xml).expect("rom list should parse");
        assert_eq!(items.len(), 2);
        assert!(items.iter().any(|item| {
            item.flash_type == "card"
                && item.version == "OS3.0.3.0.WMCCNXM"
                && item.date == "2026-01-30"
                && item.region == "china"
        }));
        assert!(items.iter().any(|item| {
            item.flash_type == "fastboot"
                && item.branch == "Stable"
                && item.filename.ends_with(".tgz")
                && item.region_label == "国际版"
        }));
    }
}
