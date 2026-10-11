//! Structure tool window (Alt+7), File Structure popup (Ctrl+F12) and the
//! code part of the navigation-bar breadcrumbs, all from the language
//! analysis' tags-query structure.

use crate::widgets;
use crate::{IdeApp, theme};
use eframe::egui::{self, Color32};
use jetthing_core::Document;
use jetthing_language::{StructureItem, SymbolKind, structure};
use serde_json::json;
use std::path::PathBuf;
use std::sync::Arc;

/// Last computed structure and the edit it is waiting out.
#[derive(Clone)]
struct Cached {
    path: PathBuf,
    /// Text revision the items were computed from.
    items_revision: u64,
    items: Arc<Vec<StructureItem>>,
    /// Newest revision seen, and when (egui time) it was first seen.
    seen_revision: u64,
    last_edit: f64,
}

/// While the user types, the previous structure is reused until this long
/// after the latest edit: the tags query walks the whole file, which takes
/// hundreds of milliseconds on very large files, too slow for every keystroke.
const STALE_FOR: f64 = 0.25;

/// Structure of `document`. A newly opened file is computed at once; after an
/// edit the previous items are kept until typing pauses for [`STALE_FOR`].
pub fn items(ctx: &egui::Context, document: &Document) -> Arc<Vec<StructureItem>> {
    let id = egui::Id::new("structure_view_cache");
    let revision = document.text.revision();
    let now = ctx.input(|input| input.time);
    if let Some(mut cached) = ctx
        .data(|data| data.get_temp::<Cached>(id))
        .filter(|cached| cached.path == document.path)
    {
        if cached.items_revision == revision {
            return cached.items;
        }
        if cached.seen_revision != revision {
            cached.seen_revision = revision;
            cached.last_edit = now;
        }
        if now - cached.last_edit < STALE_FOR {
            ctx.request_repaint_after(std::time::Duration::from_secs_f64(STALE_FOR));
            let items = cached.items.clone();
            ctx.data_mut(|data| data.insert_temp(id, cached));
            return items;
        }
    }
    let items = Arc::new(document.analysis.structure());
    ctx.data_mut(|data| {
        data.insert_temp(
            id,
            Cached {
                path: document.path.clone(),
                items_revision: revision,
                items: items.clone(),
                seen_revision: revision,
                last_edit: now,
            },
        )
    });
    items
}

/// Depth-first rows of the structure tree.
pub fn flatten<'a>(
    items: &'a [StructureItem],
    depth: usize,
    out: &mut Vec<(usize, &'a StructureItem)>,
) {
    for item in items {
        out.push((depth, item));
        flatten(&item.children, depth + 1, out);
    }
}

/// Badge letter and color per kind (original drawings: a filled circle with a letter).
fn badge(kind: SymbolKind) -> (&'static str, Color32) {
    match kind {
        SymbolKind::Class => ("C", Color32::from_rgb(0x54, 0x8A, 0xF7)),
        SymbolKind::Interface => ("I", Color32::from_rgb(0x5F, 0xB8, 0x65)),
        SymbolKind::Enum => ("E", Color32::from_rgb(0xD5, 0x96, 0x37)),
        SymbolKind::Function | SymbolKind::Method | SymbolKind::Constructor => {
            ("m", Color32::from_rgb(0xE5, 0x6A, 0x73))
        }
        SymbolKind::Field => ("f", Color32::from_rgb(0xC7, 0x7D, 0xBB)),
        SymbolKind::Constant => ("c", Color32::from_rgb(0xC7, 0x7D, 0xBB)),
        SymbolKind::Module => ("M", Color32::from_rgb(0x86, 0x8A, 0x91)),
        SymbolKind::Type => ("T", Color32::from_rgb(0x2A, 0xAC, 0xB8)),
        SymbolKind::Macro => ("!", Color32::from_rgb(0x86, 0x8A, 0x91)),
        SymbolKind::Tag => ("<", Color32::from_rgb(0xD5, 0xB7, 0x78)),
        SymbolKind::Key => ("k", Color32::from_rgb(0xC7, 0x7D, 0xBB)),
        SymbolKind::Heading => ("#", Color32::from_rgb(0x86, 0x8A, 0x91)),
        SymbolKind::Other => ("•", Color32::from_rgb(0x86, 0x8A, 0x91)),
    }
}

