use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "rmz")]
#[command(about = "Safe file deletion with recovery - A modern CLI tool built in Rust")]
#[command(version)]
#[command(long_about = r#"
rmz is a modern replacement for the rm command that moves files to a trash zone 
instead of permanently deleting them. Files can be restored, listed, and managed 
safely with full metadata tracking.

Examples:
  rmz delete file.txt           # Move file to trash
  rmz restore --interactive     # Interactively restore files  
  rmz list --since=yesterday    # List recently deleted files
  rmz purge --days=30           # Permanently delete old files
"#)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Disable colored output
    #[arg(long, global = true)]
    pub no_color: bool,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Move files to trash zone
    Delete {
        /// Files or directories to delete
        #[arg(required = true)]
        paths: Vec<PathBuf>,

        /// Force deletion without confirmation
        #[arg(short, long)]
        force: bool,

        /// Show what would be deleted without actually doing it
        #[arg(long)]
        dry_run: bool,

        /// Add a tag/reason for deletion
        #[arg(short, long)]
        tag: Option<String>,

        /// Interactive mode with confirmation prompts
        #[arg(short, long)]
        interactive: bool,

        /// Recursively delete directories and their contents
        #[arg(short, long)]
        recursive: bool,
    },

    /// Restore files from trash zone
    Restore {
        /// Specific file name or pattern to restore
        #[arg(conflicts_with = "interactive")]
        file: Option<String>,

        /// File ID to restore
        #[arg(long, conflicts_with_all = ["file", "interactive"])]
        id: Option<String>,

        /// Interactive selection using fuzzy finder
        #[arg(short, long)]
        interactive: bool,

        /// Restore all files matching pattern
        #[arg(long)]
        all: bool,

        /// Restore to specific path instead of original location
        #[arg(long)]
        to: Option<PathBuf>,

        /// Overwrite existing files without confirmation
        #[arg(long)]
        force: bool,

        /// Automatically rename restored file if target exists
        #[arg(long, conflicts_with = "force")]
        rename: bool,
    },

    /// List deleted files in trash zone
    List {
        /// Output in JSON format
        #[arg(long)]
        json: bool,

        /// Filter by file name pattern
        #[arg(long)]
        filter: Option<String>,

        /// Show files deleted since specific date (e.g., 'yesterday', '2024-01-01')
        #[arg(long)]
        since: Option<String>,

        /// Group results by date or tag
        #[arg(long, value_enum)]
        group_by: Option<GroupBy>,

        /// Maximum number of files to show
        #[arg(short, long)]
        limit: Option<usize>,
    },

    /// Show deletion history and audit logs
    Log {
        /// Show detailed operation logs
        #[arg(short, long)]
        detailed: bool,

        /// Filter by operation type
        #[arg(long, value_enum)]
        operation: Option<OperationType>,

        /// Show logs since specific date
        #[arg(long)]
        since: Option<String>,
    },

    /// Permanently delete files from trash zone
    Purge {
        /// Purge all files
        #[arg(long, conflicts_with_all = ["days", "size", "id"])]
        all: bool,

        /// Purge files older than N days
        #[arg(long)]
        days: Option<u32>,

        /// Purge when trash exceeds size limit (e.g., '100MB', '1GB')
        #[arg(long)]
        size: Option<String>,

        /// Purge specific file by ID
        #[arg(long)]
        id: Option<String>,

        /// Confirm before purging
        #[arg(short, long)]
        interactive: bool,
    },

    /// Manage protected paths
    Protect {
        #[command(subcommand)]
        action: ProtectAction,
    },

    /// Show trash zone status and statistics
    Status {
        /// Show detailed breakdown
        #[arg(short, long)]
        detailed: bool,
    },

    /// Configure rmz settings
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },

    /// Run system diagnostics
    Doctor {
        /// Fix issues automatically
        #[arg(long)]
        fix: bool,
    },

    /// Generate shell completion scripts
    Completions {
        /// Shell type
        #[arg(value_enum)]
        shell: clap_complete::Shell,
    },
}

#[derive(Subcommand, Debug)]
pub enum ProtectAction {
    /// Add path to protection list
    Add {
        /// Paths to protect from deletion
        paths: Vec<PathBuf>,
    },

    /// Remove path from protection list
    Remove {
        /// Paths to unprotect
        paths: Vec<PathBuf>,
    },

    /// List all protected paths
    List,
}

#[derive(Subcommand, Debug)]
pub enum ConfigAction {
    /// Show current configuration
    Show,

    /// Set configuration value
    Set {
        /// Configuration key
        key: String,
        /// Configuration value
        value: String,
    },

    /// Reset configuration to defaults
    Reset,
}

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum GroupBy {
    Date,
    Tag,
}

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum OperationType {
    Delete,
    Restore,
    Purge,
}
