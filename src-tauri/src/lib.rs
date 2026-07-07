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
