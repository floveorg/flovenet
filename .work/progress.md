# Progreso del plan de resiliencia

Branch: `resilience-plan` · Inicio: 2026-05-28

## Iteración 1 — Hito 1 (descubrimiento de peers)

### Objetivos
- **1.1** flag `--bootstrap-peers` (cli/src/lib.rs + daemon/src/main.rs).
- **1.2** mDNS auto-discovery en el behaviour del swarm.
- Test pragmático: 3 daemons locales se descubren entre sí; docker compose
  con 3 nodos + gateway forma red.

### Cambios aplicados
- `Cargo.toml`: añadida feature `mdns` a libp2p 0.54.
- `daemon/src/networking/swarm.rs`:
  - Nuevo campo `mdns: mdns::tokio::Behaviour` en `FlovenetBehaviour`.
  - `handle_mdns_event`: registra peers en Kademlia, marca como
    `explicit_peer` en gossipsub y hace `dial` automático.
  - `bootstrap_kademlia` quitado el `dead_code` + ahora acepta multiaddrs
    sin sufijo `/p2p/<peer_id>` haciendo dial directo.
- `cli/src/lib.rs`: nuevo arg `--bootstrap-peers` (lista CSV de multiaddrs).
- `daemon/src/main.rs`: parsea la lista y la pasa a `run_daemon`, que
  llama a `network.bootstrap_kademlia(...)` antes del event loop.
- `docker-compose.yml`: node1 con puerto libp2p fijo 4001; node2/3 +
  gateway se conectan con `--bootstrap-peers /dns4/node1/tcp/4001`.

### Validación
- Local: `.work/test-discovery.sh` — 3 daemons en localhost, espera
  ~25s, verifica que cada uno descubre ≥2 peers por mDNS.
- Docker: `.work/test-docker.sh` — `docker compose up`, espera 30s,
  exige que cada servicio vea ≥1 peer.

### Estado
- [x] Patch aplicado y compila release (host).
- [x] `cargo test -p cli -p daemon` → 17/17 ok (2026-05-28).
- [x] `.work/test-discovery.sh` → 3/3 daemons locales se descubren por mDNS.
- [x] `Dockerfile.fast` + imagen `flovenet:latest` construida (host binary).
- [x] `.work/test-docker.sh` → 4/4 servicios (node1/2/3+gateway) ven los
      otros 3 peers cada uno (matriz N×N completa). Logs muestran
      `bootstrap dial (no peer_id yet)` desde node2/3/gateway hacia
      `/dns4/node1/tcp/4001` y mDNS discovering recíproco dentro del
      bridge docker.
- [x] README actualizado: fase F1 anotada, punto 1 de "limitaciones"
      tachado.
- [ ] Commit.

## Iteraciones siguientes (planeadas)

- **Iter 2**: Hito 0 — verificación de firma en TrustGraph,
  atribución de reputación al peer correcto, tests.
- **Iter 3**: Hito 2.1 + 2.2 — `SocialStore` conectado al gateway,
  índice sled. Test E2E: register → createPost → reiniciar gateway →
  feed sigue presente.
- **Iter 4**: Hito 3.2 — WebSocket de subscripciones GraphQL real.
