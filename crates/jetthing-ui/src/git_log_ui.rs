//! Git tool window, Log tab: branches pane, filterable commit list with a
//! commit graph, and the selected commit's changed files and message.

use crate::icons::{self, Icon};
use crate::widgets::{self, list_row, row_text, tool_window_header};
use crate::{IdeApp, theme};
use eframe::egui::{self, Color32, Rect, RichText, Sense, Stroke, pos2, vec2};
use jetthing_core::ToolWindow;
use jetthing_git::{ChangeKind, CommitInfo, GraphEdge, GraphRow};
use serde_json::{Value, json};
use std::time::{SystemTime, UNIX_EPOCH};

const BRANCHES_WIDTH: f32 = 240.0;
const DETAILS_WIDTH: f32 = 320.0;
const ROW: f32 = 24.0;
const LANE: f32 = 14.0;
const DATE_WIDTH: f32 = 120.0;
const AUTHOR_WIDTH: f32 = 110.0;

/// Lane colours, cycled by column (observed log graph palette, approximate).
const LANE_COLORS: [Color32; 6] = [
    Color32::from_rgb(0x4E, 0x9D, 0x6C),
    Color32::from_rgb(0x56, 0xA8, 0xF5),
    Color32::from_rgb(0xD5, 0x96, 0x37),
    Color32::from_rgb(0xB1, 0x89, 0xF5),
    Color32::from_rgb(0xE3, 0x77, 0x74),
    Color32::from_rgb(0x2A, 0xAC, 0xB8),
];

fn lane_color(column: usize) -> Color32 {
    LANE_COLORS[column % LANE_COLORS.len()]
}

pub fn filter_id() -> egui::Id {
    egui::Id::new("git_log_filter")
}

