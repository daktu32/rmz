use crate::domain::TrashItem;
use crate::infra::{trash_store::TrashStoreInterface, ConfigManager, TrashStore};
use anyhow::Result;
use chrono::{Duration, Utc};
use std::collections::HashMap;

/// Execute status command
pub fn execute(detailed: bool, verbose: bool) -> Result<()> {
    let config = ConfigManager::load()?;
    let trash_store = TrashStore::new(config.trash_path.clone());

    // Get all items from trash
    let items = trash_store.list()?;

    if items.is_empty() {
        println!("🗑️  Trash Status: Empty");
        println!("No files in trash zone");
        return Ok(());
    }

    // Calculate basic statistics
    let total_files = items.len();
    let total_size: u64 = items.iter().map(|item| item.meta.size).sum();
    let total_size_human = format_size(total_size);

    // Find oldest and newest items
    let oldest = items
        .iter()
        .min_by_key(|item| item.meta.deleted_at)
        .map(|item| &item.meta.deleted_at);
    let newest = items
        .iter()
        .max_by_key(|item| item.meta.deleted_at)
        .map(|item| &item.meta.deleted_at);

    // Basic status output
    println!("🗑️  Trash Status");
    println!("{}", "─".repeat(50));
    println!("📁 Location: {}", config.trash_path.display());
    println!("📊 Files: {}", total_files);
    println!("💾 Total Size: {}", total_size_human);

    if let (Some(oldest), Some(newest)) = (oldest, newest) {
        println!("🕐 Oldest: {}", oldest.format("%Y-%m-%d %H:%M:%S"));
        println!("🕐 Newest: {}", newest.format("%Y-%m-%d %H:%M:%S"));
    }

    if detailed {
        show_detailed_status(&items, &config, verbose)?;
    }

    // Show configuration info if verbose
    if verbose {
        show_config_status(&config)?;
    }

    Ok(())
}

fn show_detailed_status(
    items: &[TrashItem],
    config: &crate::domain::Config,
    verbose: bool,
) -> Result<()> {
    println!("\n📈 Detailed Statistics");
    println!("{}", "─".repeat(50));

    // Group by time periods
    show_time_breakdown(items)?;

    // Group by file types
    show_file_type_breakdown(items)?;

    // Group by size ranges
    show_size_breakdown(items)?;

    // Group by tags
    show_tag_breakdown(items)?;

    // Show storage breakdown by date directories
    show_storage_breakdown(items, config)?;

    // Show recent activity
    if verbose {
        show_recent_activity(items)?;
    }

    Ok(())
}

fn show_time_breakdown(items: &[TrashItem]) -> Result<()> {
    let now = Utc::now();
    let mut today = 0;
    let mut yesterday = 0;
    let mut this_week = 0;
    let mut this_month = 0;
    let mut older = 0;

    for item in items {
        let age = now.signed_duration_since(item.meta.deleted_at);

        if age < Duration::days(1) {
            today += 1;
        } else if age < Duration::days(2) {
            yesterday += 1;
        } else if age < Duration::weeks(1) {
            this_week += 1;
        } else if age < Duration::days(30) {
            this_month += 1;
        } else {
            older += 1;
        }
    }

    println!("\n⏰ By Time Period:");
    if today > 0 {
        println!("   Today:      {}", today);
    }
    if yesterday > 0 {
        println!("   Yesterday:  {}", yesterday);
    }
    if this_week > 0 {
        println!("   This week:  {}", this_week);
    }
    if this_month > 0 {
        println!("   This month: {}", this_month);
    }
    if older > 0 {
        println!("   Older:      {}", older);
    }

    Ok(())
}

fn show_file_type_breakdown(items: &[TrashItem]) -> Result<()> {
    let mut type_counts: HashMap<String, (usize, u64)> = HashMap::new();

    for item in items {
        let extension = item
            .meta
            .original_path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("(no extension)")
            .to_lowercase();

        let entry = type_counts.entry(extension).or_insert((0, 0));
        entry.0 += 1;
        entry.1 += item.meta.size;
    }

    if !type_counts.is_empty() {
        println!("\n📄 By File Type:");
        let mut sorted_types: Vec<_> = type_counts.iter().collect();
        sorted_types.sort_by(|a, b| b.1 .0.cmp(&a.1 .0)); // Sort by count

        for (ext, (count, size)) in sorted_types.iter().take(10) {
            println!("   {}: {} files ({})", ext, count, format_size(*size));
        }

        if sorted_types.len() > 10 {
            let remaining = sorted_types.len() - 10;
            println!("   ... and {} more types", remaining);
        }
    }

    Ok(())
}

