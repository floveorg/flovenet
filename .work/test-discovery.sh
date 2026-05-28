#!/usr/bin/env bash
# Test pragmático: lanza 3 daemons locales en puertos distintos, captura logs,
# y verifica que se descubren por mDNS y forman conexiones.
#
# Pre: target/release/daemon ya compilado.

set -uo pipefail

BIN="/home/kdeneon/flovenet/target/release/daemon"
LOGS="/home/kdeneon/flovenet/.work/discovery-test"
DURATION_SECS="${DURATION_SECS:-25}"

if [[ ! -x "$BIN" ]]; then
  echo "FATAL: $BIN no existe. Build primero." >&2
  exit 1
fi

rm -rf "$LOGS"
mkdir -p "$LOGS"

cleanup() {
  echo "→ killing daemons..."
  for pid in $(cat "$LOGS"/*.pid 2>/dev/null); do
    kill "$pid" 2>/dev/null || true
  done
  sleep 1
  for pid in $(cat "$LOGS"/*.pid 2>/dev/null); do
    kill -9 "$pid" 2>/dev/null || true
  done
}
trap cleanup EXIT INT TERM

echo "→ launching 3 daemons on dynamic libp2p ports, api 9091/9092/9093"
RUST_LOG=info,libp2p_mdns=debug,libp2p_kad=info "$BIN" daemon \
  --port 0 --api-port 9091 --roles compute,storage \
  > "$LOGS/node1.log" 2>&1 &
echo $! > "$LOGS/node1.pid"

RUST_LOG=info,libp2p_mdns=debug,libp2p_kad=info "$BIN" daemon \
  --port 0 --api-port 9092 --roles compute \
  > "$LOGS/node2.log" 2>&1 &
echo $! > "$LOGS/node2.pid"

RUST_LOG=info,libp2p_mdns=debug,libp2p_kad=info "$BIN" daemon \
  --port 0 --api-port 9093 --roles storage \
  > "$LOGS/node3.log" 2>&1 &
echo $! > "$LOGS/node3.pid"

echo "→ letting them run for ${DURATION_SECS}s..."
sleep "$DURATION_SECS"

echo
echo "=== Resultados ==="
for n in node1 node2 node3; do
  log="$LOGS/$n.log"
  peer_id=$(grep -oE "Peer ID: [^,]+" "$log" | head -1 | sed 's/Peer ID: //')
  discovered=$(grep -c "mDNS discovered" "$log" 2>/dev/null || echo 0)
  connected=$(grep -c "Connected to" "$log" 2>/dev/null || echo 0)
  echo "$n  peer=$peer_id  mDNS_discovered=$discovered  connections=$connected"
done

echo
echo "=== Inter-pares ==="
# Cruce: ¿cada nodo vio a los otros dos?
for n in node1 node2 node3; do
  log="$LOGS/$n.log"
  echo "-- $n vio:"
  grep -oE "mDNS discovered 12D3[A-Za-z0-9]+" "$log" 2>/dev/null | sort -u | sed 's/^/    /'
done

# Criterio de éxito: cada nodo descubrió al menos 2 peers (los otros dos)
pass=0
for n in node1 node2 node3; do
  log="$LOGS/$n.log"
  count=$(grep -oE "mDNS discovered 12D3[A-Za-z0-9]+" "$log" 2>/dev/null | sort -u | wc -l)
  if [[ "$count" -ge 2 ]]; then
    pass=$((pass+1))
  fi
done

echo
if [[ "$pass" -eq 3 ]]; then
  echo "✅ PASS: los 3 nodos se descubrieron entre sí por mDNS"
  exit 0
else
  echo "❌ FAIL: sólo $pass/3 nodos descubrieron ≥2 peers"
  echo "  logs en $LOGS/"
  exit 1
fi
