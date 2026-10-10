//! Reference spec index: the feature ids in `docs/spec/features/*.md` and the
//! generated inventories in `docs/spec/data/*.json` (actions, tool windows,
//! settings pages), used to validate the `Spec` column of the parity tables.
//!
//! A `Spec` cell lists comma-separated references, each optionally in backticks:
//!
//! - a feature id such as `PLATFORM-EDIT-001`, `JAVA-NAV-019` or `GRADLE-080`;
//! - `action:<id>` for an action id from `docs/spec/data/actions.json`;
//! - `toolwindow:<id>` for a tool window id from `docs/spec/data/toolwindows.json`;
//! - `setting:<id>` for a settings page (configurable) id from `docs/spec/data/settings.json`.
//!
//! `—`, `-` or an empty cell means the row has no spec counterpart.

use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt, fs,
    path::Path,
};

pub const FEATURES_DIR: &str = "docs/spec/features";
pub const DATA_DIR: &str = "docs/spec/data";

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    P0,
    P1,
    P2,
}

impl Priority {
    pub const ALL: [Priority; 3] = [Priority::P0, Priority::P1, Priority::P2];

    pub fn name(self) -> &'static str {
        match self {
            Priority::P0 => "P0",
            Priority::P1 => "P1",
            Priority::P2 => "P2",
        }
    }

    /// Catalog weight for a spec-derived row: 5 = core daily workflow (P0),
    /// 3 = significant (P1), 1 = minor (P2).
    pub fn weight(self) -> u32 {
        match self {
            Priority::P0 => 5,
            Priority::P1 => 3,
            Priority::P2 => 1,
        }
    }

    /// First `P0`/`P1`/`P2` token in a priority cell; split priorities such as
    /// `P0 (default + macOS), P1 (rest)` count as their first (highest) value.
    fn parse(cell: &str) -> Option<Priority> {
        let bytes = cell.as_bytes();
        bytes.windows(2).find_map(|w| match w {
            b"P0" => Some(Priority::P0),
            b"P1" => Some(Priority::P1),
            b"P2" => Some(Priority::P2),
            _ => None,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpecFeature {
    pub id: String,
    /// Second table cell (feature name, possibly with action ids), Markdown as written.
    pub name: String,
    pub priority: Priority,
    /// Marked `(Ultimate)` or with a paid `$` marker in the spec.
    pub ultimate: bool,
    /// File name inside `docs/spec/features/`.
    pub file: String,
    pub line: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToolWindowSpec {
    pub id: String,
    pub anchor: String,
    pub secondary: bool,
}

/// Everything a `Spec` cell may reference.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SpecIndex {
    pub features: BTreeMap<String, SpecFeature>,
    pub actions: BTreeSet<String>,
    pub tool_windows: BTreeMap<String, ToolWindowSpec>,
    pub settings: BTreeSet<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum SpecRef {
    Feature(String),
    Action(String),
    ToolWindow(String),
    Setting(String),
}

impl SpecRef {
    pub fn parse(token: &str) -> SpecRef {
        let token = token.trim().trim_matches('`').trim();
        if let Some(id) = token.strip_prefix("action:") {
            SpecRef::Action(id.trim().to_string())
        } else if let Some(id) = token.strip_prefix("toolwindow:") {
            SpecRef::ToolWindow(id.trim().to_string())
        } else if let Some(id) = token.strip_prefix("setting:") {
            SpecRef::Setting(id.trim().to_string())
        } else {
            SpecRef::Feature(token.to_string())
        }
    }
}

impl fmt::Display for SpecRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SpecRef::Feature(id) => write!(f, "{id}"),
            SpecRef::Action(id) => write!(f, "action:{id}"),
            SpecRef::ToolWindow(id) => write!(f, "toolwindow:{id}"),
            SpecRef::Setting(id) => write!(f, "setting:{id}"),
        }
    }
}

/// Splits a `Spec` cell into raw reference tokens; dashes and blanks mean none.
pub fn split_refs(cell: &str) -> Vec<String> {
    cell.split(',')
        .map(|t| t.trim().trim_matches('`').trim())
        .filter(|t| !t.is_empty() && *t != "—" && *t != "-")
        .map(str::to_string)
        .collect()
}

impl SpecIndex {
    /// Explains why a reference does not resolve, or `None` when it does.
    pub fn check(&self, reference: &SpecRef) -> Option<String> {
        let (known, kind, source) = match reference {
            SpecRef::Feature(id) => (
                self.features.contains_key(id),
                "spec feature id",
                "docs/spec/features/*.md",
            ),
            SpecRef::Action(id) => (
                self.actions.contains(id),
                "action id",
                "docs/spec/data/actions.json",
            ),
            SpecRef::ToolWindow(id) => (
                self.tool_windows.contains_key(id),
                "tool window id",
                "docs/spec/data/toolwindows.json",
            ),
            SpecRef::Setting(id) => (
                self.settings.contains(id),
                "settings page id",
                "docs/spec/data/settings.json",
            ),
        };
        (!known).then(|| format!("unknown {kind} `{reference}` (not in {source})"))
    }

    /// References that every parity inventory must cover: all feature ids and
    /// all tool windows.
    pub fn required(&self) -> impl Iterator<Item = SpecRef> + '_ {
        self.features
            .keys()
            .map(|id| SpecRef::Feature(id.clone()))
            .chain(
                self.tool_windows
                    .keys()
                    .map(|id| SpecRef::ToolWindow(id.clone())),
            )
    }
}

