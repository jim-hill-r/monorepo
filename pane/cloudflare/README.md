# pane/cloudflare

Pane's Cloudflare Worker, `jimhillr-pane`, live at
<https://jimhillr-pane.jim-hill-r.workers.dev>. It serves the Dioxus build of
[`../web`](../web) as [Workers static
assets](https://developers.cloudflare.com/workers/static-assets/) and runs
Rust code (workers-rs) only for `/api/*`. Until Phase 3 of the [review
dashboard plan](../docs/plans/2026-11-review-dashboard.md) pane was a
Cloudflare Pages project; the Worker replaces it so pane can have server
routes in Rust.

## How requests are served

| Path | Served by | Counts against the free plan |
| --- | --- | --- |
| `/api/*` | The Worker (`run_worker_first = ["/api/*"]` in `wrangler.toml`) | Yes, 100,000 requests a day |
| Files in the build (`/`, `/data/snapshot.json`, `/reviews/...`) | Static assets | No, free and unlimited |
| Any other path (`/projects/pane`) | `index.html` (`not_found_handling = "single-page-application"`), so the Dioxus router handles it | No |

The routes live in [`src/api.rs`](src/api.rs), free of the Workers runtime so
`cargo test` checks them natively; [`src/lib.rs`](src/lib.rs) only connects
them to the runtime. Today there is one:

| Route | Answer |
| --- | --- |
| `GET /api/health` | `{"status":"ok"}` |
| Any other `/api/*` | `404` `{"error":"no such API route"}` |

Phase 3 adds the cached `/api/github/*` routes and puts the whole site behind
Cloudflare Access, with every `/api/*` route checking the
`Cf-Access-Jwt-Assertion` JWT.

## Prerequisites

Install the tools with mise (see "Dev commands" in the root `AGENTS.md`):
Rust with the `wasm32-unknown-unknown` target, Node, Wrangler and
`worker-build`.

## Building

`mise run release` builds the web app (`mise run release` in `../web`, which
writes `../web/dist/public`) and then the Worker (`worker-build --release`,
which writes `build/worker/`). Both outputs are gitignored.

## Testing

`mise run ci` runs the format check, clippy natively and for wasm, the wasm
build, `cargo test` (the routes, and `tests/integration_tests.rs`, which
checks `wrangler.toml` stays a free-plan Worker with static assets) and
[`tests/e2e.rs`](tests/e2e.rs). The end-to-end test builds the Worker, serves
it with `wrangler dev` over a stand-in assets folder, and checks that pages,
data and client-side routes come from the assets and only `/api/*` reaches
the Worker.

## Deploying

Pushes to `main` deploy pane on every push (`# deploy-on-every-push` in
`mise.toml`, since its snapshot reads the whole repo). `mise run deploy`
builds as above, checks the Workers budget with
`standards/scripts/free-tiers.sh worker` and runs `wrangler deploy`, which
uploads the Worker and the assets together. CI deploys with the
`CLOUDFLARE_API_TOKEN` (Infra Mint's deploy token, which has Workers Scripts
Write) and `CLOUDFLARE_ACCOUNT_ID` secrets.

To run it locally after `mise run release`, use `mise run dev`
(`wrangler dev`).

## Cost

$0 a month on the Workers Free plan; see [`../docs/cost.toml`](../docs/cost.toml).

## Links

- [Workers static assets](https://developers.cloudflare.com/workers/static-assets/)
- [workers-rs](https://github.com/cloudflare/workers-rs)
- [Wrangler](https://developers.cloudflare.com/workers/wrangler/)
