//! Native workbench shell.
//!
//! Layout follows the observable structure of the New UI "Islands" workbench:
//! a merged main toolbar, a tool window stripe, rounded editor/tool window
//! islands and a status bar with navigation breadcrumbs. Every user action is
//! dispatched through the typed command registry.

mod about;
mod android_run_ui;
mod android_sdk;
mod android_view;
mod blame_gutter;
mod branches_popup;
mod build_variants_ui;
mod build_window;
mod change_popup;
mod chrome;
mod completion_ui;
mod context_menus;
mod control;
mod crash_notice;
mod device_manager;
mod diff_ui;
mod disk_banner;
mod editor;
mod editor_keys;
mod findbar;
mod git_log_ui;
mod icons;
mod keybindings;
mod keys;
mod logcat_ui;
mod lsp_status;
mod main_menu;
mod merged_manifest_ui;
mod notifications;
mod palette;
mod plugin_ui;
mod popups;
mod problems_ui;
mod push_dialog;
mod settings_dialog;
mod settings_keymap;
mod settings_plugins;
mod syntax;
mod terminal_ui;
mod theme;
mod theme_engine;
mod toolbar_gradient;
mod toolchain_status;
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

pub use jetthing_core::{build_info, diagnostics};
pub use jetthing_plugins::PluginFactory;
pub use theme::install as install_theme;

pub struct IdeApp {
    state: IdeState,
    commands: CommandRegistry<IdeState>,
    palette: palette::Palette,
    terminal_selection: Option<(terminal_ui::GridPoint, terminal_ui::GridPoint)>,
    search_input: String,
    /// Find in Files option toggles and file mask.
    find_options: jetthing_core::SearchOptions,
    tree_selection: Option<PathBuf>,
    /// Active editor file last revealed in the Project tree.
    tree_revealed: Option<PathBuf>,
    caret_screen_pos: Option<egui::Pos2>,
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
    /// First stroke of a two-stroke chord waiting for its second stroke.
    pending_chord: Option<jetthing_keymap::KeyStroke>,
    /// The main menu titles are expanded in the toolbar.
    menu_bar_open: bool,
    menu_bar_popup_seen: bool,
    /// UI automation socket, when `JETTHING_CONTROL_SOCKET` is set.
    control: Option<control::Control>,
    android_sdk: android_sdk::SdkPage,
    logcat: logcat_ui::LogcatView,
    android_view: android_view::AndroidViewState,
    /// Whether background services can wake this window yet.
    waker_installed: bool,
    repaint_hook_installed: bool,
    /// Contributed command ids currently registered in `commands`.
    plugin_commands: Vec<String>,
    /// Registry generation the contributed commands were synced at.
    plugin_generation: u64,
    plugins_window: Option<plugin_ui::PluginsWindow>,
    /// Amend toggle of the Commit tool window.
    commit_amend: bool,
    /// Open push dialog.
    push_dialog: Option<push_dialog::PushDialog>,
    /// Open branch name/confirmation dialog.
    git_dialog: Option<branches_popup::GitDialog>,
    /// Open gutter change-marker popup.
    change_popup: Option<change_popup::ChangePopup>,
    deploy_balloon: android_run_ui::DeployBalloon,
}

impl IdeApp {
    /// Opens `project` (if any) and then each of `files`, relative to the project root
    /// or absolute inside it.
    pub fn new(project: Option<PathBuf>, files: Vec<PathBuf>) -> Self {
        Self::with_plugins(project, files, Vec::new())
    }

    /// Like [`Self::new`], with compiled-in plugins. Plugin state is read from
    /// the per-user configuration directory.
    pub fn with_plugins(
        project: Option<PathBuf>,
        files: Vec<PathBuf>,
        bundled: Vec<PluginFactory>,
    ) -> Self {
        let check_updates = std::env::var_os("JETTHING_NO_UPDATE_CHECK").is_none();
        let mut app = Self::build(project, files, bundled, check_updates);
        if let Some(dir) = diagnostics::active_log_dir() {
            app.state.check_crash_reports(&dir);
        }
        app
    }

    #[cfg(test)]
    fn with_update_check(
        project: Option<PathBuf>,
        files: Vec<PathBuf>,
        check_updates: bool,
    ) -> Self {
        Self::build(project, files, Vec::new(), check_updates)
    }

