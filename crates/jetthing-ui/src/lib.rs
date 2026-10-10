use eframe::egui::{self, Color32, RichText};
use jetthing_command::CommandRegistry;
use jetthing_core::{IdeState, built_in_commands};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

pub struct IdeApp {
    state: IdeState,
    commands: CommandRegistry<IdeState>,
    palette_open: bool,
    palette_query: String,
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
}

impl eframe::App for IdeApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::P)) {
            self.palette_open = true;
        }
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::S)) {
            self.dispatch("editor.save", Value::Null);
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
                    });
                });
            });
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
                        if doc.dirty { " •" } else { "" }
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
                let response = egui::ScrollArea::both()
                    .show(ui, |ui| {
                        ui.add(
                            egui::TextEdit::multiline(&mut doc.text)
                                .font(egui::TextStyle::Monospace)
                                .desired_width(f32::INFINITY)
                                .desired_rows(40)
                                .lock_focus(true),
                        )
                    })
                    .inner;
                if response.changed() {
                    doc.dirty = true;
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
