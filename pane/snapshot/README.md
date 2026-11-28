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
| `--audit FILE` | None: the saved output of `mise run //:audit`, which pane's release writes to `pane/web/target/audit.txt`. Without it `audit` is null and Standards is grey. |

It exits non-zero, writing nothing, if the root `mise.toml` has no
`[monorepo].config_roots`, if any project's `docs/cost.toml`,
`standards/radar/radar.json` or `standards/radar/limits.toml` doesn't parse
(so a bad file never shows up in pane as missing money), or if `--taken-at`
doesn't start with `YYYY-MM`.

## What it reads

| Snapshot field | Source |
| --- | --- |
| `projects[].path` | Each entry in the root `mise.toml`'s `[monorepo].config_roots` |
| `projects[].kind` | The `<type>` of the project's `standards/tasks/<type>.toml` include (`unknown` if none) |
| `projects[].deploy_url` | `DEPLOY_URL` in the project's `[env]` |
| `projects[].summary` | The first prose paragraph of the project's README (headings, badges, quotes, lists, tables and code skipped) |
| `projects[].status` | `# Priority Projects` and `# On Hold Projects` in `standards/docs/issues/README.md`; a sub-project takes its top-level project's status, and on hold wins |
| `work_items` (from issue files) | Every issue file (`<project>/docs/issues/<slug>.md`, not `README.md`) with front matter: `kind` (`todo` or `fix`), the section from `priority` (`Priority Issues` or `Backlog`), `epic`, `tags`, the body as the text, the line the body starts on, the owning project (the longest config root containing the file), and `added` as the date. |
| `work_items` (from code) | Every `TODO` / `FIX` that follows a comment opener (`//`, `#`, `/*`, `<!--`, `--`, `;`) in `.rs`, `.toml`, `.sh`, `.yml`, `.ts`, `.js`, `.py`, `.css`, `.html`, `.sql` and similar files. Openers inside a `"string"` or a `` `code span` `` are ignored. |
| `human_issues` | Each file in every project's `docs/issues/human/` (not `README.md`), with its project (the longest config root containing it), its body as the text and `added` and `unblocks` from its front matter. Pane works out the age from `taken_at`. |
| `deliverables` (live sites) | Each project with a `DEPLOY_URL` in its `[env]`. |
| `deliverables` (downloads) | Each project with a `RELEASE_TAG` in its `[env]` (see "Game builds" in `standards/docs/mise.md`): its GitHub Release, and the macOS and Android assets `<package>-macos.zip` and `<package>-android.apk`, where `<package>` is `[package].name` in its `Cargo.toml`. |
| `deliverables` (documents) | Every Markdown file (except `README.md`) in a `docs/plans/`, `docs/decisions/` or `docs/retros/` folder: title from its first `#` heading, status from `- Status:`, date from `- Date:`, `Started <date>`, or the file name, and the whole file as `body` for pane to render. |
| `reviews` | Each row of the `## Rotation` table in `docs/reviews/README.md` (`\| Date \| Project \| Deck \| PR \|`, columns found by header), newest first: project, date, the deck's `deck.md` path, the PR link if it points at the repo, and the deck's scorecard: each row of the `\| Area \| Rating \| Why \|` table under a heading that says Scorecard (columns found by header; none if the deck isn't in the repo yet). The `deep-project-review` skill writes both tables, so keep their shape. |
| `deliverables` (review decks) | Each review again, linking its slides `/reviews/<folder>/deck.html` and PDF `/reviews/<folder>/deck.pdf`, which pane's release builds. |
| `retros` | Each retro again, with its `- [ ]` / `- [x]` items under `## Actions`. |
| `costs.projects` | Each project's `docs/cost.toml` (`standards/docs/cost.md`), parsed by `cost_model::parse_cost_file`: summary, reviewed date, services with plan and expected usage (each per-project limit with its ceiling from `limits.toml`), the `[[cost]]` and `[[revenue]]` lines, and revenue and direct cost for the month of `taken_at`. A project without the file is left out. |
| `costs.history` | Every top-level project's money for the twelve months ending with the month of `taken_at`: revenue and direct cost from its `docs/cost.toml`, and its share of the month's shared costs (every radar vendor's monthly budget, today's for every month) by the PRs merged that month. A merged PR is a first-parent commit whose subject ends `(#N)` or starts `Merge pull request #`; it counts for each top-level project whose folder it changed (`git log --first-parent --name-only`). `costs.history_from_git` is false in a shallow clone or outside git, when nothing is split; pane's release fetches the history first. Each service, cost and revenue line also has the `line` of its header, for links. |
| `costs.account` | Each account-wide limit in `standards/radar/limits.toml`: every project's expected use summed and held against the budget, or the free limit when there is none (COST-005). |
| `costs.budgets` | Each vendor blip in `standards/radar/radar.json` with its monthly budget and what projects' own costs spend with it this month (COST-006), then any vendor a cost names that the radar doesn't list. Without `standards/radar/` there are no limits and no radar vendors. |
| `thresholds` | `pane/health.toml`, so the web app lights signals with the same thresholds. Missing keys take the plan's values; an unknown key fails the snapshot. The plan's values when the file isn't there. |
| `audit` | The `--audit` file: each `error[CODE]` and `warning[CODE]` line with its project (the longest config root containing the first word of the message, `:line` suffixes dropped) and file, or none for a repo-wide finding. Null when the file has no closing `audit: ok` or `audit: N error(s)` line, so an audit that crashed never shows as clean. A file that can't be read fails the snapshot. |

Everywhere, `target/`, `node_modules/`, `dist/` and hidden directories are
skipped, as are code blocks in Markdown.

The types are in [`pane_model`](../model). The parsers in `src/parse.rs` are
pure functions over file contents; `src/repo.rs` walks the checkout.
