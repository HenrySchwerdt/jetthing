//! Branches popup of the VCS toolbar widget (New UI "Git Branches" popup):
//! repository actions, then local and remote branches, each with a submenu
//! of branch operations. Also hosts the small branch name/confirm dialogs.

use crate::icons::{self, Icon};
use crate::{IdeApp, keys, theme};
use eframe::egui::{self, RichText, vec2};
use serde_json::{Value, json};

/// A dialog opened from the branches popup.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BranchDialog {
    /// Create a branch from `from` (current HEAD when `None`).
    NewBranch {
        from: Option<String>,
        name: String,
        checkout: bool,
        error: Option<String>,
    },
    Rename {
        branch: String,
        name: String,
        error: Option<String>,
    },
    CheckoutRevision {
        revision: String,
        error: Option<String>,
    },
    /// `git branch -d` refused an unmerged branch.
    ForceDelete { branch: String },
}

fn input_id() -> egui::Id {
    egui::Id::new("branch_dialog_input")
}

/// Contents of the VCS widget popup.
pub fn menu(app: &mut IdeApp, ui: &mut egui::Ui) {
    let Some(git) = app.state.git.as_ref() else {
        return;
    };
    let current = git.branch.clone();
    if !app.state.git_log.loaded {
        // Branch lists come from the log state; load them on first open.
        app.dispatch("git.log.refresh", Value::Null);
    }
    let branches = app.state.git_log.branches.clone();
    let pending = app.state.git_pending;
    ui.set_min_width(260.0);
    let mut action: Option<(&'static str, Value)> = None;
    let mut dialog = None;
    if let Some(operation) = pending {
        ui.label(RichText::new(format!("{} in progress", operation.label())).color(theme::WARNING));
        if ui.button(jetthing_core::abort_label(operation)).clicked() {
            action = Some(("git.abortOperation", Value::Null));
        }
        ui.separator();
    }
    let busy = app.state.remote_busy();
    if ui
        .add_enabled(
            !busy,
            egui::Button::new("Update Project…").shortcut_text(keys::display("⌘T")),
        )
        .clicked()
    {
        action = Some(("git.pull", Value::Null));
    }
    if ui
        .add(egui::Button::new("Commit…").shortcut_text(keys::display("⌘K")))
        .clicked()
    {
        action = Some(("toolwindow.activate.commit", Value::Null));
    }
    let mut push = false;
    if ui
        .add_enabled(
            !busy,
            egui::Button::new("Push…").shortcut_text(keys::display("⇧⌘K")),
        )
        .clicked()
    {
        push = true;
    }
    if ui.add_enabled(!busy, egui::Button::new("Fetch")).clicked() {
        action = Some(("git.fetch", Value::Null));
    }
    if let Some(progress) = &app.state.remote_progress {
        ui.label(RichText::new(progress).color(theme::TEXT_SECONDARY));
    }
    if ui.button("New Branch…").clicked() {
        dialog = Some(BranchDialog::NewBranch {
            from: None,
            name: String::new(),
            checkout: true,
            error: None,
        });
    }
    if ui.button("Checkout Tag or Revision…").clicked() {
        dialog = Some(BranchDialog::CheckoutRevision {
            revision: String::new(),
            error: None,
        });
    }
    if ui
        .add(egui::Button::new("Show Git Log").shortcut_text(keys::display("⌘9")))
        .clicked()
    {
        action = Some(("git.log.show", Value::Null));
    }
    ui.separator();
    section(ui, "Local");
    for branch in &branches.local {
        let is_current = branch.name == current;
        let mut label = branch.name.clone();
        if branch.ahead > 0 {
            label.push_str(&format!("  ↑{}", branch.ahead));
        }
        if branch.behind > 0 {
            label.push_str(&format!("  ↓{}", branch.behind));
        }
        let text = if is_current {
            RichText::new(label).color(theme::TEXT_BRIGHT).strong()
        } else {
            RichText::new(label)
        };
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 4.0;
            icons::show(
                ui,
                Icon::Branch,
                14.0,
                if is_current {
                    theme::WARNING
                } else {
                    theme::TEXT_SECONDARY
                },
            );
            ui.menu_button(text, |ui| {
                ui.set_min_width(240.0);
                branch_actions(ui, &branch.name, &current, false, &mut action, &mut dialog);
            });
        });
    }
    if !branches.remote.is_empty() {
        ui.separator();
        section(ui, "Remote");
        for branch in &branches.remote {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 4.0;
                icons::show(ui, Icon::Branch, 14.0, theme::TEXT_SECONDARY);
                ui.menu_button(branch.as_str(), |ui| {
                    ui.set_min_width(240.0);
                    branch_actions(ui, branch, &current, true, &mut action, &mut dialog);
                });
            });
        }
    }
    if action.is_some() || dialog.is_some() || push {
        ui.close();
    }
    if push {
        crate::push_dialog::open(app);
    }
    if let Some(dialog) = dialog {
        app.branch_dialog = Some(dialog);
        app.dialog_focus = true;
    }
    if let Some((id, params)) = action {
        run(app, id, params);
    }
}

