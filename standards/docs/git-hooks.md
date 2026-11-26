# Git hooks

Local commits and pushes run the same checks PR CI runs, through
[lefthook](https://lefthook.dev). A pull request's CI is then a double check:
anything it would catch should already have failed on your machine or in your
sandbox.

## Setup

Once per clone, from anywhere in the repo:

```bash
mise run //:hooks
```

This installs lefthook through mise (`mise exec lefthook@<version>`, the
`min_version` in `lefthook.yml`) and writes `.git/hooks/pre-commit` and
`.git/hooks/pre-push`. Re-run it after `lefthook.yml` changes.

## What runs

Both hooks run `standards/scripts/hooks.sh`, which runs `mise run //:audit`
and then `mise run ci` in each affected project (installing its tools with
`mise install` first). Projects are picked by `standards/scripts/affected.sh`,
the same script PR CI uses.

| Hook | Projects | Notes |
| --- | --- | --- |
| `pre-commit` | touched by the staged and unstaged changes | Records each passing project against the commit's tree. |
| `pre-push` | touched since `origin/main` (or `AFFECTED_BASE`): the list PR CI checks | Skips projects whose `ci` already passed in a pre-commit of the same tree, so commit-then-push runs CI once. |

Every project runs even after one fails, and the hook prints the failures at
the end. `mise run fix` in a project fixes most format and lint failures.

You can run either check by hand without committing:
`bash standards/scripts/hooks.sh pre-commit` (or `pre-push`).

A change to the root `mise.toml`, `standards/scripts/` or the mise CI
workflows affects every project, so its hooks run CI in every project, just
as PR CI does.

## Cloud sandboxes

Agents work in cloud sandboxes that often block GitHub release downloads and
mise's version service. Follow these steps there; each one is safe to repeat.

1. **mise missing:** `npm install -g @jdxcode/mise` (the npm registry is
   usually reachable when mise's own installer is not).
2. **Hooks:** `mise run //:hooks`. When mise can't download lefthook, the
   setup script falls back to the same version from npm, then PyPI.
   `lefthook install` writes that binary's path into the hooks.
3. **Project tools:** when `mise install` fails in a project, `hooks.sh`
   retries it with `MISE_OFFLINE=1`, which skips the version lookups and still
   installs Rust through rustup. If that fails too, it runs `mise run ci` with
   the tools already on PATH (for example the sandbox's own `cargo`), so every
   check that can run does. To do the same by hand:
   `MISE_OFFLINE=1 mise install`, then `mise run ci`.
4. **Shallow clones:** a shallow clone has no merge base with `origin/main`,
   so every file looks changed. The pre-push hook runs
   `git fetch --unshallow origin` once when it sees that.

If a check still fails only because the sandbox lacks a tool (a "command not
found", not a failure your change caused), commit with `LEFTHOOK=0`, and say
in the PR which checks failed that way. PR CI then runs them.

## Skipping

`LEFTHOOK=0 git commit ...` (or `git push --no-verify`) skips the hooks.
Agents never skip them to get a change through, only for a tool the sandbox
can't install (see above). A human may skip them for a
work-in-progress commit, but then must run the pre-push check before opening
or updating a PR.

If lefthook itself is missing, the hook prints "Can't find lefthook in PATH"
and lets the commit through. Run `mise run //:hooks` again to fix it.
