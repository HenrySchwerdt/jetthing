use super::*;
use crate::keymap::{DEFAULT_KEYMAP, MAC_KEYMAP};

const MENUS_MD: &str = include_str!("../../../docs/spec/menus.md");

/// Renders a resolved tree in the exact line format of `docs/spec/menus.md`.
fn render_like_spec(
    catalog: &Catalog,
    items: &[MenuItem],
    depth: usize,
    windows: &Keymap,
    mac: &Keymap,
    out: &mut Vec<String>,
) {
    let escape = |s: &str| s.replace('|', "\\|").replace('\n', " ");
    let indent = "  ".repeat(depth);
    for item in items {
        match item {
            MenuItem::Separator(text) => out.push(match text {
                Some(text) => format!("{indent}- --- *{text}*"),
                None => format!("{indent}- ---"),
            }),
            MenuItem::InlineGroup { .. } => {}
            MenuItem::Submenu(menu) => {
                out.push(format!(
                    "{indent}- ▸ **{}** `{}`",
                    escape(&catalog.label(&menu.id)),
                    menu.id
                ));
                render_like_spec(catalog, &menu.items, depth + 1, windows, mac, out);
            }
            MenuItem::Action(action) => {
                let sw = windows.shortcuts(&action.id).join("; ");
                let sm = mac.shortcuts(&action.id).join("; ");
                let shortcut = if sw.is_empty() && sm.is_empty() {
                    String::new()
                } else if sm.is_empty() {
                    format!(" — {sw}")
                } else {
                    format!(" — {sw} / {sm}")
                };
                out.push(format!(
                    "{indent}- {} `{}`{}",
                    escape(&catalog.label(&action.id)),
                    action.id,
                    escape(&shortcut)
                ));
            }
        }
    }
}

/// The list lines of the `docs/spec/menus.md` section for `group_id`.
fn spec_section(group_id: &str) -> Vec<String> {
    let marker = format!("(`{group_id}`)");
    let mut lines = MENUS_MD.lines();
    lines
        .by_ref()
        .find(|line| line.starts_with("## ") && line.ends_with(&marker))
        .unwrap_or_else(|| panic!("no section for {group_id}"));
    lines
        .take_while(|line| !line.starts_with("## "))
        .filter(|line| !line.trim().is_empty())
        .map(str::to_owned)
        .collect()
}

#[test]
fn bundled_catalog_loads_every_action_and_group() {
    let catalog = Catalog::bundled().unwrap();
    assert_eq!(catalog.actions().count(), 3451);
    assert_eq!(catalog.group_ids().count(), 726);
    assert_eq!(catalog.label("GotoAction"), "Find Action…");
    assert_eq!(catalog.label("ShowSettings"), "Settings…");
    assert_eq!(
        catalog.label("ActivateProblemsViewToolWindow"),
        "Problems View (tool window)"
    );
    assert_eq!(catalog.label("NoSuchAction"), "NoSuchAction");
    assert!(catalog.is_group("FileMenu"));
    assert!(!catalog.is_group("SaveAll"));
}

