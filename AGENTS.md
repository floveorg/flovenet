# Flovenet — Guía para Agentes AI

## Comandos

```bash
# Orden CI: fmt → clippy → build → test
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo build --all-targets
cargo test

# Foco en un crate
cargo test -p graphql_api
cargo build -p daemon

# App web (Vite :3000, proxyea /graphql → :8080)
cd web-dashboard && npm install && npm run dev
```

## CLI

Binario único (`daemon/src/main.rs`). Subcomandos en `cli/src/lib.rs` (clap derive):

| Comando | Uso |
|---------|-----|
| `daemon` | Nodo P2P con `--roles compute,storage`, `--api-port 9090`, `--swarm-key` |
| `api-gateway` | GraphQL gateway en `--port 8080` |
| `share` | Muestra recursos locales (CPU/RAM/disco/GPU) |
| `run` | Ejecuta WASM localmente (`--manifest _start --image path.wasm`) |
| `status` | Recursos del nodo |

## Quirks

- **CI**: `check` corre primero (fmt→clippy→build→test). `deb-package` y `docker` dependen de él. **`docker` job nunca se ejecuta** — su condición `github.ref == 'refs/heads/main'` nunca se cumple porque la rama es `master`.
- **Sin auth middleware real** — `createPost`, `follow`, `updateProfile` hardcodean `user_id = "user"`. `updateProfile` funciona si antes se llamó a `register` (guarda perfil bajo key `"user"`), pero falla tras reinicio (InMemoryStore volátil).
- **GraphQL** usa `InMemoryStore` (todo en HashMap, se pierde al reiniciar). Playground en `http://localhost:8080/graphql`.
- **GPU**: `FLOVENET_GPU_VRAM_GB` y `FLOVENET_GPU_MODEL` se chequean primero y saltan detección real. Sin esas vars, hay detección vía `/proc/driver/nvidia/gpus/` (Linux) o `nvidia-smi`/`wmic` (Windows). macOS → None.
- **Env vars útiles**: `FLOVENET_DATA_DIR`, `FLOVENET_CACHE_DIR`, `FLOVENET_PLATFORM=android` (para desambiguar Android en Linux).
- **Sub-red privada**: `dd if=/dev/urandom bs=32 count=1 of=swarm.key && daemon --swarm-key swarm.key`.
- **Android**: `flovenet-core` es `cdylib + lib`. Compilar: `cargo build --target aarch64-linux-android -p flovenet-core --release` (requiere NDK 27+; linker en `.cargo/config.toml`). Script completo: `scripts/build-android.sh`.
- **WASM images** (`wasm_images/feed_ranker`, `wasm_images/moderator`) fuera del workspace.
- **Tests Docker**: `tests/docker_integration_test.py` — requiere `docker compose up --build` y necesita `sudo` para `docker exec`.
- **`cargo-audit`** y **`cargo-deny`** tienen configs separadas (`.cargo/audit.toml`, `deny.toml`). Ignoran advisories de dependencias transitivas (libp2p, wasmtime). `cargo-deny` no acepta expresiones SPDX compuestas — listar licencias individuales.
- **Rama remota**: `master`, no `main`. CI configurado para `master`.
- **Windows cross-build**: compila `daemon` + `flovenet-core` para `x86_64-pc-windows-msvc`. Pueden aparecer falsos positivos de clippy (ej. `std::path::Path` no usado en Windows).
- **KeepAliveTimeout en test_harness**: `libp2p-swarm` por defecto usa `idle_connection_timeout = Duration::ZERO`. Con el handler de gossipsub en estado `Enabled` pero `in_mesh = false`, `connection_keep_alive()` devuelve `false`, causando `KeepAliveTimeout` inmediato. Solución: `Swarm::Config::with_tokio_executor().with_idle_connection_timeout(Duration::from_secs(30))` da tiempo a que llegue el `NotifyHandler` con el `Subscribe` antes de que el timeout mate la conexión.
- **test_harness**: Siempre hacer poll a TODOS los nodos concurrentemente, no solo al receptor. Un mensaje publicado en el emisor no se envía hasta que su swarm es polled.
- **test_harness`: `connect_all` espera a que `all_peers()` tenga `n-1` peers; NO debe subscribir topics de prueba porque eso interfiere con los topics de cada scenario.

## Estructura

- **18 miembros workspace** (16 funcionales + 2 test). El binario `daemon` depende de casi todos.
- `daemon/src/main.rs` es el entrypoint real; `cli` es solo definición de CLI.
- `flovenet-core` existe para linkeditar desde Android NDK (incluye `crate-type = ["lib", "cdylib"]` + dependencia `jni` bajo `target_os = "android"`).
- `test_harness`/`test_reporter` son bins independientes.
- Packaging: `Dockerfile` (multi-stage con cargo-chef), `deb-pkg/`, `snap/snapcraft.yaml`.
- `/metrics` y `/health` en `api_port` del daemon (axum + prometheus).

## Convenciones

- `thiserror` para errores internos, `anyhow` para errores de alto nivel.
- `async_trait` para traits async.
- Serde derive en todos los structs de datos.
- Tests unitarios inline (`#[cfg(test)] mod tests`).
