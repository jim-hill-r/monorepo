# Monorepo

Monorepo for all projects

# Getting Started

Only Rust and Npm need to be installed system wide. Everything else is managed by the monorepo cli

- Install Rust via [online instructions](https://rust-lang.org/tools/install/)
- Install npm via [nvm online instructions](https://www.nvmnode.com/guide/installation-sh.html)

Cast is the CLI tool for this monorepo. All developer commands use Cast.

- Install cast: `cargo install --path ./cast/cast_cli`

Once Cast is installed, you can use it for all other work in this monorepo.
- To start a work session, use `cast session start`.
- To create a new project, use `cast project new`.
- To install dependencies, use `cast install`.
- To execute ci checks, use `cast ci --check`.
- To fix ci issues, use `cast ci --fix`.
- To execute a ci release, use `cast ci --release`.
- To deploy a project, use `cast cd`.

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
