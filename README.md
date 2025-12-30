# Monorepo

Monorepo for all projects

# Prerequisites

- Rust: Recommend using [Rustup](https://www.rust-lang.org/tools/install) and the Rust vscode extension.
- Cargo: Recommend using [Rustup](https://www.rust-lang.org/tools/install) and the Rust vscode extension.
- Cast (cli for this monorepo): Run `cargo install --path ./cast/cli` and `code --install-extension ./cast/vscode_ext/cast.vsix`
- Additional Tooling: Run `cast toolchain install` inside the project directory

# Get Started

- To start a work session, use `cast session start`.
- To create a new project, use `cast project new`.
- To install tooling, use `cast toolchain install`.
- To execute ci checks, use `cast ci`.
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
