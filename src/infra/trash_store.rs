use crate::domain::{FileMeta, TrashItem};
use crate::infra::{meta_store::MetaStoreInterface, MetaStore};
use anyhow::Result;
use std::path::{Path, PathBuf};
use uuid::Uuid;

/// Interface for trash storage operations
pub trait TrashStoreInterface {
    fn save(&self, meta: &FileMeta, source_path: &Path) -> Result<TrashItem>;
    fn restore(&self, id: &Uuid) -> Result<PathBuf>;
    fn list(&self) -> Result<Vec<TrashItem>>;
    fn purge(&self, id: &Uuid) -> Result<()>;
    fn find_by_id(&self, id: &Uuid) -> Result<Option<TrashItem>>;
}

/// File system based trash store implementation
pub struct TrashStore {
    trash_root: PathBuf,
    meta_store: MetaStore,
}

impl TrashStore {
    pub fn new(trash_root: PathBuf) -> Self {
        let meta_store = MetaStore::new(trash_root.join("metadata"));
        Self {
            trash_root,
            meta_store,
        }
    }

    pub fn get_trash_root(&self) -> &PathBuf {
        &self.trash_root
    }

    /// Get the subdirectory for a given date
    fn get_date_subdir(&self, meta: &FileMeta) -> PathBuf {
        let date_str = meta.deleted_at.format("%Y-%m-%d").to_string();
        self.trash_root.join(date_str)
    }

    /// Generate unique filename for trash
    fn generate_trash_filename(&self, meta: &FileMeta) -> String {
        format!("{}.rmz", meta.id)
    }
}

impl TrashStoreInterface for TrashStore {
    fn save(&self, meta: &FileMeta, source_path: &Path) -> Result<TrashItem> {
        // Ensure trash directory exists
        let date_dir = self.get_date_subdir(meta);
        std::fs::create_dir_all(&date_dir)?;

        // Generate trash file path
        let filename = self.generate_trash_filename(meta);
        let trash_path = date_dir.join(&filename);

        // Move file to trash - handle cross-device links
        if let Err(e) = std::fs::rename(source_path, &trash_path) {
            // Check if this is a cross-device link error (errno 18)
            if e.raw_os_error() == Some(18) {
                // Cross-device link - use copy + remove fallback
                if source_path.is_dir() {
                    copy_dir_recursive(source_path, &trash_path)?;
                } else {
                    std::fs::copy(source_path, &trash_path)?;
                }
                
                // Remove original after successful copy
                if source_path.is_dir() {
                    std::fs::remove_dir_all(source_path)?;
                } else {
                    std::fs::remove_file(source_path)?;
                }
            } else {
                // Other error - propagate it
                return Err(e.into());
            }
        }

        // Save metadata through MetaStore
        self.meta_store.save_metadata(meta)?;

        Ok(TrashItem::new(meta.clone(), trash_path))
    }

    fn restore(&self, id: &Uuid) -> Result<PathBuf> {
        if let Some(item) = self.find_by_id(id)? {
            // Restore to original location
            let original_path = &item.meta.original_path;

            // Ensure parent directory exists
            if let Some(parent) = original_path.parent() {
                std::fs::create_dir_all(parent)?;
            }

            // Move file back
            std::fs::rename(&item.trash_path, original_path)?;

            // Remove metadata after successful restore
            self.meta_store.delete_metadata(id)?;

            Ok(original_path.clone())
        } else {
            anyhow::bail!("File with ID {} not found in trash", id);
        }
    }

    fn list(&self) -> Result<Vec<TrashItem>> {
        let mut items = Vec::new();

        if !self.trash_root.exists() {
            return Ok(items);
        }

        // Get all metadata from MetaStore
        let all_metadata = self.meta_store.list_all_metadata()?;

        // Convert metadata to TrashItems, checking if files exist
        for meta in all_metadata {
            let date_dir = self.get_date_subdir(&meta);
            let filename = self.generate_trash_filename(&meta);
            let trash_path = date_dir.join(&filename);

            // Only include items where the actual file exists
            if trash_path.exists() {
                items.push(TrashItem::new(meta, trash_path));
            } else {
                // File is missing, optionally clean up metadata
                eprintln!(
                    "Warning: Metadata exists but file missing for ID: {}",
                    meta.id
                );
            }
        }

        Ok(items)
    }

    fn purge(&self, id: &Uuid) -> Result<()> {
        if let Some(item) = self.find_by_id(id)? {
            // Remove the actual file or directory
            if item.trash_path.is_dir() {
                std::fs::remove_dir_all(&item.trash_path)?;
            } else {
                std::fs::remove_file(&item.trash_path)?;
            }

            // Remove metadata through MetaStore
            self.meta_store.delete_metadata(id)?;

            Ok(())
        } else {
            anyhow::bail!("File with ID {} not found in trash", id);
        }
    }

