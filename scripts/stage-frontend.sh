#!/usr/bin/env bash
# Stage the built editor plus the thingblock-resource pack into frontend/, the
# directory tauri.conf.json points frontendDist at. Runs after build:editor (it
# consumes that build's output). macOS + Linux path; Windows uses
# stage-frontend.ps1.
#
# The pack ships inside the frontend so the editor reads it from its own origin.
# On Windows the webview is Chromium, which gates cross-address-space requests
# into loopback: reaching the link's own /resources route over HTTP is blocked
# there. The pack is staged a second time under src-tauri/resources/ for the link
# itself (compile lib resolution, a filesystem read) — different consumers, same
# build output.
set -euo pipefail

DESKTOP_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
EDITOR_DIR="$(cd "$DESKTOP_DIR/../thingblock-editor" && pwd)"

FRONTEND_DIR="$DESKTOP_DIR/frontend"
EDITOR_BUILD="$EDITOR_DIR/packages/scratch-gui/build"
RESOURCE_PACK="$EDITOR_DIR/packages/thingblock-resource/dist/thingblock-resource"

if [ ! -d "$EDITOR_BUILD" ]; then
    echo "stage-frontend: no editor build at $EDITOR_BUILD; run build:editor first" >&2
    exit 1
fi
if [ ! -d "$RESOURCE_PACK" ]; then
    echo "stage-frontend: no resource pack at $RESOURCE_PACK; run stage:sidecar first" >&2
    exit 1
fi

echo "Staging editor build..."
rm -rf "$FRONTEND_DIR"
cp -R "$EDITOR_BUILD" "$FRONTEND_DIR"

echo "Staging thingblock-resource pack into the frontend..."
cp -R "$RESOURCE_PACK" "$FRONTEND_DIR/thingblock-resource"

# The editor reads its pack base from this global (see the editor's
# link-controller.js). Injected into the staged copy rather than built into the
# editor itself, so the editor artifact stays host-agnostic and reusable against a
# cloud backend. The inline script runs before the deferred bundle.
echo "Injecting the resource base..."
INDEX="$FRONTEND_DIR/index.html"
if ! grep -q '</head>' "$INDEX"; then
    echo "stage-frontend: no </head> in $INDEX; cannot inject the resource base" >&2
    exit 1
fi
sed 's|</head>|<script>globalThis.__THINGBLOCK_RESOURCE_BASE__="/thingblock-resource";</script></head>|' \
    "$INDEX" > "$INDEX.tmp"
mv "$INDEX.tmp" "$INDEX"

echo "Frontend staged."
