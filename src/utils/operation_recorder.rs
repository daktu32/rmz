use crate::domain::operation_log::{OperationLog, OperationResult, OperationType};
use crate::infra::operation_logger::OperationLoggerInterface;
use anyhow::Result;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

/// Helper for recording command operations
pub struct OperationRecorder {
    logger: Arc<dyn OperationLoggerInterface>,
    start_time: Instant,
    operation: OperationType,
    paths: Vec<PathBuf>,
}

impl OperationRecorder {
    pub fn new(
        logger: Arc<dyn OperationLoggerInterface>,
        operation: OperationType,
        paths: Vec<PathBuf>,
    ) -> Self {
        Self {
            logger,
            start_time: Instant::now(),
            operation,
            paths,
        }
    }

    /// Finish recording and log the result
    pub fn finish(self, result: Result<()>) -> Result<()> {
        let op_result = match result {
            Ok(_) => OperationResult::Success,
            Err(e) => OperationResult::Failed(e.to_string()),
        };
        let mut log = OperationLog::new(self.operation, self.paths, op_result);
        log = log.with_context(format!(
            "duration_ms={}",
            self.start_time.elapsed().as_millis()
        ));
        self.logger.log_operation(log)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infra::operation_logger::JsonOperationLogger;
    use tempfile::TempDir;

    #[test]
    fn test_operation_recorder_finish_success() {
        let temp = TempDir::new().unwrap();
        let logger = Arc::new(JsonOperationLogger::new(temp.path().to_path_buf()));
        let recorder = OperationRecorder::new(
            logger.clone(),
            OperationType::Delete,
            vec![temp.path().join("file.txt")],
        );
        recorder.finish(Ok(())).unwrap();
        let logs = logger.read_all_logs().unwrap();
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].result, OperationResult::Success);
    }
}
