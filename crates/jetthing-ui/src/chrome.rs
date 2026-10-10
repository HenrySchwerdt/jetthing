//! Workbench chrome: merged main toolbar, tool window stripes and status bar.

use crate::icons::{self, Icon};
use crate::widgets::{self, chrome_frame};
use crate::{IdeApp, keys, theme};
use eframe::egui::{self, Color32, Rect, Sense, vec2};
use jetthing_core::{Anchor, ToolWindow};
use serde_json::{Value, json};

pub const TOOLBAR_HEIGHT: f32 = 40.0;
pub const STATUS_BAR_HEIGHT: f32 = 26.0;
pub const STRIPE_WIDTH: f32 = 40.0;

/// Whether the window draws its own title bar (no OS decorations).
pub fn custom_title_bar() -> bool {
    !cfg!(target_os = "macos")
}

pub fn main_toolbar(app: &mut IdeApp, ctx: &egui::Context) {
    egui::TopBottomPanel::top("main_toolbar")
        .exact_height(TOOLBAR_HEIGHT)
        .show_separator_line(false)
        .frame(chrome_frame())
        .show(ctx, |ui| {
            let bar = ui.max_rect();
            let glow = app
                .state
                .settings
                .bool(jetthing_settings::ids::PROJECT_COLORS)
                .unwrap_or(true)
                .then(|| crate::toolbar_gradient::reserve(ctx));
            // Empty toolbar space drags the window, like a native title bar.
            let drag = ui.interact(bar, egui::Id::new("toolbar_drag"), Sense::click_and_drag());
            if drag.drag_started() {
                ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
            }
            if drag.double_clicked() {
                let maximized = ctx.input(|i| i.viewport().maximized.unwrap_or(false));
                ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(!maximized));
            }
            let mut content = ui.new_child(
                egui::UiBuilder::new()
                    .max_rect(bar.shrink2(vec2(6.0, 0.0)))
                    .layout(egui::Layout::left_to_right(egui::Align::Center)),
            );
            let ui = &mut content;
            ui.spacing_mut().item_spacing.x = 4.0;
            if !crate::main_menu::show(app, ui) {
                ui.add_space(4.0);
                let badge = project_widget(app, ui);
                if let Some(slot) = glow {
                    let anchor = crate::toolbar_gradient::Anchor {
                        bar,
                        badge_left: badge.left(),
                        badge_right: badge.right(),
                    };
                    let color = theme::avatar_color(&app.project_name());
                    crate::toolbar_gradient::paint(ctx, slot, anchor, color);
                }
                vcs_widget(app, ui);
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.spacing_mut().item_spacing.x = 2.0;
                if custom_title_bar() {
                    window_controls(ui);
                    ui.add_space(8.0);
                }
                if icons::button(
                    ui,
                    Icon::Search,
                    &format!(
                        "Search Everywhere ({})",
                        app.action_shortcut("SearchEverywhere").unwrap_or_default()
                    ),
                )
                .clicked()
                {
                    app.palette.show();
                }
                ui.add_space(6.0);
                run_widget(app, ui);
                crate::device_manager::device_selector(app, ui);
            });
        });
}

