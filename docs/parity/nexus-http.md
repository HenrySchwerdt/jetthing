# nexus-http parity

nexus-http is the first-party API client plugin (issue #49). It is a clean-room
implementation of the public, observable feature set of Insomnia-class API clients;
no Insomnia code, artwork or branding is used. The reference is Insomnia's public
documentation and observable behavior.

| ID | Reference | Feature | Status | Commands | Tests | Limitations / evidence |
|---|---|---|---|---|---|---|
| NEX-001 | Insomnia | Workspaces, folders, requests (model + storage) | Partial | — | `storage` round-trip, secrets, stale-file and orphan tests | engine only: one JSON file per request/folder/environment under `.nexus/`, tree order in `workspace.json`, atomic writes, unchanged files untouched; UI, drag reordering and search pending |
| NEX-002 | Insomnia | HTTP request execution | Partial | — | `tests/server.rs` against a local server | any method, query params, headers (folder headers inherited), bodies: JSON, XML, text, form-urlencoded, multipart with files, binary file, GraphQL; gzip/brotli/deflate; cancellation between phases; UI pending |
| NEX-003 | Insomnia | Transport settings | Partial | — | redirect, timeout, connection-refused and truncation tests | manual redirect following (303/301/302 → GET, 307/308 keep body, credentials dropped cross-origin, max count), per-request timeout/TLS verify, HTTP/SOCKS proxy with no-proxy list, extra CA bundle, per-host PEM client certificates, response size limit; settings UI pending |
| NEX-004 | Insomnia | Cookie jar per workspace | Partial | — | RFC 6265 domain/path/secure/expiry unit tests, server round trip | stored in `.nexus/local/`; cookie editor UI pending |
| NEX-005 | Insomnia | Environments and template tags | Partial | — | template unit tests | base + sub environment + folder variables with deep merge, `{{ a.b[0] }}` paths, nested rendering with cycle detection, tags `uuid`, `now`, `base64`, `urlencode`, `hash`, `random_int`, `env`; secrets kept out of git; variable autocomplete data available, UI pending |
| NEX-006 | Insomnia | Response chaining | Partial | — | history + server chaining tests | `{% response 'body', 'request-id', '$.token' %}` reading body (JSONPath), header, status or URL of the latest response; XPath and "send when stale" behavior pending |
| NEX-007 | Insomnia | Auth: Basic, Bearer, API key, Digest, AWS SigV4 | Partial | — | RFC 2617 Digest vector, AWS `get-vanilla` SigV4 vector, server tests | inheritance through folders; API key in header/query/cookie; NTLM not implemented |
| NEX-008 | Insomnia | Auth: OAuth 2.0 | Partial | — | RFC 7636 PKCE vector, token request/redirect parsing, loopback redirect, client-credentials server test | authorization code (+PKCE), client credentials, password, implicit; loopback redirect capture; automatic refresh and UI pending |
| NEX-009 | Insomnia | Response viewer | Missing | — | — | engine provides pretty JSON, body kind detection, JSONPath filter, timeline and timings; UI pending |
| NEX-010 | Insomnia | Response history | Partial | — | bounded history tests | last 20 responses per request in `.nexus/local/`; UI pending |
| NEX-011 | Insomnia | Assertions and collection runner | Partial | — | assertion unit tests | status/header/JSONPath/body/time/size checks; runner pending |
| NEX-012 | Insomnia | Import and export (curl, Insomnia v4/v5, Postman v2.1, OpenAPI 3) | Partial | — | importer unit tests per format, export round trip | format auto-detection; folders, environments, auth, bodies, disabled rows, Insomnia base64 response-tag filters and Postman dynamic variables mapped; OpenAPI tags → folders, `$ref`, examples or schema-generated bodies, security schemes; single-file nexus-http export with secrets blanked; Swagger 2.0, Postman v1, gRPC/WebSocket requests and Insomnia export are not supported; import UI pending |
| NEX-013 | Insomnia | Code generation | Partial | — | per-target snippet tests, curl snippet re-imports to the same request | curl, Python requests, JavaScript fetch, Go net/http, Rust reqwest, Java HttpClient, Kotlin OkHttp from the rendered request; multipart is emitted as its raw encoded body, binary bodies are omitted with a note; UI pending |
| NEX-014 | Insomnia | GraphQL | Partial | — | GraphQL body encoding test | query/variables/operation body; introspection and autocomplete pending |
| NEX-015 | Insomnia | WebSocket, Server-Sent Events, gRPC | Missing | — | — | planned |
| NEX-016 | Insomnia | Plugin packaging and standalone `--app nexus-http` | Missing | — | — | waits for the plugin API (#48) |
| NEX-017 | Insomnia | Islands-styled UI (light/dark, keyboard-first) | Missing | — | — | screenshots will live in `evidence/nexus-http/` |
