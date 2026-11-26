[![Nightly CI](https://github.com/jimhillr/monorepo/actions/workflows/nightly-ci.yml/badge.svg)](https://github.com/jimhillr/monorepo/actions/workflows/nightly-ci.yml)
[![Nightly CD](https://github.com/jimhillr/monorepo/actions/workflows/nightly-cd.yml/badge.svg)](https://github.com/jimhillr/monorepo/actions/workflows/nightly-cd.yml)

# Monorepo

Monorepo for all projects

# Getting Started

Only [mise](https://mise.jdx.dev/getting-started.html) needs to be installed
system wide. Each project lists its own tools (Rust, Node, dx, wrangler, ...)
in its `mise.toml`, and every project has the same tasks:

```bash
cd <project>
mise install      # install the project's tools
mise run ci       # format check, lint, build and test
mise run fix      # apply formatting and lint fixes
mise run dev      # run it locally
mise run release  # release build
mise run deploy   # deploy, for projects that deploy
```

From anywhere in the repo, `mise run //:audit` checks the repo standards and
`mise run '//...:ci'` runs CI in every project. See
[standards/docs/mise.md](standards/docs/mise.md).

# Contribute

- All issues are contained within the codebase as TODO's. Use todo tree extension to find work.

# Working with Git LFS Files

This repository uses Git LFS for large binary files (PDFs, archives, etc.). By default, LFS files are **not** downloaded automatically during clone to save bandwidth and storage.

To download LFS files when needed:
```bash
# Download all LFS files
git lfs fetch --all
git lfs checkout

# Download specific files
git lfs fetch --include="path/to/file.pdf"
git lfs checkout "path/to/file.pdf"

# Download files matching a pattern
git lfs fetch --include="*.pdf"
git lfs checkout
```