/// Returns the project badge's rectangle.
fn project_widget(app: &mut IdeApp, ui: &mut egui::Ui) -> Rect {
    let name = app.project_name();
    let galley = ui.painter().layout_no_wrap(
        name.clone(),
        theme::semibold(theme::UI_FONT_SIZE),
        *theme::TEXT_BRIGHT,
    );
    let width = 8.0 + 20.0 + 8.0 + galley.size().x + 22.0;
    let (rect, response) = ui.allocate_exact_size(vec2(width, 28.0), Sense::click());
    let fill = if response.hovered() {
        *theme::HOVER
    } else {
        Color32::TRANSPARENT
    };
    ui.painter().rect_filled(rect, 6.0, fill);
    let badge = Rect::from_min_size(rect.left_center() + vec2(8.0, -10.0), vec2(20.0, 20.0));
    ui.painter()
        .rect_filled(badge, 4.0, theme::avatar_color(&name));
    ui.painter().text(
        badge.center(),
        egui::Align2::CENTER_CENTER,
        theme::project_initials(&name),
        theme::semibold(10.0),
        Color32::WHITE,
    );
    let text_pos = egui::pos2(badge.right() + 8.0, rect.center().y - galley.size().y / 2.0);
    let text_right = text_pos.x + galley.size().x;
    ui.painter().galley(text_pos, galley, *theme::TEXT_BRIGHT);
    icons::paint(
        ui.painter(),
        Rect::from_min_size(
            egui::pos2(text_right + 4.0, rect.center().y - 7.0),
            vec2(14.0, 14.0),
        ),
        Icon::ChevronDown,
        *theme::TEXT_SECONDARY,
    );
    let root = app
        .state
        .project
        .as_ref()
        .map(|project| project.root().display().to_string());
    let response = response.on_hover_text(root.clone().unwrap_or_default());
    egui::Popup::menu(&response).show(|ui| {
        ui.set_min_width(260.0);
        if let Some(root) = &root {
            ui.label(egui::RichText::new(root).color(theme::TEXT_SECONDARY));
            ui.separator();
        }
        if ui.button("Open Project…").clicked() {
            app.open_project_dialog();
            ui.close();
        }
        if ui.button("New Project…").clicked() {
            app.new_project_dialog();
            ui.close();
        }
        if ui.button("Refresh Plugins").clicked() {
            app.dispatch("plugin.refresh", Value::Null);
            ui.close();
        }
        ui.label(
            egui::RichText::new(widgets::pluralize(
                app.state
                    .plugin_host
                    .registry
                    .entries()
                    .iter()
                    .filter(|entry| entry.is_active())
                    .count(),
                "plugin active",
                "plugins active",
            ))
            .color(theme::TEXT_INFO),
        );
    });
    badge
}

fn vcs_widget(app: &mut IdeApp, ui: &mut egui::Ui) {
    let Some(git) = app.state.git.as_ref() else {
        return;
    };
    let branch = git.branch.clone();
    let response = widgets::flat_widget(
        ui,
        Some((Icon::Branch, *icons::ICON)),
        &branch,
        egui::FontId::proportional(theme::UI_FONT_SIZE),
        *theme::TEXT_BRIGHT,
        true,
        28.0,
    )
    .on_hover_text("Git Branch");
    egui::Popup::menu(&response)
        .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
        .show(|ui| crate::branches_popup::menu(app, ui));
}