#[test]
fn main_menu_top_two_levels_match_spec() {
    let catalog = Catalog::bundled().unwrap();
    let menus = main_menu(&NoDynamicItems).unwrap();
    let titles: Vec<&str> = menus.iter().map(|menu| menu.text.as_str()).collect();
    assert_eq!(
        titles,
        [
            "File", "Edit", "View", "Navigate", "Code", "Refactor", "Build", "Run", "Tools", "Git",
            "Window", "Help"
        ]
    );
    // Second level: every direct row of each menu, in order, against menus.md.
    let spec = spec_section(MAIN_MENU);
    for menu in &menus {
        let header = format!("- ▸ **{}** `{}`", menu.text, menu.id);
        let start = spec.iter().position(|line| *line == header).unwrap();
        let expected: Vec<&str> = spec[start + 1..]
            .iter()
            .take_while(|line| line.starts_with("  "))
            .filter(|line| !line.starts_with("    "))
            .map(|line| line.trim_start())
            .collect();
        let actual: Vec<String> = menu
            .items
            .iter()
            .filter_map(|item| match item {
                MenuItem::Action(a) => Some(format!("{} `{}`", catalog.label(&a.id), a.id)),
                MenuItem::Submenu(s) => Some(format!("▸ **{}** `{}`", catalog.label(&s.id), s.id)),
                MenuItem::Separator(None) => Some("---".into()),
                MenuItem::Separator(Some(text)) => Some(format!("--- *{text}*")),
                MenuItem::InlineGroup { .. } => None,
            })
            .collect();
        let expected: Vec<String> = expected
            .iter()
            .map(|line| {
                let line = line.trim_start_matches("- ");
                // Drop the shortcut column; shortcuts are compared in the full test.
                match line.find("` — ") {
                    Some(end) => line[..=end].to_owned(),
                    None => line.to_owned(),
                }
            })
            .collect();
        assert_eq!(actual, expected, "second level of {}", menu.id);
    }
}

#[test]
fn every_spec_menu_section_matches_the_resolved_model() {
    let catalog = Catalog::bundled().unwrap();
    let windows = Keymap::bundled(DEFAULT_KEYMAP).unwrap();
    let mac = Keymap::bundled(MAC_KEYMAP).unwrap();
    for group in [
        MAIN_MENU,
        MAIN_TOOLBAR,
        EDITOR_POPUP,
        PROJECT_VIEW_POPUP,
        EDITOR_TAB_POPUP,
        "NavBarToolBar",
        TOOL_WINDOW_POPUP,
        "RunContextGroup",
        GENERATE,
        "RefactoringMenu",
        "Vcs.Operations.Popup",
        "WelcomeScreen.QuickStart",
    ] {
        let items = Resolver::new(catalog, &NoDynamicItems)
            .resolve(group)
            .unwrap();
        let mut actual = Vec::new();
        render_like_spec(catalog, &items, 0, &windows, &mac, &mut actual);
        let expected = spec_section(group);
        assert_eq!(actual.len(), expected.len(), "row count of {group}");
        for (index, (a, e)) in actual.iter().zip(&expected).enumerate() {
            assert_eq!(a, e, "{group} row {index}");
        }
    }
}

#[test]
fn unknown_groups_are_errors_not_panics() {
    assert_eq!(
        group("NoSuchGroup", &NoDynamicItems),
        Err(MenuError::UnknownGroup("NoSuchGroup".into()))
    );
    assert!(matches!(
        Keymap::bundled("No Such Keymap"),
        Err(MenuError::UnknownKeymap(_))
    ));
    assert!(Catalog::from_json("{").is_err());
    assert!(
        Catalog::from_json(r#"{"actions":[{"id":"a","kind":"action","text":null,"description":null,"popup":null,"useShortcutOf":null},{"id":"a","kind":"action","text":null,"description":null,"popup":null,"useShortcutOf":null}],"groups":{}}"#)
            .is_err()
    );
}

#[test]
fn cyclic_groups_are_cut() {
    let catalog = Catalog::from_json(
        r#"{"actions":[
            {"id":"Root","kind":"group","text":"Root","description":null,"popup":true,"useShortcutOf":null},
            {"id":"Loop","kind":"group","text":"Loop","description":null,"popup":true,"useShortcutOf":null},
            {"id":"Leaf","kind":"action","text":"Leaf","description":null,"popup":null,"useShortcutOf":null}
        ],"groups":{"Root":[{"ref":"Loop"}],"Loop":[{"ref":"Leaf"},{"ref":"Root"},{"ref":"Loop"}]}}"#,
    )
    .unwrap();
    let items = Resolver::new(&catalog, &NoDynamicItems)
        .resolve("Root")
        .unwrap();
    let [MenuItem::Submenu(loop_menu)] = items.as_slice() else {
        panic!("{items:?}");
    };
    assert_eq!(loop_menu.items.len(), 1);
}

