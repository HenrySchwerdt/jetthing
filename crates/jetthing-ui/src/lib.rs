//! Native workbench shell.
//!
//! Layout follows the observable structure of the New UI "Islands" workbench:
//! a merged main toolbar, a tool window stripe, rounded editor/tool window
//! islands and a status bar with navigation breadcrumbs. Every user action is
//! dispatched through the typed command registry.

mod chrome;
mod editor;
mod findbar;
mod icons;
mod keys;
mod lsp_status;
mod notifications;
mod palette;
mod popups;
mod settings_dialog;
mod syntax;
mod terminal_ui;
mod theme;
mod toolwindows;
mod welcome;
mod widgets;

use eframe::egui;
use jetthing_command::{CommandRegistry, CommandSpec};
use jetthing_core::{Anchor, IdeState, ToolWindow, built_in_commands};
use jetthing_settings::ids;
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

pub use theme::install as install_theme;

pub struct IdeApp {
    state: IdeState,
    commands: CommandRegistry<IdeState>,
    palette: palette::Palette,
    terminal_selection: Option<(terminal_ui::GridPoint, terminal_ui::GridPoint)>,
    search_input: String,
    tree_selection: Option<PathBuf>,
    completion_selected: usize,
    caret_screen_pos: Option<egui::Pos2>,
    run_label: Option<String>,
    last_shift_release: Option<Instant>,
    shift_tainted: bool,
    last_bottom: Option<ToolWindow>,
    find: findbar::FindBar,
    /// Text of the last whole-line copy, so pasting it inserts a line.
    clipboard_line: Option<String>,
    dialog: Option<popups::Dialog>,
    settings_dialog: Option<settings_dialog::SettingsDialog>,
    dialog_focus: bool,
    commit_message: String,
    /// Commit checkbox overrides; by default tracked changes are included and
    /// unversioned files are not.
    commit_overrides: std::collections::HashMap<PathBuf, bool>,
    window_focused: bool,
}

impl IdeApp {
    /// Opens `project` (if any) and then each of `files`, relative to the project root
    /// or absolute inside it.
    pub fn new(project: Option<PathBuf>, files: Vec<PathBuf>) -> Self {
        let commands = built_in_commands();
        let mut state = IdeState::default();
        if let Some(dir) = jetthing_settings::default_config_dir() {
            state.load_user_settings(&dir);
        }
        if let Some(path) = project {
            if let Err(error) = commands.execute("project.open", &mut state, json!({"path": path}))
            {
                state.status = error.to_string();
            }
            let _ = commands.execute("plugin.refresh", &mut state, Value::Null);
        }
        for file in files {
            let relative = state
                .project
                .as_ref()
                .and_then(|project| {
                    let absolute = file.canonicalize().ok()?;
                    absolute
                        .strip_prefix(project.root())
                        .ok()
                        .map(Path::to_path_buf)
                })
                .unwrap_or(file);
            if let Err(error) =
                commands.execute("editor.openFile", &mut state, json!({"path": relative}))
            {
                state.status = error.to_string();
            }
        }
        if std::env::var_os("JETTHING_NO_UPDATE_CHECK").is_none()
            && state.settings.bool(ids::CHECK_UPDATES).unwrap_or(true)
        {
            let _ = commands.execute("app.checkForUpdates", &mut state, json!({"silent": true}));
        }
        let run_label = state.project.as_ref().and_then(|project| {
            jetthing_runner::TaskPlan::detect_run(project.root())
                .or_else(|_| jetthing_runner::TaskPlan::detect_build(project.root()))
                .ok()
                .map(|plan| plan.name)
        });
        Self {
            state,
            commands,
            palette: palette::Palette::default(),
            terminal_selection: None,
            search_input: String::new(),
            tree_selection: None,
            completion_selected: 0,
            caret_screen_pos: None,
            run_label,
            last_shift_release: None,
            shift_tainted: false,
            last_bottom: None,
            find: findbar::FindBar::default(),
            clipboard_line: None,
            dialog: None,
            settings_dialog: None,
            dialog_focus: false,
            commit_message: String::new(),
            commit_overrides: std::collections::HashMap::new(),
            window_focused: true,
        }
    }

    fn dispatch(&mut self, id: &str, params: Value) {
        if let Err(error) = self.commands.execute(id, &mut self.state, params) {
            self.state.status = error.to_string();
        }
    }

