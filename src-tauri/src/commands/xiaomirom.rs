use super::{
    rom_client::{build_http_client, fetch_text},
    rom_region::{
        get_region_label, infer_region_from_filename, infer_region_from_fragment,
        infer_region_from_text,
    },
};
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use url::Url;

const XIAOMIROM_EN_SERIES_URL: &str = "https://xiaomirom.com/en/series/";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct XiaomiRomCatalogItem {
    pub codename: String,
    pub name: String,
    pub rom_count: usize,
    pub series_url: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct XiaomiRomEntry {
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

#[derive(Debug, Clone)]
struct AnchorItem {
    href: String,
    text: String,
    position: usize,
}

fn decode_html_entities(value: &str) -> String {
    value
        .replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&#34;", "\"")
        .replace("&#39;", "'")
        .replace("&apos;", "'")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&nbsp;", " ")
        .replace("&#8211;", "-")
        .replace("&#8212;", "-")
        .replace("&#8230;", "...")
}

fn normalize_whitespace(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn strip_tags(value: &str) -> String {
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

    normalize_whitespace(&decode_html_entities(result.trim()))
}

fn extract_tag_attribute(tag_html: &str, attribute: &str) -> Option<String> {
    let pattern = format!("{}=", attribute);
    let attr_start = tag_html.find(&pattern)? + pattern.len();
    let rest = &tag_html[attr_start..];
    let mut chars = rest.chars();
    let quote = chars.next()?;

    if quote == '"' || quote == '\'' {
        let value_start = attr_start + quote.len_utf8();
        let value_end_rel = tag_html[value_start..].find(quote)?;
        return Some(tag_html[value_start..value_start + value_end_rel].to_string());
    }

    let value_end_rel = rest
        .find(|ch: char| ch.is_whitespace() || ch == '>')
        .unwrap_or(rest.len());
    Some(rest[..value_end_rel].to_string())
}

fn to_absolute_url(base_url: &str, href: &str) -> Option<String> {
    let trimmed = href.trim();
    if trimmed.is_empty() || trimmed.starts_with("javascript:") || trimmed.starts_with("mailto:") {
        return None;
    }

    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        return Some(trimmed.to_string());
    }

    if trimmed.starts_with("//") {
        return Some(format!("https:{}", trimmed));
    }

    let base = Url::parse(base_url).ok()?;
    base.join(trimmed).ok().map(|url| url.to_string())
}

fn extract_window_open_url(script: &str) -> Option<String> {
    let window_open_pos = script.find("window.open(")?;
    let rest = script[window_open_pos + "window.open(".len()..].trim_start();
    let quote = rest.chars().next()?;

    if quote != '"' && quote != '\'' {
        return None;
    }

    let value_start = quote.len_utf8();
    let value_end_rel = rest[value_start..].find(quote)?;
    Some(decode_html_entities(
        &rest[value_start..value_start + value_end_rel],
    ))
}

fn collect_tag_items(base_url: &str, html: &str, tag_name: &str) -> Vec<AnchorItem> {
    let mut items = Vec::new();
    let mut cursor = 0usize;
    let open_tag = format!("<{}", tag_name);
    let close_tag = format!("</{}>", tag_name);

    while let Some(tag_start_rel) = html[cursor..].find(&open_tag) {
        let tag_start = cursor + tag_start_rel;
        let Some(tag_end_rel) = html[tag_start..].find('>') else {
            break;
        };
        let tag_end = tag_start + tag_end_rel;
        let Some(tag_close_rel) = html[tag_end + 1..].find(&close_tag) else {
            cursor = tag_end + 1;
            continue;
        };
        let tag_close = tag_end + 1 + tag_close_rel;
        let tag_html = &html[tag_start..=tag_end];
        let raw_target = extract_tag_attribute(tag_html, "href").or_else(|| {
            extract_tag_attribute(tag_html, "onclick")
                .and_then(|script| extract_window_open_url(&script))
        });
        let Some(raw_target) = raw_target else {
            cursor = tag_close + close_tag.len();
            continue;
        };
        let Some(href) = to_absolute_url(base_url, &raw_target) else {
            cursor = tag_close + close_tag.len();
            continue;
        };
        let text = strip_tags(&html[tag_end + 1..tag_close]);

        items.push(AnchorItem {
            href,
            text,
            position: tag_start,
        });

        cursor = tag_close + close_tag.len();
    }

    items
}

fn collect_anchor_items(base_url: &str, html: &str) -> Vec<AnchorItem> {
    collect_tag_items(base_url, html, "a")
}

fn collect_clickable_items(base_url: &str, html: &str) -> Vec<AnchorItem> {
    let mut items = collect_tag_items(base_url, html, "a");
    items.extend(collect_tag_items(base_url, html, "button"));
    items.sort_by(|left, right| left.position.cmp(&right.position));
    items
}

fn extract_path_slug(url: &str, marker: &str) -> String {
    let Ok(parsed) = Url::parse(url) else {
        return String::new();
    };
    let path = parsed.path();
    let Some(index) = path.find(marker) else {
        return String::new();
    };
    let tail = &path[index + marker.len()..];
    tail.trim_matches('/')
        .split('/')
        .next()
        .unwrap_or("")
        .to_string()
}

fn extract_badge_count(html: &str, start: usize) -> usize {
    let slice_end = (start + 500).min(html.len());
    let slice = &html[start..slice_end];
    let Some(badge_pos) = slice.find("badge") else {
        return 0;
    };
    let badge_slice = &slice[badge_pos..];
    let text = strip_tags(badge_slice);
    let mut number = String::new();

    for ch in text.chars() {
        if ch.is_ascii_digit() {
            number.push(ch);
        } else if !number.is_empty() {
            break;
        }
    }

    number.parse::<usize>().unwrap_or(0)
}

fn parse_catalog_from_series_index(html: &str) -> Vec<XiaomiRomCatalogItem> {
    let anchors = collect_anchor_items(XIAOMIROM_EN_SERIES_URL, html);
    let mut catalog_map = HashMap::<String, XiaomiRomCatalogItem>::new();

    for anchor in anchors {
        if !anchor.href.contains("/en/series/") {
            continue;
        }

        let codename = extract_path_slug(&anchor.href, "/en/series/");
        if codename.is_empty() {
            continue;
        }

        if anchor.href.trim_end_matches('/') == XIAOMIROM_EN_SERIES_URL.trim_end_matches('/') {
            continue;
        }

        let name = anchor.text.trim();
        if name.is_empty() || name.eq_ignore_ascii_case("Read more") {
            continue;
        }

        let next_count = extract_badge_count(html, anchor.position);
        let entry = catalog_map
            .entry(codename.clone())
            .or_insert_with(|| XiaomiRomCatalogItem {
                codename: codename.clone(),
                name: name.to_string(),
                rom_count: next_count,
                series_url: anchor.href.clone(),
            });

        if entry.name.len() < name.len() {
            entry.name = name.to_string();
        }
        if entry.rom_count == 0 && next_count > 0 {
            entry.rom_count = next_count;
        }
    }

    let mut items = catalog_map.into_values().collect::<Vec<_>>();
    items.sort_by(|left, right| {
        left.name
            .cmp(&right.name)
            .then_with(|| left.codename.cmp(&right.codename))
    });
    items
}

fn parse_model_article_urls(series_url: &str, html: &str, codename: &str) -> Vec<String> {
    let codename_key = format!("-{}-", codename.trim().to_ascii_lowercase());
    let mut seen = HashSet::new();
    let mut urls = Vec::new();

    for anchor in collect_anchor_items(series_url, html) {
        let href_lower = anchor.href.to_ascii_lowercase();
        if !href_lower.contains("/en/rom/") {
            continue;
        }
        if !href_lower.contains(&codename_key) {
            continue;
        }
        if seen.insert(anchor.href.clone()) {
            urls.push(anchor.href);
        }
    }

    urls
}

fn extract_table_cells(row_html: &str) -> Vec<String> {
    let mut cells = Vec::new();
    let mut cursor = 0usize;

    while let Some(cell_start_rel) = row_html[cursor..].find("<td") {
        let cell_start = cursor + cell_start_rel;
        let Some(tag_end_rel) = row_html[cell_start..].find('>') else {
            break;
        };
        let tag_end = cell_start + tag_end_rel;
        let Some(cell_close_rel) = row_html[tag_end + 1..].find("</td>") else {
            break;
        };
        let cell_close = tag_end + 1 + cell_close_rel;
        cells.push(strip_tags(&row_html[tag_end + 1..cell_close]));
        cursor = cell_close + 5;
    }

    cells
}

fn normalize_flash_type(value: &str) -> String {
    let lower = value.trim().to_ascii_lowercase();
    if lower.contains("fastboot") {
        return "fastboot".to_string();
    }
    if lower.contains("firmware") {
        return "firmware".to_string();
    }
    "card".to_string()
}

fn infer_branch_from_url(url: &str) -> String {
    let lower = url.to_ascii_lowercase();
    if lower.contains("-weekly-") || lower.contains(".dev") {
        return "weekly".to_string();
    }
    if lower.contains("-stable-") {
        return "stable".to_string();
    }
    String::new()
}

fn extract_filename_before_download(rest_html: &str, download_url: &str) -> String {
    let Some(anchor_href_pos) = rest_html.find(download_url) else {
        return String::new();
    };
    let preview_start = anchor_href_pos.saturating_sub(260);
    let preview = strip_tags(&rest_html[preview_start..anchor_href_pos]);
    let Some(filename_part) = preview.split('|').next_back() else {
        return String::new();
    };
    let normalized = filename_part.trim();

    if normalized.contains(".zip") || normalized.contains(".tgz") || normalized.contains(".gz") {
        return normalized.to_string();
    }

    String::new()
}

fn extract_heading_text(html: &str, tag_name: &str) -> Option<String> {
    let open_tag = format!("<{}", tag_name);
    let close_tag = format!("</{}>", tag_name);
    let tag_start = html.find(&open_tag)?;
    let tag_end = tag_start + html[tag_start..].find('>')?;
    let content_start = tag_end + 1;
    let content_end = content_start + html[content_start..].find(&close_tag)?;
    Some(strip_tags(&html[content_start..content_end]))
}

fn infer_region_from_source_url(source_url: &str) -> String {
    let Ok(parsed) = Url::parse(source_url) else {
        return String::new();
    };

    let fragment = parsed.fragment().unwrap_or("");
    if fragment.is_empty() {
        return String::new();
    }

    infer_region_from_fragment(fragment)
}

fn infer_region_from_article_html(html: &str) -> String {
    let heading = extract_heading_text(html, "h1")
        .or_else(|| extract_heading_text(html, "title"))
        .unwrap_or_default();

    infer_region_from_text(&heading)
}

fn parse_rom_entries_from_article(article_url: &str, html: &str) -> Vec<XiaomiRomEntry> {
    let mut entries = Vec::new();
    let mut cursor = 0usize;
    let article_region = infer_region_from_article_html(html);

    while let Some(row_anchor_rel) = html[cursor..].find("<tbody><tr>") {
        let row_anchor = cursor + row_anchor_rel + "<tbody><tr>".len();
        let Some(row_end_rel) = html[row_anchor..].find("</tr>") else {
            break;
        };
        let row_end = row_anchor + row_end_rel;
        let row_html = &html[row_anchor..row_end];
        let cells = extract_table_cells(row_html);

        if cells.len() < 7 {
            cursor = row_end + 5;
            continue;
        }

        let rest_start = row_end + 5;
        let next_row_start = html[rest_start..]
            .find("<tbody><tr>")
            .map(|index| rest_start + index)
            .unwrap_or(html.len());
        let rest_html = &html[rest_start..next_row_start];
        let download_anchor = collect_anchor_items(article_url, rest_html)
            .into_iter()
            .find(|item| item.href.contains("/download/"));

        let Some(download_anchor) = download_anchor else {
            cursor = next_row_start;
            continue;
        };

        let filename = extract_filename_before_download(rest_html, &download_anchor.href);
        let source_url = download_anchor.href.clone();
        let branch = infer_branch_from_url(&source_url);
        let region = infer_region_from_source_url(&source_url);
        let region = if region.is_empty() {
            let filename_region = infer_region_from_filename(&filename);
            if filename_region.is_empty() {
                article_region.clone()
            } else {
                filename_region
            }
        } else {
            region
        };
        let region_label = get_region_label(&region);

        entries.push(XiaomiRomEntry {
            name: cells[0].clone(),
            codename: cells[1].clone(),
            region,
            region_label,
            flash_type: normalize_flash_type(&cells[2]),
            version: cells[3].clone(),
            android: cells[4].clone(),
            size: cells[5].clone(),
            date: cells[6].clone(),
            branch,
            filename,
            url: vec![source_url.clone()],
            source_url,
        });

        cursor = next_row_start;
    }

    entries
}

fn is_social_or_noise_host(host: &str) -> bool {
    matches!(
        host,
        "facebook.com"
            | "www.facebook.com"
            | "twitter.com"
            | "x.com"
            | "www.linkedin.com"
            | "pinterest.com"
            | "telegram.me"
            | "service.weibo.com"
            | "hm.baidu.com"
            | "www.googletagmanager.com"
            | "pagead2.googlesyndication.com"
            | "cdnjs.cloudflare.com"
    )
}

fn score_download_anchor(url: &str, text: &str) -> i32 {
    let Ok(parsed) = Url::parse(url) else {
        return -10;
    };
    let Some(host) = parsed.host_str() else {
        return -10;
    };
    if is_social_or_noise_host(host) {
        return -10;
    }

    let mut score = 0i32;
    let lower_url = url.to_ascii_lowercase();
    let lower_text = text.to_ascii_lowercase();

    if lower_url.contains(".zip")
        || lower_url.contains(".tgz")
        || lower_url.contains(".tar")
        || lower_url.contains(".gz")
    {
        score += 5;
    }
    if host.contains("miui.com") || host.contains("xiaomi.com") || host.contains("mi.com") {
        score += 4;
    }
    if lower_text.contains("download")
        || lower_text.contains("official")
        || lower_text.contains("server")
        || lower_text.contains("mirror")
    {
        score += 3;
    }
    if host.contains("xiaomirom.com") {
        if parsed.path().starts_with("/go/") {
            score += 2;
        } else {
            score -= 4;
        }
    }

    score
}

fn find_fragment_section<'a>(html: &'a str, fragment: &str) -> Option<&'a str> {
    let patterns = [
        format!("id={}", fragment),
        format!("id=\"{}\"", fragment),
        format!("id='{}'", fragment),
    ];

    let start = patterns
        .iter()
        .filter_map(|pattern| html.find(pattern))
        .min()?;
    let tail = &html[start..];
    let mut candidates = Vec::new();

    if let Some(index) = tail[1..].find("<h2") {
        candidates.push(index + 1);
    }
    if let Some(index) = tail[1..].find("<h3") {
        candidates.push(index + 1);
    }
    if let Some(index) = tail[1..].find("<section") {
        candidates.push(index + 1);
    }

    let end = candidates
        .into_iter()
        .min()
        .map(|index| start + index)
        .unwrap_or(html.len());
    Some(&html[start..end])
}