fn cells(line: &str) -> Vec<String> {
    crate::parity::split_cells(line)
}

fn is_separator(line: &str) -> bool {
    crate::parity::is_separator(line)
}

/// The spec marks paid features with `(Ultimate)` or a `$` token; `$?`, `$,`
/// (a paid sub-tab of a free feature) and `$ for …` are not whole-feature markers.
fn is_ultimate(row: &str) -> bool {
    if row.contains("(Ultimate)") {
        return true;
    }
    row.match_indices("`$`").any(|(at, m)| {
        let rest = row[at + m.len()..].trim_start_matches(' ');
        !(rest.starts_with('?') || rest.starts_with(',') || rest.starts_with("for "))
    })
}

/// Parses the feature tables of one spec document: tables whose first header
/// cell is `ID` and that have a `Pri` column.
pub fn parse_features(file: &str, text: &str) -> Result<Vec<SpecFeature>, String> {
    let lines: Vec<&str> = text.lines().collect();
    let mut out = Vec::new();
    let mut header: Option<(usize, usize)> = None; // (column count, Pri index)
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];
        if !line.trim_start().starts_with('|') {
            header = None;
            i += 1;
            continue;
        }
        if header.is_none() {
            let head = cells(line);
            if head.first().is_some_and(|h| h == "ID")
                && lines.get(i + 1).is_some_and(|next| is_separator(next))
                && let Some(pri) = head.iter().position(|h| h == "Pri")
            {
                header = Some((head.len(), pri));
                i += 2;
                continue;
            }
            // Not a feature table: skip it entirely.
            while i < lines.len() && lines[i].trim_start().starts_with('|') {
                i += 1;
            }
            continue;
        }
        let Some((count, pri)) = header else {
            i += 1;
            continue;
        };
        let row = cells(line);
        let line_no = i + 1;
        if row.len() != count {
            return Err(format!(
                "{file}:{line_no}: row has {} cells but the table header has {count}",
                row.len()
            ));
        }
        let id = row[0].trim_matches('`').to_string();
        let priority = Priority::parse(&row[pri])
            .ok_or_else(|| format!("{file}:{line_no}: {id}: no P0/P1/P2 priority"))?;
        out.push(SpecFeature {
            id,
            name: row.get(1).cloned().unwrap_or_default(),
            priority,
            ultimate: is_ultimate(line),
            file: file.to_string(),
            line: line_no,
        });
        i += 1;
    }
    Ok(out)
}

fn read_json(path: &Path) -> Result<Value, String> {
    let text = fs::read_to_string(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    serde_json::from_str(&text).map_err(|e| format!("parse {}: {e}", path.display()))
}

fn ids<'a>(value: &'a Value, key: &str) -> impl Iterator<Item = &'a Value> {
    value
        .get(key)
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
}

/// Loads the spec under `root`. Returns `Ok(None)` when the repository has no
/// spec (for example in unit-test fixtures), so validation is skipped.
pub fn load(root: &Path) -> Result<Option<SpecIndex>, String> {
    let dir = root.join(FEATURES_DIR);
    if !dir.is_dir() {
        return Ok(None);
    }
    let mut files: Vec<_> = fs::read_dir(&dir)
        .map_err(|e| format!("read {}: {e}", dir.display()))?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|e| e == "md"))
        .collect();
    files.sort();
    let mut index = SpecIndex::default();
    for path in files {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let text =
            fs::read_to_string(&path).map_err(|e| format!("read {}: {e}", path.display()))?;
        for feature in parse_features(&name, &text)? {
            if let Some(first) = index.features.get(&feature.id) {
                return Err(format!(
                    "{name}:{}: duplicate spec feature id {} (first defined at {}:{})",
                    feature.line, feature.id, first.file, first.line
                ));
            }
            index.features.insert(feature.id.clone(), feature);
        }
    }
    let data = root.join(DATA_DIR);
    if data.is_dir() {
        let actions = read_json(&data.join("actions.json"))?;
        index.actions = ids(&actions, "actions")
            .filter_map(|a| a.get("id").and_then(Value::as_str).map(str::to_string))
            .collect();
        let windows = read_json(&data.join("toolwindows.json"))?;
        for w in ids(&windows, "toolWindows") {
            if let Some(id) = w.get("id").and_then(Value::as_str) {
                index.tool_windows.insert(
                    id.to_string(),
                    ToolWindowSpec {
                        id: id.to_string(),
                        anchor: w
                            .get("anchor")
                            .and_then(Value::as_str)
                            .unwrap_or("left")
                            .to_string(),
                        secondary: w.get("secondary").and_then(Value::as_bool).unwrap_or(false),
                    },
                );
            }
        }
        let settings = read_json(&data.join("settings.json"))?;
        index.settings = ids(&settings, "configurables")
            .filter_map(|c| c.get("id").and_then(Value::as_str).map(str::to_string))
            .collect();
    }
    Ok(Some(index))
}

