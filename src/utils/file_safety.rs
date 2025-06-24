use anyhow::Result;
use dialoguer::Select;
use std::path::{Path, PathBuf};

/// Possible actions when a restore target already exists
pub enum RestoreAction {
    Overwrite,
    Skip,
    Rename,
    Cancel,
    Proceed,
}

/// Check if the restore path already exists and ask user how to proceed
pub fn check_existing_file(restore_path: &Path) -> Result<RestoreAction> {
    if restore_path.exists() {
        let metadata = std::fs::metadata(restore_path)?;
        println!("\u{26a0}\u{fe0f}  File already exists:");
        println!("   Path: {}", restore_path.display());
        println!("   Size: {}", format_size(metadata.len()));
        if let Ok(modified) = metadata.modified() {
            println!("   Modified: {:?}", modified);
        }

        let selection = Select::new()
            .with_prompt("How to proceed?")
            .items(&[
                "Overwrite existing file",
                "Skip restoration",
                "Rename restored file",
                "Cancel operation",
            ])
            .default(1)
            .interact()?;

        let action = match selection {
            0 => RestoreAction::Overwrite,
            1 => RestoreAction::Skip,
            2 => RestoreAction::Rename,
            _ => RestoreAction::Cancel,
        };
        Ok(action)
    } else {
        Ok(RestoreAction::Proceed)
    }
}

/// Generate a non-conflicting restore path by appending an incremental suffix
pub fn generate_safe_restore_path(original: &Path) -> PathBuf {
    let mut counter = 1;
    loop {
        let new_path = if let Some(stem) = original.file_stem() {
            let ext = original
                .extension()
                .map(|e| format!(".{}", e.to_string_lossy()))
                .unwrap_or_default();
            original.with_file_name(format!(
                "{}_restored_{}{}",
                stem.to_string_lossy(),
                counter,
                ext
            ))
        } else {
            original.with_extension(format!("restored_{}", counter))
        };

        if !new_path.exists() {
            return new_path;
        }
        counter += 1;
    }
}

fn format_size(size: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size_f = size as f64;
    let mut unit_index = 0;

    while size_f >= 1024.0 && unit_index < UNITS.len() - 1 {
        size_f /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", size, UNITS[0])
    } else {
        format!("{:.1} {}", size_f, UNITS[unit_index])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs;

    #[test]
    fn test_generate_safe_restore_path() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("example.txt");
        fs::write(&file, "a").unwrap();
        let new_path = generate_safe_restore_path(&file);
        assert!(new_path != file);
        assert!(!new_path.exists());
    }

    #[test]
    fn test_check_existing_file_proceed() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("missing.txt");
        let action = check_existing_file(&path).unwrap();
        matches!(action, RestoreAction::Proceed);
    }
}

