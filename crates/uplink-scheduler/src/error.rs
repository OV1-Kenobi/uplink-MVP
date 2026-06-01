//! Scheduler error types.
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SchedulerError {
    #[error("stream not found: {0}")]
    StreamNotFound(String),
    #[error("storage error: {0}")]
    Storage(String),
    #[error("policy violation: {0}")]
    PolicyViolation(String),
}
