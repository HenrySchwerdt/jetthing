# mongo-raabit

A clean-room, Studio 3T Pro-class MongoDB GUI for Jetthing (issue #50). It installs as a
Jetthing plugin and will also run standalone (`jetthing --app mongo-raabit`) once the
plugin API (#48) lands.

| Crate | Path | Purpose |
|---|---|---|
| `mongo-raabit` | `.` | egui views: Connection Manager, connection tree, collection tabs with Table/Tree/JSON results; `examples/standalone.rs` runs it in its own window |
| `mongo-raabit-core` | `core/` | headless: connection profiles, SSH tunnels, secrets, metadata, query/paging/CRUD, explain, index/user management, aggregation stages with preview, schema sampler, SQL → MQL, query code generation, import/export |

The core is built on the official [`mongodb`](https://crates.io/crates/mongodb) driver.
External tools (`ssh`, later `mongosh`) are discovered on `PATH` at runtime and never
bundled.

## Tests

```bash
mongod --dbpath /tmp/mongo-data --fork --logpath /tmp/mongod.log   # optional
cargo test -p mongo-raabit-core
```

Tests that need a server use `MONGO_RAABIT_TEST_URI` (default
`mongodb://127.0.0.1:27017`) and print a skip message when it is unreachable.
Seed the sample database used for screenshots with
`mongosh plugins/mongo-raabit/scripts/seed-sample-data.js`.

Parity status: `docs/parity/mongo-raabit.md`.
