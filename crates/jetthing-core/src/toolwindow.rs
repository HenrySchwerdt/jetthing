//! Tool window placement model.
//!
//! Mirrors the observable New UI behaviour: tool windows are docked to a side
//! (anchor), and at most one tool window per anchor is visible at a time.

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Anchor {
    Left,
    Bottom,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ToolWindow {
    Project,
    Commit,
    Find,
    Build,
    Problems,
    Terminal,
    DeviceManager,
}

impl ToolWindow {
    pub const ALL: [ToolWindow; 7] = [
        ToolWindow::Project,
        ToolWindow::Commit,
        ToolWindow::Find,
        ToolWindow::Build,
        ToolWindow::Problems,
        ToolWindow::Terminal,
        ToolWindow::DeviceManager,
    ];

    pub fn id(self) -> &'static str {
        match self {
            ToolWindow::Project => "project",
            ToolWindow::Commit => "commit",
            ToolWindow::Find => "find",
            ToolWindow::Build => "build",
            ToolWindow::Problems => "problems",
            ToolWindow::Terminal => "terminal",
            ToolWindow::DeviceManager => "deviceManager",
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            ToolWindow::Project => "Project",
            ToolWindow::Commit => "Commit",
            ToolWindow::Find => "Find",
            ToolWindow::Build => "Build",
            ToolWindow::Problems => "Problems",
            ToolWindow::Terminal => "Terminal",
            ToolWindow::DeviceManager => "Device Manager",
        }
    }

    /// Default activation shortcut (macOS notation, see `docs/spec/toolwindows.md`).
    pub fn shortcut(self) -> &'static str {
        match self {
            ToolWindow::Project => "⌘1",
            ToolWindow::Commit => "⌘0",
            ToolWindow::Find => "⌘3",
            ToolWindow::Build => "⌘4",
            ToolWindow::Problems => "⌘6",
            ToolWindow::Terminal => "⌥F12",
            // Unbound by default, like the reference IDE.
            ToolWindow::DeviceManager => "",
        }
    }

    pub fn anchor(self) -> Anchor {
        match self {
            ToolWindow::Project | ToolWindow::Commit => Anchor::Left,
            ToolWindow::Find | ToolWindow::Build | ToolWindow::Problems | ToolWindow::Terminal => {
                Anchor::Bottom
            }
            ToolWindow::DeviceManager => Anchor::Right,
        }
    }

    pub fn parse(id: &str) -> Option<ToolWindow> {
        ToolWindow::ALL.into_iter().find(|window| window.id() == id)
    }
}

impl fmt::Display for ToolWindow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.title())
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ToolWindowLayout {
    left: Option<ToolWindow>,
    bottom: Option<ToolWindow>,
    right: Option<ToolWindow>,
}

impl ToolWindowLayout {
    fn slot(&mut self, anchor: Anchor) -> &mut Option<ToolWindow> {
        match anchor {
            Anchor::Left => &mut self.left,
            Anchor::Bottom => &mut self.bottom,
            Anchor::Right => &mut self.right,
        }
    }

    pub fn visible(&self, anchor: Anchor) -> Option<ToolWindow> {
        match anchor {
            Anchor::Left => self.left,
            Anchor::Bottom => self.bottom,
            Anchor::Right => self.right,
        }
    }

    pub fn is_visible(&self, window: ToolWindow) -> bool {
        self.visible(window.anchor()) == Some(window)
    }

    /// Shows `window`, replacing whatever was visible at the same anchor.
    pub fn show(&mut self, window: ToolWindow) {
        *self.slot(window.anchor()) = Some(window);
    }

    pub fn hide(&mut self, window: ToolWindow) {
        let slot = self.slot(window.anchor());
        if *slot == Some(window) {
            *slot = None;
        }
    }

    /// Toggles `window` and returns whether it is visible afterwards.
    pub fn toggle(&mut self, window: ToolWindow) -> bool {
        if self.is_visible(window) {
            self.hide(window);
            false
        } else {
            self.show(window);
            true
        }
    }

    pub fn hide_all(&mut self) {
        self.left = None;
        self.bottom = None;
        self.right = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_tool_window_per_anchor() {
        let mut layout = ToolWindowLayout::default();
        layout.show(ToolWindow::Project);
        layout.show(ToolWindow::Terminal);
        layout.show(ToolWindow::Build);
        assert!(layout.is_visible(ToolWindow::Project));
        assert!(layout.is_visible(ToolWindow::Build));
        assert!(!layout.is_visible(ToolWindow::Terminal));
        assert!(!layout.toggle(ToolWindow::Build));
        assert_eq!(layout.visible(Anchor::Bottom), None);
        assert!(layout.toggle(ToolWindow::Commit));
        assert!(!layout.is_visible(ToolWindow::Project));
        layout.show(ToolWindow::DeviceManager);
        assert_eq!(
            layout.visible(Anchor::Right),
            Some(ToolWindow::DeviceManager)
        );
        assert!(
            layout.is_visible(ToolWindow::Commit),
            "anchors are independent"
        );
        layout.hide_all();
        assert_eq!(layout.visible(Anchor::Right), None);
    }

    #[test]
    fn ids_round_trip() {
        for window in ToolWindow::ALL {
            assert_eq!(ToolWindow::parse(window.id()), Some(window));
        }
        assert_eq!(ToolWindow::parse("nope"), None);
    }
}