fn resolve_download_urls_from_html(page_url: &str, html: &str) -> Vec<String> {
    let fragment = Url::parse(page_url)
        .ok()
        .and_then(|url| url.fragment().map(|value| value.to_string()));
    let target_scope = fragment
        .as_deref()
        .and_then(|value| find_fragment_section(html, value))
        .unwrap_or(html);

    let collect_urls = |scope_html: &str| {
        let mut ranked = Vec::<(i32, String)>::new();
        let mut seen = HashSet::new();

        for anchor in collect_clickable_items(page_url, scope_html) {
            let score = score_download_anchor(&anchor.href, &anchor.text);
            if score < 3 {
                continue;
            }
            if seen.insert(anchor.href.clone()) {
                ranked.push((score, anchor.href));
            }
        }

        ranked.sort_by(|left, right| right.0.cmp(&left.0));
        ranked.into_iter().map(|(_, url)| url).collect::<Vec<_>>()
    };

    let scoped_urls = collect_urls(target_scope);
    if !scoped_urls.is_empty() {
        return scoped_urls;
    }

    collect_urls(html)
}

#[tauri::command]
pub async fn fetch_xiaomirom_catalog() -> Result<Vec<XiaomiRomCatalogItem>, String> {
    let client = build_http_client("XiaomiROM")?;
    let html = fetch_text(&client, XIAOMIROM_EN_SERIES_URL, "XiaomiROM").await?;
    let catalog = parse_catalog_from_series_index(&html);

    if catalog.is_empty() {
        return Err("未能从 XiaomiROM 系列页解析出任何机型".to_string());
    }

    Ok(catalog)
}

