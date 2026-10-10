//! nexus-http: an Insomnia-class API client for Jetthing.
//!
//! [`NexusApp`] renders the whole client into any `egui::Ui` — a plugin tool
//! window, an editor tab or the standalone app window — using the host's theme
//! colors. The engine lives in `nexus-http-core`; [`state::NexusState`] is the
//! UI-independent controller.

pub mod oauth;
pub mod state;
pub mod theme;
pub mod views;
pub mod widgets;

use egui::{
    CornerRadius, CursorIcon, Key, KeyboardShortcut, Modifiers, Rect, Sense, Ui, pos2, vec2,
};
use nexus_http_core::model::{Auth, Item};
use nexus_http_core::template::Context;
use state::NexusState;
use std::path::Path;
use theme::Palette;
use views::overlays::{Overlay, OverlayUi};

pub const PLUGIN_ID: &str = "dev.jetthing.nexus-http";

const GAP: f32 = 2.0;
const RADIUS: u8 = 10;

pub struct NexusApp {
    pub state: NexusState,
    sidebar: views::sidebar::SidebarUi,
    response: views::response::ResponseUi,
    overlays: OverlayUi,
    sidebar_width: f32,
    /// Fraction of the editor area used by the request pane.
    split: f32,
    token_job: Option<(nexus_http_core::model::Id, oauth::TokenJob)>,
}

impl NexusApp {
    /// Opens (or creates) the workspace stored under `dir/.nexus`.
    pub fn open(dir: &Path) -> Self {
        Self::with_state(NexusState::open(dir))
    }

    pub fn with_state(state: NexusState) -> Self {
        Self {
            state,
            sidebar: Default::default(),
            response: Default::default(),
            overlays: Default::default(),
            sidebar_width: 270.0,
            split: 0.5,
            token_job: None,
        }
    }

    pub fn open_overlay(&mut self, overlay: Overlay) {
        self.overlays.open(overlay);
    }

    /// Sends the selected request.
    pub fn send_selected(&mut self) {
        if let Some(id) = self.state.selected.clone() {
            self.state.send(&id);
        }
    }

    /// Background work; returns true while the UI should keep repainting.
    pub fn poll(&mut self) -> bool {
        if let Some((id, job)) = &self.token_job
            && let Some(result) = job.poll()
        {
            let id = id.clone();
            self.token_job = None;
            match result {
                Ok(token) => {
                    if let Some(Item::Request(r)) = self.state.workspace.find_item_mut(&id)
                        && let Auth::OAuth2(cfg) = &mut r.auth
                    {
                        cfg.token = Some(token);
                    } else if let Some(Item::Folder(f)) = self.state.workspace.find_item_mut(&id)
                        && let Auth::OAuth2(cfg) = &mut f.auth
                    {
                        cfg.token = Some(token);
                    }
                    self.state.touch();
                    self.state.notify("OAuth 2.0 token received");
                }
                Err(e) => self.state.notify(format!("Token request failed: {e}")),
            }
        }
        self.state.poll() || self.token_job.is_some()
    }

    fn start_token_fetch(&mut self) {
        let Some(id) = self.state.selected.clone() else {
            return;
        };
        let Some((folders, request)) = self.state.workspace.find_request(&id) else {
            return;
        };
        let Auth::OAuth2(cfg) = &request.auth else {
            return;
        };
        let ctx = Context::for_request(&self.state.workspace, &folders)
            .with_responses(&self.state.history);
        let render = |s: &str| ctx.render(s).map_err(|e| e.to_string());
        let rendered = (|| -> Result<_, String> {
            let mut c = (**cfg).clone();
            c.authorization_url = render(&c.authorization_url)?;
            c.access_token_url = render(&c.access_token_url)?;
            c.client_id = render(&c.client_id)?;
            c.client_secret = render(&c.client_secret)?;
            c.redirect_url = render(&c.redirect_url)?;
            c.scope = render(&c.scope)?;
            c.audience = render(&c.audience)?;
            c.username = render(&c.username)?;
            c.password = render(&c.password)?;
            Ok(c)
        })();
        match rendered {
            Ok(c) => {
                let job = oauth::TokenJob::start(c, self.state.workspace.settings.clone());
                self.token_job = Some((id, job));
                self.state.notify("Fetching OAuth 2.0 token…");
            }
            Err(e) => self.state.notify(format!("Cannot fetch token: {e}")),
        }
    }