    fn open_dialog(&mut self, dialog: popups::Dialog) {
        self.dialog = Some(dialog);
        self.dialog_focus = true;
    }

    fn open_project_dialog(&mut self) {
        self.open_dialog(popups::Dialog::OpenProject {
            path: String::new(),
            error: None,
        });
    }

    fn new_project_dialog(&mut self) {
        self.open_dialog(popups::Dialog::NewProject {
            name: "untitled".into(),
            parent: std::env::var("HOME")
                .or_else(|_| std::env::var("USERPROFILE"))
                .unwrap_or_else(|_| ".".into()),
            kind: "java-gradle-kotlin".into(),
            group: "org.example".into(),
            error: None,
        });
    }

    fn open_project_path(&mut self, path: &str, ctx: &egui::Context) -> Result<(), String> {
        self.commands
            .execute(
                "project.open",
                &mut self.state,
                json!({"path": path.trim()}),
            )
            .map_err(|error| error.to_string())?;
        self.project_opened(ctx);
        Ok(())
    }

    fn project_opened(&mut self, ctx: &egui::Context) {
        self.dispatch("plugin.refresh", Value::Null);
        self.tree_selection = None;
        self.commit_overrides.clear();
        self.commit_message.clear();
        self.search_input.clear();
        self.terminal_selection = None;
        self.find = findbar::FindBar::default();
        self.run_label = self.state.project.as_ref().and_then(|project| {
            jetthing_runner::TaskPlan::detect_run(project.root())
                .or_else(|_| jetthing_runner::TaskPlan::detect_build(project.root()))
                .ok()
                .map(|plan| plan.name)
        });
        ctx.send_viewport_cmd(egui::ViewportCommand::Title(format!(
            "{} – Jetthing",
            self.project_name()
        )));
    }

    fn editor_focused(&self, ctx: &egui::Context) -> bool {
        ctx.memory(|memory| memory.has_focus(editor::editor_id()))
    }

    fn primary_selection_text(&self) -> Option<String> {
        let document = self.state.documents.get(self.state.active_document?)?;
        let primary = document.selections.primary();
        document.text.slice(primary.start(), primary.end()).ok()
    }

    /// Saves modified files silently (autosave); nothing to save is not an error.
    fn autosave(&mut self) {
        if self.state.documents.iter().any(|d| d.text.is_dirty()) {
            self.dispatch("editor.saveAll", Value::Null);
        }
    }

    fn toggle_tool_window(&mut self, window: ToolWindow) {
        self.dispatch("toolwindow.toggle", json!({"id": window.id()}));
    }

    fn file_name(path: &Path) -> String {
        path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("untitled")
            .to_owned()
    }

    fn project_name(&self) -> String {
        self.state
            .project
            .as_ref()
            .map(|project| Self::file_name(project.root()))
            .unwrap_or_else(|| "No project".into())
    }

    /// Commands that can run without parameters (all declared fields optional).
    fn parameterless(spec: &CommandSpec) -> bool {
        match serde_json::from_str::<Value>(&spec.params) {
            Ok(Value::Object(fields)) => fields
                .values()
                .all(|value| value.as_str().is_some_and(|kind| kind.ends_with('?'))),
            _ => false,
        }
    }

