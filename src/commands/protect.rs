use crate::cli::ProtectAction;
use crate::domain::Config;
use anyhow::Result;
use std::path::PathBuf;

#[cfg(feature = "colors")]
use colored::Colorize;

/// Execute the protect command
pub fn execute(action: ProtectAction, verbose: bool) -> Result<()> {
    match action {
        ProtectAction::Add { paths } => add_protected_paths(paths, verbose),
        ProtectAction::Remove { paths } => remove_protected_paths(paths, verbose),
        ProtectAction::List => list_protected_paths(verbose),
    }
}

/// Add paths to the protection list
fn add_protected_paths(paths: Vec<PathBuf>, verbose: bool) -> Result<()> {
    let mut config = Config::load()?;
    let mut added_count = 0;
    let mut already_protected = Vec::new();
    
    for path in paths {
        // Canonicalize the path if possible
        let canonical_path = if path.exists() {
            std::fs::canonicalize(&path).unwrap_or(path)
        } else {
            path
        };
        
        if config.protected_paths.contains(&canonical_path) {
            already_protected.push(canonical_path);
        } else {
            config.add_protected_path(canonical_path.clone());
            added_count += 1;
            
            if verbose {
                #[cfg(feature = "colors")]
                println!("🛡️  Protected: {}", canonical_path.display().to_string().green());
                #[cfg(not(feature = "colors"))]
                println!("Protected: {}", canonical_path.display());
            }
        }
    }
    
    // Save the updated configuration
    config.save()?;
    
    // Show summary
    if added_count > 0 {
        #[cfg(feature = "colors")]
        println!("✅ Added {} path{} to protection list", 
            added_count.to_string().bold().green(),
            if added_count == 1 { "" } else { "s" }
        );
        #[cfg(not(feature = "colors"))]
        println!("Added {} path{} to protection list", 
            added_count,
            if added_count == 1 { "" } else { "s" }
        );
    }
    
    if !already_protected.is_empty() {
        #[cfg(feature = "colors")]
        println!("⚠️  {} path{} already protected:", 
            already_protected.len().to_string().yellow(),
            if already_protected.len() == 1 { " is" } else { "s are" }
        );
        #[cfg(not(feature = "colors"))]
        println!("{} path{} already protected:", 
            already_protected.len(),
            if already_protected.len() == 1 { " is" } else { "s are" }
        );
        
        for path in &already_protected {
            #[cfg(feature = "colors")]
            println!("   • {}", path.display().to_string().dimmed());
            #[cfg(not(feature = "colors"))]
            println!("   • {}", path.display());
        }
    }
    
    Ok(())
}

/// Remove paths from the protection list
fn remove_protected_paths(paths: Vec<PathBuf>, verbose: bool) -> Result<()> {
    let mut config = Config::load()?;
    let mut removed_count = 0;
    let mut not_protected = Vec::new();
    
    for path in paths {
        // Try to match by exact path or canonicalized path
        let canonical_path = if path.exists() {
            std::fs::canonicalize(&path).unwrap_or(path.clone())
        } else {
            path.clone()
        };
        
        let removed = config.remove_protected_path(&path) || 
                     config.remove_protected_path(&canonical_path);
        
        if removed {
            removed_count += 1;
            
            if verbose {
                #[cfg(feature = "colors")]
                println!("🔓 Unprotected: {}", path.display().to_string().red());
                #[cfg(not(feature = "colors"))]
                println!("Unprotected: {}", path.display());
            }
        } else {
            not_protected.push(path);
        }
    }
    
    // Save the updated configuration
    config.save()?;
    
    // Show summary
    if removed_count > 0 {
        #[cfg(feature = "colors")]
        println!("✅ Removed {} path{} from protection list", 
            removed_count.to_string().bold().green(),
            if removed_count == 1 { "" } else { "s" }
        );
        #[cfg(not(feature = "colors"))]
        println!("Removed {} path{} from protection list", 
            removed_count,
            if removed_count == 1 { "" } else { "s" }
        );
    }
    
    if !not_protected.is_empty() {
        #[cfg(feature = "colors")]
        println!("⚠️  {} path{} not found in protection list:", 
            not_protected.len().to_string().yellow(),
            if not_protected.len() == 1 { " was" } else { "s were" }
        );
        #[cfg(not(feature = "colors"))]
        println!("{} path{} not found in protection list:", 
            not_protected.len(),
            if not_protected.len() == 1 { " was" } else { "s were" }
        );
        
        for path in &not_protected {
            #[cfg(feature = "colors")]
            println!("   • {}", path.display().to_string().dimmed());
            #[cfg(not(feature = "colors"))]
            println!("   • {}", path.display());
        }
    }
    
    Ok(())
}

