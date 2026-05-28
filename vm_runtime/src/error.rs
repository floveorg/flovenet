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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display_invalid_manifest() {
        let e = RuntimeError::InvalidManifest("empty image_cid".into());
        assert_eq!(e.to_string(), "manifest validation error: empty image_cid");
    }

    #[test]
    fn test_error_display_execution() {
        let e = RuntimeError::Execution("wasm error".into());
        assert_eq!(e.to_string(), "execution error: wasm error");
    }

    #[test]
    fn test_error_display_timeout() {
        let e = RuntimeError::Timeout(60);
        assert_eq!(e.to_string(), "timeout: exceeded 60s");
    }

    #[test]
    fn test_error_display_resource_limit() {
        let e = RuntimeError::ResourceLimit;
        assert_eq!(e.to_string(), "resource limit exceeded");
    }

    #[test]
    fn test_error_debug() {
        let e = RuntimeError::Execution("test".into());
        assert!(format!("{e:?}").contains("Execution"));
    }
}
