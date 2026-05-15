pub fn normalize_region_key(value: &str) -> String {
    let lower = value.trim().to_ascii_lowercase();
    if lower.is_empty() {
        return String::new();
    }

    if lower.contains("china mainland")
        || lower.contains("mainland china")
        || lower == "cn"
        || lower.contains("china")
    {
        return "china".to_string();
    }
    if lower.contains("europe") || lower.contains("eea") {
        return "europe".to_string();
    }
    if lower.contains("global") || lower == "mi" || lower.contains("international") {
        return "global".to_string();
    }
    if lower.contains("india") || lower == "in" {
        return "india".to_string();
    }
    if lower.contains("indonesia") || lower == "id" {
        return "indonesia".to_string();
    }
    if lower.contains("russia") || lower == "ru" {
        return "russia".to_string();
    }
    if lower.contains("taiwan") || lower == "tw" {
        return "taiwan".to_string();
    }
    if lower.contains("turkey") || lower == "tr" {
        return "turkey".to_string();
    }
    if lower.contains("japan") || lower == "jp" {
        return "japan".to_string();
    }
    if lower.contains("singapore") || lower == "sg" {
        return "singapore".to_string();
    }
    if lower.contains("latin america") || lower.contains("latinamerica") || lower.contains("latam")
    {
        return "latinamerica".to_string();
    }

    String::new()
}

pub fn fallback_region_key(value: &str) -> String {
    let normalized = value.trim().to_ascii_lowercase();
    if normalized.is_empty() {
        return String::new();
    }

    normalized
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric() || *ch == '-' || *ch == '_')
        .collect::<String>()
        .trim_matches(|ch| ch == '-' || ch == '_')
        .to_string()
}

pub fn get_region_label(region: &str) -> String {
    match region {
        "china" => "中国版".to_string(),
        "global" => "国际版".to_string(),
        "europe" => "欧洲版 (EEA)".to_string(),
        "india" => "印度版".to_string(),
        "indonesia" => "印尼版".to_string(),
        "russia" => "俄罗斯版".to_string(),
        "taiwan" => "中国台湾版".to_string(),
        "turkey" => "土耳其版".to_string(),
        "japan" => "日本版".to_string(),
        "singapore" => "新加坡版".to_string(),
        _ => region.trim().to_string(),
    }
}

pub fn infer_region_from_fragment(fragment: &str) -> String {
    let lower = fragment.trim().to_ascii_lowercase();
    if lower.is_empty() {
        return String::new();
    }

    for suffix in ["-fastboot", "-recovery", "-firmware", "-ota"] {
        if let Some(region_part) = lower.strip_suffix(suffix) {
            let normalized = normalize_region_key(region_part);
            if !normalized.is_empty() {
                return normalized;
            }
            return fallback_region_key(region_part);
        }
    }

    let normalized = normalize_region_key(&lower);
    if !normalized.is_empty() {
        return normalized;
    }

    fallback_region_key(&lower)
}

pub fn infer_region_from_text(value: &str) -> String {
    normalize_region_key(value)
}

pub fn infer_region_from_version(version: &str) -> String {
    let upper = version.trim().to_ascii_uppercase();
    if upper.len() < 4 {
        return String::new();
    }

    let suffix = &upper[upper.len() - 4..];
    match suffix {
        "CNXM" => "china".to_string(),
        "MIXM" => "global".to_string(),
        "EUXM" => "europe".to_string(),
        "INXM" => "india".to_string(),
        "IDXM" => "indonesia".to_string(),
        "RUXM" => "russia".to_string(),
        "TWXM" => "taiwan".to_string(),
        "TRXM" => "turkey".to_string(),
        "JPXM" => "japan".to_string(),
        "SGXM" => "singapore".to_string(),
        _ => String::new(),
    }
}

pub fn infer_region_from_filename(filename: &str) -> String {
    let lower = filename.trim().to_ascii_lowercase();
    if lower.is_empty() {
        return String::new();
    }

    for (marker, region) in [
        ("_cn_", "china"),
        ("_eea_", "europe"),
        ("_global_", "global"),
        ("_in_", "india"),
        ("_id_", "indonesia"),
        ("_ru_", "russia"),
        ("_tw_", "taiwan"),
        ("_tr_", "turkey"),
        ("_jp_", "japan"),
        ("_sg_", "singapore"),
        ("_latinamerica_", "latinamerica"),
        ("_latam_", "latinamerica"),
    ] {
        if lower.contains(marker) {
            return region.to_string();
        }
    }

    let normalized = normalize_region_key(&lower);
    if !normalized.is_empty() {
        return normalized;
    }

    String::new()
}

#[cfg(test)]
mod tests {
    use super::{
        get_region_label, infer_region_from_filename, infer_region_from_fragment,
        infer_region_from_text, infer_region_from_version,
    };

    #[test]
    fn should_map_known_region_to_chinese_label() {
        assert_eq!(infer_region_from_version("OS3.0.4.0.WMCMIXM"), "global");
        assert_eq!(get_region_label("global"), "国际版");
    }

    #[test]
    fn should_preserve_unknown_region_key_for_display() {
        assert_eq!(
            infer_region_from_fragment("latinamerica-fastboot"),
            "latinamerica"
        );
        assert_eq!(
            infer_region_from_text("Latin America Stable"),
            "latinamerica"
        );
        assert_eq!(
            infer_region_from_filename("foo_latinamerica_bar.zip"),
            "latinamerica"
        );
        assert_eq!(get_region_label("latinamerica"), "latinamerica");
    }
}