pub fn show(app: &mut IdeApp, ui: &mut egui::Ui) {
    let hide = tool_window_header(ui, "Git", |ui| {
        if icons::button(ui, Icon::Refresh, "Refresh").clicked() {
            app.dispatch("git.log.refresh", Value::Null);
        }
    });
    if hide {
        app.toggle_tool_window(ToolWindow::Git);
        return;
    }
    if app.state.git.is_none() {
        ui.add_space(12.0);
        ui.vertical_centered(|ui| {
            ui.label(
                RichText::new("The project is not a Git repository.").color(theme::TEXT_SECONDARY),
            );
        });
        return;
    }
    if !app.state.git_log.loaded {
        app.dispatch("git.log.refresh", Value::Null);
    }
    let area = ui.available_rect_before_wrap();
    ui.allocate_rect(area, Sense::hover());
    let branches_rect = Rect::from_min_size(area.min, vec2(BRANCHES_WIDTH, area.height()));
    let details_rect = Rect::from_min_max(
        pos2(
            (area.right() - DETAILS_WIDTH).max(branches_rect.right() + 200.0),
            area.top(),
        ),
        area.max,
    );
    let commits_rect = Rect::from_min_max(
        pos2(branches_rect.right(), area.top()),
        pos2(details_rect.left(), area.bottom()),
    );
    let divider = Stroke::new(1.0_f32, theme::POPUP_BORDER);
    ui.painter()
        .vline(branches_rect.right(), area.y_range(), divider);
    ui.painter()
        .vline(details_rect.left(), area.y_range(), divider);
    let mut actions: Vec<(&'static str, Value)> = Vec::new();
    ui.scope_builder(egui::UiBuilder::new().max_rect(branches_rect), |ui| {
        branches_pane(app, ui, &mut actions);
    });
    ui.scope_builder(egui::UiBuilder::new().max_rect(commits_rect), |ui| {
        commits_pane(app, ui, &mut actions);
    });
    ui.scope_builder(egui::UiBuilder::new().max_rect(details_rect), |ui| {
        details_pane(app, ui, &mut actions);
    });
    for (id, params) in actions {
        app.dispatch(id, params);
    }
}

fn branches_pane(app: &IdeApp, ui: &mut egui::Ui, actions: &mut Vec<(&'static str, Value)>) {
    let log = &app.state.git_log;
    let font = egui::FontId::proportional(theme::UI_FONT_SIZE);
    egui::ScrollArea::vertical()
        .id_salt("git_log_branches")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            ui.add_space(4.0);
            let mut entry = |ui: &mut egui::Ui,
                             indent: f32,
                             icon: Option<Icon>,
                             label: &str,
                             filter: Option<&str>,
                             bold: bool| {
                let selected = log.branch.as_deref() == filter && filter.is_some();
                let (rect, response) = list_row(ui, selected);
                let mut x = rect.left() + 12.0 + indent;
                if let Some(icon) = icon {
                    icons::paint(
                        ui.painter(),
                        Rect::from_min_size(pos2(x, rect.center().y - 7.0), vec2(14.0, 14.0)),
                        icon,
                        theme::WARNING,
                    );
                    x += 20.0;
                }
                row_text(
                    ui,
                    rect,
                    x,
                    label,
                    if bold {
                        theme::semibold(theme::UI_FONT_SIZE)
                    } else {
                        font.clone()
                    },
                    theme::TEXT,
                );
                if response.clicked() {
                    let branch = if selected { None } else { filter };
                    actions.push(("git.log.filterBranch", json!({ "branch": branch })));
                }
                response
            };
            entry(ui, 0.0, None, "HEAD (Current Branch)", Some("HEAD"), false)
                .on_hover_text("Show only commits reachable from HEAD");
            group(ui, "Local");
            for branch in &log.branches.local {
                let current = log.branches.current.as_deref() == Some(branch.name.as_str());
                let mut label = branch.name.clone();
                if branch.ahead > 0 {
                    label.push_str(&format!("  ↑{}", branch.ahead));
                }
                if branch.behind > 0 {
                    label.push_str(&format!("  ↓{}", branch.behind));
                }
                let response = entry(
                    ui,
                    18.0,
                    Some(Icon::Branch),
                    &label,
                    Some(&branch.name),
                    current,
                );
                if let Some(upstream) = &branch.upstream {
                    response.on_hover_text(format!("Tracking {upstream}"));
                }
            }
            if !log.branches.remote.is_empty() {
                group(ui, "Remote");
                for branch in &log.branches.remote {
                    entry(ui, 18.0, Some(Icon::Branch), branch, Some(branch), false);
                }
            }
            if !log.branches.tags.is_empty() {
                group(ui, "Tags");
                for tag in &log.branches.tags {
                    entry(ui, 18.0, None, tag, Some(tag), false);
                }
            }
        });
}

fn group(ui: &mut egui::Ui, title: &str) {
    let (rect, _) = list_row(ui, false);
    icons::paint(
        ui.painter(),
        Rect::from_min_size(
            pos2(rect.left() + 8.0, rect.center().y - 7.0),
            vec2(14.0, 14.0),
        ),
        Icon::ChevronDown,
        theme::TEXT_SECONDARY,
    );
    row_text(
        ui,
        rect,
        rect.left() + 28.0,
        title,
        egui::FontId::proportional(theme::UI_FONT_SIZE),
        theme::TEXT,
    );
}

