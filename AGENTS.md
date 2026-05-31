# Flovenet — Guía para Agentes AI

## Cómo usar este repo

```bash
# Orden CI: fmt → clippy → build → test
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo build --release
cargo test

# Solo un crate
cargo test -p crypto
cargo build -p daemon

# App web
cd web-dashboard && npm install && npm run dev  # Vite en :3000, proxyea /graphql → :8080
```

## CLI

Un solo binario (`daemon`) con subcomandos definidos en `cli/src/lib.rs` (clap derive):

| Comando | Uso |
|---------|-----|
| `daemon` | Nodo P2P con `--roles compute,storage` |
| `api-gateway` | GraphQL gateway en `--port 8080` |
| `share` | Muestra recursos locales |
| `run` | Ejecuta WASM localmente |
| `status` | Recursos del nodo |

## Quirks importantes

- **`updateProfile` siempre falla** — el store interno usa key "user" que no existe. No tocar sin rediseñar el auth middleware.
- **No hay auth middleware real** — `createPost` y otras mutaciones funcionan sin token JWT.
- **GraphQL gateway guarda todo en `InMemoryStore`** — se pierde al reiniciar.
- **GPU** solo se detecta via `FLOVENET_GPU_VRAM_GB` y `FLOVENET_GPU_MODEL` (env vars).
- **Sub-red privada**: generar `swarm.key` (32 bytes raw) y pasar `--swarm-key swarm.key`.
- **Android**: `flovenet-core` es `cdylib` + `lib`, compila con `cargo build --target aarch64-linux-android -p flovenet-core --release`, requiere NDK 27+.
- **WASM images** (`wasm_images/feed_ranker`, `wasm_images/moderator`) están excluidas del workspace.
- **Tests de integración Docker**: `tests/docker_integration_test.py` — requiere `docker compose up --build` primero.

## Estructura

- **18 miembros workspace**. El binario `daemon` depende de casi todos los crates internos.
- `cli` es solo definición de CLI (clap). `daemon` es el entrypoint real.
- `flovenet-core` existe para linkeditar desde Android NDK.
- `test_harness`/`test_reporter` son bins de integración independientes.
- Packaging: `Dockerfile` (multi-stage con cargo-chef), `deb-pkg/`, `snap/snapcraft.yaml`.

## Convenciones

- `thiserror` para errores internos, `anyhow` para errores de alto nivel (ej. binario).
- `async_trait` para traits async.
- `use` ordenado: std → externo → interno.
- Serde derive en todos los structs de datos.
- Tests unitarios inline (`#[cfg(test)] mod tests`).
