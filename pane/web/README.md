# pane_web

Pane's web app: one place to review the monorepo (see the
[review dashboard plan](../docs/plans/2026-11-review-dashboard.md)). It loads
`/data/snapshot.json`, which the release builds with
[`pane_snapshot`](../snapshot), and shows it with a left nav that folds into
one column under 720 px.

| Route | Page |
| --- | --- |
| `/` | Today: counts, human issues waiting on Jim oldest first, and what is new since the last visit (human issues, deliverables, TODOs and FIXes; see below) |
| `/approve` | Approve: green PRs waiting on Jim's review, clear ones first, each with its summary, attention badges and an "Approve on GitHub" link to its review form, then red-labelled PRs and stacks. It reads GitHub's public API from the browser with no token; the rules are `pane_model::queue` ([decision 0014](../../standards/docs/decisions/0014-faster-pr-approvals.md)) |
| `/acknowledge` | Acknowledge: every PR merged since the first visit, newest first, with its summary and attention badges, until Jim acknowledges it (one or all, kept in `localStorage` as `pane.acks.v1`); "Flag for follow-up" opens GitHub's editor with a `fix` issue file for it. Rules in `pane_model::acks` ([decision 0014](../../standards/docs/decisions/0014-faster-pr-approvals.md)) |
| `/projects` | Every project, priority first, with type, status, README summary and open items |
| `/projects/<path>` | One project, such as `/projects/pane`: status, type, README summary, live URL, sub-projects, latest plan, decision and retro, its latest review and its scorecard counts, its deliverables, its open TODOs and FIXes (sub-projects included), and its Money: this month's revenue, direct cost, shared cost and net, its services and plans, and its own cost and revenue lines from `docs/cost.toml` |
| `/money` | Money (`src/money.rs`): this month's revenue, direct cost, shared-cost share and net for each project with any, a totals row, the total net for the last 12 months as a bar chart (and as a table), and each vendor's monthly radar budget against projects' spend. Every number links to its line in `docs/cost.toml` or to `standards/radar/radar.json` on GitHub |
| `/work` | Every `TODO` and `FIX` from issue files and code comments, FIXes first, filterable by project, kind, where it sits (priority, backlog, other, code comments) and tag, each linking to its line on GitHub |
| `/human-issues` | Each open human task, gathered from every project's `docs/issues/human/`, oldest first, with its project, age, what it unblocks and a link to the file on GitHub |
| `/deliverables` | Live sites (`DEPLOY_URL`), game downloads (`RELEASE_TAG`), review decks, and plans, decisions and retros |
| `/docs/<path>` | One plan, decision or retro rendered from its Markdown (`src/markdown.rs`), with a link to the file on GitHub |
| `/reviews` | The review rotation from `docs/reviews/README.md`, newest first: date, project, the deck's slides and PDF, its scorecard's ratings counted (such as 2 Good · 4 Fair · 1 Poor), and its pull request |
| `/review/<folder>` | One review deck, such as `/review/2026-11-25-cookbook`: its slides in the page (arrow keys, `f` or the Full screen button), its PDF, its pull request for comments, `deck.md` on GitHub, and its scorecard as a table (area, rating, why) |
| `/health`, `/add` | Placeholders naming the phase that fills them in |

It uses the [design system](../../design_system/README.md)'s professional
theme: `design_system_dioxus::Stylesheets` loads the tokens, and
`assets/main.css` uses only `--ds-` tokens, so the theme picker in the footer
can switch it to any theme, light, dark or accessible mode.

The footer shows the snapshot's commit and how long ago it was taken. Pure
helpers for the pages (dates, ages, sorting, counts, filters, GitHub links)
are in `src/view.rs` with unit tests.

