use crate::cli::OperationType;
use crate::domain::{operation_log::OperationLogger, Config};
use anyhow::Result;
use chrono::{DateTime, Utc};

#[cfg(feature = "colors")]
use colored::Colorize;

/// Execute the log command
pub fn execute(
    detailed: bool,
    operation: Option<OperationType>,
    since: Option<String>,
    verbose: bool,
) -> Result<()> {
    let config = Config::load()?;
    let log_file_path = config.logs_path().join("operations.jsonl");
    let logger = OperationLogger::new(log_file_path);
    
    // Parse since date if provided
    let since_datetime = if let Some(since_str) = since {
        Some(parse_since_date(&since_str)?)
    } else {
        None
    };
    
    // Convert CLI OperationType to domain OperationType if needed
    let operation_filter = operation.map(|op| convert_operation_type(op));
    
    // Read filtered logs
    let logs = logger.read_filtered_logs(
        operation_filter,
        since_datetime,
        None, // No limit for now
    )?;
    
    if logs.is_empty() {
        println!("No operations found");
        return Ok(());
    }
    
    if detailed {
        display_detailed_logs(&logs, verbose)
    } else {
        display_summary_logs(&logs, verbose)
    }
}

/// Display logs in detailed format
fn display_detailed_logs(logs: &[crate::domain::operation_log::OperationLog], verbose: bool) -> Result<()> {
    for (index, log) in logs.iter().enumerate() {
        if index > 0 {
            println!(); // Add spacing between entries
        }
        
        let timestamp_str = log.timestamp.format("%Y-%m-%d %H:%M:%S UTC").to_string();
        let operation_str = format!("{:?}", log.operation);
        
        #[cfg(feature = "colors")]
        {
            println!("📋 {} {}", 
                format!("Log #{}", logs.len() - index).bold().blue(),
                format!("({})", &log.id.to_string()[..8]).dimmed()
            );
            println!("🕒 {}", timestamp_str.yellow());
            println!("⚡ {}", operation_str.cyan().bold());
            println!("👤 {}", log.user.green());
            println!("📁 Paths:");
            for path in &log.paths {
                println!("   • {}", path.display().to_string().white());
            }
            if !log.file_ids.is_empty() {
                println!("🗃️  File IDs:");
                for file_id in &log.file_ids {
                    println!("   • {}", file_id.to_string()[..8].dimmed());
                }
            }
            println!("📊 {}", log.result_display());
            if let Some(context) = &log.context {
                println!("📝 Context: {}", context.italic());
            }
        }
        
        #[cfg(not(feature = "colors"))]
        {
            println!("Log #{} ({})", logs.len() - index, &log.id.to_string()[..8]);
            println!("Time: {}", timestamp_str);
            println!("Operation: {}", operation_str);
            println!("User: {}", log.user);
            println!("Paths:");
            for path in &log.paths {
                println!("  • {}", path.display());
            }
            if !log.file_ids.is_empty() {
                println!("File IDs:");
                for file_id in &log.file_ids {
                    println!("  • {}", &file_id.to_string()[..8]);
                }
            }
            println!("Result: {}", log.result_display());
            if let Some(context) = &log.context {
                println!("Context: {}", context);
            }
        }
    }
    
    if verbose {
        #[cfg(feature = "colors")]
        println!("\n📈 Total: {} operations", logs.len().to_string().bold().green());
        #[cfg(not(feature = "colors"))]
        println!("\nTotal: {} operations", logs.len());
    }
    
    Ok(())
}

/// Display logs in summary format
fn display_summary_logs(logs: &[crate::domain::operation_log::OperationLog], verbose: bool) -> Result<()> {
    #[cfg(feature = "colors")]
    println!("{}", "Recent Operations:".bold().underline());
    #[cfg(not(feature = "colors"))]
    println!("Recent Operations:");
    println!();
    
    for log in logs.iter().take(20) { // Show latest 20 in summary
        let time_ago = format_relative_time(log.timestamp);
        let description = log.description();
        let result_icon = match log.result {
            crate::domain::operation_log::OperationResult::Success => "✅",
            crate::domain::operation_log::OperationResult::Failed(_) => "❌",
            crate::domain::operation_log::OperationResult::Cancelled => "⚠️",
        };
        
        #[cfg(feature = "colors")]
        println!("{} {} {} {} {}", 
            result_icon,
            time_ago.dimmed(),
            format!("{:?}", log.operation).cyan(),
            description.white(),
            format!("({})", log.user).green().italic()
        );
        
        #[cfg(not(feature = "colors"))]
        println!("{} {} {:?} {} ({})", 
            result_icon,
            time_ago,
            log.operation,
            description,
            log.user
        );
    }
    
    if logs.len() > 20 {
        #[cfg(feature = "colors")]
        println!("\n{} Use --detailed flag to see all {} operations", 
            "💡".yellow(),
            logs.len().to_string().bold()
        );
        #[cfg(not(feature = "colors"))]
        println!("\nUse --detailed flag to see all {} operations", logs.len());
    }
    
    if verbose {
        println!();
        display_operation_stats(logs)?;
    }
    
    Ok(())
}

