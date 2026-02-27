#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
DIST_DIR="$PROJECT_DIR/dist"

# Extract version from Cargo.toml
VERSION=$(grep '^version' "$PROJECT_DIR/Cargo.toml" | head -1 | sed 's/.*"\(.*\)".*/\1/')
BINARY="$PROJECT_DIR/target/release/termdemo"
ARCH="x86_64"

# Track temp dirs for cleanup
CLEANUP_DIRS=()
cleanup() { rm -rf "${CLEANUP_DIRS[@]}"; }
trap cleanup EXIT

echo "=== Building termdemo $VERSION packages ==="
echo ""

# Create dist directory
rm -rf "$DIST_DIR"
mkdir -p "$DIST_DIR"

# ── Step 1: Build release binary ──────────────────────────────────────────────
echo "── Building release binary..."
cd "$PROJECT_DIR"
cargo build --release
strip "$BINARY"
echo "   Binary: $BINARY ($(du -h "$BINARY" | cut -f1))"
echo ""

# ── Step 2: Build .deb package ────────────────────────────────────────────────
echo "── Building .deb package..."
DEB_NAME="termdemo_${VERSION}_amd64"
DEB_DIR=$(mktemp -d)
CLEANUP_DIRS+=("$DEB_DIR")

mkdir -p "$DEB_DIR/usr/bin"
mkdir -p "$DEB_DIR/usr/share/doc/termdemo"
mkdir -p "$DEB_DIR/DEBIAN"

cp "$BINARY" "$DEB_DIR/usr/bin/termdemo"
cp "$SCRIPT_DIR/deb/copyright" "$DEB_DIR/usr/share/doc/termdemo/copyright"

# Generate control file with actual version
sed "s/VERSION_PLACEHOLDER/$VERSION/" "$SCRIPT_DIR/deb/control" > "$DEB_DIR/DEBIAN/control"

dpkg-deb --build --root-owner-group "$DEB_DIR" "$DIST_DIR/${DEB_NAME}.deb"
echo "   Output: dist/${DEB_NAME}.deb"
echo ""

# ── Step 3: Build .pkg.tar.zst package (Arch) ────────────────────────────────
echo "── Building .pkg.tar.zst package..."
if command -v makepkg &>/dev/null && command -v bsdtar &>/dev/null; then
    PKG_BUILD_DIR=$(mktemp -d)
    CLEANUP_DIRS+=("$PKG_BUILD_DIR")

    # Generate PKGBUILD with actual version and absolute paths
    sed \
        -e "s/VERSION_PLACEHOLDER/$VERSION/" \
        -e "s|\.\./\.\./\.\./target/release/termdemo|$PROJECT_DIR/target/release/termdemo|" \
        -e "s|\.\./\.\./\.\./LICENSE|$PROJECT_DIR/LICENSE|" \
        "$SCRIPT_DIR/arch/PKGBUILD" > "$PKG_BUILD_DIR/PKGBUILD"

    cd "$PKG_BUILD_DIR"
    if PKGDEST="$DIST_DIR" makepkg -f --nodeps 2>&1 | tail -5; then
        echo "   Output: dist/termdemo-${VERSION}-1-${ARCH}.pkg.tar.zst"
    else
        echo "   FAIL: makepkg exited with an error"
    fi
elif command -v makepkg &>/dev/null; then
    echo "   SKIP: bsdtar not found (install libarchive-tools)"
else
    echo "   SKIP: makepkg not found (install base-devel on Arch)"
fi
echo ""

# ── Step 4: Build .rpm package ────────────────────────────────────────────────
echo "── Building .rpm package..."
cd "$PROJECT_DIR"
if command -v cargo-generate-rpm &>/dev/null; then
    cargo generate-rpm
    RPM_FILE=$(find target/generate-rpm -name '*.rpm' -type f | head -1)
    if [ -n "$RPM_FILE" ]; then
        cp "$RPM_FILE" "$DIST_DIR/"
        echo "   Output: dist/$(basename "$RPM_FILE")"
    fi
else
    echo "   SKIP: cargo-generate-rpm not found"
    echo "   Install with: cargo install cargo-generate-rpm"
fi
echo ""

# ── Step 5: Build portable .tar.gz ───────────────────────────────────────────
echo "── Building portable .tar.gz..."
TARBALL_NAME="termdemo-${VERSION}-linux-${ARCH}"
TARBALL_DIR=$(mktemp -d)
CLEANUP_DIRS+=("$TARBALL_DIR")

mkdir -p "$TARBALL_DIR/$TARBALL_NAME"
cp "$BINARY" "$TARBALL_DIR/$TARBALL_NAME/termdemo"
cp "$PROJECT_DIR/LICENSE" "$TARBALL_DIR/$TARBALL_NAME/"

cat > "$TARBALL_DIR/$TARBALL_NAME/README" <<'READMEEOF'
termdemo - A terminal demo engine with visual effects and transitions

USAGE
    ./termdemo              Run the default demo sequence
    ./termdemo --interactive    Run in interactive mode

INSTALL
    Copy the binary to a directory on your PATH:
        sudo cp termdemo /usr/local/bin/

LICENSE
    MIT License - see LICENSE file for details.
READMEEOF

tar czf "$DIST_DIR/${TARBALL_NAME}.tar.gz" -C "$TARBALL_DIR" "$TARBALL_NAME"
echo "   Output: dist/${TARBALL_NAME}.tar.gz"
echo ""

# ── Summary ───────────────────────────────────────────────────────────────────
echo "=== Package build complete ==="
echo ""
ls -lh "$DIST_DIR/"
