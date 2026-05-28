# Flovenet

Red descentralizada P2P para conectar redes sociales. Infraestructura Rust + libp2p + WASM + IPFS + GraphQL Gateway.

## Stack

| Capa | Tecnología |
|------|-----------|
| Lenguaje | Rust (edition 2021) |
| Networking | libp2p 0.54 (TCP + Noise + Yamux + Kademlia + Gossipsub + Identify + Ping) |
| Ejecución | Wasmtime 24 (WASI preview1) |
| Almacenamiento | LocalBackend → IpfsBackend (Kubo) → S3Backend (MinIO; AWS via Basic auth → no SigV4) → HybridBackend |
| API | async-graphql + axum (HTTP POST + playground; **WebSocket no expuesto todavía**) |
| Cripto | Ed25519 + ChaCha20-Poly1305 + argon2id |
| Identidad | Ed25519 keys + keystore cifrado |
| Reputación | CRDT eventualmente consistente |
| Trust | Web of Trust (2º orden) |

## Arquitectura

### Diseñada (objetivo)

```
                    App Web                     App Móvil
                       │                            │
                       └──────────┬─────────────────┘
                                  │ GraphQL (HTTP + WS objetivo)
                                  ▼
                         Gateway Node
                    ┌─────────────────────┐
                    │  graphql_api        │
                    │  + identity (auth)  │
                    │  + storage (IPFS/S3)│
                    │  + social_protocol  │
                    └────────┬────────────┘
                             │ libp2p
                             ▼
                    ┌─────────────────────┐
                    │   Red P2P           │
                    │  ┌─────┐ ┌─────┐    │
                    │  │comp.│ │stor.│    │
                    │  └─────┘ └─────┘    │
                    │  ┌─────┐ ┌─────┐    │
                    │  │valid│ │ ai  │    │
                    │  └─────┘ └─────┘    │
                    └─────────────────────┘
```

### Real (hoy)

```
   App Web ─HTTP POST─┐
                      ▼
              ┌─────────────────────┐
              │ Gateway (axum)      │
              │ - InMemoryStore     │◄── feed/perfiles/follows
              │   (volátil)         │    NO se persisten
              │ - swarm libp2p      │    NO publica al daemon
              └─────────────────────┘
                          ✕ (sin conexión real con la red P2P)

   Daemon ─┐
           ▼
   ┌─────────────────────┐
   │ Swarm libp2p        │
   │ - gossipsub + KAD   │
   │ - request/response  │   (sin --bootstrap, sin mDNS:
   │ - market + p2p_cache│    arranca aislado en cada instancia)
   │ - reputation gossip │
   └─────────────────────┘
                  ▲
                  │ /metrics + /health (axum)
                  │ → SocialStore creado pero NO enlazado al gateway
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
# NOTA: wasm_images/feed_ranker está EXCLUIDO del workspace.
# Antes de este comando hay que compilarlo aparte:
#   (cd wasm_images/feed_ranker && cargo build --release --target wasm32-wasi)
cargo run --release -- run --manifest _start --image wasm_images/feed_ranker.wasm

# Con GPU (via env var para testing)
FLOVENET_GPU_VRAM_GB=24 FLOVENET_GPU_MODEL="RTX 4090" cargo run --release -- share --role ai
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

# Subscripción a nuevos posts (definida en el schema, pero el gateway
# todavía NO sirve `graphql-ws` ni `graphql-transport-ws` por la ruta
# `/graphql` — solo HTTP POST y GET=playground. Sin transporte WS no
# llegan eventos al cliente).
subscription { newPosts { cid content author } }
```

## Sub-red privada (PSK) — **inerte hoy**

```bash
# Generar swarm key (32 bytes)
dd if=/dev/urandom bs=32 count=1 of=swarm.key

# Iniciar nodo con clave
cargo run -- daemon --swarm-key swarm.key
```

