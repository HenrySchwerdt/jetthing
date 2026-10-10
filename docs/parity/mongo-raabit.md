# mongo-raabit parity

mongo-raabit is a first-party Jetthing plugin (`plugins/mongo-raabit/`, issue #50): a
clean-room MongoDB GUI modelled on the publicly observable feature set of Studio 3T Pro.
No Studio 3T code, artwork or branding is used. The headless core
(`plugins/mongo-raabit/core`, crate `mongo-raabit-core`) is built on the official
`mongodb` Rust driver and has no UI dependencies.

Tests marked *live* run against a real `mongod` (`MONGO_RAABIT_TEST_URI`, default
`mongodb://127.0.0.1:27017`) and print a skip message when none is reachable. The
evidence runs used MongoDB Community 8.0.32 on Ubuntu 24.04.

| ID | Reference | Feature | Status | Commands | Tests | Limitations / evidence |
|---|---|---|---|---|---|---|
| MR-001 | Studio 3T | Connection profiles (standalone, replica set, sharded, SRV) | Partial | — | URI import/export round-trip, validation and store tests | core model + JSON store with folders, duplicate, URI import with password extraction, masked display URI; Connection Manager UI pending |
| MR-002 | Studio 3T | Authentication (SCRAM, X.509, LDAP, Kerberos, AWS) and TLS | Partial | — | URI generation tests | mechanisms map to driver `authMechanism`/`authSource`; only SCRAM verified against a live server; Kerberos needs the driver `gssapi-auth` feature (not enabled) |
| MR-003 | Studio 3T | SSH tunnel | Partial | — | argument-building and missing-secret tests | system `ssh -N -L` discovered on PATH (agent, key, password via askpass); not yet verified against a live sshd |
| MR-004 | Studio 3T | Secret storage for credentials | Partial | — | memory store test | `SecretStore` abstraction; OS keychain backend pending |
| MR-005 | Studio 3T | Test connection and server info | Implemented | — | live connect + actionable error test | version, topology (standalone / replica set / mongos), round trip |
| MR-006 | Studio 3T | Connection tree metadata (databases, collections, views, GridFS, sizes) | Partial | — | live metadata test | list/create/drop/rename databases and collections, views with pipeline, GridFS bucket detection, `$collStats` sizes; tree UI pending |
| MR-007 | Studio 3T | Query bar (filter, projection, sort, skip, limit) in shell syntax | Partial | — | shell parser round-trip/error tests, query parsing test | mongosh literal syntax incl. ObjectId/ISODate/NumberLong/regex/Extended JSON with error offsets; UI and field autocomplete pending |
| MR-008 | Studio 3T | Result paging, count and query time | Partial | — | live paging test | pages honor query skip/limit, has-more detection, elapsed time; UI pending |
| MR-009 | Studio 3T | Document CRUD (add, edit, inline field edit, clone, delete) | Partial | — | live CRUD test | `_id`-safe replace, `$set`/`$unset` by path, clone with fresh ObjectId, bulk delete; UI pending |
| MR-010 | Studio 3T | Explain plan | Partial | — | explain summary test, live COLLSCAN→IXSCAN test | executionStats summary and stage tree incl. SBE `queryPlan`; visual plan UI pending |
| MR-011 | Studio 3T | Index Manager (list, create, drop, hide, usage) | Partial | — | live index test | unique/sparse/TTL/partial/hidden, sizes and `$indexStats` ops; UI pending |
| MR-012 | Studio 3T | User and role manager | Partial | — | live users/roles test | list/create/drop users, list roles with privileges; UI pending |
| MR-013 | Studio 3T | Server status and current operations | Partial | — | live status test | `serverStatus`, `$currentOp`, `killOp`; UI pending |
| MR-014 | Studio 3T | Aggregation Editor with per-stage preview | Partial | — | pipeline model tests, live preview/`$out`/sampling test | stages with enable/disable/reorder, per-stage input/output preview, writing stages never run in previews, input sampling, stage templates and operator autocomplete list, shell code export; editor UI pending |
| MR-015 | Studio 3T | Visual Query Builder | Missing | — | — | planned |
| MR-016 | Studio 3T | IntelliShell (mongosh) | Missing | — | — | planned |
| MR-017 | Studio 3T | SQL Query (SQL → MQL) | Partial | — | translation unit tests, live execution test | `sqlparser`-based SELECT/DISTINCT/WHERE (comparison, IN, BETWEEN, LIKE/ILIKE, IS NULL, NOT)/GROUP BY/HAVING/ORDER BY/LIMIT/OFFSET, COUNT/SUM/AVG/MIN/MAX incl. COUNT(DISTINCT), INNER/LEFT JOIN → `$lookup`+`$unwind`, scalar functions and arithmetic; simple queries become `find`; subqueries/UNION unsupported; UI pending |
| MR-018 | Studio 3T | Query Code generation (JS, Python, Java, C#, Go, Rust, PHP, Ruby) | Partial | — | per-language literal and escaping tests | runnable find/aggregate snippets for the official drivers with typed BSON literals; generated JS/Python/Ruby/PHP/Go snippets checked with `node --check`, `py_compile`, `ruby -c`, `php -l`, `gofmt` during development (not in CI); UI pending |
| MR-019 | Studio 3T | Schema Explorer | Partial | — | analyzer unit test, live sampling test | random/first/last sampling with filter; per-field probability, type distribution, nulls, min/max/mean, histogram, string/array lengths, date range, top values, nested and array element paths; charts UI pending |
| MR-020 | Studio 3T | Import / Export (JSON, CSV, BSON, SQL) | Partial | — | format round-trip tests, CSV mapping/error tests, live export→import test | export JSON array/lines (relaxed, canonical, shell), CSV with column discovery, mongodump-compatible BSON, SQL INSERTs; import JSON/Extended JSON/shell syntax, BSON, CSV with field mapping and types; insert/upsert/drop-first with per-document failure report and cancellation; SQL import, wizard UI and task scheduler pending |
| MR-021 | Studio 3T | Table / Tree / JSON result views | Missing | — | — | planned |
| MR-022 | Studio 3T | Standalone app (`jetthing --app mongo-raabit`) | Missing | — | — | waits for the plugin API (#48) |
