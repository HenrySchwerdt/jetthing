#!/usr/bin/env python3
"""Extract a machine-readable IntelliJ IDEA spec from IntelliJ Community data files.

Clean-room note: this script only reads declarative, Apache-2.0 licensed data files
(plugin/action XML, keymap XML, message bundles, theme JSON) from a checkout of
https://github.com/JetBrains/intellij-community and writes facts derived from them
(ids, labels, shortcuts, tree structure, colors). It never reads or emits Java/Kotlin
source code. See docs/spec/data/NOTICE.md for attribution.

Usage:
    git clone --depth 1 --branch idea/262.10968.63 --filter=blob:none --no-checkout \
        https://github.com/JetBrains/intellij-community.git ic
    (cd ic && git sparse-checkout set --no-cone '*.xml' '**/messages/*.properties' \
        '/platform/platform-resources/src/themes/' '/build.txt')
    python3 -I docs/spec/tools/extract_intellij_data.py ic docs/spec
"""

from __future__ import annotations

import json
import re
import sys
import xml.etree.ElementTree as ET
from collections import OrderedDict, defaultdict
from pathlib import Path

# Directories that are not part of the IntelliJ IDEA distribution (build tooling,
# other products, internal/dev-only plugins, Classic UI).
EXCLUDED_PREFIXES = (
    ".idea", ".idea.bazel", ".ownership", "build/", "build.xml", "fleet/", "lib/",
    "libraries/", "native/", "python/", "tools/", "wintools/", "community-resources/",
    "android-customization/", "images/",
    "plugins/classic-ui/", "plugins/dev/", "plugins/devkit/", "plugins/evaluation-plugin/",
    "plugins/performanceTesting/",
)
TEST_PARTS = {"testData", "testdata", "test", "tests", "testSrc", "testResources"}

PRIMARY_KEYMAPS = OrderedDict([
    ("$default", "Windows"),
    ("Mac OS X 10.5+", "macOS"),
    ("Default for GNOME", "GNOME"),
    ("Default for KDE", "KDE"),
    ("Default for XWin", "XWin"),
])

CONFIGURABLE_GROUPS = OrderedDict([
    # groupId -> (display name, top-level order). Order matches the Settings tree of
    # IntelliJ IDEA; groups are interleaved with root configurables below.
    ("appearance", "Appearance & Behavior"),
    ("editor", "Editor"),
    ("build", "Build, Execution, Deployment"),
    ("language", "Languages & Frameworks"),
    ("tools", "Tools"),
])


def rel(root: Path, p: Path) -> str:
    return p.relative_to(root).as_posix()


def included(relpath: str) -> bool:
    if relpath.startswith(EXCLUDED_PREFIXES):
        return False
    return not (set(relpath.split("/")) & TEST_PARTS)


# ---------------------------------------------------------------- bundles

def load_properties(path: Path) -> dict[str, str]:
    out: dict[str, str] = {}
    try:
        text = path.read_text(encoding="utf-8")
    except UnicodeDecodeError:
        text = path.read_text(encoding="latin-1")
    lines = text.splitlines()
    i = 0
    while i < len(lines):
        line = lines[i].lstrip()
        i += 1
        if not line or line[0] in "#!":
            continue
        while line.endswith("\\") and not line.endswith("\\\\") and i < len(lines):
            line = line[:-1] + lines[i].lstrip()
            i += 1
        m = re.match(r"((?:\\.|[^=:\s])+)\s*[=:\s]\s*(.*)", line)
        if not m:
            continue
        key = m.group(1).replace("\\", "")
        val = m.group(2)
        val = re.sub(r"\\u([0-9a-fA-F]{4})", lambda mm: chr(int(mm.group(1), 16)), val)
        val = val.replace("\\\\", "\x01").replace("\\n", "\n").replace("\\t", "\t")
        val = val.replace("\\", "").replace("\x01", "\\")
        val = val.replace("''", "'")
        out[key] = val
    return out


class Bundles:
    def __init__(self, root: Path):
        self.by_name: dict[str, dict[str, str]] = {}
        self.global_keys: dict[str, str] = {}
        for p in sorted(root.rglob("messages/*.properties")):
            r = rel(root, p)
            if not included(r) or re.search(r"_[a-z]{2}(_[A-Z]{2})?\.properties$", p.name):
                continue
            props = load_properties(p)
            name = "messages." + p.stem
            self.by_name.setdefault(name, {}).update(props)
            for k, v in props.items():
                self.global_keys.setdefault(k, v)
        # ActionsBundle wins for action/group keys.
        for k, v in self.by_name.get("messages.ActionsBundle", {}).items():
            self.global_keys[k] = v

    def get(self, key: str, bundle: str | None = None) -> str | None:
        if bundle and bundle in self.by_name and key in self.by_name[bundle]:
            return self.by_name[bundle][key]
        return self.global_keys.get(key)


def clean_text(s: str | None) -> str | None:
    if s is None:
        return None
    s = s.replace("_", "", 1) if re.search(r"(^|[^_])_[A-Za-z0-9]", s) else s  # mnemonic marker
    s = s.replace("\\&", "\0").replace("&&", "\0").replace("&", "").replace("\0", "&")
    return s.strip()


# ---------------------------------------------------------------- XML

def parse_xml(path: Path) -> ET.Element | None:
    try:
        data = path.read_bytes()
        # Drop DOCTYPE/entity declarations that ElementTree cannot resolve.
        data = re.sub(rb"<!DOCTYPE[^>]*(\[[^\]]*\])?>", b"", data)
        data = re.sub(rb"&(?!amp;|lt;|gt;|quot;|apos;|#)[A-Za-z0-9_.]+;", b"", data)
        return ET.fromstring(data)
    except (ET.ParseError, OSError):
        return None


