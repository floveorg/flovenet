# Spec-Driven Development · Plan de trabajo en fases

> Plan derivado de `Spec Driven Development.txt` para un sistema P2P de
> cómputo + almacenamiento distribuido en Rust (slots Firecracker/Wasmtime,
> IPFS, libp2p, PQC, biometría, reputación recíproca, jerarquía de nodos).

---

## Fase 0 · Bootstrap (1–2 semanas)

**Objetivo:** repo + scaffolding + decisiones técnicas congeladas antes de
escribir código de producto.

- Workspace Cargo con crates: `daemon`, `resource_manager`, `vm_runtime`,
  `market_protocol`, `reputation_engine`, `ipfs_layer`, `crypto`, `cli`.
- Specs en Markdown (BDD/Gherkin-like) por user story — base para
  `cargo test` + `proptest`.
- CI: `cargo test`, `clippy`, `fmt`, `cargo-audit`, `cargo-deny`, fuzzing
  skeleton (`cargo-fuzz`).
- Observabilidad desde día 1: `tracing` + `tracing-subscriber`, exporter
  `prometheus`.
- CLI `clap` con subcomandos placeholder (`daemon`, `share`, `run`,
  `status`).

**Exit:** `cargo run -- daemon` levanta proceso vacío, expone `/metrics`,
logs estructurados.

---

## Fase 1 · Networking + recursos locales (PoC parcial)

**Objetivo:** un nodo descubre peers y anuncia slots. Sin compute aún.

- `libp2p` con: Noise + Yamux + Kademlia DHT + Gossipsub + Identify + Ping.
- `resource_manager`: detecta CPU/RAM/disco con `sysinfo`; expone slots
  disponibles (`{slots, cpu_freq, gpu, vram, uptime, role}`).
- **Modelo de slot atómico:** 1 vCPU / 1 GiB RAM / 8 GiB disco.
  Combinable Nx.
- **Roles declarados** (jerarquía pedida en el documento):
  - `storage` — almacenamiento + pinning IPFS
  - `validation` — claves + verificación de jobs
  - `compute` — procesamiento general
  - `ai` — comparten >2 GiB VRAM

  Cada nodo se auto-anuncia con su rol (puede ser híbrido).
- Gossipsub topic `slots/announce` con TTL.
- Test multi-nodo local (3–5 procesos) en docker-compose.

**Exit:** dos nodos arrancan, se descubren, listan slots mutuamente vía CLI.

---

## Fase 2 · IPFS layer mínima

**Objetivo:** subir/bajar contenido inmutable + pinning local. Solo
metadatos y manifests, no imágenes pesadas todavía.

- Empezar con **Kubo embebido vía HTTP API** (`rust-ipfs-api`) por madurez;
  migración a `ipfs-embed` cuando estabilice.
- API interna: `add(bytes) -> CID`, `get(cid)`, `pin(cid)`, `unpin(cid)`,
  `ls_pins()`.
- Política de pinning local: pin de todo lo que el nodo sube + lo que
  consume frecuentemente.
- Replicación: deferida a Fase 5 (IPFS Cluster).
- Pruebas: subir manifest, recuperarlo desde otro nodo solo con el CID.

**Exit:** un nodo sube `manifest.json`, otro lo descarga por CID y verifica
hash.

---

## Fase 3 · Scheduling + lanzamiento de "máquinas" (MVP)

**Objetivo:** ejecutar un job aislado en un slot remoto. Primero local,
luego cross-node.

**3a · Runtime local primero:**

- `vm_runtime` con abstracción `Runner`:
  - `WasmtimeRunner` (default, low risk, WASI snapshot1).
  - `FirecrackerRunner` (Linux only, via subprocess + jailer; bindings
    `firepilot` o propios).
- Límites por slot con `cgroups-rs` (CPU/RAM) + quota disco.
- Manifest mínimo: `{cid_image, entrypoint, args, max_duration,
  slots_required}`.

**3b · Distribuido:**

- `market_protocol` libp2p behaviour: oferta/demanda con request-response.
- Matching local-decision: cada proveedor decide aceptar según su política.
- Transfer de imagen: delta P2P sobre libp2p stream; fallback IPFS solo
  para assets grandes.
- Pre-baking de imágenes base populares (Alpine, debian-slim).

**Exit:** nodo A envía manifest WASM a nodo B; B ejecuta en sandbox,
devuelve resultado + métricas.

---

## Fase 4 · Cripto, identidad y biometría

**Objetivo:** todo el tráfico y el almacenamiento son post-cuántico seguros.

