use crate::domain::OperationLog;
use anyhow::Result;
use chrono::{Date, Utc};
use std::path::PathBuf;

/// Interface for logging operations
pub trait OperationLoggerInterface: Send + Sync {
    fn log_operation(&self, operation: OperationLog) -> Result<()>;
    fn read_all_logs(&self) -> Result<Vec<OperationLog>>;
}

/// JSON file based operation logger
pub struct JsonOperationLogger {
    log_dir: PathBuf,
}

impl JsonOperationLogger {
    pub fn new(log_dir: PathBuf) -> Self {
        Self { log_dir }
    }

    fn get_log_file_path(&self, date: Date<Utc>) -> PathBuf {
        self.log_dir
            .join(format!("operations-{}.jsonl", date.format("%Y-%m-%d")))
    }
}

impl OperationLoggerInterface for JsonOperationLogger {
    fn log_operation(&self, operation: OperationLog) -> Result<()> {
        std::fs::create_dir_all(&self.log_dir)?;
        let log_file = self.get_log_file_path(operation.timestamp.date());
        let log_line = serde_json::to_string(&operation)?;
        use std::fs::OpenOptions;
        use std::io::Write;
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_file)?;
        writeln!(file, "{}", log_line)?;
        file.sync_all()?;
        Ok(())
    }

    fn read_all_logs(&self) -> Result<Vec<OperationLog>> {
        let mut logs = Vec::new();
        if !self.log_dir.exists() {
            return Ok(logs);
        }
        for entry in std::fs::read_dir(&self.log_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_file() {
                let content = std::fs::read_to_string(entry.path())?;
                for line in content.lines() {
                    if let Ok(log) = serde_json::from_str::<OperationLog>(line) {
                        logs.push(log);
                    }
                }
            }
        }
        logs.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        Ok(logs)
    }
}
