# Plan de Implementación — Flovenet

> Red descentralizada P2P para conectar redes sociales.
> Infraestructura Rust + libp2p + WASM + IPFS + GraphQL Gateway.

## Stack técnico

| Capa | Tecnología |
|------|-----------|
| Lenguaje | Rust (nightly) |
| Networking | libp2p (Noise + Yamux + Kademlia + Gossipsub) |
| Ejecución | Wasmtime (WASI) → Firecracker (póstumo) |
| Almacenamiento | IPFS (Kubo) → trait → S3 (MinIO/AWS) |
| API | async-graphql + axum + WebSocket |
| Cripto | X25519 + Ed25519 + ChaCha20-Poly1305 → PQC híbrido (póstumo) |
| Identidad | Ed25519 keys + biometría local (argon2id) |
| Reputación | CRDT eventualmente consistente |
| Trust | Web of Trust (2º orden) |
| CI/CD | cargo test, clippy, fmt, audit, deny, fuzz |

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

## Workspace Cargo

```
flovenet/
├── Cargo.toml               (workspace)
├── daemon/                  — proceso principal
├── resource_manager/        — CPU/RAM/GPU/disco
├── vm_runtime/              — trait Runner + WasmtimeRunner
├── market_protocol/         — libp2p behaviour oferta/demanda
├── p2p_cache/               — BitSwap-lite block exchange (libp2p request-response)
├── reputation_engine/       — CRDT reputación
├── ipfs_layer/              — IpfsBackend (Kubo HTTP API)
├── storage/                 — trait StorageBackend + LocalBackend + S3Backend + IpfsBackend
├── crypto/                  — primitivas criptográficas
├── identity/                — keystore + perfiles + firmas
├── scheduler/               — matching + placement
├── trust_graph/             — Web of Trust
├── social_protocol/         — Post, Profile, Follow, Feed
├── graphql_api/             — async-graphql + axum + WS
└── cli/                     — clap CLI
```

## Fases de implementación

> ✅ = Completada  |  🔄 = En progreso  |  ⬜ = Pendiente

---

### Fase 0 — Bootstrap ✅ (1–2 sem)

**Objetivo:** repo + scaffolding + decisiones congeladas.

**Logrado:**
- ✅ Workspace Cargo con 16 crates
- ✅ Traits foundation: `StorageBackend`, `Runner`, `Scheduler`, `CryptoProvider`, `TrustProvider`
- ✅ CLI con `clap`: `daemon`, `api-gateway`, `share`, `run`, `status`
- ✅ CI/CD: `cargo test`, `clippy`, `fmt`, audit, deny
- ✅ Observabilidad: `tracing` + `tracing-subscriber` + Prometheus `/metrics`

---

### Fase 1 — Networking + Gateway Discovery ✅ (2–3 sem)

**Objetivo:** nodo descubre peers, anuncia recursos. Gateway discovery en DHT.

**Logrado:**
- ✅ libp2p Swarm: Kademlia + Gossipsub + Identify + Ping + Noise + Yamux
- ✅ `resource_manager` con detección real de CPU/RAM/disco vía `sysinfo`
- ✅ Roles: `storage`, `validation`, `compute`, `ai`, `social`
- ✅ NodeDescriptor con slots atómicos
- ✅ Gateway discovery structures preparadas (DHT)
- ✅ Heartbeat cada 30s en Gossipsub topic `node/status`
- ✅ Topics sociales: `slots/announce`, `social/post`, `social/profile`, `social/follow`
- ✅ Docker compose para test multi-nodo

---

### Fase 2 — Storage Layer ✅ (2–3 sem)

**Objetivo:** almacenamiento persistente + modelos sociales.

**Logrado:**
- ✅ `trait StorageBackend` completo: `add`, `get`, `pin`, `unpin`, `ls_pins`, `delete`
- ✅ `LocalBackend` filesystem con data dir + `pins.json`
- ✅ `IpfsBackend` vía Kubo HTTP API (`/api/v0/add`, `/cat`, `/pin/*`, `/block/rm`)
- ✅ `SocialStore` con CRUD typed de `Profile`, `Post`
- ✅ Modelos sociales: `Post`, `Profile`, `Follow`, `Feed`
- ✅ Almacenamiento determinístico SHA2-256 → CIDv1

