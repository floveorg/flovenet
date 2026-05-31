# Plan de Pruebas — Flovenet en nodo1

## Stack instalado

| Componente | Versión |
|------------|---------|
| Rust | 1.96.0 |
| Docker | 29.5.2 |
| Docker Compose | v5.1.4 |
| SO | Ubuntu 26.04 LTS x86_64 |
| Binario | `target/release/daemon` |

---

## Paso 1 — Daemon P2P (nodo individual)

```bash
./target/release/daemon daemon --port 0 --api-port 9091 --roles compute,storage
```

- Escucha en puerto aleatorio libp2p
- API HTTP en `http://localhost:9091`
- Roles: compute (CPU/RAM) + storage (disco)

---

## Paso 2 — Gateway GraphQL

```bash
# Terminal 2
./target/release/daemon api-gateway --port 8080
```

- Abrir `http://localhost:8080/graphql` (playground)
- Probar queries/mutations:

```graphql
# Registro
mutation {
  register(email: "test@x.com", password: "1234", displayName: "Nodo1") {
    token
    profile { peerId }
  }
}

# Crear post
mutation {
  createPost(content: "Hola desde nodo1") { cid content timestamp }
}

# Feed
query {
  feed(limit: 10) {
    post { content author }
    author { displayName }
  }
}

# Subscripción nuevos posts
subscription {
  newPosts { cid content author }
}
```

---

## Paso 3 — Red multi-nodo con Docker Compose

```bash
docker compose up --build
```

Lanza 4 servicios:
- `node1` — daemon (compute+storage, puerto 9091)
- `node2` — daemon (compute, puerto 9092)
- `node3` — daemon (storage, puerto 9093)
- `gateway` — GraphQL API (puerto 8080)

Probar descubrimiento P2P entre nodos.

---

## Paso 4 — Ejecución WASM local

```bash
./target/release/daemon run --manifest _start --image wasm_images/feed_ranker.wasm
```

Requiere wasmtime 24 (incluido en binario).

---

## Paso 5 — Sub-red privada (PSK)

```bash
dd if=/dev/urandom bs=32 count=1 of=swarm.key
./target/release/daemon daemon --swarm-key swarm.key --api-port 9094
```

---

## Paso 6 — Empaquetar .deb local

```bash
./scripts/build-deb.sh 0.1.0 amd64
sudo dpkg -i target/flovenet_0.1.0_amd64.deb
systemctl status flovenet-daemon
journalctl -u flovenet-daemon -f
```

---

## Paso 7 — Release CI/CD (GitHub Actions)

Archivo: `.github/workflows/release.yml`

**Trigger**: push tag `v*`

**Jobs**:
- `build-deb (amd64)` — ubuntu-24.04, target x86_64
- `build-deb (arm64)` — ubuntu-24.04-arm, target aarch64
- `create-release` — junta artifacts + SHA256 checksums + crea GitHub Release

**Para lanzar un release**:
```bash
git tag v0.1.0
git push origin v0.1.0
# GitHub Actions produce .deb amd64 + arm64 en releases
```
