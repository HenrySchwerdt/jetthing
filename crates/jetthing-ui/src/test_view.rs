//! Test runner tree ("Test Results" → suites → cases) and the selected test's details.

use crate::build_window::format_duration;
use crate::icons::{self, Icon};
use crate::widgets::{list_row, row_text};
use crate::{IdeApp, theme};
use eframe::egui::{self, Color32, RichText, vec2};
use jetthing_runner::{TestCase, TestOutcome, TestResults};
use std::time::Duration;

const INDENT: f32 = 18.0;

#[derive(Clone, Default)]
struct View {
    /// (suite, case); a suite alone when `case` is empty.
    selected: Option<(String, String)>,
    hide_passed: bool,
}

fn view_id() -> egui::Id {
    egui::Id::new("test_view")
}

/// Summary line of the console header, e.g. `1 test passed` / `2 tests failed, 3 passed`.
pub fn summary(results: &TestResults) -> (String, Color32) {
    let counts = results.counts();
    let tests = |n: usize| if n == 1 { "test" } else { "tests" };
    if counts.failed > 0 {
        let mut text = format!("{} {} failed", counts.failed, tests(counts.failed));
        if counts.passed > 0 {
            text.push_str(&format!(", {} passed", counts.passed));
        }
        if counts.skipped > 0 {
            text.push_str(&format!(", {} ignored", counts.skipped));
        }
        (text, *theme::ERROR)
    } else if counts.passed > 0 {
        let mut text = format!("{} {} passed", counts.passed, tests(counts.passed));
        if counts.skipped > 0 {
            text.push_str(&format!(", {} ignored", counts.skipped));
        }
        (text, *theme::SUCCESS)
    } else if counts.skipped > 0 {
        (
            format!("{} {} ignored", counts.skipped, tests(counts.skipped)),
            *theme::WARNING,
        )
    } else {
        ("No tests were found".into(), *theme::TEXT_SECONDARY)
    }
}

fn outcome_icon(failed: bool, skipped: bool) -> (Icon, Color32) {
    if failed {
        (Icon::Error, *theme::ERROR)
    } else if skipped {
        (Icon::Warning, *theme::WARNING)
    } else {
        (Icon::Success, *theme::SUCCESS)
    }
}

fn duration_text(duration: Option<Duration>) -> String {
    duration.map(format_duration).unwrap_or_default()
}

/// Paints the tree; returns whether a toolbar toggle should be drawn as active.
pub fn tree(app: &IdeApp, ui: &mut egui::Ui, running: bool) {
    let mut view: View = ui.data(|data| data.get_temp(view_id())).unwrap_or_default();
    let results = app.state.runner.tests();
    ui.horizontal(|ui| {
        ui.add_space(8.0);
        ui.checkbox(&mut view.hide_passed, "Hide passed");
    });
    egui::ScrollArea::both()
        .id_salt("test_tree")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            let counts = results.counts();
            let (rect, response) = list_row(ui, view.selected.is_none());
            let x = rect.left() + 10.0;
            if running {
                let spinner = egui::Rect::from_min_size(
                    egui::pos2(x, rect.center().y - 6.0),
                    vec2(12.0, 12.0),
                );
                ui.put(
                    spinner,
                    egui::Spinner::new().size(12.0).color(*theme::ACCENT),
                );
            } else {
                let (icon, color) = outcome_icon(counts.failed > 0, counts.total() == 0);
                paint_icon(ui, x, rect, icon, color);
            }
            row_text(
                ui,
                rect,
                x + 20.0,
                "Test Results",
                egui::FontId::proportional(theme::UI_FONT_SIZE),
                *theme::TEXT_BRIGHT,
            );
            right_text(ui, rect, &duration_text(results.duration()));
            if response.clicked() {
                view.selected = None;
            }
            for suite in results.suites() {
                if view.hide_passed && !suite.failed() {
                    continue;
                }
                let selected = view
                    .selected
                    .as_ref()
                    .is_some_and(|(s, c)| s == &suite.name && c.is_empty());
                let (rect, response) = list_row(ui, selected);
                let x = rect.left() + 10.0 + INDENT;
                let skipped = suite
                    .cases
                    .iter()
                    .all(|case| case.outcome == TestOutcome::Skipped);
                let (icon, color) = outcome_icon(suite.failed(), skipped);
                paint_icon(ui, x, rect, icon, color);
                row_text(
                    ui,
                    rect,
                    x + 20.0,
                    short_suite(&suite.name),
                    egui::FontId::proportional(theme::UI_FONT_SIZE),
                    *theme::TEXT,
                );
                right_text(ui, rect, &duration_text(suite.duration()));
                let response = response.on_hover_text(&suite.name);
                if response.clicked() {
                    view.selected = Some((suite.name.clone(), String::new()));
                }
                for case in &suite.cases {
                    if view.hide_passed && !case.failed() {
                        continue;
                    }
                    let selected = view
                        .selected
                        .as_ref()
                        .is_some_and(|(s, c)| s == &suite.name && c == &case.name);
                    let (rect, response) = list_row(ui, selected);
                    let x = rect.left() + 10.0 + INDENT * 2.0;
                    let (icon, color) =
                        outcome_icon(case.failed(), case.outcome == TestOutcome::Skipped);
                    paint_icon(ui, x, rect, icon, color);
                    row_text(
                        ui,
                        rect,
                        x + 20.0,
                        &case.name,
                        egui::FontId::proportional(theme::UI_FONT_SIZE),
                        *theme::TEXT,
                    );
                    right_text(ui, rect, &duration_text(case.duration));
                    if response.clicked() {
                        view.selected = Some((suite.name.clone(), case.name.clone()));
                    }
                }
            }
        });
    ui.data_mut(|data| data.insert_temp(view_id(), view));
}

