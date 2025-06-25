#[cfg(feature = "fzf")]
use duct::cmd;

use crate::domain::TrashItem;
use anyhow::{anyhow, Result};

#[cfg(feature = "colors")]
use colored::Colorize;

/// Interface for fuzzy finding operations
pub trait FzfInterface {
    fn select_trash_item(&self, items: &[TrashItem]) -> Result<Option<TrashItem>>;
    fn select_multiple_trash_items(&self, items: &[TrashItem]) -> Result<Vec<TrashItem>>;
}

/// FZF-based fuzzy finder implementation
pub struct FzfSelector {
    /// Whether fzf is available on the system
    available: bool,
}

impl FzfSelector {
    pub fn new() -> Self {
        let available = Self::check_fzf_availability();
        if !available {
            eprintln!("Warning: fzf not found. Interactive mode will use basic prompts.");
        }
        Self { available }
    }

    pub fn is_available(&self) -> bool {
        self.available
    }

    #[cfg(feature = "fzf")]
    fn check_fzf_availability() -> bool {
        which::which("fzf").is_ok()
    }

    #[cfg(not(feature = "fzf"))]
    fn check_fzf_availability() -> bool {
        false
    }

    /// Create a formatted display line for a trash item
    fn format_item(&self, item: &TrashItem, index: usize) -> String {
        let relative_time = format_relative_time(item.meta.deleted_at);
        let size_str = format_size(item.meta.size);
        
        #[cfg(feature = "colors")]
        {
            format!(
                "{}: {} {} {} {}",
                format!("{:3}", index + 1).cyan(),
                item.meta.original_path.display().to_string().white().bold(),
                format!("({})", size_str).yellow(),
                format!("deleted {}", relative_time).dimmed(),
                item.meta.id.to_string()[..8].bright_black()
            )
        }
        #[cfg(not(feature = "colors"))]
        {
            format!(
                "{:3}: {} ({}) deleted {} {}",
                index + 1,
                item.meta.original_path.display(),
                size_str,
                relative_time,
                &item.meta.id.to_string()[..8]
            )
        }
    }

    /// Parse the selected line back to find the corresponding item
    fn parse_selection(&self, selection: &str, items: &[TrashItem]) -> Result<TrashItem> {
        // Extract the index from the beginning of the line
        let parts: Vec<&str> = selection.split(':').collect();
        if parts.is_empty() {
            return Err(anyhow!("Invalid selection format"));
        }
        
        let index_str = parts[0].trim();
        let index: usize = index_str.parse()
            .map_err(|_| anyhow!("Could not parse selection index: {}", index_str))?;
        
        if index == 0 || index > items.len() {
            return Err(anyhow!("Selection index out of range: {}", index));
        }
        
        Ok(items[index - 1].clone())
    }

    #[cfg(feature = "fzf")]
    fn run_fzf(&self, input: &str, multi: bool) -> Result<String> {
        let mut fzf_cmd = cmd!("fzf", "--ansi", "--height=40%", "--layout=reverse", "--border")
            .stdin_bytes(input);
        
        if multi {
            fzf_cmd = fzf_cmd.arg("--multi");
        }
        
        let output = fzf_cmd.read()
            .map_err(|e| anyhow!("fzf execution failed: {}", e))?;
        
        if output.trim().is_empty() {
            return Err(anyhow!("No selection made"));
        }
        
        Ok(output)
    }

    #[cfg(not(feature = "fzf"))]
    fn run_fzf(&self, _input: &str, _multi: bool) -> Result<String> {
        Err(anyhow!("fzf feature not enabled"))
    }
}

impl FzfInterface for FzfSelector {
    fn select_trash_item(&self, items: &[TrashItem]) -> Result<Option<TrashItem>> {
        if items.is_empty() {
            return Ok(None);
        }

        if !self.available {
            return Err(anyhow!("fzf is not available"));
        }

        // Format items for fzf display
        let formatted_items: Vec<String> = items
            .iter()
            .enumerate()
            .map(|(i, item)| self.format_item(item, i))
            .collect();
        
        let input = formatted_items.join("\n");
        
        match self.run_fzf(&input, false) {
            Ok(selection) => {
                let selected_item = self.parse_selection(selection.trim(), items)?;
                Ok(Some(selected_item))
            }
            Err(e) => {
                if e.to_string().contains("No selection made") {
                    Ok(None)
                } else {
                    Err(e)
                }
            }
        }
    }