#[tauri::command]
pub async fn fetch_xiaomirom_model_roms(series_url: String) -> Result<Vec<XiaomiRomEntry>, String> {
    let trimmed_series_url = series_url.trim();
    if trimmed_series_url.is_empty() {
        return Err("seriesUrl 不能为空".to_string());
    }

    let client = build_http_client("XiaomiROM")?;
    let series_html = fetch_text(&client, trimmed_series_url, "XiaomiROM").await?;
    let codename = extract_path_slug(trimmed_series_url, "/en/series/");
    if codename.is_empty() {
        return Err(format!(
            "无法从系列地址中解析 codename: {}",
            trimmed_series_url
        ));
    }

    let article_urls = parse_model_article_urls(trimmed_series_url, &series_html, &codename);
    if article_urls.is_empty() {
        return Err(format!("未找到 {} 的 ROM 文章列表", codename));
    }

    let mut entry_map = HashMap::<String, XiaomiRomEntry>::new();

    for article_url in article_urls {
        let article_html = fetch_text(&client, &article_url, "XiaomiROM").await?;
        for entry in parse_rom_entries_from_article(&article_url, &article_html) {
            let key = format!(
                "{}|{}|{}|{}",
                entry.codename, entry.version, entry.flash_type, entry.source_url
            );
            entry_map.entry(key).or_insert(entry);
        }
    }

    let mut entries = entry_map.into_values().collect::<Vec<_>>();
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
pub async fn resolve_xiaomirom_download_urls(page_url: String) -> Result<Vec<String>, String> {
    let trimmed_page_url = page_url.trim();
    if trimmed_page_url.is_empty() {
        return Err("pageUrl 不能为空".to_string());
    }

    let client = build_http_client("XiaomiROM")?;
    let clean_page_url = Url::parse(trimmed_page_url)
        .map(|mut url| {
            let fragment = url.fragment().map(|value| value.to_string());
            url.set_fragment(None);
            (url.to_string(), fragment)
        })
        .map_err(|error| format!("下载页地址格式无效: {}", error))?;

    let html = fetch_text(&client, &clean_page_url.0, "XiaomiROM").await?;
    let resolve_target_url = match clean_page_url.1 {
        Some(fragment) => format!("{}#{}", clean_page_url.0, fragment),
        None => clean_page_url.0.clone(),
    };
    let urls = resolve_download_urls_from_html(&resolve_target_url, &html);

    if urls.is_empty() {
        return Err(format!(
            "未能从 XiaomiROM 下载页解析出直链: {}",
            trimmed_page_url
        ));
    }

    Ok(urls)
}

#[cfg(test)]
mod tests {
    use super::{
        parse_catalog_from_series_index, parse_rom_entries_from_article,
        resolve_download_urls_from_html,
    };

    #[test]
    fn should_parse_catalog_items() {
        let html = r#"
        <div class="card">
          <a href="/en/series/houji/">Xiaomi 14</a>
          <span class="badge badge-primary badge-pill">371</span>
        </div>
        <div class="card">
          <a href="/en/series/aurora/">Xiaomi 15</a>
          <span class="badge badge-primary badge-pill">29</span>
        </div>
        "#;

        let items = parse_catalog_from_series_index(html);
        assert_eq!(items.len(), 2);
        assert!(items
            .iter()
            .any(|item| item.codename == "houji" && item.rom_count == 371));
    }

    #[test]
    fn should_parse_rom_entries() {
        let html = r#"
        <tbody><tr>
          <td>Xiaomi 14</td>
          <td>houji</td>
          <td>Fastboot ROM</td>
          <td>OS3.0.302.0.WNCCNXM</td>
          <td>16</td>
          <td>10.3 GB</td>
          <td>2026-04-03</td>
        </tr></tbody>
        <p><strong>Download OS3.0.302.0.WNCCNXM</strong></p>
        <p>houji_images_OS3.0.302.0.WNCCNXM_20260403.0000.00_16.0_cn_e980aa1348.tgz |
        <a href="/en/download/xiaomi-14-houji-stable-OS3.0.302.0.WNCCNXM/#china-fastboot">Download</a></p>
        "#;

        let items = parse_rom_entries_from_article(
            "https://xiaomirom.com/en/rom/xiaomi-14-houji-china-fastboot-recovery-rom/",
            html,
        );

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].codename, "houji");
        assert_eq!(items[0].flash_type, "fastboot");
        assert_eq!(items[0].branch, "stable");
        assert_eq!(items[0].region, "china");
        assert_eq!(items[0].region_label, "中国版");
        assert!(items[0].source_url.contains("/en/download/"));
    }

    #[test]
    fn should_preserve_unknown_region_key_as_label() {
        let html = r#"
        <tbody><tr>
          <td>Xiaomi 14</td>
          <td>houji</td>
          <td>Recovery ROM</td>
          <td>OS3.0.302.0.WNCCNXM</td>
          <td>16</td>
          <td>7.3 GB</td>
          <td>2026-04-03</td>
        </tr></tbody>
        <p><strong>Download OS3.0.302.0.WNCCNXM</strong></p>
        <p>houji_ota_full-OS3.0.302.0.WNCCNXM-user-16.0.zip |
        <a href="/en/download/xiaomi-14-houji-stable-OS3.0.302.0.WNCCNXM/#latinamerica-recovery">Download</a></p>
        "#;

        let items = parse_rom_entries_from_article(
            "https://xiaomirom.com/en/rom/xiaomi-14-houji-fastboot-recovery-rom/",
            html,
        );

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].region, "latinamerica");
        assert_eq!(items[0].region_label, "latinamerica");
    }

    #[test]
    fn should_resolve_download_urls() {
        let html = r#"
        <h2 id="china-fastboot">China Fastboot</h2>
        <p><a href="https://bigota.d.miui.com/V16.0.1/file_a.tgz">Official Link 1</a></p>
        <p><a href="https://cdnorg.d.miui.com/V16.0.1/file_a.tgz">Official Link 2</a></p>
        <h2 id="china-recovery">China Recovery</h2>
        <p><a href="https://bigota.d.miui.com/V16.0.1/file_b.zip">Official Link 1</a></p>
        "#;

        let urls = resolve_download_urls_from_html(
            "https://xiaomirom.com/en/download/xiaomi-14-houji-stable-OS3.0.302.0.WNCCNXM/#china-fastboot",
            html,
        );

        assert_eq!(urls.len(), 2);
        assert!(urls[0].ends_with(".tgz"));
    }

    #[test]
    fn should_resolve_download_urls_from_onclick_buttons() {
        let html = r#"
        <div class="card">
          <div class="card-header"><h2 id="china-recovery">China Recovery</h2></div>
          <div class="card-body">
            <dd class="col-sm-9">
              <div class="download-group btn-group me-3">
                <button type="button" class="btn btn-warning" onclick="window.open('https://bigota.d.miui.com/OS1.0.5.0/file_a.zip','_blank');">Xiaomi Server</button>
                <div class="btn-group dropdown" role="group">
                  <button type="button" class="dropdown-toggle btn btn-warning dropdown-toggle-split"></button>
                  <div class="dropdown-menu bg-warning">
                    <a class="dropdown-item" onclick="window.open('https://bn.d.miui.com/OS1.0.5.0/file_a.zip','_blank');">Xiaomi Server #2</a>
                    <a class="dropdown-item" onclick="window.open('https://cdnorg.d.miui.com/OS1.0.5.0/file_a.zip','_blank');">Xiaomi Server #3</a>
                  </div>
                </div>
              </div>
            </dd>
          </div>
        </div>
        "#;

        let urls = resolve_download_urls_from_html(
            "https://xiaomirom.com/en/download/mi-10-umi-stable-OS1.0.5.0.TJBCNXM/#china-recovery",
            html,
        );

        assert_eq!(urls.len(), 3);
        assert!(urls[0].ends_with(".zip"));
        assert!(urls.iter().any(|url| url.contains("bigota.d.miui.com")));
    }
}
