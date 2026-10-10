# nexus-http

A clean-room, Insomnia-class API client that ships as a first-party Jetthing plugin
and will also run standalone (`jetthing --app nexus-http`). Tracking issue: #49.

| Crate | Purpose |
|---|---|
| `core` (`nexus-http-core`) | Headless engine with no UI dependency: data model, git-friendly `.nexus/` storage, `{{ variables }}` and `{% tags %}` templating with response chaining, auth (Basic, Bearer, API key, Digest, OAuth 2.0, AWS SigV4), request execution on reqwest + rustls with redirects, cookies, proxies, client certificates, timings and a timeline, JSONPath filters, assertions and response history. |

The UI crate (Islands-styled egui views) and importers/code generation follow in
later slices. Status and evidence: `docs/parity/nexus-http.md`.

## Storage layout

```text
.nexus/
  .gitignore                      ignores local/
  workspaces/<workspace-id>/
    workspace.json                name, settings, tree order, environment order
    items/<item-id>.json          one folder (without children) or request
    environments/base.json
    environments/<env-id>.json    secret values are blanked here
  local/<workspace-id>/           machine-local, not committed
    secrets.json  cookies.json  history.json
```

## Templates

`{{ host }}`, `{{ api.version }}`, `{{ users[0].id }}` and `{{ _.token }}` read the
merged variables (base environment < sub environment < folder variables). Tags:
`{% uuid %}`, `{% now 'iso'|'millis'|'unix'|'date'|'time' %}`,
`{% base64 'encode', value %}`, `{% urlencode value %}`, `{% hash 'sha256', value %}`,
`{% random_int 1, 10 %}`, `{% env 'NAME' %}` and
`{% response 'body', 'req_id', '$.token' %}` for chaining.

## Tests

`cargo test -p nexus-http-core` runs unit tests (including the RFC 2617 Digest,
RFC 7636 PKCE and AWS SigV4 `get-vanilla` vectors) and end-to-end tests against a
local HTTP server.
