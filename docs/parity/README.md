# Parity inventory format

Every Markdown file in this directory except `README.md` and the generated
`dashboard.md` is an **area**. `cargo run -p xtask -- parity` parses all areas and
regenerates `dashboard.md` and the summary block in the top-level `README.md`;
`cargo run -p xtask -- parity --check` fails when any area is malformed or an ID is
reused. Feature PRs should not commit the generated files, so parallel PRs that add rows
do not conflict; they are regenerated on main. `cargo test -p xtask` also validates the committed inventory.

## Tables

A parity table is a Markdown table whose header row names at least an `ID` and a
`Status` column. Columns are found by header name, so their order is free. The
canonical 8-column header is:

```markdown
| ID | Reference | Feature | Status | Weight | Commands | Tests | Limitations / evidence |
|---|---|---|---|---:|---|---|---|
```

The older 7-column header (no `Weight`) is still accepted, and every row in such a
table has weight 1. Tables whose header lacks `ID` or `Status` are ignored, so areas
may contain other tables.

Rules enforced by the parser (errors point at `file:line`):

- every data row has the same number of cells as its header (escape a literal pipe as `\|`);
- `ID` is non-empty and unique across **all** areas — prefix IDs by area, for example
  `IJ-ED-001` or `PLAT-001`;
- `Status` is exactly one of `Missing`, `Planned`, `Partial`, `Implemented`, `Verified`;
- `Weight` is empty (meaning 1) or an integer from 1 to 100. The IntelliJ catalog uses
  1-5: 5 = core daily workflow, 3 = significant, 2 = moderate, 1 = minor.

Put `[Ultimate]` in any cell (conventionally at the start of `Feature`) to mark an
IntelliJ IDEA Ultimate-only feature; the dashboard reports these separately.

## Status meanings

- **Missing**: nothing exists in Jetthing yet.
- **Planned**: a design or command name exists, no working behavior.
- **Partial**: some of the listed scope works; `Limitations / evidence` says what is missing.
- **Implemented**: the listed scope works and the `Tests` column names the evidence.
- **Verified**: independently checked against the reference, with reproducible evidence
  recorded in this directory. Do not use it without that evidence.

## Scoring

`% implemented` = (weight of Implemented + Verified rows + half the weight of Partial
rows) / total weight. `% verified` = weight of Verified rows / total weight. Both are
reported overall, per area, and per edition scope.

## Spec links

Tables may add a `Spec` column (the canonical header now has 9 columns):

```markdown
| ID | Reference | Feature | Status | Weight | Spec | Commands | Tests | Limitations / evidence |
|---|---|---|---|---:|---|---|---|---|
```

A `Spec` cell is `—` (no link) or a comma-separated list of references, each optionally
in backticks:

- a feature id from `docs/spec/features/*.md`, for example `PLATFORM-EDIT-001`;
- `action:<id>`, an action id from `docs/spec/data/actions.json`;
- `toolwindow:<id>`, a tool-window id from `docs/spec/data/toolwindows.json`;
- `setting:<id>`, a configurable id from `docs/spec/data/settings.json`.

`parity --check` fails on references that do not exist in the spec, and on any spec
feature id or tool window that no row links. `cargo run -p xtask -- parity --sync-spec`
appends a `Missing` row for every unlinked reference to `spec-<featurefile>.md`
(tool windows go to `spec-platform.md` as `SPEC-TW-*` rows), in spec document order,
with the weight taken from the spec priority: P0 = 5, P1 = 3, P2 = 1. Keep the table
that `--sync-spec` appends to as the last table in its file. When a catalog row later
covers a spec feature, link it from that row and delete the generated row.

A spec feature's status is the least complete status among the rows linking it. The
dashboard's **Spec coverage** section reports the priority-weighted percentage per
priority and per spec family (Platform, Java, Gradle), plus how many tool windows,
settings pages and actions are linked.
