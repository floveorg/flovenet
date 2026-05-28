# Plan: Flovenet Multiplataforma (Ubuntu, Android, Windows)

## Objetivo
Convertir Flovenet de una red Docker-centric a una **aplicación instalable** que corra nativamente en Ubuntu, Android y Windows, permitiendo a cualquier usuario:
- **Suscribirse** (registrarse/unirse a la red) desde cualquier plataforma.
- **Compartir capacidad de cómputo** (CPU, RAM, GPU, almacenamiento) sin importar su OS.

---

## Estado Actual del Proyecto

### ✅ Completado (Mayo 2026)

#### Fase 1: Fundación Multiplataforma
- **Refactor `resource_manager`**: `Platform` enum (`Linux`/`Windows`/`Macos`/`Android`), auto-detección por `#[cfg]` + env vars.
- **GPU detection multiplataforma**: Linux (/proc/driver/nvidia), Windows (nvidia-smi → wmic fallback), Android (JNI), env var override (`FLOVENET_GPU_VRAM_GB`, `FLOVENET_GPU_MODEL`).
- **`hardware_detector` trait**: Implementación separada por plataforma con `#[cfg]`.
- **Paths multiplataforma**: `default_data_dir()` y `default_cache_dir()` usando `dirs` crate (XDG en Linux, %APPDATA% en Windows, /data/data/ en Android).
- **`vm_runtime`**: Hardcoded `/tmp/flovenet/wasm_cache` reemplazado por `std::env::temp_dir().join("flovenet").join("wasm_cache")`.
- **Gestión de caché WASM**: directorio por plataforma usando `temp_dir()`.

#### Fase 2: Windows (parcial)
- **GPU detection vía nvidia-smi + wmic**: Detección de VRAM y modelo de GPU en Windows sin dependencia NVML.
- **Cross-compilación Windows configurada**: `.cargo/config.toml` con linker `x86_64-w64-mingw32-gcc`.
- **Pendiente**: Windows service wrapper, system tray, MSI installer.

#### Fase 3: Core Library (`flovenet-core`)
- **Crate `flovenet-core` creado**: `crate-type = ["lib", "cdylib"]` produce `.rlib` para desktop y `.so` para Android NDK.
- **JNI bridge**: Funciones exportadas `init`, `getPeerId`, `getResources`, `getPlatform`.
- **Cross-compilación Android configurada**: target `aarch64-linux-android` con linker `aarch64-linux-android21-clang`.
- **flovenet-core no incluye wasmtime**: Android actúa como relay sin ejecución WASM.
- **Dependencias mínimas**: solo `resource_manager`, `uuid`, `dirs`, `serde`, `tokio`. JNI condicional con `#[cfg(target_os = "android")]`.

#### Fase 4: Android (pendiente)
- **No iniciado**: Falta app Kotlin, NDK build, etc.

#### Fase 5: Ubuntu Packaging
- **Paquete .deb**: `deb-pkg/` con control, postinst, prerm, systemd services.
- **Systemd services**: `flovenet-daemon.service` (P2P node, puerto 9090), `flovenet-gateway.service` (GraphQL API, puerto 8080).
- **Hardening**: NoNewPrivileges, PrivateTmp, ProtectSystem, ProtectHome.
- **Snap**: `snap/snapcraft.yaml` con confinement strict.
- **Script `scripts/build-deb.sh`**: Build automatizado del .deb con man page.

#### Fase 6: Web Dashboard
- **Frontend**: Vite + React 19 + TypeScript + urql GraphQL.
- **Páginas**: Login, Register, Dashboard (stats + actions), Profile (search user + follow/unfollow), Feed (post creation + timeline), Network (node discovery).
- **Proxy**: Vite proxy `http://localhost:8080/graphql` para desarrollo.

#### Fase 7: CI/CD
- **Build matrix**: `check` (fmt + clippy + test), `cross-build` (linux + windows), `dashboard` (Node 20), `docker`, `audit`, `deny`.
- **Dockerfile**: Multi-stage con cargo-chef, binary renombrado a `flovenet`.

---

## Plan de Implementación (Siguientes Pasos)

### Hito A: Hacer Funcionar en Ubuntu (Inmediato)
- [x] Compilar el daemon: `cargo build --release --bin daemon`
- [x] Verificar estado: `./target/release/daemon status`
- [x] CLI funciona con subcomandos: daemon, api-gateway, share, run, status
- [ ] Probar daemon completo: `./target/release/daemon daemon --api-port 9090`
- [ ] Probar gateway: `./target/release/daemon api-gateway --port 8080`
- [ ] Probar web dashboard contra gateway local
- [ ] Buildear .deb: `./scripts/build-deb.sh`
- [ ] Instalar .deb localmente y verificar systemd services

