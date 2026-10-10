//! Vector icons drawn on a 16×16 grid in a thin-outline style.
//!
//! These are original drawings made for Jetthing (no third-party artwork is
//! copied); they follow the general conventions of contemporary IDE icon sets:
//! 1px outlines, rounded joins, and a single neutral color with semantic accents.

use crate::theme;
use eframe::egui::{self, Color32, Pos2, Rect, Shape, Stroke, epaint::PathShape, vec2};
use std::path::{Component, Path};

/// Neutral icon color (palette `Gray11`).
pub const ICON: theme::Token = theme::ICON;
const fn rgb(r: u8, g: u8, b: u8) -> Color32 {
    Color32::from_rgb(r, g, b)
}
const SOURCE_ROOT: Color32 = rgb(0x54, 0x8A, 0xF7);
const TEST_ROOT: Color32 = rgb(0x5F, 0xAD, 0x65);
const RESOURCES_ROOT: Color32 = rgb(0xB1, 0x8A, 0xDB);
const EXCLUDED: Color32 = rgb(0xD0, 0x8A, 0x4B);
const IMAGE: Color32 = rgb(0x5F, 0xAD, 0x65);
const ARCHIVE: Color32 = rgb(0xD9, 0xB0, 0x4C);
const HIDDEN: Color32 = rgb(0x6C, 0x70, 0x7E);

/// A page icon with a coloured badge holding one or two letters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Badge {
    pub label: &'static str,
    pub color: Color32,
}

const fn badge(label: &'static str, color: Color32) -> Badge {
    Badge { label, color }
}

/// What a folder is for, shown by its colour like the reference IDE's project view.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FolderKind {
    Plain,
    /// `src/main/java`, `src/main/kotlin`: blue.
    Source,
    /// `src/test/java`, `src/androidTest/kotlin`: green.
    Test,
    /// `src/*/resources`, `src/*/res`.
    Resources,
    /// `build`, `target`, `out`, `node_modules`: orange.
    Excluded,
    /// `.git`, `.idea`, `.gradle`: dimmed.
    Hidden,
    /// A directory inside a source root (a Java package).
    Package,
}

impl FolderKind {
    /// Classifies a project-relative directory path.
    pub fn classify(path: &Path) -> FolderKind {
        let parts: Vec<String> = path
            .components()
            .filter_map(|c| match c {
                Component::Normal(part) => Some(part.to_string_lossy().into_owned()),
                _ => None,
            })
            .collect();
        let Some(last) = parts.last() else {
            return FolderKind::Plain;
        };
        if HIDDEN_DIRS.contains(&last.as_str()) {
            return FolderKind::Hidden;
        }
        if parts.iter().any(|p| EXCLUDED_DIRS.contains(&p.as_str())) {
            return FolderKind::Excluded;
        }
        // `src/<set>/<kind>[/package…]`
        if let Some(i) = (0..parts.len().saturating_sub(2)).find(|&i| parts[i] == "src") {
            let set = parts[i + 1].to_ascii_lowercase();
            let is_test = set.starts_with("test") || set.starts_with("androidtest");
            let depth = parts.len() - (i + 3);
            match parts[i + 2].as_str() {
                "java" | "kotlin" if depth > 0 => return FolderKind::Package,
                "java" | "kotlin" if is_test => return FolderKind::Test,
                "java" | "kotlin" => return FolderKind::Source,
                "resources" | "res" if depth == 0 => return FolderKind::Resources,
                _ => {}
            }
        }
        FolderKind::Plain
    }

    fn fill(self) -> Color32 {
        match self {
            FolderKind::Plain | FolderKind::Package => *theme::FOLDER_FILL,
            FolderKind::Source => SOURCE_ROOT.gamma_multiply(0.45),
            FolderKind::Test => TEST_ROOT.gamma_multiply(0.45),
            FolderKind::Resources => RESOURCES_ROOT.gamma_multiply(0.45),
            FolderKind::Excluded => EXCLUDED.gamma_multiply(0.45),
            FolderKind::Hidden => Color32::TRANSPARENT,
        }
    }

    fn outline(self) -> Option<Color32> {
        match self {
            FolderKind::Plain | FolderKind::Package => None,
            FolderKind::Source => Some(SOURCE_ROOT),
            FolderKind::Test => Some(TEST_ROOT),
            FolderKind::Resources => Some(RESOURCES_ROOT),
            FolderKind::Excluded => Some(EXCLUDED),
            FolderKind::Hidden => Some(HIDDEN),
        }
    }
}

