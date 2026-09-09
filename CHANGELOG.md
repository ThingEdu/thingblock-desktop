# Changelog

App-level release notes for ThingBlock. The app is built from three repos — `thingblock-editor`
(UI, blocks, codegen), `thingblock-link` (local helper), `thingblock-desktop` (Tauri shell) — so
this file is the one place that says what changed across all of them.

Entries name the PR that carries the change and whether it is merged or still open, because a
preview installer can contain work that no `main`/`master` has yet.

## Unreleased — preview 0.3.2

Built from `ci/preview-build` (desktop) against the `preview/all-work` branches of the editor and
link repos. Installers: `x86_64-pc-windows-msvc` (nsis), `aarch64-apple-darwin` (dmg), Linux
(deb/rpm). Unsigned — Gatekeeper and SmartScreen will warn.

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

- **Degree-based motion** — editor **#5 (open)**. The old blocks took a raw PCA9685 pulse count,
  which children could not reason about, and snapped to the target at full speed. New blocks take
  0–180°, take a duration, and interpolate one step per 20 ms PWM frame.
- **A release block** — editor **#5 (open)**. Holding a servo at an angle makes it hunt and buzz;
  releasing the channel stops it.
- **Concurrent motion** — editor **#8 (open)**. Two "move servo … over N seconds" blocks ran one
  after the other, so S1 finished before S2 started. The command is now split from the wait:
  `start servo … over N seconds` returns immediately and `wait for servos` blocks until all of them
  arrive, so several servos move together.
- **Pulse reference** (unchanged, for anyone reading generated code): 102 = 0°, 307 = 90°,
  512 = 180°, at the 50 Hz prescaler `init ThingBot` sets.

### Sound

- **Note-based music blocks** — editor **#7 (open)**. The buzzer block took a raw pulse value. The
  new blocks take note names and durations so a child who reads music can play a tune directly.
- **Known constraint:** the PCA9685 has one prescaler shared by all 16 channels. Servo motion needs
  50 Hz; musical pitch needs the prescaler to change. Music and servo motion therefore cannot run at
  the same time on this board.

### OLED

- **Nothing displayed, with no error** — editor **#13 (open)**. A correctly wired SSD1306 stayed
  blank for every `oled print`. `Adafruit_GFX`'s constructor leaves `textcolor` at `0xFFFF`, and
  `Adafruit_SSD1306::drawPixel` acts only on `SSD1306_WHITE` (1), `SSD1306_BLACK` (0) and
  `SSD1306_INVERSE` (2) — any other value falls through the switch and no pixel is written. Since
  `init oled` never set a colour, every print drew zero pixels and `refresh oled display` pushed a
  blank frame. The panel was alive and refreshing the whole time, which made it look exactly like
  bad wiring or a wrong I2C address.

  `init oled` now emits `oled.setTextColor(SSD1306_WHITE)` right after `begin()`, as every Adafruit
  example does. `set oled text …` still overrides it.

### Boards and codegen

- **Missing `init ThingBot` no longer breaks the compile** — editor **#9 (open)**. Pin `#define`s
  were only emitted by that one block, so a program without it failed with `'SERVO_1' was not
  declared`. Every hardware block now registers the board's pin map, PWM object and boot-time init
  itself, idempotently.
- **Numeric comparisons were quoted** — editor **#16 (open)**. A comparison against a numeric
  literal generated `if (S == "0")` instead of `if (S == 0)`, because the comparison generators
  shadowed every operand through the text path.

### Live mode (Telemetrix over BLE)

- **Restore a board's live-mode firmware from inside the app** — editor **#10 (open)**, link
  **#7 (open)**. Compiling and uploading a sketch erases the Telemetrix firmware, which left a
  student unable to get back to live mode without outside help. A device pack can now declare
  firmware images, the board menu offers them, and the flash reuses the upload progress modal.
  Verified on hardware.
- **The serial monitor is freed around a flash** — editor **#10**. A flash while the monitor held
  the port failed with the port busy; upload and firmware-flash now share one helper that closes the
  monitor and reopens it afterwards.

### Startup and helper

- **Resource packs survive a slow helper start** — editor **#6 (open)**. Measured helper startup
  ranged from 7 to 47 seconds; the pack-index retry window was 2 seconds, so a cold start silently
  left the board list empty. The window is now ~79.5 s across 14 attempts with backoff.
- **Resource responses carry cache headers** — link **#6 (open)**.
- **Windows compiles find bundled libraries** — link **#8 (open)**. `canonicalize()` returns `\\?\`
  verbatim paths on Windows, which the toolchain rejects, producing
  `Adafruit_PWMServoDriver.h: No such file or directory`. Paths now go through `dunce::simplified()`.
  **Unverified on real Windows** — the hypothesis matches the code and the symptom, but it has not
  been reproduced or confirmed on a Windows machine.

### Desktop shell

- **The packaged app can open a web inspector** — desktop **#3 (open)**. Release builds compiled the
  `tauri` crate without the `devtools` feature, so a packaged app had no inspector at all and
  frontend faults were undiagnosable in the field. Set `THINGBLOCK_DEVTOOLS` in the environment to
  open it; it stays off for ordinary users.
- **Lockfile version synced** — desktop **#4 (open)**. `package-lock.json` sat at 0.2.1 after the
  0.3.2 bump, so every `npm install` left the tree dirty.

### Known gaps

- `operator_contains` and the list blocks still make the same numeric-quoting assumption that
  editor #16 fixes for comparisons.
- `thingblock-link` runs `Ble::discover().await` before `axum::serve` (`src/server/router.rs:59`),
  so BLE adapter discovery blocks HTTP and WebSocket startup. Not yet addressed.
- `packages/thingblock-resource/test/packIds.test.ts:26` fails `tsc --noEmit` on `main` (a stubbed
  `ScratchBlocks` is passed where the full module type is expected). Pre-existing; the test itself
  passes under Vitest.
- Installers are unsigned.
