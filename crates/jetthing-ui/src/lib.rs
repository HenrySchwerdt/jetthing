use eframe::egui::{self, Color32, RichText};
use jetthing_command::CommandRegistry;
use jetthing_core::{IdeState, built_in_commands};
use jetthing_language::{LanguageAnalysis, SyntaxKind};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

pub struct IdeApp {
    state: IdeState,
    commands: CommandRegistry<IdeState>,
    palette_open: bool,
    palette_query: String,
    terminal_input: String,
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

    fn highlighted_job(
        text: &str,
        analysis: &LanguageAnalysis,
        wrap_width: f32,
    ) -> egui::text::LayoutJob {
        let mut job = egui::text::LayoutJob::default();
        job.wrap.max_width = wrap_width;
        let base = egui::TextFormat {
            font_id: egui::FontId::monospace(14.0),
            color: Color32::from_rgb(205, 210, 220),
            ..Default::default()
        };
        let mut cursor = 0;
        for span in &analysis.spans {
            if span.start_byte < cursor
                || span.end_byte > text.len()
                || !text.is_char_boundary(span.start_byte)
                || !text.is_char_boundary(span.end_byte)
            {
                continue;
            }
            job.append(&text[cursor..span.start_byte], 0.0, base.clone());
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
                &text[span.start_byte..span.end_byte],
                0.0,
                egui::TextFormat {
                    color,
                    ..base.clone()
                },
            );
            cursor = span.end_byte;
        }
        job.append(&text[cursor..], 0.0, base);
        job
    }
}

impl eframe::App for IdeApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::P)) {
            self.palette_open = true;
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
                            ui.label(format!(
                                "{} problem{}",
                                count,
                                if count == 1 { "" } else { "s" }
                            ));
                        }
                    });
                });
            });
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
                    }
                }
            });
            ui.separator();
            if let Some(index) = self.state.active_document {
                let doc = &mut self.state.documents[index];
                let mut snapshot = doc.text.text();
                let analysis = doc.analysis.clone();
                let mut layouter =
                    |ui: &egui::Ui, buffer: &dyn egui::TextBuffer, wrap_width: f32| {
                        ui.fonts_mut(|fonts| {
                            fonts.layout_job(Self::highlighted_job(
                                buffer.as_str(),
                                &analysis,
                                wrap_width,
                            ))
                        })
                    };
                let response = egui::ScrollArea::both()
                    .show(ui, |ui| {
                        ui.add(
                            egui::TextEdit::multiline(&mut snapshot)
                                .font(egui::TextStyle::Monospace)
                                .layouter(&mut layouter)
                                .desired_width(f32::INFINITY)
                                .desired_rows(40)
                                .lock_focus(true),
                        )
                    })
                    .inner;
                if response.changed() {
                    if let Err(error) = doc.text.replace_from_snapshot(&snapshot) {
                        self.state.status = error.to_string();
                    } else {
                        doc.analysis = jetthing_language::analyze(&doc.path, &snapshot);
                    }
                }
            } else {
                ui.centered_and_justified(|ui| {
                    ui.label("Double-click a file in the Project tool window");
                });
            }
        });
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
