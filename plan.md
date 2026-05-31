# Plan de pruebas de red reales

## Diagnóstico

### ✅ Lo que ya funciona

- **178 unit tests** en 26 crates (`cargo test`)
- **P2P gossipsub local** — `test_p2p_gossip_message_exchange` intercambia mensajes entre 2 nodos en localhost
- **test_harness** (`flovenet-test`) — 3 escenarios: mesh de N nodos, gossip propagation, mensajes secuenciales. Corre en localhost sin Docker
- **test_reporter** (`flovenet-report`) — formatea resultados de tests (terminal/markdown/json)
- **GraphQL API** en gateway — playground, register, login, createPost, feed, follow

### ⚠️ Docker integration test (parcial)

`tests/docker_integration_test.py` corre contra 4 containers. Último reporte:
**49/67 scenarios pasaron, 18 fallaron**

| Falla | Causa raíz |
|-------|-----------|
| `gateway_register_u1` missing fields | Helper `GF()` no navega correctamente en response anidado (`register.profile.displayName`) |
| `p2p_listening_*` | `--port 0` = puerto aleatorio, `/proc/net/tcp` nunca los encuentra |
| `cross_tcp_*_p2p` | Misma causa: puertos P2P no existen donde se esperan |
| `updateProfile` | Bug conocido: mutations hardcodean `user_id = "user"`, funciona post-register pero falla si no hay perfil |
| `gateway→node1_api` timeout | Containers no alcanzan a inicializar antes del test |

### ❌ Lo que no funciona / no existe

- **PSK a nivel transporte**: `build_transport()` log warning — swarm.key no cifra realmente
- **Sin discovery**: Kademlia se crea pero `bootstrap_kademlia()` nunca se llama. Nodos no se encuentran
- **Job market sin descubrimiento**: No hay forma de saber qué nodos existen
- **InMemoryStore volátil**: GraphQL pierde estado al reiniciar
- **Auth ficticio**: `createPost`, `follow`, etc. hardcodean `user_id = "user"`
- **docker CI job nunca corre**: Condición `github.ref == 'refs/heads/main'` pero rama es `master`
- **Sin tests de frontend web**

---

## Roadmap

### Fase 1 — Arreglar infraestructura Docker

| Acción | Archivo | Dificultad |
|--------|---------|-----------|
| Puertos fijos en docker-compose (cambiar `--port 0` a `--port 9091`, etc.) | `docker-compose.yml` | Trivial |
| Healthchecks para esperar inicialización | `docker-compose.yml` | Trivial |
| Build y verify local | `Dockerfile` | Trivial |

### Fase 2 — Arreglar CI

| Acción | Archivo | Dificultad |
|--------|---------|-----------|
| Docker job: cambiar `main` → `master` | `.github/workflows/ci.yml` | Trivial |
| Agregar job `p2p-test` que corra `flovenet-test` | `.github/workflows/ci.yml` | Baja |
| Opcional: job `integration-test` con docker compose | `.github/workflows/ci.yml` | Media |

### Fase 3 — Arreglar docker_integration_test.py

| Acción | Dificultad |
|--------|-----------|
| Arreglar helper `GF()` para response anidados | Baja |
| Agregar `retry` / `wait_for_healthy()` antes de cada sección | Baja |
| Arreglar búsqueda de puertos P2P (leer logs o usar puertos fijos) | Baja |
| Marcar `updateProfile` como expected failure con comentario claro | Trivial |

### Fase 4 — Mejorar test_harness (opcional pero recomendado)

| Escenario | Prioridad |
|-----------|-----------|
| `p2p_job_offer` — oferta de trabajo entre 2 nodos | Alta |
| `p2p_block_exchange` — intercambio de bloques | Alta |
| `p2p_reputation_sync` — reputación vía gossipsub | Media |
| `multi_hop_gossip` — 4 nodos en cadena | Media |
| `p2p_discovery` — Kademlia bootstrap + find_node | Alta (habilita red real) |

### Fase 5 — Features para red usable (más adelante)

| Feature | Esfuerzo |
|---------|----------|
| mdns o relay para descubrimiento automático | Alta |
| LocalBackend persistente para gateway | Media |
| Auth middleware real (validar JWT en mutations) | Media |
| PSK real a nivel transporte | Baja (ya hay infraestructura) |
| CI con Docker integration test automático | Media |

---

## Uso rápido

```bash
# Build local
docker build -t flovenet:latest .

# Docker compose con 3 nodos + gateway
docker compose up --build

# test_harness nativo
cargo run -p test_harness -- --scenarios all -o report.json
cargo run -p test_reporter -- report.json -f terminal

# Prueba manual
# Terminal 1: Gateway
cargo run -- api-gateway --port 8080
# Terminal 2: Nodo
cargo run -- daemon --port 9091 --api-port 9092 --roles compute,storage
```
