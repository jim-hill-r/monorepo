import { defineConfig, devices } from '@playwright/test';

/**
 * A device with `jimhillr-evals` added to its user agent. Every test run
 * says it is synthetic, so the PostHog snippet doesn't load and the
 * telemetry Worker drops its events (standards/docs/evals.md, EVL-005).
 */
const synthetic = (device: (typeof devices)[string]) => ({
  ...device,
  userAgent: `${device.userAgent} jimhillr-evals`,
});

/**
 * Playwright configuration for Cahokia web application tests.
 *
 * By default the tests start the Dioxus dev server (`mise run dev`) and run
 * against http://localhost:8080.
 *
 * With BASE_URL set (the `smoke` task of cahokia/cloudflare after a deploy,
 * see standards/docs/evals.md), they run against that site instead: no dev
 * server, and every test records its video, screenshot and trace. Only
 * signed-out tests are tagged `@smoke`, since production has no test
 * account. Run them against production by hand with:
 *   BASE_URL=https://jimhillr-cahokia.pages.dev npx playwright test --grep @smoke --project chromium
 *
 * @see https://playwright.dev/docs/test-configuration
 */
const liveURL = process.env.BASE_URL;

/**
 * The sandbox Chromium fallback (standards/tasks/dioxus-web.toml) has no
 * Firefox or WebKit, so with PLAYWRIGHT_CHROMIUM_PATH set only the Chromium
 * projects run; CI runs them all.
 */
const onlyChromiumInSandbox = (project: { use?: { defaultBrowserType?: string } }) =>
  !process.env.PLAYWRIGHT_CHROMIUM_PATH ||
  (project.use?.defaultBrowserType ?? 'chromium') === 'chromium';

export default defineConfig({
  testDir: './tests',

  /* Wait for the dev server's first build before running tests. A live
     site is already built. */
  globalSetup: liveURL ? undefined : './global-setup.ts',
  
  /* Run tests in files in parallel */
  fullyParallel: true,
  
  /* Fail the build on CI if you accidentally left test.only in the source code. */
  forbidOnly: !!process.env.CI,
  
  /* Retry on CI only */
  retries: process.env.CI ? 1 : 0,

  /* The unoptimised dev build loads slowly on CI runners; tests that visit
     several pages need more than the default 30s there. */
  timeout: process.env.CI ? 60 * 1000 : 30 * 1000,
  
  /* Use a limited number of workers on CI to balance speed and stability. */
  workers: process.env.CI ? 2 : 4,

  /* Reporter to use. */
  reporter: 'list',
  
  /* Shared settings for all the projects below. */
  use: {
    /* Base URL to use in actions like `await page.goto('/')`. */
    baseURL: liveURL ?? 'http://localhost:8080',

    /* Collect trace when retrying the failed test. A live (smoke) run
       records the trace, screenshot and video of every test as evidence
       (standards/docs/plans/2026-11-deploy-feedback.md, phase 3a). */
    trace: liveURL ? 'on' : 'on-first-retry',

    screenshot: liveURL ? 'on' : 'off',

    video: liveURL ? 'on' : 'off',
    // A sandbox whose Chromium doesn't match this Playwright can point at it
    // (the e2e task in standards/tasks/dioxus-web.toml sets it).
    launchOptions: { executablePath: process.env.PLAYWRIGHT_CHROMIUM_PATH || undefined },
  },

  /* Configure projects for major browsers */
  projects: [
    {
      name: 'chromium',
      use: { ...synthetic(devices['Desktop Chrome']) },
    },

    {
      name: 'firefox',
      use: { ...synthetic(devices['Desktop Firefox']) },
    },

    {
      name: 'webkit',
      use: { ...synthetic(devices['Desktop Safari']) },
    },

    /* Test against mobile viewports. */
    {
      name: 'Mobile Chrome',
      use: { ...synthetic(devices['Pixel 5']) },
    },
    {
      name: 'Mobile Safari',
      use: { ...synthetic(devices['iPhone 12']) },
    },
  ].filter(onlyChromiumInSandbox),

  /* Run your local dev server before starting the tests, unless they run
     against a live site. */
  webServer: liveURL ? undefined : {
    // `mise run dev` is the project's standard dev task (dx serve on port 8080).
    command: 'mise run dev',
    url: 'http://localhost:8080',
    reuseExistingServer: !process.env.CI,
    timeout: 600 * 1000, // first dx build in CI is slow
    cwd: __dirname,
    
    stdout: 'pipe',

    stderr: 'pipe',
  },
});