fn run_widget(app: &mut IdeApp, ui: &mut egui::Ui) {
    let running = app.state.runner.is_running();
    // Right-to-left layout: rightmost item first.
    if running
        && icons::button_colored(
            ui,
            Icon::Stop,
            *theme::STOP_RED,
            &format!("Stop ({})", app.action_shortcut("Stop").unwrap_or_default()),
            false,
        )
        .clicked()
    {
        app.dispatch("run.cancel", Value::Null);
    }
    let detected = app.state.detected_build.clone();
    let can_build = app.commands.is_enabled("build.project", &app.state);
    let build_tip = match &detected {
        None if app.state.project.is_some() => "No supported build system detected".to_owned(),
        _ => format!(
            "Build Project ({})",
            app.action_shortcut("CompileDirty").unwrap_or_default()
        ),
    };
    let build = ui.add_enabled_ui(can_build, |ui| icons::button(ui, Icon::Build, &build_tip));
    if build.inner.clicked() {
        app.dispatch("build.project", Value::Null);
    }
    // Android apps have no `gradlew run`; Run deploys 'app' to the selected device.
    let android_run = app.state.android.project.is_some();
    let run_command = if android_run {
        "android.app.deploy"
    } else {
        "run.project"
    };
    let can_run = app.commands.is_enabled(run_command, &app.state);
    let run_color = if can_run {
        *theme::RUN_GREEN
    } else {
        *theme::TEXT_DISABLED
    };
    let run_tip = match &detected {
        _ if android_run => format!(
            "Run 'app' ({})",
            app.action_shortcut("Run").unwrap_or_default()
        ),
        Some(build) if build.run_unavailable.is_some() => {
            format!(
                "Run unavailable: {}",
                build.run_unavailable.as_deref().unwrap_or_default()
            )
        }
        None if app.state.project.is_some() => "No supported build system detected".to_owned(),
        _ => format!("Run ({})", app.action_shortcut("Run").unwrap_or_default()),
    };
    let run = ui.add_enabled_ui(can_run, |ui| {
        icons::button_colored(ui, Icon::Run, run_color, &run_tip, false)
    });
    // Disabled widgets do not show the normal tooltip; keep the reason discoverable.
    if !can_run {
        run.response.on_disabled_hover_text(&run_tip);
    } else if run.inner.clicked() {
        app.dispatch(run_command, Value::Null);
    }
    if let Some(build) = detected {
        let response = widgets::flat_widget(
            ui,
            None,
            &build.run_label,
            egui::FontId::proportional(theme::UI_FONT_SIZE),
            *theme::TEXT_BRIGHT,
            true,
            28.0,
        );
        egui::Popup::menu(&response).show(|ui| {
            ui.set_min_width(220.0);
            for (id, label) in [
                ("run.project", "Run"),
                ("build.project", "Build Project"),
                ("test.project", "Run All Tests"),
                ("run.rerun", "Rerun Last Task"),
            ] {
                let enabled = app.commands.is_enabled(id, &app.state);
                if ui.add_enabled(enabled, egui::Button::new(label)).clicked() {
                    app.dispatch(id, Value::Null);
                    ui.close();
                }
            }
            if let Some(reason) = &build.run_unavailable {
                ui.separator();
                ui.label(
                    egui::RichText::new(format!("Run unavailable: {reason}"))
                        .color(theme::TEXT_INFO)
                        .size(theme::UI_FONT_SIZE - 1.0),
                );
            }
        });
    }
}

pub(crate) fn window_controls(ui: &mut egui::Ui) {
    let ctx = ui.ctx().clone();
    let control = |ui: &mut egui::Ui, kind: u8, hover: Color32| {
        let (rect, response) = ui.allocate_exact_size(vec2(40.0, TOOLBAR_HEIGHT), Sense::click());
        if response.hovered() {
            ui.painter().rect_filled(rect, 0.0, hover);
        }
        let center = rect.center();
        let stroke = egui::Stroke::new(1.0_f32, icons::ICON);
        match kind {
            0 => {
                ui.painter()
                    .line_segment([center + vec2(-5.0, 4.0), center + vec2(5.0, 4.0)], stroke);
            }
            1 => {
                ui.painter().rect_stroke(
                    Rect::from_center_size(center, vec2(10.0, 10.0)),
                    1.0,
                    stroke,
                    egui::StrokeKind::Middle,
                );
            }
            _ => {
                ui.painter()
                    .line_segment([center + vec2(-5.0, -5.0), center + vec2(5.0, 5.0)], stroke);
                ui.painter()
                    .line_segment([center + vec2(5.0, -5.0), center + vec2(-5.0, 5.0)], stroke);
            }
        }
        response
    };
    if control(ui, 2, Color32::from_rgb(0xC4, 0x2B, 0x1C)).clicked() {
        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
    }
    if control(ui, 1, *theme::HOVER).clicked() {
        let maximized = ctx.input(|i| i.viewport().maximized.unwrap_or(false));
        ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(!maximized));
    }
    if control(ui, 0, *theme::HOVER).clicked() {
        ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
    }
}