    fn build(
        project: Option<PathBuf>,
        files: Vec<PathBuf>,
        bundled: Vec<PluginFactory>,
        check_updates: bool,
    ) -> Self {
        let commands = built_in_commands();
        let mut state = IdeState::default();
        if let Some(dir) = jetthing_settings::default_config_dir() {
            state.load_user_settings(&dir);
        }
        state.init_plugins(jetthing_plugins::PluginStore::user(), bundled);
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
        if check_updates && state.settings.bool(ids::CHECK_UPDATES).unwrap_or(true) {
            let _ = commands.execute("app.checkForUpdates", &mut state, json!({"silent": true}));
        }
        Self {
            state,
            commands,
            palette: palette::Palette::default(),
            terminal_selection: None,
            search_input: String::new(),
            find_options: jetthing_core::SearchOptions::default(),
            tree_selection: None,
            tree_revealed: None,
            caret_screen_pos: None,
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
            pending_chord: None,
            menu_bar_open: false,
            menu_bar_popup_seen: false,
            control: None,
            android_sdk: android_sdk::SdkPage::default(),
            logcat: logcat_ui::LogcatView::default(),
            android_view: android_view::AndroidViewState::default(),
            waker_installed: false,
            repaint_hook_installed: false,
            plugin_commands: Vec::new(),
            plugin_generation: u64::MAX,
            plugins_window: None,
            change_popup: None,
            git_dialog: None,
            push_dialog: None,
            deploy_balloon: android_run_ui::DeployBalloon::default(),
            commit_amend: false,
        }
    }

    /// Standalone app mode (`jetthing --app <plugin-id>`): a window showing
    /// only the plugin's app view.
    pub fn standalone(plugin: &str, bundled: Vec<PluginFactory>) -> Result<Self, String> {
        let mut app = Self::with_plugins(None, Vec::new(), bundled);
        plugin_ui::app_mode_error(&app.state.plugin_host.registry, plugin)?;
        app.state.plugin_host.app_mode = Some(plugin.to_owned());
        Ok(app)
    }

    /// Window title for standalone app mode.
    pub fn app_title(&self) -> Option<String> {
        let plugin = self.state.plugin_host.app_mode.as_deref()?;
        let entry = self.state.plugin_host.registry.get(plugin)?;
        Some(entry.manifest.app_title())
    }