def local(tag: str) -> str:
    return tag.rsplit("}", 1)[-1] if isinstance(tag, str) else ""


# ---------------------------------------------------------------- actions

class Catalog:
    def __init__(self) -> None:
        self.actions: OrderedDict[str, dict] = OrderedDict()
        self.group_children: dict[str, list] = defaultdict(list)
        self.additions: list[tuple[str, dict]] = []  # (child id, add-to-group attrs)
        self.plugin_shortcuts: list[dict] = []
        self.configurables: list[dict] = []
        self.toolwindows: list[dict] = []
        self.advanced_settings: list[dict] = []

    def ingest(self, root_el: ET.Element, source: str, bundles: Bundles) -> None:
        plugin_bundle = None
        for el in root_el.iter():
            if local(el.tag) == "resource-bundle" and el.text:
                plugin_bundle = el.text.strip()
                break
        for el in root_el.iter():
            tag = local(el.tag)
            if tag == "actions":
                bundle = el.get("resource-bundle") or plugin_bundle
                for child in el:
                    self._action_like(child, None, source, bundle, bundles)
            elif tag in ("applicationConfigurable", "projectConfigurable"):
                self._configurable(el, None, tag, source, el.get("bundle") or plugin_bundle, bundles)
            elif tag == "toolWindow" and el.get("id"):
                self.toolwindows.append({
                    "id": el.get("id"),
                    "anchor": el.get("anchor", "left"),
                    "secondary": el.get("secondary") == "true",
                    "sideTool": el.get("sideTool") == "true",
                    "canCloseContents": el.get("canCloseContents") == "true",
                    "doNotActivateOnStart": el.get("doNotActivateOnStart") == "true",
                    "icon": (el.get("icon") or "").rsplit(".", 1)[-1] or None,
                    "source": source,
                })
            elif tag == "advancedSetting" and el.get("id") and \
                    el.get("id") not in {a["id"] for a in self.advanced_settings}:
                sid = el.get("id")
                bundle = el.get("bundle") or plugin_bundle
                self.advanced_settings.append({
                    "id": sid,
                    "default": el.get("default"),
                    "enum": el.get("enumClass", "").rsplit(".", 1)[-1] or None,
                    "group": bundles.get(el.get("groupKey") or "", bundle) or el.get("groupKey"),
                    "title": bundles.get(f"advanced.setting.{sid}", bundle),
                    "description": bundles.get(f"advanced.setting.{sid}.description", bundle),
                    "source": source,
                })

    def _shortcuts(self, el: ET.Element, action_id: str, source: str) -> None:
        for sc in el:
            t = local(sc.tag)
            if t == "keyboard-shortcut":
                self.plugin_shortcuts.append({
                    "action": action_id, "keymap": sc.get("keymap", "$default"), "kind": "keyboard",
                    "first": sc.get("first-keystroke"), "second": sc.get("second-keystroke"),
                    "remove": sc.get("remove") == "true", "replaceAll": sc.get("replace-all") == "true",
                    "source": source,
                })
            elif t == "mouse-shortcut":
                self.plugin_shortcuts.append({
                    "action": action_id, "keymap": sc.get("keymap", "$default"), "kind": "mouse",
                    "first": sc.get("keystroke"), "second": None,
                    "remove": sc.get("remove") == "true", "replaceAll": sc.get("replace-all") == "true",
                    "source": source,
                })

    def _action_like(self, el, parent: str | None, source: str, bundle, bundles: Bundles) -> None:
        tag = local(el.tag)
        if tag == "separator":
            if parent:
                self.group_children[parent].append({"separator": True, "text": el.get("text")})
            return
        if tag == "reference":
            ref = el.get("ref") or el.get("id")
            if ref and parent:
                self.group_children[parent].append({"ref": ref})
            for a in el:
                if local(a.tag) == "add-to-group" and ref:
                    self.additions.append((ref, dict(a.attrib)))
            return
        if tag not in ("action", "group"):
            return
        aid = el.get("id") or (el.get("class") or "").rsplit(".", 1)[-1]
        if not aid:
            return
        kind = tag
        key = f"{'action' if kind == 'action' else 'group'}.{aid}"
        text = el.get("text") or bundles.get(f"{key}.text", el.get("resource-bundle") or bundle)
        desc = el.get("description") or bundles.get(f"{key}.description", bundle)
        entry = self.actions.get(aid)
        if entry is None:
            entry = {"id": aid, "kind": kind, "text": clean_text(text), "description": desc,
                     "popup": el.get("popup") == "true" if kind == "group" else None,
                     "useShortcutOf": el.get("use-shortcut-of"), "source": source,
                     "searchable": el.get("searchable") != "false",
                     "internal": el.get("internal") == "true"}
            self.actions[aid] = entry
        if parent:
            self.group_children[parent].append({"ref": aid})
        self._shortcuts(el, aid, source)
        for child in el:
            ct = local(child.tag)
            if ct == "add-to-group":
                self.additions.append((aid, dict(child.attrib)))
            elif kind == "group" and ct in ("action", "group", "reference", "separator"):
                self._action_like(child, aid, source, bundle, bundles)

    def _configurable(self, el, parent_id, tag, source, bundle, bundles: Bundles) -> None:
        cid = el.get("id") or (el.get("instance") or el.get("provider") or "").rsplit(".", 1)[-1]
        b = el.get("bundle") or bundle
        name = el.get("displayName")
        if not name and el.get("key"):
            name = bundles.get(el.get("key"), b)
        weight = el.get("groupWeight")
        self.configurables.append({
            "id": cid,
            "parentId": parent_id or el.get("parentId"),
            "groupId": el.get("groupId"),
            "displayName": name.replace("\\&", "&") if name else name,
            "level": "project" if tag == "projectConfigurable" else "application",
            "groupWeight": int(weight) if weight and weight.lstrip("-").isdigit() else 0,
            "nonDefaultProject": el.get("nonDefaultProject") == "true",
            "dynamic": el.get("dynamic") == "true",
            "source": source,
        })
        for child in el:
            if local(child.tag) == "configurable":
                self._configurable(child, cid, tag, source, b, bundles)

    # ---- group resolution
    def resolve_groups(self) -> dict[str, list]:
        groups = {k: [dict(c) for c in v] for k, v in self.group_children.items()}
        # Plugin load order is not specified, so an addition anchored to an action that
        # another plugin adds later is retried until no more anchors resolve.
        pending = [(c, a) for c, a in self.additions if a.get("group-id")]
        final_pass = False
        while pending:
            retry = []
            for child, attrs in pending:
                lst = groups.setdefault(attrs["group-id"], [])
                if any(c.get("ref") == child for c in lst):
                    continue
                anchor = attrs.get("anchor", "last")
                relto = attrs.get("relative-to-action")
                idx = None
                if anchor in ("before", "after") and relto:
                    for i, c in enumerate(lst):
                        if c.get("ref") == relto:
                            idx = i if anchor == "before" else i + 1
                            break
                    if idx is None and not final_pass:
                        retry.append((child, attrs))
                        continue
                if anchor == "first":
                    idx = 0
                if idx is None:
                    lst.append({"ref": child})
                else:
                    lst.insert(idx, {"ref": child})
            if len(retry) == len(pending):
                final_pass = True
            pending = retry
        return groups


