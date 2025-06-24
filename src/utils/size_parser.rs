use anyhow::{anyhow, Result};

/// Parse human-readable size strings like "10MB" or "1.5GB" into bytes.
pub fn parse_size(input: &str) -> Result<u64> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(anyhow!("Size string is empty"));
    }

    let mut number_part = String::new();
    let mut unit_part = String::new();
    for c in trimmed.chars() {
        if c.is_ascii_digit() || c == '.' {
            number_part.push(c);
        } else {
            unit_part.push(c);
        }
    }

    let value: f64 = number_part
        .parse()
        .map_err(|_| anyhow!("Invalid size number: {}", number_part))?;

    let unit = unit_part.trim().to_lowercase();
    let multiplier = match unit.as_str() {
        "" | "b" => 1f64,
        "k" | "kb" => 1024f64,
        "m" | "mb" => 1024f64 * 1024f64,
        "g" | "gb" => 1024f64 * 1024f64 * 1024f64,
        "t" | "tb" => 1024f64 * 1024f64 * 1024f64 * 1024f64,
        _ => return Err(anyhow!("Invalid size unit: {}", unit_part)),
    };

    Ok((value * multiplier) as u64)
}
