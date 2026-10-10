//! Turning a stream of key events into resolved bindings.
//!
//! The matcher is UI-toolkit independent: a front end converts its native key
//! events into [`KeyInput`] values (using [`ModifierTapDetector`] for
//! modifier-only gestures) and passes explicit millisecond timestamps, which
//! keeps all timing behaviour deterministic under test.

use crate::keymap::{KeyContext, ResolvedKeymap};
use crate::keystroke::{KeySequence, Keystroke, Modifier, Modifiers, Stroke};

/// A normalized key event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyInput {
    /// A non-modifier key was pressed.
    Key(Keystroke),
    /// A modifier went down while no other modifier was held.
    ModifierPressed(Modifier),
    /// A modifier was pressed and released on its own, with no key in between.
    ModifierTapped(Modifier),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MatcherConfig {
    /// Maximum gap between the two taps of a double-modifier gesture.
    pub double_tap_ms: u64,
    /// How long a chord prefix waits for its second stroke.
    pub chord_timeout_ms: u64,
}

impl Default for MatcherConfig {
    fn default() -> Self {
        Self {
            double_tap_ms: 400,
            chord_timeout_ms: 2_000,
        }
    }
}

/// Outcome of feeding one input.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MatchResult {
    /// Indices into [`ResolvedKeymap::bindings`] to execute, in order.
    pub fired: Vec<usize>,
    /// The front end should swallow the key event (it completed or continued
    /// a shortcut).
    pub consumed: bool,
    /// A chord prefix is waiting for its next stroke.
    pub pending: bool,
}

/// Stateful shortcut recognizer supporting two-stroke chords and
/// double-modifier gestures.
#[derive(Debug, Clone, Default)]
pub struct KeystrokeMatcher {
    config: MatcherConfig,
    pending: Vec<Stroke>,
    pending_since: u64,
    /// Binding that matches the pending prefix exactly; fired if the chord is
    /// abandoned or times out.
    fallback: Option<usize>,
    last_tap: Option<(Modifier, u64)>,
    /// Second press of a double-modifier gesture is being held.
    double_held: Option<Modifier>,
}

impl KeystrokeMatcher {
    pub fn new(config: MatcherConfig) -> Self {
        Self {
            config,
            ..Self::default()
        }
    }

    pub fn config(&self) -> MatcherConfig {
        self.config
    }

    /// The strokes of an unfinished chord (for a status-bar hint).
    pub fn pending(&self) -> &[Stroke] {
        &self.pending
    }

    pub fn reset(&mut self) {
        self.pending.clear();
        self.fallback = None;
        self.last_tap = None;
        self.double_held = None;
    }

    /// Fires a timed-out chord prefix's own binding, if any.
    pub fn tick(&mut self, now_ms: u64) -> MatchResult {
        let mut result = MatchResult::default();
        if !self.pending.is_empty()
            && now_ms.saturating_sub(self.pending_since) > self.config.chord_timeout_ms
        {
            result.fired.extend(self.fallback.take());
            self.pending.clear();
        }
        result.pending = !self.pending.is_empty();
        result
    }

    pub fn feed(
        &mut self,
        keymap: &ResolvedKeymap,
        input: KeyInput,
        now_ms: u64,
        active: &[KeyContext],
    ) -> MatchResult {
        let mut result = self.tick(now_ms);
        match input {
            KeyInput::ModifierPressed(modifier) => {
                self.double_held = self.recent_tap(modifier, now_ms).then_some(modifier);
            }
            KeyInput::ModifierTapped(modifier) => {
                self.double_held = None;
                if !self.pending.is_empty() {
                    // Modifier taps never continue a keyboard chord.
                } else if self.recent_tap(modifier, now_ms) {
                    self.last_tap = None;
                    let keys = KeySequence::single(Stroke::DoubleModifier(modifier));
                    if let Some(&index) = keymap.lookup(&keys, active).exact.first() {
                        result.fired.push(index);
                        result.consumed = true;
                    }
                } else {
                    self.last_tap = Some((modifier, now_ms));
                }
            }
            KeyInput::Key(stroke) => {
                self.last_tap = None;
                if let Some(modifier) = self.double_held.take()
                    && self.pending.is_empty()
                    && stroke.modifiers.contains(modifier)
                    && let Ok(keys) = KeySequence::new(vec![
                        Stroke::DoubleModifier(modifier),
                        Stroke::Key(stroke),
                    ])
                    && let Some(&index) = keymap.lookup(&keys, active).exact.first()
                {
                    result.fired.push(index);
                    result.consumed = true;
                } else {
                    self.feed_key(keymap, stroke, now_ms, active, &mut result);
                }
            }
        }
        result.pending = !self.pending.is_empty();
        result
    }