pub fn paint_badge(painter: &egui::Painter, center: egui::Pos2, kind: SymbolKind) {
    let (letter, color) = badge(kind);
    painter.circle_filled(center, 7.0, color.gamma_multiply(0.85));
    painter.text(
        center,
        egui::Align2::CENTER_CENTER,
        letter,
        egui::FontId::proportional(10.0),
        *theme::editor::BACKGROUND,
    );
}

/// Paints one structure row; returns its response and the chevron rect.
/// `expanded` draws a chevron for items with children (the tool window collapses them).
pub fn row(
    ui: &mut egui::Ui,
    depth: usize,
    item: &StructureItem,
    selected: bool,
    expanded: Option<bool>,
) -> widgets::TreeRowOut {
    let kind = item.kind;
    let paint = move |painter: &egui::Painter, rect: egui::Rect| {
        paint_badge(painter, rect.center(), kind);
    };
    widgets::tree_row(
        ui,
        widgets::TreeRow {
            depth,
            expanded,
            selected,
            paint_icon: Some(&paint),
            color: *theme::TEXT_BRIGHT,
            secondary: item
                .detail
                .as_deref()
                .map(|detail| (detail, *theme::TEXT_SECONDARY)),
            ..widgets::TreeRow::new(&item.name)
        },
    )
}

/// Moves the caret to the item's name and focuses the editor.
pub fn navigate(app: &mut IdeApp, ctx: &egui::Context, item: &StructureItem) {
    let Some(document) = app
        .state
        .active_document
        .and_then(|i| app.state.documents.get(i))
    else {
        return;
    };
    let line_start = document.text.line_to_char(item.line).unwrap_or(0);
    let column = document
        .text
        .byte_to_char(item.name_range.start)
        .saturating_sub(line_start);
    app.dispatch(
        "editor.gotoLine",
        json!({"line": item.line + 1, "column": column + 1}),
    );
    ctx.memory_mut(|memory| memory.request_focus(crate::editor::editor_id()));
}

/// The innermost item containing the primary caret (to highlight in lists).
pub fn item_at_caret<'a>(
    document: &Document,
    items: &'a [StructureItem],
) -> Option<&'a StructureItem> {
    let byte = document
        .text
        .char_to_byte(document.selections.primary().head);
    structure::path_at(items, byte).last().copied()
}

/// Names of the declarations around the caret, outermost first (breadcrumbs).
pub fn breadcrumbs(ctx: &egui::Context, document: &Document) -> Vec<(SymbolKind, String)> {
    let items = items(ctx, document);
    let byte = document
        .text
        .char_to_byte(document.selections.primary().head);
    structure::path_at(&items, byte)
        .into_iter()
        .map(|item| (item.kind, crumb_label(&item.name)))
        .collect()
}

/// Short breadcrumb label: qualified names keep their last segment
/// (`androidx.…ConstraintLayout` → `ConstraintLayout`), long ones are cut.
fn crumb_label(name: &str) -> String {
    let name = name.lines().next().unwrap_or_default().trim();
    let short = if name.len() > 24 && name.contains('.') && !name.contains(' ') {
        name.rsplit('.').next().unwrap_or(name)
    } else {
        name
    };
    if short.chars().count() > 32 {
        format!("{}…", short.chars().take(31).collect::<String>())
    } else {
        short.to_owned()
    }
}

