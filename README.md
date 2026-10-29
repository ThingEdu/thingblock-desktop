# thingblock-desktop

The ThingBlock desktop app. It bundles the [thingblock-editor](../thingblock-editor) UI and the
[thingblock-link](../thingblock-link) helper, which compiles and uploads firmware, into one
installable app.

## Prerequisites

- Rust + Cargo
- Node.js + npm
- `thingblock-editor` and `thingblock-link` cloned next to this repo:

```text
<parent>/
├── thingblock-editor/
├── thingblock-link/
└── thingblock-desktop/
```

## Setup

```sh
npm install
npm --prefix ../thingblock-editor/packages/thingblock-gui install
```

## Run in dev

```sh
npm run tauri dev
```

The first run downloads the Arduino core (~50 MB); later runs skip it. On a low-RAM machine,
prefix the command with `CARGO_BUILD_JOBS=2` if the build gets killed.

## Build an installer

```sh
npm run dist         # production installer
npm run dist:debug   # faster unoptimized build, for local testing
```

Installers land in `src-tauri/target/release/bundle/<format>/`. Only the host platform is built
locally; release installers for all platforms come from CI.

## Lint / format

```sh
cargo fmt --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
```

## Contributing

See [`AGENTS.md`](AGENTS.md) for conventions and the checklist to run before submitting changes.
