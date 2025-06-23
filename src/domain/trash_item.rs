use crate::domain::FileMeta;
use std::path::PathBuf;

/// Represents a complete trash item (metadata + physical file location)
#[derive(Debug, Clone)]
pub struct TrashItem {
    /// File metadata
    pub meta: FileMeta,

    /// Path where the file is currently stored in trash zone
    pub trash_path: PathBuf,
}

impl TrashItem {
    /// Create a new trash item
    pub fn new(meta: FileMeta, trash_path: PathBuf) -> Self {
        Self { meta, trash_path }
    }

    /// Check if the physical file exists in trash
    pub fn exists(&self) -> bool {
        self.trash_path.exists()
    }

    /// Get the size of the actual file in trash (may differ from metadata if file was modified)
    pub fn actual_size(&self) -> anyhow::Result<u64> {
        let metadata = std::fs::metadata(&self.trash_path)?;
        Ok(metadata.len())
    }

    /// Verify file integrity using checksum (if available)
    pub fn verify_integrity(&self) -> anyhow::Result<bool> {
        if let Some(expected_checksum) = &self.meta.checksum {
            let actual_checksum = self.calculate_checksum()?;
            Ok(actual_checksum == *expected_checksum)
        } else {
            // If no checksum is stored, we assume integrity is fine
            Ok(true)
        }
    }

    /// Calculate checksum of the current file
    pub fn calculate_checksum(&self) -> anyhow::Result<String> {
        use std::fs::File;
        use std::io::{BufReader, Read};

        let file = File::open(&self.trash_path)?;
        let mut reader = BufReader::new(file);
        let mut hasher = sha2::Sha256::new();
        let mut buffer = [0u8; 8192];

        loop {
            let bytes_read = reader.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
        }

        use sha2::Digest;
        Ok(format!("{:x}", hasher.finalize()))
    }

    /// Get the trash subdirectory (based on deletion date)
    pub fn trash_subdirectory(&self) -> String {
        self.meta.deleted_at.format("%Y-%m-%d").to_string()
    }

    /// Get the metadata file path corresponding to this trash item
    pub fn metadata_path(&self) -> PathBuf {
        self.trash_path.with_extension("meta.json")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::FileMeta;
    use std::fs;
    use tempfile::{NamedTempFile, TempDir};

    fn create_test_meta() -> FileMeta {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_path_buf();
        fs::write(&path, "test content").unwrap();
        FileMeta::from_path(&path).unwrap()
    }

    #[test]
    fn test_trash_item_creation() {
        let meta = create_test_meta();
        let temp_dir = TempDir::new().unwrap();
        let trash_path = temp_dir.path().join("test_file");
        fs::write(&trash_path, "test content").unwrap();

        let item = TrashItem::new(meta.clone(), trash_path.clone());

        assert_eq!(item.meta.id, meta.id);
        assert_eq!(item.trash_path, trash_path);
        assert!(item.exists());
    }

    #[test]
    fn test_actual_size() {
        let meta = create_test_meta();
        let temp_dir = TempDir::new().unwrap();
        let trash_path = temp_dir.path().join("test_file");
        fs::write(&trash_path, "different content").unwrap();

        let item = TrashItem::new(meta, trash_path);

        let actual_size = item.actual_size().unwrap();
        assert_eq!(actual_size, "different content".len() as u64);
    }

    #[test]
    fn test_trash_subdirectory() {
        let meta = create_test_meta();
        let temp_dir = TempDir::new().unwrap();
        let trash_path = temp_dir.path().join("test_file");

        let item = TrashItem::new(meta, trash_path);
        let subdir = item.trash_subdirectory();

        // Should be in format YYYY-MM-DD
        assert_eq!(subdir.len(), 10);
        assert!(subdir.contains("-"));
    }

    #[test]
    fn test_metadata_path() {
        let meta = create_test_meta();
        let temp_dir = TempDir::new().unwrap();
        let trash_path = temp_dir.path().join("abc123.file");

        let item = TrashItem::new(meta, trash_path);
        let meta_path = item.metadata_path();

        assert_eq!(
            meta_path.to_string_lossy(),
            format!("{}/abc123.meta.json", temp_dir.path().display())
        );
    }
}