const HIDDEN_DIRS: [&str; 4] = [".git", ".idea", ".gradle", ".jetthing"];
const EXCLUDED_DIRS: [&str; 5] = ["build", "target", "out", "node_modules", "dist"];

/// Exact (lower-case) file names; checked before extensions.
const NAME_BADGES: &[(&str, Badge)] = &[
    ("androidmanifest.xml", badge("An", rgb(0x5F, 0xAD, 0x65))),
    ("dockerfile", badge("D", rgb(0x4F, 0x9B, 0xE0))),
    (".gitignore", badge("Gi", rgb(0xE0, 0x70, 0x55))),
    (".gitattributes", badge("Gi", rgb(0xE0, 0x70, 0x55))),
    (".editorconfig", badge("Ec", rgb(0x9D, 0xA0, 0xA8))),
    ("license", badge("Li", rgb(0xD9, 0xB0, 0x4C))),
    ("licence", badge("Li", rgb(0xD9, 0xB0, 0x4C))),
    ("copying", badge("Li", rgb(0xD9, 0xB0, 0x4C))),
    ("license.md", badge("Li", rgb(0xD9, 0xB0, 0x4C))),
    ("license.txt", badge("Li", rgb(0xD9, 0xB0, 0x4C))),
    ("readme", badge("Rd", rgb(0x6B, 0x9B, 0xFA))),
    ("readme.md", badge("Rd", rgb(0x6B, 0x9B, 0xFA))),
    ("readme.txt", badge("Rd", rgb(0x6B, 0x9B, 0xFA))),
    ("gradlew", badge("G", rgb(0x4F, 0xA8, 0xA0))),
    ("gradlew.bat", badge("G", rgb(0x4F, 0xA8, 0xA0))),
    ("makefile", badge("Mk", rgb(0x9D, 0xA0, 0xA8))),
];

/// Extensions (lower-case, without the dot) per badge; `gradle.kts` and similar
/// compound extensions are matched by suffix.
const EXTENSION_BADGES: &[(&[&str], Badge)] = &[
    (&["java"], badge("J", rgb(0xE0, 0x88, 0x55))),
    (&["kt"], badge("K", rgb(0xA9, 0x7B, 0xF0))),
    (&["kts"], badge("Ks", rgb(0xA9, 0x7B, 0xF0))),
    (&["gradle", "gradle.kts"], badge("G", rgb(0x4F, 0xA8, 0xA0))),
    (&["groovy", "gvy"], badge("Gr", rgb(0x4F, 0x9B, 0xE0))),
    (&["xml", "xsd", "xsl"], badge("<>", rgb(0xE0, 0x9A, 0x55))),
    (&["json", "jsonc"], badge("{}", rgb(0xF2, 0xC5, 0x5C))),
    (&["yaml", "yml"], badge("Y", rgb(0xC0, 0x7B, 0x9B))),
    (&["toml"], badge("T", rgb(0x9D, 0xA0, 0xA8))),
    (&["properties"], badge("Pr", rgb(0x9D, 0xA0, 0xA8))),
    (&["lock"], badge("Lk", rgb(0x9D, 0xA0, 0xA8))),
    (&["md", "markdown"], badge("M", rgb(0x6B, 0x9B, 0xFA))),
    (&["rs"], badge("RS", rgb(0xE0, 0x88, 0x55))),
    (&["go"], badge("Go", rgb(0x4F, 0xB8, 0xD0))),
    (&["py"], badge("Py", rgb(0x5C, 0x8F, 0xD6))),
    (&["js", "mjs", "cjs"], badge("JS", rgb(0xF2, 0xC5, 0x5C))),
    (&["ts"], badge("TS", rgb(0x4F, 0x9B, 0xE0))),
    (&["tsx", "jsx"], badge("Tx", rgb(0x4F, 0xB8, 0xD0))),
    (&["html", "htm"], badge("H", rgb(0xE0, 0x70, 0x55))),
    (&["css", "scss"], badge("C", rgb(0x6B, 0x9B, 0xFA))),
    (&["sh", "bash", "zsh"], badge(">_", rgb(0x5F, 0xAD, 0x65))),
    (&["bat", "cmd", "ps1"], badge(">_", rgb(0x5F, 0xAD, 0x65))),
    (&["sql"], badge("Sq", rgb(0xC0, 0x7B, 0x9B))),
    (&["proto"], badge("Pb", rgb(0x4F, 0xA8, 0xA0))),
    (&["csv", "tsv"], badge("Cs", rgb(0x5F, 0xAD, 0x65))),
];

