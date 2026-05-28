# Flovenet — Guía para Agentes AI

> Documento de orientación para agentes (humanos o IA) que vayan a
> trabajar en este repo. Se mantiene **sincronizado con `README.md`**
> y `plan-crossplatform.md`. Si actualizas uno, actualiza los tres.

## Stack tecnológico (estado real)

| Componente | Tecnología | Nota |
|-----------|------------|------|
| Lenguaje | Rust edition 2021 (toolchain 1.85+) | |
| Workspace | Cargo workspace, 16 crates funcionales + 2 de test | |
| Networking | libp2p 0.54: TCP + Noise + Yamux + Kademlia + Gossipsub + Identify + Ping + **mDNS** | sin QUIC, sin WebSocket-transport, sin AutoNAT/relay |
| WASM | Wasmtime 24 (WASI preview1) | |
| Almacenamiento | LocalBackend → IpfsBackend (Kubo) → S3Backend (MinIO; **AWS via Basic auth, no SigV4**) → HybridBackend | replicación entre tiers locales del mismo proceso, no cross-node |
| API GraphQL | async-graphql 7 + axum 0.7 | HTTP POST + playground OK; **WS no expuesto en `/graphql`** todavía |
| Criptografía | ed25519-dalek 2, ChaCha20-Poly1305, argon2 0.5 | |
| Identidad | Ed25519 keys + keystore cifrado + PeerId (CIDv1) | |
| Frontend | React 19 + Vite 6 + urql (GraphQL client) | |
| Mobile | Android NDK (JNI) via `flovenet-core` (cdylib) | scaffolding Gradle, **app Kotlin no implementada** |

## Estructura del workspace

### Crates funcionales (16)
- **flovenet-core** — Core multiplataforma con JNI bridge para Android.
- **daemon** — Binario principal: swarm libp2p, market handler, metrics
  (prometheus). El binario único provee subcomandos `daemon`,
  `api-gateway`, `share`, `run`, `status`.
- **cli** — Definición clap derive (Cli + Commands). Flag relevante:
  `--bootstrap-peers <CSV de multiaddrs>` desde Hito 1.
- **resource_manager** — Detección CPU/RAM/GPU/disco, `NodeDescriptor`,
  `NodeRole`, `Platform`.
- **vm_runtime** — Trait `Runner` + `WasmtimeRunner` con fuel metering.
- **market_protocol** — libp2p request-response (`/flovenet/job/1.0.0`)
  para ofertas de trabajo. Hoy NO firma ofertas/respuestas.
- **p2p_cache** — Block exchange (`/flovenet/block/1.0.0`), BitSwap-lite.
- **reputation_engine** — CRDT LWW de scores. Los eventos los emite el
  REQUESTER tras recibir `JobResponse` (Hito 0.3 — 2026-05-28); siguen
  sin firma criptográfica (TODO Hito 6.1).
- **ipfs_layer** — `IpfsBackend` que llama a la HTTP API de Kubo.
- **storage** — Trait `StorageBackend` + Local + IPFS + S3 + Hybrid.
- **crypto** — `EncryptedBlob`, `SignedEnvelope`, KDF argon2id.
  Re-exporta `SigningKey/VerifyingKey/Signer/Verifier/Signature` de
  ed25519-dalek para reutilizar.
- **identity** — `KeyStore` cifrado en disco, `PeerId` desde pubkey.
- **scheduler** — `LocalScheduler` + matching + ranking ponderado
  por reputación.
- **trust_graph** — Web of Trust. **Verificación Ed25519 operativa**
  desde Hito 0.2 (2026-05-28): `TrustEdge` ahora lleva `signer_pubkey`
  y `add_edge` rechaza firmas inválidas. Edges legacy (sin pubkey/sig)
  se aceptan con `warn!` para compat.
- **social_protocol** — Modelos de `Post`, `Profile`, `Follow`, `Feed`
  y `SocialStore` sobre `StorageBackend`. ⚠️ **No conectado al gateway
  todavía**: `graphql_api::AppState` aún usa `InMemoryStore` volátil.
- **graphql_api** — Schema async-graphql + axum + auth JWT. Falta:
  cablear `SocialStore` y servir subscripciones WS.

