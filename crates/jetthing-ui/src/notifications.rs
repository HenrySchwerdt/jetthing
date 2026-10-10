//! Notification balloon (bottom right), used for IDE updates.

use crate::{IdeApp, theme};
use eframe::egui::{self, RichText};
use jetthing_core::{CURRENT_VERSION, UpdateState};
use serde_json::Value;

/// Restarts the IDE with the same arguments (after an update was installed).
pub fn restart(ctx: &egui::Context) {
    if let Ok(exe) = std::env::current_exe() {
        let args: Vec<_> = std::env::args_os().skip(1).collect();
        if std::process::Command::new(exe).args(args).spawn().is_ok() {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }
}

pub fn show(app: &mut IdeApp, ctx: &egui::Context) {
    if !app.state.update_notification {
        return;
    }
    let screen = ctx.content_rect();
    let width = 340.0;
    let mut action: Option<&'static str> = None;
    let mut open_url = None;
    let mut do_restart = false;
    egui::Area::new(egui::Id::new("notification_balloon"))
        .order(egui::Order::Foreground)
        .fixed_pos(egui::pos2(
            screen.right() - width - 16.0,
            screen.bottom() - 150.0 - crate::chrome::STATUS_BAR_HEIGHT,
        ))
        .show(ctx, |ui| {
            egui::Frame::popup(ui.style())
                .fill(theme::NOTIFICATION)
                .stroke(egui::Stroke::new(1.0_f32, theme::NOTIFICATION))
                .corner_radius(8)
                .inner_margin(egui::Margin::same(14))
                .show(ui, |ui| {
                    ui.set_width(width - 28.0);
                    let (title, body) = match &app.state.update_state {
                        UpdateState::Available { version, .. } => (
                            format!("Jetthing {version} is available"),
                            format!("You are using {CURRENT_VERSION}."),
                        ),
                        UpdateState::Installing(version) => (
                            format!("Updating to {version}…"),
                            "Downloading and verifying the release.".into(),
                        ),
                        UpdateState::Installed(version) => (
                            format!("Jetthing {version} installed"),
                            "Restart the IDE to finish the update.".into(),
                        ),
                        UpdateState::UpToDate(_) => (
                            "You have the latest version".into(),
                            format!("Jetthing {CURRENT_VERSION} is up to date."),
                        ),
                        UpdateState::Failed(error) => ("Update check failed".into(), error.clone()),
                        UpdateState::Checking => ("Checking for updates…".into(), String::new()),
                        UpdateState::Idle => (String::new(), String::new()),
                    };
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(title)
                                .font(theme::semibold(theme::UI_FONT_SIZE))
                                .color(theme::TEXT_BRIGHT),
                        );
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if crate::icons::button(ui, crate::icons::Icon::Close, "Dismiss")
                                .clicked()
                            {
                                action = Some("app.dismissUpdate");
                            }
                        });
                    });
                    if !body.is_empty() {
                        ui.label(RichText::new(body).color(theme::TEXT));
                    }
                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        let link = |ui: &mut egui::Ui, text: &str| {
                            ui.add(
                                egui::Label::new(RichText::new(text).color(theme::MODIFIED))
                                    .sense(egui::Sense::click()),
                            )
                            .on_hover_cursor(egui::CursorIcon::PointingHand)
                            .clicked()
                        };
                        match &app.state.update_state {
                            UpdateState::Available { release, .. } => {
                                let installable = jetthing_core::update_installable().is_ok();
                                if installable {
                                    if link(ui, "Update and Restart") {
                                        action = Some("app.installUpdate");
                                    }
                                } else if link(ui, "Download") {
                                    open_url = Some(release.html_url.clone());
                                }
                                ui.add_space(12.0);
                                if link(ui, "Release Notes") {
                                    open_url = Some(release.html_url.clone());
                                }
                            }
                            UpdateState::Installing(_) | UpdateState::Checking => {
                                ui.add(egui::Spinner::new().size(14.0).color(theme::ACCENT));
                            }
                            UpdateState::Installed(_) if link(ui, "Restart") => {
                                do_restart = true;
                            }
                            _ => {}
                        }
                    });
                });
        });
    if let Some(url) = open_url {
        ctx.open_url(egui::OpenUrl::new_tab(url));
    }
    if do_restart {
        app.dispatch("workspace.save", Value::Null);
        restart(ctx);
    }
    if let Some(id) = action {
        app.dispatch(id, Value::Null);
        // Keep the balloon for progress after starting an install.
        if id == "app.installUpdate" {
            app.state.update_notification = true;
        }
    }
    if matches!(
        app.state.update_state,
        UpdateState::Checking | UpdateState::Installing(_)
    ) {
        ctx.request_repaint_after(std::time::Duration::from_millis(200));
    }
}
