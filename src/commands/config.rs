use crate::cli::ConfigAction;
use crate::domain::Config;
use anyhow::{anyhow, Result};
use std::path::PathBuf;

#[cfg(feature = "colors")]
use colored::Colorize;

/// Execute the config command
pub fn execute(action: ConfigAction, verbose: bool) -> Result<()> {
    match action {
        ConfigAction::Show => show_config(verbose),
        ConfigAction::Set { key, value } => set_config_value(key, value, verbose),
        ConfigAction::Reset => reset_config(verbose),
    }
}

/// Show current configuration
fn show_config(verbose: bool) -> Result<()> {
    let config = Config::load()?;
    let config_path = Config::config_file_path();
    
    #[cfg(feature = "colors")]
    println!("{}", "Current Configuration:".bold().underline());
    #[cfg(not(feature = "colors"))]
    println!("Current Configuration:");
    println!();
    
    // Show config file location
    if verbose {
        #[cfg(feature = "colors")]
        println!("📁 Config file: {}", config_path.display().to_string().cyan());
        #[cfg(not(feature = "colors"))]
        println!("Config file: {}", config_path.display());
        println!();
    }
    
    // Core settings
    #[cfg(feature = "colors")]
    println!("{}", "Core Settings:".green().bold());
    #[cfg(not(feature = "colors"))]
    println!("Core Settings:");
    
    println!("  trash_path: {}", config.trash_path.display());
    println!("  auto_clean_days: {}", 
        config.auto_clean_days.map_or("disabled".to_string(), |days| days.to_string())
    );
    println!("  max_trash_size: {}", 
        config.max_trash_size.map_or("unlimited".to_string(), |size| format_size(size))
    );
    println!();
    
    // UI settings
    #[cfg(feature = "colors")]
    println!("{}", "UI Settings:".blue().bold());
    #[cfg(not(feature = "colors"))]
    println!("UI Settings:");
    
    println!("  colors: {}", config.colors);
    println!("  require_confirmation: {}", config.require_confirmation);
    println!("  use_fzf: {}", config.use_fzf);
    println!("  date_format: {}", config.date_format);
    println!();
    
    // Protected paths
    #[cfg(feature = "colors")]
    println!("{}", "Protected Paths:".red().bold());
    #[cfg(not(feature = "colors"))]
    println!("Protected Paths:");
    
    if config.protected_paths.is_empty() {
        println!("  (none)");
    } else {
        let display_count = if verbose { config.protected_paths.len() } else { 5 };
        
        for (i, path) in config.protected_paths.iter().enumerate() {
            if i >= display_count {
                let remaining = config.protected_paths.len() - display_count;
                #[cfg(feature = "colors")]
                println!("  ... and {} more (use --verbose to see all)", remaining.to_string().dimmed());
                #[cfg(not(feature = "colors"))]
                println!("  ... and {} more (use --verbose to see all)", remaining);
                break;
            }
            println!("  {}", path.display());
        }
    }
    println!();
    
    // Storage info
    if verbose {
        show_storage_info(&config)?;
    }
    
    Ok(())
}

/// Set a configuration value
fn set_config_value(key: String, value: String, verbose: bool) -> Result<()> {
    let mut config = Config::load()?;
    let original_value = value.clone(); // Clone for later display
    
    match key.as_str() {
        "trash_path" => {
            let new_path = PathBuf::from(&value);
            if !new_path.is_absolute() {
                return Err(anyhow!("trash_path must be an absolute path"));
            }
            config.trash_path = new_path;
        }
        "auto_clean_days" => {
            if value.to_lowercase() == "disabled" || value.to_lowercase() == "none" {
                config.auto_clean_days = None;
            } else {
                let days: u32 = value.parse()
                    .map_err(|_| anyhow!("auto_clean_days must be a number or 'disabled'"))?;
                config.auto_clean_days = Some(days);
            }
        }
        "max_trash_size" => {
            if value.to_lowercase() == "unlimited" || value.to_lowercase() == "none" {
                config.max_trash_size = None;
            } else {
                let size = parse_size(&value)?;
                config.max_trash_size = Some(size);
            }
        }
        "colors" => {
            config.colors = parse_bool(&value)?;
        }
        "require_confirmation" => {
            config.require_confirmation = parse_bool(&value)?;
        }
        "use_fzf" => {
            config.use_fzf = parse_bool(&value)?;
        }
        "date_format" => {
            // Validate the date format by trying to format a test date
            let test_date = chrono::Utc::now();
            test_date.format(&value).to_string(); // This will panic if format is invalid
            config.date_format = value;
        }
        _ => {
            return Err(anyhow!("Unknown configuration key: {}", key));
        }
    }
    
    // Ensure directories exist with new config
    config.ensure_directories()?;
    
    // Save the configuration
    config.save()?;
    
    if verbose {
        #[cfg(feature = "colors")]
        println!("✅ Set {} = {}", key.green(), original_value.cyan());
        #[cfg(not(feature = "colors"))]
        println!("Set {} = {}", key, original_value);
    }
    
    #[cfg(feature = "colors")]
    println!("✅ Configuration updated successfully");
    #[cfg(not(feature = "colors"))]
    println!("Configuration updated successfully");
    
    Ok(())
}

