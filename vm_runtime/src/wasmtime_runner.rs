use std::time::Instant;
use wasmtime::{Config, Engine, Linker, Module, Store};
use wasmtime_wasi::preview1::{self, WasiP1Ctx};
use wasmtime_wasi::WasiCtxBuilder;

use crate::{error::RuntimeError, Manifest, Result, RunMetrics, RunResult, Runner};

pub struct WasmtimeRunner;

impl WasmtimeRunner {
    pub fn new() -> Self {
        Self
    }
}

impl Default for WasmtimeRunner {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl Runner for WasmtimeRunner {
    async fn run(&self, manifest: Manifest) -> Result<RunResult> {
        let start = Instant::now();

        if manifest.image_cid.is_empty() {
            return Err(RuntimeError::InvalidManifest("empty image_cid".into()));
        }
        if manifest.entrypoint.is_empty() {
            return Err(RuntimeError::InvalidManifest("empty entrypoint".into()));
        }
        if manifest.max_duration_secs == 0 || manifest.max_duration_secs > 3600 {
            return Err(RuntimeError::InvalidManifest(
                "max_duration out of range (1-3600)".into(),
            ));
        }

        let wasm_bytes = load_wasm_image(&manifest.image_cid).await?;

        let mut config = Config::new();
        config.consume_fuel(true);
        let engine =
            Engine::new(&config).map_err(|e| RuntimeError::Execution(e.to_string()))?;

        let module = Module::new(&engine, &wasm_bytes)
            .map_err(|e| RuntimeError::Execution(format!("invalid wasm: {e}")))?;

        let wasi_ctx = WasiCtxBuilder::new()
            .inherit_stdio()
            .args(&["wasm", &manifest.entrypoint])
            .build_p1();

        let mut store = Store::new(&engine, wasi_ctx);
        store
            .set_fuel(manifest.max_duration_secs * 100_000)
            .map_err(|e| RuntimeError::Execution(e.to_string()))?;

        let mut linker: Linker<WasiP1Ctx> = Linker::new(&engine);
        preview1::add_to_linker_sync(&mut linker, |t| t)
            .map_err(|e| RuntimeError::Execution(e.to_string()))?;

        let instance = linker
            .instantiate(&mut store, &module)
            .map_err(|e| RuntimeError::Execution(e.to_string()))?;

        let func = instance
            .get_typed_func::<(), ()>(&mut store, &manifest.entrypoint)
            .or_else(|_| instance.get_typed_func::<(), ()>(&mut store, "_start"))
            .map_err(|e| RuntimeError::Execution(format!("entrypoint not found: {e}")))?;

        let blocking_task = tokio::task::spawn_blocking(move || {
            func.call(&mut store, ()).map_err(|e| RuntimeError::Execution(e.to_string()))
        });

        let timeout = std::time::Duration::from_secs(manifest.max_duration_secs);
        let result = tokio::time::timeout(timeout, blocking_task).await;

        let elapsed = start.elapsed();

        match result {
            Ok(Ok(Ok(()))) => Ok(RunResult {
                stdout: String::new(),
                stderr: String::new(),
                exit_code: 0,
                metrics: RunMetrics {
                    cpu_usage_percent: 0.0,
                    memory_usage_mb: 0.0,
                    duration_secs: elapsed.as_secs_f64(),
                },
            }),
            Ok(Ok(Err(e))) => Err(e),
            Ok(Err(e)) => Err(RuntimeError::Execution(format!("task failed: {e}"))),
            Err(_) => Err(RuntimeError::Timeout(manifest.max_duration_secs)),
        }
    }
}

async fn load_wasm_image(cid: &str) -> std::result::Result<Vec<u8>, RuntimeError> {
    let cache_path = format!("/tmp/flovenet/wasm_cache/{cid}");
    if let Ok(data) = tokio::fs::read(&cache_path).await {
        return Ok(data);
    }

    let local_path = format!("wasm_images/target/wasm32-wasi/release/{cid}.wasm");
    tokio::fs::read(&local_path)
        .await
        .map_err(|e| RuntimeError::Execution(format!("cannot load wasm image {cid}: {e}")))
}
