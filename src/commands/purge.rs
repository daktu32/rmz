use crate::domain::Config;
use crate::infra::{trash_store::TrashStoreInterface, TrashStore};
use anyhow::{anyhow, Result};
use chrono::{DateTime, Duration, Utc};
use dialoguer::Confirm;
use std::collections::HashSet;
use std::path::PathBuf;
use uuid::Uuid;

#[cfg(feature = "colors")]
use colored::Colorize;

/// Execute the purge command
pub fn execute(
    all: bool,
    days: Option<u32>,
    size: Option<String>,
    id: Option<String>,
    interactive: bool,
    verbose: bool,
) -> Result<()> {
    let config = Config::load()?;
    let trash_root = config.trash_path.clone();
    let trash_store = TrashStore::new(trash_root);
    
    if all {
        purge_all(&trash_store, interactive, verbose)
    } else if let Some(days) = days {
        purge_by_age(&trash_store, days, interactive, verbose)
    } else if let Some(size_limit) = size {
        purge_by_size(&trash_store, &size_limit, interactive, verbose)
    } else if let Some(id_str) = id {
        purge_by_id(&trash_store, &id_str, interactive, verbose)
    } else {
        // Default behavior - interactive purge
        interactive_purge(&trash_store, verbose)
    }
}

/// Purge all files from trash
fn purge_all(
    trash_store: &TrashStore,
    interactive: bool,
    verbose: bool,
) -> Result<()> {
    let items = trash_store.list()?;
    
    if items.is_empty() {
        println!("Trash is already empty");
        return Ok(());
    }
    
    if interactive {
        let msg = format!("Permanently delete {} items from trash?", items.len());
        if !Confirm::new().with_prompt(msg).interact()? {
            println!("Purge cancelled");
            return Ok(());
        }
    }
    
    let mut purged_count = 0;
    
    for item in &items {
        match trash_store.purge(&item.meta.id) {
            Ok(()) => {
                purged_count += 1;
                if verbose {
                    #[cfg(feature = "colors")]
                    println!("🗑️  Purged: {}", item.meta.original_path.display().to_string().bright_red());
                    #[cfg(not(feature = "colors"))]
                    println!("Purged: {}", item.meta.original_path.display());
                }
            }
            Err(e) => {
                eprintln!("Failed to purge {}: {}", item.meta.original_path.display(), e);
            }
        }
    }
    
    #[cfg(feature = "colors")]
    println!("✅ Purged {} items", purged_count.to_string().green().bold());
    #[cfg(not(feature = "colors"))]
    println!("Purged {} items", purged_count);
    
    Ok(())
}

/// Purge files older than specified days
fn purge_by_age(
    trash_store: &TrashStore,
    days: u32,
    interactive: bool,
    verbose: bool,
) -> Result<()> {
    let cutoff_date = Utc::now() - Duration::days(days as i64);
    let items = trash_store.list()?;
    
    let old_items: Vec<_> = items
        .into_iter()
        .filter(|item| item.meta.deleted_at < cutoff_date)
        .collect();
    
    if old_items.is_empty() {
        println!("No files older than {} days found", days);
        return Ok(());
    }
    
    if interactive {
        let msg = format!(
            "Permanently delete {} items older than {} days?",
            old_items.len(),
            days
        );
        if !Confirm::new().with_prompt(msg).interact()? {
            println!("Purge cancelled");
            return Ok(());
        }
    }
    
    let mut purged_count = 0;
    
    for item in &old_items {
        match trash_store.purge(&item.meta.id) {
            Ok(()) => {
                purged_count += 1;
                if verbose {
                    #[cfg(feature = "colors")]
                    println!("🗑️  Purged: {} (deleted {})",
                        item.meta.original_path.display().to_string().bright_red(),
                        format_relative_time(item.meta.deleted_at).dimmed()
                    );
                    #[cfg(not(feature = "colors"))]
                    println!("Purged: {} (deleted {})",
                        item.meta.original_path.display(),
                        format_relative_time(item.meta.deleted_at)
                    );
                }
            }
            Err(e) => {
                eprintln!("Failed to purge {}: {}", item.meta.original_path.display(), e);
            }
        }
    }
    
    #[cfg(feature = "colors")]
    println!("✅ Purged {} items older than {} days", 
        purged_count.to_string().green().bold(), 
        days.to_string().yellow()
    );
    #[cfg(not(feature = "colors"))]
    println!("Purged {} items older than {} days", purged_count, days);
    
    Ok(())
}

