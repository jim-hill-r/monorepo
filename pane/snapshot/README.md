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
| `projects[].status` | `# Priority Projects` and `# On Hold Projects` in the root `issues/README.md`; a sub-project takes its top-level project's status, and on hold wins |
| `work_items` (from issue files) | Every issue file (`issues/<slug>.md` or `<project>/issues/<slug>.md`, not `README.md`) with front matter: `kind` (`todo` or `fix`), the section from `priority` (`Priority Issues` or `Backlog`), `epic`, `tags`, the body as the text, the line the body starts on, and the owning project (the longest config root containing the file). |
| `work_items` (from code) | Every `TODO` / `FIX` that follows a comment opener (`//`, `#`, `/*`, `<!--`, `--`, `;`) in `.rs`, `.toml`, `.sh`, `.yml`, `.ts`, `.js`, `.py`, `.css`, `.html`, `.sql` and similar files. Openers inside a `"string"` or a `` `code span` `` are ignored. |
| `human_issues` | Each file in the root `issues/human/` (not `README.md`), with its body as the text and `added` and `unblocks` from its front matter. Pane works out the age from `taken_at`. |
| `deliverables` (live sites) | Each project with a `DEPLOY_URL` in its `[env]`. |
| `deliverables` (downloads) | Each project with a `RELEASE_TAG` in its `[env]` (see "Game builds" in `standards/docs/mise.md`): its GitHub Release, and the macOS and Android assets `<package>-macos.zip` and `<package>-android.apk`, where `<package>` is `[package].name` in its `Cargo.toml`. |
| `deliverables` (documents) | Every Markdown file (except `README.md`) in a `docs/plans/`, `docs/decisions/` or `docs/retros/` folder: title from its first `#` heading, status from `- Status:`, date from `- Date:`, `Started <date>`, or the file name, and the whole file as `body` for pane to render. |
| `reviews` | Each row of the `## Rotation` table in `docs/reviews/README.md` (`\| Date \| Project \| Deck \| PR \|`, columns found by header), newest first: project, date, the deck's `deck.md` path, the PR link if it points at the repo, and the deck's scorecard: each row of the `\| Area \| Rating \| Why \|` table under a heading that says Scorecard (columns found by header; none if the deck isn't in the repo yet). The `deep-project-review` skill writes both tables, so keep their shape. |
| `deliverables` (review decks) | Each review again, linking its slides `/reviews/<folder>/deck.html` and PDF `/reviews/<folder>/deck.pdf`, which pane's release builds. |
| `retros` | Each retro again, with its `- [ ]` / `- [x]` items under `## Actions`. |

Everywhere, `target/`, `node_modules/`, `dist/` and hidden directories are
skipped, as are code blocks in Markdown.

The types are in [`pane_model`](../model). The parsers in `src/parse.rs` are
pure functions over file contents; `src/repo.rs` walks the checkout.