/// Edge handles that resize an undecorated window.
pub fn window_resize_handles(ctx: &egui::Context) {
    if !custom_title_bar() || ctx.input(|i| i.viewport().maximized.unwrap_or(false)) {
        return;
    }
    use egui::ResizeDirection as Dir;
    let screen = ctx.content_rect();
    let edge = 5.0;
    let handles = [
        (
            Rect::from_min_max(
                screen.left_top(),
                screen.left_top() + vec2(edge, edge) * 2.0,
            ),
            Dir::NorthWest,
            egui::CursorIcon::ResizeNorthWest,
        ),
        (
            Rect::from_min_max(
                screen.right_top() - vec2(edge * 2.0, 0.0),
                screen.right_top() + vec2(0.0, edge * 2.0),
            ),
            Dir::NorthEast,
            egui::CursorIcon::ResizeNorthEast,
        ),
        (
            Rect::from_min_max(
                screen.left_bottom() - vec2(0.0, edge * 2.0),
                screen.left_bottom() + vec2(edge * 2.0, 0.0),
            ),
            Dir::SouthWest,
            egui::CursorIcon::ResizeSouthWest,
        ),
        (
            Rect::from_min_max(
                screen.right_bottom() - vec2(edge, edge) * 2.0,
                screen.right_bottom(),
            ),
            Dir::SouthEast,
            egui::CursorIcon::ResizeSouthEast,
        ),
        (
            Rect::from_min_max(screen.left_top(), screen.right_top() + vec2(0.0, edge)),
            Dir::North,
            egui::CursorIcon::ResizeVertical,
        ),
        (
            Rect::from_min_max(
                screen.left_bottom() - vec2(0.0, edge),
                screen.right_bottom(),
            ),
            Dir::South,
            egui::CursorIcon::ResizeVertical,
        ),
        (
            Rect::from_min_max(screen.left_top(), screen.left_bottom() + vec2(edge, 0.0)),
            Dir::West,
            egui::CursorIcon::ResizeHorizontal,
        ),
        (
            Rect::from_min_max(screen.right_top() - vec2(edge, 0.0), screen.right_bottom()),
            Dir::East,
            egui::CursorIcon::ResizeHorizontal,
        ),
    ];
    egui::Area::new(egui::Id::new("window_resize_handles"))
        .order(egui::Order::Foreground)
        .fixed_pos(screen.left_top())
        .interactable(true)
        .show(ctx, |ui| {
            for (index, (rect, direction, cursor)) in handles.into_iter().enumerate() {
                let response = ui
                    .interact(rect, egui::Id::new(("resize", index)), Sense::drag())
                    .on_hover_cursor(cursor);
                if response.drag_started() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::BeginResize(direction));
                }
            }
        });
}

pub(crate) fn stripe_button(app: &mut IdeApp, ui: &mut egui::Ui, window: ToolWindow, icon: Icon) {
    let selected = app.state.tool_windows.is_visible(window);
    let (rect, response) = ui.allocate_exact_size(vec2(32.0, 32.0), Sense::click());
    let fill = if selected {
        *theme::ACCENT
    } else if response.hovered() {
        *theme::HOVER
    } else {
        Color32::TRANSPARENT
    };
    ui.painter().rect_filled(rect, 6.0, fill);
    let color = if selected {
        Color32::WHITE
    } else {
        *icons::ICON
    };
    icons::paint(
        ui.painter(),
        Rect::from_center_size(rect.center(), vec2(16.0, 16.0)),
        icon,
        color,
    );
    let mut badge = None;
    if window == ToolWindow::Problems {
        let counts = app.state.problems.counts();
        if counts.errors > 0 {
            badge = Some(theme::ERROR);
        } else if counts.warnings > 0 {
            badge = Some(theme::WARNING);
        }
    }
    if window == ToolWindow::Build && app.state.runner.is_running() {
        badge = Some(theme::RUN_GREEN);
    }
    if let Some(color) = badge {
        ui.painter()
            .circle_filled(rect.right_top() + vec2(-7.0, 7.0), 3.0, color);
    }
    if response
        .on_hover_text(format!(
            "{}{}",
            window.title(),
            match window.shortcut() {
                "" => String::new(),
                shortcut => format!(" ({})", keys::display(shortcut)),
            }
        ))
        .clicked()
    {
        app.toggle_tool_window(window);
    }
}