# ---------------------------------------------------------------- keymaps

def parse_keymap_file(path: Path):
    el = parse_xml(path)
    if el is None:
        return None
    entries: dict[str, list[dict]] = OrderedDict()
    for a in el.findall("action"):
        scs = []
        for sc in a:
            t = local(sc.tag)
            if t == "keyboard-shortcut":
                scs.append({"kind": "keyboard", "first": sc.get("first-keystroke"),
                            "second": sc.get("second-keystroke")})
            elif t == "mouse-shortcut":
                scs.append({"kind": "mouse", "first": sc.get("keystroke"), "second": None})
            elif t == "keyboard-gesture-shortcut":
                scs.append({"kind": "gesture", "first": sc.get("keystroke"),
                            "second": sc.get("modifier")})
        entries[a.get("id")] = scs
    return {"name": el.get("name"), "parent": el.get("parent"), "entries": entries}


MOD_ORDER = ["ctrl", "meta", "alt", "shift"]
MOD_ALIASES = {"control": "ctrl", "ctrl": "ctrl", "meta": "meta", "alt": "alt", "shift": "shift",
               "altGraph": "altGr", "alt_graph": "altGr"}


def norm_stroke(s: str | None) -> str | None:
    if not s:
        return s
    parts = s.split()
    mods, keys = [], []
    for p in parts:
        m = MOD_ALIASES.get(p)
        if m:
            mods.append(m)
        elif p in ("pressed", "released", "typed"):
            continue
        else:
            keys.append(p)
    mods = sorted(set(mods), key=lambda m: MOD_ORDER.index(m) if m in MOD_ORDER else 9)
    return " ".join(mods + keys)


def swap_ctrl_meta(stroke: str | None) -> str | None:
    if not stroke:
        return stroke
    toks = stroke.split()
    toks = ["meta" if t == "ctrl" else "ctrl" if t == "meta" else t for t in toks]
    return norm_stroke(" ".join(toks))


def sc_key(sc: dict) -> tuple:
    return (sc["kind"], sc["first"], sc.get("second"))


def resolve_keymaps(files: dict[str, dict], plugin_shortcuts: list[dict]):
    for km in files.values():
        for scs in km["entries"].values():
            for sc in scs:
                if sc["kind"] != "gesture":
                    sc["first"] = norm_stroke(sc["first"])
                    sc["second"] = norm_stroke(sc["second"])
    # Plugin XML shortcuts are applied on top of the bundled keymap files.
    own: dict[str, dict[str, list[dict]]] = {n: {k: list(v) for k, v in km["entries"].items()}
                                              for n, km in files.items()}
    cache: dict[tuple, list] = {}

    def effective(name: str, action: str, depth=0) -> list[dict]:
        if (name, action) in cache:
            return cache[(name, action)]
        km = files.get(name)
        res: list[dict] = []
        if km is not None:
            if action in own[name]:
                res = own[name][action]
            elif km["parent"] and depth < 10:
                res = effective(km["parent"], action, depth + 1)
                if name.startswith("Mac OS X") and not km["parent"].startswith("Mac OS X"):
                    res = [dict(sc, first=swap_ctrl_meta(sc["first"]) if sc["kind"] != "gesture" else sc["first"],
                                second=swap_ctrl_meta(sc["second"]) if sc["kind"] == "keyboard" else sc["second"])
                           for sc in res]
        cache[(name, action)] = res
        return res

    for ps in plugin_shortcuts:
        name = ps["keymap"]
        if name not in files:
            continue
        cache.clear()
        cur = list(effective(name, ps["action"]))
        sc = {"kind": ps["kind"], "first": norm_stroke(ps["first"]), "second": norm_stroke(ps["second"])}
        if ps["replaceAll"]:
            cur = []
        if ps["remove"]:
            cur = [c for c in cur if sc_key(c) != sc_key(sc)]
        elif sc_key(sc) not in {sc_key(c) for c in cur}:
            cur.append(sc)
        own[name][ps["action"]] = cur
    cache.clear()
    all_actions = set()
    for n in own:
        all_actions |= set(own[n])
    out = OrderedDict()
    for name in files:
        m = OrderedDict()
        for a in sorted(all_actions):
            scs = effective(name, a)
            if scs:
                m[a] = [fmt_sc(sc) for sc in scs]
        out[name] = m
    return out


