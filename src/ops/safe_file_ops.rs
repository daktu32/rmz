use anyhow::{Context, Result};
use std::fs::{self, File, Metadata};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[derive(Debug, Clone)]
pub struct FileSnapshot {
    pub path: PathBuf,
    pub inode: u64,
    pub size: u64,
    pub modified: SystemTime,
    pub permissions: u32,
}

impl FileSnapshot {
    pub fn capture(path: &Path) -> Result<Self> {
        let metadata = fs::metadata(path)
            .with_context(|| format!("Cannot read metadata: {}", path.display()))?;
        Ok(Self::from_metadata(path.to_path_buf(), &metadata))
    }

    pub fn from_metadata(path: PathBuf, metadata: &Metadata) -> Self {
        #[cfg(unix)]
        use std::os::unix::fs::MetadataExt;

        #[cfg(unix)]
        let inode = metadata.ino();
        #[cfg(not(unix))]
        let inode = 0;

        #[cfg(unix)]
        let permissions = metadata.mode();
        #[cfg(not(unix))]
        let permissions = 0u32;

        Self {
            path,
            inode,
            size: metadata.len(),
            modified: metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH),
            permissions,
        }
    }

    pub fn verify_unchanged(&self) -> Result<()> {
        let current = Self::capture(&self.path)?;
        if current.inode != self.inode
            || current.size != self.size
            || current.modified != self.modified
            || current.permissions != self.permissions
        {
            anyhow::bail!("File changed during operation: {}", self.path.display());
        }
        Ok(())
    }
}

pub struct SafeFileOperation;

impl SafeFileOperation {
    /// Perform an atomic file move with TOCTOU checks.
    pub fn atomic_move(source: &Path, target: &Path) -> Result<()> {
        let source_file = File::open(source)
            .with_context(|| format!("Cannot open source file: {}", source.display()))?;
        let source_metadata = source_file
            .metadata()
            .with_context(|| "Cannot read source file metadata")?;

        if !source_metadata.is_file() {
            anyhow::bail!("Source is not a regular file: {}", source.display());
        }

        let snapshot = FileSnapshot::from_metadata(source.to_path_buf(), &source_metadata);

        Self::execute_atomic_move(&source_file, source, target, &snapshot)
    }

    fn execute_atomic_move(
        source_file: &File,
        source_path: &Path,
        target_path: &Path,
        snapshot: &FileSnapshot,
    ) -> Result<()> {
        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent)?;
        }

        #[cfg(unix)]
        {
            snapshot.verify_unchanged()?;
            fs::rename(source_path, target_path)
                .with_context(|| "Atomic rename failed")?;
        }

        #[cfg(windows)]
        {
            snapshot.verify_unchanged()?;
            Self::windows_atomic_move(source_path, target_path)?;
        }

        let _ = source_file; // suppress unused variable on non-Unix
        Ok(())
    }

    #[cfg(windows)]
    fn windows_atomic_move(source: &Path, target: &Path) -> Result<()> {
        // TODO: use Windows MoveFileEx for true atomicity
        fs::rename(source, target).with_context(|| "Atomic move failed on Windows")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn snapshot_detects_change() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("test.txt");
        fs::write(&file_path, "content").unwrap();

        let snapshot = FileSnapshot::capture(&file_path).unwrap();
        fs::write(&file_path, "new content").unwrap();

        assert!(snapshot.verify_unchanged().is_err());
    }

    #[test]
    fn atomic_move_moves_file() {
        let dir = TempDir::new().unwrap();
        let src = dir.path().join("src.txt");
        let dst = dir.path().join("dst.txt");
        fs::write(&src, "hello").unwrap();

        SafeFileOperation::atomic_move(&src, &dst).unwrap();

        assert!(!src.exists());
        assert!(dst.exists());
        assert_eq!(fs::read_to_string(&dst).unwrap(), "hello");
    }
}