fn show_size_breakdown(items: &[TrashItem]) -> Result<()> {
    let mut small = 0; // < 1KB
    let mut medium = 0; // 1KB - 1MB
    let mut large = 0; // 1MB - 100MB
    let mut huge = 0; // > 100MB

    for item in items {
        match item.meta.size {
            s if s < 1024 => small += 1,
            s if s < 1024 * 1024 => medium += 1,
            s if s < 100 * 1024 * 1024 => large += 1,
            _ => huge += 1,
        }
    }

    println!("\n📏 By File Size:");
    if small > 0 {
        println!("   Small (< 1KB):     {}", small);
    }
    if medium > 0 {
        println!("   Medium (1KB-1MB):  {}", medium);
    }
    if large > 0 {
        println!("   Large (1MB-100MB): {}", large);
    }
    if huge > 0 {
        println!("   Huge (> 100MB):    {}", huge);
    }

    Ok(())
}

fn show_tag_breakdown(items: &[TrashItem]) -> Result<()> {
    let mut tag_counts: HashMap<String, usize> = HashMap::new();
    let mut untagged = 0;

    for item in items {
        if item.meta.tags.is_empty() {
            untagged += 1;
        } else {
            for tag in &item.meta.tags {
                *tag_counts.entry(tag.clone()).or_insert(0) += 1;
            }
        }
    }

    if !tag_counts.is_empty() || untagged > 0 {
        println!("\n🏷️  By Tags:");

        if untagged > 0 {
            println!("   (no tags): {}", untagged);
        }

        let mut sorted_tags: Vec<_> = tag_counts.iter().collect();
        sorted_tags.sort_by(|a, b| b.1.cmp(a.1));

        for (tag, count) in sorted_tags.iter().take(10) {
            println!("   {}: {}", tag, count);
        }

        if sorted_tags.len() > 10 {
            let remaining = sorted_tags.len() - 10;
            println!("   ... and {} more tags", remaining);
        }
    }

    Ok(())
}

fn show_storage_breakdown(items: &[TrashItem], config: &crate::domain::Config) -> Result<()> {
    let mut date_sizes: HashMap<String, (usize, u64)> = HashMap::new();

    for item in items {
        let date_key = item.meta.deleted_at.format("%Y-%m-%d").to_string();
        let entry = date_sizes.entry(date_key).or_insert((0, 0));
        entry.0 += 1;
        entry.1 += item.meta.size;
    }

    if !date_sizes.is_empty() {
        println!("\n📅 Storage by Date:");
        let mut sorted_dates: Vec<_> = date_sizes.iter().collect();
        sorted_dates.sort_by(|a, b| b.0.cmp(a.0)); // Sort by date (newest first)

        for (date, (count, size)) in sorted_dates.iter().take(7) {
            println!("   {}: {} files ({})", date, count, format_size(*size));
        }

        if sorted_dates.len() > 7 {
            let remaining = sorted_dates.len() - 7;
            println!("   ... and {} more dates", remaining);
        }
    }

    // Check if trash path exists and get disk usage info
    if config.trash_path.exists() {
        if let Ok(_metadata) = std::fs::metadata(&config.trash_path) {
            println!("\n💽 Storage Info:");
            println!("   Trash directory: {}", config.trash_path.display());
            // Note: Getting actual disk space is platform-specific and complex
            // For now, just show the calculated total size
            println!(
                "   Calculated size: {}",
                format_size(items.iter().map(|i| i.meta.size).sum())
            );
        }
    }

    Ok(())
}

fn show_recent_activity(items: &[TrashItem]) -> Result<()> {
    let now = Utc::now();
    let recent_items: Vec<_> = items
        .iter()
        .filter(|item| {
            let age = now.signed_duration_since(item.meta.deleted_at);
            age < Duration::days(7)
        })
        .collect();

    if !recent_items.is_empty() {
        println!("\n🕒 Recent Activity (last 7 days):");
        for item in recent_items.iter().take(5) {
            let filename = item.meta.filename().unwrap_or("(unknown)");
            let size = format_size(item.meta.size);
            let time_ago = format_time_ago(now.signed_duration_since(item.meta.deleted_at));

            println!("   {} ({}) - {}", filename, size, time_ago);
        }

        if recent_items.len() > 5 {
            println!("   ... and {} more recent files", recent_items.len() - 5);
        }
    }

    Ok(())
}

