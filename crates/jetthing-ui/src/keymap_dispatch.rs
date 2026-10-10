//! Routes egui key events through the active keymap to registered commands.
//!
//! Only bindings that map to a jetthing command take part; keys bound to
//! unimplemented IntelliJ actions fall through to the focused widget.

use eframe::egui;
use jetthing_core::KeymapState;
use jetthing_keymap::{
    Key, KeyContext, KeyInput, Keystroke, KeystrokeMatcher, ModifierTapDetector, Modifiers,
    ResolvedKeymap,
};

pub struct KeymapDispatcher {
    keymap: ResolvedKeymap,
    revision: Option<u64>,
    matcher: KeystrokeMatcher,
    taps: ModifierTapDetector,
}

impl Default for KeymapDispatcher {
    fn default() -> Self {
        Self {
            keymap: ResolvedKeymap::empty("none", jetthing_keymap::Platform::host()),
            revision: None,
            matcher: KeystrokeMatcher::default(),
            taps: ModifierTapDetector::default(),
        }
    }
}

impl KeymapDispatcher {
    fn sync(&mut self, keymaps: &KeymapState) {
        if self.revision != Some(keymaps.revision()) {
            self.keymap = keymaps.active().dispatchable();
            self.revision = Some(keymaps.revision());
            self.matcher.reset();
        }
    }

    /// Consumes key events that complete or continue a shortcut and returns
    /// the command IDs to dispatch, in order.
    pub fn process(
        &mut self,
        ctx: &egui::Context,
        keymaps: &KeymapState,
        contexts: &[KeyContext],
    ) -> Vec<String> {
        self.sync(keymaps);
        let mut fired = Vec::new();
        ctx.input_mut(|input| {
            let now = (input.time.max(0.0) * 1000.0) as u64;
            let modifiers = convert_modifiers(input.modifiers);
            for gesture in self.taps.update(modifiers) {
                fired.extend(
                    self.matcher
                        .feed(&self.keymap, gesture, now, contexts)
                        .fired,
                );
            }
            let keymap = &self.keymap;
            let matcher = &mut self.matcher;
            let taps = &mut self.taps;
            input.events.retain(|event| {
                let egui::Event::Key {
                    key,
                    pressed: true,
                    modifiers,
                    ..
                } = event
                else {
                    return true;
                };
                taps.note_key();
                let Some(stroke) = convert_key(*key, *modifiers) else {
                    return true;
                };
                let result = matcher.feed(keymap, KeyInput::Key(stroke), now, contexts);
                fired.extend(result.fired);
                !result.consumed
            });
            fired.extend(self.matcher.tick(now).fired);
        });
        fired
            .into_iter()
            .filter_map(|index| self.keymap.binding(index)?.command.clone())
            .collect()
    }
}

pub fn convert_modifiers(modifiers: egui::Modifiers) -> Modifiers {
    Modifiers {
        ctrl: modifiers.ctrl,
        alt: modifiers.alt,
        shift: modifiers.shift,
        meta: modifiers.mac_cmd,
    }
}

pub fn convert_key(key: egui::Key, modifiers: egui::Modifiers) -> Option<Keystroke> {
    use egui::Key as E;
    let key = match key {
        E::ArrowDown => Key::Down,
        E::ArrowLeft => Key::Left,
        E::ArrowRight => Key::Right,
        E::ArrowUp => Key::Up,
        E::Escape => Key::Escape,
        E::Tab => Key::Tab,
        E::Backspace => Key::Backspace,
        E::Enter => Key::Enter,
        E::Space => Key::Space,
        E::Insert => Key::Insert,
        E::Delete => Key::Delete,
        E::Home => Key::Home,
        E::End => Key::End,
        E::PageUp => Key::PageUp,
        E::PageDown => Key::PageDown,
        E::Comma => Key::Comma,
        E::Backslash => Key::Backslash,
        E::Slash => Key::Slash,
        E::OpenBracket => Key::OpenBracket,
        E::CloseBracket => Key::CloseBracket,
        E::Backtick => Key::BackQuote,
        E::Minus => Key::Minus,
        E::Period => Key::Period,
        E::Plus => Key::Plus,
        E::Equals => Key::Equals,
        E::Semicolon => Key::Semicolon,
        E::Quote => Key::Quote,
        E::Num0 => Key::Digit(0),
        E::Num1 => Key::Digit(1),
        E::Num2 => Key::Digit(2),
        E::Num3 => Key::Digit(3),
        E::Num4 => Key::Digit(4),
        E::Num5 => Key::Digit(5),
        E::Num6 => Key::Digit(6),
        E::Num7 => Key::Digit(7),
        E::Num8 => Key::Digit(8),
        E::Num9 => Key::Digit(9),
        // Letters and F1..F24 share their canonical names with egui.
        other => Key::parse(other.name()).filter(|k| matches!(k, Key::Letter(_) | Key::F(_)))?,
    };
    Some(Keystroke::new(convert_modifiers(modifiers), key))
}

#[cfg(test)]
mod tests {
    use super::*;
    use jetthing_keymap::Platform;

    #[test]
    fn converts_egui_keys_to_keystrokes() {
        let ctrl_shift = egui::Modifiers::CTRL | egui::Modifiers::SHIFT;
        assert_eq!(
            convert_key(egui::Key::A, ctrl_shift),
            Some(Keystroke::parse("ctrl shift A", Platform::Linux).unwrap())
        );
        assert_eq!(
            convert_key(egui::Key::F12, egui::Modifiers::ALT),
            Some(Keystroke::parse("alt F12", Platform::Linux).unwrap())
        );
        assert_eq!(
            convert_key(egui::Key::Backtick, egui::Modifiers::CTRL),
            Some(Keystroke::parse("ctrl `", Platform::Linux).unwrap())
        );
        let mac_cmd = egui::Modifiers::MAC_CMD;
        assert_eq!(
            convert_key(egui::Key::S, mac_cmd),
            Some(Keystroke::parse("meta S", Platform::Macos).unwrap())
        );
        assert_eq!(
            convert_key(egui::Key::BrowserBack, egui::Modifiers::NONE),
            None
        );
    }

    #[test]
    fn dispatches_mapped_keystrokes_and_passes_unbound_ones_through() {
        let keymaps = KeymapState::new(Platform::Linux);
        let mut dispatcher = KeymapDispatcher::default();
        let ctx = egui::Context::default();
        let key = |key, modifiers| egui::Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers,
        };
        let raw = egui::RawInput {
            events: vec![
                key(egui::Key::F9, egui::Modifiers::CTRL),
                key(egui::Key::N, egui::Modifiers::CTRL),
            ],
            ..Default::default()
        };
        let mut fired = Vec::new();
        let mut remaining = Vec::new();
        let _ = ctx.run(raw, |ctx| {
            fired = dispatcher.process(ctx, &keymaps, &[KeyContext::Editor]);
            remaining = ctx.input(|i| i.events.clone());
        });
        // Ctrl+F9 (Build Project) is mapped; Ctrl+N (Go to Class) is unbound.
        assert_eq!(fired, ["build.project"]);
        assert_eq!(remaining.len(), 1);
    }
}
