use crate::domain::operation_log::{OperationLog, OperationType};
use anyhow::Result;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, OnceLock};

/// Interface for operation logging
pub trait OperationLoggerInterface: Send + Sync {
    fn log(&self, entry: OperationLog) -> Result<()>;
    fn get_logs(&self, limit: Option<usize>) -> Result<Vec<OperationLog>>;
    fn get_logs_filtered(
        &self,
        operation_type: Option<OperationType>,
        since: Option<chrono::DateTime<chrono::Utc>>,
        limit: Option<usize>,
    ) -> Result<Vec<OperationLog>>;
}

/// File-based operation logger implementation
pub struct FileOperationLogger {
    log_path: PathBuf,
}

impl FileOperationLogger {
    pub fn new(log_path: PathBuf) -> Self {
        Self { log_path }
    }
}

impl OperationLoggerInterface for FileOperationLogger {
    fn log(&self, entry: OperationLog) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = self.log_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Serialize to JSON Lines format
        let json_line = serde_json::to_string(&entry)?;
        
        // Append to log file
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_path)?;
        
        writeln!(file, "{}", json_line)?;
        file.sync_all()?;
        
        Ok(())
    }

    fn get_logs(&self, limit: Option<usize>) -> Result<Vec<OperationLog>> {
        self.get_logs_filtered(None, None, limit)
    }

    fn get_logs_filtered(
        &self,
        operation_type: Option<OperationType>,
        since: Option<chrono::DateTime<chrono::Utc>>,
        limit: Option<usize>,
    ) -> Result<Vec<OperationLog>> {
        if !self.log_path.exists() {
            return Ok(Vec::new());
        }

        let contents = std::fs::read_to_string(&self.log_path)?;
        let mut logs = Vec::new();

        for line in contents.lines() {
            if line.trim().is_empty() {
                continue;
            }

            match serde_json::from_str::<OperationLog>(line) {
                Ok(log) => {
                    // Apply filters
                    if log.matches_filter(operation_type.as_ref(), since.as_ref()) {
                        logs.push(log);
                    }
                }
                Err(e) => {
                    eprintln!("Warning: Failed to parse log line: {} ({})", line, e);
                }
            }
        }

        // Sort by timestamp (newest first)
        logs.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        // Apply limit if specified
        if let Some(limit) = limit {
            logs.truncate(limit);
        }

        Ok(logs)
    }
}

/// Global operation logger instance
static OPERATION_LOGGER: OnceLock<Arc<dyn OperationLoggerInterface>> = OnceLock::new();

/// Initialize the global operation logger
pub fn init_operation_logger(log_path: PathBuf) -> Result<()> {
    let logger = Arc::new(FileOperationLogger::new(log_path));
    OPERATION_LOGGER.set(logger)
        .map_err(|_| anyhow::anyhow!("Operation logger already initialized"))?;
    Ok(())
}

/// Get the global operation logger
pub fn get_operation_logger() -> Result<Arc<dyn OperationLoggerInterface>> {
    OPERATION_LOGGER.get()
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("Operation logger not initialized"))
}

/// Log an operation using the global logger
pub fn log_operation(entry: OperationLog) -> Result<()> {
    // Try to get the logger, but don't fail if it's not initialized
    if let Ok(logger) = get_operation_logger() {
        logger.log(entry)?;
    } else {
        // Initialize with default path if not already initialized
        let config = crate::domain::Config::load()?;
        let log_path = config.trash_path.join("operations.jsonl");
        init_operation_logger(log_path)?;
        
        // Try again
        if let Ok(logger) = get_operation_logger() {
            logger.log(entry)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::operation_log::{OperationType, OperationResult};
    use chrono::Utc;
    use std::path::PathBuf;
    use tempfile::TempDir;

    #[test]
    fn test_file_operation_logger() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("operations.jsonl");
        let logger = FileOperationLogger::new(log_path.clone());

        // Create a test log entry
        let entry = OperationLog::new(
            OperationType::Delete,
            vec![PathBuf::from("/test/file.txt")],
            OperationResult::Success,
        );

        // Log the entry
        logger.log(entry.clone()).unwrap();

        // Verify the log was written
        assert!(log_path.exists());

        // Read back the logs
        let logs = logger.get_logs(None).unwrap();
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].operation, entry.operation);
        assert_eq!(logs[0].paths, entry.paths);
    }

    #[test]
    fn test_operation_logger_filtering() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("operations.jsonl");
        let logger = FileOperationLogger::new(log_path);

        // Create test entries
        let delete_entry = OperationLog::new(
            OperationType::Delete,
            vec![PathBuf::from("/test/file1.txt")],
            OperationResult::Success,
        );

        let restore_entry = OperationLog::new(
            OperationType::Restore,
            vec![PathBuf::from("/test/file2.txt")],
            OperationResult::Success,
        );

        // Log the entries
        logger.log(delete_entry).unwrap();
        logger.log(restore_entry).unwrap();

        // Test filtering by operation type
        let delete_logs = logger.get_logs_filtered(Some(OperationType::Delete), None, None).unwrap();
        assert_eq!(delete_logs.len(), 1);
        assert_eq!(delete_logs[0].operation, OperationType::Delete);

        let restore_logs = logger.get_logs_filtered(Some(OperationType::Restore), None, None).unwrap();
        assert_eq!(restore_logs.len(), 1);
        assert_eq!(restore_logs[0].operation, OperationType::Restore);

        // Test getting all logs
        let all_logs = logger.get_logs(None).unwrap();
        assert_eq!(all_logs.len(), 2);
    }
}