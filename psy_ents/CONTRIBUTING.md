# Contributing to psy_ents

## Getting Started

### Install Toolchain

psy_ents is a docs-only project: it needs no toolchain beyond mise itself
(see the root AGENTS.md "Dev commands"):

```bash
cd psy_ents
mise install
```

### Build

There is nothing to build. Each story is a Marp deck (`marp: true` front
matter, the `uncover` theme). Preview one with the Marp VS Code extension, or:

```bash
npx @marp-team/marp-cli "The Psy Ents.md" --preview
```

### Test

To run the project's CI task, as CI runs it (the docs-only task from
`standards/tasks/docs.toml`, which has nothing to check yet):

```bash
mise run ci
```

`mise run //:audit` from the repo root checks the repo standards, this
project's README and CONTRIBUTING included.