const IMAGE_EXTENSIONS: [&str; 7] = ["png", "jpg", "jpeg", "svg", "webp", "gif", "ico"];
const ARCHIVE_EXTENSIONS: [&str; 7] = ["jar", "zip", "tar", "gz", "tgz", "war", "aar"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Icon {
    MainMenu,
    ChevronDown,
    ChevronUp,
    ChevronRight,
    Folder,
    File,
    /// A page with a coloured letter badge (language and config files).
    FileBadge(Badge),
    FileImage,
    FileArchive,
    /// A folder tinted by its role.
    FolderKind(FolderKind),
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
    /// Curved arrow back (rollback / revert).
    Rollback,
    /// Two-pane compare (show diff).
    Diff,
    Error,
    Warning,
    /// Phone outline (devices, Device Manager).
    Device,
    Logcat,
    Pause,
    ScrollEnd,
    SoftWrap,
    WeakWarning,
    Pin,
    Success,
    Rerun,
    /// Stacked squares (Build Variants).
    Variants,
}

impl Icon {
    /// File icon by well-known file name, then extension; unknown files get the
    /// generic page.
    pub fn for_file(name: &str) -> Icon {
        let lower = name.to_ascii_lowercase();
        if let Some((_, badge)) = NAME_BADGES.iter().find(|(known, _)| *known == lower) {
            return Icon::FileBadge(*badge);
        }
        // Compound extensions first (`build.gradle.kts` is Gradle, not Kotlin script).
        let has_ext = |ext: &str| {
            lower
                .strip_suffix(ext)
                .is_some_and(|rest| rest.ends_with('.') && rest.len() > 1)
        };
        if IMAGE_EXTENSIONS.iter().any(|e| has_ext(e)) {
            return Icon::FileImage;
        }
        if ARCHIVE_EXTENSIONS.iter().any(|e| has_ext(e)) {
            return Icon::FileArchive;
        }
        EXTENSION_BADGES
            .iter()
            .filter_map(|(exts, badge)| {
                exts.iter()
                    .filter(|e| has_ext(e))
                    .map(|e| e.len())
                    .max()
                    .map(|len| (len, badge))
            })
            .max_by_key(|(len, _)| *len)
            .map(|(_, badge)| Icon::FileBadge(*badge))
            .unwrap_or(Icon::File)
    }

    /// Folder icon by project-relative path.
    pub fn for_folder(path: &Path) -> Icon {
        match FolderKind::classify(path) {
            FolderKind::Plain => Icon::Folder,
            kind => Icon::FolderKind(kind),
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
        Icon::Folder | Icon::Project | Icon::FolderKind(_) => {
            let kind = match icon {
                Icon::FolderKind(kind) => kind,
                _ => FolderKind::Plain,
            };
            let fill = if icon == Icon::Project {
                Color32::TRANSPARENT
            } else {
                kind.fill()
            };
            let outline = kind
                .outline()
                .map(|color| Stroke::new(stroke.width, color))
                .unwrap_or(stroke);
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
                outline,
            );
            if icon == Icon::Project {
                line(&[(1.5, 7.0), (14.5, 7.0)]);
            }
            if kind == FolderKind::Package {
                painter.circle_filled(g.p(8.0, 9.5), g.len(1.6), color);
            }
        }
        Icon::File => {
            page(None);
            for y in [7.5, 9.5, 11.5] {
                line(&[(5.5, y), (10.5, y)]);
            }
        }
        Icon::FileBadge(badge) => {
            page(Some(badge.color));
            badge_text(badge.label);
        }
        Icon::FileImage => {
            page(None);
            painter.circle_filled(g.p(10.0, 8.0), g.len(1.1), IMAGE);
            closed(
                &[
                    (4.5, 13.0),
                    (7.0, 9.5),
                    (9.0, 11.5),
                    (10.5, 10.5),
                    (11.5, 13.0),
                ],
                IMAGE,
                Stroke::NONE,
            );
        }
        Icon::FileArchive => {
            page(None);
            for y in [5.5, 8.5, 11.5] {
                line(&[(8.0, y), (8.0, y + 1.5)]);
            }
            painter.rect_stroke(
                g.rect(6.5, 12.0, 9.5, 14.0),
                g.len(0.5),
                Stroke::new(stroke.width, ARCHIVE),
                egui::StrokeKind::Inside,
            );
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
        Icon::Variants => {
            painter.rect_stroke(
                g.rect(1.5, 5.5, 10.5, 14.5),
                g.len(1.5),
                stroke,
                egui::StrokeKind::Middle,
            );
            line(&[
                (4.5, 3.5),
                (4.5, 1.5),
                (14.5, 1.5),
                (14.5, 11.5),
                (12.5, 11.5),
            ]);
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
        Icon::Pin => {
            line(&[(5.0, 2.5), (11.0, 2.5)]);
            line(&[
                (6.5, 2.5),
                (6.5, 7.0),
                (4.0, 9.5),
                (12.0, 9.5),
                (9.5, 7.0),
                (9.5, 2.5),
            ]);
            line(&[(8.0, 9.5), (8.0, 14.0)]);
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
        Icon::Rollback => {
            // Arrow pointing left that curves down to the right.
            line(&[(3.0, 6.0), (10.0, 6.0)]);
            let center = g.p(10.0, 9.5);
            let points = (0..=12)
                .map(|step| {
                    let angle =
                        -std::f32::consts::FRAC_PI_2 + step as f32 / 12.0 * std::f32::consts::PI;
                    center + vec2(angle.cos(), angle.sin()) * g.len(3.5)
                })
                .collect::<Vec<_>>();
            painter.add(Shape::Path(PathShape::line(points, stroke)));
            line(&[(10.0, 13.0), (6.5, 13.0)]);
            line(&[(5.5, 3.5), (3.0, 6.0), (5.5, 8.5)]);
        }
        Icon::Diff => {
            closed(
                &[(2.0, 3.0), (7.0, 3.0), (7.0, 13.0), (2.0, 13.0)],
                Color32::TRANSPARENT,
                stroke,
            );
            closed(
                &[(9.0, 3.0), (14.0, 3.0), (14.0, 13.0), (9.0, 13.0)],
                Color32::TRANSPARENT,
                stroke,
            );
            line(&[(3.5, 6.5), (5.5, 6.5)]);
            line(&[(10.5, 6.5), (12.5, 6.5)]);
            line(&[(10.5, 9.5), (12.5, 9.5)]);
        }
        Icon::Error => {
            painter.circle_filled(g.p(8.0, 8.0), g.len(6.5), theme::ERROR);
            let white = Stroke::new(g.len(1.5).max(1.0), theme::ISLAND);
            painter.line_segment([g.p(8.0, 4.5), g.p(8.0, 9.0)], white);
            painter.circle_filled(g.p(8.0, 11.3), g.len(1.0), theme::ISLAND);
        }
        Icon::Warning | Icon::WeakWarning => {
            let fill = if icon == Icon::Warning {
                *theme::WARNING
            } else {
                *theme::editor::WEAK_WARNING_EFFECT
            };
            closed(
                &[(8.0, 1.5), (15.0, 14.0), (1.0, 14.0)],
                fill,
                Stroke::new(stroke.width, fill),
            );
            let dark = Stroke::new(g.len(1.5).max(1.0), theme::ISLAND);
            painter.line_segment([g.p(8.0, 6.0), g.p(8.0, 10.0)], dark);
            painter.circle_filled(g.p(8.0, 12.0), g.len(0.9), theme::ISLAND);
        }
        Icon::Success => {
            let check = Stroke::new(g.len(1.6).max(1.0), theme::SUCCESS);
            painter.add(Shape::Path(PathShape::line(
                vec![g.p(3.0, 8.5), g.p(6.5, 12.0), g.p(13.0, 4.5)],
                check,
            )));
        }
        Icon::Rerun => {
            let center = g.p(8.0, 8.0);
            let points = (0..=20)
                .map(|step| {
                    let angle = 0.9 + step as f32 / 20.0 * 4.6;
                    center + vec2(angle.cos(), angle.sin()) * g.len(5.5)
                })
                .collect();
            painter.add(Shape::Path(PathShape::line(points, stroke)));
            closed(
                &[(6.5, 6.0), (11.0, 8.5), (6.5, 11.0)],
                *theme::RUN_GREEN,
                Stroke::new(stroke.width * 0.5, theme::RUN_GREEN),
            );
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

    fn badge_label(name: &str) -> Option<&'static str> {
        match Icon::for_file(name) {
            Icon::FileBadge(badge) => Some(badge.label),
            _ => None,
        }
    }

    #[test]
    fn file_icons_follow_names_then_extensions() {
        // Languages.
        for (name, label) in [
            ("Main.java", "J"),
            ("A.kt", "K"),
            ("build.gradle.kts", "G"),
            ("settings.gradle", "G"),
            ("x.kts", "Ks"),
            ("a.groovy", "Gr"),
            ("pom.xml", "<>"),
            ("AndroidManifest.xml", "An"),
            ("a.json", "{}"),
            ("a.yml", "Y"),
            ("a.yaml", "Y"),
            ("Cargo.toml", "T"),
            ("app.properties", "Pr"),
            ("a.md", "M"),
            ("lib.rs", "RS"),
            ("main.go", "Go"),
            ("a.py", "Py"),
            ("a.js", "JS"),
            ("a.ts", "TS"),
            ("a.tsx", "Tx"),
            ("index.html", "H"),
            ("a.css", "C"),
            ("a.sh", ">_"),
            ("a.bat", ">_"),
            ("a.cmd", ">_"),
            ("a.ps1", ">_"),
            ("a.sql", "Sq"),
            ("a.proto", "Pb"),
            ("a.csv", "Cs"),
            // Config files and well-known names.
            ("Cargo.lock", "Lk"),
            (".gitignore", "Gi"),
            (".gitattributes", "Gi"),
            ("Dockerfile", "D"),
            (".editorconfig", "Ec"),
            ("LICENSE", "Li"),
            ("README", "Rd"),
            ("README.MD", "Rd"),
            ("gradlew", "G"),
            ("gradlew.bat", "G"),
        ] {
            assert_eq!(badge_label(name), Some(label), "{name}");
        }
        for name in ["a.png", "a.JPG", "a.svg", "a.webp"] {
            assert_eq!(Icon::for_file(name), Icon::FileImage, "{name}");
        }
        for name in ["a.jar", "a.zip"] {
            assert_eq!(Icon::for_file(name), Icon::FileArchive, "{name}");
        }
        // Unknown and plain text files fall back to the generic page.
        for name in ["notes.txt", "Unknown.xyz", "noextension", ".hidden", "png"] {
            assert_eq!(Icon::for_file(name), Icon::File, "{name}");
        }
    }

    #[test]
    fn folder_icons_follow_their_role() {
        let kind = |path: &str| FolderKind::classify(Path::new(path));
        assert_eq!(kind("src"), FolderKind::Plain);
        assert_eq!(kind("src/main"), FolderKind::Plain);
        assert_eq!(kind("src/main/java"), FolderKind::Source);
        assert_eq!(kind("app/src/main/kotlin"), FolderKind::Source);
        assert_eq!(kind("src/test/java"), FolderKind::Test);
        assert_eq!(kind("app/src/androidTest/java"), FolderKind::Test);
        assert_eq!(kind("src/main/resources"), FolderKind::Resources);
        assert_eq!(kind("app/src/main/res"), FolderKind::Resources);
        assert_eq!(kind("src/main/java/com"), FolderKind::Package);
        assert_eq!(kind("src/test/java/com/example"), FolderKind::Package);
        assert_eq!(kind("build"), FolderKind::Excluded);
        assert_eq!(kind("app/build/classes"), FolderKind::Excluded);
        assert_eq!(kind("target"), FolderKind::Excluded);
        assert_eq!(kind(".git"), FolderKind::Hidden);
        assert_eq!(kind(".idea"), FolderKind::Hidden);
        assert_eq!(kind(""), FolderKind::Plain);
        assert_eq!(Icon::for_folder(Path::new("docs")), Icon::Folder);
    }

    #[test]
    fn grid_maps_center_of_any_square() {
        let grid = Grid::new(Rect::from_min_size(pos2(10.0, 10.0), vec2(32.0, 32.0)));
        assert_eq!(grid.p(8.0, 8.0), pos2(26.0, 26.0));
        assert_eq!(grid.len(1.0), 2.0);
    }
}