/// List all protected paths
fn list_protected_paths(verbose: bool) -> Result<()> {
    let config = Config::load()?;
    
    if config.protected_paths.is_empty() {
        println!("No paths are currently protected");
        return Ok(());
    }
    
    #[cfg(feature = "colors")]
    println!("{}", "Protected Paths:".bold().underline());
    #[cfg(not(feature = "colors"))]
    println!("Protected Paths:");
    println!();
    
    // Group paths by category for better organization
    let (system_paths, user_paths) = categorize_paths(&config.protected_paths);
    
    if !system_paths.is_empty() {
        #[cfg(feature = "colors")]
        println!("{}", "System Paths:".cyan().bold());
        #[cfg(not(feature = "colors"))]
        println!("System Paths:");
        
        for path in &system_paths {
            let status = if path.exists() { "✅" } else { "❌" };
            
            #[cfg(feature = "colors")]
            println!("  {} 🛡️  {}", 
                status, 
                path.display().to_string().white()
            );
            #[cfg(not(feature = "colors"))]
            println!("  {} {}", status, path.display());
        }
        println!();
    }
    
    if !user_paths.is_empty() {
        #[cfg(feature = "colors")]
        println!("{}", "User-defined Paths:".green().bold());
        #[cfg(not(feature = "colors"))]
        println!("User-defined Paths:");
        
        for path in &user_paths {
            let status = if path.exists() { "✅" } else { "❌" };
            
            #[cfg(feature = "colors")]
            println!("  {} 🔒 {}", 
                status, 
                path.display().to_string().white()
            );
            #[cfg(not(feature = "colors"))]
            println!("  {} {}", status, path.display());
        }
        println!();
    }
    
    if verbose {
        let total_count = config.protected_paths.len();
        let existing_count = config.protected_paths.iter()
            .filter(|p| p.exists())
            .count();
        let missing_count = total_count - existing_count;
        
        #[cfg(feature = "colors")]
        {
            println!("{}", "Summary:".bold().underline());
            println!("  Total protected paths: {}", total_count.to_string().bold());
            println!("  Existing paths: {}", existing_count.to_string().green());
            if missing_count > 0 {
                println!("  Missing paths: {}", missing_count.to_string().red());
            }
        }
        #[cfg(not(feature = "colors"))]
        {
            println!("Summary:");
            println!("  Total protected paths: {}", total_count);
            println!("  Existing paths: {}", existing_count);
            if missing_count > 0 {
                println!("  Missing paths: {}", missing_count);
            }
        }
    }
    
    Ok(())
}

/// Categorize paths into system and user-defined paths
fn categorize_paths(paths: &[PathBuf]) -> (Vec<PathBuf>, Vec<PathBuf>) {
    let default_protected = Config::default_protected_paths();
    let mut system_paths = Vec::new();
    let mut user_paths = Vec::new();
    
    for path in paths {
        if default_protected.contains(path) {
            system_paths.push(path.clone());
        } else {
            user_paths.push(path.clone());
        }
    }
    
    // Sort paths for consistent display
    system_paths.sort();
    user_paths.sort();
    
    (system_paths, user_paths)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;
    
    #[test]
    fn test_categorize_paths() {
        let default_paths = Config::default_protected_paths();
        let custom_path = PathBuf::from("/custom/path");
        
        let mut test_paths = default_paths.clone();
        test_paths.push(custom_path.clone());
        
        let (system_paths, user_paths) = categorize_paths(&test_paths);
        
        // All default paths should be in system_paths
        for default_path in &default_paths {
            assert!(system_paths.contains(default_path));
        }
        
        // Custom path should be in user_paths
        assert!(user_paths.contains(&custom_path));
        assert_eq!(user_paths.len(), 1);
    }
    
    #[test]
    fn test_add_protected_paths() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        fs::write(&test_file, "test").unwrap();
        
        // Create a minimal config for testing
        let mut config = Config::default();
        config.protected_paths.clear(); // Start with empty list
        
        // Test adding a new path
        config.add_protected_path(test_file.clone());
        assert!(config.protected_paths.contains(&test_file));
        
        // Test adding duplicate path (should not duplicate)
        let original_len = config.protected_paths.len();
        config.add_protected_path(test_file.clone());
        assert_eq!(config.protected_paths.len(), original_len);
    }
    
    #[test]
    fn test_remove_protected_paths() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        fs::write(&test_file, "test").unwrap();
        
        let mut config = Config::default();
        config.protected_paths.clear();
        config.add_protected_path(test_file.clone());
        
        // Test removing existing path
        let removed = config.remove_protected_path(&test_file);
        assert!(removed);
        assert!(!config.protected_paths.contains(&test_file));
        
        // Test removing non-existent path
        let removed = config.remove_protected_path(&test_file);
        assert!(!removed);
    }
}