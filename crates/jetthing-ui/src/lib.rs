use eframe::egui::{self, Color32, RichText};
use jetthing_command::CommandRegistry;
use jetthing_core::{Document, IdeState, built_in_commands};
use jetthing_git::ChangeKind;
use jetthing_language::{LanguageAnalysis, SyntaxKind};
use jetthing_lsp::ServerState;
use jetthing_runner::{OutputStream, TaskState};
use jetthing_text::Selection;
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

mod keymap_dispatch;

pub struct IdeApp {
    state: IdeState,
    commands: CommandRegistry<IdeState>,
    palette_open: bool,
    palette_query: String,
    terminal_input: String,
    search_input: String,
    keymap: keymap_dispatch::KeymapDispatcher,
}

impl IdeApp {
    pub fn new(project: Option<PathBuf>) -> Self {
        let commands = built_in_commands();
        let mut state = IdeState::default();
        if let Some(path) = project {
            let _ = commands.execute("project.open", &mut state, json!({"path": path}));
            let _ = commands.execute("plugin.refresh", &mut state, Value::Null);
        }
        Self {
            state,
            commands,
            palette_open: false,
            palette_query: String::new(),
            terminal_input: String::new(),
            search_input: String::new(),
            keymap: keymap_dispatch::KeymapDispatcher::default(),
        }
    }
    fn dispatch(&mut self, id: &str, params: Value) {
        if let Err(error) = self.commands.execute(id, &mut self.state, params) {
            self.state.status = error.to_string();
        }
    }
    fn file_name(path: &Path) -> String {
        path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("untitled")
            .to_owned()
    }

    fn highlighted_line_job(
        text: &str,
        line_start_byte: usize,
        analysis: &LanguageAnalysis,
    ) -> egui::text::LayoutJob {
        let mut job = egui::text::LayoutJob::default();
        let base = egui::TextFormat {
            font_id: egui::FontId::monospace(14.0),
            color: Color32::from_rgb(205, 210, 220),
            ..Default::default()
        };
        let line_end_byte = line_start_byte + text.len();
        let mut cursor = line_start_byte;
        for span in analysis
            .spans
            .iter()
            .filter(|span| span.start_byte < line_end_byte && span.end_byte > line_start_byte)
        {
            let start = span.start_byte.max(line_start_byte);
            let end = span.end_byte.min(line_end_byte);
            if start < cursor || end < start {
                continue;
            }
            job.append(
                &text[cursor - line_start_byte..start - line_start_byte],
                0.0,
                base.clone(),
            );
            let color = match span.kind {
                SyntaxKind::Keyword => Color32::from_rgb(207, 142, 255),
                SyntaxKind::String => Color32::from_rgb(167, 212, 142),
                SyntaxKind::Comment => Color32::from_rgb(106, 153, 85),
                SyntaxKind::Number => Color32::from_rgb(104, 151, 187),
                SyntaxKind::Type => Color32::from_rgb(86, 182, 194),
                SyntaxKind::Function => Color32::from_rgb(220, 220, 170),
                SyntaxKind::Macro => Color32::from_rgb(255, 198, 109),
            };
            job.append(
                &text[start - line_start_byte..end - line_start_byte],
                0.0,
                egui::TextFormat {
                    color,
                    ..base.clone()
                },
            );
            cursor = end;
        }
        job.append(&text[cursor - line_start_byte..], 0.0, base);
        job
    }

