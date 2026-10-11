//! egui helpers for Jetthing plugin views.
//!
//! The engine-level [`jetthing_plugin_api`] stays toolkit-free; the egui host
//! renders plugin views by passing `&mut egui::Ui` inside a
//! [`ViewFrame`]. Plugins use [`ui`] to get it back, and the widgets below
//! (which read colors from the installed workbench visuals) to look native.

mod declarative;

pub use declarative::render_node;
pub use egui;
pub use jetthing_plugin_api::ViewFrame;

use egui::{Color32, RichText, Ui};

/// The egui `Ui` inside a frame rendered by the egui host, if any.
pub fn ui<'f>(frame: &'f mut ViewFrame<'_>) -> Option<&'f mut Ui> {
    frame.downcast_mut::<Ui>()
}

/// A view implemented with egui. Plugins can keep one per view and call
/// [`render`] from `Plugin::render_view`.
pub trait EguiView {
    fn ui(&mut self, ui: &mut Ui);
}

/// Renders `view` into `frame` when the host is egui. Returns whether it ran.
pub fn render(frame: &mut ViewFrame<'_>, view: &mut dyn EguiView) -> bool {
    match ui(frame) {
        Some(ui) => {
            view.ui(ui);
            true
        }
        None => false,
    }
}

/// Width for a text input: full width in vertical layouts; bounded in rows
/// so the widgets after it stay visible.
pub fn input_width(ui: &Ui, multiline: bool) -> f32 {
    if !multiline && ui.layout().is_horizontal() {
        (ui.available_width() * 0.6).clamp(120.0, 320.0)
    } else {
        f32::INFINITY
    }
}

/// Muted secondary text color of the current theme.
pub fn secondary_text(ui: &Ui) -> Color32 {
    ui.visuals().weak_text_color()
}

/// Tool window header row: bold title on the left, `actions` on the right.
pub fn header(ui: &mut Ui, title: &str, actions: impl FnOnce(&mut Ui)) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(title).strong());
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), actions);
    });
    ui.separator();
}

/// Small uppercase group label, as used for sections in tool windows.
pub fn section(ui: &mut Ui, title: &str) {
    ui.add_space(6.0);
    ui.label(
        RichText::new(title.to_uppercase())
            .small()
            .color(secondary_text(ui)),
    );
}

/// Centered placeholder for an empty view.
pub fn empty_state(ui: &mut Ui, title: &str, hint: &str) {
    ui.vertical_centered(|ui| {
        ui.add_space((ui.available_height() / 3.0).max(12.0));
        ui.label(RichText::new(title).size(15.0));
        ui.add_space(4.0);
        ui.label(RichText::new(hint).color(secondary_text(ui)));
    });
}

/// Key/value row with a dimmed key column.
pub fn property(ui: &mut Ui, key: &str, value: &str) {
    ui.horizontal(|ui| {
        ui.add_sized(
            [120.0, 18.0],
            egui::Label::new(RichText::new(key).color(secondary_text(ui))),
        );
        ui.label(value);
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Counter(u32);

    impl EguiView for Counter {
        fn ui(&mut self, ui: &mut Ui) {
            header(ui, "Counter", |_| {});
            self.0 += 1;
        }
    }

    #[test]
    fn renders_into_egui_frames_only() {
        let ctx = egui::Context::default();
        let mut counter = Counter(0);
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                assert!(render(&mut ViewFrame::new(ui), &mut counter));
            });
        });
        assert_eq!(counter.0, 1);
        let mut not_egui = 5_u8;
        assert!(!render(&mut ViewFrame::new(&mut not_egui), &mut counter));
    }
}