KEY_NAMES = {"BACK_SPACE": "Backspace", "ESCAPE": "Esc", "ENTER": "Enter", "DELETE": "Delete",
             "INSERT": "Insert", "HOME": "Home", "END": "End", "PAGE_UP": "Page Up",
             "PAGE_DOWN": "Page Down", "SPACE": "Space", "TAB": "Tab", "UP": "Up", "DOWN": "Down",
             "LEFT": "Left", "RIGHT": "Right", "SLASH": "/", "BACK_SLASH": "\\", "COMMA": ",",
             "PERIOD": ".", "SEMICOLON": ";", "QUOTE": "'", "BACK_QUOTE": "`", "MINUS": "-",
             "EQUALS": "=", "OPEN_BRACKET": "[", "CLOSE_BRACKET": "]", "ADD": "NumPad +",
             "SUBTRACT": "NumPad -", "MULTIPLY": "NumPad *", "DIVIDE": "NumPad /",
             "CONTEXT_MENU": "Context Menu", "PLUS": "+"}
MOD_LABEL = {"ctrl": "Ctrl", "meta": "Cmd", "alt": "Alt", "shift": "Shift", "altGr": "AltGr"}
MAC_MOD_LABEL = {"ctrl": "⌃", "meta": "⌘", "alt": "⌥", "shift": "⇧", "altGr": "AltGr"}


# Labels that IntelliJ computes at runtime (no static text in the data files). Taken from
# the menus as shown in the public IntelliJ IDEA documentation; verify in the IDE.
RUNTIME_LABELS = {
    "ShowSettings": "Settings…", "ConvertModuleGroupsToQualifiedNames": "Convert Module Groups to Qualified Names…",
    "ConvertToWindowsLineSeparators": "CRLF - Windows (\\r\\n)",
    "ConvertToUnixLineSeparators": "LF - Unix and macOS (\\n)",
    "ConvertToMacLineSeparators": "CR - Classic Mac OS (\\r)",
    "TemplateProjectProperties": "Settings for New Projects…",
    "RunConfigurationTemplatesForNewProjects": "Run Configuration Templates for New Projects…",
    "Exit": "Exit", "Back": "Back", "Forward": "Forward", "RunInspection": "Run Inspection by Name…",
    "XDebugger.AttachToProcess": "Attach to Process…", "ManageTargets": "Manage Targets…", "Stop": "Stop",
    "tasks.goto": "Open Task…", "Annotate": "Annotate with Git Blame",
    "MaximizeToolWindow": "Maximize Tool Window", "MaximizeEditorInSplit": "Maximize Editor in Split",
    "Unsplit": "Unsplit", "About": "About",
}

# Groups only visible in IntelliJ's internal mode (-Didea.is.internal=true).
INTERNAL_GROUPS = {"Internal", "InternalActions", "Internal.UI", "Internal.Errors", "UiDebugger"}


def fmt_sc(sc: dict) -> str:
    if sc["kind"] == "gesture":
        key = (sc["first"] or "").split()[-1]
        mod = {"CONTROL": "Ctrl", "SHIFT": "Shift", "ALT": "Alt", "META": "Cmd"}.get(key, key)
        return f"double {mod}" if sc.get("second") == "dblClick" else f"{mod} ({sc.get('second')})"
    if sc["kind"] == "mouse":
        toks = (sc["first"] or "").split()
        mods = [MOD_LABEL.get(MOD_ALIASES.get(t, ""), None) for t in toks]
        rest = [t for t, m in zip(toks, mods) if m is None]
        pretty = {"button1": "Click", "button2": "Middle Click", "button3": "Right Click",
                  "doubleClick": "(double)", "wheelUp": "Wheel Up", "wheelDown": "Wheel Down",
                  "button4": "Mouse Back", "button5": "Mouse Forward"}
        return "+".join([m for m in mods if m] + [" ".join(pretty.get(r, r) for r in rest)])
    def one(stroke):
        toks = stroke.split()
        mods = [MOD_LABEL[t] for t in toks if t in MOD_LABEL]
        keys = [KEY_NAMES.get(t, re.sub(r"^NUMPAD(\d)$", r"NumPad \1", t)) for t in toks if t not in MOD_LABEL]
        return "+".join(mods + keys)
    s = one(sc["first"] or "")
    if sc.get("second"):
        s += ", " + one(sc["second"])
    return s


# ---------------------------------------------------------------- main

