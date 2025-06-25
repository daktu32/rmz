use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

/// Types of operations that can be logged
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OperationType {
    Delete,
    Restore,
    Purge,
    List,
    Config,
    Status,
    Protect,
    Doctor,
}

/// Result of an operation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OperationResult {
    Success,
    Failed(String),
    Cancelled,
}

/// Log entry for operations performed by rmz
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationLog {
    /// Unique identifier for this log entry
    pub id: Uuid,

    /// Timestamp when the operation occurred
    pub timestamp: DateTime<Utc>,

    /// Type of operation
    pub operation: OperationType,

    /// Path(s) affected by the operation
    pub paths: Vec<PathBuf>,

    /// Result of the operation
    pub result: OperationResult,

    /// User who performed the operation
    pub user: String,

    /// Additional context or arguments
    pub context: Option<String>,

    /// File IDs involved (for restore/purge operations)
    pub file_ids: Vec<Uuid>,
}

impl OperationLog {
    /// Create a new operation log entry
    pub fn new(operation: OperationType, paths: Vec<PathBuf>, result: OperationResult) -> Self {
        let user = std::env::var("USER")
            .or_else(|_| std::env::var("USERNAME"))
            .unwrap_or_else(|_| "unknown".to_string());

        Self {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            operation,
            paths,
            result,
            user,
            context: None,
            file_ids: Vec::new(),
        }
    }

    /// Add context information to the log entry
    pub fn with_context(mut self, context: String) -> Self {
        self.context = Some(context);
        self
    }

    /// Add file IDs to the log entry
    pub fn with_file_ids(mut self, file_ids: Vec<Uuid>) -> Self {
        self.file_ids = file_ids;
        self
    }

    /// Check if this log entry matches a filter
    pub fn matches_filter(
        &self,
        operation_type: Option<&OperationType>,
        since: Option<&DateTime<Utc>>,
    ) -> bool {
        if let Some(op_type) = operation_type {
            if &self.operation != op_type {
                return false;
            }
        }

        if let Some(since_time) = since {
            if self.timestamp < *since_time {
                return false;
            }
        }

        true
    }

    /// Format the operation result for display
    pub fn result_display(&self) -> String {
        match &self.result {
            OperationResult::Success => "✅ Success".to_string(),
            OperationResult::Failed(error) => format!("❌ Failed: {}", error),
            OperationResult::Cancelled => "⚠️ Cancelled".to_string(),
        }
    }

    /// Get a human-readable description of the operation
    pub fn description(&self) -> String {
        let operation_name = match self.operation {
            OperationType::Delete => "Delete",
            OperationType::Restore => "Restore",
            OperationType::Purge => "Purge",
            OperationType::List => "List",
            OperationType::Config => "Config",
            OperationType::Status => "Status",
            OperationType::Protect => "Protect",
            OperationType::Doctor => "Doctor",
        };

        let paths_str = if self.paths.len() == 1 {
            self.paths[0].to_string_lossy().to_string()
        } else if self.paths.len() > 1 {
            format!("{} files", self.paths.len())
        } else if !self.file_ids.is_empty() {
            format!("{} items", self.file_ids.len())
        } else {
            "unknown".to_string()
        };

        format!("{} {}", operation_name, paths_str)
    }
}

/// Manager for operation logs
pub struct OperationLogger {
    log_file_path: PathBuf,
}

impl OperationLogger {
    /// Create a new operation logger
    pub fn new(log_file_path: PathBuf) -> Self {
        Self { log_file_path }
    }

    /// Log an operation
    pub fn log(&self, entry: OperationLog) -> anyhow::Result<()> {
        // Ensure log directory exists
        if let Some(parent) = self.log_file_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Append to log file as JSON lines
        let json_line = serde_json::to_string(&entry)? + "\n";

        use std::fs::OpenOptions;
        use std::io::Write;

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_file_path)?;

        file.write_all(json_line.as_bytes())?;
        file.flush()?;

        Ok(())
    }

    /// Read all log entries
    pub fn read_logs(&self) -> anyhow::Result<Vec<OperationLog>> {
        if !self.log_file_path.exists() {
            return Ok(Vec::new());
        }

        let content = std::fs::read_to_string(&self.log_file_path)?;
        let mut logs = Vec::new();

        for line in content.lines() {
            if !line.trim().is_empty() {
                match serde_json::from_str::<OperationLog>(line) {
                    Ok(log) => logs.push(log),
                    Err(e) => {
                        eprintln!("Warning: Failed to parse log line: {}", e);
                        // Continue parsing other lines
                    }
                }
            }
        }

        // Sort by timestamp (newest first)
        logs.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        Ok(logs)
    }

    /// Read logs with filters
    pub fn read_filtered_logs(
        &self,
        operation_type: Option<OperationType>,
        since: Option<DateTime<Utc>>,
        limit: Option<usize>,
    ) -> anyhow::Result<Vec<OperationLog>> {
        let all_logs = self.read_logs()?;

        let filtered: Vec<OperationLog> = all_logs
            .into_iter()
            .filter(|log| log.matches_filter(operation_type.as_ref(), since.as_ref()))
            .take(limit.unwrap_or(usize::MAX))
            .collect();

        Ok(filtered)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_operation_log_creation() {
        let paths = vec![PathBuf::from("/test/file.txt")];
        let log = OperationLog::new(
            OperationType::Delete,
            paths.clone(),
            OperationResult::Success,
        );

        assert_eq!(log.operation, OperationType::Delete);
        assert_eq!(log.paths, paths);
        assert_eq!(log.result, OperationResult::Success);
        assert!(!log.id.is_nil());
        assert!(log.timestamp <= Utc::now());
    }

    #[test]
    fn test_operation_log_with_context() {
        let log = OperationLog::new(OperationType::Delete, vec![], OperationResult::Success)
            .with_context("Test context".to_string());

        assert_eq!(log.context, Some("Test context".to_string()));
    }

    #[test]
    fn test_matches_filter() {
        let log = OperationLog::new(OperationType::Delete, vec![], OperationResult::Success);

        assert!(log.matches_filter(Some(&OperationType::Delete), None));
        assert!(!log.matches_filter(Some(&OperationType::Restore), None));
        assert!(log.matches_filter(None, None));

        let past_time = Utc::now() - chrono::Duration::hours(1);
        assert!(log.matches_filter(None, Some(&past_time)));

        let future_time = Utc::now() + chrono::Duration::hours(1);
        assert!(!log.matches_filter(None, Some(&future_time)));
    }

    #[test]
    fn test_operation_logger() {
        let temp_file = NamedTempFile::new().unwrap();
        let logger = OperationLogger::new(temp_file.path().to_path_buf());

        let log1 = OperationLog::new(
            OperationType::Delete,
            vec![PathBuf::from("/test1.txt")],
            OperationResult::Success,
        );

        let log2 = OperationLog::new(
            OperationType::Restore,
            vec![PathBuf::from("/test2.txt")],
            OperationResult::Failed("Test error".to_string()),
        );

        logger.log(log1.clone()).unwrap();
        logger.log(log2.clone()).unwrap();

        let logs = logger.read_logs().unwrap();
        assert_eq!(logs.len(), 2);

        // Should be sorted by timestamp (newest first)
        assert_eq!(logs[0].operation, OperationType::Restore);
        assert_eq!(logs[1].operation, OperationType::Delete);

        // Test filtering
        let delete_logs = logger
            .read_filtered_logs(Some(OperationType::Delete), None, None)
            .unwrap();
        assert_eq!(delete_logs.len(), 1);
        assert_eq!(delete_logs[0].operation, OperationType::Delete);
    }
}
