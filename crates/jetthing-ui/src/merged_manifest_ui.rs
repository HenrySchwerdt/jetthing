//! Merged Manifest view: merged lines tinted by the manifest they come from,
//! with the "Manifest Sources" legend; clicking a project line opens it.

use crate::{IdeApp, theme};
use eframe::egui::{self, Color32, RichText, vec2};
use jetthing_android::merged_manifest::ManifestSource;
use serde_json::json;

fn tint(index: usize) -> Color32 {
    const TINTS: [Color32; 6] = [
        Color32::from_rgba_premultiplied(0x23, 0x2C, 0x3D, 0xFF),
        Color32::from_rgba_premultiplied(0x2E, 0x34, 0x26, 0xFF),
        Color32::from_rgba_premultiplied(0x3A, 0x2B, 0x2E, 0xFF),
        Color32::from_rgba_premultiplied(0x2A, 0x36, 0x38, 0xFF),
        Color32::from_rgba_premultiplied(0x38, 0x33, 0x24, 0xFF),
        Color32::from_rgba_premultiplied(0x33, 0x2A, 0x3A, 0xFF),
    ];
    TINTS[index % TINTS.len()]
}

pub fn show(app: &mut IdeApp, ctx: &egui::Context) {
    if !app.state.android.merged_manifest.open {
        return;
    }
    let root = app
        .state
        .project
        .as_ref()
        .map(|p| p.root().to_owned())
        .unwrap_or_default();
    let screen = ctx.content_rect();
    let size = vec2(
        (screen.width() - 80.0).clamp(480.0, 1200.0),
        (screen.height() - 120.0).clamp(360.0, 820.0),
    );
    let mut close = ctx.input(|i| i.key_pressed(egui::Key::Escape));
    let mut open: Option<(std::path::PathBuf, usize)> = None;
    let view = &app.state.android.merged_manifest;
    egui::Area::new(egui::Id::new("merged_manifest"))
        .order(egui::Order::Foreground)
        .fixed_pos(screen.center() - size / 2.0)
        .show(ctx, |ui| {
            egui::Frame::popup(ui.style())
                .corner_radius(10)
                .inner_margin(egui::Margin::same(16))
                .show(ui, |ui| {
                    ui.set_width(size.x - 32.0);
                    ui.set_height(size.y - 32.0);
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(&view.title)
                                .font(theme::semibold(theme::UI_FONT_SIZE))
                                .color(theme::TEXT_BRIGHT),
                        );
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("Close").clicked() {
                                close = true;
                            }
                        });
                    });
                    ui.add_space(8.0);
                    let merged = match &view.result {
                        Some(Ok(merged)) => merged,
                        Some(Err(error)) => {
                            ui.label(RichText::new(error).color(theme::ERROR));
                            return;
                        }
                        None => return,
                    };
                    let legend_width = 360.0;
                    ui.horizontal_top(|ui| {
                        let font = egui::FontId::monospace(theme::EDITOR_FONT_SIZE);
                        let row = ui.fonts_mut(|f| f.row_height(&font)) + 2.0;
                        let code_width = ui.available_width() - legend_width - 16.0;
                        ui.vertical(|ui| {
                            ui.set_width(code_width);
                            ui.set_height(ui.available_height());
                            egui::ScrollArea::both()
                                .id_salt("merged_manifest_lines")
                                .auto_shrink([false, false])
                                .show_rows(ui, row, merged.lines.len(), |ui, range| {
                                    ui.spacing_mut().item_spacing.y = 0.0;
                                    for line in &merged.lines[range] {
                                        let (rect, response) = ui.allocate_exact_size(
                                            vec2(ui.available_width().max(code_width), row),
                                            egui::Sense::click(),
                                        );
                                        ui.painter().rect_filled(rect, 0.0, tint(line.source));
                                        ui.painter().text(
                                            rect.left_center() + vec2(6.0, 0.0),
                                            egui::Align2::LEFT_CENTER,
                                            &line.text,
                                            font.clone(),
                                            theme::TEXT,
                                        );
                                        let source = &merged.sources[line.source];
                                        let response = response.on_hover_text(source.label(&root));
                                        if response.clicked()
                                            && let (ManifestSource::File { path, .. }, Some(at)) =
                                                (source, line.source_line)
                                        {
                                            open = Some((path.clone(), at));
                                        }
                                    }
                                });
                        });
                        ui.add_space(16.0);
                        ui.vertical(|ui| {
                            ui.set_width(legend_width);
                            ui.label(
                                RichText::new("Manifest Sources")
                                    .font(theme::semibold(theme::UI_FONT_SIZE))
                                    .underline()
                                    .color(theme::TEXT_BRIGHT),
                            );
                            ui.add_space(6.0);
                            for (index, source) in merged.sources.iter().enumerate() {
                                ui.horizontal(|ui| {
                                    let (swatch, _) = ui.allocate_exact_size(
                                        vec2(22.0, 22.0),
                                        egui::Sense::hover(),
                                    );
                                    ui.painter().rect_filled(swatch, 2.0, tint(index));
                                    ui.painter().rect_stroke(
                                        swatch,
                                        2.0,
                                        egui::Stroke::new(1.0_f32, theme::BORDER),
                                        egui::StrokeKind::Inside,
                                    );
                                    ui.label(
                                        RichText::new(source.label(&root)).color(theme::MODIFIED),
                                    );
                                });
                            }
                            ui.add_space(10.0);
                            ui.label(
                                RichText::new(format!(
                                    "Blame report: {}",
                                    merged.blame_file.display()
                                ))
                                .color(theme::TEXT_INFO),
                            );
                        });
                    });
                });
        });
    if let Some((path, line)) = open {
        let relative = path
            .strip_prefix(&root)
            .map(|p| p.to_owned())
            .unwrap_or(path);
        app.dispatch("editor.openFile", json!({"path": relative}));
        app.dispatch("editor.gotoLine", json!({"line": line}));
        close = true;
    }
    if close {
        app.state.android.merged_manifest.open = false;
    }
}
