//! Project opening entry point for launches without a project argument.

use crate::{IdeApp, keys, theme, widgets};
use eframe::egui::{self, RichText};

pub fn show(app: &mut IdeApp, ctx: &egui::Context) {
    egui::CentralPanel::default()
        .frame(widgets::island_frame().inner_margin(32))
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space((ui.available_height() * 0.24).max(24.0));
                ui.label(
                    RichText::new("Jetthing")
                        .font(theme::semibold(32.0))
                        .color(theme::TEXT_BRIGHT),
                );
                ui.add_space(10.0);
                ui.label(RichText::new("Welcome to Jetthing").size(18.0));
                ui.add_space(8.0);
                ui.label(
                    RichText::new("Open a project folder to start working.")
                        .color(theme::TEXT_SECONDARY),
                );
                ui.add_space(24.0);
                if ui
                    .add_sized([180.0, 36.0], egui::Button::new("Open Project…"))
                    .clicked()
                {
                    app.open_project_dialog();
                }
                ui.add_space(10.0);
                ui.label(
                    RichText::new(format!("Open Project: {}", keys::display("⌘O")))
                        .color(theme::TEXT_INFO),
                );
                if !app.state.status.is_empty() {
                    ui.add_space(24.0);
                    ui.label(RichText::new(&app.state.status).color(theme::TEXT_SECONDARY));
                }
            });
        });
}