    fn dispatch(&mut self, id: &str, params: Value) {
        if let Err(error) = self.commands.execute(id, &mut self.state, params) {
            diagnostics::log(diagnostics::Level::Warn, &format!("{id}: {error}"));
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
        self.tree_revealed = None;
        self.commit_overrides.clear();
        self.commit_message.clear();
        self.search_input.clear();
        self.terminal_selection = None;
        self.find = findbar::FindBar::default();
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
        let completion_open = self
            .state
            .completion
            .as_ref()
            .is_some_and(|session| session.visible());
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
        if popup_owns_keyboard(ctx) {
            return;
        }
        if completion_open
            && !self.palette.open
            && self.dialog.is_none()
            && self.editor_focused(ctx)
        {
            for (key, delta) in [
                (Key::ArrowDown, 1),
                (Key::ArrowUp, -1),
                (Key::PageDown, 10),
                (Key::PageUp, -10),
            ] {
                if consume(Modifiers::NONE, key) {
                    self.state.move_completion_selection(delta);
                }
            }
            if consume(Modifiers::NONE, Key::Enter) {
                self.dispatch("completion.accept", json!({"replace": false}));
            }
            if consume(Modifiers::NONE, Key::Tab) {
                self.dispatch("completion.accept", json!({"replace": true}));
            }
        } else if self.state.snippet.is_some()
            && !self.palette.open
            && self.dialog.is_none()
            && self.editor_focused(ctx)
        {
            if consume(Modifiers::NONE, Key::Tab) || consume(Modifiers::NONE, Key::Enter) {
                self.dispatch("snippet.next", Value::Null);
            }
            if consume(Modifiers::SHIFT, Key::Tab) {
                self.dispatch("snippet.previous", Value::Null);
            }
        }
        if self.settings_dialog.is_some() || self.state.settings_request.is_some() {
            // The dialog is modal and handles its own keys (Escape closes it).
            return;
        }
        let terminal_focused = ctx
            .memory(|memory| memory.has_focus(toolwindows::terminal_input_id()))
            && self.dialog.is_none()
            && !self.palette.open;
        if self.dispatch_keymap(ctx, terminal_focused) {
            consumed_chord.set(true);
        }
        // Jetthing additions that the reference keymaps leave unbound.
        if consume(Modifiers::COMMAND, Key::O) {
            self.open_project_dialog();
        }
        if consume(Modifiers::CTRL, Key::Backtick) {
            self.toggle_tool_window(ToolWindow::Terminal);
        }
        self.detect_double_shift(ctx);
        if !terminal_focused && consume(Modifiers::NONE, Key::Escape) {
            if egui::Popup::is_any_open(ctx) {
                // Context menus and main-menu dropdowns close first.
                egui::Popup::close_all(ctx);
            } else if self.menu_bar_open {
                self.menu_bar_open = false;
            } else if self.state.about_open {
                self.state.about_open = false;
            } else if self.dialog.is_some() {
                self.dialog = None;
            } else if self.git_dialog.is_some() {
                self.git_dialog = None;
            } else if self.push_dialog.is_some() {
                self.push_dialog = None;
            } else if self.change_popup.is_some() {
                self.change_popup = None;
            } else if self.palette.open {
                self.palette.open = false;
            } else if self.state.completion.is_some() {
                self.dispatch("completion.close", Value::Null);
            } else if self.state.snippet.is_some() {
                self.dispatch("snippet.exit", Value::Null);
            } else if self.find.open
                && (self.editor_focused(ctx) || ctx.memory(|m| m.has_focus(findbar::query_id())))
            {
                self.find.open = false;
                ctx.memory_mut(|memory| memory.request_focus(editor::editor_id()));
            } else if self.state.tool_windows.visible(Anchor::Bottom).is_some()
                && !self.palette.open
            {
                // Escape returns focus to the editor; keep the tool window open.
                ctx.memory_mut(|memory| memory.request_focus(editor::editor_id()));
            }
        }
        // Unbound Alt chords (e.g. Alt+7) must not type their digit either; the
        // terminal gets them as shell input.
        if !terminal_focused {
            ctx.input_mut(|input| keys::strip_alt_chord_text(&mut input.events));
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
                self.run_double_modifier(jetthing_keymap::Modifier::Shift, ctx);
                self.last_shift_release = None;
            } else {
                self.last_shift_release = Some(now);
            }
        }
    }
}

/// An open menu (main menu, context menu, toolbar widget popup) owns the
/// keyboard, as in the reference IDE: Escape closes it, and no workbench
/// shortcut fires underneath it. Returns whether a popup was open.
fn popup_owns_keyboard(ctx: &egui::Context) -> bool {
    if !egui::Popup::is_any_open(ctx) {
        return false;
    }
    let escape = ctx.input_mut(|input| input.consume_key(egui::Modifiers::NONE, egui::Key::Escape));
    if escape {
        egui::Popup::close_all(ctx);
    }
    // Chords pressed while a menu is open must not reach the workbench either.
    ctx.input_mut(|input| {
        input.events.retain(|event| {
            !matches!(
                event,
                egui::Event::Key { modifiers, pressed: true, .. }
                    if modifiers.alt || modifiers.ctrl || modifiers.command
            )
        })
    });
    true
}

impl eframe::App for IdeApp {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        theme::FRAME.to_normalized_gamma_f32()
    }

    fn raw_input_hook(&mut self, ctx: &egui::Context, raw_input: &mut egui::RawInput) {
        self.control_before_frame(ctx, raw_input);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.frame(ctx);
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.state.shutdown_plugins();
        if self.state.project.is_some() {
            self.autosave();
            self.dispatch("workspace.save", Value::Null);
        }
    }
}