"New since your last visit" is in `src/visits.rs`. Items have no ids or
dates, so pane keeps a short hash of each one it showed (kind, project and
text, or a document's path) with the snapshot's commit, in `localStorage`
under `pane.visits.v1`. It keeps two visits: reloads of the same snapshot
keep comparing with the visit before, and a newer snapshot moves the last
visit into its place. Without `localStorage` the page still works and says
nothing is marked new. A changed line of text counts as new.

Documents are rendered with `pulldown-cmark`. Raw HTML in a document is
shown as text, `javascript:`, `vbscript:` and `data:` links are dropped, and
relative links point at the file on GitHub at the snapshot's commit. Every
GitHub link uses that commit, so line numbers match what pane shows.

To try it locally with real data, run `mise run release` (it builds the
snapshot into `public/data/` first), or run `pane_snapshot` once and then
`dx serve`:

```bash
cargo run -p pane_snapshot -- --out public/data/snapshot.json
dx serve --platform web --package pane_web
```

"Pane" evokes a window pane — a transparent surface through which you view the world — making it an apt metaphor for a web interface that presents information clearly and directly to the user.

## Tests

`cargo test` runs the unit tests in `src/view.rs` and `src/visits.rs`. The Playwright tests in
`tests/` check the app in Chromium: the home page loads the snapshot, the
main nav opens every page, a project page opens and shows its sections,
Today shows what is new against a stored older visit (and works without
`localStorage`), the Reviews page opens a deck as slides with its PDF, pull
request and full screen (locally the slides are stubbed, since `dev` doesn't
build the decks), and no page logs a console error. They start the app with `mise run dev`, which builds the snapshot
first, and run in CI as part of `mise run ci`.

```bash
npm ci
npm test            # every test, in Chromium
npm run smoke       # only the @smoke tests, recording video, screenshots and traces
BASE_URL=https://jimhillr-pane.jim-hill-r.workers.dev npm run smoke   # against the live site instead
```

## Prerequisites

Before running this project, you need to have:
- Rust toolchain installed (visit [rustup.rs](https://rustup.rs))
- Dioxus CLI (`dx`) installed

### Installing Dioxus CLI

Install the Dioxus CLI tool:

```bash
cargo install dioxus-cli
```

## Running in Development Mode

To run the application locally in development mode with hot-reload:

```bash
dx serve
```

This will:
- Start a local development server (typically at `http://localhost:8080`)
- Enable hot-reload, so changes to your code will automatically refresh the browser
- Provide detailed error messages and debugging information

### Development Server Options

You can customize the development server behavior:

```bash
# Serve on a specific port
dx serve --port 3000

# Open the browser automatically
dx serve --open

# Enable verbose logging
dx serve --verbose
```

## Building

To build the project for production:

```bash
mise run release
```

This first runs [`pane_snapshot`](../snapshot) to write
`public/data/snapshot.json` (a snapshot of the whole repo at the current
commit, with the time it was taken), then `dx bundle`, which copies it into
the bundle as `/data/snapshot.json`. Next, `mise run reviews` builds every
review deck in [`docs/reviews/`](../../docs/reviews) to HTML and PDF with
[`review-decks.sh`](../../standards/scripts/review-decks.sh), into
`public/reviews/<folder>/` (`deck.html`, `deck.pdf` and the deck's images),
so the site serves them under `/reviews/<folder>/`. That step needs Node and a
Chrome or Chromium (set `CHROME_PATH` if Marp can't find one); `dev` skips it.
`public/` is gitignored, so neither the snapshot nor the decks are ever
committed. The deck page embeds `/reviews/<folder>/deck.html` (Marp's own
slideshow) in an iframe; its route is `/review/<folder>`, not
`/reviews/<folder>`, so it never shadows the built files. The output will be in the `dist/` directory and ready for
deployment.

**Note:** The `Dioxus.toml` configuration disables wasm-opt (by setting level to 0) to avoid SIGABRT errors during release builds. This is a known issue where wasm-opt can crash on certain builds.

To build the project using cargo directly:

```bash
cargo build
```

To check the project without building:

```bash
cargo check
```

## Project Structure

```
pane/
├── src/
│   ├── main.rs      # App, routes and pages
│   ├── view.rs      # Pure helpers for the pages, with unit tests
│   └── markdown.rs  # Renders plans, decisions and retros
├── tests/           # Playwright tests (smoke.spec.ts)
├── Cargo.toml       # Rust dependencies and project metadata
├── Dioxus.toml      # Dioxus build configuration
├── mise.toml        # Tools and standard tasks (mise run ci, dev, ...)
└── README.md        # This file
```

## Dependencies

This project uses Dioxus 0.7 for building web applications with Rust, and
`pulldown-cmark` (HTML only, no default features) to render documents.

## Status

Phases 0 and 1 of the plan: the shell, the Today, Projects and Project, Work, Human issues and Deliverables pages, and smoke tests. Pane redeploys on every push to `main` (`# deploy-on-every-push` in `../cloudflare/mise.toml`). Pane is becoming one place to review the whole
monorepo; see the plan in
[../docs/plans/2026-11-review-dashboard.md](../docs/plans/2026-11-review-dashboard.md)
and the phased issues in [../docs/issues/](../docs/issues/).
