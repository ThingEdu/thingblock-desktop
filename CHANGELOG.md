# Changelog

App-level release notes for ThingBlock. The app is built from three repos — `thingblock-editor`
(UI, blocks, codegen), `thingblock-link` (local helper), `thingblock-desktop` (Tauri shell) — so
this file is the one place that says what changed across all of them.

Entries name the PR that carries the change and whether it is merged or still open, because a
preview installer can contain work that no `main`/`master` has yet.

## 0.9.0-preview-20261120

Built from desktop `develop` against editor `develop` and link `develop`. Everything below is
merged into `develop` only — none of it is on editor `main` or link/desktop `master`, and the
editor changes were merged without review. Changes are since 0.8.0-preview-20261115. Editor, link
and desktop share one version, `0.9.0-preview-20261120`; link has no changes beyond the version.
Installers: `x86_64-pc-windows-msvc` (nsis), `aarch64-apple-darwin` (dmg), Linux x64 (deb/rpm).
Unsigned — Gatekeeper and SmartScreen will warn.

### New editor

- **The app ships the TypeScript editor, `apps/thingblock`, instead of `thingblock-gui`** —
  desktop `8f5d67c`. It is a port of `thingblock-gui` built with Vite, on the same VM, blocks and
  resource packs. `thingblock-gui` stays in the editor repo but is no longer shipped.
- Settings → More settings… opens a settings modal with Preferences (language, theme, colour mode)
  and About (version, GitHub, licence, Scratch credit) — editor `ac8673324`, `5c56e733b`.
- Extension cards show what each extension requires: Camera AI a webcam, ThingBot Telemetrix
  Bluetooth — editor `ef18f0a8f`.
- Connecting ThingBot moves from the menu bar into the ThingBot Telemetrix pane in the side panel —
  editor `383671a48`.
- The Camera AI pane's controls move into a floating action button beside the stage, and the app
  serves the Camera AI models itself — editor `2c71754b2`, `591fa5ff1`.
- The menu bar logo is drawn as paths, so the Fredoka One web font is gone — editor `b96c14b4c`.

### Removed

- **The simulator and lessons** — the `thingblock-sim` package, the sim panel, the lesson library
  and lesson panel, and the sim models in the resource pack — editor `0d5deea2f`.
- **Example projects** — File → Examples and the examples in the resource pack — editor
  `5170f33aa`, `6ef03b477`, `370b03416`.
- **Playing Camera AI with the mouse** — editor `dc7d9a6cb`. Camera AI needs a webcam.

### Build and dependencies

- `release.yml` and `stage-frontend` build `apps/thingblock` and stage `apps/thingblock/dist`;
  `cross-env` and the webpack memory limit are dropped — desktop `8f5d67c`.
- Vulnerable dependencies patched (vitest 4.1.12, immutable 3.8.4, `npm audit fix`) — editor
  `0bebdb581`; scratch-l10n 6.1.142 drops Transifex — editor `cb4be5895`.

### Known gaps

- This is the first build of `apps/thingblock` inside the desktop shell. Connecting a board,
  uploading, flashing firmware and native Save/Open have not been tried with it yet.

## 0.8.0-preview-20261115

