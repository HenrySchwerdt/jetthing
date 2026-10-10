use jetthing_keymap::{
    BindingTarget, ConflictKind, KeyContext, KeyInput, KeySequence, KeymapFile, KeystrokeMatcher,
    MACOS, Platform, WINDOWS_LINUX, builtin_keymaps,
};

fn keys(text: &str, platform: Platform) -> KeySequence {
    KeySequence::parse(text, platform).unwrap()
}

#[test]
fn built_in_keymaps_parse_without_diagnostics_or_ambiguity() {
    let set = builtin_keymaps().unwrap();
    for name in [WINDOWS_LINUX, MACOS] {
        let (keymap, diagnostics) = set.resolve(name).unwrap();
        assert!(diagnostics.is_empty(), "{name}: {diagnostics:?}");
        assert!(keymap.bindings().len() > 200, "{name}");
        let ambiguous: Vec<_> = keymap
            .conflicts()
            .into_iter()
            .filter(|c| c.kind == ConflictKind::Ambiguous)
            .collect();
        assert!(ambiguous.is_empty(), "{name}: {ambiguous:#?}");
    }
}

#[test]
fn every_catalog_action_is_bound_in_both_default_keymaps() {
    let set = builtin_keymaps().unwrap();
    let (win, _) = set.resolve(WINDOWS_LINUX).unwrap();
    let (mac, _) = set.resolve(MACOS).unwrap();
    for action in set.catalog().iter() {
        // The public macOS page lists "Next Occurrence of the Word at Caret" as N/A.
        assert!(win.find_action(&action.id).is_some(), "{}", action.id);
        if action.id != "FindWordAtCaret" {
            assert!(mac.find_action(&action.id).is_some(), "{}", action.id);
        }
        assert!(
            action.command.is_some() != action.unbound,
            "{} needs exactly one of command/unbound",
            action.id
        );
    }
}

#[test]
fn documented_top_shortcuts_resolve_per_platform() {
    let set = builtin_keymaps().unwrap();
    let (win, _) = set.resolve(WINDOWS_LINUX).unwrap();
    let (mac, _) = set.resolve(MACOS).unwrap();
    let editor = [KeyContext::Editor];
    let cases = [
        ("SearchEverywhere", "shift shift", "shift shift"),
        ("GotoAction", "ctrl shift A", "meta shift A"),
        ("RecentFiles", "ctrl E", "meta E"),
        ("GotoClass", "ctrl N", "meta O"),
        ("GotoFile", "ctrl shift N", "meta shift O"),
        ("ReformatCode", "ctrl alt L", "meta alt L"),
        ("Run", "shift F10", "ctrl R"),
        ("Debug", "shift F9", "ctrl D"),
        ("RunAnything", "ctrl ctrl", "ctrl ctrl"),
        ("CompileDirty", "ctrl F9", "meta F9"),
        ("EditorDuplicate", "ctrl D", "meta D"),
        (
            "EditorCloneCaretAbove",
            "ctrl ctrl, ctrl UP",
            "alt alt, alt UP",
        ),
    ];
    for (action, win_keys, mac_keys) in cases {
        let target = BindingTarget::Action(action.into());
        assert_eq!(
            win.resolve(&keys(win_keys, Platform::Windows), &editor)
                .map(|b| &b.target),
            Some(&target),
            "win {action}"
        );
        assert_eq!(
            mac.resolve(&keys(mac_keys, Platform::Macos), &editor)
                .map(|b| &b.target),
            Some(&target),
            "mac {action}"
        );
    }
    // Primary-modifier notation resolves to the platform's own binding.
    assert_eq!(
        mac.resolve(&keys("mod S", Platform::Macos), &[])
            .unwrap()
            .command
            .as_deref(),
        Some("editor.saveAll")
    );
    assert_eq!(
        win.find_action("GotoClass").unwrap().keys[0].label(Platform::Windows),
        "Ctrl+N"
    );
    assert_eq!(
        mac.find_action("GotoClass").unwrap().keys[0].label(Platform::Macos),
        "⌘O"
    );
}

#[test]
fn context_specific_bindings_share_documented_keys() {
    let set = builtin_keymaps().unwrap();
    let (win, _) = set.resolve(WINDOWS_LINUX).unwrap();
    let f7 = keys("F7", Platform::Windows);
    assert_eq!(
        win.resolve(&f7, &[KeyContext::Debugger]).unwrap().target,
        BindingTarget::Action("StepInto".into())
    );
    assert_eq!(
        win.resolve(&f7, &[KeyContext::Diff]).unwrap().target,
        BindingTarget::Action("NextDiff".into())
    );
    assert!(win.resolve(&f7, &[]).is_none());
}

#[test]
fn user_override_file_inherits_from_default_and_dispatches() {
    let mut set = builtin_keymaps().unwrap();
    let user = KeymapFile::from_toml_str(
        r#"
name = "My Keymap"
parent = "Windows/Linux"

[[binding]]
action = "SaveAll"
keys = ["ctrl K, ctrl S"]

[[binding]]
action = "ActivateTerminalToolWindow"
keys = []

[[binding]]
command = "terminal.open"
keys = ["ctrl alt shift BACK_QUOTE"]
"#,
    )
    .unwrap();
    set.insert(user);
    let (keymap, diagnostics) = set.resolve("My Keymap").unwrap();
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    assert_eq!(keymap.platform, Platform::Windows);
    assert!(keymap.find_action("ActivateTerminalToolWindow").is_none());
    // The user chord now has a prefix conflict with the inherited Commit (ctrl K).
    assert!(
        keymap
            .conflicts()
            .iter()
            .any(|c| c.kind == ConflictKind::Prefix
                && c.keys == "ctrl K"
                && c.other_keys == "ctrl K, ctrl S")
    );

    let dispatchable = keymap.dispatchable();
    let mut matcher = KeystrokeMatcher::default();
    let stroke =
        |text| KeyInput::Key(jetthing_keymap::Keystroke::parse(text, Platform::Windows).unwrap());
    let first = matcher.feed(&dispatchable, stroke("ctrl K"), 0, &[]);
    assert!(first.pending && first.consumed);
    let second = matcher.feed(&dispatchable, stroke("ctrl S"), 50, &[]);
    let fired: Vec<_> = second
        .fired
        .iter()
        .map(|&i| dispatchable.binding(i).unwrap().command.clone().unwrap())
        .collect();
    assert_eq!(fired, ["editor.saveAll"]);
}
