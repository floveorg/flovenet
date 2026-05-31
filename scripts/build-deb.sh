#!/bin/bash
set -euo pipefail

# Build Flovenet .deb package
# Usage: ./scripts/build-deb.sh [version] [arch]
#   version: from Cargo.toml by default
#   arch:    amd64 (default) | arm64

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
VERSION="${1:-$(cd "$PROJECT_DIR" && grep '^version =' daemon/Cargo.toml | head -1 | cut -d'"' -f2)}"
ARCH="${2:-amd64}"

case "$ARCH" in
  amd64) RUST_TARGET="x86_64-unknown-linux-gnu"; DEB_ARCH="amd64" ;;
  arm64) RUST_TARGET="aarch64-unknown-linux-gnu"; DEB_ARCH="arm64" ;;
  *) echo "Unsupported arch: $ARCH (use amd64 or arm64)"; exit 1 ;;
esac

DEB_DIR="$PROJECT_DIR/deb-pkg"
BUILD_DIR="$PROJECT_DIR/target/deb-build"

echo "==> Building flovenet v$VERSION .deb package ($DEB_ARCH)"

# Step 1: Build the release binary
echo "==> Compiling flovenet daemon (target: $RUST_TARGET)..."
cd "$PROJECT_DIR"
cargo build --release --bin daemon --target "$RUST_TARGET"

# Step 2: Prepare deb directory
echo "==> Preparing package structure..."
rm -rf "$BUILD_DIR"
PKG_DIR="$BUILD_DIR/flovenet_${VERSION}_${DEB_ARCH}"
mkdir -p "$PKG_DIR/DEBIAN"
mkdir -p "$PKG_DIR/usr/bin"
mkdir -p "$PKG_DIR/usr/share/doc/flovenet"
mkdir -p "$PKG_DIR/usr/share/man/man1"
mkdir -p "$PKG_DIR/lib/systemd/system"

# Step 3: Copy binary
cp "target/$RUST_TARGET/release/daemon" "$PKG_DIR/usr/bin/flovenet"
strip "$PKG_DIR/usr/bin/flovenet" 2>/dev/null || true

# Step 4: Copy control files
cp "$DEB_DIR/DEBIAN/control" "$PKG_DIR/DEBIAN/"
cp "$DEB_DIR/DEBIAN/postinst" "$PKG_DIR/DEBIAN/"
cp "$DEB_DIR/DEBIAN/prerm" "$PKG_DIR/DEBIAN/"
chmod 755 "$PKG_DIR/DEBIAN/postinst" "$PKG_DIR/DEBIAN/prerm"

# Step 5: Copy systemd services
cp "$DEB_DIR/lib/systemd/system/"*.service "$PKG_DIR/lib/systemd/system/"

# Step 6: Copy docs
cp "$PROJECT_DIR/README.md" "$PKG_DIR/usr/share/doc/flovenet/"

# Step 7: Generate man page
cat > "$PKG_DIR/usr/share/man/man1/flovenet.1" << 'MANEOF'
.TH FLOVENET 1 "2026" "Flovenet" "User Commands"
.SH NAME
flovenet \- decentralized compute sharing network
.SH SYNOPSIS
.B flovenet
[\fIdaemon\fR|\fIapi-gateway\fR|\fIshare\fR|\fIrun\fR|\fIstatus\fR]
.SH DESCRIPTION
Flovenet is a P2P network for sharing computing resources.
.SH COMMANDS
.TP
.B daemon
Start a P2P node with specified roles.
.TP
.B api-gateway
Start the GraphQL API gateway.
.TP
.B share
Display resource information for a role.
.TP
.B run
Execute a WASM job locally.
.TP
.B status
Display node resource information.
MANEOF
gzip -9 "$PKG_DIR/usr/share/man/man1/flovenet.1"

# Step 8: Update metadata
sed -i "s/Version: .*/Version: $VERSION/" "$PKG_DIR/DEBIAN/control"
sed -i "s/Architecture: .*/Architecture: $DEB_ARCH/" "$PKG_DIR/DEBIAN/control"

# Step 9: Build .deb
echo "==> Building .deb package..."
fakeroot dpkg-deb --build "$PKG_DIR" "$PROJECT_DIR/target/flovenet_${VERSION}_${DEB_ARCH}.deb"

echo "==> Done!"
echo "    Package: $PROJECT_DIR/target/flovenet_${VERSION}_${DEB_ARCH}.deb"
echo "    Install: sudo dpkg -i target/flovenet_${VERSION}_${DEB_ARCH}.deb"