    fn select_multiple_trash_items(&self, items: &[TrashItem]) -> Result<Vec<TrashItem>> {
        if items.is_empty() {
            return Ok(Vec::new());
        }

        if !self.available {
            return Err(anyhow!("fzf is not available"));
        }

        // Format items for fzf display
        let formatted_items: Vec<String> = items
            .iter()
            .enumerate()
            .map(|(i, item)| self.format_item(item, i))
            .collect();
        
        let input = formatted_items.join("\n");
        
        match self.run_fzf(&input, true) {
            Ok(selections) => {
                let mut selected_items = Vec::new();
                for line in selections.lines() {
                    if !line.trim().is_empty() {
                        let item = self.parse_selection(line.trim(), items)?;
                        selected_items.push(item);
                    }
                }
                Ok(selected_items)
            }
            Err(e) => {
                if e.to_string().contains("No selection made") {
                    Ok(Vec::new())
                } else {
                    Err(e)
                }
            }
        }
    }
}

/// Fallback interactive selector using dialoguer
pub struct DialoguerSelector;

impl DialoguerSelector {
    pub fn new() -> Self {
        Self
    }
}

impl FzfInterface for DialoguerSelector {
    fn select_trash_item(&self, items: &[TrashItem]) -> Result<Option<TrashItem>> {
        if items.is_empty() {
            return Ok(None);
        }

        use dialoguer::{theme::ColorfulTheme, Select};

        let formatted_items: Vec<String> = items
            .iter()
            .enumerate()
            .map(|(i, item)| {
                let relative_time = format_relative_time(item.meta.deleted_at);
                let size_str = format_size(item.meta.size);
                format!(
                    "{}: {} ({}) deleted {}",
                    i + 1,
                    item.meta.original_path.display(),
                    size_str,
                    relative_time
                )
            })
            .collect();

        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Select a file to restore")
            .items(&formatted_items)
            .interact_opt()?;

        match selection {
            Some(index) => Ok(Some(items[index].clone())),
            None => Ok(None),
        }
    }

    fn select_multiple_trash_items(&self, items: &[TrashItem]) -> Result<Vec<TrashItem>> {
        if items.is_empty() {
            return Ok(Vec::new());
        }

        use dialoguer::{theme::ColorfulTheme, MultiSelect};

        let formatted_items: Vec<String> = items
            .iter()
            .enumerate()
            .map(|(i, item)| {
                let relative_time = format_relative_time(item.meta.deleted_at);
                let size_str = format_size(item.meta.size);
                format!(
                    "{}: {} ({}) deleted {}",
                    i + 1,
                    item.meta.original_path.display(),
                    size_str,
                    relative_time
                )
            })
            .collect();

        let selections = MultiSelect::with_theme(&ColorfulTheme::default())
            .with_prompt("Select files to restore (use space to select, enter to confirm)")
            .items(&formatted_items)
            .interact()?;

        let selected_items: Vec<TrashItem> = selections
            .into_iter()
            .map(|i| items[i].clone())
            .collect();

        Ok(selected_items)
    }
}

/// Create an appropriate selector based on system capabilities
pub fn create_selector() -> Box<dyn FzfInterface> {
    let fzf_selector = FzfSelector::new();
    if fzf_selector.is_available() {
        Box::new(fzf_selector)
    } else {
        Box::new(DialoguerSelector::new())
    }
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

/// Format relative time (e.g., "2 hours ago", "3 days ago")
fn format_relative_time(datetime: chrono::DateTime<chrono::Utc>) -> String {
    let now = chrono::Utc::now();
    let duration = now.signed_duration_since(datetime);
    
    if let Ok(std_duration) = duration.to_std() {
        let total_seconds = std_duration.as_secs();
        
        if total_seconds < 60 {
            "just now".to_string()
        } else if total_seconds < 3600 {
            let minutes = total_seconds / 60;
            format!("{} minute{} ago", minutes, if minutes == 1 { "" } else { "s" })
        } else if total_seconds < 86400 {
            let hours = total_seconds / 3600;
            format!("{} hour{} ago", hours, if hours == 1 { "" } else { "s" })
        } else {
            let days = total_seconds / 86400;
            format!("{} day{} ago", days, if days == 1 { "" } else { "s" })
        }
    } else {
        "unknown".to_string()
    }
}