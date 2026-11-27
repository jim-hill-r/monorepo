# Contributing to sparkhill

## Getting Started

### Install Toolchain

The `sparkhill` folder itself is a docs-only project and needs no toolchain
beyond mise (see the root AGENTS.md "Dev commands"). Each project in it has
its own `mise.toml` and CONTRIBUTING.md:

- [website](website/CONTRIBUTING.md) and its deploy project
  [cloudflare](cloudflare/CONTRIBUTING.md)
- [webapp](webapp/CONTRIBUTING.md) (Blue Eel) and its deploy project
  [webapp_cloudflare](webapp_cloudflare/CONTRIBUTING.md)

Install a project's tools from its folder, for example:

```bash
cd sparkhill/webapp
mise install
```

### Build

Build from the project's folder with `mise run build`, or `mise run release`
for what a deploy ships.

### Test

Run `mise run ci` in each project you changed. In this folder it runs the
docs-only CI task, which has nothing to check yet.

## Recording decisions

Decisions that shape sparkhill go in `docs/decisions/` as numbered files
(`NNNN-short-title.md`), listed in its README.
