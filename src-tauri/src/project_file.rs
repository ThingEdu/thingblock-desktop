//! Native "Save" / "Open" for project files.
//!
//! The editor runs in a webview, where saving a project is a browser download: the file lands in
//! Downloads without the user ever choosing a folder, so it doesn't feel saved. These commands give
//! the editor the native dialogs instead. The editor asks for a path (`pick_save_path` /
//! `pick_open_path`), then reads or writes the bytes. Writes and reads are only allowed for paths the
//! user picked in one of those dialogs during this run, so the webview can't touch arbitrary files.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use tauri::ipc::{InvokeBody, Request, Response};
use tauri::{AppHandle, Manager, State};
use tauri_plugin_dialog::DialogExt;

/// Project file extension, without the dot.
const PROJECT_EXTENSION: &str = "tb";
/// File (under the app config dir) that remembers the folder of the last project saved or opened.
const LAST_DIR_FILE: &str = "last-project-dir.txt";
/// Header the editor puts the URI-encoded target path in when it sends a raw project body.
const PATH_HEADER: &str = "x-project-path";

#[derive(Default)]
pub struct ProjectFileState {
    /// Paths the user picked in a dialog this run; the only ones `write`/`read` accept.
    allowed: Mutex<HashSet<PathBuf>>,
}

impl ProjectFileState {
    fn allow(&self, path: &Path) {
        self.allowed.lock().unwrap().insert(path.to_path_buf());
    }
    fn is_allowed(&self, path: &Path) -> bool {
        self.allowed.lock().unwrap().contains(path)
    }
}

/// Give a chosen save path the project extension if the user typed a name without one.
pub fn with_project_extension(path: PathBuf) -> PathBuf {
    let has_ext = path
        .extension()
        .map(|ext| ext.eq_ignore_ascii_case(PROJECT_EXTENSION))
        .unwrap_or(false);
    if has_ext {
        path
    } else {
        let mut name = path.file_name().map(|n| n.to_os_string()).unwrap_or_default();
        name.push(".");
        name.push(PROJECT_EXTENSION);
        path.with_file_name(name)
    }
}

/// Decode the editor's URI-encoded path header (`encodeURIComponent` on the JS side).
pub fn decode_path_header(value: &str) -> Option<PathBuf> {
    let bytes = value.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' {
            let hex = std::str::from_utf8(bytes.get(i + 1..i + 3)?).ok()?;
            out.push(u8::from_str_radix(hex, 16).ok()?);
            i += 3;
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    let decoded = String::from_utf8(out).ok()?;
    if decoded.is_empty() {
        None
    } else {
        Some(PathBuf::from(decoded))
    }
}

fn last_dir_file(app: &AppHandle) -> Option<PathBuf> {
    app.path().app_config_dir().ok().map(|dir| dir.join(LAST_DIR_FILE))
}

fn last_dir(app: &AppHandle) -> Option<PathBuf> {
    let saved = std::fs::read_to_string(last_dir_file(app)?).ok()?;
    let dir = PathBuf::from(saved.trim());
    if dir.is_dir() {
        Some(dir)
    } else {
        None
    }
}

fn remember_dir(app: &AppHandle, path: &Path) {
    let (Some(file), Some(dir)) = (last_dir_file(app), path.parent()) else {
        return;
    };
    if let Some(parent) = file.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let _ = std::fs::write(file, dir.to_string_lossy().as_bytes());
}

fn default_dir(app: &AppHandle) -> Option<PathBuf> {
    last_dir(app).or_else(|| app.path().document_dir().ok())
}

/// Show the native Save dialog. Returns the chosen path (with `.tb`), or `None` when cancelled.
#[tauri::command]
pub async fn pick_save_path(
    app: AppHandle,
    state: State<'_, ProjectFileState>,
    suggested_name: String,
) -> Result<Option<String>, String> {
    let mut dialog = app
        .dialog()
        .file()
        .add_filter("ThingBlock", &[PROJECT_EXTENSION])
        .set_file_name(suggested_name);
    if let Some(dir) = default_dir(&app) {
        dialog = dialog.set_directory(dir);
    }
    let Some(chosen) = dialog.blocking_save_file() else {
        return Ok(None);
    };
    let path = with_project_extension(chosen.into_path().map_err(|e| e.to_string())?);
    state.allow(&path);
    Ok(Some(path.to_string_lossy().into_owned()))
}

