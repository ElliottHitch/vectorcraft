//! The Brushes panel's Delete Brush question ([`ask`]): Yes or No when none of the brushes is in
//! use, else Remove Strokes or Cancel. Yes and Remove Strokes run `brush.delete`, which leaves the
//! strokes that used the brushes as plain strokes.
//!
//! Fields: `message` (the question, shown as the heading), `used` (whether a brush is in use) and
//! `params` (the `brush.delete` params).

use serde_json::{Value, json};

use super::{DialogAction, DialogSpec};
use crate::VectorcraftApp;
use crate::state::Dialog;
use crate::widgets;

/// The dialog kind of the Delete Brush question.
pub const KIND: &str = "brushDelete";

pub(super) const SPEC: DialogSpec =
    DialogSpec { heading: |d| d.str("message"), body_with_actions: Some(body), confirm, ok: None, max_width: Some(540.0), ..DialogSpec::FORM };

/// Ask whether to delete the brushes `params` names (`used`: one of them is in use).
pub fn ask(app: &mut VectorcraftApp, params: Value, used: bool) {
    let message = if used {
        tl!("One or more of the brushes are in use and cannot be deleted until their strokes are expanded or removed.")
    } else {
        tl!("Delete the brush selection?")
    };
    app.ui.dialog = Some(Dialog::new(KIND, json!({"params": params, "used": used, "message": message})));
}

fn body(_: &mut VectorcraftApp, ui: &mut egui::Ui, d: &mut Dialog) -> DialogAction {
    let mut action = DialogAction::None;
    ui.horizontal(|ui| {
        if d.bool("used") {
            if widgets::primary_button_enabled(ui, tl!("Remove Strokes"), true).clicked() {
                action = DialogAction::Confirm;
            }
            if widgets::secondary_button(ui, tl!("Cancel")).clicked() {
                action = DialogAction::Cancel;
            }
        } else {
            if widgets::primary_button_enabled(ui, tl!("Yes"), true).clicked() {
                action = DialogAction::Confirm;
            }
            if widgets::secondary_button(ui, tl!("No")).clicked() {
                action = DialogAction::Cancel;
            }
        }
    });
    action
}

/// Closes before running, so a dialog the command opens stays open.
fn confirm(app: &mut VectorcraftApp, d: &Dialog) -> Result<Value, String> {
    let params = d.fields.get("params").cloned().unwrap_or_else(|| json!({}));
    app.ui.dialog = None;
    app.run("brush.delete", params)
}

#[cfg(test)]
mod tests;