    fn handle_shortcuts(&mut self, ctx: &egui::Context) {
        use egui::{Key, Modifiers};
        // Tool window activation uses ⌘N on macOS and Alt+N elsewhere.
        let tool_modifier = if cfg!(target_os = "macos") {
            Modifiers::COMMAND
        } else {
            Modifiers::ALT
        };
        let completion_open = !self.state.lsp.completions().is_empty();
        let consumed_chord = std::cell::Cell::new(false);
        // Exact modifier matching: egui's consume_key ignores extra Shift/Alt, which
        // would let ⌘F swallow ⇧⌘F.
        let consume = |modifiers: Modifiers, key: Key| {
            let hit = ctx.input_mut(|input| {
                let before = input.events.len();
                input.events.retain(|event| {
                    !matches!(
                        event,
                        egui::Event::Key { key: k, modifiers: m, pressed: true, .. }
                            if *k == key && m.matches_exact(modifiers)
                    )
                });
                input.events.len() != before
            });
            if hit && (modifiers.alt || modifiers.ctrl || modifiers.command) {
                consumed_chord.set(true);
            }
            hit
        };
        if completion_open
            && !self.palette.open
            && self.dialog.is_none()
            && self.editor_focused(ctx)
        {
            let count = self
                .state
                .lsp
                .completions()
                .len()
                .min(editor::COMPLETION_LIMIT);
            if consume(Modifiers::NONE, Key::ArrowDown) && count > 0 {
                self.completion_selected = (self.completion_selected + 1) % count;
            }
            if consume(Modifiers::NONE, Key::ArrowUp) && count > 0 {
                self.completion_selected = (self.completion_selected + count - 1) % count;
            }
            if consume(Modifiers::NONE, Key::Enter) || consume(Modifiers::NONE, Key::Tab) {
                self.accept_completion(self.completion_selected);
            }
        }
        let mac = cfg!(target_os = "macos");
        // Settings: ⌘, on macOS, Ctrl+Alt+S elsewhere.
        if (mac && consume(Modifiers::COMMAND, Key::Comma))
            || (!mac && consume(Modifiers::CTRL | Modifiers::ALT, Key::S))
        {
            self.dispatch("settings.open", Value::Null);
        }
        if self.settings_dialog.is_some() || self.state.settings_request.is_some() {
            // The dialog is modal and handles its own keys (Escape closes it).
            return;
        }
        if ctx.memory(|memory| memory.has_focus(toolwindows::terminal_input_id()))
            && self.dialog.is_none()
            && !self.palette.open
        {
            if consume(Modifiers::COMMAND | Modifiers::SHIFT, Key::A) {
                self.palette.show_tab(palette::Tab::Actions);
            }
            if consume(Modifiers::COMMAND, Key::O) {
                self.open_project_dialog();
            }
            if consume(Modifiers::ALT, Key::F12) || consume(Modifiers::CTRL, Key::Backtick) {
                self.toggle_tool_window(ToolWindow::Terminal);
            }
            self.detect_double_shift(ctx);
            if consumed_chord.get() {
                ctx.input_mut(|input| {
                    input
                        .events
                        .retain(|event| !matches!(event, egui::Event::Text(_)))
                });
            }
            return;
        }
        if consume(Modifiers::COMMAND, Key::O) {
            self.open_project_dialog();
        }
        if consume(Modifiers::COMMAND | Modifiers::SHIFT, Key::A) {
            self.palette.show_tab(palette::Tab::Actions);
        }
        // Go to File: ⇧⌘O on macOS, Ctrl+Shift+N elsewhere.
        if (mac && consume(Modifiers::COMMAND | Modifiers::SHIFT, Key::O))
            || (!mac && consume(Modifiers::COMMAND | Modifiers::SHIFT, Key::N))
        {
            self.palette.show_tab(palette::Tab::Files);
        }
        if consume(Modifiers::COMMAND, Key::E) {
            self.open_dialog(popups::Dialog::Recent {
                query: String::new(),
                selected: usize::from(self.state.recent_files.len() > 1),
            });
        }
        if self.state.active_document.is_some() {
            // Go to Line: ⌘L on macOS, Ctrl+G elsewhere.
            if (mac && consume(Modifiers::COMMAND, Key::L))
                || (!mac && consume(Modifiers::COMMAND, Key::G))
            {
                self.open_dialog(popups::Dialog::GotoLine {
                    input: String::new(),
                });
            }
            if consume(Modifiers::COMMAND, Key::F) {
                let selection = self.primary_selection_text();
                self.find.show(false, selection);
            }
            if consume(Modifiers::COMMAND, Key::R) {
                let selection = self.primary_selection_text();
                self.find.show(true, selection);
            }
            let next =
                consume(Modifiers::NONE, Key::F3) || (mac && consume(Modifiers::COMMAND, Key::G));
            let previous = consume(Modifiers::SHIFT, Key::F3)
                || (mac && consume(Modifiers::COMMAND | Modifiers::SHIFT, Key::G));
            if (next || previous) && !self.find.query.is_empty() {
                let id = if next {
                    "editor.findNext"
                } else {
                    "editor.findPrevious"
                };
                self.dispatch(id, self.find.params());
            }
        }
        self.detect_double_shift(ctx);
        if consume(Modifiers::CTRL, Key::Space) {
            self.dispatch("language.complete", Value::Null);
        }
        if consume(Modifiers::NONE, Key::Escape) {
            if self.dialog.is_some() {
                self.dialog = None;
            } else if self.palette.open {
                self.palette.open = false;
            } else if self.find.open
                && (self.editor_focused(ctx) || ctx.memory(|m| m.has_focus(findbar::query_id())))
            {
                self.find.open = false;
                ctx.memory_mut(|memory| memory.request_focus(editor::editor_id()));
            } else if !self.state.lsp.completions().is_empty() {
                self.state.lsp.clear_completions();
            } else if self.state.tool_windows.visible(Anchor::Bottom).is_some()
                && !self.palette.open
            {
                // Escape returns focus to the editor; keep the tool window open.
                ctx.memory_mut(|memory| memory.request_focus(editor::editor_id()));
            }
        }
        if consume(Modifiers::COMMAND, Key::S) {
            self.autosave();
        }
        if consume(Modifiers::COMMAND | Modifiers::SHIFT, Key::Z) {
            self.dispatch("editor.redo", Value::Null);
        }
        if consume(Modifiers::COMMAND, Key::Z) {
            self.dispatch("editor.undo", Value::Null);
        }
        let close_tab = consume(Modifiers::COMMAND, Key::W)
            || consume(Modifiers::COMMAND, Key::F4)
            || consume(Modifiers::CTRL, Key::F4);
        if close_tab && self.state.active_document.is_some() {
            self.dispatch("editor.close", Value::Null);
        }
        if consume(Modifiers::COMMAND, Key::F9) {
            self.dispatch("build.project", Value::Null);
        }
        // Run: ⌃R on macOS, Shift+F10 elsewhere (Ctrl+R is Replace there).
        if (mac && consume(Modifiers::CTRL, Key::R)) || consume(Modifiers::SHIFT, Key::F10) {
            self.dispatch("run.project", Value::Null);
        }
        if consume(Modifiers::COMMAND, Key::F2) && self.state.runner.is_running() {
            self.dispatch("run.cancel", Value::Null);
        }
        if consume(Modifiers::CTRL, Key::Backtick) || consume(Modifiers::ALT, Key::F12) {
            self.toggle_tool_window(ToolWindow::Terminal);
        }
        if consume(Modifiers::COMMAND | Modifiers::SHIFT, Key::F) {
            self.dispatch("toolwindow.activate.find", Value::Null);
            self.search_focus_requested(ctx);
        }
        if consume(Modifiers::COMMAND, Key::K) {
            self.dispatch("toolwindow.activate.commit", Value::Null);
        }
        if consume(Modifiers::COMMAND | Modifiers::SHIFT, Key::F12) {
            self.dispatch("toolwindow.hideAll", Value::Null);
        }
        for (key, window) in [
            (Key::Num1, ToolWindow::Project),
            (Key::Num0, ToolWindow::Commit),
            (Key::Num3, ToolWindow::Find),
            (Key::Num4, ToolWindow::Build),
            (Key::Num6, ToolWindow::Problems),
        ] {
            if consume(tool_modifier, key) {
                self.toggle_tool_window(window);
            }
        }
        // Some platforms also emit the chord's character as text (e.g. Alt+1 → "1");
        // a consumed shortcut must not type into the focused field.
        if consumed_chord.get() {
            ctx.input_mut(|input| {
                input
                    .events
                    .retain(|event| !matches!(event, egui::Event::Text(_)))
            });
        }
    }

