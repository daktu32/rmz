use crate::domain::operation_log::OperationType;
use crate::domain::{Config, FileMeta};
use crate::infra::meta_store::MetaStoreInterface;
use crate::infra::operation_logger::JsonOperationLogger;
use crate::infra::trash_store::TrashStoreInterface;
use crate::infra::{ConfigManager, MetaStore, TrashStore};
use crate::utils::OperationRecorder;
use anyhow::Result;
use std::path::PathBuf;
use std::sync::Arc;

/// Execute delete command
pub fn execute(
    paths: Vec<PathBuf>,
    force: bool,
    dry_run: bool,
    tag: Option<String>,
    interactive: bool,
    verbose: bool,
) -> Result<()> {
    let config = ConfigManager::load()?;
    ConfigManager::initialize(&config)?;

    let trash_store = TrashStore::new(config.trash_path.clone());
    let orig_paths = paths.clone();
    let meta_store = MetaStore::new(config.metadata_path());

    if dry_run {
        println!("DRY RUN: Would delete the following files:");
        for path in &paths {
            println!("  {}", path.display());
        }
        return Ok(());
    }

    let mut deleted_files = Vec::new();

    for path in paths {
        let options = DeleteOptions {
            force,
            interactive,
            verbose,
        };
        match delete_single_file(&path, &config, &trash_store, &meta_store, &tag, &options) {
            Ok(meta) => {
                deleted_files.push(meta);
                if verbose {
                    println!("✅ Moved to trash: {}", path.display());
                }
            }
            Err(e) => {
                eprintln!("❌ Failed to delete {}: {}", path.display(), e);
            }
        }
    }

    if !deleted_files.is_empty() {
        println!(
            "Successfully moved {} file(s) to trash",
            deleted_files.len()
        );
    }
    let logger = Arc::new(JsonOperationLogger::new(config.logs_path()));
    let recorder = OperationRecorder::new(logger, OperationType::Delete, orig_paths);
    recorder.finish(Ok(()))?;

    Ok(())
}

struct DeleteOptions {
    force: bool,
    interactive: bool,
    verbose: bool,
}

fn delete_single_file(
    path: &PathBuf,
    config: &Config,
    trash_store: &TrashStore,
    meta_store: &MetaStore,
    tag: &Option<String>,
    options: &DeleteOptions,
) -> Result<FileMeta> {
    // Check if file exists
    if !path.exists() {
        anyhow::bail!("File does not exist: {}", path.display());
    }

    // Check if path is protected
    if config.is_protected(path) {
        anyhow::bail!("Path is protected from deletion: {}", path.display());
    }

    // Interactive confirmation if needed
    if options.interactive && !options.force && !confirm_deletion(path)? {
        anyhow::bail!("Deletion cancelled by user");
    }

    // Create metadata
    let mut meta = FileMeta::from_path(path.as_path())?;

    // Add tag if provided
    if let Some(tag_value) = tag {
        meta.add_tag(tag_value.clone());
    }

    if options.verbose {
        println!("Moving {} to trash...", path.display());
    }

    // Move to trash and save metadata
    let _trash_item = trash_store.save(&meta, path)?;
    meta_store.save_metadata(&meta)?;

    Ok(meta)
}

fn confirm_deletion(path: &std::path::Path) -> Result<bool> {
    use dialoguer::Confirm;

    let confirmed = Confirm::new()
        .with_prompt(format!("Move '{}' to trash?", path.display()))
        .default(false)
        .interact()?;

    Ok(confirmed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::{NamedTempFile, TempDir};

    #[test]
    fn test_delete_single_file_success() {
        let temp_dir = TempDir::new().unwrap();
        let config = Config {
            trash_path: temp_dir.path().join("trash"),
            protected_paths: Vec::new(),
            ..Config::default()
        };

        let trash_store = TrashStore::new(config.trash_path.clone());
        let meta_store = MetaStore::new(config.metadata_path());

        // Create test file
        let test_file = NamedTempFile::new().unwrap();
        let file_path = test_file.path().to_path_buf();
        fs::write(&file_path, "test content").unwrap();

        // Delete file
        let options = DeleteOptions {
            force: true,
            interactive: false,
            verbose: false,
        };
        let result = delete_single_file(
            &file_path,
            &config,
            &trash_store,
            &meta_store,
            &None,
            &options,
        );

        match &result {
            Ok(_) => {}
            Err(e) => panic!("Delete failed: {}", e),
        }
        assert!(!file_path.exists()); // Original file should be gone
    }

    #[test]
    fn test_delete_nonexistent_file() {
        let temp_dir = TempDir::new().unwrap();
        let config = Config {
            trash_path: temp_dir.path().join("trash"),
            protected_paths: Vec::new(),
            ..Config::default()
        };

        let trash_store = TrashStore::new(config.trash_path.clone());
        let meta_store = MetaStore::new(config.metadata_path());

        let nonexistent_path = PathBuf::from("/nonexistent/file.txt");

        let options = DeleteOptions {
            force: true,
            interactive: false,
            verbose: false,
        };
        let result = delete_single_file(
            &nonexistent_path,
            &config,
            &trash_store,
            &meta_store,
            &None,
            &options,
        );

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("File does not exist"));
    }

    #[test]
    fn test_delete_protected_file() {
        let temp_dir = TempDir::new().unwrap();
        let mut config = Config::default();
        config.trash_path = temp_dir.path().join("trash");

        // Add a protected path
        let protected_dir = temp_dir.path().join("protected");
        fs::create_dir_all(&protected_dir).unwrap();
        config.add_protected_path(protected_dir.clone());

        let trash_store = TrashStore::new(config.trash_path.clone());
        let meta_store = MetaStore::new(config.metadata_path());

        // Create test file in protected directory
        let protected_file = protected_dir.join("important.txt");
        fs::write(&protected_file, "important content").unwrap();

        let options = DeleteOptions {
            force: true,
            interactive: false,
            verbose: false,
        };
        let result = delete_single_file(
            &protected_file,
            &config,
            &trash_store,
            &meta_store,
            &None,
            &options,
        );

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("protected"));
        assert!(protected_file.exists()); // File should still exist
    }
}
