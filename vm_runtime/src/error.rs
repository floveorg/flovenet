use thiserror::Error;

#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error("manifest validation error: {0}")]
    InvalidManifest(String),

    #[error("execution error: {0}")]
    Execution(String),

    #[error("timeout: exceeded {0}s")]
    Timeout(u64),

    #[error("resource limit exceeded")]
    ResourceLimit,
}