    /// Activating a tool window moves keyboard focus into it; hiding it returns
    /// focus to the editor.
    fn follow_tool_window_focus(&mut self, ctx: &egui::Context) {
        let bottom = self.state.tool_windows.visible(Anchor::Bottom);
        if bottom == self.last_bottom {
            return;
        }
        let target = match bottom {
            Some(ToolWindow::Terminal) => Some(toolwindows::terminal_input_id()),
            Some(ToolWindow::Find) => Some(toolwindows::find_input_id()),
            None if self.state.active_document.is_some() => Some(editor::editor_id()),
            _ => None,
        };
        if let Some(id) = target {
            ctx.memory_mut(|memory| memory.request_focus(id));
        }
        self.last_bottom = bottom;
    }

    fn search_focus_requested(&self, ctx: &egui::Context) {
        ctx.memory_mut(|memory| memory.request_focus(toolwindows::find_input_id()));
    }

    /// "Search Everywhere" opens on a quick double press of Shift alone.
    fn detect_double_shift(&mut self, ctx: &egui::Context) {
        // egui reports no key events for Shift itself, so any key or text event
        // while Shift is held means it was used as a modifier.
        let used_as_modifier = ctx.input(|input| {
            input.events.iter().any(|event| {
                matches!(
                    event,
                    egui::Event::Key { pressed: true, .. } | egui::Event::Text(_)
                )
            })
        });
        if used_as_modifier {
            self.shift_tainted = true;
            self.last_shift_release = None;
        }
        let shift_down = ctx.input(|input| input.modifiers.shift_only());
        let was_down = ctx.data(|data| {
            data.get_temp::<bool>(egui::Id::new("shift_down"))
                .unwrap_or(false)
        });
        ctx.data_mut(|data| data.insert_temp(egui::Id::new("shift_down"), shift_down));
        if shift_down && !was_down {
            self.shift_tainted = false;
        }
        if was_down && !shift_down && !self.shift_tainted {
            let now = Instant::now();
            if self
                .last_shift_release
                .is_some_and(|last| now.duration_since(last) < Duration::from_millis(400))
            {
                self.palette.show();
                self.last_shift_release = None;
            } else {
                self.last_shift_release = Some(now);
            }
        }
    }

