use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::Duration;

use tauri::path::BaseDirectory;
use tauri::{Manager, WindowEvent};
use tauri_plugin_shell::process::{CommandChild, CommandEvent};
use tauri_plugin_shell::ShellExt;

/// WS port the bundled `thingblock-link` sidecar listens on. Contract with the
/// editor, which connects to `ws://localhost:<this>` — keep both sides in lockstep.
const LINK_WS_PORT: u16 = 3030;

/// How long to wait for the sidecar to exit after a graceful shutdown request
/// before force-killing it, so a hung sidecar can never block the app from closing.
const SIDECAR_SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(3);

/// Tracks the sidecar's lifecycle so the app can request a graceful shutdown (and
/// fall back to a hard kill) when the window closes.
#[derive(Default)]
struct LinkSidecarState {
    child: Mutex<Option<CommandChild>>,
    exited_rx: Mutex<Option<tauri::async_runtime::Receiver<()>>>,
    /// Set once we've started our own shutdown sequence, so the `window.close()`
    /// it ends with isn't mistaken for a fresh user close request.
    closing: AtomicBool,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(LinkSidecarState::default())
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            spawn_link_sidecar(app.handle())?;
            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                let state = window.state::<LinkSidecarState>();
                if state.closing.swap(true, Ordering::SeqCst) {
                    // This is the close we triggered ourselves after stopping the
                    // sidecar; let it proceed instead of looping.
                    return;
                }
                api.prevent_close();
                let window = window.clone();
                tauri::async_runtime::spawn(async move {
                    stop_link_sidecar(&window.state::<LinkSidecarState>()).await;
                    let _ = window.close();
                });
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// Request the sidecar to shut down gracefully (see `watch_stdin_for_shutdown` on
/// the link side) and wait for it to exit, force-killing it only if it doesn't
/// exit within `SIDECAR_SHUTDOWN_TIMEOUT`.
async fn stop_link_sidecar(state: &LinkSidecarState) {
    let write_result = state
        .child
        .lock()
        .unwrap()
        .as_mut()
        .map(|child| child.write(b"\n"));
    if let Some(Err(e)) = write_result {
        // The sidecar is likely already gone; the exit wait below will resolve
        // immediately (or the timeout fallback below will no-op on an empty child).
        log::warn!("failed to signal thingblock-link to shut down: {e}");
    }

    let exited_rx = state.exited_rx.lock().unwrap().take();
    let exited_gracefully = match exited_rx {
        Some(mut rx) => tokio::time::timeout(SIDECAR_SHUTDOWN_TIMEOUT, rx.recv())
            .await
            .is_ok(),
        None => false,
    };

    if !exited_gracefully {
        if let Some(child) = state.child.lock().unwrap().take() {
            log::warn!(
                "thingblock-link did not exit within {SIDECAR_SHUTDOWN_TIMEOUT:?}; forcing kill"
            );
            if let Err(e) = child.kill() {
                log::error!("failed to force-kill thingblock-link: {e}");
            }
        }
    }
}

/// Launch the `thingblock-link` helper as a sidecar process and pump its output
/// into the log. The editor (in the webview) talks to it over WebSocket; this
/// shell owns the process lifecycle, including stopping it when the window closes.
///
/// Path resolution happens here (fail fast on broken packaging); the arduino
/// config seeding and the actual spawn run on a background task, because the
/// first-run seed copies ~260 MB and must not hold up window creation.
fn spawn_link_sidecar(app: &tauri::AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    // The link resolves compile libs from a resource-pack directory bundled
    // beside the app. Pass it explicitly rather than relying on the link's
    // "next to the executable" default, since Tauri stages sidecar binaries and
    // resources into separate locations across platforms.
    let resource_root = app
        .path()
        .resolve("thingblock-resource", BaseDirectory::Resource)?;
    // The link spawns arduino-cli from the path we hand it; point it at the copy
    // bundled beside the app rather than the link's compile-time source path. The
    // binary is staged under `bin/` (a directory resource, uniform across
    // platforms); only its filename carries the Windows `.exe`.
    let arduino_cli_rel = if cfg!(windows) {
        "bin/arduino-cli.exe"
    } else {
        "bin/arduino-cli"
    };
    let arduino_cli = app
        .path()
        .resolve(arduino_cli_rel, BaseDirectory::Resource)?;
    // The daemon runs against `arduino-cli.yaml` + `data/` (the link's
    // `--config-dir` contract). The bundled seed lives in the read-only resource
    // dir, but arduino-cli writes into its `directories.*` (downloads, on-demand
    // core installs), so it runs against a writable per-user copy instead.
    //
    // The copy lives in a deliberately short dir (`%LOCALAPPDATA%\ThingBlock` on
    // Windows), not under the app's `com.thingblock.desktop` data dir: the esp32
    // GCC toolchains resolve their C++ multilib headers through include paths
    // deep enough that the longer base pushed them past Windows' 260-char
    // MAX_PATH, breaking compiles with `bits/c++config.h: No such file`.
    let seed_dir = app.path().resolve("arduino", BaseDirectory::Resource)?;
    let config_dir = app.path().local_data_dir()?.join("ThingBlock");

    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        let seeded = tauri::async_runtime::spawn_blocking({
            let seed_dir = seed_dir.clone();
            let config_dir = config_dir.clone();
            move || seed_arduino_config(&seed_dir, &config_dir)
        })
        .await;
        match seeded {
            Ok(Ok(())) => {}
            Ok(Err(e)) => {
                log::error!(
                    "failed to seed arduino config dir {}: {e}",
                    config_dir.display()
                );
                return;
            }
            Err(e) => {
                log::error!("arduino config seeding task panicked: {e}");
                return;
            }
        }
        // The window may have been closed while seeding ran; a sidecar spawned
        // now would outlive the shutdown sequence that already finished.
        if app
            .state::<LinkSidecarState>()
            .closing
            .load(Ordering::SeqCst)
        {
            return;
        }
        if let Err(e) = start_link_sidecar(&app, &resource_root, &arduino_cli, &config_dir) {
            log::error!("failed to start thingblock-link: {e}");
        }
    });

    Ok(())
}

