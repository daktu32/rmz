use anyhow::{anyhow, Result};
use chrono::Duration;

/// Parse duration strings like "30d", "7w", "1y" into chrono::Duration.
/// Supported units: d (days), w (weeks), m (months=30d), y (years=365d).
pub fn parse_duration(input: &str) -> Result<Duration> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(anyhow!("Duration string is empty"));
    }

    let mut number_part = String::new();
    let mut unit_part = String::new();
    for c in trimmed.chars() {
        if c.is_ascii_digit() {
            number_part.push(c);
        } else {
            unit_part.push(c);
        }
    }

    let value: i64 = number_part
        .parse()
        .map_err(|_| anyhow!("Invalid duration number: {}", number_part))?;

    let unit = unit_part.trim().to_lowercase();
    match unit.as_str() {
        "d" | "day" | "days" => Ok(Duration::days(value)),
        "w" | "week" | "weeks" => Ok(Duration::weeks(value)),
        "m" | "month" | "months" => Ok(Duration::days(value * 30)),
        "y" | "year" | "years" => Ok(Duration::days(value * 365)),
        _ => Err(anyhow!("Invalid duration unit: {}", unit_part)),
    }
}
