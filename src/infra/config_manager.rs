use crate::domain::Config;
use anyhow::Result;

/// Manager for application configuration
pub struct ConfigManager;

impl ConfigManager {
    /// Load configuration (wrapper around Config::load)
    pub fn load() -> Result<Config> {
        Config::load()
    }

    /// Save configuration (wrapper around Config::save)
    pub fn save(config: &Config) -> Result<()> {
        config.save()
    }

    /// Initialize configuration directories
    pub fn initialize(config: &Config) -> Result<()> {
        config.ensure_directories()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_default_config() {
        // This will create a default config if none exists
        let config = ConfigManager::load().unwrap();
        assert!(!config.protected_paths.is_empty());
        assert!(config.colors);
    }
}