struct Ctx {
    git: bool,
}

impl ActionContext for Ctx {
    fn is_enabled(&self, binding: &Binding) -> bool {
        binding.target != Target::Command("run.cancel")
    }
    fn is_visible(&self, id: &str) -> bool {
        match id {
            VCS_MENU_WITHOUT_VCS => !self.git,
            VCS_MENU_GIT => self.git,
            _ => true,
        }
    }
    fn text(&self, id: &str) -> Option<String> {
        (id == VCS_MENU && !self.git).then(|| "VCS".into())
    }
}

#[test]
fn unbound_actions_are_disabled_not_hidden() {
    let menus = main_menu(&BuiltInDynamicItems).unwrap();
    let file = &menus[0];
    let keymap = Keymap::bundled(DEFAULT_KEYMAP).unwrap();
    let rows = present(&file.items, &Ctx { git: true }, &keymap);
    let find = |text: &str| {
        rows.iter()
            .find(|row| row.text() == Some(text))
            .unwrap_or_else(|| panic!("{text} missing: {rows:?}"))
    };
    assert!(find("Save All").is_enabled());
    assert!(find("Exit").is_enabled());
    let print = find("Print…");
    assert!(!print.is_enabled());
    let Presented::Action { shortcut, .. } = find("Save All") else {
        panic!()
    };
    assert_eq!(shortcut.as_deref(), Some("Ctrl+S"));
    // File | New gets its runtime entries, Recent Projects keeps its declared rows.
    let Presented::Submenu { menu, enabled, .. } = find("New") else {
        panic!()
    };
    assert!(*enabled);
    let new_rows = present(&menu.items, &Ctx { git: true }, &keymap);
    let new: Vec<&str> = new_rows.iter().filter_map(|row| row.text()).collect();
    assert_eq!(new, ["Project…", "File", "Directory"]);
}

#[test]
fn presentation_normalizes_separators() {
    let menus = main_menu(&BuiltInDynamicItems).unwrap();
    for menu in &menus {
        for git in [true, false] {
            let rows = present(&menu.items, &Ctx { git }, &NoShortcuts);
            assert!(!matches!(rows.first(), Some(Presented::Separator(None))));
            assert!(!matches!(rows.last(), Some(Presented::Separator(_))));
            assert!(
                rows.windows(2).all(|pair| !matches!(
                    pair,
                    [Presented::Separator(_), Presented::Separator(_)]
                )),
                "{}",
                menu.id
            );
        }
    }
    // Run starts with the runtime Run/Debug executors, not with a separator.
    let run = menus.iter().find(|menu| menu.id == "RunMenu").unwrap();
    let rows = present(&run.items, &Ctx { git: true }, &NoShortcuts);
    assert_eq!(rows[0].text(), Some("Run"));
    assert!(rows[0].is_enabled());
    assert_eq!(rows[1].text(), Some("Debug"));
    assert!(!rows[1].is_enabled());
}

#[test]
fn vcs_menu_follows_the_active_vcs() {
    let menus = main_menu(&NoDynamicItems).unwrap();
    let vcs = menus.iter().find(|menu| menu.id == VCS_MENU).unwrap();
    let texts = |git: bool| -> Vec<String> {
        present(&vcs.items, &Ctx { git }, &NoShortcuts)
            .iter()
            .filter_map(|row| row.text().map(str::to_owned))
            .collect()
    };
    let with_git = texts(true);
    let without = texts(false);
    assert!(without.contains(&"Enable Version Control Integration…".to_owned()));
    assert!(!with_git.contains(&"Enable Version Control Integration…".to_owned()));
    assert!(with_git.contains(&"Push…".to_owned()), "{with_git:?}");
    assert!(!without.contains(&"Push…".to_owned()));
    let entry = [MenuItem::Submenu(vcs.clone())];
    let main = present(&entry, &Ctx { git: false }, &NoShortcuts);
    assert_eq!(main[0].text(), Some("VCS"));
}

