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
 * Playwright tests for the Blue Eel web app (sparkhill/webapp).
 *
 * They run against `mise run dev` (dx serve on http://localhost:8080), which
 * the webServer below starts, or reuses outside CI.
 *
 * @see https://playwright.dev/docs/test-configuration
 */
export default defineConfig({
  testDir: './e2e',

  /* Wait for the dev server's first build before running tests. */
  globalSetup: './global-setup.ts',

  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 1 : 0,

  /* The unoptimised dev build loads slowly on CI runners, and the level
     tests trace several letters. */
  timeout: process.env.CI ? 90 * 1000 : 60 * 1000,
  workers: process.env.CI ? 2 : 4,
  reporter: 'list',

  use: {
    baseURL: 'http://localhost:8080',
    trace: 'on-first-retry',
    screenshot: 'off',
    video: 'off',
  },

  projects: [
    { name: 'chromium', use: { ...synthetic(devices['Desktop Chrome']) } },
    { name: 'Mobile Chrome', use: { ...synthetic(devices['Pixel 5']) } },
  ],

  webServer: {
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
