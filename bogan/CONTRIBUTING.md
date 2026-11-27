# Contributing to bogan

## Getting Started

Bogan is a docs-only project today: a README, cost and threat notes, and
issue files, with no code yet. The first code waits on the product brief
(`issues/write-product-brief.md`).

### Install Toolchain

Bogan needs no toolchain beyond [mise](https://mise.jdx.dev) itself (see
"Dev commands" in the root [AGENTS.md](../AGENTS.md)). Set up the git hooks
once per clone, then install the project's tools:

```bash
mise run //:hooks
cd bogan
mise install
```

### Build

There is nothing to build. The docs are Markdown, read on GitHub.

### Test

To run the project's CI task, as CI runs it:

```bash
mise run ci
```

It runs the shared docs-only tasks in
[standards/tasks/docs.toml](../standards/tasks/docs.toml), which check
nothing yet. `mise run //:audit` from the repo root checks the repo
standards, this project's README and CONTRIBUTING included.

## Adding docs

- Put design docs in `docs/`, and plans in `docs/plans/` (plans live in the
  repo, per the root [AGENTS.md](../AGENTS.md)).
- Track work as one file per issue in `issues/`, in the format in
  [standards/docs/issue-management.md](../standards/docs/issue-management.md).
- Keep `docs/cost.toml` current in any change that adds a service, a deploy
  or a cost ([standards/docs/cost.md](../standards/docs/cost.md)).