fn show_config_status(config: &crate::domain::Config) -> Result<()> {
    println!("\n⚙️  Configuration");
    println!("{}", "─".repeat(50));
    println!("📂 Trash Path: {}", config.trash_path.display());
    println!("📂 Metadata Path: {}", config.metadata_path().display());

    if config.protected_paths.is_empty() {
        println!("🛡️  Protected Paths: None");
    } else {
        println!(
            "🛡️  Protected Paths: {} configured",
            config.protected_paths.len()
        );
        for (i, path) in config.protected_paths.iter().enumerate() {
            if i < 3 {
                println!("   - {}", path.display());
            } else {
                println!("   ... and {} more", config.protected_paths.len() - 3);
                break;
            }
        }
    }

    // Check directory existence and permissions
    println!("\n🔍 Health Check:");
    if config.trash_path.exists() {
        println!("   ✅ Trash directory exists");
    } else {
        println!("   ⚠️  Trash directory does not exist (will be created)");
    }

    if config.metadata_path().exists() {
        println!("   ✅ Metadata directory exists");
    } else {
        println!("   ⚠️  Metadata directory does not exist (will be created)");
    }

    Ok(())
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

fn format_time_ago(duration: Duration) -> String {
    let days = duration.num_days();
    let hours = duration.num_hours();
    let minutes = duration.num_minutes();

    if days > 0 {
        format!("{} day{} ago", days, if days == 1 { "" } else { "s" })
    } else if hours > 0 {
        format!("{} hour{} ago", hours, if hours == 1 { "" } else { "s" })
    } else if minutes > 0 {
        format!(
            "{} minute{} ago",
            minutes,
            if minutes == 1 { "" } else { "s" }
        )
    } else {
        "just now".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{Config, FileMeta};
    use crate::infra::trash_store::TrashStoreInterface;
    use chrono::Utc;
    use std::fs;
    use tempfile::{NamedTempFile, TempDir};

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(100), "100 B");
        assert_eq!(format_size(1024), "1.0 KB");
        assert_eq!(format_size(1536), "1.5 KB");
        assert_eq!(format_size(1024 * 1024), "1.0 MB");
        assert_eq!(format_size(1024 * 1024 * 1024), "1.0 GB");
    }

    #[test]
    fn test_format_time_ago() {
        let _now = Utc::now();

        let just_now = Duration::seconds(30);
        assert_eq!(format_time_ago(just_now), "just now");

        let minutes_ago = Duration::minutes(5);
        assert_eq!(format_time_ago(minutes_ago), "5 minutes ago");

        let hour_ago = Duration::hours(1);
        assert_eq!(format_time_ago(hour_ago), "1 hour ago");

        let hours_ago = Duration::hours(3);
        assert_eq!(format_time_ago(hours_ago), "3 hours ago");

        let day_ago = Duration::days(1);
        assert_eq!(format_time_ago(day_ago), "1 day ago");

        let days_ago = Duration::days(5);
        assert_eq!(format_time_ago(days_ago), "5 days ago");
    }

    #[test]
    fn test_status_empty_trash() {
        let temp_dir = TempDir::new().unwrap();
        let config = Config {
            trash_path: temp_dir.path().join("trash"),
            protected_paths: Vec::new(),
            ..Config::default()
        };

        let trash_store = TrashStore::new(config.trash_path.clone());
        let items = trash_store.list().unwrap();
        assert!(items.is_empty());

        // Execute status should handle empty trash gracefully
        // Note: This test mainly verifies no panic occurs
        // since execute() prints to stdout
    }

    #[test]
    fn test_status_with_items() {
        let temp_dir = TempDir::new().unwrap();
        let config = Config {
            trash_path: temp_dir.path().join("trash"),
            protected_paths: Vec::new(),
            ..Config::default()
        };

        let trash_store = TrashStore::new(config.trash_path.clone());

        // Create test files with different characteristics
        let test_file1 = NamedTempFile::new().unwrap();
        let test_file2 = NamedTempFile::new().unwrap();
        fs::write(test_file1.path(), "small content").unwrap();
        fs::write(test_file2.path(), &vec![0u8; 2048]).unwrap(); // 2KB file

        let mut meta1 = FileMeta::from_path(&test_file1.path()).unwrap();
        let mut meta2 = FileMeta::from_path(&test_file2.path()).unwrap();

        // Add different tags
        meta1.add_tag("test".to_string());
        meta2.add_tag("large".to_string());

        trash_store.save(&meta1, test_file1.path()).unwrap();
        trash_store.save(&meta2, test_file2.path()).unwrap();

        let items = trash_store.list().unwrap();
        assert_eq!(items.len(), 2);

        // Test time breakdown (should not panic)
        let result = show_time_breakdown(&items);
        assert!(result.is_ok());

        // Test file type breakdown
        let result = show_file_type_breakdown(&items);
        assert!(result.is_ok());

        // Test size breakdown
        let result = show_size_breakdown(&items);
        assert!(result.is_ok());

        // Test tag breakdown
        let result = show_tag_breakdown(&items);
        assert!(result.is_ok());
    }

    #[test]
    fn test_show_config_status() {
        let temp_dir = TempDir::new().unwrap();
        let mut config = Config {
            trash_path: temp_dir.path().join("trash"),
            protected_paths: Vec::new(),
            ..Config::default()
        };

        // Add some protected paths
        config.add_protected_path(temp_dir.path().join("important"));
        config.add_protected_path(temp_dir.path().join("system"));

        // Test config status display (should not panic)
        let result = show_config_status(&config);
        assert!(result.is_ok());
    }
}