    fn find_by_id(&self, id: &Uuid) -> Result<Option<TrashItem>> {
        // Load metadata for the specific ID
        if let Some(meta) = self.meta_store.load_metadata(id)? {
            let date_dir = self.get_date_subdir(&meta);
            let filename = self.generate_trash_filename(&meta);
            let trash_path = date_dir.join(&filename);

            // Check if the actual file exists
            if trash_path.exists() {
                Ok(Some(TrashItem::new(meta, trash_path)))
            } else {
                // Metadata exists but file is missing
                eprintln!("Warning: Metadata exists but file missing for ID: {}", id);
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }
}

/// Recursively copy a directory and all its contents to a new location
fn copy_dir_recursive(source: &Path, destination: &Path) -> Result<()> {
    // Create the destination directory
    std::fs::create_dir_all(destination)?;
    
    // Iterate through the source directory
    for entry in std::fs::read_dir(source)? {
        let entry = entry?;
        let source_path = entry.path();
        let dest_path = destination.join(entry.file_name());
        
        if source_path.is_dir() {
            // Recursively copy subdirectory
            copy_dir_recursive(&source_path, &dest_path)?;
        } else {
            // Copy file
            std::fs::copy(&source_path, &dest_path)?;
            
            // Preserve file permissions
            if let Ok(_metadata) = std::fs::metadata(&source_path) {
                if let Ok(permissions) = std::fs::metadata(&dest_path) {
                    let mut dest_permissions = permissions.permissions();
                    dest_permissions.set_readonly(false);
                    std::fs::set_permissions(&dest_path, dest_permissions).ok();
                }
            }
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::FileMeta;
    use std::fs;
    use tempfile::{NamedTempFile, TempDir};

    #[test]
    fn test_trash_store_save() {
        let temp_dir = TempDir::new().unwrap();
        let trash_store = TrashStore::new(temp_dir.path().to_path_buf());

        // Create a test file
        let test_file = NamedTempFile::new().unwrap();
        let file_path = test_file.path().to_path_buf();
        fs::write(&file_path, "test content").unwrap();

        // Create metadata
        let meta = FileMeta::from_path(&file_path).unwrap();

        // Save to trash
        let trash_item = trash_store.save(&meta, &file_path).unwrap();

        // Verify file was moved
        assert!(!file_path.exists());
        assert!(trash_item.trash_path.exists());
        assert_eq!(
            fs::read_to_string(&trash_item.trash_path).unwrap(),
            "test content"
        );
    }

    #[test]
    fn test_generate_trash_filename() {
        let temp_dir = TempDir::new().unwrap();
        let trash_store = TrashStore::new(temp_dir.path().to_path_buf());

        let test_file = NamedTempFile::new().unwrap();
        let meta = FileMeta::from_path(&test_file.path().to_path_buf()).unwrap();

        let filename = trash_store.generate_trash_filename(&meta);
        assert!(filename.ends_with(".rmz"));
        assert!(filename.contains(&meta.id.to_string()));
    }

    #[test]
    fn test_list_empty_trash() {
        let temp_dir = TempDir::new().unwrap();
        let trash_store = TrashStore::new(temp_dir.path().to_path_buf());

        let items = trash_store.list().unwrap();
        assert!(items.is_empty());
    }

    #[test]
    fn test_list_with_items() {
        let temp_dir = TempDir::new().unwrap();
        let trash_store = TrashStore::new(temp_dir.path().to_path_buf());

        // Create and save test files
        let test_file1 = NamedTempFile::new().unwrap();
        let test_file2 = NamedTempFile::new().unwrap();
        fs::write(test_file1.path(), "content1").unwrap();
        fs::write(test_file2.path(), "content2").unwrap();

        let meta1 = FileMeta::from_path(&test_file1.path().to_path_buf()).unwrap();
        let meta2 = FileMeta::from_path(&test_file2.path().to_path_buf()).unwrap();

        trash_store.save(&meta1, test_file1.path()).unwrap();
        trash_store.save(&meta2, test_file2.path()).unwrap();

        // List items
        let items = trash_store.list().unwrap();
        assert_eq!(items.len(), 2);
    }

    #[test]
    fn test_find_by_id() {
        let temp_dir = TempDir::new().unwrap();
        let trash_store = TrashStore::new(temp_dir.path().to_path_buf());

        // Create and save test file
        let test_file = NamedTempFile::new().unwrap();
        fs::write(test_file.path(), "content").unwrap();
        let meta = FileMeta::from_path(&test_file.path().to_path_buf()).unwrap();

        trash_store.save(&meta, test_file.path()).unwrap();

        // Find by ID
        let found_item = trash_store.find_by_id(&meta.id).unwrap();
        assert!(found_item.is_some());

        let item = found_item.unwrap();
        assert_eq!(item.meta.id, meta.id);
        assert!(item.trash_path.exists());
    }

    #[test]
    fn test_find_by_nonexistent_id() {
        let temp_dir = TempDir::new().unwrap();
        let trash_store = TrashStore::new(temp_dir.path().to_path_buf());

        let nonexistent_id = Uuid::new_v4();
        let result = trash_store.find_by_id(&nonexistent_id).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_copy_dir_recursive() {
        let temp_dir = TempDir::new().unwrap();
        
        // Create source directory structure
        let source_dir = temp_dir.path().join("source");
        std::fs::create_dir(&source_dir).unwrap();
        std::fs::write(source_dir.join("file1.txt"), "content1").unwrap();
        
        let subdir = source_dir.join("subdir");
        std::fs::create_dir(&subdir).unwrap();
        std::fs::write(subdir.join("file2.txt"), "content2").unwrap();
        
        // Copy to destination
        let dest_dir = temp_dir.path().join("dest");
        copy_dir_recursive(&source_dir, &dest_dir).unwrap();
        
        // Verify structure was copied
        assert!(dest_dir.exists());
        assert!(dest_dir.join("file1.txt").exists());
        assert!(dest_dir.join("subdir").exists());
        assert!(dest_dir.join("subdir/file2.txt").exists());
        
        // Verify content
        assert_eq!(
            std::fs::read_to_string(dest_dir.join("file1.txt")).unwrap(),
            "content1"
        );
        assert_eq!(
            std::fs::read_to_string(dest_dir.join("subdir/file2.txt")).unwrap(),
            "content2"
        );
    }
}