def main() -> int:
    if len(sys.argv) != 3:
        print(__doc__)
        return 2
    src, out = Path(sys.argv[1]).resolve(), Path(sys.argv[2]).resolve()
    data_dir = out / "data"
    data_dir.mkdir(parents=True, exist_ok=True)
    build = (src / "build.txt").read_text().strip() if (src / "build.txt").exists() else "unknown"

    bundles = Bundles(src)
    cat = Catalog()
    xml_files = [p for p in sorted(src.rglob("*.xml"))
                 if ".git" not in p.parts and included(rel(src, p)) and "/keymaps/" not in "/" + rel(src, p)]
    # Platform first so its static group contents precede plugin additions.
    xml_files.sort(key=lambda p: (0 if rel(src, p).startswith("platform/") else 1, rel(src, p)))
    for p in xml_files:
        el = parse_xml(p)
        if el is not None:
            cat.ingest(el, rel(src, p), bundles)
    groups = cat.resolve_groups()

    keymap_files: dict[str, dict] = OrderedDict()
    for p in sorted(src.rglob("keymaps/*.xml")):
        if included(rel(src, p)) or rel(src, p).startswith("plugins/keymaps/"):
            km = parse_keymap_file(p)
            if km and km["name"]:
                km["source"] = rel(src, p)
                keymap_files[km["name"]] = km
    keymaps = resolve_keymaps(keymap_files, cat.plugin_shortcuts)
    # use-shortcut-of: actions that borrow another action's shortcut.
    for a in cat.actions.values():
        if a.get("useShortcutOf"):
            for km in keymaps.values():
                if a["useShortcutOf"] in km and a["id"] not in km:
                    km[a["id"]] = list(km[a["useShortcutOf"]])

    meta = {"source": "https://github.com/JetBrains/intellij-community",
            "tag": "idea/262.10968.63", "build": build, "license": "Apache-2.0 (derived data)"}

    def dump(name, payload):
        (data_dir / name).write_text(json.dumps({"meta": meta, **payload}, indent=1, ensure_ascii=False) + "\n")

    dump("actions.json", {"actions": list(cat.actions.values()),
                          "groups": {k: v for k, v in sorted(groups.items())}})
    dump("keymaps.json", {"keymaps": {n: {"parent": keymap_files[n]["parent"],
                                          "source": keymap_files[n]["source"],
                                          "shortcuts": keymaps[n]} for n in keymaps}})
    dump("settings.json", {"configurables": cat.configurables,
                           "advancedSettings": cat.advanced_settings})
    dump("toolwindows.json", {"toolWindows": cat.toolwindows})

    themes, schemes = extract_themes(src)
    dump("themes.json", {"themes": themes})
    dump("color-schemes.json", {"schemes": schemes})

    write_markdown(out, meta, cat, groups, keymaps, keymap_files)
    write_themes_markdown(out, meta, themes, schemes)
    print(f"actions={len(cat.actions)} groups={len(groups)} configurables={len(cat.configurables)} "
          f"advanced={len(cat.advanced_settings)} toolwindows={len(cat.toolwindows)} "
          f"keymaps={len(keymaps)} plugin_shortcuts={len(cat.plugin_shortcuts)}")
    return 0


# ---------------------------------------------------------------- themes

THEME_FILES = OrderedDict([
    ("Islands Dark", "islands/ManyIslandsDark.theme.json"),
    ("Islands Light", "islands/ManyIslandsLight.theme.json"),
    ("Islands Darcula", "islands/ManyIslandsDarcula.theme.json"),
    ("ExperimentalDark", "expUI/expUI_dark.theme.json"),
    ("ExperimentalLight", "expUI/expUI_light.theme.json"),
    ("ExperimentalLightWithLightHeader", "expUI/expUI_light_with_light_header.theme.json"),
    ("JetBrainsHighContrastTheme", "islands/HighContrast.theme.json"),
    ("Darcula", "darcula.theme.json"),
    ("IntelliJ", "intellijlaf.theme.json"),
])


def flatten_ui(ui: dict, prefix: str = "") -> dict:
    out = {}
    for k, v in ui.items():
        key = f"{prefix}.{k}" if prefix else k
        if isinstance(v, dict):
            out.update(flatten_ui(v, key))
        else:
            out[key] = v
    return out


def extract_themes(src: Path):
    tdir = src / "platform/platform-resources/src/themes"
    raw = {}
    for tid, f in THEME_FILES.items():
        p = tdir / f
        if p.exists():
            try:
                raw[tid] = json.loads(re.sub(r"^\s*//.*$", "", p.read_text(), flags=re.M))
            except json.JSONDecodeError:
                pass
    by_name = {v.get("name"): k for k, v in raw.items()}

    def chain(tid, depth=0):
        t = raw.get(tid)
        if t is None or depth > 8:
            return []
        parent = t.get("parentTheme")
        parent = parent if parent in raw else by_name.get(parent)
        return (chain(parent, depth + 1) if parent else []) + [tid]

    themes = OrderedDict()
    for tid, t in raw.items():
        colors, ui = {}, {}
        for anc in chain(tid):
            colors.update(raw[anc].get("colors", {}))
            ui.update(flatten_ui(raw[anc].get("ui", {})))

        def resolve(v, depth=0):
            if isinstance(v, str) and v in colors and depth < 10:
                return resolve(colors[v], depth + 1)
            return v
        themes[tid] = {
            "name": t.get("name"), "dark": t.get("dark"), "parentTheme": t.get("parentTheme"),
            "editorScheme": t.get("editorScheme"), "inheritance": chain(tid), "file": THEME_FILES[tid],
            "palette": {k: resolve(v) for k, v in colors.items()},
            "ui": {k: resolve(v) for k, v in sorted(ui.items())
                   if not (isinstance(v, str) and v.startswith("com.intellij."))},
        }

    # Editor color schemes: the two base schemes plus bundled ones, with plugin additions.
    scheme_files = {}
    base = parse_xml(src / "platform/platform-resources/src/DefaultColorSchemesManager.xml")
    raw_schemes = OrderedDict()
    if base is not None:
        for sc in base.findall("scheme"):
            raw_schemes[sc.get("name")] = sc
    for p in sorted(tdir.rglob("*.xml")):
        el = parse_xml(p)
        if el is not None and local(el.tag) == "scheme" and el.get("name"):
            raw_schemes[el.get("name")] = el
            scheme_files[el.get("name")] = rel(src, p)
    additions = defaultdict(list)
    for p in sorted(src.rglob("*.xml")):
        r = rel(src, p)
        if not included(r) or "/keymaps/" in "/" + r:
            continue
        try:
            text = p.read_text(errors="ignore")
        except OSError:
            continue
        for m in re.finditer(r'<additionalTextAttributes\s+scheme="([^"]+)"\s+file="([^"]+)"', text):
            fname = m.group(2).lstrip("/")
            hits = [q for q in p.parents[2].rglob(fname)] if len(p.parents) > 2 else []
            if hits:
                additions[m.group(1)].append(hits[0])

    def read_scheme(el):
        colors = {o.get("name"): o.get("value") for o in el.findall("./colors/option")}
        attrs = {}
        for o in el.findall("./attributes/option"):
            if o.get("baseAttributes"):
                attrs[o.get("name")] = {"inherit": o.get("baseAttributes")}
                continue
            v = {x.get("name"): x.get("value") for x in o.findall("./value/option")}
            attrs[o.get("name")] = v
        return colors, attrs

    schemes = OrderedDict()
    for name, el in raw_schemes.items():
        colors, attrs = {}, {}
        lineage, cur = [], name
        while cur in raw_schemes and cur not in lineage:
            lineage.insert(0, cur)
            cur = raw_schemes[cur].get("parent_scheme")
        for n in lineage:
            c, a = read_scheme(raw_schemes[n])
            colors.update(c)
            attrs.update(a)
            for f in additions.get(n, []):
                ael = parse_xml(f)
                if ael is not None:
                    _, a2 = read_scheme(ael)
                    for k, v in a2.items():
                        attrs.setdefault(k, v)
        schemes[name] = {"parent": raw_schemes[name].get("parent_scheme"), "lineage": lineage,
                         "file": scheme_files.get(name, "platform/platform-resources/src/DefaultColorSchemesManager.xml"),
                         "colors": dict(sorted(colors.items())), "attributes": dict(sorted(attrs.items()))}
    return themes, schemes


