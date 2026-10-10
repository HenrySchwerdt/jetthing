//! Which reference action ids Jetthing implements, and how.
//!
//! An action is either backed by a typed command of the registry
//! ([`Target::Command`], parameterless) or by a workbench intent the UI layer
//! performs itself, such as opening a dialog ([`Target::Workbench`]). Every
//! other action of the catalog is *unbound*: it stays in its menu, disabled.

/// Workbench intents: actions that open UI (dialogs, popups) rather than change
/// the engine state directly. The UI decides how and whether they are enabled.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WorkbenchAction {
    NewProject,
    NewFile,
    NewDirectory,
    OpenProject,
    FindAction,
    SearchEverywhere,
    GotoFile,
    RecentFiles,
    GotoLine,
    Find,
    Replace,
    FindNext,
    FindPrevious,
    FindInFiles,
    Paste,
    Exit,
}

/// What invoking an action does.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Target {
    /// A parameterless command id registered in the command registry.
    Command(&'static str),
    Workbench(WorkbenchAction),
}

/// One implemented action.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Binding {
    /// Reference action id (`SaveAll`, `$Undo`, …).
    pub action: &'static str,
    pub target: Target,
    /// Jetthing icon key, for toolbars and menus that show icons.
    pub icon: Option<&'static str>,
}

const fn command(action: &'static str, id: &'static str, icon: Option<&'static str>) -> Binding {
    Binding {
        action,
        target: Target::Command(id),
        icon,
    }
}

const fn workbench(action: &'static str, intent: WorkbenchAction) -> Binding {
    Binding {
        action,
        target: Target::Workbench(intent),
        icon: None,
    }
}

use WorkbenchAction as W;

/// Implemented actions. Ids that the reference IDE registers at runtime (run
/// executors, tool window activation) are listed too; [`crate::DynamicItems`]
/// places them into their dynamic groups.
pub const BINDINGS: &[Binding] = &[
    // File
    workbench("NewProject", W::NewProject),
    workbench("NewFile", W::NewFile),
    workbench("NewDir", W::NewDirectory),
    workbench("OpenFile", W::OpenProject),
    command("SaveAll", "editor.saveAll", None),
    workbench("Exit", W::Exit),
    // Edit
    command("$Undo", "editor.undo", None),
    command("$Redo", "editor.redo", None),
    command("$Cut", "editor.cut", None),
    command("$Copy", "editor.copy", None),
    workbench("$Paste", W::Paste),
    command("$SelectAll", "editor.selectAll", None),
    workbench("Find", W::Find),
    workbench("Replace", W::Replace),
    workbench("FindNext", W::FindNext),
    workbench("FindPrevious", W::FindPrevious),
    workbench("FindInPath", W::FindInFiles),
    command("EditorDuplicate", "editor.duplicate", None),
    command("EditorDeleteLine", "editor.deleteLine", None),
    command("EditorIndentSelection", "editor.indent", None),
    command("EditorUnindentSelection", "editor.unindent", None),
    command("EditorJoinLines", "editor.joinLines", None),
    command("EditorToggleCase", "editor.toggleCase", None),
    command("EditorSplitLine", "editor.splitLine", None),
    command(
        "EditorStartNewLineBefore",
        "editor.startNewLineBefore",
        None,
    ),
    command("EditorSortLines", "editor.sortLines", None),
    command("EditorReverseLines", "editor.reverseLines", None),
    command("SelectNextOccurrence", "editor.selectNextOccurrence", None),
    command("SelectAllOccurrences", "editor.selectAllOccurrences", None),
    command(
        "UnselectPreviousOccurrence",
        "editor.unselectOccurrence",
        None,
    ),
    command("EditorCloneCaretAbove", "editor.cloneCaretAbove", None),
    command("EditorCloneCaretBelow", "editor.cloneCaretBelow", None),
    command(
        "EditorAddCaretPerSelectedLine",
        "editor.addCaretsToLineEnds",
        None,
    ),
    command("EditorToggleColumnMode", "editor.toggleColumnMode", None),
    // View / Navigate
    workbench("SearchEverywhere", W::SearchEverywhere),
    workbench("GotoAction", W::FindAction),
    workbench("GotoFile", W::GotoFile),
    workbench("RecentFiles", W::RecentFiles),
    workbench("GotoLine", W::GotoLine),
    // Code
    command("CodeCompletion", "language.complete", None),
    command("CommentByLineComment", "editor.commentLine", None),
    command("MoveLineUp", "editor.moveLineUp", None),
    command("MoveLineDown", "editor.moveLineDown", None),
    // Build / Run
    command("CompileDirty", "build.project", Some("build")),
    command("Run", "run.project", Some("run")),
    command("Stop", "run.cancel", Some("stop")),
    // Git
    command(
        "CheckinProject",
        "toolwindow.activate.commit",
        Some("commit"),
    ),
    command("ChangesView.Refresh", "git.refresh", Some("refresh")),
    // Window
    command(
        "ActivateProjectToolWindow",
        "toolwindow.activate.project",
        Some("project"),
    ),
    command(
        "ActivateCommitToolWindow",
        "toolwindow.activate.commit",
        Some("commit"),
    ),
    command(
        "ActivateFindToolWindow",
        "toolwindow.activate.find",
        Some("find"),
    ),
    command(
        "ActivateBuildToolWindow",
        "toolwindow.activate.build",
        Some("build"),
    ),
    command(
        "ActivateProblemsViewToolWindow",
        "toolwindow.activate.problems",
        Some("problems"),
    ),
    command(
        "ActivateTerminalToolWindow",
        "toolwindow.activate.terminal",
        Some("terminal"),
    ),
    command("HideAllWindows", "toolwindow.hideAll", None),
    command("CloseContent", "editor.close", None),
    // Help
    command("CheckForUpdate", "app.checkForUpdates", None),
    command("About", "app.about", None),
];

/// The binding of a reference action id, if Jetthing implements it.
pub fn binding(action_id: &str) -> Option<&'static Binding> {
    BINDINGS.iter().find(|binding| binding.action == action_id)
}
