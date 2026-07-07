# thingblock-desktop

The Tauri (Rust) desktop shell for ThingBlock. It packages two independently released
projects into one installable app:

- **[thingblock-editor](../thingblock-editor)** — the web editor UI, rendered in the Tauri webview.
- **[thingblock-link](../thingblock-link)** — the local Rust helper (WebSocket server +
  `arduino-cli` gRPC client) that compiles and flashes firmware. It runs as a bundled
  **sidecar process**.

The webview loads the editor and talks to the link helper over WebSocket on
`ws://localhost:3030` — the same contract the editor already uses in browser mode. This shell
adds no new protocol; it bundles both halves, spawns the sidecar on launch, and gives users a
single double-click install.

For the full architecture and packaging contract, see
[`.agents/docs/01-07_01.architecture.md`](.agents/docs/01-07_01.architecture.md) and
[`AGENTS.md`](AGENTS.md).

## Layout

This repo expects `thingblock-editor` and `thingblock-link` checked out as **siblings** on disk:

```text
<parent>/
├── thingblock-editor/   # web UI (npm monorepo)
├── thingblock-link/     # Rust WS helper binary (built as a sidecar)
└── thingblock-desktop/  # this repo — Tauri shell
```

Neither sibling's source is vendored or submoduled here; this shell reaches out to their built
artifacts (see the architecture doc for how).

## Prerequisites

- Rust (edition 2024 toolchain) + Cargo
- Node.js + npm
- The `thingblock-editor` and `thingblock-link` repos cloned as siblings of this one

## Setup

```sh
npm install
npm --prefix ../thingblock-editor/packages/scratch-gui install
```

## Run in dev

```sh
npm run tauri dev
```

This builds `thingblock-link` in release mode, stages its binary/resources into
`src-tauri/`, starts the editor's webpack dev server (`http://localhost:8601`), and opens the
Tauri window against it. The bundled `thingblock-link` sidecar is spawned automatically and
listens on `ws://localhost:3030`.

The first staging run also seeds the arduino config bundle (downloads the `arduino:avr` core,
~50 MB); subsequent runs skip it.

On a low-RAM machine the link's release build can be OOM-killed at full parallelism; prefix
commands with `CARGO_BUILD_JOBS=2` to cap it.

## Build an installer

```sh
npm run dist         # production installer
npm run dist:debug   # faster unoptimized bundle, for local testing
```

Installers land in `src-tauri/target/release/bundle/<format>/` (Linux: `deb`, `rpm`).
Only the host platform is staged; cross-platform packaging is a CI concern.

## Lint / format

```sh
cargo fmt --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
```

## Contributing

See [`AGENTS.md`](AGENTS.md) for conventions, the link WS contract, and the checklist to run
before submitting changes.
