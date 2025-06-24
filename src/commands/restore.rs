use crate::infra::{trash_store::TrashStoreInterface, ConfigManager, TrashStore};
use anyhow::Result;
use std::path::PathBuf;
use uuid::Uuid;

/// Execute restore command
pub struct RestoreOpts {
    pub to: Option<PathBuf>,
    pub force: bool,
    pub rename: bool,
}

pub fn execute(
    file: Option<String>,
    id: Option<String>,
    interactive: bool,
    all: bool,
    opts: RestoreOpts,
    verbose: bool,
) -> Result<()> {
    let config = ConfigManager::load()?;
    let trash_store = TrashStore::new(config.trash_path.clone());

    if let Some(id_str) = id {
        // Restore by specific ID
        restore_by_id(&trash_store, &id_str, &opts, verbose)
    } else if all {
        // Restore all files (with optional filter)
        restore_all(&trash_store, file, &opts, verbose)
    } else if interactive {
        // Interactive restore using fuzzy finder
        restore_interactive(&trash_store, file, opts.to.clone(), verbose)
    } else if let Some(pattern) = file {
        // Restore by file pattern
        restore_by_pattern(&trash_store, &pattern, &opts, verbose)
    } else {
        anyhow::bail!("Must specify one of: --id, --all, --interactive, or file pattern");
    }
}

fn restore_by_id(
    trash_store: &TrashStore,
    id_str: &str,
    opts: &RestoreOpts,
    verbose: bool,
) -> Result<()> {
    // Try to parse as full UUID first, then try partial UUID matching
    let id = if let Ok(full_id) = Uuid::parse_str(id_str) {
        full_id
    } else if id_str.len() >= 8 {
        // Try to find by partial ID (minimum 8 characters for safety)
        return restore_by_partial_id(trash_store, id_str, to, verbose);
    } else {
        return Err(anyhow::anyhow!("ID must be at least 8 characters long: {}", id_str));
    };

    if let Some(item) = trash_store.find_by_id(&id)? {
        let restore_path = restore_single_item(trash_store, &item, opts, verbose)?;
        if verbose {
            println!("✅ Restored: {} -> {}", id, restore_path.display());
        } else {
            println!("Restored: {}", restore_path.display());
        }
    } else {
        anyhow::bail!("File with ID {} not found in trash", id);
    }

    Ok(())
}

fn restore_by_partial_id(
    trash_store: &TrashStore,
    partial_id: &str,
    to: Option<PathBuf>,
    verbose: bool,
) -> Result<()> {
    let items = trash_store.list()?;
    let partial_id_lower = partial_id.to_lowercase();
    
    // Find all items that start with the partial ID
    let matches: Vec<_> = items
        .into_iter()
        .filter(|item| item.meta.id.to_string().to_lowercase().starts_with(&partial_id_lower))
        .collect();
    
    match matches.len() {
        0 => anyhow::bail!("No files matching partial ID '{}' found in trash", partial_id),
        1 => {
            // Exactly one match, restore it
            let item = &matches[0];
            let _restore_path = if let Some(to_path) = to {
                if to_path.is_dir() {
                    if let Some(filename) = item.meta.filename() {
                        to_path.join(filename)
                    } else {
                        return Err(anyhow::anyhow!("Cannot determine filename for restoration"));
                    }
                } else {
                    to_path
                }
            } else {
                item.meta.original_path.clone()
            };
            
            let actual_restore_path = trash_store.restore(&item.meta.id)?;
            
            if verbose {
                println!("Restored {} -> {}", item.meta.original_path.display(), actual_restore_path.display());
            } else {
                println!("Restored: {}", actual_restore_path.display());
            }
            
            Ok(())
        }
        _ => {
            // Multiple matches, show them and ask user to be more specific
            println!("Multiple files match partial ID '{}': ", partial_id);
            println!("──────────────────────────────────────────────────────────────────────");
            
            for item in &matches {
                let id_display = item.meta.id.to_string().chars().take(8).collect::<String>();
                let filename = item.meta.filename().unwrap_or("unknown");
                println!("📄 {} - {} - {}", 
                    filename,
                    item.meta.deleted_at.format("%Y-%m-%d %H:%M:%S"),
                    id_display
                );
            }
            
            anyhow::bail!("Please provide a more specific ID to uniquely identify the file")
        }
    }
}