### Hito B: Próximo Release (Julio 2026)
- [ ] **Windows**: Service wrapper (`windows-service` crate), system tray icon, MSI installer (WiX)
- [ ] **Android**: App Kotlin + NDK build de `flovenet-core`
- [ ] **Web Dashboard**: Embed en binario con `rust-embed`, embeber frontend en gateway
- [ ] **Ubuntu PPA**: Crear Launchpad PPA para actualizaciones automáticas
- [ ] **Documentación**: Guía de instalación completa para cada plataforma
- [ ] **Release pipeline**: CI/CD automatizada con GitHub Releases + artifacts

### Hito C: Features Post-MVP
- [ ] JWT auth middleware real (validación de tokens en todas las mutaciones)
- [ ] Bootstrap peers para Kademlia DHT funcional (mesh P2P real)
- [ ] WASM image registry integrado
- [ ] Dashboard multi-idioma (i18n)

---

## Guía de Instalación y Uso en Ubuntu

### Requisitos
- Ubuntu 22.04+ (x86_64)
- OpenSSL 3
- Opcional: GPU NVIDIA con drivers + nvidia-smi

### Opción 1: Desde Código Fuente

```bash
# 1. Clonar el repositorio
git clone https://github.com/flovenet/flovenet.git
cd flovenet

# 2. Compilar (requiere Rust 1.85+)
cargo build --release --bin daemon

# 3. Verificar instalación
./target/release/daemon --help
./target/release/daemon status

# 4. Iniciar nodo P2P
./target/release/daemon daemon --api-port 9090 --roles compute,storage

# 5. En otra terminal, iniciar gateway GraphQL
./target/release/daemon api-gateway --port 8080

# 6. Abrir web dashboard en el navegador:
#    http://localhost:8080
```

### Opción 2: .deb Package (Recomendado)

```bash
# 1. Buildear el .deb
./scripts/build-deb.sh

# 2. Instalar
sudo dpkg -i target/flovenet_*.deb

# 3. Verificar servicios
systemctl status flovenet-daemon
systemctl status flovenet-gateway

# 4. Ver logs
journalctl -u flovenet-daemon -f

# 5. Abrir dashboard
#    http://localhost:8080

# 6. Desinstalar
sudo dpkg -r flovenet
```

### Opción 3: Docker

```bash
# Build
docker build -t flovenet:latest .

# Run standalone
docker run --rm -p 9090:9090 -e RUST_LOG=info flovenet:latest daemon --api-port 9090

# Run with docker-compose (3 nodos + gateway)
docker compose up -d
```

### Uso del CLI

```bash
# Estado del nodo
daemon status

# Compartir recursos (info)
daemon share --role compute

# Ejecutar WASM localmente
daemon run --image <cid> --manifest <entrypoint>

# Opciones del daemon
daemon daemon --help
#  --port <PORT>          Puerto libp2p (default: 0 = auto)
#  --api-port <API_PORT>  Puerto HTTP metrics/API (default: 9090)
#  --roles <ROLES>        compute,storage,validation,ai,social
#  --swarm-key <PATH>     PSK para red privada
```

### Configuración

Variables de entorno:

| Variable | Descripción | Default |
|----------|-------------|---------|
| `RUST_LOG` | Nivel de logging | `info` |
| `FLOVENET_DATA_DIR` | Directorio de datos | `~/.local/share/flovenet` |
| `FLOVENET_CACHE_DIR` | Directorio de caché | `~/.cache/flovenet` |
| `FLOVENET_PLATFORM` | Forzar plataforma | auto-detect |
| `FLOVENET_GPU_VRAM_GB` | VRAM GPU (GB) | auto-detect |
| `FLOVENET_GPU_MODEL` | Modelo GPU | auto-detect |

---

## Arquitectura Actual

```
                    ┌─────────────────────┐
                    │   Web Dashboard      │
                    │  (Vite + React)      │
                    └─────────┬───────────┘
                              │ HTTP (GraphQL)
                    ┌─────────▼───────────┐
                    │  API Gateway         │
                    │  (axum + async-graphql)│
                    │  Puerto 8080         │
                    └─────────┬───────────┘
                              │
          ┌───────────────────┼───────────────────┐
          │                   │                   │
  ┌───────▼───────┐   ┌──────▼──────┐   ┌──────▼───────┐
  │  Nodo P2P #1   │   │ Nodo P2P #2 │   │  Nodo P2P #3  │
  │  (compute)      │   │ (compute)   │   │  (storage)     │
  │  Puerto 9091    │   │ Puerto 9092 │   │  Puerto 9093   │
  └───────┬───────┘   └──────┬──────┘   └──────┬───────┘
          │                   │                   │
          └───────────────────┼───────────────────┘
                              │ libp2p (Kademlia + Gossipsub)
                              ▼
                    ┌─────────────────────┐
                    │   Red P2P Global     │
                    │  (DHT, reputación,   │
                    │   trust graph, jobs) │
                    └─────────────────────┘
```

### Componentes del Daemon

