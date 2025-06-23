use crate::cli::GroupBy;
use crate::domain::TrashItem;
use crate::infra::{trash_store::TrashStoreInterface, ConfigManager, TrashStore};
use anyhow::Result;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// Execute list command
pub fn execute(
    json: bool,
    filter: Option<String>,
    since: Option<String>,
    group_by: Option<GroupBy>,
    limit: Option<usize>,
    verbose: bool,
) -> Result<()> {
    let config = ConfigManager::load()?;
    let trash_store = TrashStore::new(config.trash_path.clone());

    // Get all items from trash
    let mut items = trash_store.list()?;

    // Apply since filter
    if let Some(since_str) = &since {
        let since_date = parse_since_date(since_str)?;
        items.retain(|item| item.meta.deleted_at >= since_date);
    }

    // Apply pattern filter
    if let Some(pattern) = &filter {
        items.retain(|item| item.meta.matches_pattern(pattern));
    }

    // Apply limit
    if let Some(limit_count) = limit {
        items.truncate(limit_count);
    }

    if items.is_empty() {
        if filter.is_some() || since.is_some() {
            println!("No files found matching the criteria");
        } else {
            println!("Trash is empty");
        }
        return Ok(());
    }

    if json {
        // Output in JSON format
        output_json(&items, verbose)?;
    } else if let Some(group_type) = group_by {
        // Group and display
        output_grouped(&items, &group_type, verbose)?;
    } else {
        // Simple list format
        output_simple(&items, verbose)?;
    }

    Ok(())
}

fn parse_since_date(since_str: &str) -> Result<DateTime<Utc>> {
    match since_str.to_lowercase().as_str() {
        "today" => {
            let now = Utc::now();
            let today_start = now
                .date_naive()
                .and_hms_opt(0, 0, 0)
                .ok_or_else(|| anyhow::anyhow!("Failed to create today's start time"))?
                .and_utc();
            Ok(today_start)
        }
        "yesterday" => {
            let now = Utc::now();
            let yesterday = now.date_naive() - chrono::Duration::days(1);
            let yesterday_start = yesterday
                .and_hms_opt(0, 0, 0)
                .ok_or_else(|| anyhow::anyhow!("Failed to create yesterday's start time"))?
                .and_utc();
            Ok(yesterday_start)
        }
        "week" => {
            let now = Utc::now();
            let week_ago = now - chrono::Duration::weeks(1);
            Ok(week_ago)
        }
        "month" => {
            let now = Utc::now();
            let month_ago = now - chrono::Duration::days(30);
            Ok(month_ago)
        }
        _ => {
            // Try to parse as ISO date (YYYY-MM-DD or YYYY-MM-DD HH:MM:SS)
            if let Ok(date) = chrono::NaiveDate::parse_from_str(since_str, "%Y-%m-%d") {
                let datetime = date
                    .and_hms_opt(0, 0, 0)
                    .ok_or_else(|| anyhow::anyhow!("Failed to create datetime"))?
                    .and_utc();
                Ok(datetime)
            } else if let Ok(datetime) =
                chrono::DateTime::parse_from_str(since_str, "%Y-%m-%d %H:%M:%S %z")
            {
                Ok(datetime.with_timezone(&Utc))
            } else if let Ok(datetime) =
                chrono::NaiveDateTime::parse_from_str(since_str, "%Y-%m-%d %H:%M:%S")
            {
                Ok(datetime.and_utc())
            } else {
                anyhow::bail!("Invalid date format: {}. Use 'today', 'yesterday', 'week', 'month', or YYYY-MM-DD", since_str);
            }
        }
    }
}

