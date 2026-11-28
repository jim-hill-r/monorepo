# pane

Pane is one place for Jim to review the work going on in this repo: what
needs him (human issues, `needs-human` PRs, PRs waiting for review, new
review decks), every deliverable with a link or a download, a red, amber or
green health light per project and shared system with the evidence behind
it, and each project's cost and revenue. It is named for a "single pane of
glass". Pane only reads the repo and proposes changes as pull requests, so
the repo stays the source of truth. The plan is
[docs/plans/2026-11-review-dashboard.md](docs/plans/2026-11-review-dashboard.md)
and the requirements are in [docs/requirements.md](docs/requirements.md).

## Layout

| Crate | Package | Does |
| --- | --- | --- |
| [`model/`](model/README.md) | `pane_model` | The serde types `snapshot` writes and `web` reads. |
| [`snapshot/`](snapshot/README.md) | `pane_snapshot` | Reads a checkout of the repo and writes `snapshot.json`. |
| [`web/`](web/README.md) | `pane_web` | The Dioxus web app that shows the snapshot. |
| [`cloudflare/`](cloudflare/README.md) | `pane_cloudflare` | Deploys the web app to Cloudflare. |

Open work is in [docs/issues/](docs/issues/), and how to build and test pane is in
[CONTRIBUTING.md](CONTRIBUTING.md).