/// Spawn the sidecar process and wire its lifecycle into `LinkSidecarState`.
fn start_link_sidecar(
    app: &tauri::AppHandle,
    resource_root: &Path,
    arduino_cli: &Path,
    config_dir: &Path,
) -> Result<(), tauri_plugin_shell::Error> {
    let (mut rx, child) = app
        .shell()
        .sidecar("thingblock-link")?
        .args([
            "--port",
            &LINK_WS_PORT.to_string(),
            "--resource-root",
            &resource_root.to_string_lossy(),
            "--arduino-cli",
            &arduino_cli.to_string_lossy(),
            "--config-dir",
            &config_dir.to_string_lossy(),
        ])
        .spawn()?;

    let (exited_tx, exited_rx) = tauri::async_runtime::channel::<()>(1);
    let state = app.state::<LinkSidecarState>();
    state.child.lock().unwrap().replace(child);
    state.exited_rx.lock().unwrap().replace(exited_rx);

    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        while let Some(event) = rx.recv().await {
            match event {
                CommandEvent::Stdout(bytes) | CommandEvent::Stderr(bytes) => {
                    log::info!(
                        "[thingblock-link] {}",
                        String::from_utf8_lossy(&bytes).trim_end()
                    );
                }
                CommandEvent::Terminated(payload) => {
                    let state = app.state::<LinkSidecarState>();
                    if state.closing.load(Ordering::SeqCst) {
                        log::info!("[thingblock-link] exited after shutdown request: {payload:?}");
                    } else {
                        log::error!("[thingblock-link] exited unexpectedly: {payload:?}");
                    }
                    let _ = exited_tx.send(()).await;
                    break;
                }
                _ => {}
            }
        }
    });

    Ok(())
}

/// Materialize the writable arduino config dir from the bundled seed. The yaml
/// is overwritten on every launch so config changes ship with app updates; the
/// `data/` bundle is copied only when absent — it is user-mutable (on-demand
/// core installs land there) and large, so it is seeded exactly once.
fn seed_arduino_config(seed_dir: &Path, config_dir: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(config_dir)?;
    std::fs::copy(
        seed_dir.join("arduino-cli.yaml"),
        config_dir.join("arduino-cli.yaml"),
    )?;
    let data_dst = config_dir.join("data");
    if !data_dst.exists() {
        copy_dir_recursive(&seed_dir.join("data"), &data_dst)?;
    }
    Ok(())
}

/// Recursive directory copy. Symlinks are recreated rather than dereferenced —
/// the avr-gcc toolchain in the seed relies on them (Unix only; the Windows
/// toolchain has none).
fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let dst_path = dst.join(entry.file_name());
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            copy_dir_recursive(&entry.path(), &dst_path)?;
        } else if file_type.is_symlink() {
            #[cfg(unix)]
            std::os::unix::fs::symlink(std::fs::read_link(entry.path())?, &dst_path)?;
            #[cfg(not(unix))]
            std::fs::copy(entry.path(), &dst_path)?;
        } else {
            std::fs::copy(entry.path(), &dst_path)?;
        }
    }
    Ok(())
}