> El flag `--swarm-key` se acepta y se carga, pero el transporte
> libp2p **no aplica la PSK** (el código emite el warning
> *"Swarm key (PSK) provided but transport-level PSK not yet
> implemented"* y monta TCP+Noise sin filtro de red). Hasta que se
> implemente, cualquier nodo del internet público con la misma
> versión de protocolo puede unirse.

## Fases de implementación

Leyenda: ✅ funcional · 🟡 parcial / con gaps importantes · 🔄 en curso · ⬜ pendiente.

| Fase | Estado | Notas |
|------|--------|-------|
| F0 Bootstrap | ✅ | |
| F1 Networking + Discovery | 🟡 | mDNS LAN/bridge + `--bootstrap-peers` operativos (2026-05-28). Falta AutoNAT, DCUtR, relay y QUIC/WS-transport para NAT/internet público. |
| F2 Storage Layer | ✅ | LocalBackend + IpfsBackend + S3Backend + HybridBackend. |
| F3 WASM + Scheduling MVP | ✅ | Wasmtime + LocalScheduler. |
| F4 Identidad + Cripto + Biometría | 🟡 | Ed25519 + keystore + JWT OK. Biometría no implementada. |
| F5 GraphQL API Gateway | 🟡 | Schema + auth + playground OK, pero **`AppState` usa `InMemoryStore` volátil**: no toca `SocialStore` ni red P2P, y **WS no expuesto**. Si el gateway cae, se pierde feed/perfiles/follows. |
| F6 Reputación | 🟡 | CRDT + gossip OK. Los eventos `JobSuccess/Failure` ahora los emite el **requester** sobre el **provider** (2026-05-28). Sigue sin firma criptográfica → Hito 6.1. |
| F7 Trust Graph + Validación | 🟡 | Aggregación + transitividad de 2.º orden OK. Verificación Ed25519 del `TrustEdge` operativa (2026-05-28): edges con `signer_pubkey` + `signature` se rechazan si no validan; edges legacy se aceptan con warning. |
| F8 Replicación + S3Backend + P2P Cache | 🟡 | HybridBackend replica entre tiers locales del mismo nodo. **No hay replicación cross-node automática**. `S3Backend` usa Basic auth → solo MinIO con proxy, no AWS S3 real. |
| F9 GPU Distribuida | 🔄 | |
| F10–F14 | ⬜ | |

## Estado real — limitaciones conocidas

Resumen de lo que el código **no hace todavía** (puntos descubiertos en
auditoría al revisar `daemon/src/networking/`, `graphql_api/src/lib.rs`,
`social_protocol/src/store.rs`, `storage/src/*` y el `Cargo.toml` raíz):

1. ~~**Descubrimiento de peers.** Sin `--bootstrap-peers` en la CLI, sin
   mDNS, sin AutoNAT/DCUtR. `bootstrap_kademlia()` existe pero nadie la
   llama.~~ → **resuelto parcialmente 2026-05-28**: añadidos `mdns`
   behaviour + auto-dial en `daemon/src/networking/swarm.rs` y flag
   `--bootstrap-peers /multiaddr,/multiaddr` en el CLI. Para
   internet/NAT siguen faltando AutoNAT, DCUtR, relay y QUIC/WS-transport.
2. **NAT/internet público.** Solo transporte TCP. No hay QUIC, WebRTC,
   WebSocket-transport ni relay/hole-punch. Nodos detrás de NAT no son
   alcanzables → la red no escala fuera de LAN/docker.
3. **El Gateway y el daemon no comparten estado.** `run_api_gateway`
   crea su propio swarm libp2p que no se une al daemon, no publica
   eventos al gossipsub social, no usa `SocialStore`. Toda la lógica
   social vive en `InMemoryStore` → desaparece al reiniciar.
4. **Subscripciones GraphQL inalcanzables.** `SubscriptionRoot::new_posts`
   existe pero la ruta `/graphql` solo registra `get(playground)` y
   `post(handler)` — no se monta `GraphQLSubscription` ni el upgrade WS.
5. **`InMemoryStore` no es HA.** No hay persistencia, ni replicación
   entre múltiples gateways, ni recuperación tras fallo. Levantar 2
   gateways en paralelo da feeds divergentes.
6. **`SocialStore::get_profile/get_post`** hacen *full-scan* de
   `ls_pins()` cada vez. Es O(n) por lookup; impracticable más allá de
   docenas de items.
7. **PSK inerte.** `--swarm-key` no se aplica al transporte (ver sección
   anterior).
8. ~~**Reputación auto-emitida.** El handler de jobs llama
   `record_job_outcome` sobre el `peer_id` local…~~ → **mitigado
   2026-05-28**: el provider deja de auto-puntuarse; el rating lo emite
   el requester al recibir la `JobResponse`
   (`daemon/src/networking/swarm.rs::handle_job_market_event`). Sigue
   sin firma criptográfica del requester → Hito 6.1.
9. ~~**TrustGraph sin verificación.** `add_edge` no verifica
   `edge.signature`…~~ → **resuelto 2026-05-28**: `TrustEdge` ahora
   incluye `signer_pubkey` y `add_edge` rechaza firmas inválidas; edges
   legacy (sin pubkey/signature) se aceptan con warning. Ver
   `trust_graph::VerifyOutcome`.
10. **HybridBackend = redundancia local.** Replica entre tiers del
    *mismo* proceso (Local + IPFS + S3). No reparte ni reconcilia
    contenido entre nodos del swarm; si un peer cae, sus pins se
    pierden salvo que estén también en IPFS público / S3.
11. **S3Backend usa HTTP Basic auth**, no AWS SigV4. Funciona contra
    MinIO con una shim, **no contra AWS S3 real**.
12. **`wasm_images/`** está excluido del workspace; los ejemplos del
    README requieren compilarlos antes con `cargo build --target
    wasm32-wasi` desde cada subdirectorio.
13. **Android (Fase 4 del plan-crossplatform)** no iniciada: existe
    scaffolding Gradle pero no app Kotlin funcional.

> Este bloque refleja la auditoría del 2026-05-28 y debe mantenerse
> sincronizado con `plan-crossplatform.md`. Si arreglas un punto,
> márcalo aquí mismo.

### Progreso de la iteración actual (Hito 1)

- 2026-05-28 — **mDNS + `--bootstrap-peers`** implementados y validados:
  - `.work/test-discovery.sh`: 3 daemons locales se descubren entre sí
    por mDNS (11–12 conexiones por nodo en 25s).
  - `.work/test-docker.sh`: 4 servicios (3 nodos + gateway) en docker
    compose bridge se ven todos contra todos. Combinación
    `--bootstrap-peers /dns4/node1/tcp/4001` + mDNS verificada.
  - Tests unitarios cli+daemon: 17/17 ok.
- Para iterar rápido en local: `Dockerfile.fast` reusa `target/release/daemon`
  del host (Ubuntu 24.04 base, glibc 2.39). El `Dockerfile` canónico
  (cargo-chef multi-stage) sigue siendo el de CI/release.

## Tests

```bash
cargo test        # ~178 tests
cargo clippy      # 0 warnings
cargo fmt         # format
```
