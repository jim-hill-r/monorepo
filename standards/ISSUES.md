# Priority Issues

- TODO: Port the checks the removed `standards` CLI ran into `standards/scripts/audit.sh`: NAM-002 (directory matches package name), NAM-004 (`poc_` prefix), DOC-001 to DOC-004 (README.md and CONTRIBUTING.md with a Getting Started section) and LIN-001/LIN-002 (Cargo lints). About 20 projects miss a README.md or CONTRIBUTING.md today, so add them as warnings first, or fix the projects in the same PR.

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