---

### Fase 3 — WASM + Scheduling MVP ✅ (3–4 sem)

**Objetivo:** ejecutar un job aislado en un slot remoto.

**Logrado:**
- ✅ `WasmtimeRunner` implementa `trait Runner` (WASI preview1, wasmtime 24)
- ✅ Sandbox con fuel limit + timeout por tokio
- ✅ `market_protocol` con `json::Behaviour` request-response (`/flovenet/job/1.0.0`)
- ✅ `LocalScheduler` con matching por slots/RAM/disk/GPU/roles
- ✅ Pre-baking: feed_ranker.wasm + moderator.wasm (wasm32-wasip1)
- ✅ Daemon integrado: `run` ejecuta WASM local, `share` anuncia slots, `status` muestra recursos

---

### Fase 4 — Identidad + Cripto + Biometría ✅ (2–3 sem)

**Objetivo:** tráfico firmado, cifrado en reposo, identidad de usuarios.

**Logrado:**
- ✅ `crypto` crate: `EncryptedBlob::encrypt/decrypt` con ChaCha20-Poly1305, `SignedEnvelope::sign/verify` con Ed25519
- ✅ `identity` crate: `KeyStore` cifrado con argon2id, `generate_keypair` + `derive_key_from_password`
- ✅ `PeerId::from_public_key_bytes` — deriva PeerId CIDv1 desde clave pública
- ✅ `Post::sign_with` / `verify_signature` y `Follow::sign_with` / `verify_signature`
- ✅ Test suite: 17 tests todos pasando, clippy clean

---

### Fase 5 — GraphQL API Gateway ✅ (3–4 sem) ⭐

**Objetivo:** super-nodos exponen GraphQL para frontends web/móvil.

**Logrado:**
- ✅ Schema GraphQL completo: Query (profile, post, feed, search, followers, following), Mutation (register, login, createPost, deletePost, follow, unfollow, updateProfile), Subscription (newPosts)
- ✅ Auth: registro con email+password → gateway genera par Ed25519 cifrado con argon2id; sesión JWT
- ✅ InMemoryStore (HashMap + Arc<RwLock<>>) para perfiles, posts, follows durante la sesión
- ✅ Resolvers conectados con store real: feed devuelve posts con autores, search filtra, followers/following resuelven perfiles
- ✅ WebSocket subscriptions vía broadcast::channel
- ✅ Playground UI en `/graphql`
- ✅ CORS configurado
- ✅ Handler HTTP directo con serde_json parse de async_graphql::Request/Response
- ✅ Integrado en daemon: `api-gateway` comando levanta gateway funcional

---

### Fase 6 — Reputación ✅ (2–3 sem)

**Objetivo:** modelo "no-dinero, código abierto recíproco".

**Logrado:**
- ✅ `reputation_engine` con CRDT LWW (Last-Writer-Wins) merge por peer_id
- ✅ Score formula: `50 + net_contribution_hours × 10 × uptime_mult × success_mult × diversity_mult + bonus`
- ✅ Contribution/Consumption events, weighted uptime tracking
- ✅ Bonus events: `BonusContent`, `BonusVerification`
- ✅ "Deuda suave": scheduler rankea por reputación pero acepta cualquier nodo
- ✅ Leaderboard: `top_n()` devuelve N mejores scores ordenados
- ✅ CRDT merge de estados remotos vía Gossipsub topic `reputation/score`
- ✅ `ReputationEvent` enum con 7 variantes, timestamps para LWW
- ✅ 9 tests unitarios (baseline, contribution, consumption, leaderboard, CRDT merge, bonus, old-event rejection, top-N, job outcome)
- ✅ `LocalScheduler::rank_candidates()` rankea nodos por composite (recursos × reputación)
- ✅ Daemon: publica reputación cada 3 heartbeats, procesa gossip entrante, registra job outcomes
- ✅ Scheduler usa reputación para ranking: `test_rank_prefers_higher_reputation`

