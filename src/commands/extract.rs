use crate::infra::{trash_store::TrashStoreInterface, TrashStore, create_selector};
use crate::domain::{Config, TrashItem};
use anyhow::Result;
use std::path::PathBuf;

#[cfg(feature = "colors")]
use colored::Colorize;

/// Execute the extract command
pub fn execute(
    from: Option<String>,
    file: Option<String>,
    interactive: bool,
    all: bool,
    to: Option<PathBuf>,
    tree: bool,
    verbose: bool,
) -> Result<()> {
    let config = Config::load()?;
    let trash_store = TrashStore::new(config.trash_path.clone());

    if let Some(from_id) = from {
        // Extract from specific directory ID
        extract_from_directory(&trash_store, &from_id, file, to, tree, verbose)
    } else if interactive {
        // Interactive extraction
        extract_interactive(&trash_store, file, to, verbose)
    } else if all {
        // Extract all files matching pattern
        extract_all(&trash_store, file, to, verbose)
    } else if let Some(filename) = file {
        // Extract specific file by name
        extract_by_filename(&trash_store, &filename, to, verbose)
    } else {
        anyhow::bail!("Must specify one of: --from, --file, --interactive, or --all");
    }
}

fn extract_from_directory(
    trash_store: &TrashStore,
    from_id: &str,
    file: Option<String>,
    to: Option<PathBuf>,
    tree: bool,
    verbose: bool,
) -> Result<()> {
    // Find the directory item
    let items = trash_store.list()?;
    let dir_item = items
        .iter()
        .find(|item| item.meta.id.to_string().starts_with(from_id))
        .ok_or_else(|| anyhow::anyhow!("Directory with ID {} not found", from_id))?;

    if verbose {
        #[cfg(feature = "colors")]
        println!("📁 Extracting from: {}", dir_item.meta.original_path.display().to_string().cyan());
        #[cfg(not(feature = "colors"))]
        println!("📁 Extracting from: {}", dir_item.meta.original_path.display());
    }

    // For now, find files that have similar path prefix (simple approach)
    let dir_path_str = dir_item.meta.original_path.to_string_lossy().to_string();
    let child_files: Vec<&TrashItem> = items
        .iter()
        .filter(|item| {
            let item_path_str = item.meta.original_path.to_string_lossy();
            item_path_str.starts_with(&dir_path_str) && item.meta.id != dir_item.meta.id
        })
        .collect();

    if child_files.is_empty() {
        if tree {
            // Just extract the directory itself
            extract_single_file(trash_store, dir_item, to, verbose)?;
            return Ok(());
        } else {
            println!("No individual files found for this directory. Use 'rmz restore --id {}' to restore the entire directory.", from_id);
            return Ok(());
        }
    }

    if tree {
        display_tree_structure(&child_files, verbose)?;
        return Ok(());
    }

    if let Some(filename) = file {
        // Extract specific file from directory
        let target_file = child_files
            .iter()
            .find(|item| {
                item.meta.original_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| n.contains(&filename))
                    .unwrap_or(false)
            })
            .ok_or_else(|| anyhow::anyhow!("File '{}' not found in directory", filename))?;

        extract_single_file(trash_store, target_file, to, verbose)?;
    } else {
        // List all files in directory
        display_directory_contents(&child_files, verbose)?;
    }

    Ok(())
}

fn extract_interactive(
    trash_store: &TrashStore,
    filter: Option<String>,
    to: Option<PathBuf>,
    verbose: bool,
) -> Result<()> {
    let mut items = trash_store.list()?;

    // Apply filter if provided
    if let Some(filter_pattern) = &filter {
        let pattern_lower = filter_pattern.to_lowercase();
        items = items
            .into_iter()
            .filter(|item| {
                item.meta.original_path
                    .to_string_lossy()
                    .to_lowercase()
                    .contains(&pattern_lower)
            })
            .collect();
    }

    if items.is_empty() {
        #[cfg(feature = "colors")]
        println!("📂 {}", "No files available for extraction".yellow());
        #[cfg(not(feature = "colors"))]
        println!("📂 No files available for extraction");
        return Ok(());
    }

    // Sort by deletion time (newest first)
    items.sort_by(|a, b| b.meta.deleted_at.cmp(&a.meta.deleted_at));

    let selector = create_selector();

    if verbose {
        #[cfg(feature = "colors")]
        println!("🔍 {}", "Launching interactive file selector for extraction...".cyan());
        #[cfg(not(feature = "colors"))]
        println!("🔍 Launching interactive file selector for extraction...");
    }

    match selector.select_trash_item(&items)? {
        Some(selected_item) => {
            extract_single_file(trash_store, &selected_item, to, verbose)?;
        }
        None => {
            #[cfg(feature = "colors")]
            println!("⚠️  {}", "No file selected for extraction".yellow());
            #[cfg(not(feature = "colors"))]
            println!("⚠️ No file selected for extraction");
        }
    }

    Ok(())
}

fn extract_all(
    trash_store: &TrashStore,
    filter: Option<String>,
    to: Option<PathBuf>,
    verbose: bool,
) -> Result<()> {
    let mut items = trash_store.list()?;

    // Apply filter if provided
    if let Some(filter_pattern) = &filter {
        let pattern_lower = filter_pattern.to_lowercase();
        items = items
            .into_iter()
            .filter(|item| {
                item.meta.original_path
                    .to_string_lossy()
                    .to_lowercase()
                    .contains(&pattern_lower)
            })
            .collect();
    }

    if items.is_empty() {
        println!("No files found matching the criteria");
        return Ok(());
    }

    #[cfg(feature = "colors")]
    println!("📦 Extracting {} files...", items.len().to_string().green().bold());
    #[cfg(not(feature = "colors"))]
    println!("📦 Extracting {} files...", items.len());

    for item in &items {
        extract_single_file(trash_store, item, to.clone(), verbose)?;
    }

    Ok(())
}

