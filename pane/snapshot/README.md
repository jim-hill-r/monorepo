# pane_snapshot

Reads a checkout of the monorepo and writes the `snapshot.json` that
[pane](../) shows. It is named for what it makes: a snapshot of the repo at
one commit, built fresh on every deploy and never committed.

## Usage

```bash
cargo run -p pane_snapshot -- --out snapshot.json
```

| Flag | Default |
| --- | --- |
| `--repo DIR` | The git top level of the current directory |
| `--out FILE` | `snapshot.json` (parent directories are created) |
| `--commit SHA` | `git rev-parse HEAD` in the repo |
| `--taken-at TIME` | Now, UTC, RFC 3339 |

It exits non-zero, writing nothing, if the root `mise.toml` has no
`[monorepo].config_roots`.

## What it reads

| Snapshot field | Source |
| --- | --- |
| `projects[].path` | Each entry in the root `mise.toml`'s `[monorepo].config_roots` |
| `projects[].kind` | The `<type>` of the project's `standards/tasks/<type>.toml` include (`unknown` if none) |
| `projects[].deploy_url` | `DEPLOY_URL` in the project's `[env]` |
| `projects[].summary` | The first prose paragraph of the project's README (headings, badges, quotes, lists, tables and code skipped) |
| `projects[].status` | `# Priority Projects` and `# On Hold Projects` in the root `ISSUES.md`; a sub-project takes its top-level project's status, and on hold wins |
| `work_items` | Every `- TODO` / `- FIX` list item in every `ISSUES.md`, with its file, line, `#` section, `##`/`###` epic (joined as `Epic / Phase`), tags and owning project (the longest config root containing the file). `target/`, `node_modules/`, `dist/` and hidden directories are skipped. |

The types are in [`pane_model`](../model). The parsers in `src/parse.rs` are
pure functions over file contents; `src/repo.rs` walks the checkout.

Human issues, code comments, plans, decisions and retros come next (see
[pane/ISSUES.md](../ISSUES.md)).