/// Show the native Open dialog for a `.tb` file. Returns the chosen path, or `None` when cancelled.
#[tauri::command]
pub async fn pick_open_path(
    app: AppHandle,
    state: State<'_, ProjectFileState>,
) -> Result<Option<String>, String> {
    let mut dialog = app
        .dialog()
        .file()
        .add_filter("ThingBlock", &[PROJECT_EXTENSION]);
    if let Some(dir) = default_dir(&app) {
        dialog = dialog.set_directory(dir);
    }
    let Some(chosen) = dialog.blocking_pick_file() else {
        return Ok(None);
    };
    let path = chosen.into_path().map_err(|e| e.to_string())?;
    state.allow(&path);
    remember_dir(&app, &path);
    Ok(Some(path.to_string_lossy().into_owned()))
}

/// Write the raw request body to a path the user picked. The path comes in the `x-project-path`
/// header so the project bytes travel as a raw body instead of a JSON number array.
#[tauri::command]
pub fn write_project_file(
    app: AppHandle,
    state: State<'_, ProjectFileState>,
    request: Request<'_>,
) -> Result<(), String> {
    let path = request
        .headers()
        .get(PATH_HEADER)
        .and_then(|value| value.to_str().ok())
        .and_then(decode_path_header)
        .ok_or("missing project path")?;
    if !state.is_allowed(&path) {
        return Err(format!("not a path chosen in the save dialog: {}", path.display()));
    }
    let InvokeBody::Raw(bytes) = request.body() else {
        return Err("expected the project as a raw body".into());
    };
    std::fs::write(&path, bytes).map_err(|e| format!("{}: {e}", path.display()))?;
    remember_dir(&app, &path);
    Ok(())
}

/// Read a project the user picked in the Open dialog.
#[tauri::command]
pub fn read_project_file(state: State<'_, ProjectFileState>, path: String) -> Result<Response, String> {
    let path = PathBuf::from(path);
    if !state.is_allowed(&path) {
        return Err(format!("not a path chosen in the open dialog: {}", path.display()));
    }
    let bytes = std::fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(Response::new(bytes))
}

/// Show the project's file name in the window title, e.g. "Xe dò line.tb — ThingBlock".
#[tauri::command]
pub fn set_window_title(window: tauri::WebviewWindow, title: String) -> Result<(), String> {
    window.set_title(&title).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adds_the_extension_when_missing() {
        assert_eq!(with_project_extension("/a/Dự án".into()), PathBuf::from("/a/Dự án.tb"));
        assert_eq!(with_project_extension("/a/x.tb".into()), PathBuf::from("/a/x.tb"));
        assert_eq!(with_project_extension("/a/x.TB".into()), PathBuf::from("/a/x.TB"));
        assert_eq!(with_project_extension("/a/x.sb3".into()), PathBuf::from("/a/x.sb3.tb"));
    }

    #[test]
    fn decodes_uri_encoded_paths() {
        // encodeURIComponent('/Users/em/Dự án ThingBlock.tb')
        let header = "%2FUsers%2Fem%2FD%E1%BB%B1%20%C3%A1n%20ThingBlock.tb";
        assert_eq!(decode_path_header(header), Some(PathBuf::from("/Users/em/Dự án ThingBlock.tb")));
        assert_eq!(decode_path_header(""), None);
        assert_eq!(decode_path_header("%zz"), None);
        assert_eq!(decode_path_header("%E1"), None);
    }

    #[test]
    fn only_dialog_paths_are_allowed() {
        let state = ProjectFileState::default();
        let chosen = PathBuf::from("/tmp/chosen.tb");
        assert!(!state.is_allowed(&chosen));
        state.allow(&chosen);
        assert!(state.is_allowed(&chosen));
        assert!(!state.is_allowed(Path::new("/etc/passwd")));
    }
}
