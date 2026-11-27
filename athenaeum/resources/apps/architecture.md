# App architecture

How apps in this repo are built today. The rules live elsewhere; this note
points to them rather than restating them.

- **Clients are static.** Web apps are built to static files and served from
  Cloudflare Pages, named `jimhillr-<project>`
  ([naming](../../../standards/docs/naming.md#cloudflare-pages-projects)).
  Server code, when an app needs it, runs on Cloudflare Workers.
- **Sign-in is Auth0 through [`auth_sdk`](../../../auth_sdk/README.md):** the
  OAuth 2.0 authorization code flow with PKCE for single-page and mobile
  apps.
- **Data is files first.** Results and data are files committed next to the
  project that owns them. When a project needs queries over them, use
  committed SQLite; a hosted database (Cloudflare D1) is the last resort, for
  data a deployed service writes at runtime. See "Files before databases" in
  [AGENTS.md](../../../AGENTS.md) and
  [decision 0004](../../../standards/docs/decisions/0004-database-engine.md).
- **Vendors and technologies** come from the
  [tech radar](../../../standards/docs/tech-radar.md). Anything not on it
  needs Jim's sign-off.

## Superseded

An earlier version of this note suggested using the database as the only
API, sending the sign-in JWT straight to SurrealDB or SpacetimeDB. Neither is
an approved vendor, and decision 0004 replaced that idea.
