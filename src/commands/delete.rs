use crate::domain::{Config, FileMeta};
use crate::infra::meta_store::MetaStoreInterface;
use crate::infra::trash_store::TrashStoreInterface;
use crate::infra::{ConfigManager, MetaStore, TrashStore};
use anyhow::Result;
use std::path::PathBuf;

/// Execute delete command
pub fn execute(
    paths: Vec<PathBuf>,
    force: bool,
    dry_run: bool,
    tag: Option<String>,
    interactive: bool,
    recursive: bool,
    verbose: bool,
) -> Result<()> {
    let config = ConfigManager::load()?;
    ConfigManager::initialize(&config)?;

    let trash_store = TrashStore::new(config.trash_path.clone());
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
            recursive,
            verbose,
        };
        match delete_path(&path, &config, &trash_store, &meta_store, &tag, &options) {
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

    Ok(())
}

struct DeleteOptions {
    force: bool,
    interactive: bool,
    recursive: bool,
    verbose: bool,
}

/// Delete a path (file or directory)
fn delete_path(
    path: &PathBuf,
    config: &Config,
    trash_store: &TrashStore,
    meta_store: &MetaStore,
    tag: &Option<String>,
    options: &DeleteOptions,
) -> Result<FileMeta> {
    // Check if file exists
    if !path.exists() {
        anyhow::bail!("Path does not exist: {}", path.display());
    }

    // Check if it's a directory
    if path.is_dir() {
        delete_directory(path, config, trash_store, meta_store, tag, options)
    } else {
        delete_single_file(path, config, trash_store, meta_store, tag, options)
    }
}

/// Delete a directory
fn delete_directory(
    path: &PathBuf,
    config: &Config,
    trash_store: &TrashStore,
    meta_store: &MetaStore,
    tag: &Option<String>,
    options: &DeleteOptions,
) -> Result<FileMeta> {
    // Check if directory is empty
    let is_empty = path.read_dir()?.next().is_none();
    
    if !is_empty && !options.recursive {
        anyhow::bail!(
            "Cannot remove directory '{}': Directory not empty (use -r to delete recursively)", 
            path.display()
        );
    }

    // Check if path is protected
    if config.is_protected(path) {
        anyhow::bail!("Directory is protected from deletion: {}", path.display());
    }

    // For non-empty directories, show warning and get confirmation
    if !is_empty && !options.force {
        let file_count = count_directory_contents(path)?;
        
        if options.interactive {
            println!("⚠️  Directory '{}' contains {} items", path.display(), file_count);
            if !confirm_directory_deletion(path, file_count)? {
                anyhow::bail!("Directory deletion cancelled by user");
            }
        } else {
            println!("⚠️  Recursively deleting directory '{}' with {} items", path.display(), file_count);
        }
    }

    // Create metadata for the directory
    let mut meta = FileMeta::from_path(path.as_path())?;
    
    // For directories, calculate total size recursively
    if path.is_dir() {
        meta.size = calculate_directory_size(path)?;
    }

    // Add tag if provided
    if let Some(tag_value) = tag {
        meta.add_tag(tag_value.clone());
    }

    if options.verbose {
        if is_empty {
            println!("Moving empty directory {} to trash...", path.display());
        } else {
            println!("Moving directory {} and its contents to trash...", path.display());
        }
    }

    // Move entire directory to trash
    let _trash_item = trash_store.save(&meta, path)?;
    meta_store.save_metadata(&meta)?;

    Ok(meta)
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

/// Count the total number of files and directories in a directory recursively
fn count_directory_contents(path: &std::path::Path) -> Result<usize> {
    let mut count = 0;
    
    fn count_recursive(path: &std::path::Path, count: &mut usize) -> Result<()> {
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            *count += 1;
            
            if entry.path().is_dir() {
                count_recursive(&entry.path(), count)?;
            }
        }
        Ok(())
    }
    
    count_recursive(path, &mut count)?;
    Ok(count)
}

/// Calculate the total size of a directory recursively
fn calculate_directory_size(path: &std::path::Path) -> Result<u64> {
    let mut total_size = 0;
    
    fn calculate_recursive(path: &std::path::Path, total_size: &mut u64) -> Result<()> {
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let metadata = entry.metadata()?;
            
            if metadata.is_file() {
                *total_size += metadata.len();
            } else if metadata.is_dir() {
                calculate_recursive(&entry.path(), total_size)?;
            }
        }
        Ok(())
    }
    
    calculate_recursive(path, &mut total_size)?;
    Ok(total_size)
}

fn confirm_deletion(path: &std::path::Path) -> Result<bool> {
    use dialoguer::Confirm;

    let confirmed = Confirm::new()
        .with_prompt(format!("Move '{}' to trash?", path.display()))
        .default(false)
        .interact()?;

    Ok(confirmed)
}

