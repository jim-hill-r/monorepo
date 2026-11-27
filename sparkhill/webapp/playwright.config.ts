import { defineConfig, devices } from '@playwright/test';

/**
 * Playwright tests for the Blue Eel web app (sparkhill/webapp).
 *
 * By default they run against `mise run dev` (dx serve on
 * http://localhost:8080), which the webServer below starts, or reuses
 * outside CI.
 *
 * With BASE_URL set (the `smoke` task after webapp_cloudflare deploys, see
 * "After deploy" in standards/docs/evals.md), they run against that site
 * instead: no dev server, and every test records its video, screenshot and
 * trace. Run the `@smoke` tests against production by hand with:
 *   BASE_URL=https://jimhillr-sparkhill-webapp.pages.dev npx playwright test --grep @smoke --project chromium
 *
 * @see https://playwright.dev/docs/test-configuration
 */
const liveURL = process.env.BASE_URL;

/* Runs against a live site say who they are in the user agent. Blue Eel
   has no analytics (SPRK-006), so nothing drops them today; it keeps the
   live runs recognisable in Cloudflare's logs, like every other project's. */
const SYNTHETIC_UA = 'jimhillr-evals/0.1 (synthetic check)';

/** A device preset, with the synthetic user agent added on live runs. */
function device(name: keyof typeof devices) {
  const preset = devices[name];
  return liveURL ? { ...preset, userAgent: `${preset.userAgent} ${SYNTHETIC_UA}` } : preset;
}

export default defineConfig({
  testDir: './e2e',

  /* Wait for the dev server's first build before running tests. A live
     site is already built. */
  globalSetup: liveURL ? undefined : './global-setup.ts',

  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 1 : 0,

  /* The unoptimised dev build loads slowly on CI runners, and the level
     tests trace several letters. */
  timeout: process.env.CI ? 90 * 1000 : 60 * 1000,
  workers: process.env.CI ? 2 : 4,
  reporter: 'list',

  use: {
    baseURL: liveURL ?? 'http://localhost:8080',
    /* A live (smoke) run records every test as evidence
       (standards/docs/plans/2026-11-deploy-feedback.md, phase 3a). */
    trace: liveURL ? 'on' : 'on-first-retry',
    screenshot: liveURL ? 'on' : 'off',
    video: liveURL ? 'on' : 'off',
  },

  projects: [
    { name: 'chromium', use: { ...device('Desktop Chrome') } },
    { name: 'Mobile Chrome', use: { ...device('Pixel 5') } },
  ],

  /* Start the dev server, unless the tests run against a live site. */
  webServer: liveURL
    ? undefined
    : {
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