pub fn left_stripe(app: &mut IdeApp, ctx: &egui::Context) {
    egui::SidePanel::left("left_stripe")
        .exact_width(STRIPE_WIDTH)
        .resizable(false)
        .show_separator_line(false)
        .frame(chrome_frame().inner_margin(egui::Margin::symmetric(4, 6)))
        .show(ctx, |ui| {
            ui.spacing_mut().item_spacing.y = 6.0;
            ui.vertical(|ui| {
                stripe_button(app, ui, ToolWindow::Project, Icon::Project);
                stripe_button(app, ui, ToolWindow::Commit, Icon::Commit);
                if app.state.android.project.is_some() {
                    stripe_button(app, ui, ToolWindow::BuildVariants, Icon::Variants);
                }
                let (rect, _) = ui.allocate_exact_size(vec2(32.0, 1.0), Sense::hover());
                ui.painter().line_segment(
                    [
                        rect.left_center() + vec2(6.0, 0.0),
                        rect.right_center() - vec2(6.0, 0.0),
                    ],
                    egui::Stroke::new(1.0_f32, theme::TOOLBAR_SEPARATOR),
                );
                ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
                    stripe_button(app, ui, ToolWindow::Git, Icon::Branch);
                    if app.state.android.project.is_some() {
                        stripe_button(app, ui, ToolWindow::Logcat, Icon::Logcat);
                    }
                    stripe_button(app, ui, ToolWindow::Problems, Icon::Problems);
                    stripe_button(app, ui, ToolWindow::Terminal, Icon::Terminal);
                    stripe_button(app, ui, ToolWindow::Build, Icon::Build);
                    stripe_button(app, ui, ToolWindow::Find, Icon::Find);
                });
            });
        });
}

/// Whether the right stripe offers the Device Manager (Android projects).
pub(crate) fn shows_device_manager(app: &IdeApp) -> bool {
    app.state.android.project.is_some()
        || app.state.tool_windows.is_visible(ToolWindow::DeviceManager)
}

/// Right tool window stripe. Without right-anchored tool windows (non-Android
/// projects) it is a narrow gutter so the editor island keeps its rounded edge.
/// Plugin tool windows join this stripe (`plugin_ui::right_side`).
pub fn right_stripe(app: &mut IdeApp, ctx: &egui::Context) {
    if !shows_device_manager(app) {
        egui::SidePanel::right("right_stripe")
            .exact_width(6.0)
            .resizable(false)
            .show_separator_line(false)
            .frame(chrome_frame())
            .show(ctx, |_| {});
        return;
    }
    egui::SidePanel::right("right_stripe")
        .exact_width(STRIPE_WIDTH)
        .resizable(false)
        .show_separator_line(false)
        .frame(chrome_frame().inner_margin(egui::Margin::symmetric(4, 6)))
        .show(ctx, |ui| {
            ui.spacing_mut().item_spacing.y = 6.0;
            stripe_button(app, ui, ToolWindow::DeviceManager, Icon::Device);
        });
}

/// First line of `text`, shortened to `max` characters with an ellipsis.
fn first_line_truncated(text: &str, max: usize) -> String {
    let line = text.lines().next().unwrap_or_default();
    if line.chars().count() > max {
        let mut short: String = line.chars().take(max.saturating_sub(1)).collect();
        short.push('…');
        short
    } else {
        line.to_owned()
    }
}

