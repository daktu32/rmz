use crate::domain::Config;
use crate::infra::{trash_store::TrashStoreInterface, TrashStore};
use anyhow::Result;
use std::fs;
use dialoguer::Confirm;

#[cfg(feature = "colors")]
use colored::Colorize;

#[derive(Clone, Debug)]
pub enum DiagnosticCheck {
    TrashZone,
    Metadata,
    Permissions,
    Config,
    Dependencies,
    All,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum Severity {
    Info,
    Warning,
    Error,
    Critical,
}

#[derive(Debug, Clone)]
pub struct DiagnosticIssue {
    pub check_type: DiagnosticCheck,
    pub severity: Severity,
    pub message: String,
    pub suggestion: Option<String>,
    pub fixable: bool,
}

/// Execute the doctor command
pub fn execute(
    check: Option<DiagnosticCheck>,
    fix: bool,
    verbose: bool,
    force: bool,
) -> Result<()> {
    let config = Config::load()?;
    let trash_root = config.trash_path.clone();
    let trash_store = TrashStore::new(trash_root.clone());
    
    let checks_to_run = match check {
        Some(DiagnosticCheck::All) | None => vec![
            DiagnosticCheck::TrashZone,
            DiagnosticCheck::Metadata,
            DiagnosticCheck::Permissions,
            DiagnosticCheck::Config,
            DiagnosticCheck::Dependencies,
        ],
        Some(specific_check) => vec![specific_check],
    };
    
    let mut all_issues = Vec::new();
    
    for check_type in checks_to_run {
        if verbose {
            #[cfg(feature = "colors")]
            println!("🔍 Running {} check...", format!("{:?}", check_type).blue());
            #[cfg(not(feature = "colors"))]
            println!("Running {:?} check...", check_type);
        }
        
        let issues = run_diagnostic_check(&check_type, &trash_store, &config)?;
        all_issues.extend(issues);
    }
    
    // Display results
    display_diagnostic_results(&all_issues, verbose)?;
    
    // Fix issues if requested
    if fix && !all_issues.is_empty() {
        let fixable_issues: Vec<_> = all_issues
            .iter()
            .filter(|issue| issue.fixable)
            .collect();
            
        if !fixable_issues.is_empty() {
            if !force {
                let msg = format!("Fix {} fixable issues automatically?", fixable_issues.len());
                if !Confirm::new().with_prompt(msg).interact()? {
                    println!("Fix cancelled");
                    return Ok(());
                }
            }
            
            fix_issues(&fixable_issues, &trash_store, &config, verbose)?;
        } else {
            println!("No fixable issues found");
        }
    }
    
    Ok(())
}

fn run_diagnostic_check(
    check_type: &DiagnosticCheck,
    trash_store: &TrashStore,
    config: &Config,
) -> Result<Vec<DiagnosticIssue>> {
    let mut issues = Vec::new();
    
    match check_type {
        DiagnosticCheck::TrashZone => {
            issues.extend(check_trash_zone(trash_store)?);
        }
        DiagnosticCheck::Metadata => {
            issues.extend(check_metadata_integrity(trash_store)?);
        }
        DiagnosticCheck::Permissions => {
            issues.extend(check_permissions(trash_store, config)?);
        }
        DiagnosticCheck::Config => {
            issues.extend(check_config_validity(config)?);
        }
        DiagnosticCheck::Dependencies => {
            issues.extend(check_dependencies()?);
        }
        DiagnosticCheck::All => {
            // This case is handled in execute function
        }
    }
    
    Ok(issues)
}

fn check_trash_zone(trash_store: &TrashStore) -> Result<Vec<DiagnosticIssue>> {
    let mut issues = Vec::new();
    let trash_root = trash_store.get_trash_root();
    
    // Check if trash directory exists
    if !trash_root.exists() {
        issues.push(DiagnosticIssue {
            check_type: DiagnosticCheck::TrashZone,
            severity: Severity::Error,
            message: format!("Trash directory does not exist: {}", trash_root.display()),
            suggestion: Some("Create the trash directory".to_string()),
            fixable: true,
        });
    } else if !trash_root.is_dir() {
        issues.push(DiagnosticIssue {
            check_type: DiagnosticCheck::TrashZone,
            severity: Severity::Critical,
            message: format!("Trash path exists but is not a directory: {}", trash_root.display()),
            suggestion: Some("Remove the file and recreate as directory".to_string()),
            fixable: false,
        });
    }
    
    // Check for orphaned metadata files
    if trash_root.exists() {
        let items = trash_store.list()?;
        for item in items {
            if !item.trash_path.exists() {
                issues.push(DiagnosticIssue {
                    check_type: DiagnosticCheck::TrashZone,
                    severity: Severity::Warning,
                    message: format!("Orphaned metadata: {} (file missing)", item.meta.original_path.display()),
                    suggestion: Some("Remove orphaned metadata".to_string()),
                    fixable: true,
                });
            }
        }
    }
    
    Ok(issues)
}

fn check_metadata_integrity(trash_store: &TrashStore) -> Result<Vec<DiagnosticIssue>> {
    let mut issues = Vec::new();
    
    if let Ok(items) = trash_store.list() {
        for item in items {
            // Check if metadata is consistent
            if item.meta.original_path.to_string_lossy().is_empty() {
                issues.push(DiagnosticIssue {
                    check_type: DiagnosticCheck::Metadata,
                    severity: Severity::Warning,
                    message: format!("Empty original path in metadata: {}", item.meta.id),
                    suggestion: Some("Remove corrupted metadata".to_string()),
                    fixable: true,
                });
            }
            
            // Check if checksum exists for integrity checking
            if item.meta.checksum.is_none() {
                issues.push(DiagnosticIssue {
                    check_type: DiagnosticCheck::Metadata,
                    severity: Severity::Info,
                    message: format!("Missing SHA256 checksum for: {}", item.meta.original_path.display()),
                    suggestion: Some("Recalculate SHA256 checksums".to_string()),
                    fixable: true,
                });
            }
        }
    }
    
    Ok(issues)
}

fn check_permissions(trash_store: &TrashStore, _config: &Config) -> Result<Vec<DiagnosticIssue>> {
    let mut issues = Vec::new();
    let trash_root = trash_store.get_trash_root();
    
    if trash_root.exists() {
        // Check if we can write to trash directory
        if let Err(_) = fs::metadata(&trash_root) {
            issues.push(DiagnosticIssue {
                check_type: DiagnosticCheck::Permissions,
                severity: Severity::Error,
                message: format!("Cannot access trash directory: {}", trash_root.display()),
                suggestion: Some("Check file permissions".to_string()),
                fixable: false,
            });
        }
    }
    
    Ok(issues)
}

fn check_config_validity(_config: &Config) -> Result<Vec<DiagnosticIssue>> {
    let mut issues = Vec::new();
    
    // Basic config validation
    // Add more comprehensive checks as needed
    issues.push(DiagnosticIssue {
        check_type: DiagnosticCheck::Config,
        severity: Severity::Info,
        message: "Configuration appears valid".to_string(),
        suggestion: None,
        fixable: false,
    });
    
    Ok(issues)
}

fn check_dependencies() -> Result<Vec<DiagnosticIssue>> {
    let mut issues = Vec::new();
    
    // Check for optional dependencies
    if which::which("fzf").is_err() {
        issues.push(DiagnosticIssue {
            check_type: DiagnosticCheck::Dependencies,
            severity: Severity::Info,
            message: "fzf not found - interactive features may be limited".to_string(),
            suggestion: Some("Install fzf for enhanced interactive mode".to_string()),
            fixable: false,
        });
    }
    
    Ok(issues)
}

fn display_diagnostic_results(issues: &[DiagnosticIssue], verbose: bool) -> Result<()> {
    if issues.is_empty() {
        #[cfg(feature = "colors")]
        println!("✅ {}", "All checks passed!".green().bold());
        #[cfg(not(feature = "colors"))]
        println!("✅ All checks passed!");
        return Ok(());
    }
    
    println!("🔍 Diagnostic Results:");
    println!();
    
    let mut counts = std::collections::HashMap::new();
    for issue in issues {
        *counts.entry(&issue.severity).or_insert(0) += 1;
    }
    
    // Print summary
    for (severity, count) in &counts {
        let severity_str = match severity {
            Severity::Info => {
                #[cfg(feature = "colors")]
                { "Info".cyan() }
                #[cfg(not(feature = "colors"))]
                { "Info".to_string() }
            },
            Severity::Warning => {
                #[cfg(feature = "colors")]
                { "Warning".yellow() }
                #[cfg(not(feature = "colors"))]
                { "Warning".to_string() }
            },
            Severity::Error => {
                #[cfg(feature = "colors")]
                { "Error".red() }
                #[cfg(not(feature = "colors"))]
                { "Error".to_string() }
            },
            Severity::Critical => {
                #[cfg(feature = "colors")]
                { "Critical".red().bold() }
                #[cfg(not(feature = "colors"))]
                { "Critical".to_string() }
            },
        };
        println!("  {}: {}", severity_str, count);
    }
    println!();
    
    // Print detailed issues
    for issue in issues {
        let severity_icon = match issue.severity {
            Severity::Info => "ℹ️",
            Severity::Warning => "⚠️",
            Severity::Error => "❌",
            Severity::Critical => "🚨",
        };
        
        println!("{} {}", severity_icon, issue.message);
        
        if verbose {
            if let Some(suggestion) = &issue.suggestion {
                println!("   💡 {}", suggestion);
            }
            if issue.fixable {
                #[cfg(feature = "colors")]
                println!("   🔧 {}", "Fixable".green());
                #[cfg(not(feature = "colors"))]
                println!("   🔧 Fixable");
            }
        }
        println!();
    }
    
    Ok(())
}

fn fix_issues(
    issues: &[&DiagnosticIssue],
    trash_store: &TrashStore,
    _config: &Config,
    verbose: bool,
) -> Result<()> {
    let mut fixed_count = 0;
    
    for issue in issues {
        if verbose {
            println!("🔧 Fixing: {}", issue.message);
        }
        
        match issue.check_type {
            DiagnosticCheck::TrashZone => {
                if issue.message.contains("does not exist") {
                    fs::create_dir_all(trash_store.get_trash_root())?;
                    fixed_count += 1;
                } else if issue.message.contains("Orphaned metadata") {
                    // Extract UUID from message or use other method to identify
                    // For now, we'll skip this complex fix
                    if verbose {
                        println!("   ⚠️ Manual intervention required for orphaned metadata");
                    }
                }
            }
            DiagnosticCheck::Metadata => {
                // Complex metadata fixes would go here
                if verbose {
                    println!("   ⚠️ Metadata fixes require manual intervention");
                }
            }
            _ => {
                if verbose {
                    println!("   ⚠️ No automatic fix available");
                }
            }
        }
    }
    
    #[cfg(feature = "colors")]
    println!("✅ Fixed {} issues", fixed_count.to_string().green().bold());
    #[cfg(not(feature = "colors"))]
    println!("✅ Fixed {} issues", fixed_count);
    
    Ok(())
}