fn output_json(items: &[TrashItem], verbose: bool) -> Result<()> {
    #[derive(serde::Serialize)]
    struct JsonItem<'a> {
        id: &'a uuid::Uuid,
        filename: Option<&'a str>,
        original_path: &'a std::path::PathBuf,
        deleted_at: &'a DateTime<Utc>,
        size: u64,
        human_size: String,
        tags: &'a Vec<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        permissions: Option<u32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        deleted_by: Option<&'a str>,
        #[serde(skip_serializing_if = "Option::is_none")]
        trash_path: Option<&'a std::path::PathBuf>,
    }

    let json_items: Vec<JsonItem> = items
        .iter()
        .map(|item| JsonItem {
            id: &item.meta.id,
            filename: item.meta.filename(),
            original_path: &item.meta.original_path,
            deleted_at: &item.meta.deleted_at,
            size: item.meta.size,
            human_size: item.meta.human_readable_size(),
            tags: &item.meta.tags,
            permissions: if verbose {
                Some(item.meta.permissions)
            } else {
                None
            },
            deleted_by: if verbose {
                Some(&item.meta.deleted_by)
            } else {
                None
            },
            trash_path: if verbose {
                Some(&item.trash_path)
            } else {
                None
            },
        })
        .collect();

    let json = serde_json::to_string_pretty(&json_items)?;
    println!("{}", json);
    Ok(())
}

fn output_grouped(items: &[TrashItem], group_by: &GroupBy, verbose: bool) -> Result<()> {
    match group_by {
        GroupBy::Date => {
            let mut groups: HashMap<String, Vec<&TrashItem>> = HashMap::new();

            for item in items {
                let date_key = item.meta.deleted_at.format("%Y-%m-%d").to_string();
                groups.entry(date_key).or_default().push(item);
            }

            let mut sorted_dates: Vec<_> = groups.keys().collect();
            sorted_dates.sort_by(|a, b| b.cmp(a)); // Newest first

            for date in sorted_dates {
                let items_for_date = &groups[date];
                println!("\n📅 {} ({} files)", date, items_for_date.len());
                println!("{}", "─".repeat(50));

                for item in items_for_date {
                    output_single_item(item, verbose, "  ");
                }
            }
        }
        GroupBy::Tag => {
            let mut groups: HashMap<String, Vec<&TrashItem>> = HashMap::new();

            for item in items {
                if item.meta.tags.is_empty() {
                    groups
                        .entry("(no tags)".to_string())
                        .or_default()
                        .push(item);
                } else {
                    for tag in &item.meta.tags {
                        groups.entry(tag.clone()).or_default().push(item);
                    }
                }
            }

            let mut sorted_tags: Vec<_> = groups.keys().collect();
            sorted_tags.sort();

            for tag in sorted_tags {
                let items_for_tag = &groups[tag];
                println!("\n🏷️  {} ({} files)", tag, items_for_tag.len());
                println!("{}", "─".repeat(50));

                for item in items_for_tag {
                    output_single_item(item, verbose, "  ");
                }
            }
        }
    }

    Ok(())
}

fn output_simple(items: &[TrashItem], verbose: bool) -> Result<()> {
    println!("Files in trash ({} total):", items.len());
    println!("{}", "─".repeat(70));

    for item in items {
        output_single_item(item, verbose, "");
    }

    // Summary statistics
    let total_size: u64 = items.iter().map(|item| item.meta.size).sum();
    let total_size_human = format_size(total_size);

    println!("{}", "─".repeat(70));
    println!("Total: {} files, {}", items.len(), total_size_human);

    Ok(())
}

fn output_single_item(item: &TrashItem, verbose: bool, prefix: &str) {
    let filename = item.meta.filename().unwrap_or("(unknown)");
    let size = item.meta.human_readable_size();
    let deleted_time = item.meta.deleted_at.format("%Y-%m-%d %H:%M:%S");

    if verbose {
        println!("{}📄 {} ({})", prefix, filename, size);
        println!("{}   ID: {}", prefix, item.meta.id);
        println!(
            "{}   Original: {}",
            prefix,
            item.meta.original_path.display()
        );
        println!(
            "{}   Deleted: {} by {}",
            prefix, deleted_time, item.meta.deleted_by
        );
        if !item.meta.tags.is_empty() {
            println!("{}   Tags: {}", prefix, item.meta.tags.join(", "));
        }
        println!("{}   Trash: {}", prefix, item.trash_path.display());
        println!();
    } else {
        let tags_display = if item.meta.tags.is_empty() {
            String::new()
        } else {
            format!(" [{}]", item.meta.tags.join(", "))
        };

        println!(
            "{}📄 {} ({}) - {} - {}{}",
            prefix,
            filename,
            size,
            deleted_time,
            item.meta.id.to_string().chars().take(8).collect::<String>(),
            tags_display
        );
    }
}

