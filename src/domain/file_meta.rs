use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

/// Metadata for a file that has been moved to trash
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FileMeta {
    /// Unique identifier for this trashed file
    pub id: Uuid,

    /// Original path where the file was located
    pub original_path: PathBuf,

    /// When the file was deleted
    pub deleted_at: DateTime<Utc>,

    /// Size of the file in bytes
    pub size: u64,

    /// Original file permissions (Unix mode)
    pub permissions: u32,

    /// Tags associated with this deletion
    pub tags: Vec<String>,

    /// Optional checksum for integrity verification
    pub checksum: Option<String>,

    /// User who performed the deletion (for future multi-user support)
    pub deleted_by: String,
}

impl FileMeta {
    /// Create new FileMeta from a file path
    pub fn from_path(path: &std::path::Path) -> anyhow::Result<Self> {
        let metadata = std::fs::metadata(path)?;
        let size = metadata.len();

        // Get permissions (Unix-specific)
        #[cfg(unix)]
        let permissions = {
            use std::os::unix::fs::PermissionsExt;
            metadata.permissions().mode()
        };

        #[cfg(not(unix))]
        let permissions = 0o644; // Default for non-Unix systems

        // Get current user
        let deleted_by = std::env::var("USER")
            .or_else(|_| std::env::var("USERNAME"))
            .unwrap_or_else(|_| "unknown".to_string());

        Ok(Self {
            id: Uuid::new_v4(),
            original_path: path.to_path_buf(),
            deleted_at: Utc::now(),
            size,
            permissions,
            tags: Vec::new(),
            checksum: None,
            deleted_by,
        })
    }

    /// Add a tag to this file
    pub fn add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
    }

    /// Get the original filename
    pub fn filename(&self) -> Option<&str> {
        self.original_path
            .file_name()
            .and_then(|name| name.to_str())
    }

    /// Get the directory where the file was originally located
    pub fn original_directory(&self) -> Option<&std::path::Path> {
        self.original_path.parent()
    }

    /// Check if this file matches a search pattern
    pub fn matches_pattern(&self, pattern: &str) -> bool {
        let filename = self.filename().unwrap_or("");
        let path_str = self.original_path.to_string_lossy();

        filename.contains(pattern)
            || path_str.contains(pattern)
            || self.tags.iter().any(|tag| tag.contains(pattern))
    }

    /// Format file size in human-readable format
    pub fn human_readable_size(&self) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
        let mut size = self.size as f64;
        let mut unit_index = 0;

        while size >= 1024.0 && unit_index < UNITS.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }

        if unit_index == 0 {
            format!("{} {}", self.size, UNITS[0])
        } else {
            format!("{:.1} {}", size, UNITS[unit_index])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;

    #[test]
    fn test_file_meta_creation() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_path_buf();
        fs::write(&path, "test content").unwrap();

        let meta = FileMeta::from_path(&path).unwrap();

        assert_eq!(meta.original_path, path);
        assert_eq!(meta.size, 12); // "test content".len()
        assert!(!meta.id.is_nil());
        assert!(meta.deleted_at <= Utc::now());
    }

    #[test]
    fn test_add_tag() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_path_buf();
        fs::write(&path, "content").unwrap();

        let mut meta = FileMeta::from_path(&path).unwrap();
        meta.add_tag("important".to_string());
        meta.add_tag("project-x".to_string());
        meta.add_tag("important".to_string()); // Duplicate should be ignored

        assert_eq!(meta.tags.len(), 2);
        assert!(meta.tags.contains(&"important".to_string()));
        assert!(meta.tags.contains(&"project-x".to_string()));
    }

    #[test]
    fn test_matches_pattern() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_path_buf();
        fs::write(&path, "content").unwrap();

        let mut meta = FileMeta::from_path(&path).unwrap();
        meta.add_tag("work".to_string());

        assert!(meta.matches_pattern("work"));
        assert!(meta.matches_pattern("tmp")); // Should match path
        assert!(!meta.matches_pattern("nonexistent"));
    }

    #[test]
    fn test_human_readable_size() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_path_buf();

        // Create different sized files
        fs::write(&path, "a").unwrap();
        let meta1 = FileMeta::from_path(&path).unwrap();
        assert_eq!(meta1.human_readable_size(), "1 B");

        fs::write(&path, &vec![0u8; 1536]).unwrap(); // 1.5 KB
        let meta2 = FileMeta::from_path(&path).unwrap();
        assert_eq!(meta2.human_readable_size(), "1.5 KB");
    }
}
