# Contributing to athenaeum

## Getting Started

### Install Toolchain

athenaeum is a docs-only project: it needs no toolchain beyond mise itself
(see the root AGENTS.md "Dev commands"):

```bash
cd athenaeum
mise install
```

### Build

There is nothing to build. The notes are Markdown, read on GitHub.

### Test

To run the project's CI task, as CI runs it (for a docs-only project it has
nothing to check yet):

```bash
mise run ci
```

`mise run //:audit` from the repo root checks the repo standards, this
project's README and CONTRIBUTING included.

## Adding a note

- Put study notes in `learning/<subject>/` and reference notes for building
  things here in `resources/<area>/`. Start a new folder with a `README.md`
  when no existing one fits.
- Link to articles, papers and books instead of committing copies of them.
- Keep notes short and current; when a note goes stale, fix it or delete it.