#[cfg(test)]
mod tests {
    use super::*;

    const FEATURES: &str = "# Spec\n\n| Area code | Section |\n|---|---|\n| EDIT | Editor |\n\n\
| ID | Feature (action id) | Behaviour | Pri |\n|---|---|---|---|\n\
| PLATFORM-EDIT-001 | Caret (`EditorLeft`) | moves \\| jumps | P0 |\n\
| PLATFORM-EDIT-002 | Split | x | P0 (default), P1 (rest) |\n\
| PLATFORM-TOOL-010 | Database tool window (Ultimate) | x | P2 |\n\
| JAVA-PROF-001 | Profiler | runs; `$` (inferred) | P2 |\n\
| JAVA-INSP-014 | Problems | tab (`$`, inferred) | P0 |\n\n\
| ID | Abbrev | Pri |\n|---|---|---|\n| JAVA-LT-001 | `psvm` | P1 |\n";

    #[test]
    fn parses_feature_tables_priorities_and_paid_markers() {
        let features = parse_features("platform.md", FEATURES).unwrap();
        let ids: Vec<_> = features.iter().map(|f| f.id.as_str()).collect();
        assert_eq!(
            ids,
            [
                "PLATFORM-EDIT-001",
                "PLATFORM-EDIT-002",
                "PLATFORM-TOOL-010",
                "JAVA-PROF-001",
                "JAVA-INSP-014",
                "JAVA-LT-001"
            ]
        );
        assert_eq!(features[0].name, "Caret (`EditorLeft`)");
        assert_eq!(features[1].priority, Priority::P0);
        assert_eq!(features[5].priority, Priority::P1);
        let paid: Vec<bool> = features.iter().map(|f| f.ultimate).collect();
        assert_eq!(paid, [false, false, true, true, false, false]);
        assert_eq!(features[0].line, 9);
    }

    #[test]
    fn rejects_rows_without_priority() {
        let text = "| ID | Feature | Pri |\n|---|---|---|\n| X-1 | f | soon |\n";
        let e = parse_features("x.md", text).unwrap_err();
        assert!(e.starts_with("x.md:3: X-1: no P0/P1/P2"), "{e}");
    }

    #[test]
    fn refs_parse_by_prefix_and_resolve_against_the_index() {
        assert_eq!(
            split_refs("`PLATFORM-EDIT-001`, `toolwindow:Problems View`, setting:vcs.Git"),
            [
                "PLATFORM-EDIT-001",
                "toolwindow:Problems View",
                "setting:vcs.Git"
            ]
        );
        assert!(split_refs(" — ").is_empty());
        let mut index = SpecIndex::default();
        for f in parse_features("platform.md", FEATURES).unwrap() {
            index.features.insert(f.id.clone(), f);
        }
        index.actions.insert("GotoAction".into());
        index.settings.insert("vcs.Git".into());
        index.tool_windows.insert(
            "Problems View".into(),
            ToolWindowSpec {
                id: "Problems View".into(),
                anchor: "bottom".into(),
                secondary: false,
            },
        );
        for ok in [
            "PLATFORM-EDIT-001",
            "action:GotoAction",
            "toolwindow:Problems View",
            "setting:vcs.Git",
        ] {
            assert_eq!(index.check(&SpecRef::parse(ok)), None, "{ok}");
        }
        let e = index.check(&SpecRef::parse("PLATFORM-EDIT-999")).unwrap();
        assert!(
            e.contains("unknown spec feature id `PLATFORM-EDIT-999`"),
            "{e}"
        );
        let e = index.check(&SpecRef::parse("action:Nope")).unwrap();
        assert!(e.contains("actions.json"), "{e}");
        assert_eq!(index.required().count(), 7);
    }

    #[test]
    fn repository_spec_loads_with_expected_counts() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
        let index = load(&root).unwrap().expect("repository has a spec");
        // Totals recorded in the spec's summary sections.
        let platform = index
            .features
            .keys()
            .filter(|id| id.starts_with("PLATFORM-"))
            .count();
        assert_eq!(platform, 385);
        assert_eq!(index.features.len() - platform, 682);
        let p0 = index
            .features
            .values()
            .filter(|f| f.priority == Priority::P0)
            .count();
        assert_eq!(p0, 167 + 208);
        assert!(index.actions.contains("GotoAction"));
        assert!(index.tool_windows.contains_key("Project"));
        assert!(index.settings.contains("preferences.keymap"));
    }
}