- **PQC híbrido desde día 1**: `pqcrypto-kyber` (KEM) + X25519 para
  handshake; firmas con `pqcrypto-dilithium` + Ed25519.
- Cifrado en reposo: AES-256-GCM con claves derivadas del KEM.
- Biometría: **solo local**, nunca por red. Usa la huella/cara como input
  a `argon2id` → unlock de la clave privada en keystore local
  (SecretService Linux / Keychain Mac / Credential Manager Windows /
  Android Keystore).
- `crypto` crate centraliza primitivas, expone `EncryptedBlob`,
  `SignedEnvelope`.
- Auditoría externa de criptografía antes de Fase 6.

**Exit:** un nodo cifra archivo, lo sube a IPFS, otro nodo solo puede
descifrar si tiene la clave (probado con biometría local en demo CLI).

---

## Fase 5 · Replicación, IPFS Cluster y caché P2P propia

**Objetivo:** reducir la dependencia crítica de IPFS y garantizar
disponibilidad.

- IPFS Cluster en nodos `storage` (super-peers): `replication_factor_min=3`,
  `max=10`.
- Políticas: contenido personal repl=1–3; proyectos comunitarios repl=5–10
  en nodos con alto uptime.
- **Content Cache P2P propio** (BitSwap-lite) para imágenes base y deltas
  — bypassa IPFS para hot-path de lanzamiento.
- Garbage collection con whitelist de pines locales + remotos validados.
- Swarm key privada opcional para sub-redes confiables.

**Exit:** caída controlada de N nodos no rompe disponibilidad de contenido
pinneado con `min=3`.

---

## Fase 6 · Reputación y economía de compartir (Beta)

**Objetivo:** el modelo "no-dinero, código abierto recíproco" es operativo.

- `reputation_engine` con CRDT (estado eventualmente consistente, sin
  blockchain todavía):
  - `Contribution Score = Σ(slot_horas_donadas × uptime) −
    Σ(slot_horas_consumidas)`.
  - Bonus por verificación exitosa, uptime, contribución a proyectos del
    pool comunitario.
  - "Deuda suave": cola más larga, no exclusión.
- Pool comunitario: % de slots reservados para proyectos open-source
  aprobados (governance ligera vía Web of Trust en F7).
- Verificación de jobs: nodos `validation` re-ejecutan muestras aleatorias
  y firman resultados.
- Leaderboards locales (no globales aún) para evitar gaming.

**Exit:** dos nodos con scores distintos compiten por un slot escaso — el
de mayor score gana.

---

## Fase 7 · Trust graph y escala

**Objetivo:** la red se gobierna sola a escala.

- Web of Trust: cada nodo firma a quién considera fiable; transitividad
  limitada (2º orden).
- Trust graph influye en: matching, pinning colaborativo, governance del
  pool comunitario, validación de jobs.
- Sharding de DHT por región/rol para latencia.
- Soporte multi-plataforma completo: Linux/macOS/Windows + Android
  Foreground Service.
- GPU compartida: MIG/MPS para nodos `ai`; scheduling fraccionado de VRAM
  (mínimo 2 GiB por slot AI).

**Exit:** red de 50+ nodos heterogéneos, jobs cruzan regiones, AI workloads
corren en slots VRAM.

---

## Validación de la jerarquía de nodos

**Viable, sí — con matices:**

- **Híbrido por defecto:** un nodo debe poder declarar múltiples roles
  simultáneos (`storage+compute` es lo común). Roles puros solo en
  super-peers grandes.
- **`validation` no debe ser élite cerrada:** rotación por reputación +
  muestreo aleatorio, si no se convierte en oligarquía y rompe el espíritu
  del proyecto.
- **Nodos `ai` con >2 GiB VRAM:** realista para GPUs consumer modernas
  (RTX 3060+, M-series con unified memory). Cuidado con la fragmentación
  de VRAM — usar MIG en NVIDIA datacenter o slicing por proceso en consumer.
- **Riesgo de centralización:** si los `storage`/`validation` se concentran
  en pocos super-peers, la red se vuelve federada de facto. Mitigación: el
  score de contribución debe premiar **diversidad geográfica + entrar como
  rol simple primero**, no solo magnitud.

---

## Mapeo con el roadmap original del documento

| Roadmap original | Fases de este plan |
|---|---|
| PoC: libp2p comparte archivos vía IPFS y anuncia recursos | F0 + F1 + F2 |
| MVP: ejecución básica de jobs WASM + cifrado | F3 + F4 |
| Beta: soporte multi-plataforma + incentivos básicos | F5 + F6 |
| Escala: Trust graph + permisos avanzados | F7 |
