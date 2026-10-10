//! Gutter run icons and their popup (Run / Debug / Modify Run Configuration…).

use crate::icons::{self, Icon};
use crate::{IdeApp, keys, theme};
use eframe::egui::{self, vec2};
use jetthing_core::RunnableKind;
use serde_json::json;

#[derive(Clone, Copy)]
struct Open {
    line: usize,
    position: egui::Pos2,
    /// Pass in which the popup opened; that pass's click must not close it.
    opened: u64,
}

fn state_id() -> egui::Id {
    egui::Id::new("gutter_run_popup")
}

/// Paints a run icon centred at `center` (single test, test class/module or main).
pub fn paint_icon(painter: &egui::Painter, center: egui::Pos2, kind: RunnableKind) {
    let rect = egui::Rect::from_center_size(center, vec2(13.0, 13.0));
    match kind {
        RunnableKind::Main | RunnableKind::Test => {
            icons::paint(painter, rect, Icon::Run, theme::RUN_GREEN)
        }
        RunnableKind::TestContainer => icons::paint(painter, rect, Icon::Rerun, theme::RUN_GREEN),
    }
}

pub fn open(ctx: &egui::Context, line: usize, position: egui::Pos2) {
    let opened = ctx.cumulative_pass_nr();
    ctx.data_mut(|data| {
        data.insert_temp(
            state_id(),
            Some(Open {
                line,
                position,
                opened,
            }),
        )
    });
}

pub fn show(app: &mut IdeApp, ctx: &egui::Context) {
    let Some(Some(open)) = ctx.data(|data| data.get_temp::<Option<Open>>(state_id())) else {
        return;
    };
    let marks = app.state.gutter_marks();
    let Some(mark) = marks.iter().find(|mark| mark.line == open.line).cloned() else {
        close(ctx);
        return;
    };
    let target = mark.label.trim_start_matches("Run ").to_owned();
    let mut action = None;
    let area = egui::Area::new(egui::Id::new("gutter_run_popup_area"))
        .order(egui::Order::Foreground)
        .fixed_pos(open.position + vec2(8.0, 4.0))
        .show(ctx, |ui| {
            egui::Frame::popup(ui.style()).show(ui, |ui| {
                ui.set_min_width(280.0);
                let running = app.state.runner.is_running();
                let run = ui.add_enabled(
                    !running,
                    egui::Button::new(format!("▶  {}", mark.label))
                        .frame(false)
                        .shortcut_text(keys::display("⌃⇧R"))
                        .min_size(vec2(270.0, 24.0)),
                );
                if run.clicked() {
                    action = Some(false);
                }
                ui.add_enabled(
                    false,
                    egui::Button::new(format!("     Debug {target}"))
                        .frame(false)
                        .min_size(vec2(270.0, 24.0)),
                )
                .on_disabled_hover_text("The debugger is not available yet");
                if ui
                    .add(
                        egui::Button::new("     Modify Run Configuration…")
                            .frame(false)
                            .min_size(vec2(270.0, 24.0)),
                    )
                    .clicked()
                {
                    action = Some(true);
                }
            });
        });
    let escape = ctx.input(|input| input.key_pressed(egui::Key::Escape));
    let clicked_elsewhere = ctx.cumulative_pass_nr() > open.opened
        && ctx.input(|input| input.pointer.any_click())
        && !area.response.rect.contains(
            ctx.input(|input| input.pointer.interact_pos())
                .unwrap_or_default(),
        )
        && action.is_none();
    if let Some(edit) = action {
        close(ctx);
        app.dispatch("run.gutter", json!({"line": open.line, "edit": edit}));
    } else if escape || clicked_elsewhere {
        close(ctx);
    }
}

fn close(ctx: &egui::Context) {
    ctx.data_mut(|data| data.insert_temp::<Option<Open>>(state_id(), None));
}
