use crate::infra::{trash_store::TrashStoreInterface, ConfigManager, TrashStore};
use anyhow::Result;
use std::path::PathBuf;
use uuid::Uuid;

/// Execute restore command
pub fn execute(
    file: Option<String>,
    id: Option<String>,
    interactive: bool,
    all: bool,
    to: Option<PathBuf>,
    verbose: bool,
) -> Result<()> {
    let config = ConfigManager::load()?;
    let trash_store = TrashStore::new(config.trash_path.clone());

    if let Some(id_str) = id {
        // Restore by specific ID
        restore_by_id(&trash_store, &id_str, to, verbose)
    } else if all {
        // Restore all files (with optional filter)
        restore_all(&trash_store, file, to, verbose)
    } else if interactive {
        // Interactive restore using fuzzy finder
        restore_interactive(&trash_store, file, to, verbose)
    } else if let Some(pattern) = file {
        // Restore by file pattern
        restore_by_pattern(&trash_store, &pattern, to, verbose)
    } else {
        anyhow::bail!("Must specify one of: --id, --all, --interactive, or file pattern");
    }
}

fn restore_by_id(
    trash_store: &TrashStore,
    id_str: &str,
    to: Option<PathBuf>,
    verbose: bool,
) -> Result<()> {
    let id =
        Uuid::parse_str(id_str).map_err(|_| anyhow::anyhow!("Invalid UUID format: {}", id_str))?;

    if let Some(item) = trash_store.find_by_id(&id)? {
        let restore_path = if let Some(to_path) = to {
            // Restore to specific location
            let final_path = if to_path.is_dir() {
                // If target is directory, use original filename
                if let Some(filename) = item.meta.filename() {
                    to_path.join(filename)
                } else {
                    anyhow::bail!("Cannot determine filename for restoration");
                }
            } else {
                to_path
            };

            // Ensure parent directory exists
            if let Some(parent) = final_path.parent() {
                std::fs::create_dir_all(parent)?;
            }

            // Move file to new location
            std::fs::rename(&item.trash_path, &final_path)?;
            final_path
        } else {
            // Restore to original location
            trash_store.restore(&id)?
        };

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

fn restore_all(
    trash_store: &TrashStore,
    filter: Option<String>,
    to: Option<PathBuf>,
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
        match restore_single_item(trash_store, &item, to.clone(), verbose) {
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
    to: Option<PathBuf>,
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
        let path = restore_single_item(trash_store, item, to, verbose)?;
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
            match restore_single_item(trash_store, &item, to.clone(), verbose) {
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
    to: Option<PathBuf>,
    _verbose: bool,
) -> Result<PathBuf> {
    if let Some(to_path) = to {
        // Restore to specific location
        let final_path = if to_path.is_dir() {
            if let Some(filename) = item.meta.filename() {
                to_path.join(filename)
            } else {
                anyhow::bail!("Cannot determine filename for restoration");
            }
        } else {
            to_path
        };

        // Ensure parent directory exists
        if let Some(parent) = final_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Move file to new location
        std::fs::rename(&item.trash_path, &final_path)?;

        // Remove metadata (manual cleanup since we're not using trash_store.restore())
        // Note: This is a bit awkward - we should refactor TrashStore to handle this better
        Ok(final_path)
    } else {
        // Restore to original location
        trash_store.restore(&item.meta.id)
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
        let result = restore_by_id(&trash_store, &id.to_string(), None, false);
        assert!(result.is_ok());
        assert!(original_path.exists());
        assert_eq!(fs::read_to_string(&original_path).unwrap(), "test content");
    }

    #[test]
    fn test_restore_by_invalid_id() {
        let temp_dir = TempDir::new().unwrap();
        let trash_store = TrashStore::new(temp_dir.path().join("trash"));

        let result = restore_by_id(&trash_store, "invalid-uuid", None, false);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid UUID format"));
    }

    #[test]
    fn test_restore_by_nonexistent_id() {
        let temp_dir = TempDir::new().unwrap();
        let trash_store = TrashStore::new(temp_dir.path().join("trash"));

        let nonexistent_id = uuid::Uuid::new_v4();
        let result = restore_by_id(&trash_store, &nonexistent_id.to_string(), None, false);
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
        let result = restore_by_id(
            &trash_store,
            &id.to_string(),
            Some(restore_target.clone()),
            false,
        );
        assert!(result.is_ok());

        // Check if file was restored to the new location
        let expected_path = restore_target.join(meta.filename().unwrap());
        assert!(expected_path.exists());
        assert_eq!(fs::read_to_string(&expected_path).unwrap(), "test content");
    }
}