    fn editor_surface(ui: &mut egui::Ui, document: &mut Document) -> Result<bool, String> {
        let row_height = 20.0;
        let gutter_width = 52.0;
        let char_width =
            ui.fonts_mut(|fonts| fonts.glyph_width(&egui::FontId::monospace(14.0), 'm'));
        let line_count = document.text.len_lines();
        let output = egui::ScrollArea::both()
            .auto_shrink([false, false])
            .show_viewport(ui, |ui, viewport| {
                let content_size = egui::vec2(
                    ui.available_width().max(1600.0),
                    line_count as f32 * row_height,
                );
                let (rect, response) =
                    ui.allocate_exact_size(content_size, egui::Sense::click_and_drag());
                if response.clicked() || response.drag_started() {
                    response.request_focus();
                }
                let position_to_offset = |position: egui::Pos2| {
                    let line = (((position.y - rect.top()) / row_height).floor().max(0.0) as usize)
                        .min(line_count.saturating_sub(1));
                    let line_text = document.text.line(line).unwrap_or_default();
                    let content = line_text.trim_end_matches(['\r', '\n']);
                    let column = (((position.x - rect.left() - gutter_width) / char_width)
                        .round()
                        .max(0.0) as usize)
                        .min(content.chars().count());
                    document.text.line_to_char(line).unwrap_or(0) + column
                };
                if response.clicked()
                    && let Some(position) = response.interact_pointer_pos()
                {
                    let offset = position_to_offset(position);
                    let modifiers = ui.input(|input| input.modifiers);
                    if modifiers.alt {
                        document.selections.add(Selection::caret(offset));
                    } else if modifiers.shift {
                        let anchor = document.selections.primary().anchor;
                        document.selections.set_single(Selection {
                            anchor,
                            head: offset,
                        });
                    } else {
                        document.selections.set_single(Selection::caret(offset));
                    }
                }
                if response.drag_started()
                    && let Some(position) = response.interact_pointer_pos()
                {
                    document
                        .selections
                        .set_single(Selection::caret(position_to_offset(position)));
                }
                if response.dragged()
                    && let Some(position) = response.interact_pointer_pos()
                {
                    let offset = position_to_offset(position);
                    let anchor = document.selections.primary().anchor;
                    document.selections.set_single(Selection {
                        anchor,
                        head: offset,
                    });
                }

                let first_line =
                    ((viewport.top() / row_height).floor().max(0.0) as usize).min(line_count);
                let last_line =
                    ((viewport.bottom() / row_height).ceil() as usize + 1).min(line_count);
                for line in first_line..last_line {
                    let y = rect.top() + line as f32 * row_height;
                    let line_start = document.text.line_to_char(line).unwrap_or(0);
                    let line_text = document.text.line(line).unwrap_or_default();
                    let content = line_text.trim_end_matches(['\r', '\n']);
                    let line_end = line_start + content.chars().count();
                    ui.painter().text(
                        egui::pos2(rect.left() + 6.0, y + 2.0),
                        egui::Align2::LEFT_TOP,
                        format!("{}", line + 1),
                        egui::FontId::monospace(13.0),
                        Color32::from_rgb(95, 101, 115),
                    );
                    for selection in document.selections.selections() {
                        let start = selection.start().max(line_start).min(line_end);
                        let end = selection.end().max(line_start).min(line_end);
                        if start < end {
                            let x1 = rect.left()
                                + gutter_width
                                + (start - line_start) as f32 * char_width;
                            let x2 =
                                rect.left() + gutter_width + (end - line_start) as f32 * char_width;
                            ui.painter().rect_filled(
                                egui::Rect::from_min_max(
                                    egui::pos2(x1, y),
                                    egui::pos2(x2, y + row_height),
                                ),
                                0.0,
                                Color32::from_rgba_unmultiplied(65, 105, 160, 120),
                            );
                        }
                    }
                    let line_start_byte = document.text.line_to_byte(line).unwrap_or(0);
                    let job =
                        Self::highlighted_line_job(content, line_start_byte, &document.analysis);
                    let galley = ui.fonts_mut(|fonts| fonts.layout_job(job));
                    ui.painter().galley(
                        egui::pos2(rect.left() + gutter_width, y + 2.0),
                        galley,
                        Color32::WHITE,
                    );
                    if response.has_focus() {
                        for selection in document.selections.selections() {
                            if selection.head >= line_start && selection.head <= line_end {
                                let x = rect.left()
                                    + gutter_width
                                    + (selection.head - line_start) as f32 * char_width;
                                ui.painter().line_segment(
                                    [egui::pos2(x, y + 1.0), egui::pos2(x, y + row_height - 1.0)],
                                    egui::Stroke::new(1.5_f32, Color32::from_rgb(220, 225, 235)),
                                );
                            }
                        }
                    }
                }
                response
            });

        let response = output.inner;
        if !response.has_focus() {
            return Ok(false);
        }
        let events = ui.input(|input| input.events.clone());
        let initial_revision = document.text.revision();
        for event in events {
            match event {
                egui::Event::Text(text) if !text.is_empty() => {
                    document
                        .selections
                        .insert(&mut document.text, &text)
                        .map_err(|error| error.to_string())?;
                }
                egui::Event::Key {
                    key: egui::Key::Enter,
                    pressed: true,
                    ..
                } => {
                    document
                        .selections
                        .insert(&mut document.text, "\n")
                        .map_err(|error| error.to_string())?;
                }
                egui::Event::Key {
                    key: egui::Key::Tab,
                    pressed: true,
                    ..
                } => {
                    document
                        .selections
                        .insert(&mut document.text, "    ")
                        .map_err(|error| error.to_string())?;
                }
                egui::Event::Key {
                    key: egui::Key::Backspace,
                    pressed: true,
                    ..
                } => {
                    document
                        .selections
                        .backspace(&mut document.text)
                        .map_err(|error| error.to_string())?;
                }
                egui::Event::Key {
                    key: egui::Key::Delete,
                    pressed: true,
                    ..
                } => {
                    document
                        .selections
                        .delete_forward(&mut document.text)
                        .map_err(|error| error.to_string())?;
                }
                egui::Event::Key {
                    key: egui::Key::A,
                    pressed: true,
                    modifiers,
                    ..
                } if modifiers.command => {
                    document.selections.set_single(Selection {
                        anchor: 0,
                        head: document.text.len_chars(),
                    });
                }
                egui::Event::Key {
                    key,
                    pressed: true,
                    modifiers,
                    ..
                } if matches!(key, egui::Key::ArrowLeft | egui::Key::ArrowRight) => {
                    let current = document.selections.primary();
                    let head = match key {
                        egui::Key::ArrowLeft => current.head.saturating_sub(1),
                        egui::Key::ArrowRight => (current.head + 1).min(document.text.len_chars()),
                        _ => current.head,
                    };
                    let anchor = if modifiers.shift {
                        current.anchor
                    } else {
                        head
                    };
                    document.selections.set_single(Selection { anchor, head });
                }
                _ => {}
            }
        }
        Ok(document.text.revision() != initial_revision)
    }
}

