# Priority Issues

- TODO (agent-generated): Make the git hooks usable in cloud sandboxes (Jim, 2026-11-26: hooks were skipped with `LEFTHOOK=0` on PR #582 because of this). Today a commit that only touches a root Markdown file such as ISSUES.md, or a merge from main, makes `hooks.sh` run `mise run ci` in about every project (8 to 14 minutes), and six projects can't build in the sandbox (my_lang needs LLVM headers, controlled_chaos and starcraft need alsa, cahokia/web and cookbook/web need dioxus-cli, luggage needs utoipa-swagger-ui downloads), so the commit is refused. Fix: on a merge commit, check only the projects the branch itself changed since the merge base, not main's incoming changes; don't treat root Markdown files as touching every project; and when a project's build fails only because a system tool or download is missing, have the hook install it through the project's `ci:deps` task or report it as "not checked here, PR CI will" instead of failing. Add tests for each case.
- TODO: Port the checks the removed `standards` CLI ran into `standards/scripts/audit.sh`: NAM-002 (directory matches package name), NAM-004 (`poc_` prefix), DOC-001 to DOC-004 (README.md and CONTRIBUTING.md with a Getting Started section) and LIN-001/LIN-002 (Cargo lints). About 20 projects miss a README.md or CONTRIBUTING.md today, so add them as warnings first, or fix the projects in the same PR.
- TODO (agent-generated): Make the git hooks pass in a cloud sandbox for projects with Playwright tests (cookbook/web, cahokia/web). `npx playwright install --with-deps` in `standards/tasks/dioxus-web.toml` can't download its Chromium there, and dioxus-cli can't be installed, so agents must commit with `LEFTHOOK=0`. Options: use the sandbox's preinstalled Chromium (`/opt/pw-browsers`, via `PLAYWRIGHT_BROWSERS_PATH`) when present, or have `hooks.sh` skip just the Playwright step when the browser or `dx` is missing and say so, leaving it to PR CI.

# Backlog

## Standards (broken down)

- TODO: Create Documentation Standards - Establish documentation requirements and tooling: README.md template for new projects, API documentation standards (rustdoc), Architecture decision records (ADRs), Changelog format and maintenance.
- TODO: Implement Testing Standards - Define testing requirements and coverage goals: minimum code coverage thresholds, testing framework conventions, integration test patterns, benchmark standards.
- TODO: Dependency Management Policy - Create policies for managing dependencies: approved dependency list, version management strategy, security vulnerability scanning, license compliance checking.
- TODO: CI/CD Integration - Integrate standards enforcement into CI/CD pipelines: GitHub Actions workflows, automated code review checks, quality gates for PR merges.
- TODO: Pre-commit hooks. `mise run ci` for the changed projects (see `mise run //:affected`) should run as a part of pre-commit.
- TODO: Code Style Guidelines - Document and enforce code style conventions: naming conventions, module organization patterns, error handling patterns, async/await best practices.
- TODO: Build and Release Standards - Standardize build and release processes. Versioning strategy (semver): we use semver plus datetimes as a build identifier. We should remove any incrementing build counters.

## Browser tests on a system browser (decision 0005)

- TODO (agent-generated): Launch the Chromium and Pixel 5 Playwright projects in `cookbook/web` and `cahokia/web` on a system browser: `launchOptions.executablePath = process.env.CHROME_PATH` when set, else `channel: 'chrome'` when `CI` is set. With `CHROME_PATH` set and `CI` unset, run only those two projects. `global-setup.ts` uses the same launch options. See `standards/docs/decisions/0005-browser-testing.md`.
- TODO (agent-generated): In the `e2e` task of `standards/tasks/dioxus-web.toml` (added by PR #613), when `CI` is unset, find a Chrome (`CHROME_PATH`, `google-chrome`, `chromium`), export `CHROME_PATH` and `PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD=1` and run the tests instead of skipping; in CI install only `npx playwright install --with-deps firefox webkit`. Add each case to `standards/tests/e2e-task.test.sh`, and update `standards/docs/testing.md`, `toolchain.md` and `git-hooks.md` to say which browsers run where. Decision 0005.
- TODO (agent-generated): Move render-only checks (links, text, class names) from the cookbook/web and cahokia/web Playwright specs to Rust tests that render components with `dioxus-ssr`, keeping browser tests for interaction, accessibility and smoke UAT. Decision 0005.

# Priority Projects

# On Hold Projects
