//! Language server status bar widget and notification balloons.

use crate::{IdeApp, icons::Icon, theme, widgets};
use eframe::egui::{self, RichText, vec2};
use jetthing_lsp::{ServerInfo, ServerStatus, describe_progress};
use serde_json::{Value, json};

/// Short status for the server of the active document, or running progress.
fn summary(app: &IdeApp, servers: &[ServerInfo]) -> Option<(String, bool, bool)> {
    // Background work anywhere is shown first, like the reference IDE's
    // "Indexing…" status bar entry.
    if let Some((server, progress)) = servers
        .iter()
        .find_map(|server| Some((server, server.progress.first()?)))
    {
        return Some((
            format!("{}: {}", server.id, describe_progress(progress)),
            true,
            false,
        ));
    }
    let active = app.state.active_absolute_path()?;
    let (provider, _) = app.state.lsp.registry().for_path(&active)?;
    let id = provider.spec().id;
    if app.state.lsp.missing_servers().any(|missing| missing == id) {
        return Some((format!("{id}: not installed"), false, true));
    }
    let server = servers
        .iter()
        .filter(|server| server.id == id && active.starts_with(&server.root))
        .max_by_key(|server| server.root.components().count())?;
    Some(match &server.status {
        ServerStatus::Starting => (
            match &server.message {
                Some(message) => format!("{id}: {message}"),
                None => format!("{id}: starting…"),
            },
            true,
            false,
        ),
        // A healthy server needs no text; the widget stays as an icon-only entry.
        ServerStatus::Ready => (String::new(), false, false),
        ServerStatus::Restarting { attempt } => {
            (format!("{id}: restarting ({attempt})…"), true, true)
        }
        ServerStatus::Failed { .. } => (format!("{id}: failed"), false, true),
    })
}

/// Status bar entry; hover lists every server, right-click offers a restart.
pub fn status_widget(app: &mut IdeApp, ui: &mut egui::Ui, font: egui::FontId) {
    let servers = app.state.lsp.servers();
    let Some((text, busy, problem)) = summary(app, &servers) else {
        return;
    };
    let icon = if problem {
        Some((Icon::Warning, *theme::WARNING))
    } else if text.is_empty() {
        Some((Icon::Success, *theme::TEXT_INFO))
    } else {
        None
    };
    let color = if problem {
        *theme::TEXT_SECONDARY
    } else if busy {
        *theme::TEXT_INFO
    } else {
        *theme::TEXT_SECONDARY
    };
    let response = widgets::flat_widget(ui, icon, &text, font, color, false, 22.0);
    if busy {
        ui.add(egui::Spinner::new().size(12.0).color(theme::TEXT_INFO));
    }
    let response = response.on_hover_ui(|ui| {
        ui.set_max_width(420.0);
        ui.label(RichText::new("Language servers").strong());
        if servers.is_empty() {
            ui.label("No language server is running.");
        }
        for server in &servers {
            let state = match &server.status {
                ServerStatus::Starting => "starting".to_owned(),
                ServerStatus::Ready => "ready".to_owned(),
                ServerStatus::Restarting { attempt } => format!("restarting (attempt {attempt})"),
                ServerStatus::Failed { reason } => {
                    format!("failed: {}", reason.lines().next().unwrap_or_default())
                }
            };
            ui.label(format!(
                "{} ({}) — {state}\n{}",
                server.id,
                server.language,
                server.root.display()
            ));
            for progress in &server.progress {
                ui.label(RichText::new(describe_progress(progress)).color(theme::TEXT_INFO));
            }
        }
        for missing in app.state.lsp.missing_servers() {
            ui.label(format!("{missing} — not installed"));
        }
        ui.label(RichText::new("Right-click for actions").color(theme::TEXT_INFO));
    });
    let mut restart = false;
    response.context_menu(|ui| {
        if ui.button("Restart Language Servers").clicked() {
            restart = true;
            ui.close();
        }
    });
    if restart {
        app.dispatch("language.restartServers", Value::Null);
    }
}

/// Balloons for missing or crashing servers, stacked above the update balloon.
pub fn notices(app: &mut IdeApp, ctx: &egui::Context) {
    let notices = app.state.lsp.notices().to_vec();
    if notices.is_empty() {
        return;
    }
    let width = 380.0;
    let mut bottom = crate::chrome::STATUS_BAR_HEIGHT + 16.0;
    if app.state.update_notification {
        bottom += 170.0;
    }
    let mut dismiss = None;
    let mut restart = false;
    let mut copy = None;
    let mut setup = None;
    egui::Area::new(egui::Id::new("lsp_notices"))
        .order(crate::notifications::BALLOON_ORDER)
        .anchor(egui::Align2::RIGHT_BOTTOM, vec2(-16.0, -bottom))
        .show(ctx, |ui| {
            ui.set_width(width);
            for notice in notices.iter().rev().take(3) {
                egui::Frame::popup(ui.style())
                    .fill(*theme::NOTIFICATION)
                    .stroke(egui::Stroke::new(1.0_f32, theme::NOTIFICATION))
                    .corner_radius(8)
                    .inner_margin(egui::Margin::same(14))
                    .show(ui, |ui| {
                        ui.set_width(width - 28.0);
                        ui.horizontal(|ui| {
                            let icon = if notice.error {
                                (Icon::Error, *theme::ERROR)
                            } else {
                                (Icon::Warning, *theme::WARNING)
                            };
                            crate::icons::show(ui, icon.0, 16.0, icon.1);
                            ui.label(
                                RichText::new(&notice.title)
                                    .font(theme::semibold(theme::UI_FONT_SIZE))
                                    .color(theme::TEXT_BRIGHT),
                            );
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if crate::icons::button(ui, Icon::Close, "Dismiss").clicked() {
                                        dismiss = Some(notice.id.clone());
                                    }
                                },
                            );
                        });
                        ui.label(RichText::new(&notice.body).color(theme::TEXT));
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
                            if let Some(component) = &notice.setup
                                && link(ui, "Set up")
                            {
                                setup = Some((notice.id.clone(), component.clone()));
                            }
                            ui.add_space(12.0);
                            if notice.id.starts_with("missing:") && link(ui, "Copy Install Hint") {
                                copy = Some(notice.body.clone());
                            }
                            ui.add_space(12.0);
                            if link(ui, "Restart Language Servers") {
                                restart = true;
                            }
                        });
                    });
                ui.add_space(8.0);
            }
        });
    if let Some(text) = copy {
        ctx.copy_text(text);
    }
    if let Some(id) = dismiss {
        app.dispatch("language.dismissNotice", json!({ "id": id }));
    }
    if restart {
        app.dispatch("language.restartServers", Value::Null);
    }
    if let Some((id, component)) = setup {
        // Installs the server and what it needs (jdtls → JDK); servers restart
        // when the setup finishes.
        app.dispatch("toolchain.setup", json!({ "components": [component] }));
        app.dispatch("language.dismissNotice", json!({ "id": id }));
    }
}