fn commits_pane(app: &IdeApp, ui: &mut egui::Ui, actions: &mut Vec<(&'static str, Value)>) {
    let log = &app.state.git_log;
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        ui.add_space(8.0);
        let mut filter = log.filter.clone();
        let response = widgets::search_field(ui, filter_id(), &mut filter, "Text or hash", 180.0);
        if response.changed() {
            actions.push(("git.log.filterText", json!({ "text": filter })));
        }
        ui.add_space(8.0);
        let label = match &log.branch {
            Some(branch) => format!("Branch: {branch}"),
            None => "Branch".to_owned(),
        };
        egui::ComboBox::from_id_salt("git_log_branch_filter")
            .selected_text(label)
            .width(140.0)
            .show_ui(ui, |ui| {
                if ui.selectable_label(log.branch.is_none(), "All").clicked() {
                    actions.push(("git.log.filterBranch", json!({})));
                }
                for branch in log
                    .branches
                    .local
                    .iter()
                    .map(|b| &b.name)
                    .chain(&log.branches.remote)
                {
                    if ui
                        .selectable_label(log.branch.as_ref() == Some(branch), branch)
                        .clicked()
                    {
                        actions.push(("git.log.filterBranch", json!({ "branch": branch })));
                    }
                }
            });
    });
    ui.add_space(4.0);
    if let Some(error) = &log.error {
        ui.label(RichText::new(error).color(theme::ERROR));
        return;
    }
    let visible = log.visible();
    if visible.is_empty() {
        ui.add_space(12.0);
        ui.vertical_centered(|ui| {
            let text = if log.commits.is_empty() {
                "No commits yet"
            } else {
                "No commits matching the filters"
            };
            ui.label(RichText::new(text).color(theme::TEXT_SECONDARY));
        });
        return;
    }
    // The graph only makes sense for the unfiltered history.
    let show_graph = log.filter.trim().is_empty();
    let lanes = if show_graph {
        log.graph.iter().map(row_width).max().unwrap_or(1)
    } else {
        0
    };
    let graph_width = lanes as f32 * LANE + 8.0;
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_secs() as i64);
    egui::ScrollArea::vertical()
        .id_salt("git_log_commits")
        .auto_shrink([false, false])
        .show_rows(ui, ROW, visible.len(), |ui, range| {
            ui.spacing_mut().item_spacing.y = 0.0;
            for row in range {
                let index = visible[row];
                let commit = &log.commits[index];
                let selected = log.selected.as_ref() == Some(&commit.hash);
                let (rect, response) =
                    ui.allocate_exact_size(vec2(ui.available_width(), ROW), Sense::click());
                let fill = if selected {
                    theme::SELECTION
                } else if response.hovered() {
                    theme::HOVER
                } else {
                    Color32::TRANSPARENT
                };
                ui.painter()
                    .rect_filled(rect.shrink2(vec2(4.0, 0.0)), 4.0, fill);
                if show_graph && let Some(graph_row) = log.graph.get(index) {
                    paint_graph(ui.painter(), rect, graph_row, commit);
                }
                commit_row(ui, rect, commit, graph_width, now);
                if response.clicked() {
                    actions.push(("git.log.selectCommit", json!({ "hash": commit.hash })));
                }
            }
        });
}

fn row_width(row: &GraphRow) -> usize {
    row.edges
        .iter()
        .map(|edge| match edge {
            GraphEdge::Through { column }
            | GraphEdge::Into { column }
            | GraphEdge::OutOf { column } => *column,
        })
        .chain([row.column])
        .max()
        .unwrap_or(0)
        + 1
}

fn paint_graph(painter: &egui::Painter, rect: Rect, row: &GraphRow, commit: &CommitInfo) {
    let x = |column: usize| rect.left() + 12.0 + column as f32 * LANE;
    let (top, mid, bottom) = (rect.top(), rect.center().y, rect.bottom());
    let node = pos2(x(row.column), mid);
    for edge in &row.edges {
        let (from, to, color) = match *edge {
            GraphEdge::Through { column } => {
                (pos2(x(column), top), pos2(x(column), bottom), column)
            }
            GraphEdge::Into { column } => (pos2(x(column), top), node, column),
            GraphEdge::OutOf { column } => (node, pos2(x(column), bottom), column.max(row.column)),
        };
        painter.line_segment([from, to], Stroke::new(1.5_f32, lane_color(color)));
    }
    let color = lane_color(row.column);
    if commit.parents.len() > 1 {
        painter.circle_stroke(node, 3.5, Stroke::new(1.5_f32, color));
        painter.circle_filled(node, 2.0, theme::ISLAND);
    } else {
        painter.circle_filled(node, 4.0, color);
    }
}