/// Structure tool window body.
pub fn tool_window(app: &mut IdeApp, ui: &mut egui::Ui) {
    let Some(document) = app
        .state
        .active_document
        .and_then(|i| app.state.documents.get(i))
    else {
        ui.label(egui::RichText::new("No file open").color(theme::TEXT_SECONDARY));
        return;
    };
    let items = items(ui.ctx(), document);
    if items.is_empty() {
        ui.label(egui::RichText::new("No structure for this file").color(theme::TEXT_SECONDARY));
        return;
    }
    let current = item_at_caret(document, &items).map(|item| item.name_range.clone());
    let mut rows = Vec::new();
    flatten(&items, 0, &mut rows);
    let collapsed_id = egui::Id::new("structure_collapsed").with(&document.path);
    let mut collapsed: std::collections::HashSet<usize> = ui
        .data(|data| data.get_temp(collapsed_id))
        .unwrap_or_default();
    let mut clicked = None;
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            let mut hidden_below = None;
            for (depth, item) in &rows {
                if let Some(limit) = hidden_below {
                    if *depth > limit {
                        continue;
                    }
                    hidden_below = None;
                }
                let key = item.name_range.start;
                let is_collapsed = collapsed.contains(&key);
                let selected = current.as_ref() == Some(&item.name_range);
                let has_children = !item.children.is_empty();
                let out = row(
                    ui,
                    *depth,
                    item,
                    selected,
                    has_children.then_some(!is_collapsed),
                );
                if has_children && is_collapsed {
                    hidden_below = Some(*depth);
                }
                if out.response.clicked() {
                    if has_children && out.on_chevron() {
                        if !collapsed.remove(&key) {
                            collapsed.insert(key);
                        }
                    } else {
                        clicked = Some((*item).clone());
                    }
                }
            }
        });
    ui.data_mut(|data| data.insert_temp(collapsed_id, collapsed));
    if let Some(item) = clicked {
        navigate(app, ui.ctx(), &item);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rows_are_depth_first_and_cache_follows_revisions() {
        let ctx = egui::Context::default();
        let mut document = Document::new(
            "A.java".into(),
            "class A {\n    void f() {}\n    class B {\n        void g() {}\n    }\n}\n",
        );
        let first = items(&ctx, &document);
        let mut rows = Vec::new();
        flatten(&first, 0, &mut rows);
        let names: Vec<_> = rows
            .iter()
            .map(|(depth, item)| (*depth, item.name.as_str()))
            .collect();
        assert_eq!(names, vec![(0, "A"), (1, "f"), (1, "B"), (2, "g")]);
        assert!(Arc::ptr_eq(&first, &items(&ctx, &document)));

        document.text.edit(0, 0, "// x\n").unwrap();
        let source = document.text.text();
        document.refresh_analysis(&source);
        // While typing, the previous structure is reused...
        let at = |time: f64| egui::RawInput {
            time: Some(time),
            ..Default::default()
        };
        let mut during = None;
        let _ = ctx.run(at(10.0), |ctx| during = Some(items(ctx, &document)));
        assert!(Arc::ptr_eq(&first, &during.unwrap()));
        let mut soon = None;
        let _ = ctx.run(at(10.0 + STALE_FOR / 2.0), |ctx| {
            soon = Some(items(ctx, &document))
        });
        assert!(Arc::ptr_eq(&first, &soon.unwrap()));
        // ...and rebuilt once typing pauses.
        let mut later = None;
        let _ = ctx.run(at(10.0 + STALE_FOR + 0.01), |ctx| {
            later = Some(items(ctx, &document))
        });
        assert!(!Arc::ptr_eq(&first, &later.unwrap()));
        let caret = source.find("g()").unwrap();
        document
            .selections
            .set_single(jetthing_text::Selection::caret(caret));
        let crumbs: Vec<_> = breadcrumbs(&ctx, &document)
            .into_iter()
            .map(|(_, name)| name)
            .collect();
        assert_eq!(crumbs, vec!["A", "B", "g"]);
        assert_eq!(
            crumb_label("androidx.constraintlayout.widget.ConstraintLayout"),
            "ConstraintLayout"
        );
        assert_eq!(
            crumb_label("Getting started with the very long heading"),
            "Getting started with the very l…"
        );
    }
}