| Crate | Propósito | Multiplataforma |
|-------|-----------|-----------------|
| `daemon` | Binario principal (CLI + networking + HTTP) | ✅ Linux/Win |
| `resource_manager` | Detección de recursos (CPU/RAM/GPU/disk) | ✅ Linux/Win/Android |
| `vm_runtime` | Ejecución WASM (wasmtime) | ✅ Linux/Win, ❌ Android |
| `flovenet-core` | Librería compartida (desktop .rlib, Android .so) | ✅ Linux/Win/Android |
| `cli` | Definición de CLI con clap | ✅ Linux/Win |
| `graphql_api` | Gateway GraphQL con async-graphql | ✅ Linux/Win, ❌ Android |
| `market_protocol` | Protocolo de ofertas/jobs | ✅ Linux/Win |
| `reputation_engine` | Sistema de reputación y scoring | ✅ Linux/Win/Android |
| `social_protocol` | Red social (posts, follows, feed) | ✅ Linux/Win/Android |
| `trust_graph` | Grafo de confianza descentralizado | ✅ Linux/Win/Android |
| `identity` | Gestión de identidad (Peer ID, claves) | ✅ Linux/Win/Android |
| `crypto` | Criptografía (ChaCha20-Poly1305, Argon2, ed25519) | ✅ Linux/Win/Android |
| `ipfs_layer` | Almacenamiento IPFS-like | ✅ Linux/Win |
| `storage` | Base de datos KV local (sled) | ✅ Linux/Win |
| `scheduler` | Asignación de slots y planificación | ✅ Linux/Win |
| `p2p_cache` | Caché distribuida P2P (CRDT) | ✅ Linux/Win/Android |

---

## Notas Técnicas

### Detección de Plataforma
```rust
// resource_manager/src/lib.rs
pub enum Platform { Linux, Windows, Macos, Android }

impl Platform {
    pub fn detect() -> Self {
        #[cfg(target_os = "android")]
        { Self::Android }
        #[cfg(target_os = "windows")]
        { Self::Windows }
        #[cfg(target_os = "macos")]
        { Self::Macos }
        #[cfg(target_os = "linux")]
        {
            match std::env::var("FLOVENET_PLATFORM").as_deref() {
                Ok("android") => Self::Android,
                _ => Self::Linux,
            }
        }
    }
}
```

### Cross-compilación
```bash
# Linux → Windows
rustup target add x86_64-pc-windows-msvc
sudo apt install mingw-w64
cargo build --release --target x86_64-pc-windows-msvc -p daemon

# Linux → Android (solo flovenet-core)
rustup target add aarch64-linux-android
cargo install cargo-ndk
cargo ndk -t arm64-v8a build --release -p flovenet-core
```

### Tests
```bash
# Todos los tests (excepto test_harness que requiere Docker)
cargo test --workspace --exclude test_harness --exclude test_reporter
# Resultado actual: 178 tests, 0 fallos, 0 warnings
```

---

## Estructura de Directorios

```
flovenet/
├── .cargo/config.toml         # Cross-compilación targets
├── .github/workflows/ci.yml   # CI/CD build matrix
├── Cargo.toml                 # Workspace con 18 crates
├── Dockerfile                 # Multi-stage build
├── docker-compose.yml         # 3 nodos + gateway
├── plan-crossplatform.md      # Este archivo
├── deb-pkg/                   # Paquete .deb
│   ├── DEBIAN/                #   control, postinst, prerm
│   └── lib/systemd/system/    #   *.service
├── snap/snapcraft.yaml        # Snap package
├── scripts/
│   ├── build-deb.sh           # Builder del .deb
│   └── build-dashboard.sh     # Builder del frontend
├── web-dashboard/             # Frontend React
│   ├── src/pages/             #   6 páginas
│   ├── src/graphql/           #   Queries/mutations
│   └── vite.config.ts         #   Proxy a localhost:8080
├── flovenet-core/             # Librería compartida
│   ├── src/lib.rs             #   API pública + JNI bridge
│   └── Cargo.toml             #   crate-type = ["lib", "cdylib"]
├── resource_manager/          # Recursos multiplataforma
│   ├── src/lib.rs             #   Platform, NodeResources
│   ├── src/gpu.rs             #   GPU detection
│   └── src/hardware_detector.rs # Trait + #[cfg] impls
├── daemon/src/main.rs         # Binario principal
├── vm_runtime/                # WASM runtime
├── graphql_api/               # Gateway GraphQL
├── cli/                       # CLI con clap
├── market_protocol/           # Protocolo de mercado
├── reputation_engine/         # Sistema de reputación
├── social_protocol/           # Red social P2P
├── trust_graph/               # Grafo de confianza
├── identity/                  # Identidad y claves
├── crypto/                    # Criptografía
├── ipfs_layer/                # Almacenamiento IPFS
├── storage/                   # Base de datos KV
├── scheduler/                 # Planificador
└── p2p_cache/                 # Caché distribuida
```
