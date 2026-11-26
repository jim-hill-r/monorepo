import { defineConfig, devices } from '@playwright/test';

/**
 * Playwright configuration for Pane's web app.
 *
 * `npm test` runs every test in Chromium against `mise run dev`, which builds
 * the repo snapshot and serves the app on http://localhost:8080.
 *
 * `npm run smoke` runs only the `@smoke` tests and records video, screenshots
 * and a trace of each, for the smoke UAT report in the deploy feedback plan
 * (standards/docs/plans/2026-11-deploy-feedback.md). Set BASE_URL to run
 * against a deployed site, such as a preview or production; the local server
 * is then not started.
 *
 * @see https://playwright.dev/docs/test-configuration
 */
const baseURL = process.env.BASE_URL || 'http://localhost:8080';
const local = !process.env.BASE_URL;

export default defineConfig({
  testDir: './tests',

  /* Wait for the dev server's first build before running tests. */
  globalSetup: local ? './global-setup.ts' : undefined,

  fullyParallel: true,

  /* Fail the build on CI if you accidentally left test.only in the source code. */
  forbidOnly: !!process.env.CI,

  retries: process.env.CI ? 1 : 0,

  /* The unoptimised dev build loads slowly on CI runners. */
  timeout: process.env.CI ? 60 * 1000 : 30 * 1000,

  workers: process.env.CI ? 2 : 4,

  reporter: process.env.CI ? [['list'], ['html', { open: 'never' }]] : 'list',

  use: {
    baseURL,
    trace: 'on-first-retry',
    screenshot: 'off',
    video: 'off',
  },

  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },
    {
      name: 'smoke',
      grep: /@smoke/,
      use: {
        ...devices['Desktop Chrome'],
        trace: 'on',
        screenshot: 'on',
        video: 'on',
      },
    },
  ],

  webServer: local
    ? {
        // pane/web's `dev` task builds public/data/snapshot.json, then runs dx serve on 8080.
        command: 'mise run dev',
        url: 'http://localhost:8080',
        reuseExistingServer: !process.env.CI,
        timeout: 600 * 1000, // first dx build in CI is slow
        cwd: __dirname,
        stdout: 'pipe',
        stderr: 'pipe',
      }
    : undefined,
});