    fn shortcuts(&mut self, ui: &mut Ui) {
        let cmd = Modifiers::COMMAND;
        let pressed = |ui: &mut Ui, key: Key| {
            ui.input_mut(|i| i.consume_shortcut(&KeyboardShortcut::new(cmd, key)))
        };
        if self.overlays.open != Overlay::None {
            if ui.input(|i| i.key_pressed(Key::Escape)) {
                self.overlays.open = Overlay::None;
            }
            return;
        }
        if pressed(ui, Key::Enter) {
            self.send_selected();
        }
        if pressed(ui, Key::L) {
            ui.memory_mut(|m| m.request_focus(egui::Id::new(views::request::URL_FIELD_ID)));
        }
        if pressed(ui, Key::P) {
            self.overlays.open(Overlay::Switcher);
        }
        if pressed(ui, Key::E) {
            self.overlays.open(Overlay::Environments);
        }
        if pressed(ui, Key::N) {
            let parent = self.state.selected.as_ref().and_then(|id| {
                self.state
                    .workspace
                    .find_request(id)
                    .and_then(|(f, _)| f.last().map(|f| f.id.clone()))
            });
            self.state.new_request(parent.as_ref());
            ui.memory_mut(|m| m.request_focus(egui::Id::new(views::request::URL_FIELD_ID)));
        }
        if pressed(ui, Key::S) {
            self.state.flush();
            self.state.notify("Saved");
        }
        if ui.input(|i| i.key_pressed(Key::Escape)) && self.state.is_sending() {
            self.state.cancel();
        }
    }

    /// Renders the client into `ui`, filling the available rect.
    pub fn ui(&mut self, ui: &mut Ui) {
        if self.poll() {
            ui.ctx()
                .request_repaint_after(std::time::Duration::from_millis(50));
        }
        self.shortcuts(ui);
        let pal = Palette::of(ui);
        let full = ui.available_rect_before_wrap();
        ui.painter().rect_filled(full, 0.0, pal.frame);
        let area = full.shrink(GAP);

        // Column geometry
        self.sidebar_width = self
            .sidebar_width
            .clamp(200.0, (area.width() * 0.45).max(200.0));
        let sidebar = Rect::from_min_max(
            area.min,
            pos2(area.left() + self.sidebar_width, area.bottom()),
        );
        let editor = Rect::from_min_max(pos2(sidebar.right() + GAP, area.top()), area.max);
        self.split = self.split.clamp(0.25, 0.75);
        let req_w = (editor.width() - GAP) * self.split;
        let request = Rect::from_min_max(editor.min, pos2(editor.left() + req_w, editor.bottom()));
        let response = Rect::from_min_max(pos2(request.right() + GAP, editor.top()), editor.max);

        // Splitters
        let handle = |ui: &mut Ui, x: f32, id: &str| -> f32 {
            let r = Rect::from_min_max(pos2(x - 3.0, area.top()), pos2(x + 3.0, area.bottom()));
            let resp = ui.interact(r, ui.id().with(id), Sense::drag());
            if resp.hovered() || resp.dragged() {
                ui.ctx().set_cursor_icon(CursorIcon::ResizeHorizontal);
            }
            resp.drag_delta().x
        };
        let dx = handle(ui, sidebar.right() + GAP / 2.0, "split-sidebar");
        self.sidebar_width += dx;
        let dx = handle(ui, request.right() + GAP / 2.0, "split-editor");
        if editor.width() > 0.0 {
            self.split += dx / editor.width();
        }

        let island = |ui: &mut Ui, rect: Rect, f: &mut dyn FnMut(&mut Ui)| {
            ui.painter()
                .rect_filled(rect, CornerRadius::same(RADIUS), pal.surface);
            let mut child = ui.new_child(
                egui::UiBuilder::new()
                    .max_rect(rect)
                    .layout(egui::Layout::top_down(egui::Align::Min)),
            );
            child.set_clip_rect(rect.intersect(ui.clip_rect()));
            f(&mut child);
        };

        let mut side_actions = views::sidebar::SidebarActions::default();
        island(ui, sidebar, &mut |ui| {
            views::sidebar::show(ui, &mut self.state, &mut self.sidebar, &mut side_actions)
        });

        let mut req_actions = views::request::RequestActions::default();
        let selected = self.state.selected_request().map(|r| r.id.clone());
        island(ui, request, &mut |ui| match &selected {
            Some(id) => views::request::show(ui, &mut self.state, id, &mut req_actions),
            None => welcome(ui, &pal),
        });
        island(ui, response, &mut |ui| {
            if let Some(id) = &selected {
                views::response::show(ui, &mut self.state, id, &mut self.response);
            }
            toast(ui, &mut self.state, &pal);
        });

        if side_actions.open_import {
            self.overlays.open(Overlay::Import);
        }
        if side_actions.open_environments {
            self.overlays.open(Overlay::Environments);
        }
        if side_actions.export {
            self.overlays.open(Overlay::Export);
        }
        if req_actions.send {
            self.send_selected();
        }
        if req_actions.cancel {
            self.state.cancel();
        }
        if req_actions.generate_code {
            self.overlays.open(Overlay::Code);
        }
        if req_actions.fetch_token {
            self.start_token_fetch();
        }
        views::overlays::show(ui, &mut self.state, &mut self.overlays);
    }
}

