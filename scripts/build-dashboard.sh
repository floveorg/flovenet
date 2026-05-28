#!/bin/bash
set -euo pipefail

# Build the Flovenet Web Dashboard
# Requires: Node.js 20+

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
DASHBOARD_DIR="$(dirname "$SCRIPT_DIR")/web-dashboard"
OUTPUT_DIR="/tmp/flovenet-dashboard-build"

echo "==> Installing dependencies..."
cd "$DASHBOARD_DIR"
npm install

echo "==> Building dashboard..."
npx vite build --outDir "$OUTPUT_DIR"

echo "==> Done! Dashboard built to: $OUTPUT_DIR"
echo "    Serve with: cd $OUTPUT_DIR && npx serve ."
echo "    Or embed in flovenet gateway binary with rust-embed"