fn format_size(size: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size_f = size as f64;
    let mut unit_index = 0;

    while size_f >= 1024.0 && unit_index < UNITS.len() - 1 {
        size_f /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", size, UNITS[0])
    } else {
        format!("{:.1} {}", size_f, UNITS[unit_index])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{Config, FileMeta};
    use crate::infra::trash_store::TrashStoreInterface;
    use std::fs;
    use tempfile::{NamedTempFile, TempDir};

    #[test]
    fn test_parse_since_date() {
        // Test relative dates
        let today = parse_since_date("today").unwrap();
        let yesterday = parse_since_date("yesterday").unwrap();
        let week = parse_since_date("week").unwrap();
        let month = parse_since_date("month").unwrap();

        assert!(today > yesterday);
        assert!(yesterday > week);
        assert!(week > month);

        // Test absolute date
        let absolute = parse_since_date("2024-01-01").unwrap();
        assert_eq!(absolute.format("%Y-%m-%d").to_string(), "2024-01-01");

        // Test invalid date
        assert!(parse_since_date("invalid").is_err());
    }

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(100), "100 B");
        assert_eq!(format_size(1024), "1.0 KB");
        assert_eq!(format_size(1536), "1.5 KB");
        assert_eq!(format_size(1024 * 1024), "1.0 MB");
        assert_eq!(format_size(1024 * 1024 * 1024), "1.0 GB");
    }

    #[test]
    fn test_list_empty_trash() {
        let temp_dir = TempDir::new().unwrap();
        let config = Config {
            trash_path: temp_dir.path().join("trash"),
            protected_paths: Vec::new(),
            ..Config::default()
        };

        let trash_store = TrashStore::new(config.trash_path.clone());
        let items = trash_store.list().unwrap();
        assert!(items.is_empty());
    }

    #[test]
    fn test_list_with_items() {
        let temp_dir = TempDir::new().unwrap();
        let config = Config {
            trash_path: temp_dir.path().join("trash"),
            protected_paths: Vec::new(),
            ..Config::default()
        };

        let trash_store = TrashStore::new(config.trash_path.clone());

        // Create and save test files
        let test_file1 = NamedTempFile::new().unwrap();
        let test_file2 = NamedTempFile::new().unwrap();
        fs::write(test_file1.path(), "content1").unwrap();
        fs::write(test_file2.path(), "content2").unwrap();

        let mut meta1 = FileMeta::from_path(&test_file1.path()).unwrap();
        let mut meta2 = FileMeta::from_path(&test_file2.path()).unwrap();

        // Add tags for testing
        meta1.add_tag("test".to_string());
        meta2.add_tag("important".to_string());

        trash_store.save(&meta1, test_file1.path()).unwrap();
        trash_store.save(&meta2, test_file2.path()).unwrap();

        // List items
        let items = trash_store.list().unwrap();
        assert_eq!(items.len(), 2);

        // Test filtering
        let filtered: Vec<_> = items
            .iter()
            .filter(|item| item.meta.matches_pattern("test"))
            .collect();
        assert_eq!(filtered.len(), 1);
    }

    #[test]
    fn test_json_output() {
        let temp_dir = TempDir::new().unwrap();
        let trash_store = TrashStore::new(temp_dir.path().join("trash"));

        // Create test file
        let test_file = NamedTempFile::new().unwrap();
        fs::write(test_file.path(), "test content").unwrap();
        let meta = FileMeta::from_path(&test_file.path()).unwrap();

        trash_store.save(&meta, test_file.path()).unwrap();
        let items = trash_store.list().unwrap();

        // Test JSON output (should not panic)
        let result = output_json(&items, false);
        assert!(result.is_ok());

        let result_verbose = output_json(&items, true);
        assert!(result_verbose.is_ok());
    }
}
