# Shared mise tasks

Every project directory has a `mise.toml` that lists its tools and includes one
file from here, so every project answers the same commands:

| Task | Meaning |
| --- | --- |
| `ci` | Everything CI checks; makes no artifacts (format check, lint, build, test) |
| `fix` | Auto-fix formatting and lint where safe |
| `build` / `test` | Debug build / tests |
| `dev` | Run locally |
| `release` | Release build |
| `deploy` | Ship the release (deployable projects only; a no-op elsewhere) |
| `evals` | Turn the test reports `ci` saved in `target/evals-reports/` into eval results (`../scripts/evals.sh`, [evals.md](../docs/evals.md#in-ci)) |

| File | For |
| --- | --- |
| `rust.toml` | Rust crates and Rust workspaces |
| `dioxus-web.toml` | Dioxus web apps |
| `perseus-web.toml` | Perseus / Sycamore sites |
| `bevy-game.toml` | Bevy games |
| `cloudflare-pages.toml` | Cloudflare Pages deploy projects |
| `cloudflare-worker-assets.toml` | Rust Workers that serve a web app as static assets and run only for `/api/*` |
| `docs.toml` | Docs-only projects |

A project can override or add tasks in its own `mise.toml` under `[tasks.<name>]`.
`mise run //:audit` (see `../scripts/audit.sh`) checks that every project follows this.