#[test]
fn bindings_name_real_actions_once() {
    let catalog = Catalog::bundled().unwrap();
    let windows = Keymap::bundled(DEFAULT_KEYMAP).unwrap();
    let mut seen = std::collections::HashSet::new();
    for binding in BINDINGS {
        assert!(
            seen.insert(binding.action),
            "{} bound twice",
            binding.action
        );
        // Runtime-registered ids (run executors, tool window activation) are not in
        // the action XML but have default shortcuts in the keymaps.
        assert!(
            catalog.get(binding.action).is_some()
                || !windows.shortcuts(binding.action).is_empty()
                || binding.action == "ActivateBuildToolWindow",
            "{} is not a reference action id",
            binding.action
        );
    }
    // Every bound main-menu action is reachable in the resolved menu.
    let menus = main_menu(&BuiltInDynamicItems).unwrap();
    let items: Vec<MenuItem> = menus.into_iter().map(MenuItem::Submenu).collect();
    let ids: std::collections::HashSet<&str> =
        actions(&items).iter().map(|a| a.id.as_str()).collect();
    for id in [
        "SaveAll",
        "$Undo",
        "CompileDirty",
        "Run",
        "Stop",
        "About",
        "Exit",
    ] {
        assert!(ids.contains(id), "{id} not in the main menu");
    }
}

#[test]
fn shortcuts_follow_the_keymap_and_platform_notation() {
    let windows = Keymap::bundled(DEFAULT_KEYMAP).unwrap();
    let mac = Keymap::bundled(MAC_KEYMAP).unwrap();
    assert_eq!(
        windows.menu_shortcut("GotoAction").as_deref(),
        Some("Ctrl+Shift+A")
    );
    assert_eq!(mac.menu_shortcut("GotoAction").as_deref(), Some("⇧⌘A"));
    assert_eq!(mac.menu_shortcut("ShowSettings").as_deref(), Some("⌘,"));
    assert_eq!(mac.menu_shortcut("EditorDeleteLine").as_deref(), Some("⌘⌫"));
    assert_eq!(
        mac.menu_shortcut("SearchEverywhere").as_deref(),
        Some("Double ⇧")
    );
    assert_eq!(windows.menu_shortcut("Exit"), None);
    assert_eq!(mac_glyphs("Ctrl+Alt+Shift+Cmd+K, S"), "⌃⌥⇧⌘K, S");
    assert_eq!(mac_glyphs("Cmd++"), "⌘+");
    // Keymaps derived from the macOS one use glyphs too.
    assert!(Keymap::bundled("Xcode").unwrap().mac_glyphs());
    assert!(!Keymap::bundled("Emacs").unwrap().mac_glyphs());
    assert!(Keymap::bundled_names().unwrap().len() > 20);
    assert_eq!(windows.name(), DEFAULT_KEYMAP);
    assert_eq!(mac.parent(), Some(DEFAULT_KEYMAP));
}

#[test]
fn context_menus_and_toolbar_resolve() {
    for id in [
        EDITOR_POPUP,
        PROJECT_VIEW_POPUP,
        EDITOR_TAB_POPUP,
        TOOL_WINDOW_POPUP,
        MAIN_TOOLBAR,
        GENERATE,
    ] {
        let items = group(id, &BuiltInDynamicItems).unwrap();
        assert!(!actions(&items).is_empty(), "{id} is empty");
    }
    let editor = group(EDITOR_POPUP, &NoDynamicItems).unwrap();
    let ids: Vec<&str> = actions(&editor).iter().map(|a| a.id.as_str()).collect();
    for id in ["$Cut", "$Copy", "$Paste"] {
        assert!(ids.contains(&id), "{id}");
    }
    let toolbar = group(MAIN_TOOLBAR, &NoDynamicItems).unwrap();
    let search = actions(&toolbar)
        .into_iter()
        .find(|a| a.id == "SearchEverywhere")
        .unwrap();
    assert!(search.binding.is_some());
}
