# Agent Guide: thingblock-desktop

## What this is

`thingblock-desktop` is the Tauri (Rust) desktop shell that packages two sibling repos into one installable app:

- `thingblock-editor` — the web editor, rendered in the webview.
- `thingblock-link` — the local helper, bundled as a sidecar process.

The shell holds window setup, sidecar lifecycle and bundler config. It does not vendor editor or link source and does not use submodules; it stages their built artifacts.

```text
<parent>/
├── thingblock-editor/
├── thingblock-link/
└── thingblock-desktop/   # this repo
```

## Layout

```text
src-tauri/                Tauri app (Rust)
├── binaries/             staged link sidecar (externalBin)
├── resources/            staged resource pack, arduino-cli, arduino config seed
├── tauri.conf.json       base config (Linux bundles)
├── tauri.macos.conf.json
└── tauri.windows.conf.json
frontend/                 staged editor build (gitignored)
scripts/
├── stage-sidecar.{sh,ps1}    builds and stages the link sidecar and resources
└── stage-frontend.{sh,ps1}   stages the editor build into frontend/
```

## Contracts with the siblings

- The sidecar is spawned with `--port 3030`; the webview connects to `ws://localhost:3030`. A port change must happen on both sides.
- The shell passes the sidecar `--resource-root`, `--arduino-cli` and `--config-dir`, resolved from the bundled resources.
- Dev loads the editor dev server at `http://localhost:8601`; release loads `frontend/`.

## Build, run, lint

```sh
npm run tauri dev        # run against the editor dev server + link sidecar
npm run dist             # production installer
npm run dist:debug       # unoptimized bundle for local testing
cargo fmt --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
```

`dev` and `dist` stage the sidecar and build the editor automatically. Installers land in `src-tauri/target/release/bundle/<format>/`. Only the host platform is staged locally; CI (`.github/workflows/release.yml`) builds all platforms on a `v*` tag.

On a low-RAM machine, prefix builds with `CARGO_BUILD_JOBS=2`.