fn welcome(ui: &mut Ui, pal: &Palette) {
    widgets::empty_state(
        ui,
        "Select or create a request",
        "Ctrl+N creates a request, Ctrl+P jumps to one.",
    );
    let _ = pal;
}

fn toast(ui: &mut Ui, state: &mut NexusState, pal: &Palette) {
    let Some((message, at)) = &state.notice else {
        return;
    };
    let age = at.elapsed().as_secs_f32();
    if age > 4.0 {
        state.notice = None;
        return;
    }
    let alpha = if age > 3.5 { (4.0 - age) * 2.0 } else { 1.0 };
    let rect = ui.max_rect();
    let galley = ui.painter().layout(
        message.clone(),
        egui::FontId::proportional(12.5),
        pal.text,
        rect.width() - 80.0,
    );
    let size = galley.size() + vec2(24.0, 14.0);
    let r = Rect::from_min_size(
        pos2(rect.right() - size.x - 14.0, rect.bottom() - size.y - 14.0),
        size,
    );
    ui.painter().rect(
        r,
        CornerRadius::same(8),
        pal.raised.gamma_multiply(alpha),
        egui::Stroke::new(1.0_f32, pal.border.gamma_multiply(alpha)),
        egui::StrokeKind::Inside,
    );
    ui.painter().galley(
        r.min + vec2(12.0, 7.0),
        galley,
        pal.text.gamma_multiply(alpha),
    );
    ui.ctx().request_repaint();
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexus_http_core::model::{Body, Method, Request, Workspace};

    fn render(app: &mut NexusApp, dark: bool) {
        let ctx = egui::Context::default();
        ctx.set_visuals(if dark {
            egui::Visuals::dark()
        } else {
            egui::Visuals::light()
        });
        for _ in 0..3 {
            let input = egui::RawInput {
                screen_rect: Some(Rect::from_min_size(pos2(0.0, 0.0), vec2(1400.0, 860.0))),
                ..Default::default()
            };
            let _ = ctx.run(input, |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| app.ui(ui));
            });
        }
    }

    #[test]
    fn renders_every_tab_and_overlay_headlessly() {
        let mut ws = Workspace::new("Demo");
        ws.items.push(Item::Request(
            Request::new("Create", Method::Post, "{{ base }}/items").with_body(Body::Json {
                text: "{\"a\": 1}".into(),
            }),
        ));
        let mut app = NexusApp::with_state(NexusState::in_memory(ws));
        for tab in state::RequestTab::ALL {
            app.state.request_tab = tab;
            render(&mut app, true);
        }
        for tab in state::ResponseTab::ALL {
            app.state.response_tab = tab;
            render(&mut app, false);
        }
        for overlay in [
            Overlay::Environments,
            Overlay::Switcher,
            Overlay::Import,
            Overlay::Export,
            Overlay::Code,
        ] {
            app.open_overlay(overlay);
            render(&mut app, true);
        }
    }
}