impl eframe::App for IdeApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.state.poll_search();
        self.state.poll_runner();
        self.state.poll_lsp();
        let contexts: &[jetthing_keymap::KeyContext] = if self.state.active_document.is_some() {
            &[jetthing_keymap::KeyContext::Editor]
        } else {
            &[]
        };
        for id in self.keymap.process(ctx, &self.state.keymaps, contexts) {
            if self.commands.is_enabled(&id, &self.state) {
                self.dispatch(&id, Value::Null);
            }
        }
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::P)) {
            self.palette_open = true;
        }
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, egui::Key::Space)) {
            self.dispatch("language.complete", Value::Null);
        }
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape))
            && let Some(lsp) = self.state.lsp.as_mut()
        {
            lsp.clear_completions();
        }
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::S)) {
            self.dispatch("editor.save", Value::Null);
        }
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::Z)) {
            self.dispatch("editor.undo", Value::Null);
        }
        if ctx.input_mut(|i| {
            i.consume_key(
                egui::Modifiers::COMMAND | egui::Modifiers::SHIFT,
                egui::Key::Z,
            )
        }) {
            self.dispatch("editor.redo", Value::Null);
        }
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, egui::Key::Backtick)) {
            self.dispatch("terminal.toggle", Value::Null);
        }
        if ctx.input_mut(|i| {
            i.consume_key(
                egui::Modifiers::COMMAND | egui::Modifiers::SHIFT,
                egui::Key::F,
            )
        }) {
            self.state.search_visible = true;
        }
        egui::TopBottomPanel::top("toolbar")
            .exact_height(42.0)
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    ui.heading(RichText::new("Jetthing").color(Color32::from_rgb(196, 143, 255)));
                    ui.separator();
                    if ui.button("⌘P  Search commands").clicked() {
                        self.palette_open = true;
                    }
                    if ui.button("Terminal").clicked() {
                        self.dispatch("terminal.toggle", Value::Null);
                    }
                    if ui.button("Find in Files").clicked() {
                        self.state.search_visible = true;
                    }
                    if ui.button("Git").clicked() {
                        self.state.git_visible = !self.state.git_visible;
                    }
                    if ui
                        .add_enabled(!self.state.runner.is_running(), egui::Button::new("Build"))
                        .clicked()
                    {
                        self.dispatch("build.project", Value::Null);
                    }
                    if ui.button("Problems").clicked() {
                        self.state.problems_visible = !self.state.problems_visible;
                    }
                    ui.add_space(12.0);
                    let root = self
                        .state
                        .project
                        .as_ref()
                        .map(|p| p.root().display().to_string())
                        .unwrap_or_else(|| "No project".into());
                    ui.label(root);
                });
            });
        egui::TopBottomPanel::bottom("status")
            .exact_height(24.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(&self.state.status);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(format!("{} plugins", self.state.plugins.len()));
                        if let Some(document) = self
                            .state
                            .active_document
                            .and_then(|index| self.state.documents.get(index))
                        {
                            if let Some(language) = document.analysis.language {
                                ui.label(language);
                            }
                            let count = document.analysis.diagnostics.len();
                            let count = count + self.state.active_lsp_diagnostics().len();
                            ui.label(format!(
                                "{} problem{}",
                                count,
                                if count == 1 { "" } else { "s" }
                            ));
                        }
                        if let Some(lsp) = &self.state.lsp {
                            ui.label(match lsp.state() {
                                ServerState::Starting => "LSP starting",
                                ServerState::Ready => "LSP ready",
                                ServerState::Exited => "LSP exited",
                            });
                        }
                    });
                });
            });
        if self.state.search_visible {
            egui::TopBottomPanel::bottom("project_search")
                .default_height(260.0)
                .min_height(120.0)
                .resizable(true)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.strong("Find in Files");
                        let input = ui.add(
                            egui::TextEdit::singleline(&mut self.search_input)
                                .desired_width(300.0)
                                .hint_text("Search project text"),
                        );
                        let submit = ui.button("Search").clicked()
                            || (input.lost_focus()
                                && ui.input(|i| i.key_pressed(egui::Key::Enter)));
                        if submit && !self.search_input.is_empty() {
                            self.dispatch(
                                "search.project",
                                json!({"query": self.search_input.clone()}),
                            );
                        }
                        if self.state.search.is_running() && ui.button("Cancel").clicked() {
                            self.dispatch("search.cancel", Value::Null);
                        }
                        ui.label(if self.state.search.is_running() {
                            format!("{} results…", self.state.search_results.len())
                        } else {
                            format!("{} results", self.state.search_results.len())
                        });
                        if ui.small_button("×").clicked() {
                            self.state.search_visible = false;
                        }
                    });
                    ui.separator();
                    let results = self.state.search_results.clone();
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        for result in results {
                            let label = format!(
                                "{}:{}:{}  {}",
                                result.path.display(),
                                result.line,
                                result.column,
                                result.preview
                            );
                            if ui.selectable_label(false, label).clicked() {
                                self.dispatch("editor.openFile", json!({"path": result.path}));
                            }
                        }
                    });
                });
        }
        if self.state.problems_visible {
            egui::TopBottomPanel::bottom("problems")
                .default_height(210.0)
                .min_height(100.0)
                .resizable(true)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.strong("Problems");
                        if ui.small_button("×").clicked() {
                            self.state.problems_visible = false;
                        }
                    });
                    ui.separator();
                    if let Some(error) = &self.state.lsp_error {
                        ui.colored_label(Color32::from_rgb(238, 135, 135), error);
                    }
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        if let Some(document) = self
                            .state
                            .active_document
                            .and_then(|index| self.state.documents.get(index))
                        {
                            for diagnostic in &document.analysis.diagnostics {
                                ui.label(format!(
                                    "{}:{}:{}  {}",
                                    document.path.display(),
                                    diagnostic.row,
                                    diagnostic.column,
                                    diagnostic.message
                                ));
                            }
                        }
                        for diagnostic in self.state.active_lsp_diagnostics() {
                            let color = if diagnostic.severity == Some(1) {
                                Color32::from_rgb(238, 135, 135)
                            } else {
                                Color32::from_rgb(224, 190, 100)
                            };
                            ui.colored_label(
                                color,
                                format!(
                                    "{}:{}  {}",
                                    diagnostic.start_line + 1,
                                    diagnostic.start_character + 1,
                                    diagnostic.message
                                ),
                            );
                        }
                    });
                });
        }
        if self.state.build_visible {
            egui::TopBottomPanel::bottom("build_output")
                .default_height(230.0)
                .min_height(100.0)
                .resizable(true)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        let title = self
                            .state
                            .runner
                            .plan()
                            .map(|plan| plan.name.as_str())
                            .unwrap_or("Build");
                        ui.strong(title);
                        let state = match self.state.runner.state() {
                            TaskState::Idle => "idle".into(),
                            TaskState::Running => "running…".into(),
                            TaskState::Exited(code) => format!("finished ({code})"),
                            TaskState::Cancelled => "cancelled".into(),
                        };
                        ui.label(state);
                        if self.state.runner.is_running() && ui.button("Stop").clicked() {
                            self.dispatch("run.cancel", Value::Null);
                        }
                        if ui.small_button("×").clicked() {
                            self.state.build_visible = false;
                        }
                    });
                    ui.separator();
                    egui::ScrollArea::vertical()
                        .stick_to_bottom(true)
                        .show(ui, |ui| {
                            for chunk in self.state.runner.output() {
                                let color = match chunk.stream {
                                    OutputStream::Stdout => Color32::from_rgb(205, 210, 220),
                                    OutputStream::Stderr => Color32::from_rgb(238, 135, 135),
                                };
                                ui.label(RichText::new(&chunk.text).monospace().color(color));
                            }
                        });
                });
        }
        if self.state.terminal_visible {
            egui::TopBottomPanel::bottom("terminal")
                .default_height(240.0)
                .min_height(100.0)
                .resizable(true)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.strong("Terminal");
                        if ui.small_button("×").clicked() {
                            self.state.terminal_visible = false;
                        }
                        if self.state.terminal.as_ref().is_some_and(|t| t.is_exited()) {
                            ui.colored_label(Color32::from_rgb(220, 110, 110), "process exited");
                        }
                    });
                    ui.separator();
                    let mut transcript = self
                        .state
                        .terminal
                        .as_mut()
                        .map(|terminal| terminal.poll().to_owned())
                        .unwrap_or_default();
                    egui::ScrollArea::vertical()
                        .stick_to_bottom(true)
                        .max_height(ui.available_height() - 30.0)
                        .show(ui, |ui| {
                            ui.add(
                                egui::TextEdit::multiline(&mut transcript)
                                    .font(egui::TextStyle::Monospace)
                                    .desired_width(f32::INFINITY)
                                    .interactive(false),
                            );
                        });
                    let input = ui.add(
                        egui::TextEdit::singleline(&mut self.terminal_input)
                            .font(egui::TextStyle::Monospace)
                            .desired_width(f32::INFINITY)
                            .hint_text("Type a command and press Enter"),
                    );
                    if input.lost_focus()
                        && ui.input(|i| i.key_pressed(egui::Key::Enter))
                        && !self.terminal_input.is_empty()
                    {
                        let command = std::mem::take(&mut self.terminal_input) + "\r";
                        self.dispatch("terminal.sendText", json!({"text": command}));
                        input.request_focus();
                    }
                });
        }
        if self.state.git_visible {
            egui::SidePanel::right("git")
                .default_width(330.0)
                .min_width(220.0)
                .resizable(true)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.strong("Version Control");
                        if ui.button("Refresh").clicked() {
                            self.dispatch("git.refresh", Value::Null);
                        }
                        if ui.small_button("×").clicked() {
                            self.state.git_visible = false;
                        }
                    });
                    ui.separator();
                    let Some(git) = self.state.git.clone() else {
                        ui.label("The project is not a Git repository.");
                        return;
                    };
                    ui.label(format!("Branch: {}", git.branch));
                    ui.label(format!("{} changed files", git.changes.len()));
                    ui.separator();
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        for change in git.changes {
                            ui.group(|ui| {
                                ui.horizontal(|ui| {
                                    let marker = match change.kind {
                                        ChangeKind::Untracked => "?",
                                        ChangeKind::Modified => "M",
                                        ChangeKind::Added => "A",
                                        ChangeKind::Deleted => "D",
                                        ChangeKind::Renamed => "R",
                                        ChangeKind::Conflicted => "!",
                                    };
                                    if ui
                                        .selectable_label(
                                            false,
                                            format!("{marker} {}", change.path.display()),
                                        )
                                        .clicked()
                                        && change.kind != ChangeKind::Deleted
                                    {
                                        self.dispatch(
                                            "editor.openFile",
                                            json!({"path": change.path.clone()}),
                                        );
                                    }
                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            if change.unstaged && ui.small_button("Stage").clicked()
                                            {
                                                self.dispatch(
                                                    "git.stage",
                                                    json!({"path": change.path.clone()}),
                                                );
                                            }
                                            if change.staged && ui.small_button("Unstage").clicked()
                                            {
                                                self.dispatch(
                                                    "git.unstage",
                                                    json!({"path": change.path.clone()}),
                                                );
                                            }
                                        },
                                    );
                                });
                            });
                        }
                    });
                });
        }
        egui::SidePanel::left("project")
            .default_width(250.0)
            .resizable(true)
            .show(ctx, |ui| {
                ui.heading("Project");
                ui.separator();
                egui::ScrollArea::vertical().show(ui, |ui| {
                    let entries = self.state.files.clone();
                    for entry in entries {
                        let depth = entry.relative_path.components().count().saturating_sub(1);
                        ui.horizontal(|ui| {
                            ui.add_space(depth as f32 * 10.0);
                            let icon = if entry.is_dir { "▾" } else { "  " };
                            let response = ui.selectable_label(
                                false,
                                format!("{icon} {}", Self::file_name(&entry.relative_path)),
                            );
                            if response.double_clicked() && !entry.is_dir {
                                self.dispatch(
                                    "editor.openFile",
                                    json!({"path": entry.relative_path}),
                                );
                            }
                        });
                    }
                });
            });
        let mut changed_document = None;
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                for index in 0..self.state.documents.len() {
                    let doc = &self.state.documents[index];
                    let title = format!(
                        "{}{}",
                        Self::file_name(&doc.path),
                        if doc.text.is_dirty() { " •" } else { "" }
                    );
                    if ui
                        .selectable_label(self.state.active_document == Some(index), title)
                        .clicked()
                    {
                        self.state.active_document = Some(index);
                        if let Some(lsp) = self.state.lsp.as_mut() {
                            lsp.clear_completions();
                        }
                    }
                }
            });
            ui.separator();
            if let Some(index) = self.state.active_document {
                let doc = &mut self.state.documents[index];
                match Self::editor_surface(ui, doc) {
                    Ok(true) => {
                        let snapshot = doc.text.text();
                        doc.analysis = jetthing_language::analyze(&doc.path, &snapshot);
                        changed_document = Some((
                            doc.path.clone(),
                            snapshot,
                            doc.text.revision().min(i32::MAX as u64) as i32,
                        ));
                    }
                    Ok(false) => {}
                    Err(error) => self.state.status = error,
                }
            } else {
                ui.centered_and_justified(|ui| {
                    ui.label("Double-click a file in the Project tool window");
                });
            }
        });
        if let Some((path, text, version)) = changed_document {
            self.state.sync_document_lsp(&path, &text, version);
        }
        let completions = self
            .state
            .lsp
            .as_ref()
            .map(|lsp| lsp.completions().to_vec())
            .unwrap_or_default();
        if !completions.is_empty() {
            egui::Window::new("Code Completion")
                .collapsible(false)
                .resizable(false)
                .default_width(360.0)
                .show(ctx, |ui| {
                    for item in completions.iter().take(30) {
                        let label = match item.detail.as_deref() {
                            Some(detail) => format!("{}    {detail}", item.label),
                            None => item.label.clone(),
                        };
                        if ui.selectable_label(false, label).clicked() {
                            let text = item
                                .insert_text
                                .clone()
                                .unwrap_or_else(|| item.label.clone());
                            self.dispatch(
                                "editor.insertText",
                                json!({"position": self.state.active_caret().unwrap_or(0), "text": text}),
                            );
                        }
                    }
                });
        }
        if self.palette_open {
            let mut open = true;
            egui::Window::new("Search Everywhere")
                .collapsible(false)
                .resizable(false)
                .default_width(520.0)
                .open(&mut open)
                .show(ctx, |ui| {
                    let response = ui.text_edit_singleline(&mut self.palette_query);
                    response.request_focus();
                    let query = self.palette_query.to_lowercase();
                    let specs = self.commands.specs();
                    for command in specs
                        .into_iter()
                        .filter(|c| {
                            query.is_empty()
                                || c.label.to_lowercase().contains(&query)
                                || c.id.to_lowercase().contains(&query)
                        })
                        .take(12)
                    {
                        let enabled = self.commands.is_enabled(&command.id, &self.state);
                        if ui
                            .add_enabled(
                                enabled,
                                egui::Button::new(format!(
                                    "{}   {}",
                                    command.label,
                                    command.shortcut.unwrap_or_default()
                                )),
                            )
                            .clicked()
                        {
                            self.dispatch(&command.id, Value::Null);
                            self.palette_open = false;
                        }
                    }
                });
            self.palette_open &= open;
        }
    }
}