/// Display operation statistics
fn display_operation_stats(logs: &[crate::domain::operation_log::OperationLog]) -> Result<()> {
    use std::collections::HashMap;
    
    let mut operation_counts = HashMap::new();
    let mut result_counts = HashMap::new();
    
    for log in logs {
        *operation_counts.entry(format!("{:?}", log.operation)).or_insert(0) += 1;
        let result_key = match &log.result {
            crate::domain::operation_log::OperationResult::Success => "Success",
            crate::domain::operation_log::OperationResult::Failed(_) => "Failed",
            crate::domain::operation_log::OperationResult::Cancelled => "Cancelled",
        };
        *result_counts.entry(result_key).or_insert(0) += 1;
    }
    
    #[cfg(feature = "colors")]
    {
        println!("{}", "Operation Summary:".bold().underline());
        for (op, count) in operation_counts {
            println!("  {}: {}", op.cyan(), count.to_string().yellow());
        }
        println!();
        println!("{}", "Result Summary:".bold().underline());
        for (result, count) in result_counts {
            let color_count = match result {
                "Success" => count.to_string().green(),
                "Failed" => count.to_string().red(),
                "Cancelled" => count.to_string().yellow(),
                _ => count.to_string().white(),
            };
            println!("  {}: {}", result, color_count);
        }
    }
    
    #[cfg(not(feature = "colors"))]
    {
        println!("Operation Summary:");
        for (op, count) in operation_counts {
            println!("  {}: {}", op, count);
        }
        println!();
        println!("Result Summary:");
        for (result, count) in result_counts {
            println!("  {}: {}", result, count);
        }
    }
    
    Ok(())
}

/// Parse since date string into DateTime<Utc>
fn parse_since_date(since_str: &str) -> Result<DateTime<Utc>> {
    match since_str.to_lowercase().as_str() {
        "today" => {
            let now = Utc::now();
            Ok(now.date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc())
        }
        "yesterday" => {
            let yesterday = Utc::now() - chrono::Duration::days(1);
            Ok(yesterday.date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc())
        }
        "week" | "this-week" => {
            let week_ago = Utc::now() - chrono::Duration::weeks(1);
            Ok(week_ago)
        }
        "month" | "this-month" => {
            let month_ago = Utc::now() - chrono::Duration::days(30);
            Ok(month_ago)
        }
        _ => {
            // Try to parse as ISO date
            if let Ok(date) = chrono::DateTime::parse_from_rfc3339(since_str) {
                Ok(date.with_timezone(&Utc))
            } else if let Ok(date) = chrono::NaiveDateTime::parse_from_str(since_str, "%Y-%m-%d %H:%M:%S") {
                Ok(date.and_utc())
            } else if let Ok(date) = chrono::NaiveDate::parse_from_str(since_str, "%Y-%m-%d") {
                Ok(date.and_hms_opt(0, 0, 0).unwrap().and_utc())
            } else {
                anyhow::bail!("Invalid date format: {}", since_str);
            }
        }
    }
}

/// Convert CLI OperationType to domain OperationType
fn convert_operation_type(cli_op: OperationType) -> crate::domain::operation_log::OperationType {
    match cli_op {
        OperationType::Delete => crate::domain::operation_log::OperationType::Delete,
        OperationType::Restore => crate::domain::operation_log::OperationType::Restore,
        OperationType::Purge => crate::domain::operation_log::OperationType::Purge,
        OperationType::Status => crate::domain::operation_log::OperationType::Status,
        OperationType::Protect => crate::domain::operation_log::OperationType::Protect,
        OperationType::Doctor => crate::domain::operation_log::OperationType::Doctor,
    }
}

/// Format relative time (e.g., "2 hours ago", "3 days ago")
fn format_relative_time(datetime: DateTime<Utc>) -> String {
    let now = Utc::now();
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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Datelike;
    
    #[test]
    fn test_parse_since_date() {
        // Test special keywords
        let today = parse_since_date("today").unwrap();
        let now = Utc::now();
        assert_eq!(today.date_naive(), now.date_naive());
        
        let yesterday = parse_since_date("yesterday").unwrap();
        let expected = (now - chrono::Duration::days(1)).date_naive();
        assert_eq!(yesterday.date_naive(), expected);
        
        // Test ISO date
        let iso_date = parse_since_date("2024-01-01T00:00:00Z").unwrap();
        assert_eq!(iso_date.year(), 2024);
        assert_eq!(iso_date.month(), 1);
        assert_eq!(iso_date.day(), 1);
        
        // Test simple date
        let simple_date = parse_since_date("2024-01-01").unwrap();
        assert_eq!(simple_date.year(), 2024);
        assert_eq!(simple_date.month(), 1);
        assert_eq!(simple_date.day(), 1);
    }
    
    #[test]
    fn test_convert_operation_type() {
        assert_eq!(
            convert_operation_type(OperationType::Delete),
            crate::domain::operation_log::OperationType::Delete
        );
        assert_eq!(
            convert_operation_type(OperationType::Restore),
            crate::domain::operation_log::OperationType::Restore
        );
        assert_eq!(
            convert_operation_type(OperationType::Purge),
            crate::domain::operation_log::OperationType::Purge
        );
    }
    
    #[test]
    fn test_format_relative_time() {
        let now = Utc::now();
        
        // Test recent time
        let recent = now - chrono::Duration::seconds(30);
        assert_eq!(format_relative_time(recent), "just now");
        
        // Test minutes ago
        let minutes_ago = now - chrono::Duration::minutes(5);
        assert_eq!(format_relative_time(minutes_ago), "5 minutes ago");
        
        // Test hours ago
        let hours_ago = now - chrono::Duration::hours(2);
        assert_eq!(format_relative_time(hours_ago), "2 hours ago");
        
        // Test days ago
        let days_ago = now - chrono::Duration::days(3);
        assert_eq!(format_relative_time(days_ago), "3 days ago");
    }
}