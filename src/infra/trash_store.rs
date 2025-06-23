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

        // Move file to trash
        std::fs::rename(source_path, &trash_path)?;

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
            // Remove the actual file
            std::fs::remove_file(&item.trash_path)?;

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
}