/// Reset configuration to defaults
fn reset_config(verbose: bool) -> Result<()> {
    let config_path = Config::config_file_path();
    
    if config_path.exists() {
        std::fs::remove_file(&config_path)?;
        if verbose {
            #[cfg(feature = "colors")]
            println!("🗑️  Removed existing config file: {}", config_path.display().to_string().red());
            #[cfg(not(feature = "colors"))]
            println!("Removed existing config file: {}", config_path.display());
        }
    }
    
    // Create new default config
    let default_config = Config::default();
    default_config.ensure_directories()?;
    default_config.save()?;
    
    if verbose {
        #[cfg(feature = "colors")]
        println!("📝 Created new default config: {}", config_path.display().to_string().green());
        #[cfg(not(feature = "colors"))]
        println!("Created new default config: {}", config_path.display());
    }
    
    #[cfg(feature = "colors")]
    println!("✅ Configuration reset to defaults");
    #[cfg(not(feature = "colors"))]
    println!("Configuration reset to defaults");
    
    Ok(())
}

/// Show storage information
fn show_storage_info(config: &Config) -> Result<()> {
    #[cfg(feature = "colors")]
    println!("{}", "Storage Information:".yellow().bold());
    #[cfg(not(feature = "colors"))]
    println!("Storage Information:");
    
    // Trash directory info
    if config.trash_path.exists() {
        let trash_size = calculate_directory_size(&config.trash_path)?;
        println!("  Trash size: {}", format_size(trash_size));
        
        // Count files in trash
        let file_count = count_files_in_directory(&config.trash_path)?;
        println!("  Files in trash: {}", file_count);
    } else {
        println!("  Trash directory: (not created yet)");
    }
    
    // Metadata directory info
    let metadata_path = config.metadata_path();
    if metadata_path.exists() {
        let metadata_size = calculate_directory_size(&metadata_path)?;
        println!("  Metadata size: {}", format_size(metadata_size));
    } else {
        println!("  Metadata directory: (not created yet)");
    }
    
    // Logs directory info
    let logs_path = config.logs_path();
    if logs_path.exists() {
        let logs_size = calculate_directory_size(&logs_path)?;
        println!("  Logs size: {}", format_size(logs_size));
    } else {
        println!("  Logs directory: (not created yet)");
    }
    
    println!();
    
    Ok(())
}

/// Parse a boolean value from string
fn parse_bool(value: &str) -> Result<bool> {
    match value.to_lowercase().as_str() {
        "true" | "yes" | "on" | "1" | "enabled" => Ok(true),
        "false" | "no" | "off" | "0" | "disabled" => Ok(false),
        _ => Err(anyhow!("Invalid boolean value: {}. Use true/false, yes/no, on/off, 1/0, or enabled/disabled", value)),
    }
}

/// Parse size string (e.g., "100MB", "1GB") to bytes
fn parse_size(size_str: &str) -> Result<u64> {
    let size_str = size_str.to_uppercase();
    
    let (number_part, unit_part) = if size_str.ends_with("KB") {
        (size_str.trim_end_matches("KB"), 1024u64)
    } else if size_str.ends_with("MB") {
        (size_str.trim_end_matches("MB"), 1024u64.pow(2))
    } else if size_str.ends_with("GB") {
        (size_str.trim_end_matches("GB"), 1024u64.pow(3))
    } else if size_str.ends_with("TB") {
        (size_str.trim_end_matches("TB"), 1024u64.pow(4))
    } else if size_str.ends_with("B") {
        (size_str.trim_end_matches("B"), 1u64)
    } else {
        // Assume bytes if no unit
        (size_str.as_str(), 1u64)
    };
    
    let number: f64 = number_part.parse()
        .map_err(|_| anyhow!("Invalid size format: {}", size_str))?;
    
    Ok((number * unit_part as f64) as u64)
}

