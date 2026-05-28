#!/usr/bin/env bash
# Test docker compose: 3 nodos + gateway, verifica peers descubiertos.
set -uo pipefail

cd "$(dirname "$0")/.."
LOGS="/home/kdeneon/flovenet/.work/docker-test"
DURATION_SECS="${DURATION_SECS:-30}"

rm -rf "$LOGS"
mkdir -p "$LOGS"

echo "→ bringing up compose stack..."
docker compose up -d
echo "→ stack up. waiting ${DURATION_SECS}s for peer formation..."
sleep "$DURATION_SECS"

echo
echo "=== Capturando logs ==="
for svc in node1 node2 node3 gateway; do
  docker compose logs --no-color "$svc" > "$LOGS/$svc.log" 2>&1
done

echo
echo "=== Resumen por contenedor ==="
for svc in node1 node2 node3 gateway; do
  log="$LOGS/$svc.log"
  peer_id=$(grep -oE "Peer ID: [^,]+" "$log" | head -1 | sed 's/Peer ID: //')
  listen=$(grep -oE "Listening on [^ ]+" "$log" | head -3 | tr '\n' ' ')
  mdns_disc=$(grep -c "mDNS discovered" "$log" 2>/dev/null || echo 0)
  connected=$(grep -c "Connected to" "$log" 2>/dev/null || echo 0)
  echo "$svc:"
  echo "  PeerID: $peer_id"
  echo "  Listen: $listen"
  echo "  mDNS_discovered: $mdns_disc"
  echo "  Connected_events: $connected"
done

echo
echo "=== Peer-to-peer matrix ==="
# Para cada nodo, ¿qué peers vio?
for svc in node1 node2 node3 gateway; do
  log="$LOGS/$svc.log"
  others=$(grep -oE "(mDNS discovered|Connected to) 12D3[A-Za-z0-9]+" "$log" 2>/dev/null \
           | sed -E 's/(mDNS discovered|Connected to) //' | sort -u)
  count=$(echo "$others" | grep -c "^12D3" || echo 0)
  echo "$svc vio $count peer(s) distintos:"
  echo "$others" | sed 's/^/    /' | head -6
done

# Criterio: cada uno de los 4 servicios debe haber visto >=1 peer
echo
pass=0
for svc in node1 node2 node3 gateway; do
  log="$LOGS/$svc.log"
  c=$(grep -oE "(mDNS discovered|Connected to) 12D3[A-Za-z0-9]+" "$log" 2>/dev/null \
      | sed -E 's/(mDNS discovered|Connected to) //' | sort -u | grep -c "^12D3" || echo 0)
  if [[ "$c" -ge 1 ]]; then pass=$((pass+1)); fi
done

echo
if [[ "$pass" -eq 4 ]]; then
  echo "✅ PASS: los 4 servicios vieron al menos 1 peer"
  RC=0
else
  echo "❌ FAIL: solo $pass/4 servicios vieron peers"
  RC=1
fi

echo
echo "→ tearing down..."
docker compose down 2>&1 | tail -5
exit $RC
