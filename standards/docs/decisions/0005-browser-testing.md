# 0005. Keep Playwright, but run it on a browser we already have

- Status: Proposed (needs Jim's sign-off)
- Date: 2026-11-26
- Asked by: Jim ("The downloading of chrome has always been a challenge. It
  depends on node which I don't really like.")

## Context

Two web apps have browser end-to-end tests today: `cookbook/web` (10 specs)
and `cahokia/web` (5 specs), about 2,100 lines of TypeScript on Playwright
1.57. `standards/tasks/dioxus-web.toml` runs them with `npm ci`,
`npx playwright install --with-deps` and `npm test`. Each config runs five
projects: Chromium, Firefox, WebKit, Pixel 5 and iPhone 12.

What hurts:

1. **The browser download.** `npx playwright install` fetches Playwright's
   own patched builds of Chromium, Firefox and WebKit. Cloud sandboxes can't
   reach that download, so PR #613 makes the e2e step skip itself outside CI.
   The builds also exist only for the operating systems Playwright supports,
   which is why CI is pinned to `ubuntu-24.04`.
2. **Node.** The tests, the runner and its config need Node and npm, in a
   repo that is otherwise Rust.

What the tests and plans use, which any replacement must cover:

- Locators with auto-waiting and web-first assertions (`toBeVisible`,
  `toHaveURL`, `toHaveClass` and similar, about 200 uses).
- Accessibility checks with `@axe-core/playwright`.
- `page.clock` (fake time for the leap-year test), `page.route` (network
  stubs for the auth tests), `page.evaluate`, mobile device emulation.
- The `webServer` and `globalSetup` that start `mise run dev` and wait for
  the first `dx` build.
- Decision 0001: recorded smoke UAT on every PR preview and after each
  deploy, **with screenshots, video, a trace and an HTML report** published
  to Cloudflare Pages, as the default evidence on PRs.

### What was checked in this sandbox

- The sandbox has a browser: `/usr/bin/google-chrome` is Chrome for Testing
  158. Playwright 1.57 (`playwright-core`, installed with
  `PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD=1`) launched it with
  `executablePath: '/usr/bin/google-chrome'`, loaded a page and took a
  screenshot. **No download was needed.** PR #613's note that the sandbox's
  Chromium is "a different build from the one the projects pin" is true, but
  Playwright drives it anyway.
- The sandbox's `chromedriver` is version 147 and refused that same browser:
  "This version of ChromeDriver only supports Chrome version 147. Current
  browser version is 158." This is the main cost of every WebDriver option
  below: the driver and the browser have to be kept in step.

## Options

| | Node-free | Browsers | Video, trace, HTML report (0001) | axe, clock, network stubs | Browser download | Migration |
| --- | --- | --- | --- | --- | --- | --- |
| A. Playwright as it is | No | Chromium, Firefox, WebKit | Yes | Yes | Playwright's own builds, every run | None |
| **B. Playwright on a system browser** | No | System Chrome everywhere; Firefox and WebKit in CI only | Yes | Yes | None for Chromium | Small: config and task changes |
| C. fantoccini or thirtyfour (Rust, WebDriver) | Yes | Chrome, Firefox (no WebKit on Linux) | Screenshots only | axe by injecting its JS; no clock; stubs only through BiDi | Browser plus a matching driver | Large: rewrite 2,100 lines, build waiting, reporting and server start |
| D. chromiumoxide or headless_chrome (Rust, CDP) | Yes | Chromium only | Screenshots; video only by stitching screencast frames | axe by injection; network and emulation through raw CDP | System Chrome, or the crate's own downloader | Large, and loses Firefox and WebKit |
| E. WebDriver BiDi from Rust | Yes | Chrome, Firefox (Safari not yet) | Screenshots only | Network intercept yes, the rest by hand | Browser plus driver (or Firefox's built-in BiDi) | Largest: no mature Rust client that we found |

Notes on the options:

- **A. Keep as it is.** Best tooling and the only option that meets 0001
  today, but it keeps both pains.
- **B. System browser.** Playwright can launch any recent Chrome or Chromium
  with `executablePath`, or the installed Google Chrome with
  `channel: 'chrome'`. GitHub's `ubuntu-24.04` runner image ships Google
  Chrome, so CI can skip the Chromium download and fetch only Firefox and
  WebKit. Sandboxes and local runs use whatever Chrome is there. Node stays.
- **C. fantoccini / thirtyfour.** Both are maintained async Rust clients for
  W3C WebDriver and would let the tests be `cargo test`. But they move the
  download problem rather than remove it (a chromedriver or geckodriver
  matched to each browser, see the 147 vs 158 failure above), there is no
  WebKit on Linux, no auto-waiting assertions, no fake clock, and nothing like
  Playwright's trace, video or report, which 0001 depends on.
- **D. chromiumoxide / headless_chrome.** Talk to Chrome directly over the
  DevTools Protocol, so no driver. They can use the system Chrome. Chromium
  only, smaller communities with irregular releases, and the same missing
  test runner, waiting and reporting as C.
- **E. WebDriver BiDi.** The cross-browser standard that Puppeteer now uses
  for Firefox and Chrome. It is the long-term answer to "Playwright without
  Node", but Safari doesn't support it yet and Rust clients are early. Worth
  watching, not adopting.
- **Not a replacement but worth doing anyway:** much of what the specs check
  (which links a page renders, class names, text) could be a Rust test that
  renders the component with `dioxus-ssr`, with no browser at all. Moving
  those checks shrinks the browser suite to real interaction and smoke tests.

Vendors and cost: every option is free and open source and adds no service
vendor. Playwright's browser builds come from Microsoft's CDN, Chrome for
Testing from Google's; neither is an account or a paid plan.

## Decision (proposed)

Choose **B**: keep Playwright and stop depending on its Chromium download.

- Chromium projects (Desktop Chrome, Pixel 5) launch a system browser: the
  path in `CHROME_PATH` when it is set, else `channel: 'chrome'` in CI.
- Outside CI (sandboxes, hooks, local runs) the e2e step runs only the
  Chromium projects when it finds a system Chrome, instead of skipping. It
  still skips, as in PR #613, when there is no browser or no `dx`.
- CI keeps all five projects. It installs only Firefox and WebKit with
  `npx playwright install --with-deps firefox webkit`.
- Node stays, limited to the e2e tests (as `standards/docs/typescript.md`
  already says).
- Revisit when a Rust WebDriver BiDi client can do auto-waiting, network
  stubs and screenshots against Chrome and Firefox, or if Playwright's Firefox
  or WebKit downloads start failing in CI. A good first test then is porting
  `cookbook/web/tests/leap-year-today-link.spec.ts` (40 lines).

## Consequences

- Agents get real browser test results in sandboxes instead of "PR CI runs
  them", as long as `dx` can be installed there (it can't yet, so this needs
  the `dx` half of the sandbox hooks TODO too).
- CI downloads two browsers instead of three.
- Chromium tests run on whatever Chrome the machine has, so a Chrome update
  can change results without a commit. Playwright supports the stable Chrome
  channel, and CI still pins the Playwright version, so this risk is small.
- The `ubuntu-24.04` pin stays until Playwright ships Firefox and WebKit
  builds for 26.04.
- Node is not removed. Jim's dislike of it is noted, and the revisit trigger
  above is how it could go later.

## Migration TODOs

These are in `standards/ISSUES.md` under "Browser tests on a system browser".
They land with this record, so they only exist once Jim accepts it.

1. In `cookbook/web/playwright.config.ts` and `cahokia/web/playwright.config.ts`,
   launch the Chromium and Pixel 5 projects with
   `launchOptions.executablePath = process.env.CHROME_PATH` when set, else
   `channel: 'chrome'` when `CI` is set. With `CHROME_PATH` set and `CI`
   unset, only those two projects run. Have `global-setup.ts` use the same
   launch options.
2. In the `e2e` task in `standards/tasks/dioxus-web.toml` (from PR #613):
   when `CI` is unset, look for a Chrome (`CHROME_PATH`, `google-chrome`,
   `chromium`), export `CHROME_PATH` and `PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD=1`,
   and run the tests instead of skipping. In CI, install only
   `firefox webkit`. Extend `standards/tests/e2e-task.test.sh` for each case.
3. Update `standards/docs/testing.md`, `toolchain.md` and `git-hooks.md` to
   say which browsers run where.
4. Optional, per project: move render-only checks (links, text, classes) from
   the specs to Rust tests with `dioxus-ssr`, keeping the browser tests for
   interaction, accessibility and smoke UAT.
