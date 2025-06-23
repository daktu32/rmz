use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Path to trash zone directory
    pub trash_path: PathBuf,

    /// Paths that are protected from deletion
    pub protected_paths: Vec<PathBuf>,

    /// Automatically clean files older than N days (None = disabled)
    pub auto_clean_days: Option<u32>,

    /// Maximum trash zone size in bytes (None = unlimited)
    pub max_trash_size: Option<u64>,

    /// Enable colored output
    pub colors: bool,

    /// Default behavior for confirmations
    pub require_confirmation: bool,

    /// Enable fzf integration when available
    pub use_fzf: bool,

    /// Date format for display
    pub date_format: String,
}

impl Default for Config {
    fn default() -> Self {
        let trash_path = Self::default_trash_path();

        Self {
            trash_path,
            protected_paths: Self::default_protected_paths(),
            auto_clean_days: Some(30),
            max_trash_size: Some(1024 * 1024 * 1024), // 1GB
            colors: true,
            require_confirmation: true,
            use_fzf: true,
            date_format: "%Y-%m-%d %H:%M:%S".to_string(),
        }
    }
}

impl Config {
    /// Get the default trash directory path
    pub fn default_trash_path() -> PathBuf {
        if let Some(data_dir) = directories::ProjectDirs::from("", "", "rmz") {
            data_dir.data_dir().join("trash")
        } else {
            // Fallback for systems without XDG support
            std::env::var("HOME")
                .map(|home| PathBuf::from(home).join(".rmz").join("trash"))
                .unwrap_or_else(|_| PathBuf::from("/tmp/rmz/trash"))
        }
    }

    /// Get default protected paths
    pub fn default_protected_paths() -> Vec<PathBuf> {
        vec![
            PathBuf::from("/bin"),
            PathBuf::from("/boot"),
            PathBuf::from("/dev"),
            PathBuf::from("/etc"),
            PathBuf::from("/lib"),
            PathBuf::from("/lib64"),
            PathBuf::from("/proc"),
            PathBuf::from("/root"),
            PathBuf::from("/run"),
            PathBuf::from("/sbin"),
            PathBuf::from("/sys"),
            PathBuf::from("/usr"),
            PathBuf::from("/var"),
        ]
    }

    /// Get the metadata directory path
    pub fn metadata_path(&self) -> PathBuf {
        self.trash_path
            .parent()
            .unwrap_or(&self.trash_path)
            .join("metadata")
    }

    /// Get the logs directory path
    pub fn logs_path(&self) -> PathBuf {
        self.trash_path
            .parent()
            .unwrap_or(&self.trash_path)
            .join("logs")
    }

    /// Get the configuration file path
    pub fn config_file_path() -> PathBuf {
        if let Some(config_dir) = directories::ProjectDirs::from("", "", "rmz") {
            config_dir.config_dir().join("config.toml")
        } else {
            // Fallback
            std::env::var("HOME")
                .map(|home| {
                    PathBuf::from(home)
                        .join(".config")
                        .join("rmz")
                        .join("config.toml")
                })
                .unwrap_or_else(|_| PathBuf::from("/tmp/rmz/config.toml"))
        }
    }

    /// Check if a path is protected
    pub fn is_protected(&self, path: &PathBuf) -> bool {
        // Convert to absolute path for comparison
        let abs_path = if path.is_absolute() {
            path.clone()
        } else {
            std::env::current_dir().unwrap_or_default().join(path)
        };

        for protected in &self.protected_paths {
            if abs_path.starts_with(protected) {
                return true;
            }
        }
        false
    }

    /// Add a path to protected paths
    pub fn add_protected_path(&mut self, path: PathBuf) {
        if !self.protected_paths.contains(&path) {
            self.protected_paths.push(path);
        }
    }

    /// Remove a path from protected paths
    pub fn remove_protected_path(&mut self, path: &PathBuf) -> bool {
        if let Some(pos) = self.protected_paths.iter().position(|p| p == path) {
            self.protected_paths.remove(pos);
            true
        } else {
            false
        }
    }

    /// Ensure all necessary directories exist
    pub fn ensure_directories(&self) -> anyhow::Result<()> {
        std::fs::create_dir_all(&self.trash_path)?;
        std::fs::create_dir_all(self.metadata_path())?;
        std::fs::create_dir_all(self.logs_path())?;

        // Create config directory if it doesn't exist
        if let Some(parent) = Self::config_file_path().parent() {
            std::fs::create_dir_all(parent)?;
        }

        Ok(())
    }

    /// Load configuration from file
    pub fn load() -> anyhow::Result<Self> {
        let config_path = Self::config_file_path();

        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            let config: Config = toml::from_str(&content)?;
            Ok(config)
        } else {
            // Create default config file
            let default_config = Config::default();
            default_config.save()?;
            Ok(default_config)
        }
    }

    /// Save configuration to file
    pub fn save(&self) -> anyhow::Result<()> {
        let config_path = Self::config_file_path();

        // Ensure parent directory exists
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = toml::to_string_pretty(self)?;
        std::fs::write(&config_path, content)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_default_config() {
        let config = Config::default();

        assert!(!config.protected_paths.is_empty());
        assert!(config.colors);
        assert!(config.require_confirmation);
        assert!(config.use_fzf);
        assert_eq!(config.auto_clean_days, Some(30));
    }

    #[test]
    fn test_is_protected() {
        let config = Config::default();

        assert!(config.is_protected(&PathBuf::from("/etc/passwd")));
        assert!(config.is_protected(&PathBuf::from("/usr/bin/ls")));
        assert!(!config.is_protected(&PathBuf::from("/home/user/file.txt")));
    }

    #[test]
    fn test_add_remove_protected_path() {
        let mut config = Config::default();
        let new_path = PathBuf::from("/custom/protected");

        config.add_protected_path(new_path.clone());
        assert!(config.protected_paths.contains(&new_path));

        let removed = config.remove_protected_path(&new_path);
        assert!(removed);
        assert!(!config.protected_paths.contains(&new_path));

        let not_removed = config.remove_protected_path(&new_path);
        assert!(!not_removed);
    }

    #[test]
    fn test_ensure_directories() {
        let temp_dir = TempDir::new().unwrap();
        let mut config = Config::default();
        config.trash_path = temp_dir.path().join("test_trash");

        config.ensure_directories().unwrap();

        assert!(config.trash_path.exists());
        assert!(config.metadata_path().exists());
        assert!(config.logs_path().exists());
    }
}
