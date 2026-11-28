# Standards

[![Eval coverage](https://raw.githubusercontent.com/jimhillr/monorepo/evals-results/badges/standards.svg)](https://github.com/jimhillr/monorepo/blob/evals-results/verdicts.json)

The monorepo's standards, the shared mise tasks every project uses, and the
scripts that check the repo.

## What is here

| Path | Contents |
| --- | --- |
| [`docs/`](./docs/) | The standards: Rust, testing, documentation, naming, issue management, pull requests, workflow conventions, toolchain, and [mise](./docs/mise.md). |
| [`tasks/`](./tasks/README.md) | One shared task file per project type (`rust`, `dioxus-web`, `perseus-web`, `bevy-game`, `cloudflare-pages`, `docs`). Every project's `mise.toml` includes exactly one. |
| `scripts/audit.sh` | `mise run //:audit`: checks every project has a valid `mise.toml`, the task files define the standard tasks, `config_roots` lists every project, and names are snake_case. Runs on every pull request. |
| `scripts/affected.sh` | `mise run //:affected`: lists the projects a change touches, for CI. |

## Standards documentation

- [Build and CI](./docs/build-and-ci.md)
- [Documentation](./docs/documentation.md)
- [Issue management](./docs/issue-management.md)
- [mise](./docs/mise.md)
- [Naming](./docs/naming.md)
- [Pull requests](./docs/pull-requests.md)
- [Rust](./docs/rust.md)
- [Testing](./docs/testing.md)
- [Toolchain](./docs/toolchain.md)
- [Workflow conventions](./docs/workflow-conventions.md)

## Checks

`mise run //:audit` is the only automated standards check. The Rust
`standards` CLI that used to audit naming (NAM-002, NAM-004), documentation
(DOC-001 to DOC-004) and lint (LIN-001, LIN-002) rules and write ISSUES.md
TODOs was removed with Cast; porting those checks to `audit.sh` is tracked
in [docs/issues/](docs/issues/).
