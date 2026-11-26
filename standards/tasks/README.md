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

| File | For |
| --- | --- |
| `rust.toml` | Rust crates and Rust workspaces |
| `dioxus-web.toml` | Dioxus web apps |
| `perseus-web.toml` | Perseus / Sycamore sites |
| `bevy-game.toml` | Bevy games |
| `cloudflare-pages.toml` | Cloudflare Pages deploy projects |
| `cloudflare-worker.toml` | Rust Cloudflare Workers (workers-rs) |
| `docs.toml` | Docs-only projects |

A project can override or add tasks in its own `mise.toml` under `[tasks.<name>]`.
`mise run //:audit` (see `../scripts/audit.sh`) checks that every project follows this.