---

### Fase 7 — Trust Graph + Validación Distribuida ✅ (2–3 sem)

**Objetivo:** la red se gobierna sola a escala.

**Logrado:**
- ✅ `trust_graph` crate: `TrustEdge {signer, target, weight, signature, timestamp}`
- ✅ Web of Trust: `direct_trust()` + `transitive_trust()` con BFS hasta profundidad configurable y decay 50% por hop
- ✅ Trust score combinado: `direct_trust()` (100%) + `transitive_trust()` (30% peso)
- ✅ `select_validators()` combina trust + reputación (60/40) para elegir validadores
- ✅ CRDT merge (LWW por timestamp) para gossip de edges
- ✅ 7 tests unitarios (direct, transitive, stale, merge, trusted_by, validator selection)
- ✅ Gossipsub topic `trust/edge` para propagación de edges
- ✅ `trust_graph` integrado en `NodeNetwork` con `Arc<RwLock<>>`

**Diferido (post-MVP):** validación cruzada 5%, sharding DHT por región, governance vía trust graph.

---

### Fase 8 — Replicación + S3Backend + P2P Cache ✅ (3–4 sem)

**Objetivo:** alta disponibilidad + reducir dependencia crítica de IPFS + S3.

**Logrado:**
- ✅ `storage::s3::S3Backend` implementa `trait StorageBackend` (MinIO / AWS S3 vía REST + Basic auth)
- ✅ `storage::hybrid::HybridBackend` — composición en tiers (fast→slow), replica escritura, fallback lectura con promoción automática
- ✅ `p2p_cache` crate: `BlockCache` in-memory con eviction LRU, `BlockRequest`/`BlockResponse` vía `/flovenet/block/1.0.0`
- ✅ `CacheBehaviour` integrado en `FlovenetBehaviour` y `NodeNetwork`, eventos manejados en event loop
- ✅ Swarm key (PSK) opcional: CLI `--swarm-key`, `load_swarm_key()`, plumbing hasta `NodeNetwork`
- ✅ `S3Backend` con `add/get/delete/pin/unpin/ls_pins`, CID local via SHA2-256
- ✅ 6 tests storage (key_for, new, hybrid add/get, hybrid fallback)
- ✅ 4 tests p2p_cache (serde, add/has, eviction, not-found)
- ✅ 61 tests total en workspace, 0 warnings, clippy clean

**Diferido (post-MVP):** IPFS Cluster, GC whitelist

---

### Fase 9 — GPU Distribuida 🔄 (3–4 sem)

**Objetivo:** workloads IA (moderación, recomendación, clasificación) en GPU compartida.

**Logrado:**
- ✅ `resource_manager::gpu::GpuSlot` con unidades fraccionadas 2/4/8 GiB VRAM, `create_slots()` automático
- ✅ `resource_manager::gpu::detect_gpu()` — detecta GPU via env var (`FLOVENET_GPU_VRAM_GB`/`_MODEL`) o `/proc/driver/nvidia/gpus/` en Linux
- ✅ `NodeResources::detect()` ahora población real de `gpu_vram_gb` y `gpu_model`
- ✅ `NodeDescriptor::slots_for_role(Ai)` limitado por CPU min(8) y VRAM min(2 GiB/slot)
- ✅ `GpuSlot::slots_needed()` calcula slots requeridos para una cantidad de VRAM
- ✅ 5 tests GPU (create_slots x3, slots_needed, detect_gpu)
- ✅ Comandos `share` y `status` muestran info GPU
- ✅ Cero warnings, clippy clean

**Pendiente:**
- ⬜ GPU remota: anunciar slots en Gossipsub, matching remoto con `NodeRole::Ai`
- ⬜ Casos sociales WASM con GPU (moderación, clasificación)
- ⬜ MIG/slicing real para GPUs consumer

---

### Fase 10 — Multi-plataforma + Android (3–4 sem)

**Objetivo:** expandir la red con nodos ligeros.

**Tareas:**
- Linux/macOS/Windows completo
- Android Foreground Service (roles: storage, validation; compute opcional)
- `resource_manager` adaptado a cada plataforma
- Gateway accesible desde Android

