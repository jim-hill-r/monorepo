# Contributing to bogan

## Getting Started

### Install Toolchain

bogan is a docs-only project for now: it needs no toolchain beyond mise
itself (see the root AGENTS.md "Dev commands"):

```bash
cd bogan
mise install
```

### Build

There is nothing to build yet. Design docs are Markdown, read on GitHub.

### Test

To run the project's CI task, as CI runs it (the docs-only task from
`standards/tasks/docs.toml`, which has nothing to check yet):

```bash
mise run ci
```

`mise run //:audit` from the repo root checks the repo standards, this
project's README and CONTRIBUTING included.

## Adding a design doc

Put new design docs, plans and decisions in `docs/`, as Markdown.
