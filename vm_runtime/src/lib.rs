pub mod error;
pub mod wasmtime_runner;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub image_cid: String,
    pub entrypoint: String,
    pub args: Vec<String>,
    pub max_duration_secs: u64,
    pub slots_required: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub metrics: RunMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunMetrics {
    pub cpu_usage_percent: f64,
    pub memory_usage_mb: f64,
    pub duration_secs: f64,
}

#[async_trait]
pub trait Runner: Send + Sync {
    async fn run(&self, manifest: Manifest) -> Result<RunResult>;
}

pub type Result<T> = std::result::Result<T, error::RuntimeError>;
