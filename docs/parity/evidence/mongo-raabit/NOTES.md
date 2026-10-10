# mongo-raabit evidence

Captured under Xvfb (1600×1000, `JETTHING_RENDERER=glow`) from the standalone example
window (1440×900) against MongoDB Community 8.0.32, seeded with
`plugins/mongo-raabit/scripts/seed-sample-data.js` (`raabit_shop`: 240 customers,
1,200 orders, products, a view, a time series collection and a GridFS bucket).
The example will be replaced by `jetthing --app dev.jetthing.mongo-raabit` once the
plugin API (#48) lands.

```bash
cargo run -p mongo-raabit --example standalone -- --uri mongodb://127.0.0.1:27017 \
    --open raabit_shop.customers [--view tree|json] [--drill items] [--light]
cargo run -p mongo-raabit --example standalone -- --demo-profiles --manager
```

There is no Studio 3T reference capture (clean-room: behavior is taken from public
documentation); the look follows the Jetthing Islands theme instead.

| File | Scene | Notes |
|---|---|---|
| `welcome-dark.png` | Welcome tab | quick actions, saved connections with masked URIs |
| `manager-dark.png`, `manager-light.png` | Connection Manager | folders, filter, Server/Auth/TLS/SSH/Advanced pages, live masked connection string |
| `table-dark.png`, `table-light.png` | Collection tab, Table view | connection tree with sizes/counts, view/time series/GridFS icons, query bar, paging, count, query time, type-colored cells, nested values as drill-down pills |
| `drilldown-dark.png` | Table drill-down | `orders` filtered on status/total, drilled into `items` (one row per array element, `#` keeps the source document) |
| `tree-dark.png` | Tree view | expandable documents with key/value/type columns |
| `json-dark.png` | JSON view | syntax-colored mongo shell syntax per document |
| `document-editor-dark.png` | Edit Document | syntax-colored editor with live validation (errors give line and column), Format, Save (`_id`-safe replace) |
| `inline-edit-dark.png` | Inline cell edit | double-click a scalar cell; Enter/focus loss saves with `$set`, Esc cancels; strings stay strings |
| `delete-confirm-light.png` | Delete confirmation | lists the `_id`s to delete |
| `explain-collscan-dark.png`, `explain-index-dark.png` | Explain plan | stats cards, COLLSCAN warning, winning plan as a left-to-right stage flow, raw output toggle |
| `query-code-dark.png` | Query Code | mongo shell plus 8 driver languages, copy; passwords are replaced by `<password>` |

```bash
cargo run -p mongo-raabit --example standalone -- --uri mongodb://127.0.0.1:27017 \
    --open raabit_shop.orders --dialog edit|insert|delete|explain|code [--edit-cell tier]
```

Known gaps: aggregation/SQL/shell/schema tabs, index manager and import/export wizard
are not in these slices; credentials are kept in memory only (OS keychain pending).
