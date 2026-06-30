#!/usr/bin/env bash
# Build the sibling thingblock-link repo and stage its binary, resource pack, and
# the host-platform arduino-cli into src-tauri/ so Tauri can bundle them.
set -euo pipefail

DESKTOP_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
LINK_DIR="$(cd "$DESKTOP_DIR/../thingblock-link" && pwd)"
TARGET_TRIPLE="$(rustc -vV | sed -n 's/^host: //p')"

BIN_DIR="$DESKTOP_DIR/src-tauri/binaries"
RES_DIR="$DESKTOP_DIR/src-tauri/resources"

echo "Building thingblock-link (release)..."
cargo build --release --manifest-path "$LINK_DIR/Cargo.toml"

echo "Staging sidecar binary for $TARGET_TRIPLE..."
mkdir -p "$BIN_DIR"
cp "$LINK_DIR/target/release/thingblock-link" "$BIN_DIR/thingblock-link-$TARGET_TRIPLE"

echo "Staging thingblock-resource pack..."
rm -rf "$RES_DIR/thingblock-resource"
mkdir -p "$RES_DIR"
cp -R "$LINK_DIR/thingblock-resource" "$RES_DIR/thingblock-resource"

# The link runs arduino-cli at a path we pass it (--arduino-cli); bundle the
# host-platform binary as a resource so the path resolves inside the installed
# app. Only the host platform is staged here; cross-platform packaging is a CI
# concern (Windows would also need the `.exe` dest wired into tauri.conf/lib.rs).
echo "Staging host arduino-cli..."
case "$TARGET_TRIPLE" in
    *windows*)     CLI_SRC="arduino-cli_win_64bit/arduino-cli.exe" ;;
    *apple-darwin*) CLI_SRC="arduino-cli_mac_arm64/arduino-cli" ;;
    *)             CLI_SRC="arduino-cli_linux_64bit/arduino-cli" ;;
esac
cp -p "$LINK_DIR/arduino-cli-binaries/$CLI_SRC" "$RES_DIR/arduino-cli"

echo "Sidecar staged."
