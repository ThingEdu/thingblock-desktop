use tauri::path::BaseDirectory;
use tauri::Manager;
use tauri_plugin_shell::process::CommandEvent;
use tauri_plugin_shell::ShellExt;

/// WS port the bundled `thingblock-link` sidecar listens on. Contract with the
/// editor, which connects to `ws://localhost:<this>` — keep both sides in lockstep.
const LINK_WS_PORT: u16 = 3030;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
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
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// Launch the `thingblock-link` helper as a sidecar process and pump its output
/// into the log. The editor (in the webview) talks to it over WebSocket; this
/// shell only owns the process lifecycle.
fn spawn_link_sidecar(app: &tauri::AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    // The link resolves compile libs from a resource-pack directory bundled
    // beside the app. Pass it explicitly rather than relying on the link's
    // "next to the executable" default, since Tauri stages sidecar binaries and
    // resources into separate locations across platforms.
    let resource_root = app
        .path()
        .resolve("thingblock-resource", BaseDirectory::Resource)?;
    // The link spawns arduino-cli from the path we hand it; point it at the copy
    // bundled beside the app rather than the link's compile-time source path.
    let arduino_cli = app.path().resolve("arduino-cli", BaseDirectory::Resource)?;

    let (mut rx, _child) = app
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
                    log::error!("[thingblock-link] exited: {:?}", payload);
                    break;
                }
                _ => {}
            }
        }
    });

    Ok(())
}