impl IdeApp {
    /// Builds one workbench frame. Separate from `eframe::App::update` so a
    /// headless `egui::Context` can drive it (typing-latency bench).
    pub fn frame(&mut self, ctx: &egui::Context) {
        if !self.waker_installed {
            let ctx = ctx.clone();
            self.state.waker.set(move || ctx.request_repaint());
            self.waker_installed = true;
        }
        self.state.poll_files();
        if !self.repaint_hook_installed {
            let ctx = ctx.clone();
            self.state
                .repaint
                .set(std::sync::Arc::new(move || ctx.request_repaint()));
            self.repaint_hook_installed = true;
        }
        self.state.poll_search();
        self.state.poll_runner();
        self.state.poll_lsp();
        self.state.poll_updates();
        if self.state.poll_android() {
            ctx.request_repaint();
        }
        self.state.poll_git();
        self.state.poll_toolchains();
        if self.state.toolchains.service.is_busy() {
            ctx.request_repaint_after(Duration::from_millis(200));
        }
        self.state.poll_remote();
        if self.state.lsp.is_busy() {
            ctx.request_repaint_after(Duration::from_millis(100));
        } else if !self.state.lsp.servers().is_empty() || self.state.android.devices.is_tracking() {
            // Diagnostics, server messages and device changes arrive without user input.
            ctx.request_repaint_after(Duration::from_millis(500));
        }
        plugin_ui::sync(self, ctx);
        if let Some(plugin) = self.state.plugin_host.app_mode.clone() {
            plugin_ui::handle_keybindings(self, ctx);
            chrome::window_resize_handles(ctx);
            plugin_ui::app_window(self, ctx, &plugin);
            palette::show(self, ctx);
            plugin_ui::notifications(self, ctx);
            self.control_after_frame(ctx);
            return;
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
        if !self.window_focused && focused {
            // Pick up changes made in other applications right away.
            self.state.request_git_refresh();
        }
        self.window_focused = focused;
        self.handle_shortcuts(ctx);
        plugin_ui::handle_keybindings(self, ctx);
        self.follow_tool_window_focus(ctx);
        theme::apply_settings(ctx, &self.state.settings);

        chrome::window_resize_handles(ctx);
        chrome::main_toolbar(self, ctx);
        chrome::status_bar(self, ctx);
        if self.state.project.is_none() {
            welcome::show(self, ctx);
        } else {
            chrome::left_stripe(self, ctx);
            plugin_ui::right_side(self, ctx);
            toolwindows::left(self, ctx);
            device_manager::right(self, ctx);
            toolwindows::bottom(self, ctx);
            editor::central(self, ctx);
        }
        change_popup::show(self, ctx);
        branches_popup::show_dialog(self, ctx);
        push_dialog::show(self, ctx);
        completion_ui::show(self, ctx);
        palette::show(self, ctx);
        popups::show(self, ctx);
        settings_dialog::show(self, ctx);
        android_sdk::show(self, ctx);
        merged_manifest_ui::show(self, ctx);
        notifications::show(self, ctx);
        crash_notice::show(self, ctx);
        about::show(self, ctx);
        android_run_ui::show(self, ctx);
        lsp_status::notices(self, ctx);
        plugin_ui::plugins_window(self, ctx);
        plugin_ui::notifications(self, ctx);
        // Include operations started by controls in this frame, and keep
        // polling even when their notification balloon is hidden.
        if self.state.updates.is_busy() {
            ctx.request_repaint_after(Duration::from_millis(200));
        }
        self.control_after_frame(ctx);
    }

    /// Read-only view of the IDE state (tests, benches, automation).
    pub fn state(&self) -> &IdeState {
        &self.state
    }

    /// Gives keyboard focus to the editor surface.
    pub fn focus_editor(&self, ctx: &egui::Context) {
        ctx.memory_mut(|memory| memory.request_focus(editor::editor_id()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escape_closes_an_open_menu_and_shortcuts_do_not_run_under_it() {
        let ctx = egui::Context::default();
        let key = |key, modifiers| egui::Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers,
        };
        let menu = egui::Id::new("tree_context_menu");
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::Popup::open_id(ctx, menu)
        });
        assert!(egui::Popup::is_any_open(&ctx));
        let input = egui::RawInput {
            events: vec![
                key(egui::Key::Num6, egui::Modifiers::ALT),
                key(egui::Key::Escape, egui::Modifiers::NONE),
            ],
            ..Default::default()
        };
        let _ = ctx.run(input, |ctx| {
            assert!(popup_owns_keyboard(ctx));
            let left = ctx.input(|input| input.events.len());
            assert_eq!(left, 0, "Alt+6 and Escape were consumed by the menu");
        });
        assert!(!egui::Popup::is_any_open(&ctx));
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            assert!(!popup_owns_keyboard(ctx), "no menu, shortcuts run normally");
        });
    }

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
