# Flovenet

Red descentralizada P2P para conectar redes sociales. Infraestructura Rust + libp2p + WASM + IPFS + GraphQL Gateway.

## Stack

| Capa | Tecnología |
|------|-----------|
| Lenguaje | Rust (edition 2021) |
| Networking | libp2p (Noise + Yamux + Kademlia + Gossipsub) |
| Ejecución | Wasmtime 24 (WASI preview1) |
| Almacenamiento | LocalBackend → IpfsBackend (Kubo) → S3Backend (MinIO/AWS) → HybridBackend |
| API | async-graphql + axum + WebSocket |
| Cripto | Ed25519 + ChaCha20-Poly1305 + argon2id |
| Identidad | Ed25519 keys + keystore cifrado |
| Reputación | CRDT eventualmente consistente |
| Trust | Web of Trust (2º orden) |

## Arquitectura

```
                    App Web                     App Móvil
                       │                            │
                       └──────────┬─────────────────┘
                                  │ GraphQL (WS)
                                  ▼
                         Gateway Node
                    ┌─────────────────────┐
                    │  graphql_api        │
                    │  (async-graphql     │
                    │   + axum + WS)      │
                    ├─────────────────────┤
                    │  identity (auth)    │
                    │  storage (IPFS/S3)  │
                    │  social_protocol    │
                    └────────┬────────────┘
                             │ libp2p
                             ▼
                    ┌─────────────────────┐
                    │   Red P2P           │
                    │  ┌─────┐ ┌─────┐   │
                    │  │comp.│ │stor.│   │
                    │  └─────┘ └─────┘   │
                    │  ┌─────┐ ┌─────┐   │
                    │  │valid│ │ ai  │   │
                    │  └─────┘ └─────┘   │
                    └─────────────────────┘
```

## Workspace (16 crates funcionales + 2 de test)

```
flovenet/
├── Cargo.toml               (workspace root)
├── flovenet-core/           — core multiplataforma (JNI Android)
├── daemon/                  — proceso principal (binario)
├── cli/                     — CLI con clap
├── resource_manager/        — CPU/RAM/GPU/disco
├── vm_runtime/              — trait Runner + WasmtimeRunner
├── market_protocol/         — libp2p behaviour oferta/demanda
├── p2p_cache/               — BitSwap-lite block exchange
├── reputation_engine/       — CRDT reputación
├── ipfs_layer/              — IpfsBackend (Kubo HTTP API)
├── storage/                 — StorageBackend trait + Local + S3 + Hybrid
├── crypto/                  — primitivas criptográficas
├── identity/                — keystore + PeerId
├── scheduler/               — matching + placement + reputación
├── trust_graph/             — Web of Trust
├── social_protocol/         — Post, Profile, Follow, Feed
├── graphql_api/             — async-graphql + axum + WS
├── test_harness/            — harness de integración
├── test_reporter/           — reporter de resultados
```

## Uso rápido

```bash
# Compilar
cargo build --release

# Ver recursos del nodo
cargo run --release -- status

# Compartir recursos como nodo compute
cargo run --release -- share --role compute

# Iniciar daemon completo
cargo run --release -- daemon --port 0 --api-port 9090 --roles compute,storage

# Iniciar gateway GraphQL
cargo run --release -- api-gateway --port 8080

# Ejecutar un WASM localmente
cargo run --release -- run --manifest _start --image wasm_images/feed_ranker.wasm

# Con GPU (via env var para testing)
FLOVENET_GPU_VRAM_GB=24 FLOVENET_GPU_MODEL="RTX 4090" cargo run --release -- share --role ai
```

## CI/CD

Cada push desencadena el pipeline **CI** (fmt → clippy → build → test, audit, deny, dashboard, deb-package, cross-build).

| Job | Descripción |
|-----|------------|
| check | fmt + clippy + build + test (ubuntu) |
| cross-build | Build para `x86_64-unknown-linux-gnu` + `x86_64-pc-windows-msvc` |
| dashboard | Build web (Vite) |
| deb-package | Genera `.deb` |
| docker | Build imagen Docker |
| audit | `cargo audit` (advisories ignorados: libp2p, wasmtime) |
| deny | `cargo deny` (licencias, bans, fuentes) |

## Release

Al pushear un tag `v*` se activa el workflow **Release** que compila para todas las plataformas y crea un GitHub Release con los binarios:

| Plataforma | Archivo |
|------------|---------|
| Linux (amd64) | `flovenet_*.deb` |
| Windows (x86_64) | `daemon.exe` |
| macOS (Intel) | `daemon-x86_64-apple-darwin` |
| macOS (Apple Silicon) | `daemon-aarch64-apple-darwin` |

```bash
# Crear y publicar un release
git tag -a v0.2.0 -m "v0.2.0"
git push origin v0.2.0
```

## Instalación

### Debian/Ubuntu

```bash
# Desde un .deb
sudo dpkg -i flovenet_0.2.0_amd64.deb

# Servicios systemd disponibles:
#   flovenet-daemon.service    (P2P en :9090)
#   flovenet-gateway.service   (GraphQL en :8080)
```

### macOS / Windows

Descargar el binario desde GitHub Releases y ejecutar:

```bash
./daemon daemon --port 0 --api-port 9090 --roles compute,storage
```

## Docker

```bash
# Construir y lanzar 3 nodos + gateway
docker compose up --build

# O construir individual
docker build -t flovenet .
docker run -e RUST_LOG=info flovenet daemon --port 0 --api-port 9090
```

## GraphQL API

El gateway expone GraphQL en `http://localhost:8080/graphql` (con playground).

```graphql
# Registro
mutation { register(email: "user@x.com", password: "pass", displayName: "Alice") { token profile { peerId } } }

# Crear post
mutation { createPost(content: "Hola mundo") { cid content timestamp } }

# Feed
query { feed(limit: 10) { post { content author } author { displayName } } }

# Subscripción a nuevos posts
subscription { newPosts { cid content author } }
```

## Sub-red privada (PSK)

```bash
# Generar swarm key (32 bytes)
dd if=/dev/urandom bs=32 count=1 of=swarm.key

# Iniciar nodo con clave
cargo run -- daemon --swarm-key swarm.key
```

## Tests

```bash
cargo test        # ~178 tests
cargo clippy      # 0 warnings
cargo fmt         # format
```

## Fases de implementación

| Fase | Estado |
|------|--------|
| F0 Bootstrap | ✅ |
| F1 Networking + Discovery | ✅ |
| F2 Storage Layer | ✅ |
| F3 WASM + Scheduling MVP | ✅ |
| F4 Identidad + Cripto + Biometría | ✅ |
| F5 GraphQL API Gateway | ✅ |
| F6 Reputación | ✅ |
| F7 Trust Graph + Validación | ✅ |
| F8 Replicación + S3Backend + P2P Cache | ✅ |
| F9 GPU Distribuida | 🔄 |
| F10–F14 | ⬜ |