def write_themes_markdown(out: Path, meta, themes, schemes) -> None:
    lines = [f"<!-- Generated by docs/spec/tools/extract_intellij_data.py from IntelliJ Community "
             f"{meta['tag']}. Do not edit by hand. -->\n",
             "# Themes and editor color schemes\n",
             "IntelliJ IDEA 2026.2 ships the *Islands* look as the default New UI appearance. "
             "Themes are layered: a theme names a `parentTheme`, defines a named color `palette` and "
             "maps UI keys (`Component.property`) to palette names or literal values. Values below are "
             "fully resolved through the parent chain. Full data: [data/themes.json](data/themes.json) "
             "and [data/color-schemes.json](data/color-schemes.json).\n",
             "## Bundled UI themes\n", "| Theme id | Name | Dark | Inherits | Editor scheme | Resolved UI keys |",
             "|---|---|---|---|---|---|"]
    for tid, t in themes.items():
        lines.append(f"| `{tid}` | {t['name']} | {t['dark']} | {' → '.join(t['inheritance'])} | "
                     f"{t['editorScheme'] or '—'} | {len(t['ui'])} |")
    for tid in ("Islands Dark", "Islands Light"):
        t = themes.get(tid)
        if not t:
            continue
        lines += [f"\n## {tid}: semantic palette\n", "| Token | Value |", "|---|---|"]
        for k, v in t["palette"].items():
            if not re.match(r"^(gray|blue|green|red|yellow|orange|purple|teal|pink|white|black)(-\d+)?$", k):
                lines.append(f"| `{k}` | `{v}` |")
        lines += [f"\n## {tid}: shape and size metrics\n", "| UI key | Value |", "|---|---|"]
        for k, v in t["ui"].items():
            if re.search(r"(arc|Arc|[sS]ize|[wW]idth|[hH]eight|[iI]nsets|[bB]orderWidth|Alpha|[mM]argin|gap|Gap)", k) \
                    and not isinstance(v, (dict, list)):
                lines.append(f"| `{k}` | `{v}` |")
    lines += ["\n## Editor color schemes\n", "| Scheme | Inherits | Colors | Text attributes | File |",
              "|---|---|---|---|---|"]
    for n, sc in schemes.items():
        lines.append(f"| {n} | {' → '.join(sc['lineage'])} | {len(sc['colors'])} | {len(sc['attributes'])} | `{sc['file']}` |")
    keys = ["DEFAULT_KEYWORD", "DEFAULT_STRING", "DEFAULT_NUMBER", "DEFAULT_LINE_COMMENT",
            "DEFAULT_DOC_COMMENT", "DEFAULT_FUNCTION_DECLARATION", "DEFAULT_INSTANCE_FIELD",
            "DEFAULT_STATIC_FIELD", "DEFAULT_CONSTANT", "DEFAULT_METADATA", "DEFAULT_IDENTIFIER",
            "DEFAULT_PARAMETER", "DEFAULT_LOCAL_VARIABLE", "DEFAULT_CLASS_NAME", "DEFAULT_INTERFACE_NAME",
            "ERRORS_ATTRIBUTES", "WARNING_ATTRIBUTES", "WRONG_REFERENCES_ATTRIBUTES",
            "NOT_USED_ELEMENT_ATTRIBUTES", "DEPRECATED_ATTRIBUTES", "TODO_DEFAULT_ATTRIBUTES",
            "SEARCH_RESULT_ATTRIBUTES", "IDENTIFIER_UNDER_CARET_ATTRIBUTES", "TEXT"]
    for n in ("Islands Dark", "Light", "Dark", "Darcula", "Default"):
        sc = schemes.get(n)
        if not sc:
            continue
        lines += [f"\n### {n}: key editor colors\n", "| Key | Value |", "|---|---|"]
        for k in ("TEXT", "CARET_ROW_COLOR", "SELECTION_BACKGROUND", "LINE_NUMBERS_COLOR",
                  "LINE_NUMBER_ON_CARET_ROW_COLOR", "GUTTER_BACKGROUND", "INDENT_GUIDE", "CARET_COLOR",
                  "RIGHT_MARGIN_COLOR", "WHITESPACES", "LOOKUP_COLOR", "DOCUMENTATION_COLOR"):
            if k in sc["colors"]:
                lines.append(f"| `{k}` | `#{sc['colors'][k]}` |")
        lines += ["", "| Text attribute | Foreground | Background | Font type | Effect |", "|---|---|---|---|---|"]
        for k in keys:
            v = sc["attributes"].get(k)
            if not v:
                continue
            if "inherit" in v:
                lines.append(f"| `{k}` | inherits `{v['inherit']}` | | | |")
            else:
                ft = {"1": "bold", "2": "italic", "3": "bold italic"}.get(v.get("FONT_TYPE", ""), "")
                eff = f"{v.get('EFFECT_TYPE', '')} #{v['EFFECT_COLOR']}" if v.get("EFFECT_COLOR") else ""
                fg = f"#{v['FOREGROUND']}" if v.get("FOREGROUND") else ""
                bg = f"#{v['BACKGROUND']}" if v.get("BACKGROUND") else ""
                lines.append(f"| `{k}` | {fg} | {bg} | {ft} | {eff} |")
    (out / "themes.md").write_text("\n".join(lines) + "\n")


