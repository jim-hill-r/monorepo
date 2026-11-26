# Toolchain Management

How development tools are installed and run across the monorepo. The rules
for the task runner itself are in [mise.md](mise.md).

## One tool manager: mise

Every tool a project needs is listed in the `[tools]` table of its
`mise.toml`, and `mise install` (in the project directory) installs exactly
those versions. CI does the same through `jdx/mise-action`, so local and CI
toolchains match.

| Need | `[tools]` entry |
| --- | --- |
| Rust (stable, with rustfmt and clippy) | `rust = { version = "stable", components = "rustfmt,clippy" }` |
| Rust for the web (wasm) | add `targets = "wasm32-unknown-unknown"` |
| Dioxus CLI | `"cargo:dioxus-cli" = "<dioxus version>"` plus `cargo-binstall = "latest"` |
| Perseus CLI | `"cargo:perseus-cli" = "0.4.2"` |
| Node.js | `node = "24"` |
| Wrangler | `"npm:wrangler" = "4"` |

System packages that mise does not manage (for example LLVM for my_lang, or
the audio and windowing libraries the Bevy games need) are installed by the
project's `ci:deps` task in CI. Locally, install them with your package
manager; the project's README or CONTRIBUTING lists them.

Git LFS is needed for the repository's large files: install it with your
package manager and run `git lfs install` once.

## Pinning versions

- Pin every tool except Rust stable to a version or major series in
  `mise.toml`.
- `dioxus-cli` MUST match the `dioxus` crate version in the project's
  `Cargo.lock`; `dx` refuses to run otherwise.
- Playwright comes from the project's `package-lock.json` (`npm ci`); its
  browsers are installed by the `test` and `ci` tasks.
- Change a version in a PR, and let the PR's CI run the affected projects.

## Use the standard tasks

Run work through the standard tasks (`mise run ci`, `fix`, `build`, `test`,
`dev`, `release`, `deploy`) rather than calling the underlying tools in
scripts or config. For example, Playwright's `webServer` runs
`mise run dev`, not `dx serve`. Calling a tool directly is fine for
interactive debugging or a flag the tasks do not expose.

## Platforms

- CI runs on `ubuntu-24.04` (Playwright cannot install browsers on 26.04 yet).
- macOS works the same way; use `brew` for system packages.
- Windows is not a supported platform today.

## Related documentation

- [mise.md](mise.md): rules, CI/CD and decisions
- [`standards/tasks/`](../tasks/README.md): the shared task files
- `.github/workflows/README.md`: the CI/CD workflows
- `AGENTS.md`: Dev commands