/// Purge files when trash exceeds size limit
fn purge_by_size(
    trash_store: &TrashStore,
    size_limit: &str,
    interactive: bool,
    verbose: bool,
) -> Result<()> {
    let target_bytes = parse_size(size_limit)?;
    let items = trash_store.list()?;
    
    // Calculate current size
    let mut total_size = 0u64;
    let mut item_sizes = Vec::new();
    
    for item in items {
        let size = calculate_item_size(&item.trash_path)?;
        total_size += size;
        item_sizes.push((item, size));
    }
    
    if total_size <= target_bytes {
        #[cfg(feature = "colors")]
        println!("Trash size ({}) is within limit ({})", 
            format_size(total_size).green(),
            format_size(target_bytes).yellow()
        );
        #[cfg(not(feature = "colors"))]
        println!("Trash size ({}) is within limit ({})", 
            format_size(total_size),
            format_size(target_bytes)
        );
        return Ok(());
    }
    
    // Sort by oldest first
    item_sizes.sort_by_key(|(item, _)| item.meta.deleted_at);
    
    let excess_bytes = total_size - target_bytes;
    let mut to_purge = Vec::new();
    let mut purge_size = 0u64;
    
    for (item, size) in item_sizes {
        to_purge.push((item, size));
        purge_size += size;
        if purge_size >= excess_bytes {
            break;
        }
    }
    
    if interactive {
        let msg = format!(
            "Trash exceeds limit by {}. Purge {} oldest items ({}) to free space?",
            format_size(excess_bytes),
            to_purge.len(),
            format_size(purge_size)
        );
        if !Confirm::new().with_prompt(msg).interact()? {
            println!("Purge cancelled");
            return Ok(());
        }
    }
    
    let mut purged_count = 0;
    let mut freed_bytes = 0u64;
    
    for (item, size) in &to_purge {
        match trash_store.purge(&item.meta.id) {
            Ok(()) => {
                purged_count += 1;
                freed_bytes += size;
                if verbose {
                    #[cfg(feature = "colors")]
                    println!("🗑️  Purged: {} ({})",
                        item.meta.original_path.display().to_string().bright_red(),
                        format_size(*size).dimmed()
                    );
                    #[cfg(not(feature = "colors"))]
                    println!("Purged: {} ({})",
                        item.meta.original_path.display(),
                        format_size(*size)
                    );
                }
            }
            Err(e) => {
                eprintln!("Failed to purge {}: {}", item.meta.original_path.display(), e);
            }
        }
    }
    
    #[cfg(feature = "colors")]
    println!("✅ Purged {} items, freed {}", 
        purged_count.to_string().green().bold(),
        format_size(freed_bytes).yellow()
    );
    #[cfg(not(feature = "colors"))]
    println!("Purged {} items, freed {}", purged_count, format_size(freed_bytes));
    
    Ok(())
}

/// Purge specific file by ID
fn purge_by_id(
    trash_store: &TrashStore,
    id_str: &str,
    interactive: bool,
    verbose: bool,
) -> Result<()> {
    // Try to parse as full UUID first
    let id = match Uuid::parse_str(id_str) {
        Ok(uuid) => uuid,
        Err(_) => {
            // Try partial UUID matching
            let items = trash_store.list()?;
            let partial_id_lower = id_str.to_lowercase();
            
            let matches: Vec<_> = items
                .into_iter()
                .filter(|item| {
                    item.meta.id.to_string().to_lowercase().starts_with(&partial_id_lower)
                })
                .collect();
            
            match matches.len() {
                0 => return Err(anyhow!("No file found with ID starting with: {}", id_str)),
                1 => matches[0].meta.id,
                _ => {
                    eprintln!("Multiple files match ID prefix '{}':", id_str);
                    for item in matches {
                        println!("  {} - {}", 
                            &item.meta.id.to_string()[..8],
                            item.meta.original_path.display()
                        );
                    }
                    return Err(anyhow!("Please provide a more specific ID"));
                }
            }
        }
    };
    
    // Find the item
    let item = trash_store.find_by_id(&id)?
        .ok_or_else(|| anyhow!("File with ID {} not found", id))?;
    
    if interactive {
        let msg = format!(
            "Permanently delete '{}'?",
            item.meta.original_path.display()
        );
        if !Confirm::new().with_prompt(msg).interact()? {
            println!("Purge cancelled");
            return Ok(());
        }
    }
    
    trash_store.purge(&id)?;
    
    if verbose {
        #[cfg(feature = "colors")]
        println!("🗑️  Purged: {}", item.meta.original_path.display().to_string().bright_red());
        #[cfg(not(feature = "colors"))]
        println!("Purged: {}", item.meta.original_path.display());
    }
    
    #[cfg(feature = "colors")]
    println!("✅ File permanently deleted");
    #[cfg(not(feature = "colors"))]
    println!("File permanently deleted");
    
    Ok(())
}