fn restore_all(
    trash_store: &TrashStore,
    filter: Option<String>,
    opts: &RestoreOpts,
    verbose: bool,
) -> Result<()> {
    let items = trash_store.list()?;

    if items.is_empty() {
        println!("No files in trash to restore");
        return Ok(());
    }

    let filtered_items: Vec<_> = if let Some(pattern) = &filter {
        items
            .into_iter()
            .filter(|item| item.meta.matches_pattern(pattern))
            .collect()
    } else {
        items
    };

    if filtered_items.is_empty() {
        if filter.is_some() {
            println!("No files matching the pattern found in trash");
        }
        return Ok(());
    }

    // Confirm restoration of multiple files
    if !confirm_restore_all(&filtered_items)? {
        println!("Restoration cancelled");
        return Ok(());
    }

    let mut restored_count = 0;
    for item in filtered_items {
        match restore_single_item(trash_store, &item, opts, verbose) {
            Ok(path) => {
                restored_count += 1;
                if verbose {
                    println!("✅ Restored: {}", path.display());
                }
            }
            Err(e) => {
                eprintln!(
                    "❌ Failed to restore {}: {}",
                    item.meta.filename().unwrap_or("unknown"),
                    e
                );
            }
        }
    }

    println!("Restored {} file(s)", restored_count);
    Ok(())
}

fn restore_by_pattern(
    trash_store: &TrashStore,
    pattern: &str,
    opts: &RestoreOpts,
    verbose: bool,
) -> Result<()> {
    let items = trash_store.list()?;
    let matching_items: Vec<_> = items
        .into_iter()
        .filter(|item| item.meta.matches_pattern(pattern))
        .collect();

    if matching_items.is_empty() {
        println!("No files matching '{}' found in trash", pattern);
        return Ok(());
    }

    if matching_items.len() == 1 {
        // Single match, restore directly
        let item = &matching_items[0];
        let path = restore_single_item(trash_store, item, opts, verbose)?;
        println!("Restored: {}", path.display());
    } else {
        // Multiple matches, show list and ask for selection
        println!("Multiple files match '{}': ", pattern);
        for (i, item) in matching_items.iter().enumerate() {
            println!(
                "  {}: {} ({})",
                i + 1,
                item.meta.filename().unwrap_or("unknown"),
                item.meta.deleted_at.format("%Y-%m-%d %H:%M:%S")
            );
        }

        // For now, restore all matching files
        // TODO: Add interactive selection
        for item in matching_items {
            match restore_single_item(trash_store, &item, opts, verbose) {
                Ok(path) => {
                    if verbose {
                        println!("✅ Restored: {}", path.display());
                    }
                }
                Err(e) => {
                    eprintln!(
                        "❌ Failed to restore {}: {}",
                        item.meta.filename().unwrap_or("unknown"),
                        e
                    );
                }
            }
        }
    }

    Ok(())
}

fn restore_interactive(
    _trash_store: &TrashStore,
    _filter: Option<String>,
    _to: Option<PathBuf>,
    _verbose: bool,
) -> Result<()> {
    // TODO: Implement fuzzy finder integration
    anyhow::bail!("Interactive restore not yet implemented. Use 'rmz list' to see available files and restore by ID.");
}

fn restore_single_item(
    trash_store: &TrashStore,
    item: &crate::domain::TrashItem,
    opts: &RestoreOpts,
    _verbose: bool,
) -> Result<PathBuf> {
    use crate::utils::file_safety::{
        check_existing_file, generate_safe_restore_path, RestoreAction,
    };

    let mut final_path = if let Some(to_path) = opts.to.as_ref() {
        if to_path.is_dir() {
            if let Some(filename) = item.meta.filename() {
                to_path.join(filename)
            } else {
                anyhow::bail!("Cannot determine filename for restoration");
            }
        } else {
            to_path.clone()
        }
    } else {
        item.meta.original_path.clone()
    };

    if final_path.exists() {
        if opts.force {
            // overwrite
        } else if opts.rename {
            final_path = generate_safe_restore_path(&final_path);
        } else {
            match check_existing_file(&final_path)? {
                RestoreAction::Overwrite => {}
                RestoreAction::Skip => {
                    anyhow::bail!("Restoration skipped");
                }
                RestoreAction::Rename => {
                    final_path = generate_safe_restore_path(&final_path);
                }
                RestoreAction::Cancel => {
                    anyhow::bail!("Restoration cancelled");
                }
                RestoreAction::Proceed => {}
            }
        }
    }

    if opts.to.is_none() && final_path == item.meta.original_path {
        // Use TrashStore to handle metadata cleanup
        trash_store.restore(&item.meta.id)?;
        Ok(final_path)
    } else {
        if let Some(parent) = final_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::rename(&item.trash_path, &final_path)?;
        Ok(final_path)
    }
}

