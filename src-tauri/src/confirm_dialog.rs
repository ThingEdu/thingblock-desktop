//! A native yes/no question for the editor.
//!
//! The dialog plugin swaps the webview's `window.confirm` for an async version, which the editor's
//! synchronous `if (!confirm(...))` checks read as "yes" — so a changed project was replaced
//! without asking. The editor now awaits this command instead, which also lets it put its own
//! (Vietnamese) labels on the buttons.

use tauri::AppHandle;
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};

/// Ask `message` in a native dialog. Returns true when the user picked the `ok_label` button.
#[tauri::command]
pub async fn confirm_dialog(
    app: AppHandle,
    message: String,
    title: Option<String>,
    ok_label: Option<String>,
    cancel_label: Option<String>,
) -> bool {
    let (ok, cancel) = labels(ok_label, cancel_label);
    app.dialog()
        .message(message)
        .title(title.unwrap_or_else(|| "ThingBlock".into()))
        .kind(MessageDialogKind::Warning)
        .buttons(MessageDialogButtons::OkCancelCustom(ok, cancel))
        .blocking_show()
}

/// Button labels, falling back to English when the editor sent none (or blank ones).
fn labels(ok: Option<String>, cancel: Option<String>) -> (String, String) {
    let pick = |label: Option<String>, fallback: &str| {
        label
            .filter(|l| !l.trim().is_empty())
            .unwrap_or_else(|| fallback.into())
    };
    (pick(ok, "OK"), pick(cancel, "Cancel"))
}

#[cfg(test)]
mod tests {
    use super::labels;

    #[test]
    fn uses_the_editors_labels() {
        assert_eq!(
            labels(Some("Thay thế".into()), Some("Hủy".into())),
            ("Thay thế".into(), "Hủy".into())
        );
    }

    #[test]
    fn falls_back_to_english_for_missing_or_blank_labels() {
        assert_eq!(labels(None, Some(" ".into())), ("OK".into(), "Cancel".into()));
    }
}