pub fn status_bar(app: &mut IdeApp, ctx: &egui::Context) {
    egui::TopBottomPanel::bottom("status_bar")
        .exact_height(STATUS_BAR_HEIGHT)
        .show_separator_line(false)
        .frame(chrome_frame().inner_margin(egui::Margin::symmetric(8, 0)))
        .show(ctx, |ui| {
            ui.horizontal_centered(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                let font = egui::FontId::proportional(theme::UI_FONT_SIZE - 1.0);
                // Navigation bar breadcrumbs: project › directories › file.
                let mut crumbs = vec![app.project_name()];
                if let Some(document) = app
                    .state
                    .active_document
                    .and_then(|index| app.state.documents.get(index))
                {
                    crumbs.extend(
                        document
                            .path
                            .components()
                            .map(|part| part.as_os_str().to_string_lossy().into_owned()),
                    );
                }
                let last = crumbs.len().saturating_sub(1);
                for (index, crumb) in crumbs.iter().enumerate() {
                    let icon = if index == 0 {
                        Some((Icon::Project, *theme::TEXT_SECONDARY))
                    } else if index == last {
                        Some((Icon::for_file(crumb), *icons::ICON))
                    } else {
                        None
                    };
                    let color = if index == last {
                        *theme::TEXT
                    } else {
                        *theme::TEXT_SECONDARY
                    };
                    widgets::flat_widget(ui, icon, crumb, font.clone(), color, false, 22.0);
                    if index != last {
                        icons::show(ui, Icon::ChevronRight, 12.0, *theme::TEXT_INFO);
                    }
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    crate::plugin_ui::status_widgets(app, ui, &font);
                    let secondary = *theme::TEXT_SECONDARY;
                    crate::lsp_status::status_widget(app, ui, font.clone());
                    crate::toolchain_status::status_widget(app, ui, font.clone());
                    let active = app
                        .state
                        .active_document
                        .and_then(|index| app.state.documents.get(index));
                    if let Some(document) = active {
                        // Bounded detection: never copy the whole buffer per frame.
                        let indent = jetthing_text::format::detect_indent(&document.text)
                            .unwrap_or(jetthing_text::format::Indent::Spaces(4));
                        let separator =
                            jetthing_text::format::detect_line_separator(&document.text);
                        let caret = document.selections.primary().head;
                        let (line, column) =
                            document.text.char_to_line_column(caret).unwrap_or((0, 0));
                        if let Some(language) = document.analysis.language {
                            widgets::flat_widget(
                                ui,
                                None,
                                language,
                                font.clone(),
                                secondary,
                                false,
                                22.0,
                            );
                        }
                        widgets::flat_widget(
                            ui,
                            None,
                            &indent.label(),
                            font.clone(),
                            secondary,
                            false,
                            22.0,
                        );
                        widgets::flat_widget(
                            ui,
                            None,
                            "UTF-8",
                            font.clone(),
                            secondary,
                            false,
                            22.0,
                        );
                        widgets::flat_widget(
                            ui,
                            None,
                            separator.label(),
                            font.clone(),
                            secondary,
                            false,
                            22.0,
                        );
                        widgets::flat_widget(
                            ui,
                            None,
                            &crate::editor_keys::caret_position_label(
                                &document.text,
                                &document.selections,
                                (line, column),
                            ),
                            font.clone(),
                            secondary,
                            false,
                            22.0,
                        );
                        if document.selections.column_mode() {
                            widgets::flat_widget(
                                ui,
                                None,
                                "Column",
                                font.clone(),
                                secondary,
                                false,
                                22.0,
                            );
                        }
                    }
                    if let jetthing_core::UpdateState::Available { version, .. }
                    | jetthing_core::UpdateState::Installed(version) = &app.state.update_state
                    {
                        let label = if matches!(
                            app.state.update_state,
                            jetthing_core::UpdateState::Installed(_)
                        ) {
                            "Restart to update".to_owned()
                        } else {
                            format!("Update {version}")
                        };
                        if widgets::flat_widget(
                            ui,
                            Some((Icon::Refresh, *theme::MODIFIED)),
                            &label,
                            font.clone(),
                            *theme::MODIFIED,
                            false,
                            22.0,
                        )
                        .clicked()
                        {
                            app.state.update_notification = true;
                        }
                    }
                    if app.state.runner.is_running() || app.state.search.is_running() {
                        ui.add(egui::Spinner::new().size(12.0).color(theme::ACCENT));
                        ui.add_space(6.0);
                    }
                    if !app.state.status.is_empty() {
                        let status = first_line_truncated(&app.state.status, 70);
                        let response = widgets::flat_widget(
                            ui,
                            None,
                            &status,
                            font.clone(),
                            *theme::TEXT_INFO,
                            false,
                            22.0,
                        );
                        let response = response.on_hover_text(app.state.status.clone());
                        if response.clicked()
                            && app.state.tool_windows.visible(Anchor::Bottom).is_none()
                        {
                            app.dispatch("toolwindow.toggle", json!({"id": "build"}));
                        }
                    }
                });
            });
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncates_status_messages() {
        assert_eq!(first_line_truncated("short", 10), "short");
        assert_eq!(first_line_truncated("abcdefghijkl", 5), "abcd…");
        assert_eq!(first_line_truncated("one\ntwo", 10), "one");
    }
}