fn commit_row(ui: &egui::Ui, rect: Rect, commit: &CommitInfo, graph_width: f32, now: i64) {
    let font = egui::FontId::proportional(theme::UI_FONT_SIZE);
    let date = relative_time(now, commit.time);
    // Fixed author and date columns, like the reference table.
    let date_left = rect.right() - 12.0 - DATE_WIDTH;
    let author_left = date_left - 10.0 - AUTHOR_WIDTH;
    for (left, width, text, color) in [
        (date_left, DATE_WIDTH, date.as_str(), theme::TEXT_SECONDARY),
        (
            author_left,
            AUTHOR_WIDTH,
            commit.author.as_str(),
            theme::TEXT,
        ),
    ] {
        let cell = Rect::from_min_max(pos2(left, rect.top()), pos2(left + width, rect.bottom()));
        row_text(ui, cell, left, text, font.clone(), color);
    }
    // Ref labels (branches, tags) right-aligned before the author.
    let mut refs_left = author_left - 10.0;
    for label in commit.refs.iter().rev() {
        let text = label.strip_prefix("HEAD -> ").unwrap_or(label);
        let is_tag = text.starts_with("tag: ");
        let text = text.strip_prefix("tag: ").unwrap_or(text);
        let galley =
            ui.painter()
                .layout_no_wrap(text.to_owned(), font.clone(), theme::TEXT_SECONDARY);
        let width = galley.size().x + 22.0;
        refs_left -= width;
        if refs_left < rect.left() + graph_width + 120.0 {
            break;
        }
        icons::paint(
            ui.painter(),
            Rect::from_min_size(pos2(refs_left, rect.center().y - 7.0), vec2(14.0, 14.0)),
            if is_tag { Icon::Commit } else { Icon::Branch },
            if label.starts_with("HEAD -> ") {
                theme::WARNING
            } else {
                theme::TEXT_SECONDARY
            },
        );
        ui.painter().galley(
            pos2(refs_left + 18.0, rect.center().y - galley.size().y / 2.0),
            galley,
            theme::TEXT_SECONDARY,
        );
        refs_left -= 6.0;
    }
    let subject_rect = Rect::from_min_max(
        pos2(rect.left() + graph_width + 6.0, rect.top()),
        pos2(
            refs_left.max(rect.left() + graph_width + 40.0),
            rect.bottom(),
        ),
    );
    row_text(
        ui,
        subject_rect,
        subject_rect.left(),
        &commit.subject,
        font,
        theme::TEXT_BRIGHT,
    );
}

