# Agent Guide: thingblock-desktop

## What this is

`thingblock-desktop` is the **Tauri (Rust) desktop shell** that packages the two existing
ThingBlock projects into one installable app:

- **`thingblock-editor`** — the web editor UI, rendered in the Tauri webview.
- **`thingblock-link`** — the local Rust helper (WebSocket server + `arduino-cli` gRPC client)
  that compiles and flashes firmware. It runs as a **bundled sidecar process**, not as a library.

The webview loads the editor and talks to the link helper over WebSocket on `ws://localhost:3030`
— exactly the contract the editor already uses in browser mode. The desktop shell adds **no new
protocol**; it just bundles both halves, spawns the sidecar on launch, and gives users a single
double-click install instead of "run the helper, then open the web editor."

**This repo stays small.** It is the shell and the glue: window setup, sidecar lifecycle, bundler
config. It does **not** vendor the editor or the link source — see the layout below.

## How the three repos relate

The three are independent git repos checked out as siblings on disk:

```text
<parent>/
├── thingblock-editor/   # web UI (npm monorepo)
├── thingblock-link/     # Rust WS helper binary (built as a sidecar)
└── thingblock-desktop/  # THIS repo — Tauri shell
```

`thingblock-desktop` reaches out to the siblings rather than embedding them:

- The **editor** is consumed as a built frontend: dev points at its webpack dev server
  (`http://localhost:8601`), release points at its production build
  (`thingblock-editor/packages/scratch-gui/build`).
- The **link** is consumed as a **prebuilt sidecar binary**: `cargo build --release` in the link
  repo, then its binary is staged into `src-tauri/binaries/` (bundled via Tauri `externalBin`).
  Its runtime data — the `thingblock-resource/` pack and the host-platform `arduino-cli` — is staged
  into `src-tauri/resources/` (`arduino-cli` under `resources/bin/`, a directory resource so the
  Tauri `resources` map is identical on every platform) and bundled via Tauri `resources`. The
  resource **pack is produced by the editor** workspace `@thingblock/thingblock-resource` and staged
  from its `dist/thingblock-resource/` output — the link repo's own copy is a gitignored dev
  convenience, not the source of truth. `scripts/stage-sidecar.sh` (macOS/Linux) and
  `scripts/stage-sidecar.ps1` (Windows) do the staging; `beforeBuildCommand`/`beforeDevCommand` run
  the right one automatically.

Do not copy editor or link source into this repo, and do not add them as git submodules. They are
released independently; this shell pins to built artifacts.

## The link contract (don't drift from it)

- The sidecar is spawned with `--port 3030` (the link's default WS port). The webview connects to
  `ws://localhost:3030`. If this port ever changes, it must change on both sides in lockstep — it
  is a contract with the editor, not a local detail.
- The link resolves its `thingblock-resource/` pack and `arduino-cli` from paths we pass it
  (`--resource-root`, `--arduino-cli`). The shell resolves both from `BaseDirectory::Resource` and
  passes them on spawn, so the link finds the bundled copies instead of its compile-time dev paths.
  Without `--arduino-cli`, the link falls back to its in-tree `arduino-cli-binaries/`, which only
  exists on a dev checkout — so a packaged build must always pass it.
- The link owns its own tray icon and event loop as a standalone helper. When run as a sidecar it
  is a separate process from the Tauri window; that is expected.

## Build, run, lint

Standard Rust toolchain (edition 2024) for the shell; npm for the Tauri CLI and frontend glue.

```sh
npm run tauri dev        # run the desktop app against the editor dev server + link sidecar
npm run dist             # production installer (= tauri build): stages sidecar, builds the editor
npm run dist:debug       # faster unoptimized bundle for local testing
cargo fmt --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
```

`dev`/`dist` run `stage:sidecar` and the editor build automatically (via
`beforeDevCommand`/`beforeBuildCommand`). Installers land in
`src-tauri/target/release/bundle/<format>/` (Linux: deb, rpm, AppImage).

On a low-RAM machine the link's release build can be OOM-killed at full parallelism; prefix commands
with `CARGO_BUILD_JOBS=2` to cap it.

Only the **host platform** is staged (one ~36 MB `arduino-cli`); cross-platform packaging is a CI
concern. Per-platform bundle targets live in `tauri.<platform>.conf.json` (Tauri auto-merges them):
base `tauri.conf.json` is Linux (`deb`/`rpm`), `tauri.macos.conf.json` is `dmg`, and
`tauri.windows.conf.json` is `nsis` plus PowerShell build hooks. Windows builds via
`stage-sidecar.ps1` (the sidecar binary and `arduino-cli` carry `.exe`; `lib.rs` resolves the
`.exe` name under `cfg!(windows)`), so no Git Bash is required on Windows.

CI: `.github/workflows/release.yml` builds macOS (`aarch64-apple-darwin`) and Windows
(`x86_64-pc-windows-msvc`) on a version tag (`v*`) and drafts a GitHub Release with the installers.
Artifacts are currently **unsigned** (Gatekeeper/SmartScreen warnings expected).

## Agent defaults

Use these unless the user asks otherwise:

1. Keep changes minimal and scoped to the request. Don't refactor, add features, or restyle code
   you weren't asked to touch.
2. This shell is a standalone app, not a published library — restructure internals freely. The
   contracts to preserve are external: the WS port/protocol with the editor, and the
   sidecar + resource-dir packaging layout the link expects.
3. Comments explain the current code, not its history. If something is counterintuitive, explain
   why it is correct now.
4. Fix root causes, not symptoms. Don't add fallbacks or validation for states that cannot happen.
5. When fixing a bug, add a failing test first, then fix until it and the rest of the suite pass.
6. Surface invalid states explicitly — prefer an explicit `Err`/`panic!` with a useful message
   over silent failure. Log actionable context via `tracing`: `warn!` for recoverable states,
   `error!` for invalid required data.
7. Validate only at boundaries — process spawn/exit of the sidecar, and anything crossing the
   webview IPC. Trust internal code.

## Conventions

- **Commits** follow [Conventional Commits](https://www.conventionalcommits.org/).
- Keep `Cargo.toml` and `package.json` sections in alphabetical order.
- Design docs live in `.agents/docs/` (mirrors the link repo).

## Before submitting changes

- **Scope**: changes confined to the request; nothing extra added.
- **Build clean**: `npm run tauri build` (or at least `cargo build --manifest-path src-tauri/Cargo.toml`).
- **Lint clean**: `cargo clippy ... -D warnings`, `cargo fmt --check`.
- **Docs in sync**: if you change a convention, the WS port, or the packaging layout, update this
  file accordingly.
- **Commit format**: Conventional Commits.
