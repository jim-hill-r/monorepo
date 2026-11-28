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

Both hooks run `standards/scripts/hooks.sh`, which runs `mise run //:audit`,
scans for secrets with gitleaks (SEC-001, `standards/scripts/secrets-check.sh`:
the staged diff in pre-commit, the commits being pushed in pre-push; see
[security.md](./security.md)), runs cargo-deny and `npm audit` for any
lockfile the change touches (SEC-003 and SEC-004, warnings only,
`standards/scripts/deps-check.sh`), and then `mise run ci` in each affected project (installing its tools with
`mise install` first). Projects are picked by
`standards/scripts/affected.sh --changed-only`, the same list PR CI checks.

| Hook | Projects | Notes |
| --- | --- | --- |
| `pre-commit` | touched by the staged and unstaged changes; when committing a merge, the projects the branch changes compared with the branch merged in (`MERGE_HEAD`), so merging `main` doesn't check `main`'s changes | Records each passing project against the commit's tree. |
| `pre-push` | touched since `origin/main` (or `AFFECTED_BASE`), which it fetches first so a stale copy doesn't make everything merged since look changed | Skips projects whose `ci` already passed in a pre-commit of the same tree, so commit-then-push runs CI once. |

Every project runs even after one fails, and the hook prints the failures at
the end. `mise run fix` in a project fixes most format and lint failures.

You can run either check by hand without committing:
`bash standards/scripts/hooks.sh pre-commit` (or `pre-push`).

A change to the root `mise.toml`, `standards/scripts/` or the mise CI
workflows doesn't check every project, in the hooks or in PR CI
(`--changed-only`). It checks only the projects that test those files,
`standards` (its own scripts and `standards/tests/`) and `jimhillr` (the
workflow tests, through the `# affected-by:` lines in its `mise.toml`), plus,
for a script, the projects whose `mise.toml` or included
`standards/tasks/*.toml` file names it. Pushes to `main` and the nightly runs
still check every project. A Markdown file outside any project, such as
`AGENTS.md`, checks only `jimhillr`.

Before running a project's tools, the hook clears git's own variables
(`GIT_DIR`, `GIT_INDEX_FILE` and the rest of `git rev-parse --local-env-vars`),
so tests that run git in their own temporary repos aren't pointed at this one.

`standards/tests/hooks.test.sh` tests these cases.

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
5. **Playwright tests:** the `e2e` task of Dioxus web apps
   (`standards/tasks/dioxus-web.toml`) needs npm packages, `dx` and a
   Chromium. Get them like this:
   - **dx:** compiling `cargo:dioxus-cli` through mise fails there, because
     mise first installs `cargo-binstall` and can't verify its attestation
     (sigstore's TUF host is blocked). Download the prebuilt binary instead,
     then tell mise not to install either tool. Use the release that matches
     the project's `"cargo:dioxus-cli"` in its `mise.toml` (0.6.3, the latest
     0.6.x, for `sparkhill/webapp`): another minor version
     builds the app differently and fails its tests. dx 0.7 serves `public/`
     at the site root, for example, so a 0.6 app's
     `/assets/manifest.webmanifest` answers with `index.html`. 0.6 releases
     name the archive with the version (`...-gnu-v0.6.3.tar.gz`) and 0.7
     releases don't, so the snippet tries both:

     ```sh
     v=0.7.3
     d=$(mktemp -d)
     u="https://github.com/DioxusLabs/dioxus/releases/download/v$v/dx-x86_64-unknown-linux-gnu"
     { curl -fsSL "$u-v$v.tar.gz" || curl -fsSL "$u.tar.gz"; } | tar xz -C "$d"
     install -m755 "$d/dx" ~/.cargo/bin/dx
     dx --version  # check it says $v
     export MISE_DISABLE_TOOLS=cargo-binstall,cargo:dioxus-cli
     ```

     After switching dx versions, delete the project's `target/dx` so no
     files from the other version's build are served.

   - **Chromium:** when Playwright can't download its own browser build, the
     task uses the Chromium in `PLAYWRIGHT_CHROMIUM_PATH`, else the cloud
     sandbox's own at `/opt/pw-browsers/chromium`. Each app's
     `playwright.config.ts` launches `PLAYWRIGHT_CHROMIUM_PATH` when it is
     set, and then runs only its Chromium projects (no Firefox or WebKit).
     An app opts in by reading `PLAYWRIGHT_CHROMIUM_PATH` in that config
     (pane, cahokia, cookbook, bear_down and sparkhill/webapp do); other
     apps still skip.
     The task also skips tests tagged `@network`, which need a service the
     sandbox can't reach (cahokia's Auth0 sign-in); tag any such test that
     way. PR CI runs every browser and every test.

   With nothing usable, outside CI (when `CI` is unset) the task skips the
   tests with "Playwright tests not checked here; PR CI runs them" instead of
   failing, so the hooks pass. In CI the same gap fails the task. Say in the
   PR whether the Playwright tests ran only in PR CI.

6. **gitleaks:** it is a GitHub release download, so a sandbox often can't
   get it. The secret scan then prints `warning[SEC-001]` and the hook
   carries on; PR CI's **Secret scan** job runs it. Say so in the PR, or
   install it through the Go module proxy first:
   `go install github.com/zricethezav/gitleaks/v8@v8.30.1` (then put
   `~/go/bin` on `PATH`).

7. **System libraries and downloads:** a project whose build needs something
   a sandbox can't install defines a `ci:check-tools` task that fails,
   naming what is missing. The hook runs it first and, when it fails, skips
   the project with "not checked here, PR CI checks it" instead of failing
   the commit, and lists the skipped projects at the end. Say in the PR which
   projects were skipped. Today that is:

   | Project | Needs | Defined in |
   | --- | --- | --- |
   | `controlled_chaos`, `starcraft` | alsa and libudev (`pkg-config`) | `standards/tasks/bevy-game.toml` |
   | `my_lang` | LLVM 18 headers | `my_lang/mise.toml` |
   | `luggage` | the Swagger UI zip from GitHub, or `SWAGGER_UI_DOWNLOAD_URL` | `luggage/mise.toml` |
   | `infra` | registry.opentofu.org, for `tofu validate` | `infra/mise.toml` |
   | `tofu_state` | `wrangler` and `tofu`, for `tests/e2e.sh` | `tofu_state/mise.toml` |
   | `pane/cloudflare` | `wrangler` and `worker-build`, for `tests/e2e.rs` | `pane/cloudflare/mise.toml` |

   `cahokia/web` and `cookbook/web` need none: without `dx` their Rust
   checks still run and the Playwright step skips itself (step 5). Add a
   `ci:check-tools` task to any other project with the same problem, rather
   than skipping the hooks.

With these steps an agent needs no `LEFTHOOK=0` for a missing tool. If a
check still fails only because the sandbox lacks something (a "command not
found", not a failure your change caused), add the missing `ci:check-tools`
check in the same PR. As a last resort, commit with `LEFTHOOK=0` without
asking first, and say in the PR which checks didn't run. PR CI then runs
them.

## Skipping

`LEFTHOOK=0 git commit ...` (or `git push --no-verify`) skips the hooks.
Agents never skip them to get a change through, only for a tool the sandbox
can't install that no `ci:check-tools` task covers yet (see above). A human may skip them for a
work-in-progress commit, but then must run the pre-push check before opening
or updating a PR.

If lefthook itself is missing, the hook prints "Can't find lefthook in PATH"
and lets the commit through. Run `mise run //:hooks` again to fix it.
