# Flovenet — Guía para Agentes AI

## Stack Tecnológico

| Componente | Tecnología |
|-----------|------------|
| Lenguaje | Rust edition 2021 |
| Workspace | Cargo workspace (18 members) |
| Networking | libp2p 0.54 (Noise + Yamux + Kademlia + Gossipsub + Identify) |
| WASM | Wasmtime 24 (WASI preview1) |
| Almacenamiento | LocalBackend → IpfsBackend (Kubo) → S3Backend (MinIO/AWS) → HybridBackend |
| API GraphQL | async-graphql 7 + axum 0.7 + WebSocket |
| Criptografía | ed25519-dalek 2, ChaCha20-Poly1305, argon2 0.5 |
| Identidad | Ed25519 keys + keystore cifrado + PeerId (CIDv1) |
| Frontend | React 19 + Vite 6 + urql (GraphQL client) |
| Mobile | Android NDK (JNI) via flovenet-core (cdylib) |

## Estructura del Workspace

### Crates funcionales (16)
- **flovenet-core** — Core multiplataforma con JNI bridge para Android
- **daemon** — Binario principal, networking, metrics (prometheus)
- **cli** — CLI via clap derive (daemon, api-gateway, share, run, status)
- **resource_manager** — Detección de CPU/RAM/GPU/disco, NodeDescriptor, NodeRole
- **vm_runtime** — Trait Runner + WasmtimeRunner con fuel metering
- **market_protocol** — libp2p request-response para ofertas de trabajo
- **p2p_cache** — Block exchange (BitSwap-lite) via libp2p
- **reputation_engine** — CRDT eventualmente consistente (LWW)
- **ipfs_layer** — IpfsBackend usando Kubo HTTP API
- **storage** — StorageBackend trait + Local + S3 + Hybrid (tiered)
- **crypto** — EncryptedBlob (ChaCha20-Poly1305), SignedEnvelope (Ed25519), argon2id KDF
- **identity** — KeyStore cifrado en disco, PeerId from pubkey
- **scheduler** — LocalScheduler + SlotMatching + CandidateRanking con reputación
- **trust_graph** — Web of Trust con transitividad de 2º orden
- **social_protocol** — Post, Profile, Follow, Feed (modelos de datos)
- **graphql_api** — async-graphql schema (Query, Mutation, Subscription) + auth JWT

### Crates de test (2)
- **test_harness** — Escenarios de integración
- **test_reporter** — Reporte de resultados

## Convenciones de Código

- `use` ordenado: std → externo → interno
- `async_trait` para traits async
- `thiserror` para errores internos, `anyhow` para errores de alto nivel
- Serde derive en todos los structs de datos
- Tests unitarios inline en cada módulo (`#[cfg(test)] mod tests`)
- Tests de integración en `tests/`

## CI/CD

- GitHub Actions en `.github/workflows/ci.yml`
- Docker: `Dockerfile` + `docker-compose.yml` (3 nodos + gateway)
- Debian packaging en `deb-pkg/`
- Snap package en `snap/snapcraft.yaml`
- Android build script en `scripts/build-android.sh`

## Comandos Útiles

```bash
cargo build --release
cargo test
cargo clippy
cargo fmt
cargo run --release -- daemon --port 0 --api-port 9090 --roles compute,storage
cargo run --release -- api-gateway --port 8080
```