### Crates de test (2)
- **test_harness** — Escenarios de integración.
- **test_reporter** — Reporte de resultados.

## Convenciones de código

- `use` ordenado: std → externo → interno.
- `async_trait` para traits async.
- `thiserror` para errores internos, `anyhow` para errores de alto nivel.
- `serde` derive en todos los structs de datos. Para campos añadidos
  en versiones posteriores usar `#[serde(default)]` (ejemplo:
  `TrustEdge::signer_pubkey`) para compat con peers antiguos.
- Tests unitarios inline en cada módulo (`#[cfg(test)] mod tests`).
- Tests de integración / harnesses en `tests/` (Python) y `.work/`
  (scripts pragmáticos que requieren binario release compilado).

## CI/CD

- GitHub Actions en `.github/workflows/ci.yml` (check + cross-build
  linux/windows + dashboard Node 20 + docker + audit + deny).
- Dockerfile principal: multi-stage cargo-chef → debian:bookworm-slim.
- `Dockerfile.fast`: imagen Ubuntu 24.04 que reusa
  `target/release/daemon` del host. Solo para iteración rápida en dev,
  NO para CI/release.
- Debian packaging en `deb-pkg/`, snap en `snap/snapcraft.yaml`,
  Android script en `scripts/build-android.sh`.

## Comandos útiles

```bash
# Build + tests
cargo build --release
cargo test --workspace --release
cargo clippy --all-targets -- -D warnings
cargo fmt --check

# Lanzar un daemon (puerto libp2p dinámico, api en 9090, dos roles)
cargo run --release --bin daemon -- daemon \
  --port 0 --api-port 9090 --roles compute,storage

# Daemon con peer de bootstrap explícito (multiaddr con o sin /p2p/<id>)
cargo run --release --bin daemon -- daemon \
  --port 0 --api-port 9092 \
  --bootstrap-peers /dns4/node1/tcp/4001

# Gateway GraphQL standalone
cargo run --release --bin daemon -- api-gateway --port 8080

# Tests pragmáticos (requieren target/release/daemon ya compilado)
.work/test-discovery.sh           # 3 daemons locales con mDNS
docker build -f Dockerfile.fast -t flovenet:latest .
.work/test-docker.sh              # compose con 3 nodos + gateway
```

## Política de modificaciones

- **Toda mejora va con su test.** Si añades una feature, deja
  `cargo test --workspace` verde y, si toca la red, también
  `.work/test-docker.sh`.
- **Actualiza la documentación en el mismo commit**: README
  (estado real + fases + limitaciones), AGENTS.md (este fichero),
  y `.work/progress.md` (diario de la iteración).
- Si cierras un punto de la lista "limitaciones conocidas" del README,
  táchalo con `~~…~~` + fecha. No lo borres.
- Si añades dependencia que cambia el wire format de un mensaje
  gossipsub o request-response, **bumpea el protocol string**
  (`/flovenet/<feature>/X.Y`) y documenta en el commit message.
- Honestidad por defecto: si una mejora deja un gap conocido
  (p.ej. "el rating sigue sin firma"), dilo en README y en el código
  con un comment + TODO al hito correspondiente.

## Estado del plan de resiliencia

Branch principal: `master`. Trabajo en curso en `resilience-plan`.

| Hito | Estado | Commit |
|---|---|---|
| Hito 1.1 — `--bootstrap-peers` CLI | ✅ | `60fbe1a` |
| Hito 1.2 — mDNS auto-discovery | ✅ | `60fbe1a` |
| Hito 0.2 — TrustEdge signature verify | ✅ | `8c2366d` |
| Hito 0.3 — reputación atribuida al peer correcto | ✅ | `8c2366d` |
| Hito 0.1 — PSK real | ⬜ | — |
| Hito 2.1 — gateway sobre SocialStore | ⬜ | — |
| Hito 2.2 — índice sled/redb | ⬜ | — |
| Hito 3.2 — WS subscripciones reales | ⬜ | — |
| Resto | ver `README.md` | |

Plan detallado y orden de prioridades: ver el bloque "Plan paso a paso"
del propio `README.md` y los hitos completos en `plan-crossplatform.md`.