# ---------------------------------------------------------------- markdown

def md_escape(s: str | None) -> str:
    return (s or "").replace("|", "\\|").replace("\n", " ")


def shortcut_cell(keymaps, km, aid) -> str:
    return md_escape("; ".join(keymaps.get(km, {}).get(aid, []))) or ""


def write_markdown(out: Path, meta, cat: Catalog, groups, keymaps, keymap_files) -> None:
    header = (f"<!-- Generated by docs/spec/tools/extract_intellij_data.py from IntelliJ Community "
              f"{meta['tag']} (build {meta['build']}). Do not edit by hand. -->\n\n")
    acts = cat.actions

    def label(aid):
        a = acts.get(aid)
        m = re.fullmatch(r"Activate(.+)ToolWindow", aid)
        return ((a and a.get("text")) or RUNTIME_LABELS.get(aid)
                or (m and f"{re.sub(r'(?<=[a-z])(?=[A-Z])', ' ', m.group(1))} (tool window)") or aid)

    # ---- menus
    lines = [header, "# Menus and action groups\n",
             "Resolved structure of the main menu and the most important context menus / toolbars. "
             "Each line shows the label, the action id in backticks, and default shortcuts "
             "(Windows `$default` / macOS). `---` is a separator; ▸ marks a submenu (popup group); "
             "non-popup groups are inlined in place, as IntelliJ renders them.\n"]

    def render(gid, depth, seen, max_depth=8):
        res = []
        for c in groups.get(gid, []):
            if c.get("separator"):
                res.append("  " * depth + "- ---" + (f" *{c['text']}*" if c.get("text") else ""))
                continue
            ref = c["ref"]
            a = acts.get(ref, {"kind": "action", "text": None})
            if a.get("internal") or ref in INTERNAL_GROUPS:
                continue
            if a["kind"] == "group":
                if ref in seen or depth > max_depth:
                    continue
                if a.get("popup") and a.get("text"):
                    res.append("  " * depth + f"- ▸ **{md_escape(label(ref))}** `{ref}`")
                    res += render(ref, depth + 1, seen | {ref})
                else:
                    res += render(ref, depth, seen | {ref})
            else:
                sw = "; ".join(keymaps.get("$default", {}).get(ref, []))
                sm = "; ".join(keymaps.get("Mac OS X 10.5+", {}).get(ref, []))
                sc = f" — {sw}" + (f" / {sm}" if sm else "") if sw or sm else ""
                res.append("  " * depth + f"- {md_escape(label(ref))} `{ref}`{md_escape(sc)}")
        return res

    for gid, title in [("MainMenu", "Main menu"), ("MainToolbarNewUI", "Main toolbar (New UI)"),
                       ("EditorPopupMenu", "Editor context menu"),
                       ("ProjectViewPopupMenu", "Project view context menu"),
                       ("EditorTabPopupMenu", "Editor tab context menu"),
                       ("NavBarToolBar", "Navigation bar toolbar"),
                       ("ToolWindowContextMenu", "Tool window context menu"),
                       ("RunContextGroup", "Run context group"),
                       ("GenerateGroup", "Generate (Alt+Insert)"),
                       ("RefactoringMenu", "Refactor This (Ctrl+Alt+Shift+T)"),
                       ("Vcs.Operations.Popup", "VCS operations popup (Alt+`)"),
                       ("WelcomeScreen.QuickStart", "Welcome screen quick start")]:
        if gid in groups:
            lines += [f"\n## {title} (`{gid}`)\n"] + render(gid, 0, {gid})
    (out / "menus.md").write_text("\n".join(lines) + "\n")

    # ---- keymap reference
    kl = [header, "# Default keymaps\n",
          "Effective default shortcuts per bundled keymap, after applying keymap inheritance "
          "(`parent=`), plugin-declared shortcuts, `use-shortcut-of`, and the macOS rule that "
          "swaps Ctrl and Cmd for shortcuts inherited from a non-macOS parent. Full data for all "
          f"{len(keymaps)} keymaps is in [data/keymaps.json](data/keymaps.json).\n",
          "| Keymap (file name) | Shown as | Parent | Own entries |", "|---|---|---|---|"]
    for n, km in keymap_files.items():
        kl.append(f"| `{n}` | {PRIMARY_KEYMAPS.get(n, n)} | `{km['parent'] or '—'}` | {len(km['entries'])} |")
    kl += ["", "## Primary keymaps (actions with at least one default shortcut)\n",
           "| Action | Id | Windows | macOS | GNOME | KDE | XWin |", "|---|---|---|---|---|---|---|"]
    ids = sorted(set().union(*[set(keymaps.get(k, {})) for k in PRIMARY_KEYMAPS]),
                 key=lambda a: (label(a) or a).lower())
    for aid in ids:
        kl.append(f"| {md_escape(label(aid))} | `{aid}` | " +
                  " | ".join(shortcut_cell(keymaps, k, aid) for k in PRIMARY_KEYMAPS) + " |")
    (out / "keymaps.md").write_text("\n".join(kl) + "\n")

    # ---- settings tree
    confs = cat.configurables
    by_parent = defaultdict(list)
    ids_known = {c["id"] for c in confs}
    for c in confs:
        # parentId and groupId are interchangeable; groupId usually names a top-level group.
        parent = c["parentId"] or c["groupId"] or "other"
        if parent in CONFIGURABLE_GROUPS:
            p = f"group:{parent}"
        elif parent == "root":
            p = "root"
        elif parent in ids_known:
            p = parent
        else:
            p = f"missing:{parent}"
        by_parent[p].append(c)

    def sort_key(c):
        return (-c["groupWeight"], (c["displayName"] or c["id"] or "").lower())

    sl = [header, "# Settings tree\n",
          "Configurables registered by IntelliJ Community 2026.2 modules bundled with IntelliJ IDEA, "
          "arranged as the Settings dialog shows them: top-level groups, then children sorted by "
          "`groupWeight` (descending) and name. `[P]` marks project-level pages (stored in `.idea/`), "
          "others are application-level. Commercial-only pages (Database, Spring, JavaScript, ...) "
          "are not in this data; see [settings.md](settings.md).\n"]

    def render_conf(c, depth, seen):
        res = [("  " * depth) + f"- {md_escape(c['displayName'] or '(unnamed)')} `{c['id']}`" +
               (" [P]" if c["level"] == "project" else "")]
        if c["id"] in seen:
            return res
        for ch in sorted(by_parent.get(c["id"], []), key=sort_key):
            res += render_conf(ch, depth + 1, seen | {c["id"]})
        return res

    roots = sorted(by_parent.get("root", []), key=sort_key)
    sl.append("## Top-level pages (`groupId=\"root\"`)\n")
    for c in roots:
        sl += render_conf(c, 0, set())
    for gid, name in CONFIGURABLE_GROUPS.items():
        sl.append(f"\n## {name} (`{gid}`)\n")
        for c in sorted(by_parent.get(f"group:{gid}", []), key=sort_key):
            sl += render_conf(c, 0, set())
    other = sorted(k for k in by_parent if k.startswith("missing:"))
    sl.append("\n## Pages whose parent is outside the Community data\n")
    for k in other:
        for c in sorted(by_parent.get(k, []), key=sort_key):
            sl.append(f"- {md_escape(c['displayName'] or '(unnamed)')} `{c['id']}` → parent `{k.split(':',1)[-1]}`")
    sl += ["\n## Advanced Settings page\n", "| Group | Setting | Id | Default |", "|---|---|---|---|"]
    for s in sorted(cat.advanced_settings, key=lambda s: ((s["group"] or ""), (s["title"] or s["id"]))):
        sl.append(f"| {md_escape(s['group'])} | {md_escape(s['title'])} | `{s['id']}` | `{md_escape(s['default'])}` |")
    (out / "settings-tree.md").write_text("\n".join(sl) + "\n")

    # ---- tool windows
    tl = [header, "# Tool windows\n",
          "All `toolWindow` registrations in IntelliJ Community 2026.2 that ship with IntelliJ IDEA. "
          "Anchor/secondary define the default stripe position (secondary = lower/right-hand part of "
          "the stripe in the New UI). Shortcuts are the defaults of the matching `Activate…ToolWindow` "
          "action.\n", "| Id | Anchor | Secondary | Windows | macOS | Module |", "|---|---|---|---|---|---|"]
    for t in sorted(cat.toolwindows, key=lambda t: (t["anchor"], t["secondary"], t["id"].lower())):
        aid = "Activate" + re.sub(r"[^A-Za-z0-9]", "", t["id"]) + "ToolWindow"
        tl.append(f"| {md_escape(t['id'])} | {t['anchor']} | {'yes' if t['secondary'] else 'no'} | "
                  f"{shortcut_cell(keymaps, '$default', aid)} | {shortcut_cell(keymaps, 'Mac OS X 10.5+', aid)} | "
                  f"`{t['source'].split('/resources')[0].split('/src/')[0]}` |")
    registered = {"Activate" + re.sub(r"[^A-Za-z0-9]", "", t["id"]) + "ToolWindow" for t in cat.toolwindows}
    tl += ["", "## Tool windows registered at runtime\n",
           "These have an `Activate…ToolWindow` action with a default shortcut but are created "
           "programmatically (Run, Debug, Find, Services, Build, Maven, ...), so their stripe "
           "position comes from the UI documentation ([ui/workbench.md](ui/workbench.md)).\n",
           "| Tool window | Action | Windows | macOS |", "|---|---|---|---|"]
    rt = sorted({a for km in ("$default", "Mac OS X 10.5+") for a in keymaps.get(km, {})
                 if re.fullmatch(r"Activate.+ToolWindow", a) and a not in registered})
    for aid in rt:
        tl.append(f"| {md_escape(label(aid))} | `{aid}` | {shortcut_cell(keymaps, '$default', aid)} | "
                  f"{shortcut_cell(keymaps, 'Mac OS X 10.5+', aid)} |")
    (out / "toolwindows.md").write_text("\n".join(tl) + "\n")


if __name__ == "__main__":
    sys.exit(main())
