#!/usr/bin/env bash
# Build the sibling thingblock-link repo and stage its binary, the editor-produced
# thingblock-resource pack, and the host-platform arduino-cli into src-tauri/ so
# Tauri can bundle them. macOS + Linux path; Windows uses stage-sidecar.ps1.
set -euo pipefail

DESKTOP_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
LINK_DIR="$(cd "$DESKTOP_DIR/../thingblock-link" && pwd)"
EDITOR_DIR="$(cd "$DESKTOP_DIR/../thingblock-editor" && pwd)"
TARGET_TRIPLE="$(rustc -vV | sed -n 's/^host: //p')"

BIN_DIR="$DESKTOP_DIR/src-tauri/binaries"
RES_DIR="$DESKTOP_DIR/src-tauri/resources"

echo "Building thingblock-link (release)..."
cargo build --release --manifest-path "$LINK_DIR/Cargo.toml"

echo "Staging sidecar binary for $TARGET_TRIPLE..."
mkdir -p "$BIN_DIR"
cp "$LINK_DIR/target/release/thingblock-link" "$BIN_DIR/thingblock-link-$TARGET_TRIPLE"

# The resource pack type-checks against @scratch/scratch-blocks (type-only imports,
# erased from the built output), so its tsc step needs that package's emitted
# declarations. A fresh `npm ci` installs but does not build workspace packages, so
# build scratch-blocks first to produce its dist/types — matching the editor
# monorepo's own build order.
echo "Building @scratch/scratch-blocks (resource pack's type declarations)..."
npm --prefix "$EDITOR_DIR/packages/scratch-blocks" run build

# The pack is produced by the editor workspace @thingblock/thingblock-resource;
# build it and stage its output so the bundled pack always tracks that single
# source of truth (the link repo's copy is a gitignored dev convenience).
echo "Building thingblock-resource pack..."
npm --prefix "$EDITOR_DIR/packages/thingblock-resource" run build

echo "Staging thingblock-resource pack..."
rm -rf "$RES_DIR/thingblock-resource"
mkdir -p "$RES_DIR"
cp -R "$EDITOR_DIR/packages/thingblock-resource/dist/thingblock-resource" "$RES_DIR/thingblock-resource"

# The link runs arduino-cli at a path we pass it (--arduino-cli); bundle the
# host-platform binary as a resource so the path resolves inside the installed
# app. It goes under bin/ (a directory resource, uniform across platforms) so the
# tauri.conf resource map needs no per-platform override. Only the host platform
# is staged here; cross-platform packaging is a CI concern.
echo "Staging host arduino-cli..."
case "$TARGET_TRIPLE" in
    *apple-darwin*) CLI_SRC="arduino-cli_mac_arm64/arduino-cli" ;;
    *)             CLI_SRC="arduino-cli_linux_64bit/arduino-cli" ;;
esac
mkdir -p "$RES_DIR/bin"
cp -p "$LINK_DIR/arduino-cli-binaries/$CLI_SRC" "$RES_DIR/bin/arduino-cli"

echo "Sidecar staged."
