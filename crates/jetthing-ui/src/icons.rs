//! Vector icons drawn on a 16×16 grid in a thin-outline style.
//!
//! These are original drawings made for Jetthing (no third-party artwork is
//! copied); they follow the general conventions of contemporary IDE icon sets:
//! 1px outlines, rounded joins, and a single neutral color with semantic accents.

use crate::theme;
use eframe::egui::{self, Color32, Pos2, Rect, Shape, Stroke, epaint::PathShape, vec2};

/// Neutral icon color (palette `Gray11`).
pub const ICON: theme::Token = theme::ICON;
const RUST: Color32 = Color32::from_rgb(0xE0, 0x88, 0x55);
const TOML: Color32 = Color32::from_rgb(0x9D, 0xA0, 0xA8);
const MARKDOWN: Color32 = Color32::from_rgb(0x6B, 0x9B, 0xFA);
const JSON: Color32 = Color32::from_rgb(0xF2, 0xC5, 0x5C);
const SCRIPT: Color32 = Color32::from_rgb(0x5F, 0xAD, 0x65);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Icon {
    MainMenu,
    ChevronDown,
    ChevronUp,
    ChevronRight,
    Folder,
    File,
    FileRust,
    FileToml,
    FileMarkdown,
    FileJson,
    FileScript,
    Project,
    Commit,
    Branch,
    Terminal,
    Problems,
    Build,
    Find,
    Run,
    Stop,
    Search,
    Close,
    Minimize,
    Refresh,
    Error,
    Warning,
    /// Phone outline (devices, Device Manager).
    Device,
    Logcat,
    Pause,
    ScrollEnd,
    SoftWrap,
}

impl Icon {
    /// File icon by extension or well-known file name.
    pub fn for_file(name: &str) -> Icon {
        let lower = name.to_ascii_lowercase();
        let extension = lower.rsplit_once('.').map(|(_, ext)| ext).unwrap_or("");
        match extension {
            "rs" => Icon::FileRust,
            "toml" | "lock" => Icon::FileToml,
            "md" | "markdown" => Icon::FileMarkdown,
            "json" | "jsonc" => Icon::FileJson,
            "sh" | "bash" | "zsh" | "py" | "js" | "ts" => Icon::FileScript,
            _ => Icon::File,
        }
    }
}

/// Maps 16×16 design coordinates into `rect`.
struct Grid {
    origin: Pos2,
    scale: f32,
}

impl Grid {
    fn new(rect: Rect) -> Self {
        let side = rect.width().min(rect.height());
        let scale = side / 16.0;
        let origin = rect.center() - vec2(8.0 * scale, 8.0 * scale);
        Self { origin, scale }
    }
    fn p(&self, x: f32, y: f32) -> Pos2 {
        self.origin + vec2(x * self.scale, y * self.scale)
    }
    fn len(&self, value: f32) -> f32 {
        value * self.scale
    }
    fn rect(&self, x0: f32, y0: f32, x1: f32, y1: f32) -> Rect {
        Rect::from_min_max(self.p(x0, y0), self.p(x1, y1))
    }
}

