# 0001. The website is a static export on Cloudflare Pages

- Status: Accepted
- Date: 2026-11-27
- Asked by: Jim ("Refactor sparkhill app backend to our new architecture on
  cloudflare", 2026-11-27)

## Context

sparkhill's only backend was the website's Perseus server: `perseus-axum`
with its default Axum server on Tokio (`#[perseus::main(perseus_axum::dflt_server)]`).
It was never deployed. The webapp (Blue Eel) is a client-only Dioxus app
with no backend; its letter patterns are bundled since PR 709, so it calls
no outside service.

The repo's Cloudflare architecture is:

- **Static sites on Cloudflare Pages**, deployed by direct upload from a
  `<project>/cloudflare` folder (`standards/tasks/cloudflare-pages.toml`),
  named `jimhillr-<project>`: cookbook, cahokia, pane, bear_down.
- **Rust Workers (`workers-rs`)** only for logic that must run at request
  time (`standards/tasks/cloudflare-worker.toml`: the telemetry Worker,
  PR 675, and the OpenTofu state Worker, decision 0009 in `standards/`).
- **D1** only when a deployed service must write data at runtime
  (decision 0004 in `standards/`).

The website renders one page, a translated tagline, with no request-time
data, no revalidation and no incremental generation.

## Options

| Option | What runs on Cloudflare | Money and limits | Claude Code effort |
| --- | --- | --- | --- |
| **Static export on Pages** (chosen) | Static files only | $0: 1 of the 25 budgeted Pages projects; static requests are free and unmetered; 0 Pages builds (direct upload) | This PR |
| Perseus server in a Worker | Not possible as is: perseus-axum needs Tokio and a TCP listener, which Workers (wasm32) don't have; it would mean a custom Perseus integration for `workers-rs` | $0 inside 100,000 requests a day, but every page view is a metered request | Several PRs, plus upkeep of an integration nobody else maintains |
| Rewrite the site in Dioxus, then deploy | Static files only | $0, as above | 1 to 2 PRs; Perseus is on Hold in the tech radar, so worth doing when the site grows |

## Decision

1. The website is **export-only**: `#[perseus::main_export]`, and
   `perseus-axum` is removed. Tokio stays, because the Perseus engine runs at
   build time (`perseus export`).
2. It deploys to the **Pages project `jimhillr-sparkhill`** from the new
   `sparkhill/cloudflare` project, at <https://jimhillr-sparkhill.pages.dev>,
   like the other `cloudflare` projects.
3. **No Worker and no database.** If sparkhill later needs request-time
   logic (a contact form, saved Blue Eel progress), it gets its own Rust
   Worker on `cloudflare-worker.toml`, and D1 only if it must store data
   (decision 0004), each in its own decision.

## Cost

$0 a month. One Pages project of the 25 budgeted (100 free), uploaded by
`wrangler pages deploy`, so no Pages builds; one deploy per push to `main`
that touches the website, plus the nightly rebuild. No Workers requests,
no new vendor, no payment method. Claude Code effort: this one PR.

## Consequences

- `perseus serve` (SSR) no longer works, so `mise run dev` in `website`
  runs `perseus export -s -w`, which serves the exported files as Pages
  will.
- The webapp is still not deployed; the root issue
  `pages-deploys-smoke-playwright-tests-blueeel.md` covers it, and it can
  share this Pages project under a sub-path or get its own.
