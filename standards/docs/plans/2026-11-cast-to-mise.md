# Plan: replace Cast with mise

Started 2026-11-26. Rules and decisions that come out of this plan are kept in
[../mise.md](../mise.md).

Cast is removed in four PRs, and every project gets its own `mise.toml` that
includes a shared task file for its type, so `mise run ci` means the same thing
everywhere. CI becomes one job per changed project, run in parallel.

## Where we started

- No root Rust workspace. Six projects are Cargo workspaces of their own:
  cahokia, cookbook, pane, luggage, cast and monorepo.
- The only cross-project link is `auth_sdk`, used by cahokia and cookbook
  through `../auth_sdk` path dependencies.
- Cast (about 14,000 lines in `cast/core`, plus a CLI and a VS Code extension)
  detected what to run from `Cargo.toml` and a `framework` key in 29
  `Cast.toml` files.
- CI built Cast, then ran `cast ci --only-changed --check --recursive 2
  --headless` in one job. Main CI ran `--release` and committed `*/artifacts/*`
  back to main. CD triggered on those commits and ran `wrangler pages deploy`
  for the three Cloudflare Pages projects. Main CI was already red.

## Target design

- Each project directory owns a `mise.toml` with its tools and one include of
  `standards/tasks/<type>.toml`. Types: rust, dioxus-web, perseus-web,
  bevy-game, cloudflare-pages, docs.
- Standard tasks in every project: `ci`, `fix`, `build`, `test`, `dev`,
  `release`, `deploy`, plus a `ci:deps` hook for system packages.
- The root `mise.toml` installs nothing. It enables mise's monorepo mode and
  holds `//:audit`, `//:affected` and `//:projects`.
- CI: a plan job runs the audit and lists affected projects, then a matrix runs
  `mise run ci` once per project. Main runs `ci`, `release`, then `deploy`
  where a project has one. Nightly runs every project.
- `standards/` becomes the standards docs plus the shared task files, with no
  Rust CLI. The nightly ISSUES.md audit is replaced by the `//:audit` PR check.

## Migration order

1. **Add mise beside Cast** (PR #557). Shared task files, a `mise.toml` per
   project, and a non-blocking `pull-request-mise-ci.yml`.
2. **Switch CI/CD to mise.** Replace the PR, main and nightly workflows and the
   artifacts-triggered CD; delete the nightly audit and the commit-artifacts
   step; drop the Cast-specific workflow tests in `monorepo/workflow_tests`.
3. **Remove Cast.** Delete `cast/`, every `Cast.toml`, `.cast` in
   `.gitignore`, the standards Rust CLI, committed artifacts and the Cast
   entries in `.vscode/settings.json`. Rewrite the AGENTS.md "Dev commands"
   section, README, REPOSITORY_STRUCTURE.md, the CONTRIBUTING files and
   `standards/docs/`.
4. **Make every project green** in mise CI, or note bigger fixes in the
   project's ISSUES.md.

## Decisions (Jim, 2026-11-26)

All four went with the recommendation:

- Stop committing build artifacts.
- Keep `auth_sdk` as a path dependency for now.
- Keep per-project Cargo workspaces.
- Replace the nightly standards audit with a PR check.

## Learned in PR 1

- Cast ran clippy and build without `--all-targets`, so test code was never
  linted. The task files match that for now.
- `ubuntu-latest` is 26.04, where Playwright 1.57 cannot install browsers, so
  mise CI runs on `ubuntu-24.04`.
- `dx` refuses to run unless its version matches the `dioxus` crate, so each
  app pins `dioxus-cli` to its locked or resolved dioxus version.
- my_lang needs LLVM 18 (inkwell), installed through its `ci:deps` task.

## Learned in PR 2

- The Cast CI followed changed projects only by directory, so a change to
  `cookbook/core` never re-tested `cookbook/web`. `affected.sh` now follows
  Cargo path dependencies and `# affected-by:` lines too.
- The web e2e tests run in Chromium, Firefox and WebKit, so CI installs all
  three Playwright browsers, as Cast did.
- Deploys now need the `CLOUDFLARE_API_TOKEN` and `CLOUDFLARE_ACCOUNT_ID`
  repository secrets; the Cast CD workflow passed none.

## Learned in PR 3

- The removed `standards` CLI also checked naming, documentation and Cargo
  lint rules that `audit.sh` does not cover yet. About 20 projects lack a
  README.md or CONTRIBUTING.md, so those checks come back as a follow-up
  (`standards/ISSUES.md`) rather than as new failures in this PR.
- `standards/` is now a docs-type project whose `ci` task checks the scripts
  parse and runs the audit.