    fn accept_completion(&mut self, index: usize) {
        let Some(item) = self.state.lsp.completions().get(index).cloned() else {
            return;
        };
        let text = item.insert_text.unwrap_or(item.label);
        self.dispatch(
            "editor.insertText",
            json!({"position": self.state.active_caret().unwrap_or(0), "text": text}),
        );
        self.state.lsp.clear_completions();
        self.completion_selected = 0;
    }
}

impl eframe::App for IdeApp {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        theme::FRAME.to_normalized_gamma_f32()
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.state.poll_search();
        self.state.poll_runner();
        self.state.poll_lsp();
        self.state.poll_updates();
        if self.state.lsp.is_busy() {
            ctx.request_repaint_after(Duration::from_millis(100));
        } else if !self.state.lsp.servers().is_empty() {
            // Diagnostics and server messages arrive without user input.
            ctx.request_repaint_after(Duration::from_millis(500));
        }
        if self.state.search.is_running() || self.state.runner.is_running() {
            ctx.request_repaint_after(Duration::from_millis(50));
        }
        if let Some(terminal) = self.state.terminal.as_mut() {
            terminal.poll();
            ctx.request_repaint_after(Duration::from_millis(100));
        }
        // Like the reference IDE, save all files when the window loses focus.
        let focused = ctx.input(|input| input.focused);
        if self.window_focused
            && !focused
            && self
                .state
                .settings
                .bool(ids::AUTOSAVE_FOCUS)
                .unwrap_or(true)
        {
            self.autosave();
        }
        self.window_focused = focused;
        self.handle_shortcuts(ctx);
        self.follow_tool_window_focus(ctx);
        theme::apply_settings(ctx, &self.state.settings);

        chrome::window_resize_handles(ctx);
        chrome::main_toolbar(self, ctx);
        chrome::status_bar(self, ctx);
        if self.state.project.is_none() {
            welcome::show(self, ctx);
        } else {
            chrome::left_stripe(self, ctx);
            chrome::right_stripe(ctx);
            toolwindows::left(self, ctx);
            toolwindows::bottom(self, ctx);
            editor::central(self, ctx);
        }
        editor::completion_popup(self, ctx);
        palette::show(self, ctx);
        popups::show(self, ctx);
        settings_dialog::show(self, ctx);
        notifications::show(self, ctx);
        lsp_status::notices(self, ctx);
        // Include operations started by controls in this frame, and keep
        // polling even when their notification balloon is hidden.
        if self.state.updates.is_busy() {
            ctx.request_repaint_after(Duration::from_millis(200));
        }
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        if self.state.project.is_some() {
            self.autosave();
            self.dispatch("workspace.save", Value::Null);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parameterless_commands_are_detected_from_their_schema() {
        let specs = built_in_commands().specs();
        let find = |id: &str| specs.iter().find(|spec| spec.id == id).unwrap();
        assert!(IdeApp::parameterless(find("editor.save")));
        assert!(IdeApp::parameterless(find("editor.close")));
        assert!(!IdeApp::parameterless(find("editor.openFile")));
        assert!(!IdeApp::parameterless(find("toolwindow.toggle")));
    }
}