fn details_pane(app: &IdeApp, ui: &mut egui::Ui, actions: &mut Vec<(&'static str, Value)>) {
    let log = &app.state.git_log;
    let Some(details) = &log.details else {
        ui.add_space(12.0);
        ui.vertical_centered(|ui| {
            ui.label(RichText::new("No commit selected").color(theme::TEXT_SECONDARY));
        });
        return;
    };
    let commit = log
        .commits
        .iter()
        .find(|commit| commit.hash == details.hash);
    let files_height = (ui.available_height() * 0.55).max(80.0);
    egui::ScrollArea::vertical()
        .id_salt("git_log_files")
        .max_height(files_height)
        .auto_shrink([false, true])
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            ui.add_space(4.0);
            for file in &details.files {
                let (rect, response) = list_row(ui, false);
                let name = IdeApp::file_name(&file.path);
                let mut x = rect.left() + 12.0;
                icons::paint(
                    ui.painter(),
                    Rect::from_min_size(pos2(x, rect.center().y - 8.0), vec2(16.0, 16.0)),
                    Icon::for_file(&name),
                    icons::ICON,
                );
                x += 22.0;
                x = row_text(
                    ui,
                    rect,
                    x,
                    &name,
                    egui::FontId::proportional(theme::UI_FONT_SIZE),
                    crate::toolwindows::change_color(Some(file.kind)),
                );
                if let Some(parent) = file.path.parent().filter(|p| !p.as_os_str().is_empty()) {
                    row_text(
                        ui,
                        rect,
                        x + 8.0,
                        &parent.display().to_string(),
                        egui::FontId::proportional(theme::UI_FONT_SIZE),
                        theme::TEXT_INFO,
                    );
                }
                let response = response.on_hover_text(match file.kind {
                    ChangeKind::Added => "Added",
                    ChangeKind::Deleted => "Deleted",
                    ChangeKind::Renamed => "Renamed",
                    _ => "Modified",
                });
                if response.double_clicked() {
                    actions.push((
                        "git.log.showFileDiff",
                        json!({ "hash": details.hash, "path": file.path }),
                    ));
                }
            }
        });
    ui.painter().hline(
        ui.max_rect().x_range(),
        ui.cursor().top(),
        Stroke::new(1.0_f32, theme::POPUP_BORDER),
    );
    egui::ScrollArea::vertical()
        .id_salt("git_log_message")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.add_space(8.0);
            egui::Frame::new()
                .inner_margin(egui::Margin::symmetric(12, 0))
                .show(ui, |ui| {
                    let mut lines = details.message.lines();
                    if let Some(subject) = lines.next() {
                        ui.label(
                            RichText::new(subject)
                                .font(theme::semibold(theme::UI_FONT_SIZE))
                                .color(theme::TEXT_BRIGHT),
                        );
                    }
                    let body: Vec<&str> = lines.collect();
                    if !body.join("").trim().is_empty() {
                        ui.add_space(4.0);
                        ui.label(RichText::new(body.join("\n").trim()).color(theme::TEXT));
                    }
                    ui.add_space(8.0);
                    if let Some(commit) = commit {
                        ui.label(
                            RichText::new(format!(
                                "{} {} <{}> on {}",
                                commit.short,
                                commit.author,
                                commit.email,
                                absolute_time(commit.time)
                            ))
                            .color(theme::TEXT_SECONDARY),
                        );
                        if details.committer != commit.author {
                            ui.label(
                                RichText::new(format!(
                                    "committed by {} on {}",
                                    details.committer,
                                    absolute_time(details.commit_time)
                                ))
                                .color(theme::TEXT_SECONDARY),
                            );
                        }
                    }
                });
        });
}

/// "just now", "15 minutes ago", "3 hours ago", "Yesterday", else a date.
pub(crate) fn relative_time(now: i64, time: i64) -> String {
    let delta = now - time;
    match delta {
        d if d < 60 => "just now".into(),
        d if d < 3_600 => widgets::pluralize((d / 60) as usize, "minute ago", "minutes ago"),
        d if d < 86_400 => widgets::pluralize((d / 3_600) as usize, "hour ago", "hours ago"),
        d if d < 2 * 86_400 => "Yesterday".into(),
        _ => absolute_time(time),
    }
}

/// UTC calendar date `YYYY-MM-DD HH:MM` (civil-from-days, no time zone data).
pub(crate) fn absolute_time(time: i64) -> String {
    let days = time.div_euclid(86_400);
    let seconds = time.rem_euclid(86_400);
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    format!(
        "{year:04}-{month:02}-{day:02} {:02}:{:02}",
        seconds / 3_600,
        seconds % 3_600 / 60
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_relative_and_absolute_times() {
        let now = 1_791_640_000;
        assert_eq!(relative_time(now, now - 5), "just now");
        assert_eq!(relative_time(now, now - 15 * 60), "15 minutes ago");
        assert_eq!(relative_time(now, now - 60), "1 minute ago");
        assert_eq!(relative_time(now, now - 3 * 3_600), "3 hours ago");
        assert_eq!(relative_time(now, now - 30 * 3_600), "Yesterday");
        assert_eq!(absolute_time(0), "1970-01-01 00:00");
        assert_eq!(absolute_time(951_782_400), "2000-02-29 00:00");
        assert_eq!(absolute_time(1_791_640_000), "2026-10-10 13:46");
    }

    #[test]
    fn graph_width_counts_all_lanes() {
        let row = GraphRow {
            column: 0,
            edges: vec![
                GraphEdge::OutOf { column: 0 },
                GraphEdge::OutOf { column: 2 },
            ],
        };
        assert_eq!(row_width(&row), 3);
    }
}