fn section(ui: &mut egui::Ui, title: &str) {
    ui.label(
        RichText::new(title)
            .color(theme::TEXT_SECONDARY)
            .size(theme::UI_FONT_SIZE - 1.0),
    );
}

/// Items of a branch's submenu.
fn branch_actions(
    ui: &mut egui::Ui,
    branch: &str,
    current: &str,
    remote: bool,
    action: &mut Option<(&'static str, Value)>,
    dialog: &mut Option<BranchDialog>,
) {
    let is_current = branch == current;
    if !is_current && ui.button("Checkout").clicked() {
        *action = Some(("git.checkout", json!({ "branch": branch })));
    }
    if ui.button(format!("New Branch from '{branch}'…")).clicked() {
        *dialog = Some(BranchDialog::NewBranch {
            from: Some(branch.to_owned()),
            name: String::new(),
            checkout: true,
            error: None,
        });
    }
    if !is_current {
        ui.separator();
        if ui
            .button(format!("Merge '{branch}' into '{current}'"))
            .clicked()
        {
            *action = Some(("git.merge", json!({ "branch": branch })));
        }
        if ui
            .button(format!("Rebase '{current}' onto '{branch}'"))
            .clicked()
        {
            *action = Some(("git.rebase", json!({ "onto": branch })));
        }
    }
    if !remote {
        ui.separator();
        if ui.button("Rename…").clicked() {
            *dialog = Some(BranchDialog::Rename {
                branch: branch.to_owned(),
                name: branch.to_owned(),
                error: None,
            });
        }
        if !is_current && ui.button("Delete").clicked() {
            *action = Some(("git.deleteBranch", json!({ "branch": branch })));
        }
    }
}

/// Runs a branch command; an unmerged branch delete turns into a confirm.
fn run(app: &mut IdeApp, id: &str, params: Value) {
    match app.commands.execute(id, &mut app.state, params.clone()) {
        Ok(_) => {}
        Err(error) => {
            let message = error.to_string();
            if id == "git.deleteBranch" && message.contains("not fully merged") {
                app.branch_dialog = Some(BranchDialog::ForceDelete {
                    branch: params["branch"].as_str().unwrap_or_default().to_owned(),
                });
            }
            app.state.status = message;
        }
    }
}

fn frame(ctx: &egui::Context, title: &str, add: impl FnOnce(&mut egui::Ui)) {
    let screen = ctx.content_rect();
    let width = 380.0;
    egui::Area::new(egui::Id::new("branch_dialog"))
        .order(egui::Order::Foreground)
        .fixed_pos(egui::pos2(
            screen.center().x - width / 2.0,
            screen.top() + screen.height() * 0.2,
        ))
        .show(ctx, |ui| {
            egui::Frame::popup(ui.style())
                .corner_radius(10)
                .inner_margin(egui::Margin::same(12))
                .show(ui, |ui| {
                    ui.set_width(width - 24.0);
                    ui.label(
                        RichText::new(title)
                            .font(theme::semibold(theme::UI_FONT_SIZE))
                            .color(theme::TEXT_BRIGHT),
                    );
                    ui.add_space(6.0);
                    add(ui);
                });
        });
}

/// A text field that reports Enter; returns (submitted, response).
fn field(ui: &mut egui::Ui, text: &mut String, focus: bool) -> bool {
    let submit = ui.memory(|memory| memory.has_focus(input_id()))
        && ui.input(|input| input.key_pressed(egui::Key::Enter));
    let response = ui.add(
        egui::TextEdit::singleline(text)
            .id(input_id())
            .desired_width(f32::INFINITY)
            .margin(egui::Margin::symmetric(6, 5)),
    );
    if focus {
        response.request_focus();
    }
    submit
}

fn error_line(ui: &mut egui::Ui, error: &Option<String>) {
    if let Some(error) = error {
        ui.add_space(4.0);
        ui.label(RichText::new(error).color(theme::ERROR));
    }
}

fn buttons(ui: &mut egui::Ui, primary: &str) -> (bool, bool) {
    ui.add_space(8.0);
    let mut ok = false;
    let mut cancel = false;
    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
        let button = egui::Button::new(RichText::new(primary).color(egui::Color32::WHITE))
            .fill(theme::ACCENT)
            .min_size(vec2(72.0, 26.0));
        ok = ui.add(button).clicked();
        cancel = ui.button("Cancel").clicked();
    });
    (ok, cancel)
}