fn confirm_restore_all(items: &[crate::domain::TrashItem]) -> Result<bool> {
    use dialoguer::Confirm;

    let confirmed = Confirm::new()
        .with_prompt(format!("Restore {} file(s) from trash?", items.len()))
        .default(false)
        .interact()?;

    Ok(confirmed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{Config, FileMeta};
    use crate::infra::trash_store::TrashStoreInterface;
    use std::fs;
    use tempfile::{NamedTempFile, TempDir};

    #[test]
    fn test_restore_by_id() {
        let temp_dir = TempDir::new().unwrap();
        let config = Config {
            trash_path: temp_dir.path().join("trash"),
            protected_paths: Vec::new(),
            ..Config::default()
        };

        let trash_store = TrashStore::new(config.trash_path.clone());

        // Create and delete a test file
        let test_file = NamedTempFile::new().unwrap();
        let original_path = test_file.path().to_path_buf();
        fs::write(&original_path, "test content").unwrap();

        let meta = FileMeta::from_path(&original_path).unwrap();
        let id = meta.id;

        // Save to trash
        trash_store.save(&meta, &original_path).unwrap();
        assert!(!original_path.exists());

        // Restore by ID
        let opts = RestoreOpts {
            to: None,
            force: false,
            rename: false,
        };
        let result = restore_by_id(&trash_store, &id.to_string(), &opts, false);
        assert!(result.is_ok());
        assert!(original_path.exists());
        assert_eq!(fs::read_to_string(&original_path).unwrap(), "test content");
    }

    #[test]
    fn test_restore_by_invalid_id() {
        let temp_dir = TempDir::new().unwrap();
        let trash_store = TrashStore::new(temp_dir.path().join("trash"));

        let opts = RestoreOpts {
            to: None,
            force: false,
            rename: false,
        };
        let result = restore_by_id(&trash_store, "invalid-uuid", &opts, false);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("ID must be at least 8 characters long"));
    }

    #[test]
    fn test_restore_by_nonexistent_id() {
        let temp_dir = TempDir::new().unwrap();
        let trash_store = TrashStore::new(temp_dir.path().join("trash"));

        let nonexistent_id = uuid::Uuid::new_v4();
        let opts = RestoreOpts {
            to: None,
            force: false,
            rename: false,
        };
        let result = restore_by_id(&trash_store, &nonexistent_id.to_string(), &opts, false);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("not found in trash"));
    }

    #[test]
    fn test_restore_to_specific_location() {
        let temp_dir = TempDir::new().unwrap();
        let restore_dir = TempDir::new().unwrap();
        let trash_store = TrashStore::new(temp_dir.path().join("trash"));

        // Create and delete a test file
        let test_file = NamedTempFile::new().unwrap();
        let original_path = test_file.path().to_path_buf();
        fs::write(&original_path, "test content").unwrap();

        let meta = FileMeta::from_path(&original_path).unwrap();
        let id = meta.id;

        // Save to trash
        trash_store.save(&meta, &original_path).unwrap();

        // Restore to specific directory
        let restore_target = restore_dir.path().to_path_buf();
        let opts = RestoreOpts {
            to: Some(restore_target.clone()),
            force: false,
            rename: false,
        };
        let result = restore_by_id(&trash_store, &id.to_string(), &opts, false);
        assert!(result.is_ok());

        // Check if file was restored to the new location
        let expected_path = restore_target.join(meta.filename().unwrap());
        assert!(expected_path.exists());
        assert_eq!(fs::read_to_string(&expected_path).unwrap(), "test content");
    }
}