    fn recent_tap(&self, modifier: Modifier, now_ms: u64) -> bool {
        self.last_tap.is_some_and(|(m, at)| {
            m == modifier && now_ms.saturating_sub(at) <= self.config.double_tap_ms
        })
    }

    fn feed_key(
        &mut self,
        keymap: &ResolvedKeymap,
        stroke: Keystroke,
        now_ms: u64,
        active: &[KeyContext],
        result: &mut MatchResult,
    ) {
        let mut strokes = self.pending.clone();
        strokes.push(Stroke::Key(stroke));
        let lookup = KeySequence::new(strokes.clone())
            .map(|keys| keymap.lookup(&keys, active))
            .unwrap_or_default();
        if lookup.is_prefix {
            self.pending = strokes;
            self.pending_since = now_ms;
            self.fallback = lookup.exact.first().copied();
            result.consumed = true;
        } else if let Some(&index) = lookup.exact.first() {
            self.pending.clear();
            self.fallback = None;
            result.fired.push(index);
            result.consumed = true;
        } else if !self.pending.is_empty() {
            // The chord was abandoned: run the prefix's own binding, then
            // treat this key as the start of a new sequence.
            result.fired.extend(self.fallback.take());
            self.pending.clear();
            self.feed_key(keymap, stroke, now_ms, active, result);
        }
    }
}

/// Derives modifier-only gestures from successive modifier snapshots.
#[derive(Debug, Clone, Copy, Default)]
pub struct ModifierTapDetector {
    held: Modifiers,
    candidate: Option<Modifier>,
}

impl ModifierTapDetector {
    /// Call whenever a non-modifier key is pressed.
    pub fn note_key(&mut self) {
        self.candidate = None;
    }