/// Interactive purge mode
fn interactive_purge(trash_store: &TrashStore, verbose: bool) -> Result<()> {
    let items = trash_store.list()?;
    
    if items.is_empty() {
        println!("Trash is empty");
        return Ok(());
    }
    
    println!("Select files to purge permanently:");
    
    let mut selected_ids = HashSet::new();
    
    for (index, item) in items.iter().enumerate() {
        let prompt = format!(
            "{}: {} (deleted {})",
            index + 1,
            item.meta.original_path.display(),
            format_relative_time(item.meta.deleted_at)
        );
        
        if Confirm::new()
            .with_prompt(format!("Purge {}", prompt))
            .default(false)
            .interact()?
        {
            selected_ids.insert(item.meta.id);
        }
    }
    
    if selected_ids.is_empty() {
        println!("No files selected for purging");
        return Ok(());
    }
    
    let final_confirm = format!("Permanently delete {} selected files?", selected_ids.len());
    if !Confirm::new().with_prompt(final_confirm).interact()? {
        println!("Purge cancelled");
        return Ok(());
    }
    
    let mut purged_count = 0;
    
    for item in &items {
        if selected_ids.contains(&item.meta.id) {
            match trash_store.purge(&item.meta.id) {
                Ok(()) => {
                    purged_count += 1;
                    if verbose {
                        #[cfg(feature = "colors")]
                        println!("🗑️  Purged: {}", item.meta.original_path.display().to_string().bright_red());
                        #[cfg(not(feature = "colors"))]
                        println!("Purged: {}", item.meta.original_path.display());
                    }
                }
                Err(e) => {
                    eprintln!("Failed to purge {}: {}", item.meta.original_path.display(), e);
                }
            }
        }
    }
    
    #[cfg(feature = "colors")]
    println!("✅ Purged {} items", purged_count.to_string().green().bold());
    #[cfg(not(feature = "colors"))]
    println!("Purged {} items", purged_count);
    
    Ok(())
}

/// Parse size string (e.g., "100MB", "1GB") to bytes
fn parse_size(size_str: &str) -> Result<u64> {
    let size_str = size_str.to_uppercase();
    
    let (number_part, unit_part) = if size_str.ends_with("KB") {
        (size_str.trim_end_matches("KB"), 1024u64)
    } else if size_str.ends_with("MB") {
        (size_str.trim_end_matches("MB"), 1024u64.pow(2))
    } else if size_str.ends_with("GB") {
        (size_str.trim_end_matches("GB"), 1024u64.pow(3))
    } else if size_str.ends_with("TB") {
        (size_str.trim_end_matches("TB"), 1024u64.pow(4))
    } else if size_str.ends_with("B") {
        (size_str.trim_end_matches("B"), 1u64)
    } else {
        // Assume bytes if no unit
        (size_str.as_str(), 1u64)
    };
    
    let number: f64 = number_part.parse()
        .map_err(|_| anyhow!("Invalid size format: {}", size_str))?;
    
    Ok((number * unit_part as f64) as u64)
}

/// Format bytes to human readable size
fn format_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;
    
    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }
    
    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}

/// Calculate size of a file or directory
fn calculate_item_size(path: &PathBuf) -> Result<u64> {
    if path.is_file() {
        Ok(std::fs::metadata(path)?.len())
    } else if path.is_dir() {
        let mut total_size = 0u64;
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let entry_path = entry.path();
            total_size += calculate_item_size(&entry_path)?;
        }
        Ok(total_size)
    } else {
        Ok(0)
    }
}

/// Format relative time (e.g., "2 hours ago", "3 days ago")
fn format_relative_time(datetime: DateTime<Utc>) -> String {
    let now = Utc::now();
    let duration = now.signed_duration_since(datetime);
    
    if let Ok(std_duration) = duration.to_std() {
        let total_seconds = std_duration.as_secs();
        
        if total_seconds < 60 {
            "just now".to_string()
        } else if total_seconds < 3600 {
            let minutes = total_seconds / 60;
            format!("{} minute{} ago", minutes, if minutes == 1 { "" } else { "s" })
        } else if total_seconds < 86400 {
            let hours = total_seconds / 3600;
            format!("{} hour{} ago", hours, if hours == 1 { "" } else { "s" })
        } else {
            let days = total_seconds / 86400;
            format!("{} day{} ago", days, if days == 1 { "" } else { "s" })
        }
    } else {
        "unknown".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_size() {
        assert_eq!(parse_size("1024").unwrap(), 1024);
        assert_eq!(parse_size("1KB").unwrap(), 1024);
        assert_eq!(parse_size("1MB").unwrap(), 1024 * 1024);
        assert_eq!(parse_size("1GB").unwrap(), 1024 * 1024 * 1024);
        assert_eq!(parse_size("1.5MB").unwrap(), (1.5 * 1024.0 * 1024.0) as u64);
        assert_eq!(parse_size("500B").unwrap(), 500);
    }
    
    #[test]
    fn test_format_size() {
        assert_eq!(format_size(512), "512 B");
        assert_eq!(format_size(1024), "1.0 KB");
        assert_eq!(format_size(1536), "1.5 KB");
        assert_eq!(format_size(1024 * 1024), "1.0 MB");
        assert_eq!(format_size(1024 * 1024 * 1024), "1.0 GB");
    }
}