fn paint_icon(ui: &egui::Ui, x: f32, rect: egui::Rect, icon: Icon, color: Color32) {
    icons::paint(
        ui.painter(),
        egui::Rect::from_min_size(egui::pos2(x, rect.center().y - 7.0), vec2(14.0, 14.0)),
        icon,
        color,
    );
}

fn right_text(ui: &egui::Ui, rect: egui::Rect, text: &str) {
    if text.is_empty() {
        return;
    }
    let font = egui::FontId::proportional(theme::UI_FONT_SIZE - 1.0);
    let galley = ui
        .painter()
        .layout_no_wrap(text.to_owned(), font, *theme::TEXT_INFO);
    let pos = egui::pos2(
        rect.right() - galley.size().x - 10.0,
        rect.center().y - galley.size().y / 2.0,
    );
    ui.painter().galley(pos, galley, *theme::TEXT_INFO);
}

/// `com.example.AppTest` → `AppTest` (the tree shows simple class names).
fn short_suite(name: &str) -> &str {
    if name.starts_with("//") || name.contains('/') {
        return name;
    }
    name.rsplit(['.', ':'])
        .find(|part| !part.is_empty())
        .unwrap_or(name)
}

/// The selected test case, if any.
pub fn selected_case(app: &IdeApp, ctx: &egui::Context) -> Option<TestCase> {
    let view: View = ctx
        .data(|data| data.get_temp(view_id()))
        .unwrap_or_default();
    let (suite, case) = view.selected?;
    app.state
        .runner
        .tests()
        .suites()
        .iter()
        .find(|s| s.name == suite)?
        .cases
        .iter()
        .find(|c| c.name == case)
        .cloned()
}

/// Header above the console: status icon, summary, total count and time.
pub fn header(app: &IdeApp, ui: &mut egui::Ui) {
    let results = app.state.runner.tests();
    let (text, color) = summary(results);
    ui.horizontal(|ui| {
        let counts = results.counts();
        let (icon, icon_color) = outcome_icon(counts.failed > 0, counts.total() == 0);
        icons::show(ui, icon, 14.0, icon_color);
        ui.label(RichText::new(text).color(color));
        ui.label(
            RichText::new(format!(
                "{} {} total{}",
                counts.total(),
                if counts.total() == 1 { "test" } else { "tests" },
                results
                    .duration()
                    .map(|d| format!(", {}", format_duration(d)))
                    .unwrap_or_default()
            ))
            .color(*theme::TEXT_INFO),
        );
    });
    let progress = ui.available_rect_before_wrap();
    let bar = egui::Rect::from_min_size(progress.min, vec2(progress.width(), 2.0));
    let counts = results.counts();
    let color = if counts.failed > 0 {
        *theme::ERROR
    } else {
        *theme::SUCCESS
    };
    ui.painter().rect_filled(bar, 0.0, color);
    ui.add_space(6.0);
}

/// Failure message, stack trace and captured output of a test.
pub fn details(ui: &mut egui::Ui, case: &TestCase) {
    egui::ScrollArea::both()
        .id_salt("test_details")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let mono = egui::FontId::monospace(theme::UI_FONT_SIZE - 0.5);
            match &case.outcome {
                TestOutcome::Failed { message, details } => {
                    ui.label(
                        RichText::new(message)
                            .font(mono.clone())
                            .color(*theme::ERROR),
                    );
                    if !details.trim().is_empty() && details.trim() != message.trim() {
                        ui.label(
                            RichText::new(details.trim_end())
                                .font(mono.clone())
                                .color(*theme::editor::TEXT),
                        );
                    }
                }
                TestOutcome::Skipped => {
                    ui.label(RichText::new("Test ignored").color(*theme::WARNING));
                }
                TestOutcome::Passed => {
                    ui.label(RichText::new("Test passed").color(*theme::SUCCESS));
                }
            }
            if !case.output.trim().is_empty() {
                ui.add_space(8.0);
                ui.label(
                    RichText::new(case.output.trim_end())
                        .font(mono)
                        .color(*theme::editor::TEXT),
                );
            }
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summaries_read_like_the_reference() {
        let passed = TestResults::parse_junit(
            r#"<testsuite name="A"><testcase classname="A" name="greets()" time="0.021"/></testsuite>"#,
        )
        .unwrap();
        assert_eq!(summary(&passed).0, "1 test passed");
        let mixed = TestResults::parse_junit(
            r#"<testsuite name="A"><testcase classname="A" name="a"/><testcase classname="A" name="b"><failure/></testcase><testcase classname="A" name="c"><skipped/></testcase></testsuite>"#,
        )
        .unwrap();
        assert_eq!(summary(&mixed).0, "1 test failed, 1 passed, 1 ignored");
        assert_eq!(summary(&TestResults::default()).0, "No tests were found");
    }

    #[test]
    fn suite_names_are_shortened_like_class_names() {
        assert_eq!(short_suite("com.example.AppTest"), "AppTest");
        assert_eq!(short_suite("tests::math"), "math");
        assert_eq!(short_suite("//:app_test"), "//:app_test");
        assert_eq!(short_suite("example.com/calc"), "example.com/calc");
    }
}