    /// Call with the current modifier state; returns gestures to feed.
    pub fn update(&mut self, current: Modifiers) -> Vec<KeyInput> {
        let mut events = Vec::new();
        for modifier in Modifier::ALL {
            let was = self.held.contains(modifier);
            let now = current.contains(modifier);
            if now && !was {
                if self.held.is_empty() && current.count() == 1 {
                    self.candidate = Some(modifier);
                    events.push(KeyInput::ModifierPressed(modifier));
                } else {
                    self.candidate = None;
                }
            } else if was && !now && self.candidate == Some(modifier) {
                self.candidate = None;
                if current.is_empty() {
                    events.push(KeyInput::ModifierTapped(modifier));
                }
            }
        }
        self.held = current;
        events
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keymap::{ActionCatalog, BindingTarget, KeymapFile, KeymapSet};
    use crate::keystroke::Platform;

    fn keymap() -> ResolvedKeymap {
        let mut set = KeymapSet::new(ActionCatalog::default());
        set.insert(
            KeymapFile::from_toml_str(
                r#"
name = "Test"
platform = "linux"
[[binding]]
command = "search.everywhere"
keys = ["shift shift"]
[[binding]]
command = "run.anything"
keys = ["ctrl ctrl"]
[[binding]]
command = "caret.cloneAbove"
keys = ["ctrl ctrl, ctrl UP"]
context = "editor"
[[binding]]
command = "comment"
keys = ["ctrl K, ctrl C"]
[[binding]]
command = "commit"
keys = ["ctrl K"]
[[binding]]
command = "save"
keys = ["ctrl S"]
"#,
            )
            .unwrap(),
        );
        set.resolve("Test").unwrap().0
    }

    fn key(text: &str) -> KeyInput {
        KeyInput::Key(Keystroke::parse(text, Platform::Linux).unwrap())
    }

    fn fired(keymap: &ResolvedKeymap, result: &MatchResult) -> Vec<String> {
        result
            .fired
            .iter()
            .map(|&i| keymap.binding(i).unwrap().target.id().to_owned())
            .collect()
    }

    #[test]
    fn single_strokes_fire_immediately() {
        let keymap = keymap();
        let mut m = KeystrokeMatcher::default();
        let r = m.feed(&keymap, key("ctrl S"), 0, &[]);
        assert_eq!(fired(&keymap, &r), ["save"]);
        assert!(r.consumed && !r.pending);
        let r = m.feed(&keymap, key("ctrl Q"), 10, &[]);
        assert!(r.fired.is_empty() && !r.consumed);
    }

    #[test]
    fn two_stroke_chord_and_prefix_fallback() {
        let keymap = keymap();
        let mut m = KeystrokeMatcher::default();
        let r = m.feed(&keymap, key("ctrl K"), 0, &[]);
        assert!(r.consumed && r.pending && r.fired.is_empty());
        let r = m.feed(&keymap, key("ctrl C"), 100, &[]);
        assert_eq!(fired(&keymap, &r), ["comment"]);

        // Abandoned chord: prefix binding fires, new key is re-processed.
        m.feed(&keymap, key("ctrl K"), 1_000, &[]);
        let r = m.feed(&keymap, key("ctrl S"), 1_100, &[]);
        assert_eq!(fired(&keymap, &r), ["commit", "save"]);

        // Timed-out chord fires the prefix binding from tick().
        m.feed(&keymap, key("ctrl K"), 5_000, &[]);
        assert!(m.tick(6_000).fired.is_empty());
        let r = m.tick(7_500);
        assert_eq!(fired(&keymap, &r), ["commit"]);
        assert!(m.pending().is_empty());
    }

    #[test]
    fn double_modifier_taps() {
        let keymap = keymap();
        let mut m = KeystrokeMatcher::default();
        let tap = KeyInput::ModifierTapped(Modifier::Shift);
        assert!(m.feed(&keymap, tap, 0, &[]).fired.is_empty());
        let r = m.feed(&keymap, tap, 200, &[]);
        assert_eq!(fired(&keymap, &r), ["search.everywhere"]);
        // Too slow: two separate single taps.
        assert!(m.feed(&keymap, tap, 1_000, &[]).fired.is_empty());
        assert!(m.feed(&keymap, tap, 1_900, &[]).fired.is_empty());
        // A key between taps cancels the gesture.
        m.feed(&keymap, tap, 3_000, &[]);
        m.feed(&keymap, key("A"), 3_050, &[]);
        assert!(m.feed(&keymap, tap, 3_100, &[]).fired.is_empty());
    }

    #[test]
    fn double_modifier_hold_then_key() {
        let keymap = keymap();
        let mut m = KeystrokeMatcher::default();
        let ctrl = Modifier::Ctrl;
        m.feed(
            &keymap,
            KeyInput::ModifierPressed(ctrl),
            0,
            &[KeyContext::Editor],
        );
        m.feed(
            &keymap,
            KeyInput::ModifierTapped(ctrl),
            50,
            &[KeyContext::Editor],
        );
        m.feed(
            &keymap,
            KeyInput::ModifierPressed(ctrl),
            150,
            &[KeyContext::Editor],
        );
        let r = m.feed(&keymap, key("ctrl UP"), 300, &[KeyContext::Editor]);
        assert_eq!(fired(&keymap, &r), ["caret.cloneAbove"]);
        // Double ctrl released without a key is Run Anything.
        m.feed(&keymap, KeyInput::ModifierTapped(ctrl), 1_000, &[]);
        let r = m.feed(&keymap, KeyInput::ModifierTapped(ctrl), 1_200, &[]);
        assert_eq!(fired(&keymap, &r), ["run.anything"]);
    }

    #[test]
    fn tap_detector_reports_clean_taps_only() {
        let mut d = ModifierTapDetector::default();
        let shift = Modifiers::NONE.with(Modifier::Shift);
        assert_eq!(
            d.update(shift),
            vec![KeyInput::ModifierPressed(Modifier::Shift)]
        );
        assert_eq!(
            d.update(Modifiers::NONE),
            vec![KeyInput::ModifierTapped(Modifier::Shift)]
        );
        // Shift+A is not a tap.
        d.update(shift);
        d.note_key();
        assert!(d.update(Modifiers::NONE).is_empty());
        // Ctrl+Shift chord of modifiers is not a tap.
        d.update(shift);
        d.update(shift.with(Modifier::Ctrl));
        d.update(Modifiers::NONE.with(Modifier::Ctrl));
        assert!(d.update(Modifiers::NONE).is_empty());
    }

    #[test]
    fn contexts_gate_matching() {
        let keymap = keymap();
        let mut m = KeystrokeMatcher::default();
        let ctrl = Modifier::Ctrl;
        m.feed(&keymap, KeyInput::ModifierTapped(ctrl), 0, &[]);
        m.feed(&keymap, KeyInput::ModifierPressed(ctrl), 100, &[]);
        let r = m.feed(&keymap, key("ctrl UP"), 200, &[]);
        assert!(r.fired.is_empty(), "clone caret is editor-only");
        assert_eq!(
            keymap
                .find(&BindingTarget::Command("commit".into()))
                .unwrap()
                .keys[0]
                .canonical(),
            "ctrl K"
        );
    }
}
