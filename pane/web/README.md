# pane_web

Pane's web app: one place to review the monorepo (see the
[review dashboard plan](../docs/plans/2026-11-review-dashboard.md)). It loads
`/data/snapshot.json`, which the release builds with
[`pane_snapshot`](../snapshot), and shows it with a left nav that folds into
one column under 720 px.

| Route | Page |
| --- | --- |
| `/` | Today: counts of projects, open TODOs and FIXes, and human issues |
| `/projects` | Every project, priority first, with type, status, README summary and open items |
| `/projects/<path>` | One project, such as `/projects/pane/web` |
| `/health`, `/deliverables`, `/reviews`, `/human-issues`, `/work`, `/add` | Placeholders naming the phase that fills them in |

The footer shows the snapshot's commit and how long ago it was taken. Pure
helpers for the pages (dates, ages, sorting, counts) are in `src/view.rs`
with unit tests.

To try it locally with real data, run `mise run release` (it builds the
snapshot into `public/data/` first), or run `pane_snapshot` once and then
`dx serve`:

```bash
cargo run -p pane_snapshot -- --out public/data/snapshot.json
dx serve --platform web --package pane_web
```

"Pane" evokes a window pane — a transparent surface through which you view the world — making it an apt metaphor for a web interface that presents information clearly and directly to the user.

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
the bundle as `/data/snapshot.json`. `public/` is gitignored, so the snapshot
is never committed. The output will be in the `dist/` directory and ready for
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
│   └── main.rs      # Main application entry point
├── Cargo.toml       # Rust dependencies and project metadata
├── Dioxus.toml      # Dioxus build configuration
├── mise.toml        # Tools and standard tasks (mise run ci, dev, ...)
└── README.md        # This file
```

## Dependencies

This project uses Dioxus 0.7 for building web applications with Rust.

## Status

This is still a placeholder. Pane is becoming one place to review the whole
monorepo; see the plan in
[../docs/plans/2026-11-review-dashboard.md](../docs/plans/2026-11-review-dashboard.md)
and the phased TODOs in [../ISSUES.md](../ISSUES.md).
