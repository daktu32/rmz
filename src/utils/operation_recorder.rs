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