/// Draws the open branch dialog, if any.
pub fn show_dialog(app: &mut IdeApp, ctx: &egui::Context) {
    let Some(mut dialog) = app.branch_dialog.take() else {
        return;
    };
    let focus = std::mem::take(&mut app.dialog_focus);
    let mut keep = true;
    let mut command: Option<(&'static str, Value)> = None;
    match &mut dialog {
        BranchDialog::NewBranch {
            from,
            name,
            checkout,
            error,
        } => {
            let title = match from {
                Some(from) => format!("Create New Branch from {from}"),
                None => "Create New Branch".to_owned(),
            };
            frame(ctx, &title, |ui| {
                let submit = field(ui, name, focus);
                ui.checkbox(checkout, "Checkout branch");
                error_line(ui, error);
                let (ok, cancel) = buttons(ui, "Create");
                keep = !cancel;
                if (ok || submit) && !name.trim().is_empty() {
                    command = Some((
                        "git.newBranch",
                        json!({"name": name.trim(), "from": from, "checkout": *checkout}),
                    ));
                }
            });
        }
        BranchDialog::Rename {
            branch,
            name,
            error,
        } => {
            frame(ctx, &format!("Rename {branch}"), |ui| {
                let submit = field(ui, name, focus);
                error_line(ui, error);
                let (ok, cancel) = buttons(ui, "Rename");
                keep = !cancel;
                if (ok || submit) && !name.trim().is_empty() && name.trim() != branch {
                    command = Some((
                        "git.renameBranch",
                        json!({"branch": branch, "name": name.trim()}),
                    ));
                }
            });
        }
        BranchDialog::CheckoutRevision { revision, error } => {
            frame(ctx, "Checkout Tag or Revision", |ui| {
                let submit = field(ui, revision, focus);
                error_line(ui, error);
                let (ok, cancel) = buttons(ui, "Checkout");
                keep = !cancel;
                if (ok || submit) && !revision.trim().is_empty() {
                    command = Some(("git.checkout", json!({"branch": revision.trim()})));
                }
            });
        }
        BranchDialog::ForceDelete { branch } => {
            frame(ctx, "Delete Branch", |ui| {
                ui.label(format!(
                    "Branch {branch} is not fully merged. Its commits will only be \
                     reachable by hash. Delete it anyway?"
                ));
                let (ok, cancel) = buttons(ui, "Delete");
                keep = !cancel;
                if ok {
                    command = Some(("git.deleteBranch", json!({"branch": branch, "force": true})));
                }
            });
        }
    }
    if let Some((id, params)) = command {
        match app.commands.execute(id, &mut app.state, params) {
            Ok(_) => keep = false,
            Err(failure) => {
                let message = failure.to_string();
                match &mut dialog {
                    BranchDialog::NewBranch { error, .. }
                    | BranchDialog::Rename { error, .. }
                    | BranchDialog::CheckoutRevision { error, .. } => {
                        *error = Some(message.clone());
                    }
                    BranchDialog::ForceDelete { .. } => keep = false,
                }
                app.state.status = message;
            }
        }
    }
    if keep {
        app.branch_dialog = Some(dialog);
    }
}
