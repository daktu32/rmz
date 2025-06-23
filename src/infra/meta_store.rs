use crate::domain::FileMeta;
use anyhow::Result;
use std::path::PathBuf;
use uuid::Uuid;

/// Interface for metadata storage operations
pub trait MetaStoreInterface {
    fn save_metadata(&self, meta: &FileMeta) -> Result<()>;
    fn load_metadata(&self, id: &Uuid) -> Result<Option<FileMeta>>;
    fn list_all_metadata(&self) -> Result<Vec<FileMeta>>;
    fn delete_metadata(&self, id: &Uuid) -> Result<()>;
}

/// JSON file based metadata store
pub struct MetaStore {
    metadata_dir: PathBuf,
}

impl MetaStore {
    pub fn new(metadata_dir: PathBuf) -> Self {
        Self { metadata_dir }
    }

    fn metadata_file_path(&self, id: &Uuid) -> PathBuf {
        self.metadata_dir.join(format!("{}.json", id))
    }
}

impl MetaStoreInterface for MetaStore {
    fn save_metadata(&self, meta: &FileMeta) -> Result<()> {
        // Ensure metadata directory exists
        std::fs::create_dir_all(&self.metadata_dir)?;

        let file_path = self.metadata_file_path(&meta.id);
        let json = serde_json::to_string_pretty(meta)?;
        std::fs::write(file_path, json)?;

        Ok(())
    }

    fn load_metadata(&self, id: &Uuid) -> Result<Option<FileMeta>> {
        let file_path = self.metadata_file_path(id);

        if !file_path.exists() {
            return Ok(None);
        }

        let content = std::fs::read_to_string(file_path)?;
        let meta: FileMeta = serde_json::from_str(&content)?;
        Ok(Some(meta))
    }

    fn list_all_metadata(&self) -> Result<Vec<FileMeta>> {
        let mut metadata_list = Vec::new();

        if !self.metadata_dir.exists() {
            return Ok(metadata_list);
        }

        for entry in std::fs::read_dir(&self.metadata_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                let content = std::fs::read_to_string(&path)?;
                match serde_json::from_str::<FileMeta>(&content) {
                    Ok(meta) => metadata_list.push(meta),
                    Err(e) => {
                        eprintln!("Warning: Failed to parse metadata file {:?}: {}", path, e);
                    }
                }
            }
        }

        // Sort by deletion time (newest first)
        metadata_list.sort_by(|a, b| b.deleted_at.cmp(&a.deleted_at));

        Ok(metadata_list)
    }

    fn delete_metadata(&self, id: &Uuid) -> Result<()> {
        let file_path = self.metadata_file_path(id);

        if file_path.exists() {
            std::fs::remove_file(file_path)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::FileMeta;
    use std::fs;
    use tempfile::{NamedTempFile, TempDir};

    #[test]
    fn test_save_and_load_metadata() {
        let temp_dir = TempDir::new().unwrap();
        let meta_store = MetaStore::new(temp_dir.path().to_path_buf());

        // Create test metadata
        let test_file = NamedTempFile::new().unwrap();
        fs::write(test_file.path(), "content").unwrap();
        let meta = FileMeta::from_path(&test_file.path().to_path_buf()).unwrap();

        // Save metadata
        meta_store.save_metadata(&meta).unwrap();

        // Load metadata
        let loaded_meta = meta_store.load_metadata(&meta.id).unwrap();
        assert!(loaded_meta.is_some());

        let loaded = loaded_meta.unwrap();
        assert_eq!(loaded.id, meta.id);
        assert_eq!(loaded.original_path, meta.original_path);
        assert_eq!(loaded.size, meta.size);
    }

    #[test]
    fn test_list_all_metadata() {
        let temp_dir = TempDir::new().unwrap();
        let meta_store = MetaStore::new(temp_dir.path().to_path_buf());

        // Create multiple metadata files
        let test_file1 = NamedTempFile::new().unwrap();
        let test_file2 = NamedTempFile::new().unwrap();
        fs::write(test_file1.path(), "content1").unwrap();
        fs::write(test_file2.path(), "content2").unwrap();

        let meta1 = FileMeta::from_path(&test_file1.path().to_path_buf()).unwrap();
        let meta2 = FileMeta::from_path(&test_file2.path().to_path_buf()).unwrap();

        meta_store.save_metadata(&meta1).unwrap();
        meta_store.save_metadata(&meta2).unwrap();

        // List all metadata
        let all_meta = meta_store.list_all_metadata().unwrap();
        assert_eq!(all_meta.len(), 2);

        // Should be sorted by deletion time (newest first)
        assert!(all_meta[0].deleted_at >= all_meta[1].deleted_at);
    }

    #[test]
    fn test_delete_metadata() {
        let temp_dir = TempDir::new().unwrap();
        let meta_store = MetaStore::new(temp_dir.path().to_path_buf());

        let test_file = NamedTempFile::new().unwrap();
        fs::write(test_file.path(), "content").unwrap();
        let meta = FileMeta::from_path(&test_file.path().to_path_buf()).unwrap();

        // Save and then delete
        meta_store.save_metadata(&meta).unwrap();
        assert!(meta_store.load_metadata(&meta.id).unwrap().is_some());

        meta_store.delete_metadata(&meta.id).unwrap();
        assert!(meta_store.load_metadata(&meta.id).unwrap().is_none());
    }
}
