//! "Missing toolchain" editor notification: a bar above the editor when the
//! open file's language lacks its SDK or language server, with Set up.

use crate::{IdeApp, icons::Icon, theme};
use eframe::egui::{self, Color32, RichText};
use serde_json::json;

/// `Banner.warningBackground` / `Banner.warningBorderColor` (dark, `docs/spec/data/themes.json`).
const BACKGROUND: Color32 = Color32::from_rgb(0x44, 0x32, 0x1D);
const BORDER: Color32 = Color32::from_rgb(0x69, 0x48, 0x20);

pub fn show(app: &mut IdeApp, ui: &mut egui::Ui) {
    let Some(path) = app
        .state
        .active_document
        .and_then(|index| app.state.documents.get(index))
        .map(|document| document.path.clone())
    else {
        return;
    };
    let Some(banner) = app.state.toolchains.banner_for(&path) else {
        return;
    };
    let mut action = None;
    egui::Frame::new()
        .fill(BACKGROUND)
        .stroke(egui::Stroke::new(1.0_f32, BORDER))
        .corner_radius(6)
        .inner_margin(egui::Margin::symmetric(10, 6))
        .outer_margin(egui::Margin::symmetric(6, 4))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                crate::icons::show(ui, Icon::Warning, 14.0, theme::WARNING);
                ui.label(RichText::new(&banner.text).color(theme::TEXT_BRIGHT));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let link = |ui: &mut egui::Ui, text: &str| {
                        ui.add(
                            egui::Label::new(RichText::new(text).color(theme::MODIFIED))
                                .sense(egui::Sense::click()),
                        )
                        .on_hover_cursor(egui::CursorIcon::PointingHand)
                        .clicked()
                    };
                    if link(ui, "Dismiss") {
                        action = Some((
                            "toolchain.dismissBanner",
                            json!({ "language": banner.language }),
                        ));
                    }
                    ui.add_space(12.0);
                    if banner.busy {
                        ui.label(RichText::new("Setting up…").color(theme::TEXT_SECONDARY));
                        ui.add(egui::Spinner::new().size(12.0).color(theme::TEXT_INFO));
                    } else if link(ui, "Set up") {
                        action = Some((
                            "toolchain.setup",
                            json!({ "components": banner.components }),
                        ));
                    }
                });
            });
        });
    if let Some((id, params)) = action {
        app.dispatch(id, params);
    }
}