Built from desktop `develop` against editor `develop` and link `develop`. Everything below is
merged into `develop` only — none of it is on editor `main` or link/desktop `master`, and the
editor PRs were merged without review. Changes are since 0.6.0-preview-20261104, the last
published release (0.7.0-preview-20261110 and 0.8.0-preview-20261112 were never published).
Editor, link and desktop now share one version, `0.8.0-preview-20261115`. **Thing Smart
(ThingEdges, editor #63/#68) is not in this build.** Installers: `x86_64-pc-windows-msvc` (nsis),
`aarch64-apple-darwin` (dmg), Linux x64 (deb/rpm). Unsigned — Gatekeeper and SmartScreen will warn.

### Camera AI

- **Camera AI extension** (`cameraAi`) — editor **#60, #67 (merged into develop)**. Webcam hand
  tracking (MediaPipe hand landmarker) and head/body tracking (MediaPipe pose landmarker lite,
  loaded only when a project uses head blocks or the flap hat). Blocks for hand position and
  gestures, head position and lean, touch hats, and a playground panel next to the simulator with
  objects, physics, scores and a countdown.
- **Play without a webcam** — "Chuột (không cần webcam)" in the camera list: the pointer is the
  index fingertip, mouse buttons and keys 1–7 make gestures, arrow keys and space drive the head
  and flap.
- 11 examples under File → Examples → Camera AI and 4 graded Camera AI lessons, graded in the
  editor from VM events rather than the simulator.
- Fixes: a forever loop moving objects no longer floods the GUI with updates; gesture labels no
  longer fall back to English after `vm.setLocale`.
- macOS: the app declares camera use so the blocks get camera access — desktop `f0901c2`.
- Not yet tuned with children: the flap, nod and lean thresholds.

### Boards and peripherals

- **CoBot** (Maker Việt line-following robot, ESP32-C3) — editor **#66 (merged into develop)**.
  Drive, turn, line-eye and light blocks, and 5 examples under File → Examples → CoBot. No
  simulator for CoBot yet. Not checked on a real CoBot: the line threshold direction, the eye
  order and the colour thresholds.
- **Colour Sensor (I2C)** — editor **#66**. TCS34725 at 0x29 on any board: red, green, blue,
  brightness, "sees colour?" and the colour name.

### Simulator and lessons

- **ThingBot Sim lessons 6 and 10 rewritten** — editor **#65 (merged into develop)**. Lesson 6:
  an IR eye on IO4 picks FanRoc balls by colour. Lesson 10: an egg sorting belt driven by M1,
  with servo S1 rejecting bad eggs. The sim panel gains a ball picker and the egg belt.
- **Lessons and example notes rewritten for children**, plus an "Electronics dictionary" lesson —
  editor #61, merged through **#69**.
- **Lesson format moved to English** (`lesson.<lang>.md`, English ids, keys and rule grammar) —
  editor `11d8c1de7`. **Lesson ids changed, so progress saved by earlier previews is lost.**

### Editor layout

- The lesson panel moves to a left panel, and the lesson library is sectioned by group — editor
  `11d8c1de7`, `27fe413e7`.
- The side panel splits into a host monitor (no board) and a board monitor (serial, baud rate),
  each hideable from the panel menu — editor `1fab8fbd7`.
- The simulator is a headerless card above the code view; Run/Pause, Step, Reset and Mute move
  into a floating action button — editor `2b44924f4`.
- Connecting a board shows a connecting state and inline errors — editor `81b4f5f06`.
- Vietnamese for the board menu, the code tab, connecting messages and board descriptions —
  editor `d9e426552` (part of #48).

### Link

- **Gateway architecture** — link **#12 (merged)**. HTTP API (`/api/platforms`, `/api/boards`)
  plus WebSockets at `/ws/arduino` and `/ws/ble`; the editor adopts it in `04d83f082`. The editor
  and link must come from the same release. Only `arduino:avr` and `esp32:esp32` (pinned to
  3.3.11) are listed and installable.
- **arduino-cli daemon ends with the link** — link **#14 (merged)**. Closing the app, a force
  quit or a crash no longer leaves `arduino-cli daemon` processes running.

### Licence and build

- Editor, link and desktop are licensed AGPL-3.0-only — editor #64, link #13, desktop `1b730ba`.
- `release.yml`: macOS builds on `macos-15` (`c7601a6`); manual runs take `editor_ref` /
  `link_ref` inputs.

## 0.8.0-preview-20261112

Preview built by a manual run of `release.yml` against editor branch `release/0.8.0-preview` (the
open 0.7 stack #60, #61, #65, #66, #67 plus a pass over lessons, examples, block labels and the
generated Arduino comments for children) and link `master` (#14 merged). **Thing Smart (ThingEdges,
editor #63/#68) is not in this build** — it is held for a later release. Unsigned — Gatekeeper and
SmartScreen will warn.

- Everything listed for 0.7.0-preview-20261110 (Camera AI, mouse play, exercise games, CoBot board,
  I2C colour sensor, ThingBot Sim lessons 6 and 10, beginner glossary, macOS camera permission).
- Lessons: 16 of 19 rewritten for children; fixes three factual errors (OLED print wraps by
  default, the ThingBot start block is optional, the I2C port is 5V).
- Examples: all 52 notes rewritten, Vietnamese variable names, comments on magic numbers.
- CoBot and colour sensor: plainer block labels, Vietnamese comments in the generated Arduino code.
- `release.yml`: manual runs take `editor_ref` / `link_ref` inputs.

## 0.5.0 — 2026-10-30

Built from desktop `master` against editor `main` and link `master`; everything below is merged.
Installers: `x86_64-pc-windows-msvc` (nsis), `aarch64-apple-darwin` (dmg), Linux x64 (deb/rpm).
Unsigned — Gatekeeper and SmartScreen will warn. Resource pack `0.5`; link unchanged at `0.4.0`.

### Peripherals

- **SH1106 OLED** — editor **#24 (merged)**, `3559b4031`. The common 1.3" 128x64 OLED uses an
  SH1106 controller, which acknowledges SSD1306 writes on the same I2C address and stays dark. The
  OLED init block gains a chip field that switches to the vendored Adafruit SH110X driver; colour
  names are aliased and scroll calls become no-ops, so every OLED block works on either chip.
  Projects saved without the field keep SSD1306.
- **Rotary encoder pack** (`encoder`) — editor **#24**, `3559b4031`, `cb30c7c11`. Init (A/B/button
  pins), position in detent clicks, reset, and button pressed. Both phases are read through
  interrupts so turns are not lost while the sketch is in `delay()`; the ISR keeps its attribute
  out of the generated globals bucket.

### Editor internals (breaking for embedders, not for projects)

- **Packages renamed to the `@thingblock/` scope** — editor **#26 (merged)**, `23b9a3105`
  (`scratch-gui` → `thingblock-gui`, `scratch-vm` → `thingblock-vm`, `scratch-blocks` →
  `thingblock-blocks`, …). Desktop follows the new paths in `df705e5`.
- **VM split into owned parts and one event bus** — editor **#27 (merged)**. `vm.link`
  (`LinkController`: boards, compile, upload, firmware, monitor), `vm.devices`, `vm.workspace` and
  `vm.project` replace the VM mixins. One `EventBus` per VM is shared as `vm.events` and
  `runtime.events`; `VirtualMachine` no longer extends `EventEmitter`. Monitor input goes through
  `vm.writeMonitor()`, which writes the line to a connected board and also feeds the sensing
  `answer`. Separating public VM events from internal runtime events is tracked in editor #28.
- **Scratch project server removed** — editor `0c11d8317`. The GUI no longer fetches `#<id>`
  projects from `projects.scratch.mit.edu`, and the Scratch project/asset web stores and
  `projectHost` / `assetHost` / `projectToken` props are gone. Sharing projects online is tracked in
  editor #29.
- Removed the VM playground, `scratch-svg-renderer`, unused GUI dependencies and the unused
  upstream CI workflows.

## 0.4.0 — 2026-09-26

Built from desktop `master` against editor `main` and link `master`; everything below is merged.
Includes all of preview 0.3.2 and 0.3.3. Installers: `x86_64-pc-windows-msvc` (nsis),
`aarch64-apple-darwin` (dmg), Linux x64 (deb/rpm). Linux arm64 installers are no longer built.
Unsigned — Gatekeeper and SmartScreen will warn.

### Phone control

- **A phone can drive the board while the program runs** — editor `9ecca525d`, `b3057d2a6`. A new
  **Phone control** peripheral adds one block, `allow phone control, board name [ThingBot]`. The
  compiled program then advertises over BLE with the same ThingBot Telemetrix protocol as live
  mode, so the ThingBot Remote app drives motors, servos, the buzzer and the LEDs alongside the
  student's own blocks.
- A BLE board takes one connection: disconnect ThingBlock's live mode before scanning from the phone.
- Phone and program write the same PCA9685, so the last writer wins, and there is no failsafe yet —
  if BLE drops, the motors keep their last command.

### Editor scope (breaking)

ThingBlock is firmware-only, so stock Scratch features that could not reach a board are gone:
the sprite, costume, backdrop and sound libraries, the sound editor, the backpack, and the pen,
music, text-to-speech, translate, speech-to-text and Makey Makey extensions.

The VM now runs on a TypeScript runtime (editor `b84e55220`).

### Project files (`.tb`)

**A saved `.tb` would not reopen.** Any project that used board blocks failed to load, and the GUI
reported it as a generic "could not load project" alert with the real error only on a console a
packaged build could not show.

Two separate causes, both fixed:

1. **Pack ids were treated as VM extensions.** Resource-pack blocks (`thingBotC3`, `dht`, `oled`,
   `serial`) use the same opcode-prefix convention as VM extensions, so loading a project called
   `extensionManager.loadExtensionURL()` on each of them. Those took the remote-Worker path and
   rejected, failing the whole load.

   Fixed in editor **#11 (merged, `dd430b8`)**: each peripheral manifest now *declares* its
   `resourcePackIds`, and the guard sits inside `loadExtensionURL` itself — so every call site,
   including `shareBlocksToTarget`, is covered by construction rather than by a repeated check.

2. **The workspace rendered before the board's blocks existed.** Loading ended by emitting a
   workspace update, and the GUI answered it by handing the project XML to Blockly, which throws on
   the first block type it has no definition for (`Invalid block definition for type:
   thingBotC3_init`). A board's definitions come from packs registered by `_applyBoard`, which ran
   *after* target installation — so the render always outran the definitions and every board block
   was dropped. `loadProject` still resolved, so the only symptom was a project that opened blank.

   Also fixed in **#11**: the board is applied before targets are installed, and a board that was
   held pending (packs not yet arrived) re-emits the workspace update once it is applied.

**Format note.** A `.tb` is an sb3-shaped zip whose `project.json` carries one extra root field,
`board` — `{device, peripherals}` — which is what restores the selected board and its peripherals
on open. Nothing else about the sb3 layout changed, so a `.tb` remains readable by anything that
reads sb3, minus the board.

Renamed as part of #11: pack ids `thingbot-core` → `thingBotC3` and `viaBanhMi-core` → `viaBanhMi`,
so a pack id always equals its opcode prefix.

### Servos

- **Degree-based motion** — editor **#5 (merged)**. The old blocks took a raw PCA9685 pulse count,
  which children could not reason about, and snapped to the target at full speed. New blocks take
  0–180°, take a duration, and interpolate one step per 20 ms PWM frame.
- **A release block** — editor **#5 (merged)**. Holding a servo at an angle makes it hunt and buzz;
  releasing the channel stops it.
- **Concurrent motion** — editor **#8 (merged)**. Two "move servo … over N seconds" blocks ran one
  after the other, so S1 finished before S2 started. The command is now split from the wait:
  `start servo … over N seconds` returns immediately and `wait for servos` blocks until all of them
  arrive, so several servos move together.
- **Pulse reference** (unchanged, for anyone reading generated code): 102 = 0°, 307 = 90°,
  512 = 180°, at the 50 Hz prescaler `init ThingBot` sets.

### Sound

- **Note-based music blocks** — editor **#7 (merged)**. The buzzer block took a raw pulse value. The
  new blocks take note names and durations so a child who reads music can play a tune directly.
- **Known constraint:** the PCA9685 has one prescaler shared by all 16 channels. Servo motion needs
  50 Hz; musical pitch needs the prescaler to change. Music and servo motion therefore cannot run at
  the same time on this board.

### OLED

- **Nothing displayed, with no error** — editor **#13 (merged)**. A correctly wired SSD1306 stayed
  blank for every `oled print`. `Adafruit_GFX`'s constructor leaves `textcolor` at `0xFFFF`, and
  `Adafruit_SSD1306::drawPixel` acts only on `SSD1306_WHITE` (1), `SSD1306_BLACK` (0) and
  `SSD1306_INVERSE` (2) — any other value falls through the switch and no pixel is written. Since
  `init oled` never set a colour, every print drew zero pixels and `refresh oled display` pushed a
  blank frame. The panel was alive and refreshing the whole time, which made it look exactly like
  bad wiring or a wrong I2C address.

  `init oled` now emits `oled.setTextColor(SSD1306_WHITE)` right after `begin()`, as every Adafruit
  example does. `set oled text …` still overrides it.

### Boards and codegen

- **Missing `init ThingBot` no longer breaks the compile** — editor **#9 (merged)**. Pin `#define`s
  were only emitted by that one block, so a program without it failed with `'SERVO_1' was not
  declared`. Every hardware block now registers the board's pin map, PWM object and boot-time init
  itself, idempotently.
- **Numeric comparisons were quoted** — editor **#16 (merged)**. A comparison against a numeric
  literal generated `if (S == "0")` instead of `if (S == 0)`, because the comparison generators
  shadowed every operand through the text path.

### Live mode (Telemetrix over BLE)

- **Reinstall a board's live-mode firmware from inside the app** — editor `130d7e676`, `e2b0e7920`, link
  **#7 (merged)**. Compiling and uploading a sketch erases the Telemetrix firmware, which left a
  student unable to get back to live mode without outside help. A device pack can now declare
  firmware images; the **Connect ThingBot** dialog offers *Update my Device* (pick the USB port,
  install, stop) from the start of the scan, and the flash reuses the upload progress modal.
  Verified on hardware.
- **A live-mode disconnect is reported** — editor `56845995d`. Disconnecting ThingBot now tells the GUI, so
  the connection state no longer goes stale.
- **The serial monitor is freed around a flash** — editor `b8730e3eb`. A flash while the monitor held
  the port failed with the port busy; upload and firmware-flash now share one helper that closes the
  monitor and reopens it afterwards.

### Startup and helper

- **Resource packs survive a slow helper start** — editor **#6 (merged)**. Measured helper startup
  ranged from 7 to 47 seconds; the pack-index retry window was 2 seconds, so a cold start silently
  left the board list empty. The window is now ~79.5 s across 14 attempts with backoff.
- **Resource responses carry cache headers** — link **#6 (merged)**.
- **Windows compiles find bundled libraries** — link **#8 (merged)**. `canonicalize()` returns `\\?\`
  verbatim paths on Windows, which the toolchain rejects, producing
  `Adafruit_PWMServoDriver.h: No such file or directory`. Paths now go through `dunce::simplified()`.
  **Unverified on real Windows** — the hypothesis matches the code and the symptom, but it has not
  been reproduced or confirmed on a Windows machine.

### Desktop shell

- **The packaged app can open a web inspector** — desktop **#3 (merged)**. Release builds compiled the
  `tauri` crate without the `devtools` feature, so a packaged app had no inspector at all and
  frontend faults were undiagnosable in the field. Set `THINGBLOCK_DEVTOOLS` in the environment to
  open it; it stays off for ordinary users.
- **Lockfile version synced** — desktop **#4 (merged)**. `package-lock.json` sat at 0.2.1 after the
  0.3.2 bump, so every `npm install` left the tree dirty.

### Known gaps

- `operator_contains` and the list blocks still make the same numeric-quoting assumption that
  editor #16 fixes for comparisons.
- `thingblock-link` runs `Ble::discover().await` before `axum::serve` (`src/server/router.rs:49`),
  so BLE adapter discovery blocks HTTP and WebSocket startup. Not yet addressed.
- `packages/thingblock-resource/test/packIds.test.ts:26` fails `tsc --noEmit` on `main` (a stubbed
  `ScratchBlocks` is passed where the full module type is expected). Pre-existing; the test itself
  passes under Vitest.
- Installers are unsigned.

## 0.3.2 — 2026-08-11

- **Windows install no longer needs the Visual C++ Redistributable** — the bundled
  `thingblock-link` binary is statically linked against the MSVC runtime.
- **DHT readings were unreliable on boards faster than 16 MHz** (e.g. ESP32) — fixed in the
  bundled `thingblock-resource` pack (editor #4).

## 0.3.1 — 2026-08-09

- **Resource packs failed to load on Windows.** The editor fetched device packs (icons, board
  assets) over HTTP from the bundled link helper, which Chromium's private-network-access checks
  block on Windows. The editor build and resource pack are now staged together into `frontend/`
  and served same-origin; `globalThis.__THINGBLOCK_RESOURCE_BASE__` tells the editor to read packs
  from there instead of over the network.

## 0.3.0

- The `thingblock-desktop`, `thingblock-editor` and `thingblock-link` repos moved to the ThingEdu
  org; `release.yml` checks out `ThingEdu/*`.
- Picks up the updated `thingblock-resource` pack.

## 0.2.1

- **Linux arm64 installers** — native `aarch64` `.deb`/`.rpm`, built on an arm64 runner with the
  arm64 `arduino-cli` and AVR toolchain seed.
- **ESP32 compiles failed on Windows** with `fatal error: bits/c++config.h: No such file or
  directory`. The ESP32 GCC 14 toolchains resolve their C++ multilib headers through include paths
  deep enough that the old arduino data directory (`%LOCALAPPDATA%\com.thingblock.desktop\arduino`)
  pushed them past Windows' 260-character `MAX_PATH`. The arduino config and data now live under
  `%LOCALAPPDATA%\ThingBlock` (`~/Library/Application Support/ThingBlock` on macOS,
  `~/.local/share/ThingBlock` on Linux). The old directory is not migrated.

## 0.2.0

- The Arduino CLI config and AVR toolchain seed (~320 MB) are bundled and copied into the user data
  directory on first run, so compile and flash work offline on a clean machine (0.1.0's packaged
  app failed to start its compile daemon).
- BLE connection flow selected by the board's backend.
- The link helper shuts down gracefully when the app window closes.
- New app icon; updated VIA BanhMi board assets.
- AppImage dropped: linuxdeploy fails on the ELF toolchain binaries in the bundled arduino seed.

## 0.1.0

- First installable build: Tauri shell bundling the editor and the link helper as a sidecar, with
  the `thingblock-resource` pack and host `arduino-cli` staged alongside.
- Windows, macOS and Linux installers (`.exe`, `.dmg`, `.deb`, `.rpm`, `.AppImage`) built by
  GitHub Actions.
