# Plan: Web simple de estado y recursos

## Objetivo

Que el binario `flovenet` sirva una web simple que muestre el estado del nodo (CPU/RAM/disco/GPU/uptime/roles) y permita activar/desactivar el compartir recursos desde el navegador.

## Arquitectura

El daemon ya corre un servidor axum (puerto `--api-port`, default 9090) con `/metrics` y `/health`. Se extiende este mismo servidor:

```
daemon (axum :9090)
  ├── /metrics        (ya existe)
  ├── /health         (ya existe)
  ├── /api/status     (nuevo) → JSON con NodeResources + roles + peer_id
  ├── /api/share      (nuevo, POST) → activa/desactiva sharing
  ├── /               (nuevo) → sirve index.html embebido
  └── /assets/*       (nuevo) → sirve CSS/JS embebidos
```

La web es HTML plano (sin React, sin build step) embebido en el binario con `rust-embed`. La página usa `fetch()` contra `/api/status` y `/api/share`.

## Pasos

### 1. Crear directorio `daemon/web/` con el frontend

- `daemon/web/index.html` — página simple con:
  - Tarjetas de recursos: CPU, RAM, Disco, GPU, Uptime
  - Roles activos
  - Peer ID
  - Botón "Compartir recursos" on/off
  - Polling cada 5s a `/api/status`
- `daemon/web/style.css` — estilo minimalista (oscuro, legible)
- `daemon/web/app.js` — lógica JS: fetch, render, POST toggle

### 2. Agregar endpoints REST en `daemon/src/main.rs`

- **`GET /api/status`** → devuelve `serde_json::Value` con:
  ```json
  {
    "peer_id": "...",
    "roles": ["compute", "storage"],
    "sharing": true,
    "resources": { "cpu_cores": 8, "ram_available_gb": 15.2, ... }
  }
  ```
- **`POST /api/share`** → body `{ "enabled": true/false }` → persiste en un `Arc<AtomicBool>` compartido

Se necesita mover las variables compartidas (roles, peer_id, NodeResources) a un `AppState` de axum, o usar `Arc`s.

### 3. Embeber los archivos web con `rust-embed`

Agregar dependencia `rust-embed` al `daemon/Cargo.toml`. Usar `Embed` derive para embeber `daemon/web/*`.

Servir con axum: `Router::new().nest_service("/", get_service(ServeDir::new("...")))` o con un handler que devuelva el asset embebido.

### 4. Integrar en el build de Docker

El Dockerfile ya construye el binario. Con el contenido embebido no se necesita build separado.

### 5. (Opcional) Integrar web-dashboard React

Si después se quiere, el proxy de Vite en `web-dashboard/vite.config.ts` puede apuntar a los endpoints `/api/*` del daemon (además de `/graphql` → gateway), y el frontend React puede consumir los mismos datos.

## Archivos a modificar

| Archivo | Cambio |
|---------|--------|
| `daemon/Cargo.toml` | + `rust-embed` |
| `Cargo.toml` (workspace) | + `rust-embed` en workspace.dependencies |
| `daemon/src/main.rs` | + AppState, + endpoints `/api/status`, `/api/share`, + static file serving |
| `daemon/web/index.html` | crear |
| `daemon/web/style.css` | crear |
| `daemon/web/app.js` | crear |

## No hace falta

- GraphQL — los datos de estado son locales al nodo, no necesitan GraphQL
- React build — HTML plano es más simple y va embebido en el binario
- Base de datos — estado en memoria