/// Format bytes to human readable size
fn format_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;
    
    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }
    
    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}

/// Calculate total size of a directory
fn calculate_directory_size(dir: &PathBuf) -> Result<u64> {
    let mut total_size = 0u64;
    
    if dir.is_dir() {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() {
                total_size += std::fs::metadata(&path)?.len();
            } else if path.is_dir() {
                total_size += calculate_directory_size(&path)?;
            }
        }
    }
    
    Ok(total_size)
}

/// Count files in a directory recursively
fn count_files_in_directory(dir: &PathBuf) -> Result<usize> {
    let mut file_count = 0;
    
    if dir.is_dir() {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() {
                file_count += 1;
            } else if path.is_dir() {
                file_count += count_files_in_directory(&path)?;
            }
        }
    }
    
    Ok(file_count)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_bool() {
        // Test true values
        assert_eq!(parse_bool("true").unwrap(), true);
        assert_eq!(parse_bool("yes").unwrap(), true);
        assert_eq!(parse_bool("on").unwrap(), true);
        assert_eq!(parse_bool("1").unwrap(), true);
        assert_eq!(parse_bool("enabled").unwrap(), true);
        assert_eq!(parse_bool("TRUE").unwrap(), true); // Case insensitive
        
        // Test false values
        assert_eq!(parse_bool("false").unwrap(), false);
        assert_eq!(parse_bool("no").unwrap(), false);
        assert_eq!(parse_bool("off").unwrap(), false);
        assert_eq!(parse_bool("0").unwrap(), false);
        assert_eq!(parse_bool("disabled").unwrap(), false);
        assert_eq!(parse_bool("FALSE").unwrap(), false); // Case insensitive
        
        // Test invalid values
        assert!(parse_bool("invalid").is_err());
        assert!(parse_bool("").is_err());
    }
    
    #[test]
    fn test_parse_size() {
        assert_eq!(parse_size("1024").unwrap(), 1024);
        assert_eq!(parse_size("1KB").unwrap(), 1024);
        assert_eq!(parse_size("1MB").unwrap(), 1024 * 1024);
        assert_eq!(parse_size("1GB").unwrap(), 1024 * 1024 * 1024);
        assert_eq!(parse_size("1.5MB").unwrap(), (1.5 * 1024.0 * 1024.0) as u64);
        assert_eq!(parse_size("500B").unwrap(), 500);
        
        // Test invalid formats
        assert!(parse_size("invalid").is_err());
        assert!(parse_size("1.5.5MB").is_err());
    }
    
    #[test]
    fn test_format_size() {
        assert_eq!(format_size(512), "512 B");
        assert_eq!(format_size(1024), "1.0 KB");
        assert_eq!(format_size(1536), "1.5 KB");
        assert_eq!(format_size(1024 * 1024), "1.0 MB");
        assert_eq!(format_size(1024 * 1024 * 1024), "1.0 GB");
    }
    
    #[test]
    fn test_calculate_directory_size() {
        use tempfile::TempDir;
        use std::fs;
        
        let temp_dir = TempDir::new().unwrap();
        let dir_path = temp_dir.path().to_path_buf();
        
        // Create some test files
        fs::write(dir_path.join("file1.txt"), "hello").unwrap();
        fs::write(dir_path.join("file2.txt"), "world").unwrap();
        
        let size = calculate_directory_size(&dir_path).unwrap();
        assert_eq!(size, 10); // "hello" (5) + "world" (5)
    }
    
    #[test]
    fn test_count_files_in_directory() {
        use tempfile::TempDir;
        use std::fs;
        
        let temp_dir = TempDir::new().unwrap();
        let dir_path = temp_dir.path().to_path_buf();
        
        // Create some test files
        fs::write(dir_path.join("file1.txt"), "content").unwrap();
        fs::write(dir_path.join("file2.txt"), "content").unwrap();
        
        // Create a subdirectory with a file
        let subdir = dir_path.join("subdir");
        fs::create_dir(&subdir).unwrap();
        fs::write(subdir.join("file3.txt"), "content").unwrap();
        
        let count = count_files_in_directory(&dir_path).unwrap();
        assert_eq!(count, 3); // 2 files in root + 1 file in subdir
    }
}