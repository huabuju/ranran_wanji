use super::*;

pub(super) fn random_index(max: usize) -> Result<usize, String> {
    if max == 0 {
        return Err("随机字符集不能为空".to_string());
    }

    let mut bytes = [0u8; 4];
    fill_random_bytes(&mut bytes).map_err(|e| format!("生成 APatch SuperKey 失败: {}", e))?;
    Ok((u32::from_le_bytes(bytes) as usize) % max)
}

pub(super) fn sample_random_char(charset: &[u8]) -> Result<char, String> {
    let index = random_index(charset.len())?;
    Ok(charset[index] as char)
}

pub(super) fn shuffle_chars(chars: &mut [char]) -> Result<(), String> {
    if chars.len() <= 1 {
        return Ok(());
    }

    for index in (1..chars.len()).rev() {
        let swap_index = random_index(index + 1)?;
        chars.swap(index, swap_index);
    }

    Ok(())
}

pub(super) fn generate_apatch_super_key_value(length: usize) -> Result<String, String> {
    let safe_length = length.clamp(APATCH_SUPER_KEY_MIN_LENGTH, APATCH_SUPER_KEY_MAX_LENGTH);
    let mut characters = vec![
        sample_random_char(APATCH_SUPER_KEY_UPPERCASE)?,
        sample_random_char(APATCH_SUPER_KEY_LOWERCASE)?,
        sample_random_char(APATCH_SUPER_KEY_DIGITS)?,
    ];

    while characters.len() < safe_length {
        characters.push(sample_random_char(APATCH_SUPER_KEY_CHARSET)?);
    }

    shuffle_chars(&mut characters)?;
    let generated: String = characters.into_iter().collect();
    validate_apatch_super_key(&generated)
}

pub(super) fn validate_apatch_super_key(value: &str) -> Result<String, String> {
    let normalized = value.trim();
    if normalized.is_empty() {
        return Err("APatch SuperKey 不能为空".to_string());
    }

    let length = normalized.chars().count();
    if !(APATCH_SUPER_KEY_MIN_LENGTH..=APATCH_SUPER_KEY_MAX_LENGTH).contains(&length) {
        return Err("APatch SuperKey 长度需为 8-63 位".to_string());
    }

    if !normalized.chars().all(|ch| ch.is_ascii_alphanumeric()) {
        return Err("APatch SuperKey 只能包含字母和数字".to_string());
    }

    Ok(normalized.to_string())
}

pub(super) fn generate_apatch_super_key_impl() -> Result<String, String> {
    generate_apatch_super_key_value(APATCH_SUPER_KEY_LENGTH)
}