pub fn paint(painter: &egui::Painter, rect: Rect, icon: Icon, color: impl Into<Color32>) {
    let color = color.into();
    let g = Grid::new(rect);
    let stroke = Stroke::new(g.len(1.0).max(1.0), color);
    let line = |points: &[(f32, f32)]| {
        painter.add(Shape::Path(PathShape::line(
            points.iter().map(|&(x, y)| g.p(x, y)).collect(),
            stroke,
        )));
    };
    let closed = |points: &[(f32, f32)], fill: Color32, outline: Stroke| {
        painter.add(Shape::Path(PathShape {
            points: points.iter().map(|&(x, y)| g.p(x, y)).collect(),
            closed: true,
            fill,
            stroke: outline.into(),
        }));
    };
    let page = |accent: Option<Color32>| {
        closed(
            &[
                (3.5, 1.5),
                (9.5, 1.5),
                (12.5, 4.5),
                (12.5, 14.5),
                (3.5, 14.5),
            ],
            Color32::TRANSPARENT,
            Stroke::new(stroke.width, ICON),
        );
        line(&[(9.5, 1.5), (9.5, 4.5), (12.5, 4.5)]);
        if let Some(accent) = accent {
            painter.rect_filled(g.rect(7.0, 8.0, 15.5, 15.5), g.len(1.5), accent);
        }
    };
    let badge_text = |text: &str| {
        painter.text(
            g.p(11.25, 11.9),
            egui::Align2::CENTER_CENTER,
            text,
            theme::semibold(g.len(6.0)),
            *theme::ISLAND,
        );
    };
    match icon {
        Icon::MainMenu => {
            for y in [4.5, 8.0, 11.5] {
                line(&[(2.5, y), (13.5, y)]);
            }
        }
        Icon::ChevronDown => line(&[(4.5, 6.5), (8.0, 10.0), (11.5, 6.5)]),
        Icon::ChevronUp => line(&[(4.5, 9.5), (8.0, 6.0), (11.5, 9.5)]),
        Icon::ChevronRight => line(&[(6.5, 4.5), (10.0, 8.0), (6.5, 11.5)]),
        Icon::Folder | Icon::Project => {
            let fill = if icon == Icon::Folder {
                *theme::FOLDER_FILL
            } else {
                Color32::TRANSPARENT
            };
            closed(
                &[
                    (1.5, 3.5),
                    (6.0, 3.5),
                    (7.5, 5.0),
                    (14.5, 5.0),
                    (14.5, 13.5),
                    (1.5, 13.5),
                ],
                fill,
                stroke,
            );
            if icon == Icon::Project {
                line(&[(1.5, 7.0), (14.5, 7.0)]);
            }
        }
        Icon::File => {
            page(None);
            for y in [7.5, 9.5, 11.5] {
                line(&[(5.5, y), (10.5, y)]);
            }
        }
        Icon::FileRust => {
            page(Some(RUST));
            badge_text("RS");
        }
        Icon::FileToml => {
            page(Some(TOML));
            badge_text("T");
        }
        Icon::FileMarkdown => {
            page(Some(MARKDOWN));
            badge_text("M");
        }
        Icon::FileJson => {
            page(Some(JSON));
            badge_text("{}");
        }
        Icon::FileScript => {
            page(Some(SCRIPT));
            badge_text(">_");
        }
        Icon::Commit => {
            painter.circle_stroke(g.p(8.0, 8.0), g.len(3.0), stroke);
            line(&[(1.0, 8.0), (5.0, 8.0)]);
            line(&[(11.0, 8.0), (15.0, 8.0)]);
        }
        Icon::Branch => {
            painter.circle_stroke(g.p(4.5, 3.5), g.len(1.8), stroke);
            painter.circle_stroke(g.p(4.5, 12.5), g.len(1.8), stroke);
            painter.circle_stroke(g.p(11.5, 4.5), g.len(1.8), stroke);
            line(&[(4.5, 5.3), (4.5, 10.7)]);
            line(&[
                (11.5, 6.3),
                (11.5, 7.5),
                (10.0, 9.0),
                (6.0, 9.5),
                (4.5, 10.7),
            ]);
        }
        Icon::Terminal => {
            painter.rect_stroke(
                g.rect(1.5, 2.5, 14.5, 13.5),
                g.len(2.0),
                stroke,
                egui::StrokeKind::Middle,
            );
            line(&[(4.5, 6.0), (6.5, 8.0), (4.5, 10.0)]);
            line(&[(8.0, 10.5), (11.5, 10.5)]);
        }
        Icon::Problems => {
            painter.circle_stroke(g.p(8.0, 8.0), g.len(6.5), stroke);
            line(&[(8.0, 4.5), (8.0, 9.0)]);
            painter.circle_filled(g.p(8.0, 11.3), g.len(0.9), color);
        }
        Icon::Build => {
            // Hammer: angled head with a claw, and a diagonal handle.
            closed(
                &[
                    (6.0, 2.0),
                    (9.0, 2.0),
                    (14.0, 7.0),
                    (12.5, 8.5),
                    (8.5, 5.5),
                    (5.5, 5.5),
                ],
                Color32::TRANSPARENT,
                stroke,
            );
            closed(
                &[(8.0, 6.5), (9.5, 8.0), (3.5, 14.0), (2.0, 12.5)],
                Color32::TRANSPARENT,
                stroke,
            );
        }
        Icon::Find | Icon::Search => {
            painter.circle_stroke(g.p(7.0, 7.0), g.len(4.5), stroke);
            line(&[(10.3, 10.3), (14.0, 14.0)]);
        }
        Icon::Run => closed(
            &[(4.5, 2.5), (13.0, 8.0), (4.5, 13.5)],
            color,
            Stroke::new(stroke.width, color),
        ),
        Icon::Stop => {
            painter.rect_filled(g.rect(3.5, 3.5, 12.5, 12.5), g.len(1.5), color);
        }
        Icon::Close => {
            line(&[(4.0, 4.0), (12.0, 12.0)]);
            line(&[(12.0, 4.0), (4.0, 12.0)]);
        }
        Icon::Minimize => line(&[(3.5, 8.0), (12.5, 8.0)]),
        Icon::Logcat => {
            // Log lines of varying length with a level marker.
            painter.rect_stroke(
                g.rect(1.5, 2.5, 14.5, 13.5),
                g.len(2.0),
                stroke,
                egui::StrokeKind::Middle,
            );
            painter.circle_filled(g.p(4.5, 6.0), g.len(0.9), color);
            line(&[(6.5, 6.0), (12.0, 6.0)]);
            painter.circle_filled(g.p(4.5, 10.0), g.len(0.9), color);
            line(&[(6.5, 10.0), (10.0, 10.0)]);
        }
        Icon::Pause => {
            line(&[(5.5, 3.5), (5.5, 12.5)]);
            line(&[(10.5, 3.5), (10.5, 12.5)]);
        }
        Icon::ScrollEnd => {
            line(&[(8.0, 2.5), (8.0, 10.5)]);
            line(&[(4.5, 7.5), (8.0, 11.0), (11.5, 7.5)]);
            line(&[(3.5, 13.5), (12.5, 13.5)]);
        }
        Icon::SoftWrap => {
            line(&[(2.5, 4.0), (13.5, 4.0)]);
            line(&[(2.5, 8.0), (12.0, 8.0)]);
            line(&[(12.0, 8.0), (13.5, 9.5), (12.0, 11.5), (8.5, 11.5)]);
            line(&[(10.0, 10.0), (8.5, 11.5), (10.0, 13.0)]);
        }
        Icon::Device => {
            painter.rect_stroke(
                g.rect(4.0, 1.5, 12.0, 14.5),
                g.len(1.5),
                stroke,
                egui::StrokeKind::Middle,
            );
            line(&[(7.0, 12.5), (9.0, 12.5)]);
        }
        Icon::Refresh => {
            let center = g.p(8.0, 8.0);
            let points = (0..=20)
                .map(|step| {
                    let angle = -0.6 + step as f32 / 20.0 * 5.0;
                    center + vec2(angle.cos(), angle.sin()) * g.len(5.5)
                })
                .collect::<Vec<_>>();
            let tip = *points.last().unwrap_or(&center);
            painter.add(Shape::Path(PathShape::line(points, stroke)));
            painter.line_segment([tip, tip + vec2(g.len(3.0), g.len(0.3))], stroke);
            painter.line_segment([tip, tip + vec2(g.len(0.3), g.len(3.0))], stroke);
        }
        Icon::Error => {
            painter.circle_filled(g.p(8.0, 8.0), g.len(6.5), theme::ERROR);
            let white = Stroke::new(g.len(1.5).max(1.0), theme::ISLAND);
            painter.line_segment([g.p(8.0, 4.5), g.p(8.0, 9.0)], white);
            painter.circle_filled(g.p(8.0, 11.3), g.len(1.0), theme::ISLAND);
        }
        Icon::Warning => {
            closed(
                &[(8.0, 1.5), (15.0, 14.0), (1.0, 14.0)],
                *theme::WARNING,
                Stroke::new(stroke.width, theme::WARNING),
            );
            let dark = Stroke::new(g.len(1.5).max(1.0), theme::ISLAND);
            painter.line_segment([g.p(8.0, 6.0), g.p(8.0, 10.0)], dark);
            painter.circle_filled(g.p(8.0, 12.0), g.len(0.9), theme::ISLAND);
        }
    }
}