**Exit:** App Android se conecta a gateway, navega feed, publica posts.

---

### Fase 11 — Economía Comunitaria (2–3 sem)

**Objetivo:** pool comunitario sin blockchain.

**Tareas:**
- 10% slots reservados para proyectos open-source aprobados
- Governance vía Trust Graph + reputación
- Pool comunitario gestionado por CRDT

**Exit:** Proyecto open-source puede usar slots donados por la red.

---

### Fase 12 — PQC (póstumo, 2–3 sem)

**Objetivo:** resistencia post-cuántica. Solo cuando la red funciona establemente.

**Tareas:**
- `pqcrypto-kyber` (ML-KEM) + X25519 para handshake híbrido
- `pqcrypto-dilithium` (ML-DSA) + Ed25519 para firmas híbridas
- `CryptoProvider` trait permite intercambiar backend sin tocar otra capa
- Auditoría externa de criptografía

**Exit:** Handshake híbrido funcional, firmas duales.

---

### Fase 13 — Firecracker (póstumo, 3–4 sem)

**Objetivo:** máximo aislamiento para cargas sensibles. Solo cuando WASM runtime está maduro.

**Tareas:**
- `FirecrackerRunner` implementa `trait Runner` (Linux only, subprocess + jailer)
- Sin modificar scheduler — solo nueva implementación del trait
- Límites vía cgroups + jailer
- Soporte para imágenes base en formato ext4

**Exit:** Mismo job corre en WASM o Firecracker intercambiablemente.

---

### Fase 14 — Auditoría + Release 1.0 (3–4 sem)

**Objetivo:** preparar versión estable.

**Tareas:**
- Auditorías: seguridad, criptografía, scheduler, trust graph, replicación
- Hardening general
- Documentación completa (API reference, deployment guide)
- Release 1.0

**Exit:** Release estable, red con 50+ nodos heterogéneos.

---

## Resumen de fases y duración

| Fase | Temporal | Depende de |
|------|----------|------------|
| F0 Bootstrap | 1–2 sem | — |
| F1 Networking + Discovery | 2–3 sem | F0 |
| F2 Storage | 2–3 sem | F1 |
| F3 WASM + Scheduling | 3–4 sem | F1, F2 |
| F4 Identidad + Crypto | 2–3 sem | F2 |
| **F5 GraphQL API Gateway** | **3–4 sem** | F2, F4 |
| **F6 Reputación ✅** | **2–3 sem** | F3 |
| **F7 Trust Graph + Validación ✅** | **2–3 sem** | F4, F6 |
| **F8 Replicación + S3 + Cache ✅** | **3–4 sem** | F2, F7 |
| **F9 GPU 🔄** | **3–4 sem** | F8 |
| F10 Multi-plataforma | 3–4 sem | F8 |
| F11 Economía | 2–3 sem | F7 |
| F12 PQC | 2–3 sem | F4 (después de que F1–F11 anden) |
| F13 Firecracker | 3–4 sem | F3 (después de WASM maduro) |
| F14 Auditoría + Release | 3–4 sem | F12, F13 |

**Total estimado:** ~35–42 semanas

## Riesgos clave

1. Firecracker demasiado pronto → diferido a F13
2. PQC demasiado pronto → diferido a F12
3. Dependencia excesiva de IPFS → S3Backend + BitSwap-lite en F8
4. Centralización de validadores → rotación por reputación en F7
5. Complejidad de GPU → empezar con slicing simple en F9
6. Gateway custodia claves → riesgo de seguridad, mitigación con cifrado argon2id + JWT corto

## Principios de diseño

1. **WASM primero** — Wasmtime → Firecracker (solo si se necesita)
2. **Todo sustituible** — traits para StorageBackend, Runner, Scheduler, CryptoProvider, TrustProvider
3. **Criptografía simple al inicio** — X25519 + Ed25519; PQC después
4. **Sin blockchain** — CRDT + Trust Graph + Reputación son suficientes
5. **Gateway pragmático** — frontends no necesitan ser peers libp2p