fn confirm_directory_deletion(path: &std::path::Path, file_count: usize) -> Result<bool> {
    use dialoguer::Confirm;

    let confirmed = Confirm::new()
        .with_prompt(format!(
            "Recursively move directory '{}' and its {} items to trash?", 
            path.display(), 
            file_count
        ))
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
            recursive: false,
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
            recursive: false,
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
            recursive: false,
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

    #[test]
    fn test_delete_empty_directory() {
        let temp_dir = TempDir::new().unwrap();
        let config = Config {
            trash_path: temp_dir.path().join("trash"),
            protected_paths: Vec::new(),
            ..Config::default()
        };

        let trash_store = TrashStore::new(config.trash_path.clone());
        let meta_store = MetaStore::new(config.metadata_path());

        // Create empty directory
        let empty_dir = temp_dir.path().join("empty_dir");
        fs::create_dir(&empty_dir).unwrap();

        let options = DeleteOptions {
            force: true,
            interactive: false,
            recursive: false,
            verbose: false,
        };
        let result = delete_directory(
            &empty_dir,
            &config,
            &trash_store,
            &meta_store,
            &None,
            &options,
        );

        assert!(result.is_ok());
        assert!(!empty_dir.exists());
    }

    #[test]
    fn test_delete_non_empty_directory_without_recursive() {
        let temp_dir = TempDir::new().unwrap();
        let config = Config {
            trash_path: temp_dir.path().join("trash"),
            protected_paths: Vec::new(),
            ..Config::default()
        };

        let trash_store = TrashStore::new(config.trash_path.clone());
        let meta_store = MetaStore::new(config.metadata_path());

        // Create directory with file
        let dir_with_file = temp_dir.path().join("dir_with_file");
        fs::create_dir(&dir_with_file).unwrap();
        fs::write(dir_with_file.join("file.txt"), "content").unwrap();

        let options = DeleteOptions {
            force: true,
            interactive: false,
            recursive: false,
            verbose: false,
        };
        let result = delete_directory(
            &dir_with_file,
            &config,
            &trash_store,
            &meta_store,
            &None,
            &options,
        );

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Directory not empty"));
        assert!(dir_with_file.exists());
    }

    #[test]
    fn test_delete_non_empty_directory_with_recursive() {
        let temp_dir = TempDir::new().unwrap();
        let config = Config {
            trash_path: temp_dir.path().join("trash"),
            protected_paths: Vec::new(),
            ..Config::default()
        };

        let trash_store = TrashStore::new(config.trash_path.clone());
        let meta_store = MetaStore::new(config.metadata_path());

        // Create directory with nested structure
        let dir_with_files = temp_dir.path().join("dir_with_files");
        fs::create_dir(&dir_with_files).unwrap();
        fs::write(dir_with_files.join("file1.txt"), "content1").unwrap();
        
        let subdir = dir_with_files.join("subdir");
        fs::create_dir(&subdir).unwrap();
        fs::write(subdir.join("file2.txt"), "content2").unwrap();

        let options = DeleteOptions {
            force: true,
            interactive: false,
            recursive: true,
            verbose: false,
        };
        let result = delete_directory(
            &dir_with_files,
            &config,
            &trash_store,
            &meta_store,
            &None,
            &options,
        );

        assert!(result.is_ok());
        assert!(!dir_with_files.exists());
    }

    #[test]
    fn test_count_directory_contents() {
        let temp_dir = TempDir::new().unwrap();
        let test_dir = temp_dir.path().join("test_dir");
        fs::create_dir(&test_dir).unwrap();

        // Create files and subdirectories
        fs::write(test_dir.join("file1.txt"), "content").unwrap();
        fs::write(test_dir.join("file2.txt"), "content").unwrap();
        
        let subdir = test_dir.join("subdir");
        fs::create_dir(&subdir).unwrap();
        fs::write(subdir.join("file3.txt"), "content").unwrap();

        let count = count_directory_contents(&test_dir).unwrap();
        assert_eq!(count, 4); // 2 files + 1 subdir + 1 file in subdir
    }

    #[test]
    fn test_calculate_directory_size() {
        let temp_dir = TempDir::new().unwrap();
        let test_dir = temp_dir.path().join("test_dir");
        fs::create_dir(&test_dir).unwrap();

        // Create files with known sizes
        fs::write(test_dir.join("file1.txt"), "12345").unwrap(); // 5 bytes
        fs::write(test_dir.join("file2.txt"), "1234567890").unwrap(); // 10 bytes
        
        let subdir = test_dir.join("subdir");
        fs::create_dir(&subdir).unwrap();
        fs::write(subdir.join("file3.txt"), "123").unwrap(); // 3 bytes

        let size = calculate_directory_size(&test_dir).unwrap();
        assert_eq!(size, 18); // 5 + 10 + 3 = 18 bytes
    }
}