/// Allocates a square icon of `size` points and paints it.
pub fn show(ui: &mut egui::Ui, icon: Icon, size: f32, color: impl Into<Color32>) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(vec2(size, size), egui::Sense::hover());
    paint(ui.painter(), rect, icon, color);
    response
}

/// Toolbar-style icon button: transparent until hovered, rounded hover/pressed fill.
pub fn button(ui: &mut egui::Ui, icon: Icon, tooltip: &str) -> egui::Response {
    button_colored(ui, icon, *ICON, tooltip, false)
}

pub fn button_colored(
    ui: &mut egui::Ui,
    icon: Icon,
    color: impl Into<Color32>,
    tooltip: &str,
    selected: bool,
) -> egui::Response {
    let color = color.into();
    let size = vec2(26.0, 26.0);
    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());
    let fill = if selected {
        *theme::ACCENT
    } else if response.is_pointer_button_down_on() {
        *theme::PRESSED
    } else if response.hovered() {
        *theme::HOVER
    } else {
        Color32::TRANSPARENT
    };
    ui.painter().rect_filled(rect, 6.0, fill);
    let color = if selected { Color32::WHITE } else { color };
    paint(
        ui.painter(),
        Rect::from_center_size(rect.center(), vec2(16.0, 16.0)),
        icon,
        color,
    );
    response.on_hover_text(tooltip)
}

#[cfg(test)]
mod tests {
    use super::*;
    use eframe::egui::pos2;

    #[test]
    fn file_icons_follow_extensions() {
        assert_eq!(Icon::for_file("lib.rs"), Icon::FileRust);
        assert_eq!(Icon::for_file("Cargo.toml"), Icon::FileToml);
        assert_eq!(Icon::for_file("README.MD"), Icon::FileMarkdown);
        assert_eq!(Icon::for_file("Makefile"), Icon::File);
    }

    #[test]
    fn grid_maps_center_of_any_square() {
        let grid = Grid::new(Rect::from_min_size(pos2(10.0, 10.0), vec2(32.0, 32.0)));
        assert_eq!(grid.p(8.0, 8.0), pos2(26.0, 26.0));
        assert_eq!(grid.len(1.0), 2.0);
    }
}