fn extract_by_filename(
    trash_store: &TrashStore,
    filename: &str,
    to: Option<PathBuf>,
    verbose: bool,
) -> Result<()> {
    let items = trash_store.list()?;
    let matching_items: Vec<&TrashItem> = items
        .iter()
        .filter(|item| {
            item.meta.original_path
                .file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.contains(filename))
                .unwrap_or(false)
        })
        .collect();

    match matching_items.len() {
        0 => {
            anyhow::bail!("No files found matching '{}'", filename);
        }
        1 => {
            extract_single_file(trash_store, matching_items[0], to, verbose)?;
        }
        _ => {
            println!("Multiple files found matching '{}'. Please specify:", filename);
            for (i, item) in matching_items.iter().enumerate() {
                println!("  {}: {}", i + 1, item.meta.original_path.display());
            }
            anyhow::bail!("Use --interactive to select from multiple matches");
        }
    }

    Ok(())
}

fn extract_single_file(
    _trash_store: &TrashStore,
    item: &TrashItem,
    to: Option<PathBuf>,
    verbose: bool,
) -> Result<()> {
    let target_path = if let Some(to_path) = to {
        if to_path.is_dir() {
            // Extract to directory with original filename
            if let Some(filename) = item.meta.original_path.file_name() {
                to_path.join(filename)
            } else {
                anyhow::bail!("Cannot determine filename for extraction");
            }
        } else {
            to_path
        }
    } else {
        // Extract to current directory with original filename
        if let Some(filename) = item.meta.original_path.file_name() {
            std::env::current_dir()?.join(filename)
        } else {
            anyhow::bail!("Cannot determine filename for extraction");
        }
    };

    // Check if target already exists
    if target_path.exists() {
        anyhow::bail!("Target file already exists: {}", target_path.display());
    }

    // Create parent directory if needed
    if let Some(parent) = target_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Copy file from trash to target location
    std::fs::copy(&item.trash_path, &target_path)?;

    if verbose {
        #[cfg(feature = "colors")]
        println!("✅ {}: {} -> {}",
            "Extracted".green().bold(),
            item.meta.original_path.display().to_string().cyan(),
            target_path.display().to_string().green()
        );
        #[cfg(not(feature = "colors"))]
        println!("✅ Extracted: {} -> {}", 
            item.meta.original_path.display(), 
            target_path.display()
        );
    } else {
        #[cfg(feature = "colors")]
        println!("Extracted: {}", target_path.display().to_string().green());
        #[cfg(not(feature = "colors"))]
        println!("Extracted: {}", target_path.display());
    }

    Ok(())
}

fn display_tree_structure(files: &[&TrashItem], verbose: bool) -> Result<()> {
    #[cfg(feature = "colors")]
    println!("🌳 {}", "Directory Structure:".blue().bold());
    #[cfg(not(feature = "colors"))]
    println!("🌳 Directory Structure:");

    for (i, item) in files.iter().enumerate() {
        let is_last = i == files.len() - 1;
        let prefix = if is_last { "└── " } else { "├── " };
        
        let size_str = format_size(item.meta.size);
        let relative_time = format_relative_time(item.meta.deleted_at);
        
        if verbose {
            #[cfg(feature = "colors")]
            println!("{}📄 {} {} {} {}",
                prefix,
                item.meta.original_path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("?").white().bold(),
                format!("({})", size_str).yellow(),
                format!("deleted {}", relative_time).dimmed(),
                item.meta.id.to_string()[..8].bright_black()
            );
            #[cfg(not(feature = "colors"))]
            println!("{}📄 {} ({}) deleted {} {}",
                prefix,
                item.meta.original_path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("?"),
                size_str,
                relative_time,
                &item.meta.id.to_string()[..8]
            );
        } else {
            println!("{}📄 {}", prefix,
                item.meta.original_path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("?")
            );
        }
    }

    Ok(())
}

fn display_directory_contents(files: &[&TrashItem], verbose: bool) -> Result<()> {
    #[cfg(feature = "colors")]
    println!("📁 {} ({} files):", "Directory Contents".blue().bold(), files.len());
    #[cfg(not(feature = "colors"))]
    println!("📁 Directory Contents ({} files):", files.len());

    for item in files {
        let size_str = format_size(item.meta.size);
        
        if verbose {
            let relative_time = format_relative_time(item.meta.deleted_at);
            #[cfg(feature = "colors")]
            println!("  📄 {} {} {} {}",
                item.meta.original_path.display().to_string().white(),
                format!("({})", size_str).yellow(),
                format!("deleted {}", relative_time).dimmed(),
                item.meta.id.to_string()[..8].bright_black()
            );
            #[cfg(not(feature = "colors"))]
            println!("  📄 {} ({}) deleted {} {}",
                item.meta.original_path.display(),
                size_str,
                relative_time,
                &item.meta.id.to_string()[..8]
            );
        } else {
            #[cfg(feature = "colors")]
            println!("  📄 {} ({})", 
                item.meta.original_path.display().to_string().white(),
                size_str.yellow()
            );
            #[cfg(not(feature = "colors"))]
            println!("  📄 {} ({})", item.meta.original_path.display(), size_str);
        }
    }

    Ok(())
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

/// Format relative time (e.g., "2 hours ago", "3 days ago")
fn format_relative_time(datetime: chrono::DateTime<chrono::Utc>) -> String {
    let now = chrono::Utc::now();
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